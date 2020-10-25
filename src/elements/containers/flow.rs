use std::cmp::{max, min};
use std::iter;

use crate::output::{Ext as _, Output};
use crate::{Element, Events, Input};

use super::{Axis, Collection};

/// A one-dimensional dynamic element layout, created by the [`column`](fn.column.html) and
/// [`row`](fn.row.html) functions.
///
/// The layout algorithm works by calculating the minimum required space for each element, and then
/// giving out all extra space equally among the other elements if they support it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Flow<E> {
    /// The elements inside this container.
    pub elements: E,
    /// The axis this container operates on.
    pub axis: Axis,
    /// Whether to broadcast key inputs to all elements. If `false`, only the focused element will
    /// receive key inputs.
    pub broadcast_keys: bool,
    /// The index of the focused element of the container. This element will set the title and
    /// cursor of the container, and will receive all inputs if `broadcast_keys` is not set.
    ///
    /// It is not an error if this element does not exist.
    pub focused: Option<usize>,
    /// The direction the flow container is biased towards.
    ///
    /// If `None`, the container will evenly distribute space among its flexible elements, even if
    /// it results in there being extra space at the end. Otherwise, it will fill that extra space
    /// by unevenly giving elements at one end more space.
    pub bias: Option<End>,
}

impl<E> Flow<E> {
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

    /// Set the bias of the container.
    ///
    /// The container will fill any extra space by giving more space to the elements at the given
    /// end.
    #[must_use]
    pub fn bias(self, bias: End) -> Self {
        Self {
            bias: Some(bias),
            ..self
        }
    }
}

