use crate::{Vec2, Element, Output, Input, Events};

/// Element for the [`on`](trait.ElementExt.html#method.on) method.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct On<T, E> {
    /// The inner element.
    pub element: T,
    /// The input on which to fire the event.
    pub input: Input,
    /// The event fired when the input occurs.
    pub event: E,
}

impl<T: Element<E>, E: Clone> Element<E> for On<T, E> {
    fn draw(&self, output: &mut dyn Output) {
        self.element.draw(output);
    }
    fn ideal_size(&self, maximum: Vec2<u16>) -> Vec2<u16> {
        self.element.ideal_size(maximum)
    }
    fn handle(&self, input: Input, events: &mut dyn Events<E>) {
        if input == self.input {
            events.add(self.event.clone());
        } else {
            self.element.handle(input, events);
        }
    }
}
