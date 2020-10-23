use std::io::Write;

use crossterm::event::{
    Event, EventStream, KeyCode, KeyModifiers, MouseButton as CMouseButton, MouseEvent,
};
use crossterm::style::{self, Attribute, Color as CColor};
use crossterm::{cursor, event, terminal};
use crossterm::{execute, queue};
use crossterm_crate as crossterm;
use futures_util::future::{self, FutureExt};
use futures_util::stream::{self, StreamExt};

use crate::input::{Input, Key, KeyPress, Modifiers, Mouse, MouseButton, MouseKind};
use crate::style::{Color, Intensity, Rgb};
use crate::{CursorShape, Vec2};

use super::{Backend, ReadEvents, TerminalEvent, Tty};

/// Crossterm backend.
///
/// Currently there is no configuration here.
///
/// Crossterm supports all features except setting the cursor shape (see
/// <https://github.com/crossterm-rs/crossterm/issues/427>).
#[cfg_attr(feature = "nightly", doc(cfg(feature = "crossterm")))]
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
pub struct Crossterm {}

impl Backend for Crossterm {
    type Error = crossterm::ErrorKind;
    type Bound = Bound;

    fn bind(self, mut io: Tty) -> Result<Self::Bound, Self::Error> {
        terminal::enable_raw_mode()?;
        execute!(
            io,
            terminal::EnterAlternateScreen,
            terminal::Clear(terminal::ClearType::All),
            terminal::DisableLineWrap,
            event::EnableMouseCapture,
        )?;

        Ok(Bound {
            io,
            stream: EventStream::new(),
        })
    }
}

#[derive(Debug)]
pub struct Bound {
    io: Tty,
    stream: EventStream,
}

impl super::Bound for Bound {
    type Error = crossterm::ErrorKind;

    // General functions

    fn size(&mut self) -> Result<Vec2<u16>, Self::Error> {
        terminal::size().map(Vec2::from)
    }
    fn set_title(&mut self, title: &str) -> Result<(), Self::Error> {
        queue!(self.io, terminal::SetTitle(title))
    }

    // Cursor functions

    fn hide_cursor(&mut self) -> Result<(), Self::Error> {
        queue!(self.io, cursor::Hide)
    }
    fn show_cursor(&mut self) -> Result<(), Self::Error> {
        queue!(self.io, cursor::Show)
    }
    fn set_cursor_shape(&mut self, _shape: CursorShape) -> Result<(), Self::Error> {
        // BlockedTODO: https://github.com/crossterm-rs/crossterm/issues/427
        Ok(())
    }
    fn set_cursor_blinking(&mut self, blinking: bool) -> Result<(), Self::Error> {
        if blinking {
            queue!(self.io, cursor::EnableBlinking)
        } else {
            queue!(self.io, cursor::DisableBlinking)
        }
    }
    fn set_cursor_pos(&mut self, pos: Vec2<u16>) -> Result<(), Self::Error> {
        queue!(self.io, cursor::MoveTo(pos.x, pos.y))
    }

    // Style functions
    fn set_foreground(&mut self, foreground: Color) -> Result<(), Self::Error> {
        queue!(
            self.io,
            style::SetForegroundColor(to_crossterm_color(foreground))
        )
    }
    fn set_background(&mut self, background: Color) -> Result<(), Self::Error> {
        queue!(
            self.io,
            style::SetBackgroundColor(to_crossterm_color(background))
        )
    }
    fn set_intensity(&mut self, intensity: Intensity) -> Result<(), Self::Error> {
        queue!(
            self.io,
            style::SetAttribute(match intensity {
                Intensity::Dim => Attribute::Dim,
                Intensity::Normal => Attribute::NormalIntensity,
                Intensity::Bold => Attribute::Bold,
            })
        )
    }
    fn set_italic(&mut self, italic: bool) -> Result<(), Self::Error> {
        queue!(
            self.io,
            style::SetAttribute(if italic {
                Attribute::Italic
            } else {
                Attribute::NoItalic
            })
        )
    }
    fn set_underlined(&mut self, underlined: bool) -> Result<(), Self::Error> {
        queue!(
            self.io,
            style::SetAttribute(if underlined {
                Attribute::Underlined
            } else {
                Attribute::NoUnderline
            })
        )
    }
    fn set_blinking(&mut self, blinking: bool) -> Result<(), Self::Error> {
        queue!(
            self.io,
            style::SetAttribute(if blinking {
                Attribute::RapidBlink
            } else {
                Attribute::NoBlink
            })
        )
    }
    fn set_crossed_out(&mut self, crossed_out: bool) -> Result<(), Self::Error> {
        queue!(
            self.io,
            style::SetAttribute(if crossed_out {
                Attribute::CrossedOut
            } else {
                Attribute::NotCrossedOut
            })
        )
    }

    // Writing

    fn write(&mut self, text: &str) -> Result<(), Self::Error> {
        self.io.write_all(text.as_bytes())?;
        Ok(())
    }

