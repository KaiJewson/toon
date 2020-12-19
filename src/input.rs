//! Terminal inputs, such as keypresses, clicks and resizes.

use std::ops::{BitOr, BitOrAssign};

use crate::Vec2;

/// A user input on the terminal.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Input {
    /// A key was pressed.
    Key(KeyPress),
    /// A mouse button was pressed, released or dragged, or the mouse wheel was scrolled.
    Mouse(Mouse),
}

impl Input {
    /// Get the key press of the input.
    #[must_use]
    pub fn key(self) -> Option<KeyPress> {
        match self {
            Self::Key(press) => Some(press),
            Self::Mouse(_) => None,
        }
    }
    /// Get the mouse input of the input.
    #[must_use]
    pub fn mouse(self) -> Option<Mouse> {
        match self {
            Self::Key(_) => None,
            Self::Mouse(mouse) => Some(mouse),
        }
    }

    /// Get the modifiers of the input.
    #[must_use]
    pub fn modifiers(&self) -> Modifiers {
        match self {
            Self::Key(press) => press.modifiers,
            Self::Mouse(mouse) => mouse.modifiers,
        }
    }
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
    Release(MouseButton),
    /// The mouse was moved with a button held down.
    Drag(MouseButton),
    /// The mouse was moved with no buttons held down.
    Move,
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
    /// Only shift.
    pub const SHIFT: Self = Self {
        shift: true,
        control: false,
        alt: false,
    };
    /// Only control.
    pub const CONTROL: Self = Self {
        shift: false,
        control: true,
        alt: false,
    };
    /// Only alt.
    pub const ALT: Self = Self {
        shift: false,
        control: false,
        alt: true,
    };

    /// Returns `true` if no modifiers held down.
    #[must_use]
    pub const fn are_none(self) -> bool {
        !self.shift && !self.control && !self.alt
    }
}

impl BitOr for Modifiers {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            shift: self.shift | rhs.shift,
            control: self.control | rhs.control,
            alt: self.alt | rhs.alt,
        }
    }
}
impl BitOrAssign for Modifiers {
    fn bitor_assign(&mut self, rhs: Self) {
        self.shift |= rhs.shift;
        self.control |= rhs.control;
        self.alt |= rhs.alt;
    }
}

/// A pattern that matches inputs.
///
/// This is implemented for:
/// - Functions that take an input and return a boolean.
/// - [`Input`], [`KeyPress`], [`Mouse`] and [`char`] which just perform an equality check.
/// - [`Key`], which does not allow any modifiers to be held down.
/// - [`MouseKind`], which can occur at any position without modifiers.
/// - Tuples, which detect any one of the inputs occurring.
///
/// You can use the [`input`](crate::input!) macro to generate patterns concisely.
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

