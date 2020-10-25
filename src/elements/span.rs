use std::fmt::{Display, Write};
use std::marker::PhantomData;

use unicode_width::UnicodeWidthChar;

use crate::{
    output::{Ext as _, Output},
    Element, Events, Input, Style,
};

/// A span of text, created by the [`span`](fn.span.html) function.
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub struct Span<T, Event> {
    /// The text being displayed.
    pub text: T,
    /// The style to display the text in.
    pub style: Style,
    event: PhantomData<Event>,
}

impl<T, Event> Span<T, Event> {
    /// Create a line of text with the default style. Note that use of the [`span`](fn.span.html)
    /// function is preferred.
    #[must_use]
    pub fn new(text: T) -> Self {
        Self {
            text,
            style: Style::default(),
            event: PhantomData,
        }
    }
}

impl<T, Event> AsRef<Style> for Span<T, Event> {
    fn as_ref(&self) -> &Style {
        &self.style
    }
}
impl<T, Event> AsMut<Style> for Span<T, Event> {
    fn as_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl<T: Display, Event> Element for Span<T, Event> {
    type Event = Event;

    fn draw(&self, output: &mut dyn Output) {
        output.write((0, 0), &self.text, self.style);
    }
    fn width(&self, _height: Option<u16>) -> (u16, u16) {
        let mut width = 0;

        write!(
            crate::util::WriteCharsFn(|c| {
                width += c.width().unwrap_or(0) as u16;
                Ok(())
            }),
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
/// let element: toon::Span<_, ()> = toon::span("Hello World!").bold();
/// ```
#[must_use]
pub fn span<T: Display, Event>(text: T) -> Span<T, Event> {
    Span::new(text)
}
