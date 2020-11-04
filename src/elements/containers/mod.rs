//! Containers for several elements.

use std::cmp::max;
use std::fmt::{self, Debug, Formatter};
use std::iter;
use std::slice;

#[cfg(feature = "either")]
use either_crate::Either;

use crate::output::{Ext as _, Output};
use crate::{Element, Events, Input, Vec2};

pub use flow::*;
pub use stack::*;
pub use stretch::*;

mod flow;
mod stack;
mod stretch;

/// A collection of elements, held by containers.
///
/// This trait is implemented for vectors of elements and tuples of elements (which can be
/// different types).
///
/// Note: This trait does not work in all scenarios, it is not implemented for vectors of non
/// static elements. I really don't know why.
pub trait Collection<'a> {
    /// The events of the elements in the collection.
    type Event: 'a;

    /// An iterator over the collection.
    type Iter: Iterator<Item = &'a dyn Element<Event = Self::Event>> + DoubleEndedIterator + 'a;

    /// Iterate over the collection.
    fn iter(&'a self) -> Self::Iter;

    /// Get the number of elements in the collection.
    fn len(&'a self) -> usize {
        self.iter().count()
    }

    /// Get whether the collection is empty.
    fn is_empty(&'a self) -> bool {
        self.len() == 0
    }
}

impl<'a, 'b, T: Collection<'a> + ?Sized> Collection<'a> for &'b T {
    type Event = T::Event;
    type Iter = <T as Collection<'a>>::Iter;

    fn iter(&'a self) -> Self::Iter {
        (**self).iter()
    }
    fn len(&'a self) -> usize {
        (**self).len()
    }
}

impl<'a, E: Element + 'a> Collection<'a> for Vec<E> {
    type Event = E::Event;
    #[allow(clippy::type_complexity)]
    type Iter = iter::Map<slice::Iter<'a, E>, fn(&'a E) -> &'a dyn Element<Event = Self::Event>>;

    fn iter(&'a self) -> Self::Iter {
        (**self).iter().map(|element| element)
    }

    fn len(&'a self) -> usize {
        self.len()
    }
}

