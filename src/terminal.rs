use std::sync::atomic::{AtomicBool, Ordering};

use crate::backend::{Backend, TerminalEvent, Tty};
use crate::buffer::{Buffer, Cell, Grid};
use crate::{Color, Element, Intensity, Output, Style, Vec2};

static TERMINAL_EXISTS: AtomicBool = AtomicBool::new(false);

/// A terminal which can draw elements to a backend.
///
/// Only one terminal may exist at once; attempting to create more than one at once will panic.
#[derive(Debug)]
pub struct Terminal<B: Backend> {
    backend: B,
    /// Whether the backend has been `.reset()`.
    backend_reset: bool,
    /// Holds the previous frame to diff against.
    old_buffer: Buffer,
    /// Is always a clear buffer, kept around to avoid cloning the buffer each draw.
    buffer: Buffer,
    /// The current position of the cursor.
    ///
    /// This is the actual position of the cursor, unlike `old_buffer.cursor` which stores the
    /// position of the cursor after drawing.
    cursor_pos: Vec2<u16>,
    /// The current style being written with.
    style: Style,
}

impl<B: Backend> Terminal<B> {
    /// Create a new terminal with the given backend.
    ///
    /// # Panics
    ///
    /// Panics if a terminal already exists.
    pub fn new(config: B::Config) -> Result<Self, B::Error> {
        if TERMINAL_EXISTS.swap(true, Ordering::Acquire) {
            panic!("Terminal already exists!");
        }

        let mut backend = B::new(config, Tty::new())?;

        backend.hide_cursor()?;
        backend.set_cursor_pos(Vec2::default())?;
        backend.set_foreground(Color::Default)?;
        backend.set_background(Color::Default)?;
        backend.set_intensity(Intensity::Normal)?;
        backend.set_italic(false)?;
        backend.set_underlined(false)?;
        backend.set_blinking(false)?;
        backend.set_crossed_out(false)?;

        let buffer = Buffer::from(Grid::new(backend.size()?));

        Ok(Terminal {
            backend,
            backend_reset: false,
            old_buffer: buffer.clone(),
            buffer,
            cursor_pos: Vec2::default(),
            style: Style::default(),
        })
    }

    /// Draw an element to the terminal and wait for an event.
    ///
    /// The future produced by this function can be dropped, in which case the terminal will stop
    /// reading events.
    pub async fn draw<Event, E: Element<Event>>(&mut self, element: E) -> Result<Event, B::Error> {
        loop {
            element.draw(&mut self.buffer);

            self.diff()?;
            self.backend.flush()?;

            self.old_buffer.clear(Color::Default);
            std::mem::swap(&mut self.old_buffer, &mut self.buffer);

            match self.backend.read_event().await? {
                TerminalEvent::Input(input) => {
                    if let Some(event) = element.handle(input) {
                        return Ok(event);
                    }
                }
                TerminalEvent::Resize(size) => {
                    self.buffer.grid.resize(size);
                    self.old_buffer.grid.resize(size);
                }
            }
        }
    }

    /// Diffs `old_buffer` and `new_buffer` and draws them to the backend.
    fn diff(&mut self) -> Result<(), B::Error> {
        if self.old_buffer.title != self.buffer.title {
            self.backend.set_title(&self.buffer.title)?;
        }

        for (y, (old_line, new_line)) in self
            .old_buffer
            .grid
            .lines()
            .iter()
            .zip(self.buffer.grid.lines())
            .enumerate()
        {
            for (x, (old_cell, new_cell)) in
                old_line.cells().iter().zip(new_line.cells()).enumerate()
            {
                if new_cell == old_cell {
                    continue;
                }

                let pos = Vec2::new(x as u16, y as u16);

                let (new_contents, &new_contents_double, new_style) = match new_cell {
                    Cell::Char {
                        contents,
                        double,
                        style,
                    } => (contents, double, style),
                    Cell::Continuation => continue,
                };

                macro_rules! diff_styles {
                    ($($(.$path:ident)+ => $set_style:ident,)*) => {
                        $(
                            if self.style$(.$path)+ != new_style$(.$path)+ {
                                self.backend.$set_style(new_style$(.$path)+)?;
                            }
                        )*
                    }
                }
                diff_styles! {
                    .foreground => set_foreground,
                    .background => set_background,
                    .attributes.intensity => set_intensity,
                    .attributes.italic => set_italic,
                    .attributes.underlined => set_underlined,
                    .attributes.blinking => set_blinking,
                    .attributes.crossed_out => set_crossed_out,
                }

                if self.cursor_pos != pos {
                    self.backend.set_cursor_pos(pos)?;
                }

                self.backend.write(&new_contents)?;

                self.style = *new_style;

                let x = pos.x + if new_contents_double { 2 } else { 1 };
                let grid_width = self.buffer.grid.width();

                self.cursor_pos = Vec2 {
                    x: x % grid_width,
                    y: pos.y + x / grid_width,
                };
            }

            // Some terminals use the background color of the cursor to fill in space created by a
            // resize, so reset it.
            self.backend.set_background(Color::Default)?;
            self.style.background = Color::Default;
        }

        if let Some(new_cursor) = self.buffer.cursor {
            if self.old_buffer.cursor.is_none() {
                self.backend.show_cursor()?;
            }

            if self
                .old_buffer
                .cursor
                .map_or(true, |c| c.shape != new_cursor.shape)
            {
                self.backend.set_cursor_shape(new_cursor.shape)?;
            }
            if self
                .old_buffer
                .cursor
                .map_or(true, |c| c.blinking != new_cursor.blinking)
            {
                self.backend.set_cursor_blinking(new_cursor.blinking)?;
            }
            if self.cursor_pos != new_cursor.pos {
                self.backend.set_cursor_pos(new_cursor.pos)?;
            }
        } else if self.old_buffer.cursor.is_some() {
            self.backend.hide_cursor()?;
        }

        Ok(())
    }

