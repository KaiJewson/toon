use std::cmp;

use crate::output::{Ext as _, Output};
use crate::{Element, Events, Input, Mouse, Vec2};

use super::{Alignment, Filter};

/// A filter that makes an element float, typically used through the
/// [`float`](crate::ElementExt::float) method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Float {
    /// The horizontal and vertical alignment of the floating element. If [`None`], the element will
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
            (Some(_), Some(_)) => element.ideal_size(output_size.map(Some)),
            (Some(_), None) => Vec2::new(
                cmp::min(
                    element.ideal_width(output_size.y, Some(output_size.x)),
                    output_size.x,
                ),
                output_size.y,
            ),
            (None, Some(_)) => Vec2::new(
                output_size.x,
                cmp::min(
                    element.ideal_height(output_size.x, Some(output_size.y)),
                    output_size.y,
                ),
            ),
            (None, None) => output_size,
        };
        let size = Vec2::min(size, output_size);

        let offset =
            Vec2::zip_3_with(
                self.align,
                size,
                output_size,
                |align, size, total_size| match align {
                    Some(Alignment::Start) | None => 0,
                    Some(Alignment::Middle) => (total_size / 2).saturating_sub(size / 2),
                    Some(Alignment::End) => total_size.saturating_sub(size),
                },
            );

        (offset, size)
    }
}

impl<Event> Filter<Event> for Float {
    fn draw<E: Element>(&self, element: E, output: &mut dyn Output) {
        let (offset, size) = self.calculate_layout(&element, output.size());

        element.draw(&mut output.area(offset.map(i32::from), size));
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

#[test]
fn test_float_x() {
    use crate::ElementExt;

    let mut grid = crate::Grid::new((10, 3));

    crate::span::<_, ()>("Foo")
        .tile_y(0)
        .float_x(Alignment::Middle)
        .draw(&mut grid);

    assert_eq!(grid.contents(), ["    Foo   "; 3]);

    grid.resize_width(2);
    grid.clear();
    crate::span::<_, ()>("Foo")
        .tile_y(0)
        .float_x(Alignment::Middle)
        .draw(&mut grid);

    assert_eq!(grid.contents(), ["Fo"; 3]);
}

#[test]
fn test_float_y() {
    use crate::ElementExt;

    let mut grid = crate::Grid::new((2, 8));

    crate::span::<_, ()>("X")
        .tile((0, 0))
        .height(2)
        .float_y(Alignment::Middle)
        .draw(&mut grid);

    assert_eq!(
        grid.contents(),
        ["  ", "  ", "  ", "XX", "XX", "  ", "  ", "  ",]
    );

    grid.clear();
    crate::span::<_, ()>("X")
        .tile((0, 0))
        .height(2)
        .float_y(Alignment::End)
        .draw(&mut grid);

    assert_eq!(
        grid.contents(),
        ["  ", "  ", "  ", "  ", "  ", "  ", "XX", "XX",]
    );
}
