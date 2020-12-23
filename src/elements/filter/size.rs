use crate::{Element, Vec2};

use super::Filter;

/// A filter that sets the size of an element.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct Size {
    /// The size of the element, if it's overridden.
    pub size: Vec2<Option<u16>>,
}

impl<Event> Filter<Event> for Size {
    fn ideal_width<E: Element>(&self, element: E, height: u16, max_width: Option<u16>) -> u16 {
        self.size
            .x
            .unwrap_or_else(|| element.ideal_width(height, max_width))
    }
    fn ideal_height<E: Element>(&self, element: E, width: u16, max_height: Option<u16>) -> u16 {
        self.size
            .y
            .unwrap_or_else(|| element.ideal_height(width, max_height))
    }
    fn ideal_size<E: Element>(&self, element: E, maximum: Vec2<Option<u16>>) -> Vec2<u16> {
        if let Vec2 {
            x: Some(x),
            y: Some(y),
        } = self.size
        {
            // Avoid getting the element's size if we can
            Vec2::new(x, y)
        } else {
            let element_size = element.ideal_size(maximum);
            Vec2::new(
                self.size.x.unwrap_or(element_size.x),
                self.size.y.unwrap_or(element_size.y),
            )
        }
    }
}
