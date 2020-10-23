use crate::{input, Element, Events, Input};

use super::Filter;

/// A filter that triggers an event when an input occurs, typically used through the
/// [`on`](../trait.ElementExt.html#method.on) and
/// [`on_passive`](../trait.ElementExt.html#method.on_passive) methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct On<I, Event> {
    /// The input this filter listens for.
    pub input_pattern: I,
    /// The event fired when the input occurs.
    pub event: Event,
    /// Whether it listens to inputs passively. If `true`, this type will not intercept inputs if
    /// it triggers its event.
    pub passive: bool,
}

impl<I, Event> On<I, Event> {
    /// Create a new filter that triggers the event when the input occurs.
    ///
    /// The created filter will listen to inputs _actively_; the input if it occurs will not be
    /// passed to the inner element.
    #[must_use]
    pub const fn new(input_pattern: I, event: Event) -> Self {
        Self {
            input_pattern,
            event,
            passive: false,
        }
    }

    /// Make the filter listen to events passively.
    #[must_use]
    pub fn passive(self) -> Self {
        Self {
            passive: true,
            ..self
        }
    }
}

impl<I: input::Pattern, Event: Clone> Filter<Event> for On<I, Event> {
    fn handle(&self, element: &dyn Element<Event>, input: Input, events: &mut dyn Events<Event>) {
        let matches = self.input_pattern.matches(input);
        if matches {
            events.add(self.event.clone());
        }
        if self.passive || !matches {
            element.handle(input, events);
        }
    }
}
