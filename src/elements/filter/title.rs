use std::fmt::{self, Display};

use crate::Element;

use super::Filter;

/// A filter that sets the title of an element, created by the
/// [`title`](crate::ElementExt::title) function.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub struct Title<T> {
    /// The title of the element.
    pub title: T,
}

impl<T> Title<T> {
    /// Create a new filter that sets the title of the element.
    #[must_use]
    pub fn new(title: T) -> Self {
        Self { title }
    }
}

impl<T: Display, Event> Filter<Event> for Title<T> {
    fn title<E: Element>(&self, _element: E, title: &mut dyn fmt::Write) -> fmt::Result {
        write!(title, "{}", self.title)
    }
}
