//! Common elements for building user interfaces.
//!
//! This module aims to cover most use cases of elements so you don't have to implement
//! [`Element`](../trait.Element.html) yourself.

use std::fmt::Display;

use crate::{input, Element, Input, Vec2};

pub use containers::*;
pub use filter::*;

pub use block::*;
#[cfg(feature = "dev")]
pub use dev::Dev;
pub use map_event::*;
pub use span::*;

pub mod containers;
#[cfg(feature = "dev")]
pub mod dev;
pub mod filter;

mod block;
mod map_event;
mod span;

/// An extension trait for elements providing useful methods.
pub trait ElementExt: Element + Sized {
    /// Filter this element using the given filter.
    ///
    /// This is a shortcut method for [`Filtered::new`](filter/struct.Filtered.html#method.new).
    #[must_use]
    fn filter<F: Filter<Self::Event>>(self, filter: F) -> Filtered<Self, F> {
        Filtered::new(self, filter)
    }

    /// Trigger an event when an input occurs.
    ///
    /// The created element will listen to inputs _actively_; the input if it occurs will not be
    /// passed to the inner element.
    ///
    /// # Examples
    ///
    /// ```
    /// # use toon::ElementExt;
    /// # let element = toon::empty();
    /// # #[derive(Clone)]
    /// # enum Event { Exit }
    /// // When the 'q' key is pressed or the element is clicked an Exit event will be triggered.
    /// let element = element.on(('q', toon::input!(Mouse(Press Left))), |_| Event::Exit);
    /// ```
    #[must_use]
    fn on<I: input::Pattern, F: Fn(Input) -> Self::Event>(
        self,
        input_pattern: I,
        event: F,
    ) -> Filtered<Self, On<I, F>> {
        self.filter(On::new(input_pattern, event))
    }

    /// Trigger an event when an input occurs, passively; the inner element will still receive
    /// all inputs.
    #[must_use]
    fn on_passive<I: input::Pattern, F: Fn(Input) -> Self::Event>(
        self,
        input_pattern: I,
        event: F,
    ) -> Filtered<Self, On<I, F>> {
        self.filter(On::new(input_pattern, event).passive())
    }

    /// Make the element float in both axes with the given alignment.
    ///
    /// # Example
    ///
    /// Make the element its smallest size at the middle right of the screen.
    ///
    /// ```
    /// use toon::{Alignment, ElementExt};
    ///
    /// # let element = toon::empty::<()>();
    /// let element = element.float((Alignment::End, Alignment::Middle));
    /// ```
    #[must_use]
    fn float(self, align: impl Into<Vec2<Alignment>>) -> Filtered<Self, Float> {
        self.filter(Float::new(align.into().map(Some)))
    }

    /// Make the element in the X axis only with the given alignment.
    #[must_use]
    fn float_x(self, align: Alignment) -> Filtered<Self, Float> {
        self.filter(Float::new(Vec2::new(Some(align), None)))
    }

    /// Make the element in the Y axis only with the given alignment.
    #[must_use]
    fn float_y(self, align: Alignment) -> Filtered<Self, Float> {
        self.filter(Float::new(Vec2::new(None, Some(align))))
    }

    /// Set the title of the element.
    #[must_use]
    fn title<T: Display>(self, title: T) -> Filtered<Self, Title<T>> {
        self.filter(Title::new(title))
    }