/// A macro that generates [input patterns](Pattern).
///
/// # Examples
///
/// A pattern that matches mouse clicks with alt held down:
///
/// ```
/// toon::input!(Alt + Mouse(Press))
/// # ;
/// ```
///
/// A pattern that matches a back tab:
///
/// ```
/// toon::input!(Key(Tab) + Shift)
/// # ;
/// ```
///
/// A pattern that matches the `a` key held without anything else:
///
/// ```
/// toon::input!(Key(a) + None);
/// # ;
/// ```
///
/// # Grammar
///
/// ```text
/// pattern = part [ '+' pattern ] | '!' pattern;
/// part = '(' pattern ')' | 'Key' key-pattern | 'Mouse' mouse-pattern | modifier-pattern;
///
/// key-pattern = [ '(' key ')' ] [ 'where' '(' expression ')' ];
/// key = 'Backspace'
///     | 'Left' | 'Right' | 'Up' | 'Down'
///     | 'Home' | 'End'
///     | 'PageUp' | 'PageDown'
///     | 'Insert'
///     | 'Escape'
///     | 'F1'-'F12'
///     | 'F' expression
///     | 'Tab'
///     | 'Enter' | 'Return'
///     | 'Del' | 'Delete'
///     | '0'-'9'
///     | '!' | '%' | '^' | '&' | '*' | '-' | '_' | '=' | '+'
///     | 'a'-'z'
///     | '|' | ';' | ':' | '@' | '#' | '~' | '<' | '>' | ',' | '.' | '/' | '?'
///     | char-literal
///     | 'Char' expression;
///
/// mouse-pattern = [ '(' mouse-kind ')' ] [ 'at' mouse-at ] [ 'where' '(' expression ')' ];
/// mouse-kind = 'Press' [ mouse-button ]
///     | 'Release' [ mouse-button ]
///     | 'Drag' [ mouse-button ]
///     | 'Move'
///     | 'ScrollDown' | 'ScrollUp';
/// mouse-button = 'Left' | 'Middle' | 'Right';
/// mouse-at = '(' ( '_' | expression ) ',' ( '_' | expression ) [ ',' ] ')'
///
/// modifier-pattern = 'Shift' | 'Control' | 'Alt' | 'None';
/// ```
///
/// The expression given in the `where` part of `key-pattern` and `mouse-pattern` is a function
/// that takes a [`KeyPress`] or [`Mouse`] and returns a [`bool`].
///
/// Note that the `!` operator might not work how you expect; `!Control + Key(f)` is equal to
/// `!(Control + Key(f))` not `(!Control) + Key(f)`.
#[macro_export]
macro_rules! input {
    ($($input:tt)*) => {
        move |input: $crate::Input| -> $crate::std::primitive::bool {
            $crate::__internal_input!(input, $($input)*)
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __internal_input {
    ($input:ident, !$($rest:tt)*) => {
        !$crate::__internal_input!($input, $($rest)*)
    };
    ($input:ident, ($($inner:tt)*) $(+ $($rest:tt)*)?) => {
        $crate::__internal_input!($input, $($inner)*) $(&& $crate::__internal_input!($input, $($rest)*))?
    };
    // Key pattern
    ($input:ident, Key $(($($key:tt)*))? $(where ($f:expr))? $(+ $($rest:tt)*)?) => {{
        #[allow(unused_variables)]
        let b = $crate::std::matches!(
                $input,
                $crate::Input::Key(press) if true
                    $(&& press.key == $crate::__internal_key!($($key)*))?
                    $(&& $f(press))?
            )
                $(&& $crate::__internal_input!($input, $($rest)*))?;
        b
    }};
    // Mouse pattern
    ($input:ident,
        Mouse
        $(($($mouse:tt)*))?
        $(at ($($at:tt)*))?
        $(where ($f:expr))?
        $(+ $($rest:tt)*)?
    ) => {{
        #[allow(unused_variables, clippy::redundant_closure_call)]
        let b = $crate::std::matches!(
            $input,
            $crate::Input::Mouse(mouse) if true
                $(&& $crate::__internal_mouse_kind!(mouse, $($mouse)*))?
                $(&& $crate::__internal_mouse_at!(mouse, $($at)*))?
                $(&& ($f)(mouse))?
        )
            $(&& $crate::__internal_input!($input, $($rest)*))?;
        b
    }};
    // Modifier pattern
    ($input:ident, $modifier:ident $(+ $($rest:tt)*)?) => {
        $crate::__internal_modifier_pattern!($input, $modifier)
            $(&& $crate::__internal_input!($input, $($rest)*))?
    };
}

#[macro_export]
#[doc(hidden)]
#[rustfmt::skip]
macro_rules! __internal_key {
    (Backspace) => ($crate::Key::Backspace);
    (Left) => ($crate::Key::Left);
    (Right) => ($crate::Key::Right);
    (Up) => ($crate::Key::Up);
    (Down) => ($crate::Key::Down);
    (Home) => ($crate::Key::Home);
    (End) => ($crate::Key::End);
    (PageUp) => ($crate::Key::PageUp);
    (PageDown) => ($crate::Key::PageDown);
    (Insert) => ($crate::Key::Insert);
    (Escape) => ($crate::Key::Escape);
    (F1) => ($crate::Key::F(1));
    (F2) => ($crate::Key::F(2));
    (F3) => ($crate::Key::F(3));
    (F4) => ($crate::Key::F(4));
    (F5) => ($crate::Key::F(5));
    (F6) => ($crate::Key::F(6));
    (F7) => ($crate::Key::F(7));
    (F8) => ($crate::Key::F(8));
    (F9) => ($crate::Key::F(9));
    (F10) => ($crate::Key::F(10));
    (F11) => ($crate::Key::F(11));
    (F12) => ($crate::Key::F(12));
    (F $n:expr) => ($crate::Key::F($n));
    (Tab) => ($crate::Key::Char('\t'));
    (Enter) => ($crate::Key::Char('\n'));
    (Return) => ($crate::Key::Char('\n'));
    (Del) => ($crate::Key::Char('\x7F'));
    (Delete) => ($crate::Key::Char('\x7F'));
    (1) => ($crate::Key::Char('1'));
    (2) => ($crate::Key::Char('2'));
    (3) => ($crate::Key::Char('3'));
    (4) => ($crate::Key::Char('4'));
    (5) => ($crate::Key::Char('5'));
    (6) => ($crate::Key::Char('6'));
    (7) => ($crate::Key::Char('7'));
    (8) => ($crate::Key::Char('8'));
    (9) => ($crate::Key::Char('9'));
    (0) => ($crate::Key::Char('0'));
    (!) => ($crate::Key::Char('!'));
    (%) => ($crate::Key::Char('%'));
    (^) => ($crate::Key::Char('^'));
    (&) => ($crate::Key::Char('&'));
    (*) => ($crate::Key::Char('*'));
    (-) => ($crate::Key::Char('-'));
    (_) => ($crate::Key::Char('_'));
    (=) => ($crate::Key::Char('='));
    (+) => ($crate::Key::Char('+'));
    (a) => ($crate::Key::Char('a'));
    (b) => ($crate::Key::Char('b'));
    (c) => ($crate::Key::Char('c'));
    (d) => ($crate::Key::Char('d'));
    (e) => ($crate::Key::Char('e'));
    (f) => ($crate::Key::Char('f'));
    (g) => ($crate::Key::Char('g'));
    (h) => ($crate::Key::Char('h'));
    (i) => ($crate::Key::Char('i'));
    (j) => ($crate::Key::Char('j'));
    (k) => ($crate::Key::Char('k'));
    (l) => ($crate::Key::Char('l'));
    (m) => ($crate::Key::Char('m'));
    (n) => ($crate::Key::Char('n'));
    (o) => ($crate::Key::Char('o'));
    (p) => ($crate::Key::Char('p'));
    (q) => ($crate::Key::Char('q'));
    (r) => ($crate::Key::Char('r'));
    (s) => ($crate::Key::Char('s'));
    (t) => ($crate::Key::Char('t'));
    (u) => ($crate::Key::Char('u'));
    (v) => ($crate::Key::Char('v'));
    (w) => ($crate::Key::Char('w'));
    (x) => ($crate::Key::Char('x'));
    (y) => ($crate::Key::Char('y'));
    (z) => ($crate::Key::Char('z'));
    (|) => ($crate::Key::Char('|'));
    (;) => ($crate::Key::Char(';'));
    (:) => ($crate::Key::Char(':'));
    (@) => ($crate::Key::Char('@'));
    (#) => ($crate::Key::Char('#'));
    (~) => ($crate::Key::Char('~'));
    (<) => ($crate::Key::Char('<'));
    (>) => ($crate::Key::Char('>'));
    (,) => ($crate::Key::Char(','));
    (.) => ($crate::Key::Char('.'));
    (/) => ($crate::Key::Char('/'));
    (?) => ($crate::Key::Char('?'));
    ($c:literal) => ($crate::Key::Char($c));
    (Char $c:expr) => ($crate::Key::Char($c));
}

#[macro_export]
#[doc(hidden)]
macro_rules! __internal_mouse_kind {
    ($input:ident, Press $($button:ident)?) => {
        $crate::std::matches!(
            $input.kind,
            $crate::MouseKind::Press(button) $(if button == $crate::MouseButton::$button)?
        )
    };
    ($input:ident, Release $($button:ident)?) => {
        $crate::std::matches!(
            $input.kind,
            $crate::MouseKind::Release(button) $(if button == $crate::MouseButton::$button)?
        )
    };
    ($input:ident, Drag $($button:ident)?) => {
        $crate::std::matches!(
            $input.kind,
            $crate::MouseKind::Drag(button) $(if button == $crate::MouseButton::$button)?
        )
    };
    ($input:ident, $other:ident $(at $($at:tt)*)?) => {
        $crate::std::matches!($input.kind, $crate::MouseKind::$other)
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __internal_mouse_at {
    ($input:ident, $x:expr, $y:expr $(,)?) => {
        $input.at == $crate::Vec2::new($x, $y)
    };
    ($input:ident, _, $y:expr $(,)?) => {
        $input.at.y == $y
    };
    ($input:ident, $x:expr, _ $(,)?) => {
        $input.at.x == $x
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __internal_modifier_pattern {
    ($input:ident, Shift) => {
        $input.modifiers().shift
    };
    ($input:ident, Control) => {
        $input.modifiers().control
    };
    ($input:ident, Alt) => {
        $input.modifiers().alt
    };
    ($input:ident, None) => {
        $input.modifiers().are_none()
    };
}

#[test]
fn test_input_macro() {
    let mouse = Mouse {
        kind: MouseKind::Press(MouseButton::Middle),
        at: Vec2::new(5, 6),
        size: Vec2::new(7, 8),
        modifiers: Modifiers::SHIFT,
    };

    assert!(input!(Key).matches(Input::Key(KeyPress::from('b'))));
    assert!(input!(Mouse).matches(Input::Mouse(mouse)));
    assert!(!input!(Key).matches(Input::Mouse(mouse)));
    assert!(!input!(Mouse).matches(Input::Key(KeyPress::from('b'))));

    assert!(input!(Key(@)).matches(Input::Key(KeyPress::from('@'))));

    assert!(input!(Key(a)).matches(Input::Key(KeyPress::from('A'))));

    assert!(input!(Shift + Key(a)).matches(Input::Key(KeyPress::from('A'))));
    assert!(!input!(Shift + Key(a)).matches(Input::Key(KeyPress::from('B'))));
    assert!(!input!(Shift + Key(a)).matches(Input::Key(KeyPress::from('a'))));

    let first = input!((!Shift) + Key(a));
    let second_1 = input!(!Shift + Key(a));
    let second_2 = input!(!(Shift + Key(a)));

    assert!(first.matches(Input::Key(KeyPress::from('a'))));
    assert!(!first.matches(Input::Key(KeyPress::from('A'))));
    assert!(!first.matches(Input::Key(KeyPress::from('m'))));
    assert!(!first.matches(Input::Key(KeyPress::from('M'))));

    assert!(second_1.matches(Input::Key(KeyPress::from('a'))));
    assert!(!second_1.matches(Input::Key(KeyPress::from('A'))));
    assert!(second_1.matches(Input::Key(KeyPress::from('m'))));
    assert!(second_1.matches(Input::Key(KeyPress::from('M'))));

    assert!(second_2.matches(Input::Key(KeyPress::from('a'))));
    assert!(!second_2.matches(Input::Key(KeyPress::from('A'))));
    assert!(second_2.matches(Input::Key(KeyPress::from('m'))));
    assert!(second_2.matches(Input::Key(KeyPress::from('M'))));

    assert!(input!(Control + Key(b)).matches(Input::Key(KeyPress {
        key: Key::Char('b'),
        modifiers: Modifiers::CONTROL,
    })));

    assert!(input!(Mouse(Press)).matches(Input::Mouse(mouse)));
    assert!(!input!(Mouse(Release)).matches(Input::Mouse(mouse)));
    assert!(!input!(Mouse(Release Middle)).matches(Input::Mouse(mouse)));
    assert!(input!(Mouse(Press Middle)).matches(Input::Mouse(mouse)));
    assert!(!input!(Mouse(Press Left)).matches(Input::Mouse(mouse)));
}
