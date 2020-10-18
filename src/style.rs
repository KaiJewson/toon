//! Text styling.

/// How text is written.
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Style {
    /// The foreground color of the text.
    pub foreground: Color,
    /// The background color of the text.
    pub background: Color,
    /// The attributes of the text.
    pub attributes: Attributes,
}

impl Style {
    /// Create a style.
    #[must_use]
    pub const fn new(foreground: Color, background: Color, attributes: Attributes) -> Self {
        Self {
            foreground,
            background,
            attributes,
        }
    }
}

/// A color.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Color {
    /// The terminal's default color.
    Default,
    /// Black. Corresponds to the ANSI color black.
    Black,
    /// Dark gray. Corresponds to the ANSI color bright black.
    DarkGray,
    /// Light gray. Corresponds to the ANSI color white.
    LightGray,
    /// White. Corresponds to the ANSI color bright white.
    White,
    /// Red. Corresponds to the ANSI color bright red.
    Red,
    /// Dark Red. Corresponds to the ANSI color red.
    DarkRed,
    /// Green. Corresponds to the ANSI color bright green.
    Green,
    /// Dark Green. Corresponds to the ANSI color green.
    DarkGreen,
    /// Yellow. Corresponds to the ANSI color bright yellow.
    Yellow,
    /// Dark Yellow. Corresponds to the ANSI color yellow.
    DarkYellow,
    /// Blue. Corresponds to the ANSI color bright blue.
    Blue,
    /// Dark Blue. Corresponds to the ANSI color blue.
    DarkBlue,
    /// Magenta. Corresponds to the ANSI color bright magenta.
    Magenta,
    /// Dark Magenta. Corresponds to the ANSI color magenta.
    DarkMagenta,
    /// Cyan. Corresponds to the ANSI color bright cyan.
    Cyan,
    /// Dark Cyan. Corresponds to the ANSI color cyan.
    DarkCyan,
    /// An ANSI color.
    AnsiValue(AnsiColor),
    /// A full 24-bit RGB color.
    Rgb(Rgb),
}

impl Color {
    /// Create a color from an ANSI value.
    ///
    /// If the value is < 16 it will be mapped to the named color variants, otherwise it will be an
    /// `AnsiColor`.
    #[must_use]
    pub fn new_ansi(value: u8) -> Self {
        match value {
            0 => Self::Black,
            1 => Self::DarkRed,
            2 => Self::DarkGreen,
            3 => Self::DarkYellow,
            4 => Self::DarkBlue,
            5 => Self::DarkMagenta,
            6 => Self::DarkCyan,
            7 => Self::LightGray,
            8 => Self::DarkGray,
            9 => Self::Red,
            10 => Self::Green,
            11 => Self::Yellow,
            12 => Self::Blue,
            13 => Self::Magenta,
            14 => Self::Cyan,
            15 => Self::White,
            _ => Self::AnsiValue(AnsiColor::new(value)),
        }
    }

    /// Darken a color if it is a named color variant.
    #[must_use]
    pub fn darken(self) -> Self {
        match self {
            Self::Black | Self::DarkGray => Self::Black,
            Self::LightGray => Self::DarkGray,
            Self::White => Self::LightGray,
            Self::Red | Self::DarkRed => Self::DarkRed,
            Self::Green | Self::DarkGreen => Self::DarkGreen,
            Self::Yellow | Self::DarkYellow => Self::DarkYellow,
            Self::Blue | Self::DarkBlue => Self::DarkBlue,
            Self::Magenta | Self::DarkMagenta => Self::DarkMagenta,
            Self::Cyan | Self::DarkCyan => Self::DarkCyan,
            other => other,
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::Default
    }
}

impl From<AnsiColor> for Color {
    fn from(color: AnsiColor) -> Self {
        Self::AnsiValue(color)
    }
}
impl From<Rgb> for Color {
    fn from(rgb: Rgb) -> Self {
        Self::Rgb(rgb)
    }
}

/// An ANSI value color.
///
/// This can either be an RGB color with each part being 6 values, or it can be a grayscale color
/// from [0, 26). White and black are treated as both RGB and grayscale colors; there are also
/// shades of gray treated only as RGB colors and vice versa, because they cannot be fully
/// represented in the other format.
///
/// See [this list of ANSI colors](https://jonasjacek.github.io/colors/) for more information.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct AnsiColor(u8);

impl AnsiColor {
    /// Create an ANSI color from its numerical value.
    ///
    /// # Panics
    ///
    /// Panics if the value is < 16.
    #[must_use]
    pub fn new(value: u8) -> Self {
        assert!(value >= 16);
        Self(value)
    }

