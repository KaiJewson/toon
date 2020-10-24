use std::cmp::min;
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::io::{self, IoSliceMut, Read};
use std::sync::atomic::{AtomicBool, Ordering};

use os_pipe::PipeReader;

use crate::backend::{Backend, Bound, ReadEvents, TerminalEvent, Tty};
use crate::buffer::{Buffer, Cell, Grid};
use crate::{Color, Element, Input, Intensity, Output, Style, Vec2};

static TERMINAL_EXISTS: AtomicBool = AtomicBool::new(false);

/// A terminal which can draw elements to a backend.
///
/// For backends that aren't dummies, only one terminal may exist at once; attempting to
/// create more than one at once will panic.
///
/// Terminals automatically capture all standard output and standard error while they are alive,
/// and print it when they are dropped. However, you can also take ownership of it via the
/// [`take_captured`](#method.take_captured) method.
#[derive(Debug)]
pub struct Terminal<B: Backend> {
    /// Only `None` during destruction of the type.
    backend: Option<B::Bound>,
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
    /// The captured stdout and stderr.
    captured: Option<PipeReader>,
}

impl<B: Backend> Terminal<B> {
    /// Create a new terminal with the given backend.
    ///
    /// # Panics
    ///
    /// Panics if the backend is not a dummy and a terminal already exists.
    ///
    /// # Errors
    ///
    /// Fails if setting up the terminal fails.
    pub fn new(backend: B) -> Result<Self, Error<B::Error>> {
        if !B::is_dummy() && TERMINAL_EXISTS.swap(true, Ordering::Acquire) {
            panic!("Terminal already exists!");
        }

        let (tty, captured) = if B::is_dummy() {
            (Tty::dummy(), None)
        } else {
            let (tty, captured) = Tty::new().map_err(Error::Io)?;
            (tty, Some(captured))
        };

        let mut backend = backend.bind(tty)?;

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

        Ok(Self {
            backend: Some(backend),
            old_buffer: buffer.clone(),
            buffer,
            cursor_pos: Vec2::default(),
            style: Style::default(),
            captured,
        })
    }

    /// Draw an element to the terminal and wait for an event. If multiple events occur they will
    /// all be returned, but this function will never return an empty vector.
    ///
    /// The future produced by this function can be dropped, in which case the terminal will stop
    /// reading input.
    ///
    /// # Errors
    ///
    /// Fails when drawing to the backend fails.
    pub async fn draw<E: Element>(
        &mut self,
        element: E,
    ) -> Result<Vec<E::Event>, Error<B::Error>> {
        loop {
            element.draw(&mut self.buffer);

            self.diff()?;
            self.backend_mut().flush()?;

            Element::draw(&crate::fill::<_, ()>(Color::Default), &mut self.old_buffer);
            std::mem::swap(&mut self.old_buffer, &mut self.buffer);

            loop {
                match self.backend_mut().read_event().await? {
                    TerminalEvent::Input(mut input) => {
                        if let Input::Mouse(mouse) = &mut input {
                            mouse.size = self.buffer.size();
                        }

                        let mut events = crate::events::Vector(Vec::new());
                        element.handle(input, &mut events);
                        if !events.0.is_empty() {
                            return Ok(events.0);
                        }
                    }
                    TerminalEvent::Resize(size) => {
                        if size != self.buffer.grid.size() {
                            self.buffer.grid.resize_width(size.x);
                            self.old_buffer.grid.resize_width(size.x);

                            self.buffer
                                .grid
                                .resize_height_with_anchor(size.y, self.cursor_pos.y);
                            self.old_buffer
                                .grid
                                .resize_height_with_anchor(size.y, self.cursor_pos.y);

                            self.cursor_pos.x = min(self.cursor_pos.x, size.x - 1);
                            self.cursor_pos.y = min(self.cursor_pos.y, size.y - 1);

                            break;
                        }
                    }
                }
            }
        }
    }

