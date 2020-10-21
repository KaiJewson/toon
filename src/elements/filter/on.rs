use crate::{input, Element, Events, Input};

use super::Filter;

/// A filter that triggers an event when an input occurs, created by the
/// [`on`](../trait.ElementExt.html#method.on) function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct On<I, Event> {
    input_pattern: I,
    event: Event,
}

impl<I, Event> On<I, Event> {
    /// Create a new `On` filter. Note that use of the [`on`](../trait.ElementExt.html#method.on)
    /// function is preferred.
    #[must_use]
    pub fn new(input_pattern: I, event: Event) -> Self {
        Self {
            input_pattern,
            event,
        }
    }
}

impl<I: input::Pattern, Event: Clone> Filter<Event> for On<I, Event> {
    fn handle(&self, element: &dyn Element<Event>, input: Input, events: &mut dyn Events<Event>) {
        if self.input_pattern.matches(input) {
            events.add(self.event.clone());
        } else {
            element.handle(input, events);
        }
    }
}
