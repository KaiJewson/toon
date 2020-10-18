//! Events created by elements in response to inputs.

use std::fmt::{self, Debug, Formatter};

/// A collector of events.
///
/// This trait is sealed - it cannot be implemented outside this crate - in order to prevent
/// elements from suppressing other elements' events.
pub trait Events<E>: sealed::Sealed {
    /// Add an event to the collection of events.
    fn add(&mut self, event: E);
}

/// Extension methods for event collectors.
pub trait Ext<E>: sealed::Sealed {
    /// Map the type of event being collected.
    fn map<F: Fn(E2) -> E, E2>(&mut self, f: F) -> Map<'_, E, F>;
}

impl<E> Ext<E> for dyn Events<E> {
    fn map<F: Fn(E2) -> E, E2>(&mut self, f: F) -> Map<'_, E, F> {
        Map { inner: self, f }
    }
}

/// An event collector that collects events into a vector.
pub(crate) struct Vector<E>(pub(crate) Vec<E>);

impl<E> Events<E> for Vector<E> {
    fn add(&mut self, event: E) {
        self.0.push(event);
    }
}
impl<E> sealed::Sealed for Vector<E> {}

/// An event collector that maps events from one type to another.
pub struct Map<'a, E, F> {
    inner: &'a mut dyn Events<E>,
    f: F,
}
impl<'a, E, F> Debug for Map<'a, E, F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Map").finish()
    }
}
impl<'a, E, E2, F: Fn(E2) -> E> Events<E2> for Map<'a, E, F> {
    fn add(&mut self, event: E2) {
        self.inner.add((self.f)(event));
    }
}
impl<'a, E, F> sealed::Sealed for Map<'a, E, F> {}

mod sealed {
    pub trait Sealed {}
}
