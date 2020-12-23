//! [![Github](https://img.shields.io/badge/repository-github-24292e)](https://github.com/KaiJewson/toon)
//! [![Crates.io](https://img.shields.io/crates/v/toon)](https://crates.io/crates/toon)
//! [![docs.rs](https://docs.rs/toon/badge.svg)](https://docs.rs/toon)
//!
//! A simple, declarative, and modular TUI library.
//!
//! In Toon, every application starts out with some **state**. Then, using your state you create an
//! **element** (the [`Element`](https://docs.rs/toon/0.1/toon/trait.Element.html) trait). You pass
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
//! #[cfg(feature = "crossterm")]
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
//! implements [`Element`](https://docs.rs/toon/0.1/toon/trait.Element.html),
//! [`Output`](https://docs.rs/toon/0.1/toon/output/trait.Output.html) and
//! [`Collection`](https://docs.rs/toon/0.1/toon/elements/containers/trait.Collection.html) for
//! `Either`.
#![cfg_attr(feature = "doc_cfg", feature(doc_cfg))]
#![warn(
    clippy::cargo,
    clippy::pedantic,
    clippy::wrong_pub_self_convention,
    rust_2018_idioms,
    missing_docs,
    unused_qualifications,
    missing_debug_implementations
)]
#![allow(
    // `as u16` is used when we need to get the width of a string that is guaranteed not to exceed
    // u16.
    clippy::cast_possible_truncation,
    // socket2 hasn't released its new version yet
    clippy::multiple_crate_versions,
    clippy::non_ascii_literal,
    clippy::struct_excessive_bools,
    // See issue #74087: <https://github.com/rust-lang/rust/issues/74087>
    // It is triggered by input.rs' __internal_key! macro
    macro_expanded_macro_exports_accessed_by_absolute_paths,
)]
// For checking before a release
// #![deny(
//     clippy::dbg_macro,
//     clippy::print_stderr,
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

#[cfg(feature = "either")]
use either_crate::Either;

pub mod backend;
#[cfg(feature = "crossterm")]
#[doc(no_inline)]
pub use backend::Crossterm;
#[doc(no_inline)]
pub use backend::{Backend, Dummy};

pub mod buffer;
pub use buffer::*;

pub mod elements;
pub use elements::*;

pub mod input;
pub use input::{Input, Key, KeyPress, Modifiers, Mouse, MouseButton, MouseKind};

pub mod output;
pub use output::Output;

pub mod style;
pub use style::*;

mod events;
pub use events::Events;

mod terminal;
pub use terminal::*;

mod util;

mod vec2;
pub use vec2::Vec2;

/// A composable part of the UI.
///
/// Elements are cheap, immutable, borrowed and short-lived. They usually implement [`Copy`].
///
/// You shouldn't generally have to implement this trait yourself unless you're doing something
/// really niche. Instead, combine elements from the [`elements`] module.
pub trait Element {
    /// The type of event this element produces.
    type Event;

    /// Draw the element to the output.
    ///
    /// Elements shouldn't draw to every part of the output if they don't have to. Containers like
    /// [`Stack`] allow users to set whatever content they like for the background.
    fn draw(&self, output: &mut dyn Output);

    /// Get the ideal width that this element takes up given a fixed height and an optional maximum
    /// width.
    ///
    /// Implementors may return a value higher than the maximum from this function, in which case
    /// callers should do their best to fulfill this request, but can simply cap it off at the
    /// maximum if they wish.
    fn ideal_width(&self, height: u16, max_width: Option<u16>) -> u16;

    /// Get the ideal height that this element takes up given a fixed width and an optional maximum
    /// height.
    ///
    /// Implementors may return a value higher than the maximum from this function, in which case
    /// callers should do their best to fulfill this request, but can simply cap it off at the
    /// maximum if they wish.
    fn ideal_height(&self, width: u16, max_height: Option<u16>) -> u16;

    /// Get the ideal size of the element given an optional maximum size.
    ///
    /// Implementors may return a value higher than the maximum from this function in either
    /// dimension, in which case callers should do their best to fulfill this request, but can
    /// simply cap it off at the maximum if they wish.
    fn ideal_size(&self, maximum: Vec2<Option<u16>>) -> Vec2<u16>;

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

macro_rules! implement_element_forwarding {
    ($($name:ty),*) => {
        $(
            impl<'a, E: Element + ?Sized> Element for $name {
                type Event = E::Event;

                fn draw(&self, output: &mut dyn Output) {
                    (**self).draw(output)
                }
                fn ideal_width(&self, height: u16, max_width: Option<u16>) -> u16 {
                    (**self).ideal_width(height, max_width)
                }
                fn ideal_height(&self, width: u16, max_height: Option<u16>) -> u16 {
                    (**self).ideal_height(width, max_height)
                }
                fn ideal_size(&self, maximum: Vec2<Option<u16>>) -> Vec2<u16> {
                    (**self).ideal_size(maximum)
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
implement_element_forwarding!(&'a E, Box<E>, Arc<E>, Rc<E>);

#[cfg(feature = "either")]
impl<L: Element, R: Element<Event = L::Event>> Element for Either<L, R> {
    type Event = L::Event;

    fn draw(&self, output: &mut dyn Output) {
        match self {
            Self::Left(l) => l.draw(output),
            Self::Right(r) => r.draw(output),
        }
    }
    fn ideal_width(&self, height: u16, max_width: Option<u16>) -> u16 {
        match self {
            Self::Left(l) => l.ideal_width(height, max_width),
            Self::Right(r) => r.ideal_width(height, max_width),
        }
    }
    fn ideal_height(&self, width: u16, max_height: Option<u16>) -> u16 {
        match self {
            Self::Left(l) => l.ideal_height(width, max_height),
            Self::Right(r) => r.ideal_height(width, max_height),
        }
    }
    fn ideal_size(&self, maximum: Vec2<Option<u16>>) -> Vec2<u16> {
        match self {
            Self::Left(l) => l.ideal_size(maximum),
            Self::Right(l) => l.ideal_size(maximum),
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
