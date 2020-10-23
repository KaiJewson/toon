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

impl PartialEq<KeyPress> for Input {
    fn eq(&self, other: &KeyPress) -> bool {
        matches!(self, Self::Key(press) if press == other)
    }
}
impl PartialEq<Input> for KeyPress {
    fn eq(&self, other: &Input) -> bool {
        other == self
    }
}

impl PartialEq<char> for Input {
    fn eq(&self, &other: &char) -> bool {
        *self == KeyPress::from(other)
    }
}
impl PartialEq<Input> for char {
    fn eq(&self, other: &Input) -> bool {
        other == self
    }
}

impl PartialEq<Mouse> for Input {
    fn eq(&self, other: &Mouse) -> bool {
        matches!(self, Self::Mouse(mouse) if mouse == other)
    }
}
impl PartialEq<Input> for Mouse {
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

impl Modifiers {
    /// Returns `true` if no modifiers held down.
    #[must_use]
    pub const fn are_none(self) -> bool {
        !self.shift && !self.control && !self.alt
    }
}

/// A pattern that matches inputs.
///
/// This is implemented for:
/// - Functions that take an input and return a boolean.
/// - [`Input`](enum.Input.html), [`KeyPress`](struct.KeyPress.html), [`Mouse`](struct.Mouse.html)
/// and `char` which just perform an equality check.
/// - [`Key`](enum.Key.html), which does not allow any modifiers to be held down.
/// - [`MouseKind`](enum.MouseKind.html), which can occur at any position without modifiers.
/// - [`MouseButton`](enum.MouseButton.html), which detects when a mouse button was pressed at any
/// position without modifiers.
/// - Tuples, which detect any one of the inputs occurring.
pub trait Pattern {
    /// Whether the pattern matches this input.
    fn matches(&self, input: Input) -> bool;
}

impl<F: Fn(Input) -> bool> Pattern for F {
    fn matches(&self, input: Input) -> bool {
        (self)(input)
    }
}

impl Pattern for Input {
    fn matches(&self, input: Input) -> bool {
        *self == input
    }
}
impl Pattern for KeyPress {
    fn matches(&self, input: Input) -> bool {
        *self == input
    }
}
impl Pattern for Mouse {
    fn matches(&self, input: Input) -> bool {
        *self == input
    }
}
impl Pattern for char {
    fn matches(&self, input: Input) -> bool {
        *self == input
    }
}

impl Pattern for Key {
    fn matches(&self, input: Input) -> bool {
        matches!(input, Input::Key(press) if press.key == *self && press.modifiers.are_none())
    }
}

impl Pattern for MouseKind {
    fn matches(&self, input: Input) -> bool {
        matches!(input, Input::Mouse(mouse) if mouse.kind == *self && mouse.modifiers.are_none())
    }
}

impl Pattern for MouseButton {
    fn matches(&self, input: Input) -> bool {
        MouseKind::Press(*self).matches(input)
    }
}

macro_rules! impl_input_pattern_for_tuples {
    ($(($($param:ident),*),)*) => {
        $(
            impl<$($param: Pattern,)*> Pattern for ($($param,)*) {
                #[allow(unused_variables)]
                fn matches(&self, input: Input) -> bool {
                    #[allow(non_snake_case)]
                    let ($($param,)*) = self;
                    false
                    $(|| $param.matches(input))*
                }
            }
        )*
    }
}
impl_input_pattern_for_tuples! {
    (),
    (A),
    (A, B),
    (A, B, C),
    (A, B, C, D),
    (A, B, C, D, E),
    (A, B, C, D, E, F),
    (A, B, C, D, E, F, G),
    (A, B, C, D, E, F, G, H),
    (A, B, C, D, E, F, G, H, I),
    (A, B, C, D, E, F, G, H, I, J),
    (A, B, C, D, E, F, G, H, I, J, K),
    (A, B, C, D, E, F, G, H, I, J, K, L),
}
