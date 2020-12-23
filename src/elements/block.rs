use std::marker::PhantomData;

use crate::{Attributes, Color, Element, Events, Input, Output, Style, Vec2};

/// A block of a single color.
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Block<Event> {
    /// The color of the block. If set to [`None`] this block will be transparent.
    pub color: Option<Color>,
    event: PhantomData<Event>,
}

impl<Event> Block<Event> {
    /// Create a new block with the given color. Note that use of the [`empty`] and [`fill`]
    /// functions is preferred.
    #[must_use]
    pub const fn new(color: Option<Color>) -> Self {
        Self {
            color,
            event: PhantomData,
        }
    }
}

impl<Event> Element for Block<Event> {
    type Event = Event;

    fn draw(&self, output: &mut dyn Output) {
        if let Some(color) = self.color {
            let size = output.size();
            let style = Style::new(Color::default(), color, Attributes::default());

            for x in 0..size.x {
                for y in 0..size.y {
                    output.write_char(Vec2 { x, y }, ' ', style);
                }
            }
        }
    }
    fn ideal_width(&self, _height: u16, _max_width: Option<u16>) -> u16 {
        0
    }
    fn ideal_height(&self, _width: u16, _max_height: Option<u16>) -> u16 {
        0
    }
    fn ideal_size(&self, _maximum: Vec2<Option<u16>>) -> Vec2<u16> {
        Vec2::new(0, 0)
    }
    fn handle(&self, _input: Input, _events: &mut dyn Events<Event>) {}
}

/// Create a block of a single color.
///
/// # Examples
///
/// ```
/// // A red block.
/// let element: toon::Block<()> = toon::fill(toon::Color::Red);
/// ```
#[must_use]
pub fn fill<C: Into<Color>, Event>(color: C) -> Block<Event> {
    Block::new(Some(color.into()))
}

/// Create a transparent block.
#[must_use]
pub fn empty<Event>() -> Block<Event> {
    Block::new(None)
}

#[test]
fn test_block() {
    use crate::Styled;

    let mut grid = crate::Grid::new((5, 10));

    fill::<_, ()>(Color::Red).draw(&mut grid);

    for line in grid.lines() {
        for cell in line.cells() {
            assert_eq!(
                cell.kind(),
                crate::CellKind::Char {
                    contents: " ",
                    double: false,
                    style: Style::default().on_red(),
                },
            );
        }
    }

    let old_grid = grid.clone();
    empty::<()>().draw(&mut grid);
    assert_eq!(grid, old_grid);
}
