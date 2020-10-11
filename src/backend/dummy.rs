use std::collections::VecDeque;
use std::convert::Infallible;
use unicode_width::UnicodeWidthStr;

use futures_util::future;

use crate::style::{Color, Intensity, Style};
use crate::{Buffer, Cursor, CursorShape, Grid, Output, Vec2};

use super::{Backend, ReadEvents, TerminalEvent, Tty};

/// A dummy backend for testing.
///
/// This backend doesn't display any output to the screen, but records all operations it receives
/// and stores a terminal buffer.
pub struct Dummy {
    /// The operations the dummy backend has received.
    pub operations: Vec<Operation>,
    /// Events to feed the terminal. They will be popped from the front of the queue.
    ///
    /// If this is empty and the terminal requests an event it will return a never-completing
    /// future.
    pub events: VecDeque<TerminalEvent>,
    /// The buffer the dummy backend writes to.
    pub buffer: Buffer,
    /// The current position of the cursor.
    ///
    /// Unlike `buffer.cursor`, this stores the position of the cursor even when the cursor is
    /// hidden. This is the cursor position used for drawing.
    pub cursor_pos: Vec2<u16>,
    /// The current style being written with.
    pub style: Style,
}

impl Dummy {
    /// Create a new dummy backend with the given size.
    ///
    /// The cursor is hidden in the buffer and at `(0, 0)` in `cursor_pos`.
    #[must_use]
    pub fn new(size: Vec2<u16>) -> Self {
        Self {
            operations: Vec::new(),
            events: VecDeque::new(),
            buffer: Buffer::from(Grid::new(size)),
            cursor_pos: Vec2::new(0, 0),
            style: Style::default(),
        }
    }
}

/// An operation on a terminal backend, stored by `Dummy`.
///
/// Each variant roughly corresponds to a method on the `Backend` trait.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Operation {
    /// The title was set.
    SetTitle(String),
    /// The cursor was hidden.
    HideCursor,
    /// The cursor was shown.
    ShowCursor,
    /// The cursor's shape was set.
    SetCursorShape(CursorShape),
    /// Whether the cursor blinks was set.
    SetCursorBlinking(bool),
    /// The position of the cursor was set.
    SetCursorPos(Vec2<u16>),
    /// The foreground color was set.
    SetForeground(Color),
    /// The background color was set.
    SetBackground(Color),
    /// The text intensity was set.
    SetIntensity(Intensity),
    /// Whether the text is emphasized was set.
    SetItalic(bool),
    /// Whether the text is underlined was set.
    SetUnderlined(bool),
    /// Whether the text blinks was set.
    SetBlinking(bool),
    /// Whether the text is crossed out was set.
    SetCrossedOut(bool),
    /// Text was written to the output.
    Write(String),
    /// The output was flushed.
    Flush,
}

impl Backend for Dummy {
    type Config = Self;
    type Error = Infallible;

    // General functions

    fn new(config: Self::Config, _io: Tty) -> Result<Self, Self::Error> {
        Ok(config)
    }

    fn size(&mut self) -> Result<Vec2<u16>, Self::Error> {
        Ok(self.buffer.grid.size())
    }

    fn set_title(&mut self, title: &str) -> Result<(), Self::Error> {
        self.operations.push(Operation::SetTitle(title.to_owned()));
        self.buffer.title = title.to_owned();
        Ok(())
    }

    // Cursor functions

    fn hide_cursor(&mut self) -> Result<(), Self::Error> {
        self.operations.push(Operation::HideCursor);
        self.buffer.cursor = None;
        Ok(())
    }
    fn show_cursor(&mut self) -> Result<(), Self::Error> {
        self.operations.push(Operation::ShowCursor);
        self.buffer.cursor = Some(Cursor {
            shape: CursorShape::Block,
            blinking: false,
            pos: self.cursor_pos,
        });
        Ok(())
    }
    fn set_cursor_shape(&mut self, shape: CursorShape) -> Result<(), Self::Error> {
        self.operations.push(Operation::SetCursorShape(shape));
        self.buffer.cursor.as_mut().unwrap().shape = shape;
        Ok(())
    }
    fn set_cursor_blinking(&mut self, blinking: bool) -> Result<(), Self::Error> {
        self.operations.push(Operation::SetCursorBlinking(blinking));
        self.buffer.cursor.as_mut().unwrap().blinking = blinking;
        Ok(())
    }
    fn set_cursor_pos(&mut self, pos: Vec2<u16>) -> Result<(), Self::Error> {
        self.operations.push(Operation::SetCursorPos(pos));
        if let Some(cursor) = &mut self.buffer.cursor {
            cursor.pos = pos;
        }
        self.cursor_pos = pos;
        Ok(())
    }

    // Style functions

    fn set_foreground(&mut self, foreground: Color) -> Result<(), Self::Error> {
        self.operations.push(Operation::SetForeground(foreground));
        self.style.foreground = foreground;
        Ok(())
    }
    fn set_background(&mut self, background: Color) -> Result<(), Self::Error> {
        self.operations.push(Operation::SetBackground(background));
        self.style.background = background;
        Ok(())
    }
    fn set_intensity(&mut self, intensity: Intensity) -> Result<(), Self::Error> {
        self.operations.push(Operation::SetIntensity(intensity));
        self.style.attributes.intensity = intensity;
        Ok(())
    }
    fn set_italic(&mut self, italic: bool) -> Result<(), Self::Error> {
        self.operations.push(Operation::SetItalic(italic));
        self.style.attributes.italic = italic;
        Ok(())
    }
    fn set_underlined(&mut self, underlined: bool) -> Result<(), Self::Error> {
        self.operations.push(Operation::SetUnderlined(underlined));
        self.style.attributes.underlined = underlined;
        Ok(())
    }
    fn set_blinking(&mut self, blinking: bool) -> Result<(), Self::Error> {
        self.operations.push(Operation::SetBlinking(blinking));
        self.style.attributes.blinking = blinking;
        Ok(())
    }
    fn set_crossed_out(&mut self, crossed_out: bool) -> Result<(), Self::Error> {
        self.operations.push(Operation::SetCrossedOut(crossed_out));
        self.style.attributes.crossed_out = crossed_out;
        Ok(())
    }

    // Writing

    fn write(&mut self, text: &str) -> Result<(), Self::Error> {
        self.operations.push(Operation::Write(text.to_owned()));
        self.buffer.write(self.cursor_pos, &text, self.style);

        let x = self.cursor_pos.x + text.width() as u16;
        let grid_width = self.buffer.grid.width();
        self.cursor_pos = Vec2 {
            x: x % grid_width,
            y: self.cursor_pos.y + x / grid_width,
        };

        if let Some(cursor) = &mut self.buffer.cursor {
            cursor.pos = self.cursor_pos;
        }
        Ok(())
    }

    // Finalizing functions

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.operations.push(Operation::Flush);
        Ok(())
    }
    fn reset(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

type EventResult = Result<TerminalEvent, Infallible>;

impl<'a> ReadEvents<'a> for Dummy {
    type EventError = Infallible;
    type EventFuture = future::Either<future::Ready<EventResult>, future::Pending<EventResult>>;

    fn read_event(&'a mut self) -> Self::EventFuture {
        if let Some(event) = self.events.pop_front() {
            future::Either::Left(future::ready(Ok(event)))
        } else {
            future::Either::Right(future::pending())
        }
    }
}
