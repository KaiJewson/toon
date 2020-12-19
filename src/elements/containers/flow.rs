use std::cmp::min;
use std::iter;

use crate::Element;

use super::{Axis, Collection, InnerElement, Layout1D};

use self::private::Layout;

/// A generic dynamic [`Layout1D`], created by the [`flow`] function.
///
/// The layout algorithm works by calculating the minimum required space for each element, and then
/// giving out all extra space equally among the other elements if they support it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Flow {
    /// The direction the flow container is biased towards.
    ///
    /// If [`None`], the container will evenly distribute space among its flexible elements, even if
    /// it results in there being extra space at the end. Otherwise, it will fill that extra space
    /// by unevenly giving elements at one end more space.
    pub bias: Option<End>,
}

impl Flow {
    /// Set the bias of the container.
    ///
    /// The container will fill any extra space by giving more space to the elements at the given
    /// end.
    #[must_use]
    #[allow(clippy::unused_self)]
    pub fn bias(self, bias: End) -> Self {
        Self { bias: Some(bias) }
    }
}

impl<'a, C: Collection<'a>> Layout1D<'a, C> for Flow {
    type Layout = Layout<<C as Collection<'a>>::Iter>;

    fn layout(
        &'a self,
        elements: &'a C,
        main_axis_size: u16,
        cross_axis_size: Option<u16>,
        axis: Axis,
    ) -> Self::Layout {
        let (maximum_growth, dividing_point) =
            self.calculate_layout(main_axis_size, cross_axis_size, elements, axis);

        Layout {
            elements: elements.iter(),
            elements_len: elements.len(),
            index: 0,
            maximum_growth,
            dividing_point,
            position_accumulator: 0,
            main_axis_size,
            cross_axis_size,
            axis,
            bias: self.bias,
        }
    }
}

impl Flow {
    /// An iterator over the elements in the order of the bias. Panics if there is no bias.
    fn elements_biased_order<'a, E: Collection<'a>>(
        self,
        elements: &'a E,
    ) -> impl Iterator<Item = &'a dyn Element<Event = <E as Collection<'a>>::Event>> {
        let bias = self.bias.unwrap();

        let mut iter = elements.iter();

        iter::from_fn(move || match bias {
            End::Start => iter.next(),
            End::End => iter.next_back(),
        })
    }

    /// Calculate the layout of the flow.
    ///
    /// The first element of the tuple is how much the elements are able to grow along on the main
    /// axis. The second element of the tuple gives the index from the front (start bias) or back
    /// (end bias) at which the first element of the tuple is treated as one less. If there is no
    /// bias the value is ignored.
    fn calculate_layout<'a>(
        self,
        main_axis_size: u16,
        cross_axis_size: Option<u16>,
        elements: &'a impl Collection<'a>,
        axis: Axis,
    ) -> (u16, usize) {
        let mut main_axis_extra_space = main_axis_size.saturating_sub(
            elements
                .iter()
                .map(|element| axis.element_size(element, cross_axis_size).0)
                .fold(0, u16::saturating_add),
        );

        if main_axis_extra_space == 0 {
            return (0, elements.len());
        }

        if self.bias.is_some() {
            for maximum_growth in 1.. {
                let mut elements_grew = false;

                for (i, element) in self.elements_biased_order(elements).enumerate() {
                    let (min_main_axis_size, max_main_axis_size) =
                        axis.element_size(element, cross_axis_size);

                    if max_main_axis_size - min_main_axis_size >= maximum_growth {
                        elements_grew = true;

                        main_axis_extra_space -= 1;

                        if main_axis_extra_space == 0 {
                            return (maximum_growth, i);
                        }
                    }
                }

                if !elements_grew {
                    // We haven't filled the container, but no elements can grow to fill it, so exit.
                    return (u16::MAX, 0);
                }
            }
            unreachable!()
        } else {
            #[allow(clippy::maybe_infinite_iter)]
            let maximum_growth = (1..)
                .take_while(|&maximum_growth| {
                    let mut main_axis_extra_space = main_axis_extra_space;
                    let mut overflow = false;
                    let mut elements_grew = false;

                    for element in elements.iter() {
                        let (min_main_axis_size, max_main_axis_size) =
                            axis.element_size(element, cross_axis_size);

                        let range = max_main_axis_size - min_main_axis_size;

                        if range >= maximum_growth {
                            elements_grew = true;
                        }

                        let growth = min(range, maximum_growth);
                        main_axis_extra_space =
                            if let Some(extra) = main_axis_extra_space.checked_sub(growth) {
                                extra
                            } else {
                                overflow = true;
                                break;
                            };
                    }

                    elements_grew && !overflow
                })
                .last()
                .unwrap_or(0);
            (maximum_growth, /* ignored */ 0)
        }
    }
}

