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
    /// An ANSI color - see [here](https://jonasjacek.github.io/colors/) for the full list.
    AnsiValue(u8),
    /// A full 24-bit RGB color.
    Rgb(Rgb),
}

impl Color {
    /// Create an ANSI value RGB color from its components.
    ///
    /// # Panics
    ///
    /// Panics if r, g, or b isn't <= 5.
    #[must_use]
    pub fn ansi_rgb(r: u8, g: u8, b: u8) -> Self {
        assert!(r <= 5 && g <= 5 && b <= 5);
        Self::AnsiValue(16 + 36 * r + 6 * g + b)
    }
    /// Create an ANSI value grayscale color.
    ///
    /// # Panics
    ///
    /// Panics if shade isn't < 24.
    #[must_use]
    pub fn ansi_grayscale(shade: u8) -> Self {
        assert!(shade < 24);
        Self::AnsiValue(0xE8 + shade)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::Default
    }
}

impl From<Rgb> for Color {
    fn from(rgb: Rgb) -> Self {
        Self::Rgb(rgb)
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
