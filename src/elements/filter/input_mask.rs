use crate::{input, Element, Input, Events};

use super::Filter;

/// A type that masks the types of inputs that can go through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputMask<P> {
    /// The pattern of inputs to allow.
    pub pattern: P
}

impl<P: input::Pattern, Event> Filter<Event> for InputMask<P> {
    fn handle<E: Element<Event = Event>>(
        &self,
        element: E,
        input: Input,
        events: &mut dyn Events<Event>,
    ) {
        if self.pattern.matches(input) {
            element.handle(input, events);
        }
    }
}
