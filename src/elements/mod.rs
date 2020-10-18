//! Common elements for building user interfaces.
//!
//! This module aims to cover most use cases of elements so you don't have to implement
//! [`Element`](../trait.Element.html) yourself.

use crate::{Element, Input};

pub mod filter;
mod text;

pub use filter::*;
pub use text::*;

/// An extension trait for elements providing useful methods.
pub trait ElementExt<Event>: Element<Event> + Sized {
    /// Filter this element using the given filter.
    ///
    /// This is a shortcut method for [`Filtered::new`](filter/struct.Filtered.html#method.new).
    fn filter<F: Filter<Event>>(self, filter: F) -> Filtered<Self, F> {
        Filtered::new(self, filter)
    }

    /// Trigger an event when an input occurs.
    ///
    /// This is a shortcut method for `.filter(toon::On::new(...))`.
    fn on(self, input: impl Into<Input>, event: Event) -> Filtered<Self, On<Event>>
    where
        Event: Clone,
    {
        self.filter(On::new(input, event))
    }

    /// Erase the element's type by boxing it.
    fn boxed<'a>(self) -> Box<dyn Element<Event> + 'a>
    where
        Self: 'a,
    {
        Box::new(self)
    }
}

impl<Event, T: Element<Event>> ElementExt<Event> for T {}
