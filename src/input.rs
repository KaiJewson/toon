//! Terminal inputs, such as keypresses, clicks and resizes.

use crate::Vec2;

/// A user input on the terminal.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Input {
    /// A key was pressed.
    Key(KeyPress),
    /// A mouse button was pressed, released or dragged, or the mouse wheel was scrolled.
    Mouse(Mouse),
}

impl From<KeyPress> for Input {
    fn from(input: KeyPress) -> Self {
        Self::Key(input)
    }
}
impl From<Mouse> for Input {
    fn from(input: Mouse) -> Self {
        Self::Mouse(input)
    }
}
impl From<char> for Input {
    fn from(key: char) -> Self {
        Self::Key(KeyPress::from(key))
    }
}
impl PartialEq<char> for Input {
    fn eq(&self, &other: &char) -> bool {
        *self == Self::from(other)
    }
}
impl PartialEq<Input> for char {
    fn eq(&self, other: &Input) -> bool {
        other == self
    }
}

/// A key was pressed.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct KeyPress {
    /// Which key was pressed.
    pub key: Key,
    /// The modifiers active while the key was pressed.
    pub modifiers: Modifiers,
}

impl From<char> for KeyPress {
    fn from(key: char) -> Self {
        Self {
            key: Key::Char(key.to_ascii_lowercase()),
            modifiers: Modifiers {
                shift: key.is_ascii_uppercase(),
                ..Modifiers::default()
            },
        }
    }
}

impl PartialEq<char> for KeyPress {
    fn eq(&self, &other: &char) -> bool {
        *self == Self::from(other)
    }
}
impl PartialEq<KeyPress> for char {
    fn eq(&self, other: &KeyPress) -> bool {
        other == self
    }
}

/// Any key.
///
/// The enter/return, tab and delete keys are mapped to their respective ASCII characters.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Key {
    /// The backspace key.
    Backspace,
    /// The left arrow.
    Left,
    /// The right arrow.
    Right,
    /// The up arrow.
    Up,
    /// The down arrow.
    Down,
    /// The Home key,
    Home,
    /// The End key.
    End,
    /// The page up key.
    PageUp,
    /// The page down key.
    PageDown,
    /// The insert key.
    Insert,
    /// The escape key.
    Escape,
    /// A function key (e.g. F(5) is F5).
    F(u8),
    /// A key which maps to a character. This character will never be uppercase.
    Char(char),
}

/// A mouse button was pressed, released or dragged, or the mouse wheel was scrolled.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Mouse {
    /// What kind of mouse input it is.
    pub kind: MouseKind,
    /// Where the input occurred, zero-indexed.
    pub at: Vec2<u16>,
    /// The size of the output that captured the mouse input.
    pub size: Vec2<u16>,
    /// The modifiers active while the input occurred. Only some terminals report this.
    pub modifiers: Modifiers,
}

/// A kind of mouse input.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum MouseKind {
    /// A mouse button was pressed.
    Press(MouseButton),
    /// A mouse button was released.
    Release,
    /// A mouse button was held or dragged.
    Hold,
    /// The scroll wheel was scrolled down.
    ScrollDown,
    /// The scroll wheel was scrolled up.
    ScrollUp,
}

/// A mouse button.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum MouseButton {
    /// The left mouse button.
    Left,
    /// The middle mouse button; usually the scroll wheel.
    Middle,
    /// The right mouse button.
    Right,
}

/// Key modifiers.
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Modifiers {
    /// The shift key.
    pub shift: bool,
    /// The control key.
    pub control: bool,
    /// The alt key.
    pub alt: bool,
}
