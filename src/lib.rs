//! [![Crates.io](https://img.shields.io/crates/v/toon)](https://crates.io/crates/toon)
//! [![Github](https://img.shields.io/badge/-github-24292e)](https://github.com/KaiJewson/toon)
//! [![docs.rs](https://img.shields.io/badge/-docs.rs-informational)](https://docs.rs/toon)
//!
//! A simple, declarative, and modular TUI library.
//!
//! In Toon, every application starts out with some **state**. Then, using your state you create an
//! **element** (the [`Element`](https://docs.rs/toon/0.1/toon/trait.Element.trait) trait). You pass
//! your element to Toon using
//! [`Terminal::draw`](https://docs.rs/toon/0.1/toon/struct.Terminal.html#method.draw) and it
//! renders it to the screen, before waiting for user input. When that occurs, Toon uses your
//! element to translate it into some number of **events**, which are then used to modify your
//! state, and the cycle repeats.
//!
//! ```text
//!          Drawing                Input
//! State ────────────→ Elements ──────────→ Events
//!   ↑                                        │
//!   ╰────────────────────────────────────────╯
//! ```
//!
//! As such, your UI is a simple pure function of your state. This helps eliminate a whole class of
//! inconsistency bugs; given a certain state, your UI will look the exact same way, _always_. The
//! event system also allows you to easily trace each and every modification to your state, which
//! can be very useful.
//!
//! See the [comparison](https://github.com/KaiJewson/toon/blob/master/COMPARISON.md) to compare it
//! with the other big TUI libraries, [Cursive](https://github.com/gyscos/cursive) and
//! [tui](https://github.com/fdehau/tui-rs).
//!
//! # Example
//!
//! See the [examples](https://github.com/KaiJewson/toon/tree/master/examples) folder for more.
//!
//! Display `Hello World!` on the terminal using the Crossterm backend:
//! ```
//! # async {
//! use toon::{Crossterm, Terminal, ElementExt};
//!
//! let mut terminal = Terminal::new(Crossterm::default())?;
//!
//! terminal
//!     .draw(toon::span("Hello World!").on('q', |_| ()))
//!     .await?;
//!
//! terminal.cleanup()
//! # };
//! ```
//!
//! # Features
//!
//! Toon offers the following features, none of which are enabled by default:
//! - `crossterm`: Enable the
//! [Crossterm](https://docs.rs/toon/0.1/toon/backend/struct.Crossterm.html) backend.
//! - `dev`: Enable developer tools.
//! - `either`: Integrate with the [`either`](https://crates.io/crates/either) crate. This
//! implements `Element`, `Output` and `Collection` for `Either`.
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
    clippy::non_ascii_literal,
    clippy::struct_excessive_bools,
    // See issue #74087: <https://github.com/rust-lang/rust/issues/74087>
    // It is triggered by input.rs' __internal_key! macro
    macro_expanded_macro_exports_accessed_by_absolute_paths,
)]
// For checking before a release
// #![deny(
//     clippy::dbg_macro,
//     clippy::print_stdout,
//     clippy::todo,
//     clippy::unimplemented,
//     clippy::use_debug
// )]

use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

#[doc(hidden)]
pub use ::std;

#[cfg(feature = "either")]
pub use either_crate as either;
pub use smartstring;

#[cfg(feature = "either")]
use either_crate::Either;

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

/// A composable part of the UI.
///
/// Elements are cheap, immutable, borrowed and short-lived. They usually implement `Copy`.
///
/// You shouldn't generally have to implement this trait yourself unless you're doing something
/// really niche. Instead, combine elements from the [`elements`](elements/index.html) module.
pub trait Element {
    /// Events that this element produces.
    type Event;

    /// Draw the element to the output.
    ///
    /// Elements shouldn't draw to every part of the output if they don't have to. Containers like
    /// [`Stack`](elements/containers/struct.Stack.html) allow users to set whatever content they
    /// like for the background.
    fn draw(&self, output: &mut dyn Output);

    /// Get the inclusive range of widths the element can take up given an optional fixed height.
    ///
    /// The second value must be >= the first, otherwise panics may occur.
    fn width(&self, height: Option<u16>) -> (u16, u16);

    /// Get the inclusive range of heights the element can take up given an optional fixed width.
    ///
    /// The second value must be >= the first, otherwise panics may occur.
    fn height(&self, width: Option<u16>) -> (u16, u16);

    /// React to the input and output events if necessary.
    fn handle(&self, input: Input, events: &mut dyn Events<Self::Event>);

    /// Write the title of the element to the writer.
    ///
    /// # Errors
    ///
    /// This function should always propagate errors from the writer, and returning errors not
    /// created by the writer may result in panics.
    fn title(&self, _title: &mut dyn fmt::Write) -> fmt::Result {
        Ok(())
    }
}

impl<'a, E: Element + ?Sized> Element for &'a E {
    type Event = E::Event;

    fn draw(&self, output: &mut dyn Output) {
        (*self).draw(output)
    }
    fn width(&self, height: Option<u16>) -> (u16, u16) {
        (*self).width(height)
    }
    fn height(&self, width: Option<u16>) -> (u16, u16) {
        (*self).height(width)
    }
    fn handle(&self, input: Input, events: &mut dyn Events<Self::Event>) {
        (*self).handle(input, events)
    }
    fn title(&self, title: &mut dyn fmt::Write) -> fmt::Result {
        (*self).title(title)
    }
}

macro_rules! implement_element_forwarding {
    ($($name:ident),*) => {
        $(
            impl<'a, E: Element + ?Sized> Element for $name<E> {
                type Event = E::Event;

                fn draw(&self, output: &mut dyn Output) {
                    (**self).draw(output)
                }
                fn width(&self, height: Option<u16>) -> (u16, u16) {
                    (**self).width(height)
                }
                fn height(&self, width: Option<u16>) -> (u16, u16) {
                    (**self).height(width)
                }
                fn handle(&self, input: Input, events: &mut dyn Events<Self::Event>) {
                    (**self).handle(input, events)
                }
                fn title(&self, title: &mut dyn fmt::Write) -> fmt::Result {
                    (**self).title(title)
                }
            }
        )*
    }
}
implement_element_forwarding!(Box, Arc, Rc);

#[cfg(feature = "either")]
impl<L: Element, R: Element<Event = L::Event>> Element for Either<L, R> {
    type Event = L::Event;

    fn draw(&self, output: &mut dyn Output) {
        match self {
            Self::Left(l) => l.draw(output),
            Self::Right(r) => r.draw(output),
        }
    }
    fn width(&self, height: Option<u16>) -> (u16, u16) {
        match self {
            Self::Left(l) => l.width(height),
            Self::Right(r) => r.width(height),
        }
    }
    fn height(&self, width: Option<u16>) -> (u16, u16) {
        match self {
            Self::Left(l) => l.height(width),
            Self::Right(r) => r.height(width),
        }
    }
    fn handle(&self, input: Input, events: &mut dyn Events<Self::Event>) {
        match self {
            Self::Left(l) => l.handle(input, events),
            Self::Right(r) => r.handle(input, events),
        }
    }
    fn title(&self, title: &mut dyn fmt::Write) -> fmt::Result {
        match self {
            Self::Left(l) => l.title(title),
            Self::Right(r) => r.title(title),
        }
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
