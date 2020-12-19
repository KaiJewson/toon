use std::cmp::{min, Ordering};

use crate::Element;

use super::{Axis, Collection, InnerElement, Layout1D};

use self::private::Layout;

/// A dynamic element [`Layout1D`] where there is one flexible and many fixed sized elements,
/// created by the [`stretch`] function.
///
/// This is similar in purpose to [`Flow`](super::Flow), but less general purpose and
/// implemented much more efficiently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Stretch {
    /// The index of the stretched element of the container.
    ///
    /// It is not an error if this element does not exist.
    pub stretched: usize,
}

impl<'a, C: Collection<'a>> Layout1D<'a, C> for Stretch {
    type Layout = Layout<<C as Collection<'a>>::Iter>;

    fn layout(
        &'a self,
        elements: &'a C,
        main_axis_size: u16,
        cross_axis_size: Option<u16>,
        axis: Axis,
    ) -> Self::Layout {
        let elements_len = elements.len();
        Layout {
            elements: elements.iter(),
            elements_len,
            i: if self.stretched == 0 {
                elements_len - 1
            } else {
                0
            },
            stretched: self.stretched,
            start_offset: 0,
            end_offset: main_axis_size,
            axis,
            cross_axis_size,
        }
    }

    fn main_axis_size(
        &'a self,
        elements: &'a C,
        cross_axis_size: Option<u16>,
        axis: Axis,
    ) -> (u16, u16) {
        (
            elements
                .iter()
                .enumerate()
                .map(|(i, element)| {
                    if i == self.stretched {
                        0
                    } else {
                        axis.element_size(element, cross_axis_size).0
                    }
                })
                .sum(),
            u16::MAX,
        )
    }
}

mod private {
    use super::super::Axis;

    /// The layout of a Stretch.
    ///
    /// This iterates from the start up to but not including the stretched element, and then from the
    /// end until it reaches the stretched element.
    #[derive(Debug)]
    pub struct Layout<I> {
        /// The iterator over the elements.
        pub(super) elements: I,
        /// The original length of the iterator. Used to set `i` to after it reaches the element before
        /// the stretched one.
        pub(super) elements_len: usize,
        /// The index into the iterator.
        pub(super) i: usize,
        /// The index of the stretched element.
        pub(super) stretched: usize,

        /// The location at which free space starts.
        pub(super) start_offset: u16,
        /// The location at which free space ends.
        pub(super) end_offset: u16,

        /// The axis of the container.
        pub(super) axis: Axis,
        /// The cross axis size of the container.
        pub(super) cross_axis_size: Option<u16>,
    }
}

impl<'a, I, Event: 'a> Iterator for Layout<I>
where
    I: Iterator<Item = &'a dyn Element<Event = Event>> + DoubleEndedIterator + 'a,
{
    type Item = InnerElement<'a, Event>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start_offset >= self.end_offset {
            // There is no more space left.
            return None;
        }

        match self.i.cmp(&self.stretched) {
            // We are taking elements from before the stretched element.
            Ordering::Less => {
                let i = self.i;
                self.i += 1;
                if self.i == self.stretched {
                    // We have reached the middle, jump to the end.
                    self.i = self.elements_len - 1;
                }

                let element = self.elements.next()?;
                let main_axis_size = self.axis.element_size(element, self.cross_axis_size).0;
                let position = self.start_offset;
                self.start_offset = self.start_offset.saturating_add(main_axis_size);
                Some(InnerElement {
                    index: i,
                    element,
                    position,
                    size: main_axis_size,
                })
            }
            // We are at the last element, the stretched one.
            Ordering::Equal => {
                let element = self.elements.next()?;
                debug_assert!(self.elements.next().is_none());

                Some(InnerElement {
                    index: self.i,
                    element,
                    position: self.start_offset,
                    size: self.end_offset - self.start_offset,
                })
            }
            // We are after the stretched element and are moving backwards.
            Ordering::Greater => {
                let i = self.i;
                self.i -= 1;

                let element = self.elements.next_back()?;
                let main_axis_size = min(
                    self.axis.element_size(element, self.cross_axis_size).0,
                    self.end_offset - self.start_offset,
                );

                self.end_offset -= main_axis_size;
                Some(InnerElement {
                    index: i,
                    element,
                    position: self.end_offset,
                    size: main_axis_size,
                })
            }
        }
    }
}

/// Create a new [`Stretch`] layout.
#[must_use]
pub fn stretch(stretched: usize) -> Stretch {
    Stretch { stretched }
}

#[test]
fn test_out_of_range() {
    let mut grid = crate::Grid::new((6, 5));

    crate::column::<_, _, ()>(
        stretch(5),
        (crate::span("1"), crate::span("2"), crate::span("3")),
    )
    .draw(&mut grid);

    assert_eq!(
        grid.contents(),
        ["1     ", "2     ", "3     ", "      ", "      ",]
    );
}

#[test]
fn test_grow() {
    use crate::ElementExt;

    let mut grid = crate::Grid::new((3, 5));

    crate::column::<_, _, ()>(
        stretch(1),
        (
            crate::span("1").tile((0, 0)),
            crate::span("2").tile((0, 0)),
            crate::span("3").tile((0, 0)),
        ),
    )
    .draw(&mut grid);

    assert_eq!(grid.contents(), ["111", "222", "222", "222", "333",]);
}

#[test]
fn test_shrink() {
    use crate::ElementExt;

    let mut grid = crate::Grid::new((3, 2));

    crate::column::<_, _, ()>(
        stretch(1),
        (
            crate::span("1").tile((0, 0)),
            crate::span("2").tile((0, 0)),
            crate::span("3").tile((0, 0)),
        ),
    )
    .draw(&mut grid);

    assert_eq!(grid.contents(), ["111", "333"]);
}