macro_rules! tupiter {
    () => { iter::Empty<&'a dyn Element<Event = Self::Event>> };
    ($x:ty,) => { iter::Once<&'a dyn Element<Event = Self::Event>> };
    ($x:ty, $($xs:ty,)*) => {
        iter::Chain<
            tupiter!($x,),
            tupiter!($($xs,)*),
        >
    };
}

macro_rules! create_tupiter {
    () => { iter::empty::<&'a dyn Element<Event = Self::Event>>() };
    ($x:expr,) => { iter::once::<&'a dyn Element<Event = Self::Event>>($x) };
    ($x:expr, $($xs:expr,)*) => {
        Iterator::chain(
            create_tupiter!($x,),
            create_tupiter!($($xs,)*),
        )
    };
}

macro_rules! tuple_len {
    () => { 0 };
    ($x:ty,) => { 1 };
    ($x:ty, $($xs:ty,)*) => { 1 + tuple_len!($($xs,)*) };
}

macro_rules! impl_collection_for_tuples {
    ($(($first:ident, $($param:ident),*),)*) => {$(
        impl<'a, $first, $($param,)*> Collection<'a> for ($first, $($param,)*)
        where
            $first: Element,
            <$first as Element>::Event: 'a,
            $($param: Element<Event = <$first as Element>::Event>,)*
        {
            type Event = <$first as Element>::Event;
            type Iter = tupiter!($first, $($param,)*);

            fn iter(&'a self) -> Self::Iter {
                #[allow(non_snake_case)]
                let ($first, $($param,)*) = self;
                create_tupiter!($first, $($param,)*)
            }

            fn len(&'a self) -> usize {
                tuple_len!($first, $($param,)*)
            }
        }
    )*}
}

impl_collection_for_tuples! {
    (A,),
    (A, B),
    (A, B, C),
    (A, B, C, D),
    (A, B, C, D, E),
    (A, B, C, D, E, F),
    (A, B, C, D, E, F, G),
    (A, B, C, D, E, F, G, H),
    (A, B, C, D, E, F, G, H, I),
    (A, B, C, D, E, F, G, H, I, J),
    (A, B, C, D, E, F, G, H, I, J, K),
}

#[cfg(feature = "either")]
impl<'a, L, R> Collection<'a> for Either<L, R>
where
    L: Collection<'a>,
    R: Collection<'a, Event = <L as Collection<'a>>::Event>,
{
    type Event = <L as Collection<'a>>::Event;
    type Iter = Either<<L as Collection<'a>>::Iter, <R as Collection<'a>>::Iter>;

    fn iter(&'a self) -> Self::Iter {
        match self {
            Self::Left(l) => Either::Left(l.iter()),
            Self::Right(r) => Either::Right(r.iter()),
        }
    }
    fn len(&'a self) -> usize {
        match self {
            Self::Left(l) => l.len(),
            Self::Right(r) => r.len(),
        }
    }
}

#[allow(clippy::extra_unused_lifetimes)]
#[allow(dead_code)]
fn test_collection_implementors<'a>() {
    fn assert_is_collection<T: for<'a> Collection<'a>>() {}
    type Element = crate::Block<()>;

    assert_is_collection::<Vec<Element>>();
    assert_is_collection::<&'a Vec<Element>>();
    // FIXME: This does not work, I don't know why.
    // assert_is_collection::<Vec<&'a Element>>();
    // assert_is_collection::<&'a Vec<&'a Element>>();

    assert_is_collection::<(Element,)>();
    assert_is_collection::<(&'a Element,)>();
    assert_is_collection::<(Element, Element)>();
    assert_is_collection::<(&'a Element, Element)>();
    assert_is_collection::<(&'a Element, &'a Element)>();
    assert_is_collection::<&'a (Element, Element)>();
    assert_is_collection::<&'a (&'a Element, Element)>();
    assert_is_collection::<&'a (&'a Element, &'a Element)>();
}

/// A 1-dimensional layout of elements.
pub trait Layout1D<'a, C: Collection<'a>> {
    /// The layout of elements.
    ///
    /// This is an iterator over all the elements and where they are.
    type Layout: Iterator<Item = InnerElement<'a, <C as Collection<'a>>::Event>> + 'a;

    /// Get the layout of the elements in the collection, with an optional fixed cross axis size.
    fn layout(
        &'a self,
        elements: &'a C,
        main_axis_size: u16,
        cross_axis_size: Option<u16>,
        axis: Axis,
    ) -> Self::Layout;

    /// Get the range of main axis sizes this container can take up.
    ///
    /// By default this adds up the elements' minimum and maximum sizes.
    fn main_axis_size(
        &'a self,
        elements: &'a C,
        cross_axis_size: Option<u16>,
        axis: Axis,
    ) -> (u16, u16) {
        combine_main_axes(elements.iter().map(|element| match axis {
            Axis::X => element.width(cross_axis_size),
            Axis::Y => element.height(cross_axis_size),
        }))
    }
}

/// An element arranged by a [`Layout1D`](trait.Layout1D.html).
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

/// A 1-dimensional container of elements. It draws a [`Collection`](trait.Collection.html) with a
/// [`Layout1D`](trait.Layout1D.html), and is created by the [`column`](fn.column.html) and
/// [`row`](fn.row.html) functions.
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
    /// cursor of the container, and will receive all inputs if `broadcast_keys` is not set.
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
            .layout(
                &self.elements,
                main_axis_size,
                Some(cross_axis_size),
                self.axis,
            )
            .enumerate()
        {
            inner.element.draw(
                &mut output
                    .area(
                        self.axis.vec(inner.position, 0),
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
    fn width(&self, height: Option<u16>) -> (u16, u16) {
        match self.axis {
            Axis::X => self.layout.main_axis_size(&self.elements, height, Axis::X),
            Axis::Y => height.map_or_else(
                || combine_cross_axes(self.elements.iter().map(|element| element.width(None))),
                |height| {
                    combine_cross_axes(
                        self.layout
                            .layout(&self.elements, height, None, Axis::Y)
                            .map(|inner| inner.element.width(Some(inner.size))),
                    )
                },
            ),
        }
    }
    fn height(&self, width: Option<u16>) -> (u16, u16) {
        match self.axis {
            Axis::X => width.map_or_else(
                || combine_cross_axes(self.elements.iter().map(|element| element.height(None))),
                |width| {
                    combine_cross_axes(
                        self.layout
                            .layout(&self.elements, width, None, Axis::X)
                            .map(|inner| inner.element.height(Some(inner.size))),
                    )
                },
            ),
            Axis::Y => self.layout.main_axis_size(&self.elements, width, Axis::Y),
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

                for inner in self.layout.layout(
                    &self.elements,
                    main_axis_size,
                    Some(cross_axis_size),
                    self.axis,
                ) {
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

/// An axis: X or Y.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Axis {
    /// The x axis.
    X,
    /// The y axis.
    Y,
}

impl Axis {
    /// Construct a `Vec2` from the main value and the cross value.
    #[must_use]
    pub const fn vec<T>(self, main: T, cross: T) -> Vec2<T> {
        match self {
            Self::X => Vec2 { x: main, y: cross },
            Self::Y => Vec2 { x: cross, y: main },
        }
    }

    /// Get the main axis of the `Vec2`.
    #[must_use]
    pub fn main_of<T>(self, v: Vec2<T>) -> T {
        self.main_cross_of(v).0
    }

    /// Get the main and cross axes of the `Vec2`.
    #[must_use]
    pub fn main_cross_of<T>(self, v: Vec2<T>) -> (T, T) {
        match self {
            Self::X => (v.x, v.y),
            Self::Y => (v.y, v.x),
        }
    }
}

/// Create a row of elements with the specified layout.
///
/// By default keys inputs will not be broadcast to all elements and there will be no focused
/// element.
#[must_use]
pub fn row<E, L>(layout: L, elements: E) -> Container1D<E, L> {
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
/// By default keys inputs will not be broadcast to all elements and there will be no focused
/// element.
#[must_use]
pub fn column<E, L>(layout: L, elements: E) -> Container1D<E, L> {
    Container1D {
        elements,
        layout,
        axis: Axis::Y,
        broadcast_keys: false,
        focused: None,
    }
}

fn combine_main_axes(main_axes: impl Iterator<Item = (u16, u16)>) -> (u16, u16) {
    main_axes.fold((0, 0), |(min_acc, max_acc), (min, max)| {
        (min_acc + min, max_acc.saturating_add(max))
    })
}

fn combine_cross_axes(cross_axes: impl Iterator<Item = (u16, u16)>) -> (u16, u16) {
    cross_axes.fold((0, 0), |(min_acc, max_acc), (min_len, max_len)| {
        (max(min_acc, min_len), max(max_acc, max_len))
    })
}
