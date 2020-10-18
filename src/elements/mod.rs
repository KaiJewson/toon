//! Common elements for building user interfaces.

use crate::{Element, Input};

mod filter;
mod text;

pub use filter::*;
pub use text::*;

/// An extension trait for elements providing useful methods.
pub trait ElementExt<Event>: Element<Event> + Sized {
    /// Filter this element using the given filter.
    ///
    /// Shortcut method for `Filtered::new(element, filter)`.
    fn filter<F: Filter<Event>>(self, filter: F) -> Filtered<Self, F> {
        Filtered::new(self, filter)
    }

    /// Trigger an event when an input occurs.
    ///
    /// Shortcut method for `.filter(toon::on(...))`.
    fn on(self, input: impl Into<Input>, event: Event) -> Filtered<Self, On<Event>>
    where
        Event: Clone,
    {
        self.filter(crate::on(input, event))
    }

    /// Erase the element's type by boxing it.
    fn boxed<'a>(self) -> Box<dyn Element<Event> + 'a>
    where
        Self: 'a;
}

impl<Event, T: Element<Event>> ElementExt<Event> for T {
    fn boxed<'a>(self) -> Box<dyn Element<Event> + 'a>
    where
        Self: 'a,
    {
        Box::new(self)
    }
}
