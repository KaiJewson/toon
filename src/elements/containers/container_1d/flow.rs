use crate::Element;

use super::{Axis, Collection, InnerElement, Layout1D};

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
        cross_axis_size: u16,
        axis: Axis,
    ) -> Self::Layout {
        let (growth, dividing_point) =
            calculate_layout(main_axis_size, cross_axis_size, elements, axis);

        Layout {
            elements: elements.iter(),
            elements_len: elements.len(),
            index: 0,
            growth,
            dividing_point,
            position_accumulator: 0,
            main_axis_size,
            cross_axis_size,
            axis,
            bias: self.bias,
        }
    }
}

/// Calculate the layout of a flow.
///
/// The first element of the tuple is how much the elements are able to grow along on the main
/// axis. The second element of the tuple gives the index from the front (start bias) or back
/// (end bias) at which the first element of the tuple stops being treated as one more. If there is
/// no bias the value should be ignored.
fn calculate_layout<'a>(
    main_axis_size: u16,
    cross_axis_size: u16,
    elements: &'a impl Collection<'a>,
    axis: Axis,
) -> (u16, usize) {
    if elements.is_empty() {
        return (0, 0);
    }

    let main_axis_extra_space = main_axis_size.saturating_sub(
        elements
            .iter()
            .map(|element| axis.element_size(element, cross_axis_size))
            .fold(0, u16::saturating_add),
    );

    (
        (usize::from(main_axis_extra_space) / elements.len()) as u16,
        usize::from(main_axis_extra_space) % elements.len(),
    )
}

#[derive(Debug)]
pub struct Layout<I> {
    elements: I,
    elements_len: usize,
    index: usize,

    growth: u16,
    dividing_point: usize,

    position_accumulator: u16,

    main_axis_size: u16,
    cross_axis_size: u16,
    axis: Axis,
    bias: Option<End>,
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

        let size = self.axis.element_size(element, self.cross_axis_size);

        let growth_is_more = match self.bias {
            Some(End::Start) => index < self.dividing_point,
            Some(End::End) => self.elements_len - index - 1 < self.dividing_point,
            None => false,
        };
        let growth = if growth_is_more {
            self.growth + 1
        } else {
            self.growth
        };

        let size = size + growth;

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
    use crate::ElementExt;

    let mut grid = crate::Grid::new((5, 6));

    let mut three_lines = crate::column::<_, _, ()>(
        flow(),
        (
            crate::span("Top Line").tile_y(0),
            crate::empty(),
            crate::span("Middle Line").tile_y(0),
            crate::empty(),
            crate::span("Bottom Line").tile_y(0),
        ),
    );

    three_lines.draw(&mut grid);
    assert_eq!(
        grid.contents(),
        ["Top L", "Middl", "Botto", "     ", "     ", "     ",]
    );

    three_lines.layout.bias = Some(End::Start);
    grid.clear();
    three_lines.draw(&mut grid);
    assert_eq!(
        grid.contents(),
        ["Top L", "Top L", "     ", "Middl", "Middl", "Botto",]
    );

    three_lines.layout.bias = Some(End::End);
    grid.clear();
    three_lines.draw(&mut grid);
    assert_eq!(
        grid.contents(),
        ["Top L", "Middl", "Middl", "     ", "Botto", "Botto",]
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
        ["12345", "Middl", "Middl", "     ", "Botto", "Botto",]
    );
}
