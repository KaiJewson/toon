use crate::{Element, Vec2};

use super::Filter;

/// A filter that gives the element a fixed size ratio, typically used through the
/// [`ratio`](crate::ElementExt::ratio) method.
///
/// This is not an exact calculation; it will be rounded to the nearest cell, so in a small number
/// of cells the result might be unexpected.
///
/// Note that this does not do anything on its own. It does not guarantee that the element will be
/// drawn at this ratio, nor does it affect how the element is drawn in any way. It should be
/// combined with other types, such as [`Float`](super::Float), to make it useful.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ratio {
    /// The width divided by the height of the ideal ratio of the element.
    pub ratio: f64,
}

impl<Event> Filter<Event> for Ratio {
    fn ideal_size<E: Element>(&self, element: E, maximum: Vec2<Option<u16>>) -> Vec2<u16> {
        match maximum {
            Vec2 {
                x: Some(x),
                y: Some(y),
            } => {
                let x_from_y = (f64::from(y) * self.ratio).round() as u16;
                let y_from_x = (f64::from(x) / self.ratio).round() as u16;
                Vec2::new(u16::min(x_from_y, x), u16::min(y_from_x, y))
            }
            Vec2 {
                x: Some(x),
                y: None,
            } => Vec2::new(x, (f64::from(x) / self.ratio).round() as u16),
            Vec2 {
                x: None,
                y: Some(y),
            } => Vec2::new((f64::from(y) * self.ratio).round() as u16, y),
            Vec2 { x: None, y: None } => element.ideal_size(maximum),
        }
    }
    fn ideal_width<E: Element>(&self, _element: E, height: u16, _max_width: Option<u16>) -> u16 {
        (f64::from(height) * self.ratio).round() as u16
    }
    fn ideal_height<E: Element>(&self, _element: E, width: u16, _max_height: Option<u16>) -> u16 {
        (f64::from(width) / self.ratio).round() as u16
    }
}

#[test]
fn test_ratio() {
    use crate::{Alignment::Middle, ElementExt};

    let mut grid = crate::Grid::new((5, 4));

    let a = crate::span::<_, ()>("a").tile((0, 0));

    a.ratio(2.).float((Middle, Middle)).draw(&mut grid);
    assert_eq!(grid.contents(), ["     ", "aaaaa", "aaaaa", "aaaaa"]);

    grid.clear();
    a.ratio(0.5).float((Middle, Middle)).draw(&mut grid);
    assert_eq!(grid.contents(), [" aa  ", " aa  ", " aa  ", " aa  "]);
}
