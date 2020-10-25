use crate::output::{Ext as _, Output};
use crate::{Element, Events, Input, Mouse, Vec2};

use super::{Alignment, Filter};

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

    /// Get the offset and size of the element.
    fn calculate_layout(
        self,
        element: impl Element,
        output_size: Vec2<u16>,
    ) -> (Vec2<u16>, Vec2<u16>) {
        let width = element.width(None).0;
        let height = element.height(Some(width)).0;
        let size = Vec2::min(Vec2::new(width, height), output_size);

        let offset = self
            .align
            .zip(size)
            .zip(output_size)
            .map(|((align, size), total_size)| match align {
                Alignment::Start => 0,
                Alignment::Middle => (total_size / 2).saturating_sub(size / 2),
                Alignment::End => total_size.saturating_sub(size),
            });

        (offset, size)
    }
}

impl<Event> Filter<Event> for Float {
    fn draw<E: Element>(&self, element: E, output: &mut dyn Output) {
        let (offset, size) = self.calculate_layout(&element, output.size());

        element.draw(&mut output.area(offset, size));
    }
    fn handle<E: Element<Event = Event>>(
        &self,
        element: E,
        input: Input,
        events: &mut dyn Events<Event>,
    ) {
        let input = match input {
            Input::Key(key) => Some(Input::Key(key)),
            Input::Mouse(mouse) => {
                let (offset, size) = self.calculate_layout(&element, mouse.size);

                mouse
                    .at
                    .zip(offset)
                    .map(|(at, offset)| at.checked_sub(offset))
                    .both_some()
                    .filter(|&at| at.x < size.x && at.y < size.y)
                    .map(|at| Input::Mouse(Mouse { at, size, ..mouse }))
            }
        };
        if let Some(input) = input {
            element.handle(input, events);
        }
    }
}
