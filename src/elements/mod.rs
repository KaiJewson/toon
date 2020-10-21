//! Common elements for building user interfaces.
//!
//! This module aims to cover most use cases of elements so you don't have to implement
//! [`Element`](../trait.Element.html) yourself.
//!
//! # Filters
//!
//! Filters are Toon's way of wrapping elements. To make your own, you simply implement the
//! [`Filter`](filter/trait.Filter.html) trait, and to use one you create a
//! [`Filtered`](filter/struct.Filtered.html) via the [`ElementExt`](filter/trait.ElementExt.html)
//! extension trait.

pub use filter::*;
pub use layout::*;

pub use line::*;

pub mod filter;
pub mod layout;

mod line;
