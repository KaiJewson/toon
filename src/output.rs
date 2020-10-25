//! Outputs which elements draw to.

use std::fmt::{Display, Write};

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
    fn set_cursor(&mut self, cursor: Option<Cursor>) {
        (**self).set_cursor(cursor)
    }
}

/// Extension methods for outputs.
///
/// Due to its generic name, it is not recommended to import this trait normally. Instead you
/// should just import its methods to reduce confusion:
///
/// ```
/// use toon::output::Ext as _;
/// ```
pub trait Ext: Output {
    /// Write a type implementing `Display` to the specified position in the output.
    ///
    /// If it overflows the width of the terminal it will be cut off. Control characters will be
    /// ignored.
    fn write(&mut self, pos: impl Into<Vec2<u16>>, value: &(impl Display + ?Sized), style: Style) {
        let total_width = self.size().x;
        let mut pos = pos.into();
        let _ = write!(
            crate::util::WriteCharsFn(|c| {
                let width = match c.width() {
                    Some(width) => width,
                    None => return Ok(()),
                } as u16;

                self.write_char(pos, c, style);

                pos.x += width;

                if pos.x >= total_width {
                    Err(std::fmt::Error)
                } else {
                    Ok(())
                }
            }),
            "{}",
            value,
        );
    }

    /// Create an output that draws to the specified area of this output.
    ///
    /// You can create an area that draws beyond the bounds of this output, in which case it will
    /// all be ignored.
    #[must_use]
    fn area(self, top_left: Vec2<u16>, size: Vec2<u16>) -> Area<Self>
    where
        Self: Sized,
    {
        Area {
            inner: self,
            top_left,
            size,
        }
    }

    /// Call the callback when the cursor is set on the output.
    #[must_use]
    fn on_set_cursor<F: FnMut(&mut Self, Option<Cursor>)>(self, f: F) -> OnSetCursor<Self, F>
    where
        Self: Sized,
    {
        OnSetCursor { inner: self, f }
    }
}
impl<T: Output + ?Sized> Ext for T {}

/// An [`Output`](trait.Output.html) that draws to an area of another output, created by the
/// [`area`](trait.Ext.html#method.area) method.
#[derive(Debug)]
#[non_exhaustive]
pub struct Area<O> {
    /// The inner output.
    pub inner: O,
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

/// An [`Output`](trait.Output.html) that calls a callback when its cursor is set, created by the
/// [`on_set_cursor`](trait.Ext.html#method.on_set_cursor) function.
#[derive(Debug)]
pub struct OnSetCursor<O, F> {
    /// The inner output.
    pub inner: O,
    f: F,
}

impl<O: Output, F: FnMut(&mut O, Option<Cursor>)> Output for OnSetCursor<O, F> {
    fn size(&self) -> Vec2<u16> {
        self.inner.size()
    }
    fn write_char(&mut self, pos: Vec2<u16>, c: char, style: Style) {
        self.inner.write_char(pos, c, style);
    }
    fn set_cursor(&mut self, cursor: Option<Cursor>) {
        (self.f)(&mut self.inner, cursor);
    }
}
