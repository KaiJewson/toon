//! Developer tools for Toon.

#[cfg(not(feature = "either"))]
compile_error!("Dev mode currently requires `either` feature to be active.");

use std::cmp::max;

use either_crate::Either;
use futures_codec::Bytes;
use futures_util::future;
use futures_util::stream::{Stream, StreamExt};

use crate::{
    input, Alignment, Border, Captured, Color, Element, ElementExt, End, Mouse, MouseButton,
    MouseKind, Styled,
};

/// The state of the developer tools.
#[derive(Debug)]
pub struct Dev {
    /// Whether the dev panel is focused.
    focus: Focus,

    /// The width of the right dev panel.
    right_panel_width: u16,
    /// Whether the user is mouse resizing the right dev panel.
    right_panel_resizing: bool,

    /// The height of the bottom dev panel.
    bottom_panel_height: u16,
    /// Whether the user is mouse resizing the bottom dev panel.
    bottom_panel_resizing: bool,

    /// Whether the abort confirmation dialogue box is being shown.
    abort_confirm: bool,

    /// Data that has been read from the captured stdio.
    captured: String,
}

impl Dev {
    /// Create new developer tools state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            focus: Focus::Element,
            right_panel_width: 64,
            right_panel_resizing: false,
            bottom_panel_height: 16,
            bottom_panel_resizing: false,
            abort_confirm: false,
            captured: String::new(),
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
        let bottom_panel = self.bottom_panel().map_event(Into::into);
        let inner = self.inner(inner);

        let element = crate::stretch_row(
            0,
            if self.focus == Focus::RightDev {
                Either::Left((
                    crate::stretch_column(
                        0,
                        (
                            inner.on_passive(
                                (MouseKind::Press(MouseButton::Left), MouseKind::Move),
                                |_| EventKind::Focus(Focus::Element).into(),
                            ),
                            bottom_panel.on_passive(
                                (MouseKind::Press(MouseButton::Left), MouseKind::Move),
                                |_| EventKind::Focus(Focus::BottomDev).into(),
                            ),
                        ),
                    ),
                    right_panel,
                ))
            } else {
                Either::Right((
                    crate::stretch_column(
                        0,
                        if self.focus == Focus::BottomDev {
                            Either::Left((
                                inner.on_passive(
                                    (MouseKind::Press(MouseButton::Left), MouseKind::Move),
                                    |_| EventKind::Focus(Focus::Element).into(),
                                ),
                                bottom_panel,
                            ))
                        } else {
                            Either::Right((
                                inner,
                                bottom_panel.on_passive(
                                    (MouseKind::Press(MouseButton::Left), MouseKind::Move),
                                    |_| EventKind::Focus(Focus::BottomDev).into(),
                                ),
                            ))
                        },
                    )
                    .focus(if self.focus == Focus::BottomDev { 1 } else { 0 }),
                    right_panel.on_passive(
                        (MouseKind::Press(MouseButton::Left), MouseKind::Move),
                        |_| EventKind::Focus(Focus::RightDev).into(),
                    ),
                ))
            },
        )
        .broadcast_keys()
        .focus(if self.focus == Focus::RightDev { 1 } else { 0 })
        .on(input!(Key(Tab)), move |input| {
            EventKind::Focus(self.focus.tab(input.modifiers().shift)).into()
        })
        .on(input!(Alt + Shift + Key(h)), move |_| {
            EventKind::Resize(Some(self.right_panel_width.saturating_add(2)), None).into()
        })
        .on(input!(Alt + Shift + Key(l)), move |_| {
            EventKind::Resize(Some(self.right_panel_width - 2), None).into()
        });

        let resizing = self.right_panel_resizing || self.bottom_panel_resizing;

        let element = element
            .on(input!(Mouse(Drag) where (|_| resizing)), move |input| {
                let mouse = input.mouse().unwrap();
                EventKind::Resize(
                    if self.right_panel_resizing {
                        Some(mouse.size.x - mouse.at.x)
                    } else {
                        None
                    },
                    if self.bottom_panel_resizing {
                        Some(mouse.size.y - mouse.at.y)
                    } else {
                        None
                    },
                )
                .into()
            })
            .on(input!(Mouse(Release) where (|_| resizing)), |_| {
                EventKind::StopResizing.into()
            });

