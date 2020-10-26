use crate::{Element, Output, Input};
use crate::events::{Events, Ext as _};

/// An element that maps the event type of an element, created by the
/// [`map_event`](trait.ElementExt.html#method.map_event) function.
///
/// This is not implemented as a [`Filter`](filter/index.html) as filters do not allow changing the
/// event type due to the lack of default associated types in Rust.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapEvent<T, F> {
    /// The inner element.
    pub inner: T,
    /// The function that maps the event.
    pub f: F,
}

impl<T: Element, F: Fn(T::Event) -> Event2, Event2> Element for MapEvent<T, F> {
    type Event = Event2;

    fn draw(&self, output: &mut dyn Output) {
        self.inner.draw(output)
    }
    fn width(&self, height: Option<u16>) -> (u16, u16) {
        self.inner.width(height)
    }
    fn height(&self, width: Option<u16>) -> (u16, u16) {
        self.inner.height(width)
    }
    fn handle(&self, input: Input, events: &mut dyn Events<Self::Event>) {
        self.inner.handle(input, &mut events.map(&self.f));
    }
}
