//! A simple, declarative, and modular TUI library.
//!
//! # Examples
//!
//! Display `Hello World!` on the terminal using the Crossterm backend:
//!
//! ```
//! # async {
//! use toon::{Crossterm, Terminal, ElementExt};
//!
//! let mut terminal = Terminal::new(Crossterm::default())?;
//!
//! terminal
//!     .draw(toon::line("Hello World!", toon::Style::default()).on('q', ()))
//!     .await?;
//!
//! terminal.cleanup()
//! # };
//! ```
#![cfg_attr(feature = "nightly", feature(doc_cfg))]
#![warn(
    clippy::pedantic,
    rust_2018_idioms,
    missing_docs,
    unused_qualifications,
    missing_debug_implementations
)]
#![allow(
    // `as u16` is used when we need to get the width of a string that is guaranteed not to exceed
    // u16.
    clippy::cast_possible_truncation,
    clippy::non_ascii_literal
)]

use std::fmt::{Display, Write as _};
use std::rc::Rc;
use std::sync::Arc;

pub use smartstring;
use unicode_width::UnicodeWidthChar;

#[cfg(feature = "crossterm")]
#[doc(no_inline)]
pub use backend::Crossterm;
#[doc(no_inline)]
pub use backend::{Backend, Dummy};
pub use buffer::Buffer;
pub use elements::*;
pub use events::Events;
pub use input::{Input, Key, KeyPress, Modifiers, Mouse, MouseButton, MouseKind};
pub use style::*;
pub use terminal::*;
pub use vec2::Vec2;

pub mod backend;
pub mod buffer;
pub mod elements;
pub mod events;
pub mod input;
pub mod style;
mod terminal;
mod util;
mod vec2;

/// An element on the screen.
///
/// Elements are cheap, immutable, borrowed and short-lived. They usually implement `Copy`.
///
/// You shouldn't generally have to implement this trait yourself unless you're doing something
/// really niche. Instead, combine elements from the [`elements`](elements/index.html) module.
pub trait Element<Event> {
    /// Draw the element to the output.
    fn draw(&self, output: &mut dyn Output);

    /// Get the inclusive range of widths the element can take up given an optional fixed height.
    fn width(&self, height: Option<u16>) -> (u16, u16);

    /// Get the inclusive range of heights the element can take up given an optional fixed width.
    fn height(&self, width: Option<u16>) -> (u16, u16);

    /// React to the input and output events if necessary.
    fn handle(&self, input: Input, events: &mut dyn Events<Event>);
}

impl<'a, E: Element<Event>, Event> Element<Event> for &'a E {
    fn draw(&self, output: &mut dyn Output) {
        (*self).draw(output)
    }
    fn width(&self, height: Option<u16>) -> (u16, u16) {
        (*self).width(height)
    }
    fn height(&self, width: Option<u16>) -> (u16, u16) {
        (*self).height(width)
    }
    fn handle(&self, input: Input, events: &mut dyn Events<Event>) {
        (*self).handle(input, events)
    }
}

macro_rules! implement_element_forwarding {
    ($($name:ident),*) => {
        $(
            impl<'a, Event, E: Element<Event> + ?Sized> Element<Event> for $name<E> {
                fn draw(&self, output: &mut dyn Output) {
                    (**self).draw(output)
                }
                fn width(&self, height: Option<u16>) -> (u16, u16) {
                    (**self).width(height)
                }
                fn height(&self, width: Option<u16>) -> (u16, u16) {
                    (**self).height(width)
                }
                fn handle(&self, input: Input, events: &mut dyn Events<Event>) {
                    (**self).handle(input, events)
                }
            }
        )*
    }
}
implement_element_forwarding!(Box, Arc, Rc);

/// An output to which elements draw themselves.
pub trait Output {
    /// Get the size of the output.
    ///
    /// Attempting to draw content outside the size will silently fail.
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

    /// Write a type implementing `Display` to the output at a zero-indexed position.
    ///
    /// If the string overflows the width of the terminal it will be cut off. Control characters
    /// will be ignored.
    ///
    /// # Panics
    ///
    /// This method will panic if the value's implementation of `Display` returns an error, which
    /// most implementations won't do.
    fn write(&mut self, mut pos: Vec2<u16>, value: &dyn Display, style: Style) {
        write!(
            util::WriteCharsFn(|c| {
                let width = match c.width() {
                    Some(width) => width,
                    None => return,
                } as u16;

                self.write_char(pos, c, style);

                pos.x += width;
            }),
            "{}",
            value
        )
        .expect("formatting failed");
    }

    /// Clear the output with one color.
    fn clear(&mut self, color: Color) {
        let size = self.size();
        let style = Style::new(Color::default(), color, Attributes::default());

        for x in 0..size.x {
            for y in 0..size.y {
                self.write_char(Vec2 { x, y }, ' ', style);
            }
        }
    }

    /// Set the title of the output.
    ///
    /// If this is called multiple times the last one will be used.
    fn set_title(&mut self, title: &dyn Display);

    /// Set the cursor of the output, if there is one.
    ///
    /// If this is called multiple times the last one will be used.
    fn set_cursor(&mut self, cursor: Option<Cursor>);
}

impl<'a, O: Output> Output for &'a mut O {
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

/// Construct an ad-hoc [`Output`](trait.Output.html) using the specified callbacks.
pub fn output_with<T>(
    state: T,
    size: impl Fn(&T) -> Vec2<u16>,
    write_char: impl FnMut(&mut T, Vec2<u16>, char, Style),
    set_title: impl FnMut(&mut T, &dyn Display),
    set_cursor: impl FnMut(&mut T, Option<Cursor>),
) -> impl Output {
    struct OutputWith<T, S, WC, ST, SC> {
        state: T,
        size: S,
        write_char: WC,
        set_title: ST,
        set_cursor: SC,
    }
    impl<T, S, WC, ST, SC> Output for OutputWith<T, S, WC, ST, SC>
    where
        S: Fn(&T) -> Vec2<u16>,
        WC: FnMut(&mut T, Vec2<u16>, char, Style),
        ST: FnMut(&mut T, &dyn Display),
        SC: FnMut(&mut T, Option<Cursor>),
    {
        fn size(&self) -> Vec2<u16> {
            (self.size)(&self.state)
        }
        fn write_char(&mut self, pos: Vec2<u16>, c: char, style: Style) {
            (self.write_char)(&mut self.state, pos, c, style)
        }
        fn set_title(&mut self, title: &dyn Display) {
            (self.set_title)(&mut self.state, title)
        }
        fn set_cursor(&mut self, cursor: Option<Cursor>) {
            (self.set_cursor)(&mut self.state, cursor)
        }
    }
    OutputWith {
        state,
        size,
        write_char,
        set_title,
        set_cursor,
    }
}

/// A terminal cursor.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Cursor {
    /// The shape of the cursor.
    pub shape: CursorShape,
    /// Whether the cursor blinks.
    pub blinking: bool,
    /// The zero-indexed position of the cursor.
    pub pos: Vec2<u16>,
}

/// The shape of a cursor.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum CursorShape {
    /// A bar to the left of the character.
    Bar,
    /// A full block over the character.
    Block,
    /// An underline under the character.
    Underline,
}
