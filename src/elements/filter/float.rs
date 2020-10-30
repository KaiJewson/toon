use crate::output::{Ext as _, Output};
use crate::{Element, Events, Input, Mouse, Vec2};

use super::{Alignment, Filter};

/// A filter that makes an element float, typically used through the
/// [`float`](../trait.ElementExt.html#method.float) method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Float {
    /// The horizontal and vertical alignment of the floating element. If `None`, the element will
    /// not float in that axis.
    pub align: Vec2<Option<Alignment>>,
}

impl Float {
    /// Create a new float filter from the given alignment.
    #[must_use]
    pub fn new(align: Vec2<Option<Alignment>>) -> Self {
        Self { align }
    }

    /// Get the offset and size of the element.
    fn calculate_layout(
        self,
        element: impl Element,
        output_size: Vec2<u16>,
    ) -> (Vec2<u16>, Vec2<u16>) {
        let size = match (self.align.x, self.align.y) {
            (Some(_), Some(_)) => {
                let width = element.width(None).0;
                let height = element.height(Some(width)).0;
                Vec2::new(width, height)
            }
            (Some(_), None) => {
                let width = element.width(Some(output_size.y)).0;
                Vec2::new(width, output_size.y)
            }
            (None, Some(_)) => {
                let height = element.height(Some(output_size.x)).0;
                Vec2::new(output_size.x, height)
            }
            (None, None) => output_size,
        };
        let size = Vec2::min(size, output_size);

        let offset = self
            .align
            .zip(size)
            .zip(output_size)
            .map(|((align, size), total_size)| match align {
                Some(Alignment::Start) | None => 0,
                Some(Alignment::Middle) => (total_size / 2).saturating_sub(size / 2),
                Some(Alignment::End) => total_size.saturating_sub(size),
            });

        (offset, size)
    }
}

impl<Event> Filter<Event> for Float {
    fn draw<E: Element>(&self, element: E, output: &mut dyn Output) {
        let (offset, size) = self.calculate_layout(&element, output.size());

        element.draw(&mut output.area(offset, size));
    }
    fn width<E: Element>(&self, element: E, height: Option<u16>) -> (u16, u16) {
        let width = element.width(height);
        (
            width.0,
            if self.align.x.is_some() {
                u16::MAX
            } else {
                width.1
            },
        )
    }
    fn height<E: Element>(&self, element: E, width: Option<u16>) -> (u16, u16) {
        let height = element.height(width);
        (
            height.0,
            if self.align.y.is_some() {
                u16::MAX
            } else {
                height.1
            },
        )
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
                    .checked_sub(offset)
                    .filter(|&at| at.x < size.x && at.y < size.y)
                    .map(|at| Input::Mouse(Mouse { at, size, ..mouse }))
            }
        };
        if let Some(input) = input {
            element.handle(input, events);
        }
    }
}
