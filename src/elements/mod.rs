//! Common elements for building user interfaces.

use crate::{Element, Input};

mod text;
mod on;

pub use text::*;
pub use on::*;

/// An extension trait for elements providing useful methods.
pub trait ElementExt<E>: Element<E> + Sized {
    /// Trigger an event when an input occurs.
    fn on(self, input: impl Into<Input>, event: E) -> On<Self, E>;

    /// Erase the element's type by boxing it.
    fn boxed<'a>(self) -> Box<dyn Element<E> + 'a>
    where
        Self: 'a;
}

impl<E, T: Element<E>> ElementExt<E> for T {
    fn on(self, input: impl Into<Input>, event: E) -> On<Self, E> {
        On {
            element: self,
            input: input.into(),
            event,
        }
    }

    fn boxed<'a>(self) -> Box<dyn Element<E> + 'a>
    where
        Self: 'a,
    {
        Box::new(self)
    }
}
