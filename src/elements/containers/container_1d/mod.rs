use std::cmp;
use std::fmt::{self, Debug, Formatter};

use crate::output::{Ext as _, Output};
use crate::{Element, Events, Input, Vec2};

use super::Collection;

mod share;
pub use share::{share, End, Share};

mod r#static;
pub use r#static::Static;

mod stretch;
pub use stretch::{stretch, Stretch};

/// A 1-dimensional layout of elements, for use in a [`Container1D`].
pub trait Layout1D<'a, C: Collection<'a>> {
    /// The layout of elements.
    ///
    /// This is an iterator over all the elements and where they are.
    type Layout: Iterator<Item = InnerElement<'a, <C as Collection<'a>>::Event>>;

    /// Get the layout of the elements in the collection.
    fn layout(
        &'a self,
        elements: &'a C,
        main_axis_size: u16,
        cross_axis_size: u16,
        axis: Axis,
    ) -> Self::Layout;
}

/// An element arranged by a [`Layout1D`].
#[derive(Clone, Copy)]
pub struct InnerElement<'a, Event> {
    /// The element itself.
    pub element: &'a dyn Element<Event = Event>,
    /// The index of the element. This is used to determine whether it is focused.
    pub index: usize,
    /// The position of the element along the main axis.
    pub position: u16,
    /// The size of the element along the main axis.
    pub size: u16,
}

impl<'a, Event> Debug for InnerElement<'a, Event> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("InnerElement")
            .field("index", &self.index)
            .field("position", &self.position)
            .field("size", &self.size)
            .finish()
    }
}

/// A 1-dimensional container of elements. It draws a [`Collection`] with a [`Layout1D`], and is
/// created by the [`column()`] and [`row()`] functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Container1D<E, L> {
    /// The elements in the container.
    pub elements: E,
    /// The layout of the container.
    pub layout: L,
    /// The axis of the container.
    pub axis: Axis,
    /// Whether to broadcast key inputs to all elements. If `false`, only the focused element will
    /// receive key inputs.
    pub broadcast_keys: bool,
    /// The index of the focused element of the container. This element will set the title and
    /// cursor of the container, and will receive all inputs if
    /// [`broadcast_keys`](Self::broadcast_keys) is not set.
    ///
    /// It is not an error if this element does not exist.
    pub focused: Option<usize>,
}

impl<E, L> Container1D<E, L> {
    /// Broadcast key inputs to all elements, instead of just the focused one.
    #[must_use]
    pub fn broadcast_keys(self) -> Self {
        Self {
            broadcast_keys: true,
            ..self
        }
    }

    /// Set the focused element of the container.
    ///
    /// This element will set the title and cursor of the container, and will receive all inputs if
    /// `broadcast_keys` is not set.
    ///
    /// It is not an error if this element does not exist.
    #[must_use]
    pub fn focus(self, element: usize) -> Self {
        Self {
            focused: Some(element),
            ..self
        }
    }
}

