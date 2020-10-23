use std::fmt::{Display, Write};

use unicode_width::UnicodeWidthChar;

use crate::{Element, Events, Input, Output, Style, Vec2};

/// A span of text, created by the [`span`](fn.span.html) function.
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub struct Span<T> {
    /// The text being displayed.
    pub text: T,
    /// The style to display the text in.
    pub style: Style,
}

impl<T> Span<T> {
    /// Create a line of text with the default style. Note that use of the [`span`](fn.span.html)
    /// function is preferred.
    #[must_use]
    pub fn new(text: T) -> Self {
        Self {
            text,
            style: Style::default(),
        }
    }
}

impl<T> AsRef<Style> for Span<T> {
    fn as_ref(&self) -> &Style {
        &self.style
    }
}
impl<T> AsMut<Style> for Span<T> {
    fn as_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl<T: Display, Event> Element<Event> for Span<T> {
    fn draw(&self, output: &mut dyn Output) {
        let mut pos = 0;

        write!(
            crate::util::WriteCharsFn(|c| {
                let width = match c.width() {
                    Some(width) => width,
                    None => return,
                } as u16;

                output.write_char(Vec2::new(pos, 0), c, self.style);

                pos += width;
            }),
            "{}",
            self.text
        )
        .expect("formatting failed");
    }
    fn width(&self, _height: Option<u16>) -> (u16, u16) {
        let mut width = 0;

        write!(
            crate::util::WriteCharsFn(|c| width += c.width().unwrap_or(0) as u16),
            "{}",
            self.text
        )
        .expect("formatting failed");

        (width, width)
    }
    fn height(&self, _width: Option<u16>) -> (u16, u16) {
        (1, 1)
    }
    fn handle(&self, _input: Input, _events: &mut dyn Events<Event>) {}
}

/// Create a span of text.
///
/// It takes any type that implements `Display`. If your `Display` impl is costly, you may want to
/// convert it to a string beforehand. Otherwise you will probably want to use
/// [`format_args!`](https://doc.rust-lang.org/stable/core/macro.format_args.html) to generate the
/// type since it avoids allocation.
///
/// # Examples
///
/// ```
/// # use toon::Styled;
/// // Display `Hello World!` in bold
/// let element = toon::span("Hello World!").bold();
/// ```
#[must_use]
pub fn span<T: Display>(text: T) -> Span<T> {
    Span::new(text)
}
