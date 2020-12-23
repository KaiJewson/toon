use std::fmt;

use crate::events::Events;
use crate::{Element, Input, Output, Vec2};

/// An element that maps the event type of an element, created by the
/// [`map_event`](super::ElementExt::map_event) function.
///
/// This is not implemented as a [`Filter`](super::filter) as filters do not allow changing the
/// event type due to the lack of default associated types in Rust.
///
/// # Examples
///
/// ```
/// use toon::ElementExt;
///
/// let unit_event: toon::Block<()> = toon::fill(toon::Color::Red);
/// let i32_event = unit_event.map_event(|()| 5);
/// ```
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
    fn ideal_width(&self, height: u16, max_width: Option<u16>) -> u16 {
        self.inner.ideal_width(height, max_width)
    }
    fn ideal_height(&self, width: u16, max_height: Option<u16>) -> u16 {
        self.inner.ideal_height(width, max_height)
    }
    fn ideal_size(&self, maximum: Vec2<Option<u16>>) -> Vec2<u16> {
        self.inner.ideal_size(maximum)
    }
    fn handle(&self, input: Input, events: &mut dyn Events<Self::Event>) {
        self.inner.handle(input, &mut events.map(&self.f));
    }
    fn title(&self, title: &mut dyn fmt::Write) -> fmt::Result {
        self.inner.title(title)
    }
}
