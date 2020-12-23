use crate::{Color, Element, Output, Style, Styled, Vec2};

use super::Filter;

/// A filter that sets the background of an element, typically used through the
/// [`fill_background`](crate::ElementExt::fill_background) method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FillBackground {
    /// The color of the background.
    pub color: Color,
}

impl<Event> Filter<Event> for FillBackground {
    fn draw<E: Element>(&self, element: E, output: &mut dyn Output) {
        let size = output.size();
        for x in 0..size.x {
            for y in 0..size.y {
                output.write_char(
                    Vec2::new(x, y),
                    ' ',
                    Style::default().background(self.color),
                );
            }
        }
        element.draw(output);
    }
}

#[test]
fn test_fill_background() {
    use crate::ElementExt;

    let mut grid = crate::Grid::new((5, 2));

    crate::stack::<_, ()>((
        crate::span('x').tile((0, 0)),
        crate::span("Hi").fill_background(Color::Red),
    ))
    .draw(&mut grid);

    assert_eq!(grid.contents(), ["Hi   ", "     "]);
}
