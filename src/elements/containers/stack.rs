use std::fmt;

use crate::{Element, Events, Input, Output, Vec2};

use super::Collection;

/// A simple stack of elements, where each one is drawn on top of one another. Created by the
/// [`stack`] function.
///
/// To just fill the background of an element, use [`FillBackground`](crate::FillBackground).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Stack<E> {
    /// The elements in this container.
    pub elements: E,
    /// Whether to broadcast inputs to all elements instead of just the top one.
    pub broadcast_inputs: bool,
}

impl<E> Stack<E> {
    /// Broadcast inputs to all elements instead of just the top one.
    #[must_use]
    pub fn broadcast_inputs(self) -> Self {
        Self {
            broadcast_inputs: true,
            ..self
        }
    }
}

impl<E, Event> Element for Stack<E>
where
    for<'a> E: Collection<'a, Event = Event>,
{
    type Event = Event;

    fn draw(&self, output: &mut dyn Output) {
        for element in self.elements.iter() {
            element.draw(output);
        }
    }
    fn ideal_width(&self, height: u16, max_width: Option<u16>) -> u16 {
        self.elements
            .iter()
            .map(|element| element.ideal_width(height, max_width))
            .max()
            .unwrap_or_default()
    }
    fn ideal_height(&self, width: u16, max_height: Option<u16>) -> u16 {
        self.elements
            .iter()
            .map(|element| element.ideal_height(width, max_height))
            .max()
            .unwrap_or_default()
    }
    fn ideal_size(&self, maximum: Vec2<Option<u16>>) -> Vec2<u16> {
        self.elements
            .iter()
            .map(|element| element.ideal_size(maximum))
            .fold(Vec2::default(), Vec2::max)
    }
    fn handle(&self, input: Input, events: &mut dyn Events<Event>) {
        if self.broadcast_inputs {
            for element in self.elements.iter() {
                element.handle(input, events);
            }
        } else if let Some(last) = self.elements.iter().next_back() {
            last.handle(input, events);
        }
    }
    fn title(&self, title: &mut dyn fmt::Write) -> fmt::Result {
        if let Some(last) = self.elements.iter().next_back() {
            last.title(title)?;
        }
        Ok(())
    }
}

/// Create a [`Stack`] of elements.
///
/// By default inputs will not be broadcast to all elements.
///
/// # Examples
///
/// Create a popup over the background element.
///
/// ```
/// # let element = toon::empty();
/// # #[derive(Clone)] enum Event { ClosePopup }
/// use toon::{Alignment, ElementExt};
///
/// let element = toon::stack((
///     element,
///     toon::span("A popup message")
///         .filter(toon::Border::THIN)
///         .float((Alignment::Middle, Alignment::Middle))
///         .on('q', |_| Event::ClosePopup),
/// ));
/// ```
#[must_use]
pub fn stack<E, Event>(elements: E) -> Stack<E>
where
    for<'a> E: Collection<'a, Event = Event>,
{
    Stack {
        elements,
        broadcast_inputs: false,
    }
}

#[test]
fn test_stack() {
    use crate::{Alignment, ElementExt};

    let mut grid = crate::Grid::new((12, 10));

    stack::<_, ()>((
        crate::span("x").tile((0, 0)),
        crate::span("Foo").float((Alignment::Middle, Alignment::Middle)),
    ))
    .draw(&mut grid);

    assert_eq!(
        grid.contents(),
        [
            "xxxxxxxxxxxx",
            "xxxxxxxxxxxx",
            "xxxxxxxxxxxx",
            "xxxxxxxxxxxx",
            "xxxxxxxxxxxx",
            "xxxxxFooxxxx",
            "xxxxxxxxxxxx",
            "xxxxxxxxxxxx",
            "xxxxxxxxxxxx",
            "xxxxxxxxxxxx",
        ]
    );
}
