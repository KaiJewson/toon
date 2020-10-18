use std::fmt::{Display, Write};

use unicode_width::UnicodeWidthChar;

use crate::{Element, Events, Input, Output, Style, Vec2};

/// A single line of text.
///
/// It takes any type that implements `Display`. If your `Display` impl is costly, you may want to
/// convert it to a string beforehand. Otherwise you will probably want to use
/// [`format_args!`](https://doc.rust-lang.org/stable/core/macro.format_args.html) to generate the
/// type since it avoids allocation.
///
/// # Examples
///
/// ```
/// # async {
/// use toon::ElementExt;
/// # let mut terminal = toon::Terminal::new(toon::Crossterm::default())?;
///
/// // Display `Hello World!` until q is pressed
/// terminal.draw(toon::text("Hello World!", toon::Style::default()).on('q', ())).await?;
/// # Ok::<_, <toon::Crossterm as toon::Backend>::Error>(())
/// # };
/// ```
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

/// Create a single line of text.
///
/// Shortcut function for [`Text::new`](struct.Text.html#method.new).
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
