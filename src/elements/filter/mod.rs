//! Filters that can be applied to elements.

use std::fmt::Display;
use std::marker::PhantomData;

use crate::{Cursor, Element, Events, Input, Output, Style, Vec2};

pub use on::*;

mod on;

/// A wrapper around a single element that modifies it.
pub trait Filter<Event> {
    /// Draw the filtered element to the output.
    ///
    /// By default this method forwards to `filter_size`, `write_char`, `set_title` and
    /// `set_cursor`.
    fn draw(&self, element: &dyn Element<Event>, output: &mut dyn Output) {
        struct FilterOutput<'a, T: ?Sized, Event> {
            filter: &'a T,
            output: &'a mut dyn Output,
            event: PhantomData<Event>,
        }
        impl<'a, Event, T: Filter<Event> + ?Sized> Output for FilterOutput<'a, T, Event> {
            fn size(&self) -> Vec2<u16> {
                self.filter.filter_size(self.output.size())
            }
            fn write_char(&mut self, pos: Vec2<u16>, c: char, style: Style) {
                self.filter.write_char(self.output, pos, c, style);
            }
            fn set_title(&mut self, title: &dyn Display) {
                self.filter.set_title(self.output, title);
            }
            fn set_cursor(&mut self, cursor: Option<Cursor>) {
                self.filter.set_cursor(self.output, cursor);
            }
        }
        element.draw(&mut FilterOutput {
            filter: self,
            output,
            event: PhantomData,
        });
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

    /// Get the ideal size of the filtered element.
    ///
    /// By default this calls the element's `ideal_size` method.
    fn ideal_size(&self, element: &dyn Element<Event>, maximum: Vec2<u16>) -> Vec2<u16> {
        element.ideal_size(maximum)
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
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Filtered<T, F> {
    element: T,
    filter: F,
}

impl<T, F> Filtered<T, F> {
    /// Filter an element.
    #[must_use]
    pub fn new(element: T, filter: F) -> Self {
        Self { element, filter }
    }
}

impl<T: Element<Event>, F: Filter<Event>, Event> Element<Event> for Filtered<T, F> {
    fn draw(&self, output: &mut dyn Output) {
        self.filter.draw(&self.element, output)
    }
    fn ideal_size(&self, maximum: Vec2<u16>) -> Vec2<u16> {
        self.filter.ideal_size(&self.element, maximum)
    }
    fn handle(&self, input: Input, events: &mut dyn Events<Event>) {
        self.filter.handle(&self.element, input, events);
    }
}
