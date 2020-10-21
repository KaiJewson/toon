//! Filters that can be applied to elements.
//!
//! Filters implement the [`Filter`](trait.Filter.html) trait. You can apply a filter to an element
//! by creating a [`Filtered`](struct.Filtered.html) using the
//! [`filter`](../trait.ElementExt.html#method.filter) method or more specific shortcut methods
//! such as [`on`](../trait.ElementExt.html#method.on).

use std::fmt::Display;

use crate::{input, Cursor, Element, Events, Input, Output, Style, Vec2};

pub use on::*;

mod on;

/// An extension trait for elements providing useful methods.
pub trait ElementExt<Event>: Element<Event> + Sized {
    /// Filter this element using the given filter.
    ///
    /// This is a shortcut method for [`Filtered::new`](struct.Filtered.html#method.new).
    fn filter<F: Filter<Event>>(self, filter: F) -> Filtered<Self, F> {
        Filtered::new(self, filter)
    }

    /// Trigger an event when an input occurs.
    ///
    /// # Examples
    ///
    /// ```
    /// # let element = toon::line("");
    /// # enum Event { Exit }
    /// // When the 'q' key is pressed or the element is clicked an Exit event will be triggered.
    /// let element = element.on(('q', toon::MouseButton::Left), Event::Exit);
    /// ```
    fn on<I: input::Pattern>(self, input_pattern: I, event: Event) -> Filtered<Self, On<I, Event>>
    where
        Event: Clone,
    {
        self.filter(On::new(input_pattern, event))
    }

    /// Erase the element's type by boxing it.
    fn boxed<'a>(self) -> Box<dyn Element<Event> + 'a>
    where
        Self: 'a,
    {
        Box::new(self)
    }
}

impl<Event, T: Element<Event>> ElementExt<Event> for T {}

/// A wrapper around a single element that modifies it.
pub trait Filter<Event> {
    /// Draw the filtered element to the output.
    ///
    /// By default this method forwards to `filter_size`, `write_char`, `set_title` and
    /// `set_cursor`.
    fn draw(&self, element: &dyn Element<Event>, output: &mut dyn Output) {
        element.draw(&mut crate::output_with(
            output,
            |output| self.filter_size(output.size()),
            |output, pos, c, style| self.write_char(*output, pos, c, style),
            |output, title| self.set_title(*output, title),
            |output, cursor| self.set_cursor(*output, cursor),
        ));
    }

    /// Filter the size of the output.
    ///
    /// By default this method returns the original size.
    fn filter_size(&self, original: Vec2<u16>) -> Vec2<u16> {
        original
    }

    /// Write a single filtered character to the output.
    ///
    /// By default this method filters the parameters with `filter_write_char` and then writes it
    /// to the output.
    fn write_char(&self, base: &mut dyn Output, pos: Vec2<u16>, c: char, style: Style) {
        let (pos, c, style) = self.filter_write_char(pos, c, style);
        base.write_char(pos, c, style);
    }

    /// Filter a character to be written to the output.
    ///
    /// By default this method forwards to `filter_char` and `filter_style`, keeping the position
    /// the same.
    fn filter_write_char(&self, pos: Vec2<u16>, c: char, style: Style) -> (Vec2<u16>, char, Style) {
        (pos, self.filter_char(c), self.filter_style(style))
    }

    /// Filter the value of a character being written to the output.
    ///
    /// By default this returns the character.
    fn filter_char(&self, c: char) -> char {
        c
    }

    /// Filter the style of a character being written to the output.
    ///
    /// By default this returns the style.
    fn filter_style(&self, style: Style) -> Style {
        style
    }

    /// Set the filtered title of the output.
    ///
    /// By default this sets the title of the output to the given title.
    fn set_title(&self, base: &mut dyn Output, title: &dyn Display) {
        base.set_title(title)
    }

    /// Set the filtered cursor of the output.
    ///
    /// By default this filters the cursor with `filter_cursor` and then sets it to the output's
    /// cursor.
    fn set_cursor(&self, base: &mut dyn Output, cursor: Option<Cursor>) {
        base.set_cursor(self.filter_cursor(cursor))
    }

    /// Filter the cursor of the output.
    ///
    /// By default this returns the cursor.
    fn filter_cursor(&self, cursor: Option<Cursor>) -> Option<Cursor> {
        cursor
    }

    /// Get the inclusive range of widths the element can take up given an optional fixed height.
    ///
    /// By default this calls the element's `width` method.
    fn width(&self, element: &dyn Element<Event>, height: Option<u16>) -> (u16, u16) {
        element.width(height)
    }

    /// Get the inclusive range of heights the element can take up given an optional fixed width.
    ///
    /// By default this calls the element's `height` method.
    fn height(&self, element: &dyn Element<Event>, width: Option<u16>) -> (u16, u16) {
        element.height(width)
    }

    /// React to the input and output events if necessary.
    ///
    /// By default this calls `filter_input` and passes the element that.
    fn handle(&self, element: &dyn Element<Event>, input: Input, events: &mut dyn Events<Event>) {
        element.handle(self.filter_input(input), events)
    }

    /// Filter inputs given to the wrapped element.
    ///
    /// By default this returns the input unchanged.
    fn filter_input(&self, input: Input) -> Input {
        input
    }
}

/// An element with a filter applied.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Filtered<T, F> {
    element: T,
    filter: F,
}

impl<T, F> Filtered<T, F> {
    /// Filter an element.
    #[must_use]
    pub const fn new(element: T, filter: F) -> Self {
        Self { element, filter }
    }
}

impl<T: Element<Event>, F: Filter<Event>, Event> Element<Event> for Filtered<T, F> {
    fn draw(&self, output: &mut dyn Output) {
        self.filter.draw(&self.element, output)
    }
    fn width(&self, height: Option<u16>) -> (u16, u16) {
        self.filter.width(&self.element, height)
    }
    fn height(&self, width: Option<u16>) -> (u16, u16) {
        self.filter.height(&self.element, width)
    }
    fn handle(&self, input: Input, events: &mut dyn Events<Event>) {
        self.filter.handle(&self.element, input, events);
    }
}
