//! Filters that can be applied to elements.
//!
//! Filters implement the [`Filter`] trait. You can apply a filter to an element by creating a
//! [`Filtered`] using the [`filter`](super::ElementExt::filter) method or more specific shortcut
//! methods such as [`on`](super::ElementExt::on).

use std::fmt;
use std::marker::PhantomData;

use crate::output::Output;
use crate::{Cursor, Element, Events, Input, KeyPress, Mouse, Style, Vec2};

mod border;
pub use border::*;

mod fill_background;
pub use fill_background::*;

mod float;
pub use float::*;

mod input_mask;
pub use input_mask::*;

mod on;
pub use on::*;

mod scroll;
pub use scroll::*;

mod size;
pub use size::*;

mod tile;
pub use tile::*;

mod title;
pub use title::*;

/// A wrapper around a single element that modifies it.
pub trait Filter<Event> {
    /// Draw the filtered element to the output.
    ///
    /// By default this method forwards to [`write_char`](Self::write_char) and
    /// [`set_cursor`](Self::set_cursor).
    fn draw<E: Element>(&self, element: E, output: &mut dyn Output) {
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
    /// By default this method filters the parameters with [`filter_char`](Self::filter_char) and
    /// [`filter_style`](Self::filter_style) and then writes it to the output.
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

    /// Set the filtered cursor of the output.
    ///
    /// By default this filters the cursor with [`filter_cursor`](Self::filter_cursor) and then sets
    /// it to the output's cursor.
    fn set_cursor(&self, base: &mut dyn Output, cursor: Option<Cursor>) {
        base.set_cursor(self.filter_cursor(cursor))
    }

    /// Filter the cursor of the output.
    ///
    /// By default this returns the cursor.
    fn filter_cursor(&self, cursor: Option<Cursor>) -> Option<Cursor> {
        cursor
    }

    /// Get filtered title of the element.
    ///
    /// By default this sets the title of the output to the given title.
    ///
    /// # Errors
    ///
    /// This function should always propagate errors from the writer, and returning errors not
    /// created by the writer may result in panics.
    fn title<E: Element>(&self, element: E, title: &mut dyn fmt::Write) -> fmt::Result {
        element.title(title)
    }

    /// Get the ideal width the element takes up given a fixed height and optional maximum width.
    ///
    /// By default this calls the element's [`ideal_width`](Element::ideal_width) method.
    fn ideal_width<E: Element>(&self, element: E, height: u16, max_width: Option<u16>) -> u16 {
        element.ideal_width(height, max_width)
    }

    /// Get the ideal height the element takes up given a fixed width and optional maximum height.
    ///
    /// By default this calls the element's [`ideal_height`](Element::ideal_height) method.
    fn ideal_height<E: Element>(&self, element: E, width: u16, max_height: Option<u16>) -> u16 {
        element.ideal_height(width, max_height)
    }

    /// Get the ideal size the element takes up given an optional maximum size.
    ///
    /// By default this calls the element's [`ideal_size`](Element::ideal_size) method.
    fn ideal_size<E: Element>(&self, element: E, maximum: Vec2<Option<u16>>) -> Vec2<u16> {
        element.ideal_size(maximum)
    }

    /// React to the input and output events if necessary.
    ///
    /// By default this calls [`filter_input`](Self::filter_input) and passes the element that.
    fn handle<E: Element<Event = Event>>(
        &self,
        element: E,
        input: Input,
        events: &mut dyn Events<Event>,
    ) {
        element.handle(self.filter_input(input), events)
    }

    /// Filter inputs given to the wrapped element.
    ///
    /// By default this forwards to [`filter_key_press`](Self::filter_key_press) and
    /// [`filter_mouse`](Self::filter_mouse).
    fn filter_input(&self, input: Input) -> Input {
        match input {
            Input::Key(key) => Input::Key(self.filter_key_press(key)),
            Input::Mouse(mouse) => Input::Mouse(self.filter_mouse(mouse)),
        }
    }

    /// Filter the key input to the element.
    ///
    /// By default this returns the input unchanged.
    fn filter_key_press(&self, input: KeyPress) -> KeyPress {
        input
    }

    /// Filter the mouse input to the element.
    ///
    /// By default this returns the input unchanged.
    fn filter_mouse(&self, input: Mouse) -> Mouse {
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

impl<T: Element, F: Filter<T::Event>> Element for Filtered<T, F> {
    type Event = T::Event;

    fn draw(&self, output: &mut dyn Output) {
        self.filter.draw(&self.element, output)
    }
    fn title(&self, title: &mut dyn fmt::Write) -> fmt::Result {
        self.filter.title(&self.element, title)
    }
    fn ideal_width(&self, height: u16, max_width: Option<u16>) -> u16 {
        self.filter.ideal_width(&self.element, height, max_width)
    }
    fn ideal_height(&self, width: u16, max_height: Option<u16>) -> u16 {
        self.filter.ideal_height(&self.element, width, max_height)
    }
    fn ideal_size(&self, maximum: Vec2<Option<u16>>) -> Vec2<u16> {
        self.filter.ideal_size(&self.element, maximum)
    }
    fn handle(&self, input: Input, events: &mut dyn Events<Self::Event>) {
        self.filter.handle(&self.element, input, events);
    }
}

/// Alignment to the start, middle or end.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Alignment {
    /// Aligned to the start of the container.
    Start,
    /// Aligned to the middle of the container.
    Middle,
    /// Aligned to the end of the container.
    End,
}