mod private {
    use super::super::Axis;
    use super::End;

    #[derive(Debug)]
    pub struct Layout<I> {
        pub(super) elements: I,
        pub(super) elements_len: usize,
        pub(super) index: usize,

        pub(super) maximum_growth: u16,
        pub(super) dividing_point: usize,

        pub(super) position_accumulator: u16,

        pub(super) main_axis_size: u16,
        pub(super) cross_axis_size: Option<u16>,
        pub(super) axis: Axis,
        pub(super) bias: Option<End>,
    }
}

impl<'a, I, Event: 'a> Iterator for Layout<I>
where
    I: Iterator<Item = &'a dyn Element<Event = Event>>,
{
    type Item = InnerElement<'a, Event>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position_accumulator >= self.main_axis_size {
            return None;
        }

        let element = self.elements.next()?;
        let index = self.index;

        self.index += 1;

        let (min_size, max_size) = self.axis.element_size(element, self.cross_axis_size);

        let maximum_growth_is_less = match self.bias {
            Some(End::Start) => index > self.dividing_point,
            Some(End::End) => self.elements_len - index - 1 > self.dividing_point,
            None => false,
        };

        let size = min(
            max_size,
            min_size
                + if maximum_growth_is_less {
                    self.maximum_growth - 1
                } else {
                    self.maximum_growth
                },
        );

        let position = self.position_accumulator;
        self.position_accumulator += size;

        Some(InnerElement {
            element,
            index,
            position,
            size,
        })
    }
}

/// An end of a container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum End {
    /// The start of the container.
    Start,
    /// The end of the container.
    End,
}

/// Create a new [`Flow`] layout.
///
/// By default it will not be biased to either end; this means that it will not always totally fill
/// the container.
#[must_use]
pub fn flow() -> Flow {
    Flow { bias: None }
}

#[test]
fn test_too_large() {
    let mut grid = crate::Grid::new((5, 6));

    crate::column::<_, _, ()>(
        flow(),
        (
            crate::span("Foo"),
            crate::span("Bar"),
            crate::span("Baz"),
            crate::span("Foo2"),
            crate::span("Bar2"),
            crate::span("Baz2"),
            crate::span("Extra Line"),
        ),
    )
    .draw(&mut grid);

    assert_eq!(
        grid.contents(),
        ["Foo  ", "Bar  ", "Baz  ", "Foo2 ", "Bar2 ", "Baz2 ",]
    );
}

#[test]
fn test_biases() {
    let mut grid = crate::Grid::new((5, 6));

    let mut three_lines = crate::column::<_, _, ()>(
        flow(),
        (
            crate::span("Top Line"),
            crate::empty(),
            crate::span("Middle Line"),
            crate::empty(),
            crate::span("Bottom Line"),
        ),
    );

    three_lines.draw(&mut grid);
    assert_eq!(
        grid.contents(),
        ["Top L", "     ", "Middl", "     ", "Botto", "     ",]
    );

    three_lines.layout.bias = Some(End::Start);
    grid.clear();
    three_lines.draw(&mut grid);
    assert_eq!(
        grid.contents(),
        ["Top L", "     ", "     ", "Middl", "     ", "Botto",]
    );

    three_lines.layout.bias = Some(End::End);
    grid.clear();
    three_lines.draw(&mut grid);
    assert_eq!(
        grid.contents(),
        ["Top L", "     ", "Middl", "     ", "     ", "Botto",]
    );

    crate::row::<_, _, ()>(
        flow().bias(End::Start),
        (
            crate::empty(),
            crate::span("1"),
            crate::empty(),
            crate::empty(),
            crate::span("2"),
            crate::empty(),
            crate::span("34"),
            crate::span("5"),
            crate::empty(),
            crate::span("6"),
            crate::empty(),
        ),
    )
    .draw(&mut grid);

    assert_eq!(
        grid.contents(),
        ["12345", "     ", "Middl", "     ", "     ", "Botto",]
    );
}
