use crate::Element;

use super::{Axis, Collection, InnerElement, Layout1D};

/// A static [`Layout1D`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Static;

impl<'a, C: Collection<'a>> Layout1D<'a, C> for Static {
    type Layout = Layout<<C as Collection<'a>>::Iter>;

    fn layout(
        &'a self,
        elements: &'a C,
        main_axis_size: u16,
        cross_axis_size: u16,
        axis: Axis,
    ) -> Self::Layout {
        Layout {
            elements: elements.iter(),
            index: 0,
            offset: 0,
            main_axis_size,
            cross_axis_size,
            axis,
        }
    }
}

#[derive(Debug)]
pub struct Layout<I> {
    elements: I,
    index: usize,
    offset: u16,

    main_axis_size: u16,
    cross_axis_size: u16,
    axis: Axis,
}

impl<'a, I, Event: 'a> Iterator for Layout<I>
where
    I: Iterator<Item = &'a dyn Element<Event = Event>>,
{
    type Item = InnerElement<'a, Event>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.main_axis_size {
            return None;
        }

        let element = self.elements.next()?;
        let size = self.axis.element_size(element, self.cross_axis_size);
        let index = self.index;
        let position = self.offset;

        self.offset = self.offset.saturating_add(size);
        self.index += 1;

        Some(InnerElement {
            element,
            index,
            position,
            size,
        })
    }
}

#[test]
fn test_static() {
    let mut grid = crate::Grid::new((7, 3));

    crate::column::<_, _, ()>(
        Static,
        (
            crate::span("Some text"),
            crate::empty(),
            crate::column(Static, (crate::span("Foo"), crate::span("Bar"))),
            crate::span("Cut off"),
        ),
    )
    .draw(&mut grid);

    assert_eq!(grid.contents(), ["Some te", "Foo    ", "Bar    ",]);
}
