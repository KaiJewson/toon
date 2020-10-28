//! Developer tools for Toon.

use std::cmp::max;

use either_crate::Either;

use crate::{
    input, Alignment, Border, Color, Element, ElementExt, Input, Mouse, MouseButton, MouseKind,
    Styled,
};

/// The state of the developer tools.
#[derive(Debug, Clone)]
pub struct Dev {
    /// Whether the dev panel is focused.
    dev_focused: bool,

    /// The width of the right dev panel.
    right_panel_width: u16,
    /// Whether the user is mouse resizing the right dev panel.
    right_panel_resizing: bool,

    /// Whether the abort confirmation dialogue box is being shown.
    abort_confirm: bool,
}

impl Dev {
    /// Create new developer tools state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            dev_focused: false,
            right_panel_width: 64,
            right_panel_resizing: false,
            abort_confirm: false,
        }
    }

    /// Wrap the inner element in developer tools.
    #[must_use]
    pub fn wrap<'a, E: Element + 'a>(
        &'a self,
        inner: E,
    ) -> impl Element<Event = AppEvent<E::Event>> + 'a
    where
        <E as Element>::Event: 'static,
    {
        let right_panel = self.right_panel().map_event(Into::into);
        let inner = self.inner(inner);

        let right_panel = right_panel.on(input!(Mouse(Press Left) at (0, _)), |_| {
            EventKind::SetRightPanelResizing(true).into()
        });
        let inner = inner.on(
            input!(Mouse(Press Left) where (|m: Mouse| m.at.x == m.size.x.saturating_sub(1))),
            |_| EventKind::SetRightPanelResizing(true).into(),
        );

        let element = crate::row(if self.dev_focused {
            Either::Left((
                inner.on_passive(
                    (MouseKind::Press(MouseButton::Left), MouseKind::Move),
                    |_| EventKind::FocusElement.into(),
                ),
                right_panel,
            ))
        } else {
            Either::Right((
                inner,
                right_panel.on_passive(
                    (MouseKind::Press(MouseButton::Left), MouseKind::Move),
                    |_| EventKind::FocusDev.into(),
                ),
            ))
        })
        .focus(if self.dev_focused { 1 } else { 0 })
        .on(
            input!(Alt + Key(Char if self.dev_focused { 'h' } else { 'l' })),
            move |_| {
                if self.dev_focused {
                    EventKind::FocusElement
                } else {
                    EventKind::FocusDev
                }
                .into()
            },
        )
        .on(input!(Alt + Shift + Key(h)), move |_| {
            EventKind::ResizeRightPanel(self.right_panel_width.saturating_add(2)).into()
        })
        .on(input!(Alt + Shift + Key(l)), move |_| {
            EventKind::ResizeRightPanel(self.right_panel_width - 2).into()
        });

        let element = if self.right_panel_resizing {
            Either::Left(
                element
                    .on(input!(Mouse(Drag)), |input| {
                        let mouse = match input {
                            Input::Mouse(mouse) => mouse,
                            Input::Key(_) => unreachable!(),
                        };
                        EventKind::ResizeRightPanel(mouse.size.x - mouse.at.x).into()
                    })
                    .on(input!(Mouse(Release)), |_| {
                        EventKind::SetRightPanelResizing(false).into()
                    }),
            )
        } else {
            Either::Right(element)
        };

        if self.abort_confirm {
            Either::Left(
                crate::stack((
                    element.on(input!(Mouse(Release Left)), |_| {
                        EventKind::ToggleAbortConfirm.into()
                    }),
                    Self::abort_confimation().map_event(Into::into),
                ))
                .broadcast_inputs(),
            )
        } else {
            Either::Right(element)
        }
    }

    /// Create the right panel of the developer tools.
    fn right_panel(&self) -> impl Element<Event = EventKind> + '_ {
        crate::column((
            crate::span("Panic!")
                .bold()
                .foreground(Color::Red)
                .filter(Border::THICK.foreground(Color::Red))
                .on(input!(Mouse(Release Left)), |_| panic!("Dev panel"))
                .float_x(Alignment::Start),
            crate::span("Abort!")
                .bold()
                .foreground(Color::Red)
                .filter(Border::THICK.foreground(Color::Red))
                .on(input!(Mouse(Release Left)), |_| {
                    EventKind::ToggleAbortConfirm
                })
                .float_x(Alignment::Start),
        ))
        .title("Dev panel")
        .filter(
            Border::THIN_CURVED
                .foreground(if self.dev_focused && !self.abort_confirm {
                    Color::White
                } else {
                    Color::DarkGray
                })
                .top_title(Alignment::Start),
        )
        .width_range(self.right_panel_width, self.right_panel_width)
    }

    /// Create the inner panel of the developer tools.
    fn inner<E: Element>(&self, inner: E) -> impl Element<Event = AppEvent<E::Event>> {
        inner
            .map_event(AppEvent::Element)
            .filter(
                Border::THIN_CURVED
                    .foreground(if self.dev_focused || self.abort_confirm {
                        Color::DarkGray
                    } else {
                        Color::White
                    })
                    .top_title(Alignment::Start),
            )
            .width_range(2, u16::MAX)
    }

    /// Create a abort confirmation dialogue box.
    fn abort_confimation() -> impl Element<Event = EventKind> {
        crate::column((
            crate::span("Are you sure you sure you want to abort the process?"),
            crate::row((
                crate::empty(),
                crate::span("Yes")
                    .filter(Border::THIN)
                    .on(input!(Mouse(Release Left)), |_| std::process::abort()),
                crate::span("No")
                    .filter(Border::THIN)
                    .on(input!(Mouse(Release Left)), |_| {
                        EventKind::ToggleAbortConfirm
                    }),
                crate::empty(),
            )),
        ))
        .filter(Border::THICK)
        .on_passive(input!(Mouse(Release Left)), |_| {
            // Clicking on the popup will cause two ToggleAbortConfirm events, one on
            // the background and one here, which has the effect of keeping the popup
            // present
            EventKind::ToggleAbortConfirm
        })
        .float((Alignment::Middle, Alignment::Middle))
    }

    /// Apply the event to the developer tools state.
    pub fn apply(&mut self, event: Event) {
        match event.0 {
            EventKind::FocusDev => {
                self.dev_focused = true;
            }
            EventKind::FocusElement => {
                self.dev_focused = false;
            }
            EventKind::ToggleAbortConfirm => {
                self.abort_confirm = !self.abort_confirm;
            }
            EventKind::ResizeRightPanel(width) => {
                self.right_panel_width = max(width, 32);
            }
            EventKind::SetRightPanelResizing(resizing) => {
                self.right_panel_resizing = resizing;
            }
        }
    }
}

impl Default for Dev {
    fn default() -> Self {
        Self::new()
    }
}

/// An event in your application, either caused by developer tools or by your element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEvent<T> {
    /// An event in the developer tools themselves.
    Dev(Event),
    /// An event in your element.
    Element(T),
}

/// An event in developer tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Event(EventKind);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventKind {
    FocusDev,
    FocusElement,
    ToggleAbortConfirm,
    ResizeRightPanel(u16),
    SetRightPanelResizing(bool),
}

impl<T> From<EventKind> for AppEvent<T> {
    fn from(kind: EventKind) -> Self {
        Self::Dev(Event(kind))
    }
}
