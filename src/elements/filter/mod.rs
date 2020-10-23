//! Filters that can be applied to elements.
//!
//! Filters implement the [`Filter`](trait.Filter.html) trait. You can apply a filter to an element
//! by creating a [`Filtered`](struct.Filtered.html) using the
//! [`filter`](../trait.ElementExt.html#method.filter) method or more specific shortcut methods
//! such as [`on`](../trait.ElementExt.html#method.on).

use std::fmt::Display;
use std::marker::PhantomData;

use crate::output::Output;
use crate::{Cursor, Element, Events, Input, Style, Vec2};

pub use float::*;
pub use on::*;

mod float;
mod on;

/// A wrapper around a single element that modifies it.
pub trait Filter<Event> {
    /// Draw the filtered element to the output.
    ///
    /// By default this method forwards to `filter_size`, `write_char`, `set_title` and
    /// `set_cursor`.
    fn draw(&self, element: &dyn Element<Event>, output: &mut dyn Output) {
        struct DrawFilterOutput<'a, F: ?Sized, Event> {
            inner: &'a mut dyn Output,
            filter: &'a F,
            event: PhantomData<Event>,
        }
        impl<'a, F: Filter<Event> + ?Sized, Event> Output for DrawFilterOutput<'a, F, Event> {
            fn size(&self) -> Vec2<u16> {
                self.inner.size()
            }
            fn write_char(&mut self, pos: Vec2<u16>, c: char, style: Style) {
                self.filter.write_char(self.inner, pos, c, style);
            }
            fn set_title(&mut self, title: &dyn Display) {
                self.filter.set_title(self.inner, title);
            }
            fn set_cursor(&mut self, cursor: Option<Cursor>) {
                self.filter.set_cursor(self.inner, cursor);
            }
        }

        element.draw(&mut DrawFilterOutput {
            inner: output,
            filter: self,
            event: PhantomData,
        });
    }

    /// Write a single filtered character to the output.
    ///
    /// By default this method filters the parameters with `filter_char` and `filter_style` and then
    /// writes it to the output.
    fn write_char(&self, base: &mut dyn Output, pos: Vec2<u16>, c: char, style: Style) {
        base.write_char(pos, self.filter_char(c), self.filter_style(style));
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
    /// The inner element.
    pub element: T,
    /// The filter applied to the element.
    pub filter: F,
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
