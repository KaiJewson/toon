//! Common elements for building user interfaces.
//!
//! This module aims to cover most use cases of elements so you don't have to implement
//! [`Element`](../trait.Element.html) yourself.

use std::fmt::Display;

use crate::{input, Element, Input, Vec2};

pub use containers::*;
pub use filter::*;

pub use block::*;
pub use span::*;
pub use map_event::*;

pub mod containers;
pub mod filter;

mod block;
mod span;
mod map_event;

/// An extension trait for elements providing useful methods.
pub trait ElementExt: Element + Sized {
    /// Filter this element using the given filter.
    ///
    /// This is a shortcut method for [`Filtered::new`](filter/struct.Filtered.html#method.new).
    #[must_use]
    fn filter<F: Filter<Self::Event>>(self, filter: F) -> Filtered<Self, F> {
        Filtered::new(self, filter)
    }

    /// Trigger an event when an input occurs.
    ///
    /// The created element will listen to inputs _actively_; the input if it occurs will not be
    /// passed to the inner element.
    ///
    /// # Examples
    ///
    /// ```
    /// # use toon::ElementExt;
    /// # let element = toon::empty();
    /// # #[derive(Clone)]
    /// # enum Event { Exit }
    /// // When the 'q' key is pressed or the element is clicked an Exit event will be triggered.
    /// let element = element.on(('q', toon::MouseButton::Left), |_| Event::Exit);
    /// ```
    #[must_use]
    fn on<I: input::Pattern, F: Fn(Input) -> Self::Event>(
        self,
        input_pattern: I,
        event: F,
    ) -> Filtered<Self, On<I, F>> {
        self.filter(On::new(input_pattern, event))
    }

    /// Trigger an event when an input occurs, passively; the inner element will still receive
    /// all inputs.
    #[must_use]
    fn on_passive<I: input::Pattern, F: Fn(Input) -> Self::Event>(
        self,
        input_pattern: I,
        event: F,
    ) -> Filtered<Self, On<I, F>> {
        self.filter(On::new(input_pattern, event).passive())
    }

    /// Make the element float with the given alignment.
    ///
    /// # Example
    ///
    /// Make the element its smallest size at the middle right of the screen.
    ///
    /// ```
    /// use toon::{Alignment, ElementExt};
    ///
    /// # let element = toon::span::<_, ()>("Hello World!");
    /// let element = element.float((Alignment::End, Alignment::Middle));
    /// ```
    #[must_use]
    fn float(self, align: impl Into<Vec2<Alignment>>) -> Filtered<Self, Float> {
        self.filter(Float::new(align))
    }

    /// Set the title of the element.
    #[must_use]
    fn title<T: Display>(self, title: T) -> Filtered<Self, Title<T>> {
        self.filter(Title::new(title))
    }

    /// Map the type of event produced by the element.
    #[must_use]
    fn map_event<Event2, F: Fn(Self::Event) -> Event2>(self, f: F) -> MapEvent<Self, F> {
        MapEvent {
            inner: self,
            f,
        }
    }

    /// Erase the element's type by boxing it.
    fn boxed<'a>(self) -> Box<dyn Element<Event = Self::Event> + 'a>
    where
        Self: 'a,
    {
        Box::new(self)
    }
}
impl<T: Element> ElementExt for T {}