impl<E> Flow<E> {
    /// An iterator over the elements in the order of the bias. Panics if there is no bias.
    fn elements_biased_order<'a>(
        &'a self,
    ) -> impl Iterator<Item = &'a dyn Element<Event = <E as Collection<'a>>::Event>>
    where
        E: Collection<'a>,
    {
        let bias = self.bias.unwrap();

        let mut iter = self.elements.iter();

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
        &'a self,
        main_axis_size: u16,
        cross_axis_size: Option<u16>,
    ) -> (u16, usize)
    where
        E: Collection<'a>,
    {
        let mut main_axis_extra_space = main_axis_size.saturating_sub(
            self.elements
                .iter()
                .map(|element| match self.axis {
                    Axis::X => element.width(cross_axis_size).0,
                    Axis::Y => element.height(cross_axis_size).0,
                })
                .fold(0, u16::saturating_add),
        );

        if main_axis_extra_space == 0 {
            return (0, self.elements.len());
        }

        if self.bias.is_some() {
            for maximum_growth in 1.. {
                let mut elements_grew = false;

                for (i, element) in self.elements_biased_order().enumerate() {
                    let (min_main_axis_size, max_main_axis_size) = match self.axis {
                        Axis::X => element.width(cross_axis_size),
                        Axis::Y => element.height(cross_axis_size),
                    };

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

                    for element in self.elements.iter() {
                        let (min_main_axis_size, max_main_axis_size) = match self.axis {
                            Axis::X => element.width(cross_axis_size),
                            Axis::Y => element.height(cross_axis_size),
                        };

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

    /// An iterator over elements and their main axis sizes.
    fn element_sizes<'a>(
        &'a self,
        main_axis_size: u16,
        cross_axis_size: Option<u16>,
    ) -> impl Iterator<Item = (u16, &'a dyn Element<Event = <E as Collection<'a>>::Event>)> + 'a
    where
        E: Collection<'a>,
    {
        let (maximum_growth, dividing_point) =
            self.calculate_layout(main_axis_size, cross_axis_size);

        let elements_len = self.elements.len();

        self.elements.iter().enumerate().map(move |(i, element)| {
            let (min_main_axis_size, max_main_axis_size) = match self.axis {
                Axis::X => element.width(cross_axis_size),
                Axis::Y => element.height(cross_axis_size),
            };

            let maximum_growth_is_less = match self.bias {
                Some(End::Start) => i > dividing_point,
                Some(End::End) => elements_len - i - 1 > dividing_point,
                None => false,
            };

            let element_main_axis_size = min(
                max_main_axis_size,
                min_main_axis_size
                    + if maximum_growth_is_less {
                        maximum_growth - 1
                    } else {
                        maximum_growth
                    },
            );

            (element_main_axis_size, element)
        })
    }
}

impl<E, Event> Element for Flow<E>
where
    for<'a> E: Collection<'a, Event = Event>,
{
    type Event = Event;

    fn draw(&self, output: &mut dyn Output) {
        let (main_axis_size, cross_axis_size) = self.axis.main_cross_of(output.size());

        let mut offset = 0;

        for (i, (element_main_axis_size, element)) in self
            .element_sizes(main_axis_size, Some(cross_axis_size))
            .enumerate()
        {
            let size = self.axis.vec(element_main_axis_size, cross_axis_size);

            element.draw(
                &mut output
                    .area(self.axis.vec(offset, 0), size)
                    .on_set_cursor(|output, cursor| {
                        if self.focused == Some(i) {
                            output.set_cursor(cursor);
                        }
                    }),
            );

            offset += element_main_axis_size;
        }
    }
    fn width(&self, height: Option<u16>) -> (u16, u16) {
        match self.axis {
            Axis::X => combine_main_axes(self.elements.iter().map(|element| element.width(height))),
            Axis::Y => height.map_or_else(
                || combine_cross_axes(self.elements.iter().map(|element| element.width(None))),
                |height| {
                    combine_cross_axes(
                        self.element_sizes(height, None)
                            .map(|(height, element)| element.width(Some(height))),
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
                        self.element_sizes(width, None)
                            .map(|(width, element)| element.height(Some(width))),
                    )
                },
            ),
            Axis::Y => combine_main_axes(self.elements.iter().map(|element| element.height(width))),
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

                let mut offset = 0;

                for (element_main_axis_size, element) in
                    self.element_sizes(main_axis_size, Some(cross_axis_size))
                {
                    let local_main_axis = mouse_main_axis
                        .checked_sub(offset)
                        .filter(|&pos| pos < element_main_axis_size);

                    if let Some(local_main_axis) = local_main_axis {
                        let mut mouse = mouse;
                        mouse.at = self.axis.vec(local_main_axis, mouse_cross_axis);
                        mouse.size = self.axis.vec(element_main_axis_size, cross_axis_size);
                        element.handle(Input::Mouse(mouse), events);
                        break;
                    }

                    offset += element_main_axis_size;
                }
            }
        }
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

/// An end of a container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum End {
    /// The start of the container.
    Start,
    /// The end of the container.
    End,
}

/// Create a column of elements with the [`Flow`](struct.Flow.html) layout.
///
/// By default keys inputs will not be broadcast to all elements, there will be no focused element,
/// and the layout will not be biased.
///
/// # Example
///
/// An element that has text at the top, middle and bottom.
///
/// ```
/// let element = toon::column((
///     toon::span::<_, ()>("At the top!"),
///     toon::empty(),
///     toon::span("At the middle!"),
///     toon::empty(),
///     toon::span("At the bottom!"),
/// ));
/// ```
#[must_use]
pub fn column<E: for<'a> Collection<'a>>(elements: E) -> Flow<E> {
    Flow {
        elements,
        axis: Axis::Y,
        broadcast_keys: false,
        focused: None,
        bias: None,
    }
}

/// Create a row of elements with the [`Flow`](struct.Flow.html) layout.
///
/// By default keys inputs will not be broadcast to all elements, there will be no focused element,
/// and the layout will not be biased.
///
/// # Example
///
/// An element that has text at the left, middle and right.
///
/// ```
/// let element = toon::row((
///     toon::span::<_, ()>("On the left!"),
///     toon::empty(),
///     toon::span("In the middle!"),
///     toon::empty(),
///     toon::span("On the right!"),
/// ));
/// ```
#[must_use]
pub fn row<E: for<'a> Collection<'a>>(elements: E) -> Flow<E> {
    Flow {
        elements,
        axis: Axis::X,
        broadcast_keys: false,
        focused: None,
        bias: None,
    }
}
