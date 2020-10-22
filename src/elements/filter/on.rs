use crate::{input, Element, Events, Input};

use super::Filter;

/// A filter that triggers an event when an input occurs, created by the
/// [`on`](../trait.ElementExt.html#method.on) function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct On<I, Event> {
    /// The input this filter listens for.
    pub input_pattern: I,
    /// The event fired when the input occurs.
    pub event: Event,
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