    /// Set the minimum width of the element.
    #[must_use]
    fn min_width(self, width: u16) -> Filtered<Self, Size> {
        self.filter(Size {
            min: Vec2::new(Some(width), None),
            max: Vec2::new(None, None),
        })
    }
    /// Set the maximum width of the element.
    #[must_use]
    fn max_width(self, width: u16) -> Filtered<Self, Size> {
        self.filter(Size {
            min: Vec2::new(None, None),
            max: Vec2::new(Some(width), None),
        })
    }
    /// Set the width range (inclusive) of the element.
    #[must_use]
    fn width_range(self, min: u16, max: u16) -> Filtered<Self, Size> {
        self.filter(Size {
            min: Vec2::new(Some(min), None),
            max: Vec2::new(Some(max), None),
        })
    }
    /// Set the minimum height of the element.
    #[must_use]
    fn min_height(self, height: u16) -> Filtered<Self, Size> {
        self.filter(Size {
            min: Vec2::new(None, Some(height)),
            max: Vec2::new(None, None),
        })
    }
    /// Set the maximum height of the element.
    #[must_use]
    fn max_height(self, height: u16) -> Filtered<Self, Size> {
        self.filter(Size {
            min: Vec2::new(None, None),
            max: Vec2::new(None, Some(height)),
        })
    }
    /// Set the height range (inclusive) of the element.
    #[must_use]
    fn height_range(self, min: u16, max: u16) -> Filtered<Self, Size> {
        self.filter(Size {
            min: Vec2::new(None, Some(min)),
            max: Vec2::new(None, Some(max)),
        })
    }
    /// Set the minimum size of the element.
    #[must_use]
    fn min_size(self, size: impl Into<Vec2<u16>>) -> Filtered<Self, Size> {
        self.filter(Size {
            min: size.into().map(Some),
            max: Vec2::new(None, None),
        })
    }
    /// Set the maximum size of the element.
    #[must_use]
    fn max_size(self, size: impl Into<Vec2<u16>>) -> Filtered<Self, Size> {
        self.filter(Size {
            min: Vec2::new(None, None),
            max: size.into().map(Some),
        })
    }
    /// Set the size range (inclusive) of the element.
    #[must_use]
    fn size_range(
        self,
        min: impl Into<Vec2<u16>>,
        max: impl Into<Vec2<u16>>,
    ) -> Filtered<Self, Size> {
        self.filter(Size {
            min: min.into().map(Some),
            max: max.into().map(Some),
        })
    }

    /// Map the type of event produced by the element.
    #[must_use]
    fn map_event<Event2, F: Fn(Self::Event) -> Event2>(self, f: F) -> MapEvent<Self, F> {
        MapEvent { inner: self, f }
    }

    /// Mask the type of inputs that go through to the element according to the pattern.
    ///
    /// # Examples
    ///
    /// Prevent all inputs from reaching an element:
    ///
    /// ```rust
    /// use toon::ElementExt;
    ///
    /// # let element = toon::empty::<()>();
    /// let element = element.mask_inputs(());
    /// ```
    ///
    /// Only give the element mouse inputs:
    ///
    /// ```rust
    /// # use toon::ElementExt;
    /// # let = toon::empty::<()>();
    /// let element = element.mask_inputs(toon::input!(Mouse));
    /// ```
    #[must_use]
    fn mask_inputs<P: input::Pattern>(self, pattern: P) -> Filtered<Self, InputMask<P>> {
        self.filter(InputMask { pattern })
    }

    /// Scroll the element by a certain amount in the X axis.
    #[must_use]
    fn scroll_x(self, x: ScrollOffset) -> Filtered<Self, Scroll> {
        self.filter(Scroll {
            by: Vec2::new(Some(x), None),
        })
    }

    /// Scroll the element by a certain amount in the Y axis.
    #[must_use]
    fn scroll_y(self, y: ScrollOffset) -> Filtered<Self, Scroll> {
        self.filter(Scroll {
            by: Vec2::new(None, Some(y)),
        })
    }

    /// Scroll the element by a certain amount in both axes.
    #[must_use]
    fn scroll(self, by: impl Into<Vec2<ScrollOffset>>) -> Filtered<Self, Scroll> {
        self.filter(Scroll {
            by: by.into().map(Some),
        })
    }

    /// Erase the element's type by boxing it.
    fn boxed<'a>(self) -> Box<dyn Element<Event = Self::Event> + 'a>
    where
        Self: 'a,
    {
        Box::new(self)
    }
}
impl<T: Element> ElementExt for T {}
