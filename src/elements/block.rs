use crate::{Color, Element, Output, Input, Events};

/// A block of a single color.
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub struct Block {
    /// The color of the block. If set to `None` this block will be transparent.
    pub color: Option<Color>,
}

impl Block {
    /// Create a new block with the given color. Note that use of the [`clear`](fn.clear.html) and
    /// [`fill`](fn.fill.html) functions is preferred.
    #[must_use]
    pub const fn new(color: Option<Color>) -> Self {
        Self {
            color
        }
    }
}

impl<Event> Element<Event> for Block {
    fn draw(&self, output: &mut dyn Output) {
        if let Some(color) = self.color {
            output.clear(color);
        }
    }
    fn width(&self, _height: Option<u16>) -> (u16, u16) {
        (0, u16::MAX)
    }
    fn height(&self, _width: Option<u16>) -> (u16, u16) {
        (0, u16::MAX)
    }
    fn handle(&self, _input: Input, _events: &mut dyn Events<Event>) {}
}

/// Create a block of a single color.
///
/// # Examples
///
/// ```
/// // A red block.
/// let element = toon::fill(toon::Color::Red);
/// ```
#[must_use]
pub fn fill(color: impl Into<Color>) -> Block {
    Block::new(Some(color.into()))
}

/// Create a transparent block.
#[must_use]
pub fn empty() -> Block {
    Block::new(None)
}
