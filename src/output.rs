//! Outputs which elements draw to.

use std::fmt::Display;

use unicode_width::UnicodeWidthChar;

use crate::{Cursor, Style, Vec2};

/// An output to which elements draw themselves.
pub trait Output {
    /// Get the size of the output.
    ///
    /// Attempting to draw content outside the size will silently fail.
    #[must_use]
    fn size(&self) -> Vec2<u16>;

    /// Write a single character to the output at a zero-indexed position.
    ///
    /// Failures are intentionally ignored and not detectable - the `Output`'s current state is
    /// completely opaque.
    ///
    /// - Drawing a control character will fail.
    /// - Drawing a character out of bounds will fail.
    /// - Drawing a double-width character to the last column of the screen will fail.
    /// - Drawing a zero-width character on top of an existing character will add it to it, ignoring
    /// the zero-width character's style.
    /// - Drawing a zero-width character on the second column of a double-width character will fail.
    /// - Drawing a single-width or double-width character on top of a single-width character will
    /// completely replace it.
    /// - Drawing a single-width or double-width character to either column of a double-width
    /// character will completely replace the columns drawn to, and any other columns previously
    /// occupied by the double-width character will retain the background color of the double-width
    /// character.
    fn write_char(&mut self, pos: Vec2<u16>, c: char, style: Style);

    /// Set the title of the output.
    ///
    /// If this is called multiple times the last one will be used.
    fn set_title(&mut self, title: &dyn Display);

    /// Set the cursor of the output, if there is one.
    ///
    /// If this is called multiple times the last one will be used.
    fn set_cursor(&mut self, cursor: Option<Cursor>);
}

impl<'a, O: Output + ?Sized> Output for &'a mut O {
    fn size(&self) -> Vec2<u16> {
        (**self).size()
    }
    fn write_char(&mut self, pos: Vec2<u16>, c: char, style: Style) {
        (**self).write_char(pos, c, style)
    }
    fn set_title(&mut self, title: &dyn Display) {
        (**self).set_title(title)
    }
    fn set_cursor(&mut self, cursor: Option<Cursor>) {
        (**self).set_cursor(cursor)
    }
}

/// Extension methods for outputs.
pub trait Ext: Output + Sized {
    /// Create an output that draws to the specified area of this output.
    ///
    /// You can create an area that draws beyond the bounds of this output, in which case it will
    /// all be ignored.
    #[must_use]
    fn area(self, top_left: Vec2<u16>, size: Vec2<u16>) -> Area<Self> {
        Area {
            inner: self,
            top_left,
            size,
        }
    }

    /// Set whether the output is focused. Focused outputs will allow setting the title and cursor.
    #[must_use]
    fn focused(self, focused: bool) -> MaybeFocused<Self> {
        MaybeFocused {
            inner: self,
            focused,
        }
    }
}
impl<T: Output> Ext for T {}

/// An [`Output`](trait.Output.html) that draws to an area of another output, created by the
/// [`area`](trait.Output.html#method.area) method.
#[derive(Debug)]
pub struct Area<O> {
    inner: O,
    top_left: Vec2<u16>,
    size: Vec2<u16>,
}

impl<O: Output> Output for Area<O> {
    fn size(&self) -> Vec2<u16> {
        self.size
    }
    fn write_char(&mut self, pos: Vec2<u16>, c: char, style: Style) {
        if pos.x < self.size.x
            && pos.y < self.size.y
            && (pos.x < self.size.x - 1 || c.width() != Some(2))
        {
            self.inner.write_char(pos + self.top_left, c, style);
        }
    }
    fn set_title(&mut self, title: &dyn Display) {
        self.inner.set_title(title);
    }
    fn set_cursor(&mut self, cursor: Option<Cursor>) {
        self.inner.set_cursor(
            cursor
                .filter(|cursor| cursor.pos.x < self.size.x && cursor.pos.y < self.size.y)
                .map(|cursor| Cursor {
                    pos: cursor.pos + self.top_left,
                    ..cursor
                }),
        );
    }
}

/// An [`Output`](trait.Output.html) that may or may not be focused, created by the
/// [`focused`](trait.Output.html#method.focused) function.
#[derive(Debug)]
pub struct MaybeFocused<O> {
    inner: O,
    focused: bool,
}

impl<O: Output> Output for MaybeFocused<O> {
    fn size(&self) -> Vec2<u16> {
        self.inner.size()
    }
    fn write_char(&mut self, pos: Vec2<u16>, c: char, style: Style) {
        self.inner.write_char(pos, c, style)
    }
    fn set_title(&mut self, title: &dyn Display) {
        if self.focused {
            self.inner.set_title(title);
        }
    }
    fn set_cursor(&mut self, cursor: Option<Cursor>) {
        if self.focused {
            self.inner.set_cursor(cursor);
        }
    }
}
