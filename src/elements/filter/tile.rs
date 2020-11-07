use std::cmp::max;

use crate::output::{Ext as _, Output};
use crate::{Element, Events, Input, Mouse, Vec2};

use super::Filter;

/// A filter that tiles an element, typically used through the
/// [`tile`](../trait.ElementExt.html#method.tile) and
/// [`tile_with_offset`](../trait.ElementExt.html#method.tile_with_offset) methods.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Tile {
    /// The offsets at which the element is tiled, `None` if the element is not tiled in this axis.
    pub offset: Vec2<Option<u16>>,
}

impl Tile {
    /// Create a new tiling filter with the given offset.
    #[must_use]
    pub fn new(offset: Vec2<Option<u16>>) -> Self {
        Self { offset }
    }
}

impl Tile {
    /// Get the offset and size of the element.
    pub fn layout(self, element: impl Element, output_size: Vec2<u16>) -> (Vec2<u16>, Vec2<u16>) {
        let (width, offset_x) = self.offset.x.map_or((output_size.x, 0), |offset| {
            let width = max(element.width(None).0, 1);
            (width, offset % width)
        });
        let (height, offset_y) = self.offset.y.map_or((output_size.y, 0), |offset| {
            let height = max(element.height(Some(width)).0, 1);
            (height, offset % height)
        });
        (Vec2::new(offset_x, offset_y), Vec2::new(width, height))
    }
}

impl<Event> Filter<Event> for Tile {
    fn draw<E: Element>(&self, element: E, output: &mut dyn Output) {
        let output_size = output.size();
        if output_size.x == 0 || output_size.y == 0 {
            return;
        }

        let (offset, size) = self.layout(&element, output_size);

        let range = Vec2::zip_3_with(offset, size, output_size, |offset, size, output_size| {
            let start = if offset == 0 { 0 } else { -1 };
            let complete_end = i32::from(output_size / size);
            let end = if output_size % size == 0 {
                complete_end
            } else {
                complete_end + 1
            };
            start..end
        });

        for i in range.x.clone() {
            let x = i * i32::from(size.x) + i32::from(offset.x);
            for j in range.y.clone() {
                let y = j * i32::from(size.y) + i32::from(offset.y);

                element.draw(&mut output.area(Vec2 { x, y }, size).on_set_cursor(|_, _| {}));
            }
        }
    }
    fn width<E: Element>(&self, element: E, height: Option<u16>) -> (u16, u16) {
        let (min, max) = element.width(height);
        (
            min,
            if self.offset.x.is_some() {
                u16::MAX
            } else {
                max
            },
        )
    }
    fn height<E: Element>(&self, element: E, width: Option<u16>) -> (u16, u16) {
        let (min, max) = element.height(width);
        (
            min,
            if self.offset.y.is_some() {
                u16::MAX
            } else {
                max
            },
        )
    }
    fn handle<E: Element<Event = Event>>(
        &self,
        element: E,
        input: Input,
        events: &mut dyn Events<Event>,
    ) {
        element.handle(
            match input {
                Input::Key(key) => Input::Key(key),
                Input::Mouse(mouse) => {
                    let (offset, size) = self.layout(&element, mouse.size);

                    Input::Mouse(Mouse {
                        size,
                        at: Vec2::zip_3_with(mouse.at, offset, size, |at, offset, size| {
                            at.checked_sub(offset)
                                .unwrap_or_else(|| at + (size - offset))
                                % size
                        }),
                        ..mouse
                    })
                }
            },
            events,
        );
    }
}

#[cfg(test)]
fn test_el() -> impl Element {
    crate::column::<_, _, ()>(crate::Static, (crate::span("abc"), crate::span("def")))
}

#[test]
fn test_tile_x() {
    use crate::ElementExt;

    let mut grid = crate::Grid::new((12, 3));

    test_el().tile_x(2).draw(&mut grid);
    assert_eq!(
        grid.contents(),
        ["bcabcabcabca", "efdefdefdefd", "            ",]
    );

    test_el().tile_x(3).draw(&mut grid);
    assert_eq!(
        grid.contents(),
        ["abcabcabcabc", "defdefdefdef", "            ",]
    );
}

#[test]
fn test_tile_y() {
    use crate::ElementExt;

    let mut grid = crate::Grid::new((4, 6));

    test_el().tile_y(1).draw(&mut grid);
    assert_eq!(
        grid.contents(),
        ["def ", "abc ", "def ", "abc ", "def ", "abc ",]
    );

    test_el().tile_y(2).draw(&mut grid);
    assert_eq!(
        grid.contents(),
        ["abc ", "def ", "abc ", "def ", "abc ", "def ",]
    );
}

#[test]
fn test_tile_both() {
    use crate::ElementExt;

    let mut grid = crate::Grid::new((13, 7));

    test_el().tile((2, 1)).draw(&mut grid);

    assert_eq!(
        grid.contents(),
        [
            "efdefdefdefde",
            "bcabcabcabcab",
            "efdefdefdefde",
            "bcabcabcabcab",
            "efdefdefdefde",
            "bcabcabcabcab",
            "efdefdefdefde",
        ]
    );
}