impl<E, L, Event> Element for Container1D<E, L>
where
    for<'a> E: Collection<'a, Event = Event>,
    for<'a> L: Layout1D<'a, E>,
{
    type Event = Event;

    fn draw(&self, output: &mut dyn Output) {
        let (main_axis_size, cross_axis_size) = self.axis.main_cross_of(output.size());

        for (i, inner) in self
            .layout
            .layout(&self.elements, main_axis_size, cross_axis_size, self.axis)
            .enumerate()
        {
            inner.element.draw(
                &mut output
                    .area(
                        self.axis.vec(i32::from(inner.position), 0),
                        self.axis.vec(inner.size, cross_axis_size),
                    )
                    .on_set_cursor(|output, cursor| {
                        if self.focused == Some(i) {
                            output.set_cursor(cursor);
                        }
                    }),
            );
        }
    }
    fn title(&self, title: &mut dyn fmt::Write) -> fmt::Result {
        if let Some(i) = self.focused {
            if let Some(element) = self.elements.iter().nth(i) {
                element.title(title)?;
            }
        }
        Ok(())
    }
    fn ideal_width(&self, height: u16, max_width: Option<u16>) -> u16 {
        match self.axis {
            Axis::X => self
                .elements
                .iter()
                .map(|element| element.ideal_width(height, None))
                .sum(),
            Axis::Y => self
                .elements
                .iter()
                .map(|element| element.ideal_size(Vec2::new(max_width, None)).x)
                .max()
                .unwrap_or_default(),
        }
    }
    fn ideal_height(&self, width: u16, max_height: Option<u16>) -> u16 {
        match self.axis {
            Axis::X => self
                .elements
                .iter()
                .map(|element| element.ideal_size(Vec2::new(None, max_height)).y)
                .max()
                .unwrap_or_default(),
            Axis::Y => self
                .elements
                .iter()
                .map(|element| element.ideal_height(width, None))
                .sum(),
        }
    }
    fn ideal_size(&self, maximum: Vec2<Option<u16>>) -> Vec2<u16> {
        match self.axis {
            Axis::X => self.elements.iter().fold(Vec2::new(0, 0), |size, element| {
                let element_size = element.ideal_size(Vec2::new(None, maximum.y));
                Vec2::new(size.x + element_size.x, cmp::max(size.y, element_size.y))
            }),
            Axis::Y => self.elements.iter().fold(Vec2::new(0, 0), |size, element| {
                let element_size = element.ideal_size(Vec2::new(maximum.x, None));
                Vec2::new(cmp::max(size.x, element_size.x), size.y + element_size.y)
            }),
        }
    }
    fn handle(&self, input: Input, events: &mut dyn Events<Event>) {
        match input {
            Input::Key(_) if self.broadcast_keys => {
                for element in self.elements.iter() {
                    element.handle(input, events);
                }
            }
            Input::Key(_) => {
                if let Some(element) = self.focused.and_then(|i| self.elements.iter().nth(i)) {
                    element.handle(input, events);
                }
            }
            Input::Mouse(mouse) => {
                let (mouse_main_axis, mouse_cross_axis) = self.axis.main_cross_of(mouse.at);
                let (main_axis_size, cross_axis_size) = self.axis.main_cross_of(mouse.size);

                for inner in
                    self.layout
                        .layout(&self.elements, main_axis_size, cross_axis_size, self.axis)
                {
                    let local_main_axis = mouse_main_axis
                        .checked_sub(inner.position)
                        .filter(|&pos| pos < inner.size);

                    if let Some(local_main_axis) = local_main_axis {
                        let mut mouse = mouse;
                        mouse.at = self.axis.vec(local_main_axis, mouse_cross_axis);
                        mouse.size = self.axis.vec(inner.size, cross_axis_size);
                        inner.element.handle(Input::Mouse(mouse), events);
                        break;
                    }
                }
            }
        }
    }
}

/// Create a row of elements with the specified layout.
///
/// This takes a [`Layout1D`] and a [`Collection`].
///
/// By default keys inputs will not be broadcast to all elements and there will be no focused
/// element.
#[must_use]
pub fn row<E, L, Event>(layout: L, elements: E) -> Container1D<E, L>
where
    for<'a> E: Collection<'a, Event = Event>,
    for<'a> L: Layout1D<'a, E>,
{
    Container1D {
        elements,
        layout,
        axis: Axis::X,
        broadcast_keys: false,
        focused: None,
    }
}

/// Create a column of elements with the specified layout.
///
/// This takes a [`Layout1D`] and a [`Collection`].
///
/// By default keys inputs will not be broadcast to all elements and there will be no focused
/// element.
#[must_use]
pub fn column<E, L, Event>(layout: L, elements: E) -> Container1D<E, L>
where
    for<'a> E: Collection<'a, Event = Event>,
    for<'a> L: Layout1D<'a, E>,
{
    Container1D {
        elements,
        layout,
        axis: Axis::Y,
        broadcast_keys: false,
        focused: None,
    }
}

/// An axis: X or Y.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Axis {
    /// The x axis.
    X,
    /// The y axis.
    Y,
}

impl Axis {
    /// Construct a [`Vec2`] from the main value and the cross value.
    #[must_use]
    pub const fn vec<T>(self, main: T, cross: T) -> Vec2<T> {
        match self {
            Self::X => Vec2 { x: main, y: cross },
            Self::Y => Vec2 { x: cross, y: main },
        }
    }

    /// Get the main axis of the [`Vec2`].
    #[must_use]
    pub fn main_of<T>(self, v: Vec2<T>) -> T {
        self.main_cross_of(v).0
    }

    /// Get the cross axis of the [`Vec2`].
    #[must_use]
    pub fn cross_of<T>(self, v: Vec2<T>) -> T {
        self.main_cross_of(v).1
    }

    /// Get the main and cross axes of the [`Vec2`].
    #[must_use]
    pub fn main_cross_of<T>(self, v: Vec2<T>) -> (T, T) {
        match self {
            Self::X => (v.x, v.y),
            Self::Y => (v.y, v.x),
        }
    }

    /// Get the ideal main axis size of the element from the cross axis size.
    #[must_use]
    pub fn element_size<E: Element>(self, element: E, cross_axis_size: u16) -> u16 {
        match self {
            Self::X => element.ideal_width(cross_axis_size, None),
            Self::Y => element.ideal_height(cross_axis_size, None),
        }
    }
}
