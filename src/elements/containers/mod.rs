//! Containers for several elements.

use std::iter;
use std::marker::PhantomData;

use crate::{Element, Vec2};

pub use flow::*;

mod flow;

/// A tuple of elements that can be used with containers in this module, created by the
/// [`tuple`](fn.tuple.html) function.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Tuple<T, Event> {
    /// The tuple containing the elements.
    pub tuple: T,
    event: PhantomData<Event>,
}

macro_rules! tupiter {
    () => { iter::Empty<&'a dyn Element<Event>> };
    ($x:ty,) => { iter::Once<&'a dyn Element<Event>> };
    ($x:ty, $($xs:ty,)*) => {
        iter::Chain<
            tupiter!($x,),
            tupiter!($($xs,)*),
        >
    };
}

macro_rules! create_tupiter {
    () => { iter::empty::<&'a dyn Element<Event>>() };
    ($x:expr,) => { iter::once::<&'a dyn Element<Event>>($x) };
    ($x:expr, $($xs:expr,)*) => {
        Iterator::chain(
            create_tupiter!($x,),
            create_tupiter!($($xs,)*),
        )
    };
}

macro_rules! impl_into_iterator_for_element_tuple {
    ($(($($param:ident),*),)*) => {$(
        impl<'a, Event, $($param,)*> IntoIterator for &'a Tuple<($($param,)*), Event>
        where
            $($param: Element<Event>,)*
        {
            type Item = &'a dyn Element<Event>;
            type IntoIter = tupiter!($($param,)*);

            fn into_iter(self) -> Self::IntoIter {
                #[allow(non_snake_case)]
                let ($($param,)*) = &self.tuple;
                create_tupiter!($($param,)*)
            }
        }
    )*}
}

impl_into_iterator_for_element_tuple! {
    (),
    (A),
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

/// Create a collection of elements from a tuple.
///
/// # Examples
///
/// ```
/// // The element will display as two lines at the top and bottom of the screen.
/// let element = toon::column(toon::tuple::<_, ()>((
///     toon::span("At the top of the screen"),
///     toon::empty(),
///     toon::span("At the bottom of the screen"),
/// )));
/// ```
#[must_use]
pub fn tuple<T, Event>(tuple: T) -> Tuple<T, Event>
where
    for<'a> &'a Tuple<T, Event>: IntoIterator<Item = &'a dyn Element<Event>>,
{
    Tuple {
        tuple,
        event: PhantomData,
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

    /// Get the main and cross axis of the `Vec2`.
    #[must_use]
    pub fn main_cross_of<T>(self, v: Vec2<T>) -> (T, T) {
        match self {
            Self::X => (v.x, v.y),
            Self::Y => (v.y, v.x),
        }
    }
}
