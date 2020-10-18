use crate::{Element, Events, Input};

use super::Filter;

/// A filter that triggers an event when an input occurs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct On<Event> {
    input: Input,
    event: Event,
}

impl<Event> On<Event> {
    /// Create a new `On` filter.
    #[must_use]
    pub fn new(input: impl Into<Input>, event: Event) -> Self {
        Self {
            input: input.into(),
            event,
        }
    }
}

impl<Event: Clone> Filter<Event> for On<Event> {
    fn handle(&self, element: &dyn Element<Event>, input: Input, events: &mut dyn Events<Event>) {
        if input == self.input {
            events.add(self.event.clone());
        } else {
            element.handle(input, events);
        }
    }
}