        if self.abort_confirm {
            Either::Left(
                crate::stack((
                    element
                        .mask_inputs(())
                        .on(input!(Mouse(Release Left)), |_| {
                            EventKind::ToggleAbortConfirm.into()
                        }),
                    Self::abort_confirmation().map_event(Into::into),
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
                .foreground(if self.focus == Focus::RightDev && !self.abort_confirm {
                    Color::White
                } else {
                    Color::DarkGray
                })
                .top_title(Alignment::Start),
        )
        .min_width(self.right_panel_width)
        .on(input!(Mouse(Press Left) at (0, _)), |_| {
            EventKind::SetRightPanelResizing
        })
    }

    /// Create the bottom panel of the developer tools.
    fn bottom_panel(&self) -> impl Element<Event = EventKind> + '_ {
        // TODO: Make this more efficient.
        let contents = crate::column(
            self.captured
                .lines()
                .map(ToOwned::to_owned)
                .map(crate::span)
                .collect::<Vec<_>>(),
        )
        .scroll_y(crate::ScrollOffset::End(0));

        contents
            .title("Console")
            .filter(
                Border::THIN_CURVED
                    .foreground(if self.focus == Focus::BottomDev && !self.abort_confirm {
                        Color::White
                    } else {
                        Color::DarkGray
                    })
                    .top_title(Alignment::Start),
            )
            .size_range(
                (0, self.bottom_panel_height),
                (u16::MAX, self.bottom_panel_height),
            )
            .on(input!(Mouse(Press Left) at (_, 0)), |_| {
                EventKind::SetBottomPanelResizing
            })
    }

    /// Create the element panel.
    fn inner<E: Element>(&self, inner: E) -> impl Element<Event = AppEvent<E::Event>> {
        inner
            .map_event(AppEvent::Element)
            .filter(
                Border::THIN_CURVED
                    .foreground(if self.focus == Focus::Element && !self.abort_confirm {
                        Color::White
                    } else {
                        Color::DarkGray
                    })
                    .top_title(Alignment::Start),
            )
            .size_range((2, 2), (u16::MAX, u16::MAX))
            .on_passive(
                input!(Mouse(Press Left) where (|m: Mouse| m.at.x == m.size.x.saturating_sub(1))),
                |_| EventKind::SetRightPanelResizing.into(),
            )
            .on_passive(
                input!(Mouse(Press Left) where (|m: Mouse| m.at.y == m.size.y.saturating_sub(1))),
                |_| EventKind::SetBottomPanelResizing.into(),
            )
    }

    /// Create a abort confirmation dialogue box.
    fn abort_confirmation() -> impl Element<Event = EventKind> {
        crate::column((
            crate::span("Are you sure you sure you want to abort the process?"),
            crate::row((
                crate::fill(Color::Default),
                crate::span("Yes")
                    .filter(Border::THIN)
                    .on(input!(Mouse(Release Left)), |_| std::process::abort()),
                crate::span("No")
                    .filter(Border::THIN)
                    .on(input!(Mouse(Release Left)), |_| {
                        EventKind::ToggleAbortConfirm
                    }),
                crate::fill(Color::Default),
            ))
            .bias(End::Start),
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
            EventKind::Focus(focus) => {
                self.focus = focus;
            }
            EventKind::ToggleAbortConfirm => {
                self.abort_confirm = !self.abort_confirm;
            }
            EventKind::Resize(right_panel, bottom_panel) => {
                if let Some(right_panel) = right_panel {
                    self.right_panel_width = max(right_panel, 32);
                }
                if let Some(bottom_panel) = bottom_panel {
                    self.bottom_panel_height = max(bottom_panel, 8);
                }
            }
            EventKind::SetRightPanelResizing => {
                self.right_panel_resizing = true;
            }
            EventKind::SetBottomPanelResizing => {
                self.bottom_panel_resizing = true;
            }
            EventKind::StopResizing => {
                self.right_panel_resizing = false;
                self.bottom_panel_resizing = false;
            }
            EventKind::CapturedData(s) => {
                self.captured.push_str(&String::from_utf8_lossy(&s));
            }
        }
    }
}

impl Default for Dev {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Dev {
    fn drop(&mut self) {
        eprintln!("{}", self.captured);
    }
}

/// Which part of dev tools is focused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    /// The right dev panel.
    RightDev,
    /// The bottom dev panel.
    BottomDev,
    /// The element.
    Element,
}

impl Focus {
    fn tab(self, back: bool) -> Self {
        if back {
            match self {
                Self::RightDev => Self::Element,
                Self::BottomDev => Self::RightDev,
                Self::Element => Self::BottomDev,
            }
        } else {
            match self {
                Self::RightDev => Self::BottomDev,
                Self::BottomDev => Self::Element,
                Self::Element => Self::RightDev,
            }
        }
    }
}

/// An event in your application, either caused by developer tools or by your element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEvent<T> {
    /// An event in the developer tools themselves.
    Dev(Event),
    /// An event in your element.
    Element(T),
}

/// An event in developer tools.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event(EventKind);

#[derive(Debug, Clone, PartialEq, Eq)]
enum EventKind {
    Focus(Focus),
    ToggleAbortConfirm,
    Resize(Option<u16>, Option<u16>),
    SetRightPanelResizing,
    SetBottomPanelResizing,
    StopResizing,
    CapturedData(Bytes),
}

impl<T> From<EventKind> for AppEvent<T> {
    fn from(kind: EventKind) -> Self {
        Self::Dev(Event(kind))
    }
}

/// Create a stream of developer tools events from a program's captured stdio. This stream will
/// never terminate.
///
/// Passing these events to a developer tools will display them on the bottom panel, and it will
/// all be printed to the standard error when the program exits.
pub fn display_captured(captured: Captured) -> impl Stream<Item = Event> + Unpin {
    futures_codec::FramedRead::new(blocking::Unblock::new(captured), futures_codec::BytesCodec)
        .filter_map(|res| future::ready(res.ok()))
        .map(EventKind::CapturedData)
        .map(Event)
}
