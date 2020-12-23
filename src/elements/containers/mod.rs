//! Containers for several elements.
//!
//! # Which 1D layout to use?
//!
//! Toon's 1D container, [`Container1D`], can use multiple [layouts](Layout1D) to draw its elements.
//!
//! - [`Static`] is the simplest and fastest layout. It gives each element the minimum space it
//! needs, and any extra space is left blank.
//! - [`Stretch`] is more advanced and also fast. It gives each element except one the minimum space
//! it needs, and then gives all the rest of the space to the one element.
//! - [`Flow`] is the most advanced and the slowest. It gives each element the minimum space it
//! needs, and then distributes all remaining space evenly among elements that support it.

use std::iter;

#[cfg(feature = "either")]
use either_crate::Either;

use crate::Element;

mod container_1d;
pub use container_1d::*;

mod stack;
pub use stack::*;

/// A collection of elements, held by containers.
///
/// This trait is implemented for vectors of elements and tuples of elements (which can be
/// different types).
pub trait Collection<'a> {
    /// The events of the elements in the collection.
    type Event: 'a;

    /// An iterator over the collection.
    type Iter: Iterator<Item = &'a dyn Element<Event = Self::Event>> + DoubleEndedIterator;

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

impl<'a, E: Element> Collection<'a> for Vec<E>
where
    E::Event: 'a,
{
    type Event = E::Event;
    // We can't write the type directly, as that would mean we have to bound `E: 'a`, so we use a
    // dynamic trait object instead.
    type Iter = Box<
        dyn vec_iter_impl::IteratorAndDoubleEnded<Item = &'a dyn Element<Event = Self::Event>> + 'a,
    >;

    fn iter(&'a self) -> Self::Iter {
        Box::new(
            (**self)
                .iter()
                .map(|element| -> &'a dyn Element<Event = Self::Event> { element }),
        )
    }

    fn len(&'a self) -> usize {
        self.len()
    }
}

mod vec_iter_impl {
    /// A trait for an iterator that's double ended, for use in dynamic trait objects.
    pub trait IteratorAndDoubleEnded: Iterator + DoubleEndedIterator {}
    impl<T: Iterator + DoubleEndedIterator> IteratorAndDoubleEnded for T {}
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
    ($(($($param:ident),*),)*) => {$(
        impl<'a, Event: 'a, $($param,)*> Collection<'a> for ($($param,)*)
        where
            $($param: Element<Event = Event>,)*
        {
            type Event = Event;
            type Iter = tupiter!($($param,)*);

            fn iter(&'a self) -> Self::Iter {
                #[allow(non_snake_case)]
                let ($($param,)*) = self;
                create_tupiter!($($param,)*)
            }

            fn len(&'a self) -> usize {
                tuple_len!($($param,)*)
            }
        }
    )*}
}

impl_collection_for_tuples! {
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
    assert_is_collection::<Vec<&'a Element>>();
    assert_is_collection::<&'a Vec<&'a Element>>();

    assert_is_collection::<(Element,)>();
    assert_is_collection::<(&'a Element,)>();
    assert_is_collection::<(Element, Element)>();
    assert_is_collection::<(&'a Element, Element)>();
    assert_is_collection::<(&'a Element, &'a Element)>();
    assert_is_collection::<&'a (Element, Element)>();
    assert_is_collection::<&'a (&'a Element, Element)>();
    assert_is_collection::<&'a (&'a Element, &'a Element)>();
}
