//! Containers for several elements.

use std::iter;
use std::slice;

use crate::Element;
use crate::Vec2;

pub use flow::*;
pub use stack::*;

mod flow;
mod stack;

/// A collection of elements, held by containers.
///
/// This trait is implemented for vectors of elements and tuples of elements (which can be
/// different types).
///
/// Note that ideally all collections would just use any type whose reference implements
/// `IntoIterator` for any element type, but bugs in rustc mean that it doesn't work.
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
