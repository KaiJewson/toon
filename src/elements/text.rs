use std::fmt::{Display, Write};

use unicode_width::UnicodeWidthChar;

use crate::{Element, Events, Input, Output, Style, Vec2};

/// A single line of text.
///
/// It takes any type that implements `Display`. If your `Display` impl is costly, you may want to
/// convert it to a string beforehand.
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Text<T> {
    item: T,
    style: Style,
}

impl<T> Text<T> {
    /// Create a line of text with the given style.
    #[must_use]
    pub fn new(item: T, style: Style) -> Self {
        Self { item, style }
    }
}

/// Shortcut function for `Text::new`.
#[must_use]
pub fn text<T>(item: T, style: Style) -> Text<T> {
    Text::new(item, style)
}

impl<T: Display, E> Element<E> for Text<T> {
    fn draw(&self, output: &mut dyn Output) {
        output.write(Vec2::new(0, 0), &self.item, self.style);
    }
    fn ideal_size(&self, _maximum: Vec2<u16>) -> Vec2<u16> {
        let mut width = 0;
        write!(
            crate::util::WriteCharsFn(|c| width += c.width().unwrap_or(0) as u16),
            "{}",
            self.item
        )
        .expect("formatting failed");

        Vec2::new(width, 0)
    }
    fn handle(&self, _input: Input, _events: &mut dyn Events<E>) {}
}
