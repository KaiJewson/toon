//! Outputs which elements draw to.

use std::fmt::{Display, Write};

use unicode_width::UnicodeWidthChar;

use crate::{Cursor, Style, Vec2};

/// An output to which elements draw themselves.
///
/// See [`Ext`] for some higher-level methods on outputs.
pub trait Output {
    /// Get the size of the output.
    ///
    /// Attempting to draw content outside the size will silently fail.
    #[must_use]
    fn size(&self) -> Vec2<u16>;

    /// Write a single character to the output at a zero-indexed position.
    ///
    /// Failures are intentionally ignored and not detectable - the [`Output`]'s current state is
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

#[cfg(feature = "either")]
impl<L: Output, R: Output> Output for either_crate::Either<L, R> {
    fn size(&self) -> Vec2<u16> {
        match self {
            Self::Left(l) => l.size(),
            Self::Right(r) => r.size(),
        }
    }
    fn write_char(&mut self, pos: Vec2<u16>, c: char, style: Style) {
        match self {
            Self::Left(l) => l.write_char(pos, c, style),
            Self::Right(r) => r.write_char(pos, c, style),
        }
    }
    fn set_cursor(&mut self, cursor: Option<Cursor>) {
        match self {
            Self::Left(l) => l.set_cursor(cursor),
            Self::Right(r) => r.set_cursor(cursor),
        }
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
///
/// This trait exists because [`Output`] needs to be dyn-safe, but it is convenient to have methods
/// that use generics.
pub trait Ext: Output {
    /// Write a type implementing [`Display`] to the specified position in the output.
    ///
    /// If it overflows the width of the terminal it will be cut off. Control characters will be
    /// ignored.
    fn write(&mut self, pos: impl Into<Vec2<u16>>, value: impl Display, style: Style) {
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
    ///
    /// The `top_left` parameter is conceptually an `i17`, but that doesn't exist so we use [`i32`].
    #[must_use]
    fn area(self, top_left: impl Into<Vec2<i32>>, size: impl Into<Vec2<u16>>) -> Area<Self>
    where
        Self: Sized,
    {
        Area {
            inner: self,
            top_left: top_left.into(),
            size: size.into(),
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

/// An [`Output`] that draws to an area of another output, created by the [`area`](Ext::area)
/// method.
#[derive(Debug)]
#[non_exhaustive]
pub struct Area<O> {
    /// The inner output.
    pub inner: O,
    top_left: Vec2<i32>,
    size: Vec2<u16>,
}

impl<O: Output> Output for Area<O> {
    fn size(&self) -> Vec2<u16> {
        self.size
    }
    fn write_char(&mut self, pos: Vec2<u16>, c: char, style: Style) {
        if pos.x >= self.size.x
            || pos.y >= self.size.y
            || (pos.x == self.size.x - 1 && c.width() == Some(2))
        {
            return;
        }
        let pos = match pos
            .map(i32::from)
            .checked_add(self.top_left)
            .and_then(|v| v.try_into::<u16>().ok())
        {
            Some(pos) => pos,
            None => return,
        };
        self.inner.write_char(pos, c, style);
    }
    fn set_cursor(&mut self, cursor: Option<Cursor>) {
        self.inner.set_cursor(
            cursor
                .filter(|cursor| cursor.pos.x < self.size.x && cursor.pos.y < self.size.y)
                .and_then(|cursor| {
                    Some(Cursor {
                        pos: cursor
                            .pos
                            .map(i32::from)
                            .checked_add(self.top_left)?
                            .try_into::<u16>()
                            .ok()?,
                        ..cursor
                    })
                }),
        );
    }
}

/// An [`Output`] that calls a callback when its cursor is set, created by the
/// [`on_set_cursor`](Ext::on_set_cursor) function.
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