    // Finalizing functions

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.io.flush()?;
        Ok(())
    }
    fn reset(mut self) -> Result<Tty, Self::Error> {
        execute!(
            self.io,
            terminal::LeaveAlternateScreen,
            event::DisableMouseCapture,
            cursor::Show,
        )?;
        terminal::disable_raw_mode()?;

        Ok(self.io)
    }
}

#[allow(clippy::type_complexity)]
impl<'a> ReadEvents<'a> for Bound {
    type EventError = <Self as super::Bound>::Error;
    type EventFuture = future::Map<
        stream::Next<'a, EventStream>,
        fn(Option<crossterm::Result<Event>>) -> crossterm::Result<TerminalEvent>,
    >;

    fn read_event(&'a mut self) -> Self::EventFuture {
        self.stream
            .next()
            .map(|item| item.unwrap().map(from_crossterm_event))
    }
}

fn to_crossterm_color(color: Color) -> CColor {
    match color {
        Color::Default => CColor::Reset,
        Color::Black => CColor::Black,
        Color::DarkGray => CColor::DarkGrey,
        Color::LightGray => CColor::Grey,
        Color::White => CColor::White,
        Color::Red => CColor::Red,
        Color::DarkRed => CColor::DarkRed,
        Color::Green => CColor::Green,
        Color::DarkGreen => CColor::DarkGreen,
        Color::Yellow => CColor::Yellow,
        Color::DarkYellow => CColor::DarkYellow,
        Color::Blue => CColor::Blue,
        Color::DarkBlue => CColor::DarkBlue,
        Color::Magenta => CColor::Magenta,
        Color::DarkMagenta => CColor::DarkMagenta,
        Color::Cyan => CColor::Cyan,
        Color::DarkCyan => CColor::DarkCyan,
        Color::AnsiValue(v) => CColor::AnsiValue(v.get()),
        Color::Rgb(Rgb { r, g, b }) => CColor::Rgb { r, g, b },
    }
}

fn from_crossterm_event(event: Event) -> TerminalEvent {
    match event {
        Event::Key(key) => TerminalEvent::Input(Input::Key(KeyPress {
            key: match key.code {
                KeyCode::Backspace => Key::Backspace,
                KeyCode::Enter => Key::Char('\n'),
                KeyCode::Left => Key::Left,
                KeyCode::Right => Key::Right,
                KeyCode::Up => Key::Up,
                KeyCode::Down => Key::Down,
                KeyCode::Home => Key::Home,
                KeyCode::End => Key::End,
                KeyCode::PageUp => Key::PageUp,
                KeyCode::PageDown => Key::PageDown,
                KeyCode::Tab | KeyCode::BackTab => Key::Char('\t'),
                KeyCode::Delete => Key::Char('\x7f'),
                KeyCode::Insert => Key::Insert,
                KeyCode::F(n) => Key::F(n),
                KeyCode::Char(c) => Key::Char(c.to_ascii_lowercase()),
                KeyCode::Null => Key::Char('\0'),
                KeyCode::Esc => Key::Escape,
            },
            modifiers: {
                let mut modifiers = from_crossterm_modifiers(key.modifiers);
                modifiers.shift = modifiers.shift
                    || key.code == KeyCode::BackTab
                    || matches!(key.code, KeyCode::Char(c) if c.is_uppercase());
                modifiers
            },
        })),
        Event::Mouse(mouse) => TerminalEvent::Input(Input::Mouse({
            let (kind, x, y, modifiers) = match mouse {
                MouseEvent::Down(button, x, y, modifiers) => (
                    MouseKind::Press(from_crossterm_mouse_button(button)),
                    x,
                    y,
                    modifiers,
                ),
                MouseEvent::Up(_, x, y, m) => (MouseKind::Release, x, y, m),
                MouseEvent::Drag(_, x, y, m) => (MouseKind::Hold, x, y, m),
                MouseEvent::ScrollDown(x, y, m) => (MouseKind::ScrollDown, x, y, m),
                MouseEvent::ScrollUp(x, y, m) => (MouseKind::ScrollUp, x, y, m),
            };
            Mouse {
                kind,
                at: Vec2 { x, y },
                // Anything can go here
                size: Vec2::default(),
                modifiers: from_crossterm_modifiers(modifiers),
            }
        })),
        Event::Resize(x, y) => TerminalEvent::Resize(Vec2 { x, y }),
    }
}
fn from_crossterm_mouse_button(button: CMouseButton) -> MouseButton {
    match button {
        CMouseButton::Left => MouseButton::Left,
        CMouseButton::Middle => MouseButton::Middle,
        CMouseButton::Right => MouseButton::Right,
    }
}
fn from_crossterm_modifiers(modifiers: KeyModifiers) -> Modifiers {
    Modifiers {
        shift: modifiers.contains(KeyModifiers::SHIFT),
        control: modifiers.contains(KeyModifiers::CONTROL),
        alt: modifiers.contains(KeyModifiers::ALT),
    }
}