    /// Diffs `old_buffer` and `new_buffer` and draws them to the backend.
    fn diff(&mut self) -> Result<(), Error<B::Error>> {
        let backend = self.backend.as_mut().unwrap();

        if self.old_buffer.title != self.buffer.title {
            backend.set_title(&self.buffer.title)?;
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
                                backend.$set_style(new_style$(.$path)+)?;
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
                    backend.set_cursor_pos(pos)?;
                }

                backend.write(&new_contents)?;

                self.style = *new_style;

                self.cursor_pos = Vec2::new(
                    min(
                        pos.x + if new_contents_double { 2 } else { 1 },
                        self.buffer.grid.width() - 1,
                    ),
                    pos.y,
                );
            }
        }

        // Some terminals use the background color of the cursor to fill in space created by a
        // resize, so reset it.
        backend.set_background(Color::Default)?;
        self.style.background = Color::Default;

        if let Some(new_cursor) = self.buffer.cursor {
            if self.old_buffer.cursor.is_none() {
                backend.show_cursor()?;
            }

            if self
                .old_buffer
                .cursor
                .map_or(true, |c| c.shape != new_cursor.shape)
            {
                backend.set_cursor_shape(new_cursor.shape)?;
            }
            if self
                .old_buffer
                .cursor
                .map_or(true, |c| c.blinking != new_cursor.blinking)
            {
                backend.set_cursor_blinking(new_cursor.blinking)?;
            }
            if self.cursor_pos != new_cursor.pos {
                backend.set_cursor_pos(new_cursor.pos)?;
            }
        } else if self.old_buffer.cursor.is_some() {
            backend.hide_cursor()?;
        }

        Ok(())
    }

    /// Get a reference to the terminal's backend.
    #[must_use]
    pub fn backend(&self) -> &B::Bound {
        self.backend.as_ref().unwrap()
    }

    /// Get a mutable reference to the terminal's backend.
    #[must_use]
    pub fn backend_mut(&mut self) -> &mut B::Bound {
        self.backend.as_mut().unwrap()
    }

    /// Take the captured standard output and standard error from the terminal.
    ///
    /// The terminal will no longer print all captured data to the standard output when the program
    /// terminates.
    ///
    /// This will return `None` if the backend is a dummy backend or the captured stdio has already
    /// been taken.
    pub fn take_captured(&mut self) -> Option<Captured> {
        self.captured.take().map(Captured)
    }

    /// Clean up the terminal.
    ///
    /// This will be called in the destructor too, but use this if you want to handle errors
    /// instead of ignoring them.
    ///
    /// # Errors
    ///
    /// Fails if cleaning up the backend fails.
    pub fn cleanup(mut self) -> Result<(), Error<B::Error>> {
        self.cleanup_inner()?;
        Ok(())
    }

    fn cleanup_inner(&mut self) -> Result<(), Error<B::Error>> {
        if let Some(backend) = self.backend.take() {
            backend.reset()?.cleanup().map_err(Error::Io)?;
        }

        if let Some(mut captured) = self.captured.take() {
            io::copy(&mut captured, &mut io::stdout()).map_err(Error::Io)?;
        }

        Ok(())
    }
}

impl<B: Backend> Drop for Terminal<B> {
    fn drop(&mut self) {
        let _ = self.cleanup_inner();

        if !B::is_dummy() {
            TERMINAL_EXISTS.store(false, Ordering::Release);
        }
    }
}

/// An error in Toon.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error<B> {
    /// An error in the backend.
    Backend(B),
    /// An I/O error.
    Io(io::Error),
}

impl<B> From<B> for Error<B> {
    fn from(e: B) -> Self {
        Self::Backend(e)
    }
}

impl<B: Display> Display for Error<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Backend(e) => e.fmt(f),
            Self::Io(e) => e.fmt(f),
        }
    }
}

impl<B: StdError + 'static> StdError for Error<B> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Backend(e) => Some(e),
            Self::Io(e) => Some(e),
        }
    }
}

/// Standard output and standard error that has been captured by Toon.
///
/// Note that this is a synchronous reader, and `async-io` does not have the ability to make it
/// asynchronous on Windows (as wepoll does not support pipes). So you if you want to use it
/// asynchronously you'll have to wrap it in an
/// [`Unblock`](https://docs.rs/blocking/1/blocking/struct.Unblock.html) or similar type.
#[derive(Debug)]
pub struct Captured(PipeReader);

impl Read for Captured {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }
}
impl<'a> Read for &'a Captured {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (&self.0).read(buf)
    }
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        (&self.0).read_vectored(bufs)
    }
}

#[cfg(test)]
#[test]
fn test_diff_grid() {
    use crate::backend::Operation;
    use crate::{Attributes, Intensity};

    use unicode_width::UnicodeWidthChar;

    fn write_at(grid: &mut Grid, at: (u16, u16), text: &str, style: Style) {
        let mut pos = at.0;
        for c in text.chars() {
            grid.write_char(Vec2::new(pos, at.1), c, style);
            pos += c.width().unwrap_or(0) as u16;
        }
    }

    let mut old_grid = Grid::new(Vec2::new(16, 8));
    write_at(&mut old_grid, (2, 5), "Hello World!", Style::default());
    write_at(&mut old_grid, (3, 6), "😃", Style::default());
    let old_grid = old_grid;

    // old grid:
    // +----------------+
    // |                |
    // |                |
    // |                |
    // |                |
    // |                |
    // |  Hello World!  |
    // |   😃           |
    // |                |
    // +----------------+

    let mut new_grid = old_grid.clone();

    let mut style = Style::new(
        Color::Red,
        Color::Blue,
        Attributes {
            intensity: Intensity::Bold,
            underlined: true,
            ..Attributes::default()
        },
    );

    write_at(&mut new_grid, (15, 2), "abcd", style);
    style.foreground = Color::Green;
    write_at(&mut new_grid, (1, 5), "foo", style);
    write_at(&mut new_grid, (4, 6), "😃", style);

    let new_grid = new_grid;

    // new grid:
    // +----------------+
    // |                |
    // |                |
    // |               a| (bcd is cut off)
    // |                |
    // |                |
    // | foollo World!  |
    // |    😃          | (moved one to the right)
    // |                |
    // +----------------+

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
            Operation::SetBackground(Color::Default),
        ],
    );
}
