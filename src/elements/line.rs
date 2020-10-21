use std::fmt::{Display, Write};

use unicode_width::UnicodeWidthChar;

use crate::{Element, Events, Input, Output, Style, Vec2};

/// A single line of text, created by the [`line`](fn.line.html) function.
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Line<T> {
    item: T,
    style: Style,
}

impl<T> Line<T> {
    /// Create a line of text with the given style. Note that use of the [`line`](fn.line.html)
    /// function is preferred.
    #[must_use]
    pub fn new(item: T, style: Style) -> Self {
        Self { item, style }
    }
}

/// Create a single line of text.
///
/// It takes any type that implements `Display`. If your `Display` impl is costly, you may want to
/// convert it to a string beforehand. Otherwise you will probably want to use
/// [`format_args!`](https://doc.rust-lang.org/stable/core/macro.format_args.html) to generate the
/// type since it avoids allocation.
///
/// # Examples
///
/// ```
/// // Display `Hello World!` in bold
/// let element = toon::line("Hello World!", toon::Style::default().bold());
/// ```
#[must_use]
pub fn line<T: Display>(item: T, style: Style) -> Line<T> {
    Line::new(item, style)
}

impl<T: Display, E> Element<E> for Line<T> {
    fn draw(&self, output: &mut dyn Output) {
        output.write(Vec2::new(0, 0), &self.item, self.style);
    }
    fn width(&self, _height: Option<u16>) -> (u16, u16) {
        let mut width = 0;

        write!(
            crate::util::WriteCharsFn(|c| width += c.width().unwrap_or(0) as u16),
            "{}",
            self.item
        )
        .expect("formatting failed");

        (width, width)
    }
    fn height(&self, _width: Option<u16>) -> (u16, u16) {
        (1, 1)
    }
    fn handle(&self, _input: Input, _events: &mut dyn Events<E>) {}
}
