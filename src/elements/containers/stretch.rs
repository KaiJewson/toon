use std::cmp::{min, Ordering};
use std::fmt;

use crate::output::{Ext as _, Output};
use crate::{Element, Events, Input};

use super::{combine_cross_axes, Axis, Collection};

/// A one-dimensional element layout where there is one flexible and many fixed sized elements,
/// created by the [`stretch_column`](fn.stretch_column.html) and
/// [`stretch_row`](fn.stretch_row.html) functions.
///
/// This is similar in purpose to [`Flow`](struct.Flow.html), but less general purpose and
/// implemented much more efficiently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Stretch<E> {
    /// The elements in this container.
    pub elements: E,
    /// The axis this container operates on.
    pub axis: Axis,
    /// The index of the stretched element of the container.
    ///
    /// It is not an error if this element does not exist.
    pub stretched: usize,
    /// Whether to broadcast key inputs to all elements. If `false`, only the focused element will
    /// receive key inputs.
    pub broadcast_keys: bool,
    /// The index of the focused element of the container. This element will set the title and
    /// cursor of the container, and will receive all inputs if `broadcast_keys` is not set.
    ///
    /// It is not an error if this element does not exist.
    pub focused: Option<usize>,
}

impl<E> Stretch<E> {
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

impl<E> Stretch<E> {
    /// An iterator over indices, the elements, their offsets and their sizes.
    fn layout<'a>(
        &'a self,
        main_axis_size: u16,
        cross_axis_size: Option<u16>,
    ) -> Layout<<E as Collection<'a>>::Iter>
    where
        E: Collection<'a>,
    {
        Layout {
            elements: self.elements.iter(),
            elements_len: self.elements.len(),
            i: if self.stretched == 0 {
                self.elements.len() - 1
            } else {
                0
            },
            stretched: self.stretched,
            start_offset: 0,
            end_offset: main_axis_size,
            axis: self.axis,
            main_axis_size,
            cross_axis_size,
        }
    }
}

/// An iterator over indices, the elements, their offsets and their sizes.
///
/// This iterates from the start up to but not including the stretched element, and then from the
/// end until it reaches the stretched element.
struct Layout<I> {
    /// The iterator over the elements.
    elements: I,
    /// The original length of the iterator. Used to set `i` to after it reaches the element before
    /// the stretched one.
    elements_len: usize,
    /// The index into the iterator.
    i: usize,
    /// The index of the stretched element.
    stretched: usize,

    /// The location at which free space starts.
    start_offset: u16,
    /// The location at which free space ends.
    end_offset: u16,

    /// The axis of the container.
    axis: Axis,
    /// The main axis size of the container.
    main_axis_size: u16,
    /// The cross axis size of the container.
    cross_axis_size: Option<u16>,
}

impl<'a, I, Event: 'a> Iterator for Layout<I>
where
    I: Iterator<Item = &'a dyn Element<Event = Event>> + DoubleEndedIterator + 'a,
{
    type Item = (usize, &'a dyn Element<Event = Event>, u16, u16);

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
                let main_axis_size = match self.axis {
                    Axis::X => element.width(self.cross_axis_size).0,
                    Axis::Y => element.height(self.cross_axis_size).0,
                };
                let offset = self.start_offset;
                self.start_offset = self.start_offset.saturating_add(main_axis_size);
                Some((i, element, offset, main_axis_size))
            }
            // We are at the last element, the stretched one.
            Ordering::Equal => {
                let element = self.elements.next()?;
                debug_assert!(self.elements.next().is_none());

                Some((
                    self.i,
                    element,
                    self.start_offset,
                    self.end_offset - self.start_offset,
                ))
            }
            // We are after the stretched element and are moving backwards.
            Ordering::Greater => {
                let i = self.i;
                self.i -= 1;

                let element = self.elements.next_back()?;
                let main_axis_size = min(
                    match self.axis {
                        Axis::X => element.width(self.cross_axis_size).0,
                        Axis::Y => element.height(self.cross_axis_size).0,
                    },
                    self.end_offset - self.start_offset,
                );

                self.end_offset -= main_axis_size;
                Some((i, element, self.end_offset, main_axis_size))
            }
        }
    }
}

impl<E, Event> Element for Stretch<E>
where
    for<'a> E: Collection<'a, Event = Event>,
{
    type Event = Event;

    fn draw(&self, output: &mut dyn Output) {
        let (main_axis_size, cross_axis_size) = self.axis.main_cross_of(output.size());

        for (i, element, offset, size) in self.layout(main_axis_size, Some(cross_axis_size)) {
            let size = self.axis.vec(size, cross_axis_size);

            element.draw(
                &mut output
                    .area(self.axis.vec(offset, 0), size)
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
            Axis::X => (
                self.elements
                    .iter()
                    .enumerate()
                    .map(|(i, element)| {
                        if i == self.stretched {
                            0
                        } else {
                            element.width(height).0
                        }
                    })
                    .sum::<u16>(),
                u16::MAX,
            ),
            Axis::Y => height.map_or_else(
                || combine_cross_axes(self.elements.iter().map(|element| element.width(None))),
                |height| {
                    combine_cross_axes(
                        self.layout(height, None)
                            .map(|(_, element, _, size)| element.width(Some(size))),
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
                        self.layout(width, None)
                            .map(|(_, element, _, size)| element.height(Some(size))),
                    )
                },
            ),
            Axis::Y => (
                self.elements
                    .iter()
                    .enumerate()
                    .map(|(i, element)| {
                        if i == self.stretched {
                            0
                        } else {
                            element.height(width).0
                        }
                    })
                    .sum::<u16>(),
                u16::MAX,
            ),
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

                for (_, element, offset, size) in self.layout(main_axis_size, Some(cross_axis_size))
                {
                    let local_main_axis = mouse_main_axis
                        .checked_sub(offset)
                        .filter(|&pos| pos < size);

                    if let Some(local_main_axis) = local_main_axis {
                        let mut mouse = mouse;
                        mouse.at = self.axis.vec(local_main_axis, mouse_cross_axis);
                        mouse.size = self.axis.vec(size, cross_axis_size);
                        element.handle(Input::Mouse(mouse), events);
                        break;
                    }
                }
            }
        }
    }
}

/// Create a column of elements with the [`Stretch`](struct.Stretch.html) layout.
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn stretch_column<E: for<'a> Collection<'a>>(stretched: usize, elements: E) -> Stretch<E> {
    Stretch {
        elements,
        axis: Axis::Y,
        stretched,
        broadcast_keys: false,
        focused: None,
    }
}

/// Create a row of elements with the [`Stretch`](struct.Stretch.html) layout.
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn stretch_row<E: for<'a> Collection<'a>>(stretched: usize, elements: E) -> Stretch<E> {
    Stretch {
        elements,
        axis: Axis::X,
        stretched,
        broadcast_keys: false,
        focused: None,
    }
}