    /// Create an ANSI color from its RGB components.
    ///
    /// # Panics
    ///
    /// Panics if r, g, or b isn't <= 5.
    #[must_use]
    pub fn new_rgb(r: u8, g: u8, b: u8) -> Self {
        assert!(r <= 5 && g <= 5 && b <= 5);
        Self(16 + 36 * r + 6 * g + b)
    }
    /// Create an ANSI color grayscale color. 0 is black, 25 is white.
    ///
    /// # Panics
    ///
    /// Panics if shade isn't < 26.
    #[must_use]
    pub fn new_grayscale(shade: u8) -> Self {
        Self(match shade {
            0 => 16,
            1..=24 => 0xE7 + shade,
            25 => 0xE7,
            _ => panic!(
                "Shade {} is out of range, expected a value between 0 and 25",
                shade
            ),
        })
    }

    /// Get the ANSI value of this color.
    ///
    /// This value is guaranteed to be >= 16.
    #[must_use]
    pub fn get(self) -> u8 {
        self.0
    }

    /// Set the ANSI value of this color.
    ///
    /// # Panics
    ///
    /// Panics if the value is < 16.
    pub fn set(&mut self, value: u8) {
        assert!(value >= 16);
        self.0 = value;
    }

    /// Get the RGB components of the color, if the color is RGB.
    ///
    /// All returned values are guaranteed to be <= 5.
    #[must_use]
    pub fn rgb(self) -> Option<(u8, u8, u8)> {
        let value = self.0 - 16;
        if value >= 6 * 6 * 6 {
            return None;
        }
        Some((value / 36, value % 36 / 6, value % 6))
    }

    /// Get the grayscale shade of the color, if the color is grayscale.
    ///
    /// The returned value is guaranteed to be < 26.
    #[must_use]
    pub fn grayscale(self) -> Option<u8> {
        match self.0 {
            16 => Some(0),
            0xE7 => Some(25),
            0xE8..=0xFF => Some(self.0 - 0xE7),
            _ => None,
        }
    }
}

#[cfg(test)]
#[test]
fn test_ansi() {
    let color = AnsiColor::new_rgb(2, 3, 4);
    assert_eq!(color.get(), 110);
    assert_eq!(color.rgb(), Some((2, 3, 4)));
    assert_eq!(color.grayscale(), None);

    let color = AnsiColor::new_grayscale(13);
    assert_eq!(color.get(), 244);
    assert_eq!(color.rgb(), None);
    assert_eq!(color.grayscale(), Some(13));

    let color = AnsiColor::new_grayscale(0);
    assert_eq!(color, AnsiColor::new_rgb(0, 0, 0));
    assert_eq!(color.get(), 16);
    assert_eq!(color.rgb(), Some((0, 0, 0)));
    assert_eq!(color.grayscale(), Some(0));

    let color = AnsiColor::new_grayscale(25);
    assert_eq!(color, AnsiColor::new_rgb(5, 5, 5));
    assert_eq!(color.get(), 231);
    assert_eq!(color.rgb(), Some((5, 5, 5)));
    assert_eq!(color.grayscale(), Some(25));
}

impl From<AnsiColor> for u8 {
    fn from(ansi: AnsiColor) -> Self {
        ansi.get()
    }
}

/// A full 24-bit RGB color.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Rgb {
    /// The red component.
    pub r: u8,
    /// The green component.
    pub g: u8,
    /// The blue component.
    pub b: u8,
}

impl Rgb {
    /// Creates a new RGB color.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    /// Gets the opposite color.
    #[must_use]
    pub const fn opposite(self) -> Self {
        Self {
            r: u8::MAX - self.r,
            g: u8::MAX - self.g,
            b: u8::MAX - self.b,
        }
    }
}

/// Attributes of text. Not all of these attributes are supported by all terminals.
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
#[non_exhaustive]
pub struct Attributes {
    /// The text intensity.
    pub intensity: Intensity,
    /// Whether the text is emphasized.
    pub italic: bool,
    /// Whether the text is underlined.
    pub underlined: bool,
    /// Whether the text blinks. Not widely supported.
    pub blinking: bool,
    /// Whether the text is crossed out. Not widely supported.
    pub crossed_out: bool,
}

macro_rules! attribute_fn {
    ($(#[doc = $doc:literal] $name:ident($property:ident = $value:expr),)*) => {
        $(
            #[doc = $doc]
            #[must_use]
            pub const fn $name(self) -> Self {
                Self {
                    $property: $value,
                    ..self
                }
            }
        )*
    }
}

impl Attributes {
    /// Default attributes.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            intensity: Intensity::Normal,
            italic: false,
            underlined: false,
            blinking: false,
            crossed_out: false,
        }
    }

    attribute_fn! {
        /// Make the intensity bold.
        bold(intensity = Intensity::Bold),
        /// Make the intensity dim.
        dim(intensity = Intensity::Dim),
        /// Make the text italic.
        italic(italic = true),
        /// Underline the text.
        underlined(underlined = true),
        /// Make the text blink.
        blinking(blinking = true),
        /// Cross out the text.
        crossed_out(crossed_out = true),
    }
}

/// The intensity of text.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Intensity {
    /// Less intense text. Not widely supported.
    Dim,
    /// Normal text intensity.
    Normal,
    /// More intense text.
    Bold,
}

impl Default for Intensity {
    fn default() -> Self {
        Self::Normal
    }
}
