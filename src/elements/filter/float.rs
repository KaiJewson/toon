use crate::output::{Ext as _, Output};
use crate::{Element, Vec2};

use super::Filter;

/// A filter that makes an element float, typically used through the
/// [`float`](../trait.ElementExt.html#method.float) method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Float {
    /// The horizontal and vertical alignment of the floating element.
    pub align: Vec2<Alignment>,
}

impl Float {
    /// Create a new float filter from the given alignment.
    #[must_use]
    pub fn new(align: impl Into<Vec2<Alignment>>) -> Self {
        Self {
            align: align.into(),
        }
    }
}

/// Alignment to the start, middle or end.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Alignment {
    /// Aligned to the start of the container.
    Start,
    /// Aligned to the middle of the container.
    Middle,
    /// Aligned to the end of the container.
    End,
}

impl<Event> Filter<Event> for Float {
    fn draw<E: Element>(&self, element: &E, output: &mut dyn Output) {
        let width = element.width(None).0;
        let height = element.height(Some(width)).0;
        let size = Vec2::min(Vec2::new(width, height), output.size());

        let offset =
            self.align.zip(size).zip(output.size()).map(
                |((align, size), total_size)| match align {
                    Alignment::Start => 0,
                    Alignment::Middle => total_size / 2 - size / 2,
                    Alignment::End => total_size - size,
                },
            );

        element.draw(&mut output.area(offset, size));
    }
}
