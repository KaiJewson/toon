//! Common elements for building user interfaces.
//!
//! This module aims to cover most use cases of elements so you don't have to implement
//! [`Element`](../trait.Element.html) yourself.

use crate::{input, Element};

pub use filter::*;
pub use containers::*;

pub use block::*;
pub use span::*;

pub mod filter;
pub mod containers;

mod block;
mod span;

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
    /// # Examples
    ///
    /// ```
    /// # use toon::ElementExt;
    /// # let element = toon::empty();
    /// # #[derive(Clone)]
    /// # enum Event { Exit }
    /// // When the 'q' key is pressed or the element is clicked an Exit event will be triggered.
    /// let element = element.on(('q', toon::MouseButton::Left), Event::Exit);
    /// ```
    fn on<I: input::Pattern>(self, input_pattern: I, event: Event) -> Filtered<Self, On<I, Event>>
    where
        Event: Clone,
    {
        self.filter(On { input_pattern, event })
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
