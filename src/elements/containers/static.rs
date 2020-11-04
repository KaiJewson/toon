use crate::Element;

use super::{Axis, Collection, InnerElement, Layout1D};

use self::private::Layout;

/// A static [`Layout1D`](trait.Layout1D.html).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Static;

impl<'a, C: Collection<'a>> Layout1D<'a, C> for Static {
    type Layout = Layout<<C as Collection<'a>>::Iter>;

    fn layout(
        &'a self,
        elements: &'a C,
        main_axis_size: u16,
        cross_axis_size: Option<u16>,
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

    fn main_axis_size(
        &'a self,
        elements: &'a C,
        cross_axis_size: Option<u16>,
        axis: Axis,
    ) -> (u16, u16) {
        let size: u16 = elements
            .iter()
            .map(|element| match axis {
                Axis::X => element.width(cross_axis_size).0,
                Axis::Y => element.height(cross_axis_size).0,
            })
            .sum();

        (size, size)
    }
}

mod private {
    use super::super::Axis;

    #[derive(Debug)]
    pub struct Layout<I> {
        pub(super) elements: I,
        pub(super) index: usize,
        pub(super) offset: u16,

        pub(super) main_axis_size: u16,
        pub(super) cross_axis_size: Option<u16>,
        pub(super) axis: Axis,
    }
}

impl<'a, I, Event: 'a> Iterator for Layout<I>
where
    I: Iterator<Item = &'a dyn Element<Event = Event>> + 'a,
{
    type Item = InnerElement<'a, Event>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.main_axis_size {
            return None;
        }

        let element = self.elements.next()?;
        let size = match self.axis {
            Axis::X => element.width(self.cross_axis_size).0,
            Axis::Y => element.height(self.cross_axis_size).0,
        };
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
