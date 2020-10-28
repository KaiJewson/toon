use std::cmp;

use crate::{Element, Vec2};

use super::Filter;

/// A filter that sets the size of an element.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct Size {
    /// The minimum size of the element, if it's overridden.
    pub min: Vec2<Option<u16>>,
    /// The maximum size of the element, if it's overridden.
    pub max: Vec2<Option<u16>>,
}

impl<Event> Filter<Event> for Size {
    fn width<E: Element>(&self, element: E, height: Option<u16>) -> (u16, u16) {
        if let (Some(min), Some(max)) = (self.min.x, self.max.x) {
            // Avoid getting the element's width if we can
            (min, max)
        } else {
            let (min, max) = element.width(height);
            let min = self.min.x.unwrap_or(min);
            (min, cmp::max(self.max.x.unwrap_or(max), min))
        }
    }

    fn height<E: Element>(&self, element: E, width: Option<u16>) -> (u16, u16) {
        if let (Some(min), Some(max)) = (self.min.y, self.max.y) {
            // Avoid getting the element's height if we can
            (min, max)
        } else {
            let (min, max) = element.height(width);
            let min = self.min.y.unwrap_or(min);
            (min, cmp::max(self.max.y.unwrap_or(max), min))
        }
    }
}
