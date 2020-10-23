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
//!     .draw(toon::span("Hello World!").on('q', ()))
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

use std::rc::Rc;
use std::sync::Arc;

pub use smartstring;

#[cfg(feature = "crossterm")]
#[doc(no_inline)]
pub use backend::Crossterm;
#[doc(no_inline)]
pub use backend::{Backend, Dummy};
pub use buffer::*;
pub use elements::*;
pub use events::Events;
pub use input::{Input, Key, KeyPress, Modifiers, Mouse, MouseButton, MouseKind};
pub use output::Output;
pub use style::*;
pub use terminal::*;
pub use vec2::Vec2;

pub mod backend;
pub mod buffer;
pub mod elements;
pub mod events;
pub mod input;
pub mod output;
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

impl<'a, E: Element<Event> + ?Sized, Event> Element<Event> for &'a E {
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
