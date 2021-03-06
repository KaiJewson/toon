use std::fmt::{Display, Write};
use std::marker::PhantomData;

use unicode_width::UnicodeWidthChar;

use crate::{
    output::{Ext as _, Output},
    Element, Events, Input, Style, Vec2,
};

/// A span of text, created by the [`span`] function.
///
/// # Examples
///
/// Display black text on a white background:
///
/// ```
/// use toon::Styled;
///
/// let element: toon::Span<_, ()> = toon::span("Hello World").black().on_white();
/// ```
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Span<T, Event> {
    /// The text being displayed.
    pub text: T,
    /// The style to display the text in.
    pub style: Style,
    event: PhantomData<Event>,
}

impl<T: Display, Event> Span<T, Event> {
    /// Get the width of the span.
    pub fn width(&self) -> u16 {
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

        width
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
    fn ideal_width(&self, _height: u16, _max_width: Option<u16>) -> u16 {
        self.width()
    }
    fn ideal_height(&self, _width: u16, _max_height: Option<u16>) -> u16 {
        1
    }
    fn ideal_size(&self, _maximum: Vec2<Option<u16>>) -> Vec2<u16> {
        Vec2::new(self.width(), 1)
    }
    fn handle(&self, _input: Input, _events: &mut dyn Events<Event>) {}
}

/// Create a span of text.
///
/// It takes any type that implements [`Display`]. If your [`Display`] impl is costly, you may want
/// to convert it to a string beforehand. Otherwise you will probably want to use [`format_args!`]
/// to generate the type since it avoids allocation.
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
    Span {
        text,
        style: Style::default(),
        event: PhantomData,
    }
}

#[test]
fn test_span() {
    use crate::Styled;

    let mut grid = crate::Grid::new((3, 2));

    span::<_, ()>("asdf").black().on_white().draw(&mut grid);

    assert_eq!(grid.contents(), ["asd", "   ",]);

    for (top, bottom) in grid.lines()[0].cells().iter().zip(grid.lines()[1].cells()) {
        assert_eq!(top.style().unwrap(), Style::default().black().on_white());
        assert_eq!(bottom.style().unwrap(), Style::default());
    }
}