    /// Get a reference to the terminal's backend.
    #[must_use]
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// Get a mutable reference to the terminal's backend.
    #[must_use]
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    /// Clean up the terminal.
    ///
    /// This will be called in the destructor too, but use this if you want to handle errors
    /// instead of ignoring them.
    pub fn cleanup(mut self) -> Result<(), B::Error> {
        self.backend.reset()?;
        self.backend_reset = true;

        Ok(())
    }
}

impl<B: Backend> Drop for Terminal<B> {
    fn drop(&mut self) {
        if !self.backend_reset {
            let _ = self.backend.reset();
        }

        TERMINAL_EXISTS.store(false, Ordering::Release);
    }
}

#[cfg(test)]
#[test]
fn test_diff_grid() {
    use crate::Operation;
    use crate::{Attributes, Intensity};

    let mut old_grid = Grid::new(Vec2::new(16, 8));
    old_grid.write(Vec2::new(2, 5), &"Hello World!", Style::default());
    old_grid.write(Vec2::new(3, 6), &"😃", Style::default());

    let mut new_grid = old_grid.clone();

    let mut style = Style::new(
        Color::Red,
        Color::Blue,
        Attributes {
            intensity: Intensity::Bold,
            underlined: true,
            ..Default::default()
        },
    );

    new_grid.write(Vec2::new(15, 2), &"abcd", style);
    style.foreground = Color::Green;
    new_grid.write(Vec2::new(1, 5), &"foo", style);
    new_grid.write(Vec2::new(4, 6), &"😃", style);

    let mut backend = crate::backend::Dummy::new(old_grid.size());
    backend.buffer.grid = old_grid.clone();

    let mut terminal: Terminal<crate::backend::Dummy> = Terminal::new(backend).unwrap();
    terminal.backend_mut().operations.clear();
    terminal.old_buffer = Buffer::from(old_grid);
    terminal.buffer = Buffer::from(new_grid.clone());
    terminal.diff().unwrap();

    assert_eq!(terminal.backend().buffer.grid, new_grid);

    assert_eq!(
        terminal.backend().operations,
        &[
            Operation::SetForeground(Color::Red),
            Operation::SetBackground(Color::Blue),
            Operation::SetIntensity(Intensity::Bold),
            Operation::SetUnderlined(true),
            Operation::SetCursorPos(Vec2::new(15, 2)),
            Operation::Write("a".to_owned()),
            Operation::SetForeground(Color::Green),
            Operation::SetCursorPos(Vec2::new(1, 5)),
            Operation::Write("f".to_owned()),
            Operation::Write("o".to_owned()),
            Operation::Write("o".to_owned()),
            Operation::SetForeground(Color::Default),
            Operation::SetBackground(Color::Default),
            Operation::SetIntensity(Intensity::Normal),
            Operation::SetUnderlined(false),
            Operation::SetCursorPos(Vec2::new(3, 6)),
            Operation::Write(" ".to_owned()),
            Operation::SetForeground(Color::Green),
            Operation::SetBackground(Color::Blue),
            Operation::SetIntensity(Intensity::Bold),
            Operation::SetUnderlined(true),
            Operation::Write("😃".to_owned()),
        ],
    );
}
