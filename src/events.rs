//! Events created by elements in response to inputs.

use std::fmt::{self, Debug, Formatter};

/// A collector of events.
///
/// This trait is sealed - it cannot be implemented outside this crate - in order to prevent
/// elements from suppressing other elements' events.
pub trait Events<Event>: sealed::Sealed {
    /// Add an event to the collection of events.
    fn add(&mut self, event: Event);
}

impl<'a, T: Events<Event>, Event> Events<Event> for &'a mut T {
    fn add(&mut self, event: Event) {
        (*self).add(event);
    }
}
impl<'a, T> sealed::Sealed for &'a mut T {}

/// Extension methods for event collectors.
pub trait Ext<Event>: Events<Event> + Sized {
    /// Map the type of event being collected.
    fn map<F: Fn(Event2) -> Event, Event2>(self, f: F) -> Map<Self, F> {
        Map { inner: self, f }
    }
}

impl<T: Events<Event>, Event> Ext<Event> for T {}

/// An event collector that collects events into a vector.
pub(crate) struct Vector<E>(pub(crate) Vec<E>);

impl<E> Events<E> for Vector<E> {
    fn add(&mut self, event: E) {
        self.0.push(event);
    }
}
impl<E> sealed::Sealed for Vector<E> {}

/// An event collector that maps events from one type to another.
pub struct Map<E, F> {
    inner: E,
    f: F,
}
impl<E, F> Debug for Map<E, F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Map").finish()
    }
}
impl<E: Events<Event>, Event2, Event, F: Fn(Event2) -> Event> Events<Event2> for Map<E, F> {
    fn add(&mut self, event: Event2) {
        self.inner.add((self.f)(event));
    }
}
impl<E, F> sealed::Sealed for Map<E, F> {}

mod sealed {
    pub trait Sealed {}
}
