use std::cmp::max;

use crate::{Element, Events, Input, Output};

use super::Collection;

/// A simple stack of elements, where each one is drawn on top of one another. Created by the
/// [`stack`](fn.stack.html) function.
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

impl<E, Event> Element<Event> for Stack<E>
where
    for<'a> E: Collection<'a, Event>,
{
    fn draw(&self, output: &mut dyn Output) {
        for element in self.elements.iter() {
            element.draw(output);
        }
    }
    fn width(&self, height: Option<u16>) -> (u16, u16) {
        self.elements
            .iter()
            .map(|element| element.width(height))
            .fold((0, 0), |(min_acc, max_acc), (min_len, max_len)| {
                (max(min_acc, min_len), max(max_acc, max_len))
            })
    }
    fn height(&self, width: Option<u16>) -> (u16, u16) {
        self.elements
            .iter()
            .map(|element| element.height(width))
            .fold((0, 0), |(min_acc, max_acc), (min_len, max_len)| {
                (max(min_acc, min_len), max(max_acc, max_len))
            })
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
}

/// Create a [`Stack`](struct.Stack.html) of elements.
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
///         .on('q', Event::ClosePopup)
///         .float((Alignment::Middle, Alignment::Middle)),
/// ));
/// ```
#[must_use]
pub fn stack<E: for<'a> Collection<'a, Event>, Event>(elements: E) -> Stack<E> {
    Stack {
        elements,
        broadcast_inputs: false,
    }
}
