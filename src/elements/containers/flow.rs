use std::cmp::{max, min};

use crate::{Element, Events, Input, Output};

use super::Axis;

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
}

impl<E> Flow<E> {
    /// Calculate the layout of the flow.
    ///
    /// The first element of the tuple is how much the elements up to and including the element at
    /// the index of the second element of the tuple are able to grow along on the main axis. All
    /// elements after that can grow one cell less.
    fn calculate_layout<'a, Event>(
        &'a self,
        main_axis_size: u16,
        cross_axis_size: Option<u16>,
    ) -> (u16, usize)
    where
        &'a E: IntoIterator,
        <&'a E as IntoIterator>::Item: Element<Event>,
    {
        let main_axis_extra_space = main_axis_size.saturating_sub(
            self.elements
                .into_iter()
                .map(|element| match self.axis {
                    Axis::X => element.width(cross_axis_size).0,
                    Axis::Y => element.height(cross_axis_size).0,
                })
                .fold(0, u16::saturating_add),
        );

        // Find a maximum growth that fills the container
        (1..)
            .find_map(|maximum_growth| {
                let mut all_elements_are_at_max_size = true;
                let mut remaining_space = main_axis_extra_space;

                for (i, element) in self.elements.into_iter().enumerate() {
                    let (min_main_axis_size, max_main_axis_size) = match self.axis {
                        Axis::X => element.width(cross_axis_size),
                        Axis::Y => element.height(cross_axis_size),
                    };
                    let element_range = max_main_axis_size - min_main_axis_size;

                    if maximum_growth < element_range {
                        all_elements_are_at_max_size = false;
                    }

                    remaining_space =
                        remaining_space.saturating_sub(min(element_range, maximum_growth));

                    if remaining_space == 0 {
                        return Some((maximum_growth, i));
                    }
                }

                if all_elements_are_at_max_size {
                    // We won't be able to fill the entire container
                    return Some((u16::MAX, 0));
                }

                None
            })
            .unwrap()
    }

    /// An iterator over elements and their main axis sizes.
    fn element_sizes<'a, Event>(
        &'a self,
        main_axis_size: u16,
        cross_axis_size: Option<u16>,
    ) -> impl Iterator<Item = (u16, <&'a E as IntoIterator>::Item)>
    where
        &'a E: IntoIterator,
        <&'a E as IntoIterator>::Item: Element<Event>,
    {
        let (maximum_growth, dividing_point) =
            self.calculate_layout(main_axis_size, cross_axis_size);

        self.elements
            .into_iter()
            .enumerate()
            .map(move |(i, element)| {
                let (min_main_axis_size, max_main_axis_size) = match self.axis {
                    Axis::X => element.width(cross_axis_size),
                    Axis::Y => element.height(cross_axis_size),
                };

                let element_main_axis_size = min(
                    max_main_axis_size,
                    min_main_axis_size
                        + if i > dividing_point {
                            maximum_growth - 1
                        } else {
                            maximum_growth
                        },
                );

                (element_main_axis_size, element)
            })
    }
}

impl<E, Event> Element<Event> for Flow<E>
where
    for<'a> &'a E: IntoIterator<Item = &'a dyn Element<Event>>,
{
    fn draw(&self, output: &mut dyn Output) {
        let (main_axis_size, cross_axis_size) = self.axis.main_cross_of(output.size());

        let mut offset = 0;

        for (i, (element_main_axis_size, element)) in self
            .element_sizes(main_axis_size, Some(cross_axis_size))
            .enumerate()
        {
            let size = self.axis.vec(element_main_axis_size, cross_axis_size);

            let drawing_offset = self.axis.vec(offset, 0);

            element.draw(&mut crate::output_with(
                &mut *output,
                |_| size,
                |output, pos, c, style| output.write_char(pos + drawing_offset, c, style),
                |output, title| {
                    if self.focused == Some(i) {
                        output.set_title(title);
                    }
                },
                |output, cursor| {
                    if self.focused == Some(i) {
                        output.set_cursor(cursor);
                    }
                },
            ));

            offset += element_main_axis_size;
        }
    }
    fn width(&self, height: Option<u16>) -> (u16, u16) {
        match self.axis {
            Axis::X => combine_main_axes(
                self.elements
                    .into_iter()
                    .map(|element| element.width(height)),
            ),
            Axis::Y => height.map_or_else(
                || combine_cross_axes(self.elements.into_iter().map(|element| element.width(None))),
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
                || {
                    combine_cross_axes(
                        self.elements
                            .into_iter()
                            .map(|element| element.height(None)),
                    )
                },
                |width| {
                    combine_cross_axes(
                        self.element_sizes(width, None)
                            .map(|(width, element)| element.height(Some(width))),
                    )
                },
            ),
            Axis::Y => combine_main_axes(
                self.elements
                    .into_iter()
                    .map(|element| element.height(width)),
            ),
        }
    }
    fn handle(&self, input: Input, events: &mut dyn Events<Event>) {
        match input {
            Input::Key(_) if self.broadcast_keys => {
                for element in &self.elements {
                    element.handle(input, events);
                }
            }
            Input::Key(_) => {
                if let Some(element) = self.focused.and_then(|i| self.elements.into_iter().nth(i)) {
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

/// Create a column of elements with the [`Flow`](struct.Flow.html) layout.
#[must_use]
pub fn column<E, Event>(elements: E) -> Flow<E>
where
    for<'a> &'a E: IntoIterator<Item = &'a dyn Element<Event>>,
{
    Flow {
        elements,
        axis: Axis::Y,
        broadcast_keys: false,
        focused: None,
    }
}

/// Create a row of elements with the [`Flow`](struct.Flow.html) layout.
#[must_use]
pub fn row<E, Event>(elements: E) -> Flow<E>
where
    for<'a> &'a E: IntoIterator<Item = &'a dyn Element<Event>>,
{
    Flow {
        elements,
        axis: Axis::X,
        broadcast_keys: false,
        focused: None,
    }
}
