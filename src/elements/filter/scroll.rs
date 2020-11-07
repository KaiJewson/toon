use std::cmp::{max, min};

use crate::output::{Ext as _, Output};
use crate::{Element, Events, Input, Mouse, Vec2};

use super::Filter;

/// A filter that scrolls an element, typically used through the
/// [`scroll_x`](../trait.ElementExt.html#method.scroll_x),
/// [`scroll_y`](../trait.ElementExt.html#method.scroll_y) and
/// [`scroll`](../trait.ElementExt.html#method.scroll) methods.
///
/// This is the opposite of [`Float`](struct.Float.html); instead of drawing the element to smaller
/// viewport than the output it draws the element to a larger viewport.
///
/// Note that this is a super-basic container: it doesn't have scroll wheel support or draw a
/// scroll bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Scroll {
    /// How much to scroll by. If `None`, the element will not scroll in that dimension.
    pub by: Vec2<Option<ScrollOffset>>,
}

impl Scroll {
    /// Get the element size and absolute scroll offset of the element.
    fn layout(self, element: impl Element, output_size: Vec2<u16>) -> (Vec2<u16>, Vec2<u16>) {
        let (element_width, offset_x) = self.by.x.map_or((output_size.x, 0), |offset| {
            let element_width = max(element.width(None).0, output_size.x);
            let maximum_offset = element_width - output_size.x;
            let offset = match offset {
                ScrollOffset::Start(offset) => min(offset, maximum_offset),
                ScrollOffset::End(end) => maximum_offset.saturating_sub(end),
            };
            (element_width, offset)
        });
        let (element_height, offset_y) = self.by.y.map_or((output_size.x, 0), |offset| {
            let element_height = max(element.height(None).0, output_size.y);
            let maximum_offset = element_height - output_size.y;
            let offset = match offset {
                ScrollOffset::Start(offset) => min(offset, maximum_offset),
                ScrollOffset::End(end) => maximum_offset.saturating_sub(end),
            };
            (element_height, offset)
        });
        (
            Vec2::new(element_width, element_height),
            Vec2::new(offset_x, offset_y),
        )
    }
}

impl<Event> Filter<Event> for Scroll {
    fn draw<E: Element>(&self, element: E, output: &mut dyn Output) {
        let (element_size, offset) = self.layout(&element, output.size());

        element.draw(&mut output.area(-offset.map(i32::from), element_size));
    }
    fn width<E: Element>(&self, element: E, height: Option<u16>) -> (u16, u16) {
        if self.by.x.is_some() {
            (1, element.width(height).1)
        } else {
            element.width(height)
        }
    }
    fn height<E: Element>(&self, element: E, width: Option<u16>) -> (u16, u16) {
        if self.by.y.is_some() {
            (1, element.height(width).1)
        } else {
            element.height(width)
        }
    }
    fn handle<E: Element<Event = Event>>(
        &self,
        element: E,
        input: Input,
        events: &mut dyn Events<Event>,
    ) {
        element.handle(
            match input {
                Input::Mouse(mouse) => {
                    let (element_size, offset) = self.layout(&element, mouse.size);

                    Input::Mouse(Mouse {
                        at: mouse.at + offset,
                        size: element_size,
                        ..mouse
                    })
                }
                Input::Key(_) => input,
            },
            events,
        );
    }
}

/// How much to scroll.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::module_name_repetitions)]
pub enum ScrollOffset {
    /// Scroll from the start of the container.
    Start(u16),
    /// Scroll from the end of the container.
    End(u16),
}
