use std::fmt;

use unicode_width::UnicodeWidthChar;

use crate::{
    output::{Ext as _, Output},
    Element, Events, Input, Mouse, Style, Vec2,
};

use super::{Alignment, Filter};

/// A filter that adds a border to an element.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub struct Border {
    /// The characters that make up the sides of the border, in the order of top, left, right,
    /// bottom.
    ///
    /// These must not be double-width characters.
    pub sides: (char, char, char, char),
    /// The characters that make up the corners of the border, in the order of top left, top right,
    /// bottom left, bottom right.
    ///
    /// These must not be double-width characters.
    pub corners: (char, char, char, char),
    /// The style of the border.
    pub style: Style,
    /// The style of the title.
    pub title_style: Style,
    /// The alignment of the title if it's displayed on the top of the border.
    pub top_title_align: Option<Alignment>,
    /// The alignment of the title if it's displayed on the bottom of the border.
    pub bottom_title_align: Option<Alignment>,
    /// Whether the content has one character of padding on either side. All the constants set this
    /// to `true` as it looks a lot better.
    ///
    /// With padding:
    /// ```text
    /// ┌──────────────┐
    /// │ Hello World! │
    /// └──────────────┘
    /// ```
    /// Without padding:
    /// ```text
    /// ┌────────────┐
    /// │Hello World!│
    /// └────────────┘
    /// ```
    pub padding: bool,
}

impl Border {
    /// The base border used by the constants below, which override the `sides` and `corners`
    /// properties.
    const fn base() -> Self {
        Self {
            sides: ('X', 'X', 'X', 'X'),
            corners: ('X', 'X', 'X', 'X'),
            style: Style::default(),
            title_style: Style::default(),
            top_title_align: None,
            bottom_title_align: None,
            padding: true,
        }
    }
}

impl Border {
    /// An ASCII border using pluses.
    ///
    /// ```text
    /// +---+
    /// |   |
    /// +---+
    /// ```
    pub const ASCII_PLUS: Self = Self {
        sides: ('-', '|', '|', '-'),
        corners: ('+', '+', '+', '+'),
        ..Self::base()
    };
    /// An curved ASCII border using dots and quotes.
    ///
    /// ```text
    /// .---.
    /// |   |
    /// '---'
    /// ```
    pub const ASCII_CURVED: Self = Self {
        sides: ('-', '|', '|', '-'),
        corners: ('.', '.', '\'', '\''),
        ..Self::base()
    };
    /// A thin border.
    ///
    /// ```text
    /// ┌───┐
    /// │   │
    /// └───┘
    /// ```
    pub const THIN: Self = Self {
        sides: ('─', '│', '│', '─'),
        corners: ('┌', '┐', '└', '┘'),
        ..Self::base()
    };
    /// A thin, curved border.
    ///
    /// ```text
    /// ╭───╮
    /// │   │
    /// ╰───╯
    /// ```
    pub const THIN_CURVED: Self = Self {
        sides: ('─', '│', '│', '─'),
        corners: ('╭', '╮', '╰', '╯'),
        ..Self::base()
    };
    /// A thick border.
    ///
    /// ```text
    /// ┏━━━┓
    /// ┃   ┃
    /// ┗━━━┛
    /// ```
    pub const THICK: Self = Self {
        sides: ('━', '┃', '┃', '━'),
        corners: ('┏', '┓', '┗', '┛'),
        ..Self::base()
    };
    /// A double border.
    ///
    /// ```text
    /// ╔═══╗
    /// ║   ║
    /// ╚═══╝
    /// ```
    pub const DOUBLE: Self = Self {
        sides: ('═', '║', '║', '═'),
        corners: ('╔', '╗', '╚', '╝'),
        ..Self::base()
    };
    /// A block border. This will look connected on most terminals.
    ///
    /// ```text
    /// █▀▀▀█
    /// █   █
    /// █▄▄▄█
    /// ```
    pub const BLOCK: Self = Self {
        sides: ('▀', '█', '█', '▄'),
        corners: ('█', '█', '█', '█'),
        ..Self::base()
    };
    /// A thin braille border.
    ///
    /// ```text
    /// ⡏⠉⠉⠉⢹
    /// ⡇⠀⠀⠀⢸
    /// ⣇⣀⣀⣀⣸
    /// ```
    pub const BRAILLE_THIN: Self = Self {
        sides: ('⠉', '⡇', '⢸', '⣀'),
        corners: ('⡏', '⢹', '⣇', '⣸'),
        ..Self::base()
    };
    /// A thick braille border. This will appear like the block border on some terminals.
    ///
    /// ```text
    /// ⣿⠛⠛⠛⣿
    /// ⣿⠀⠀⠀⣿
    /// ⣿⣤⣤⣤⣿
    /// ```
    pub const BRAILLE_THICK: Self = Self {
        sides: ('⠛', '⣿', '⣿', '⣤'),
        corners: ('⣿', '⣿', '⣿', '⣿'),
        ..Self::base()
    };
}

impl Border {
    /// Set the alignment of the top title of the border.
    #[must_use]
    pub fn top_title(self, align: Alignment) -> Self {
        Self {
            top_title_align: Some(align),
            ..self
        }
    }

    /// Set the alignment of the bottom title of the border.
    #[must_use]
    pub fn bottom_title(self, align: Alignment) -> Self {
        Self {
            bottom_title_align: Some(align),
            ..self
        }
    }

    /// Set the title's style.
    #[must_use]
    pub fn title_style(self, title_style: Style) -> Self {
        Self {
            title_style,
            ..self
        }
    }

    /// Turn off the padding around the contents.
    #[must_use]
    pub fn no_padding(self) -> Self {
        Self {
            padding: false,
            ..self
        }
    }
}

impl AsRef<Style> for Border {
    fn as_ref(&self) -> &Style {
        &self.style
    }
}
impl AsMut<Style> for Border {
    fn as_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl<Event> Filter<Event> for Border {
    #[allow(clippy::too_many_lines)]
    fn draw<E: Element>(&self, element: E, output: &mut dyn Output) {
        let output_size = output.size();

        // Draw the element.
        element.draw(
            &mut output.area(
                Vec2::new(if self.padding { 2 } else { 1 }, 1),
                Vec2::new(
                    output_size
                        .x
                        .saturating_sub(if self.padding { 4 } else { 2 }),
                    output_size.y.saturating_sub(2),
                ),
            ),
        );

        // The positions of the right and bottom borders, if present.
        let Vec2 {
            x: right_border,
            y: bottom_border,
        } = output_size.map(|dimension| {
            if dimension > 1 {
                Some(dimension - 1)
            } else {
                None
            }
        });

        // Fill the padding.
        if self.padding {
            for y in 1..output_size.y.saturating_sub(1) {
                output.write_char(Vec2::new(1, y), ' ', self.style);
                if let Some(right_border) = right_border {
                    output.write_char(Vec2::new(right_border - 1, y), ' ', self.style);
                }
            }
        }

        // Write corners
        let (top_left, top_right, bottom_left, bottom_right) = self.corners;
        output.write_char(Vec2::new(0, 0), top_left, self.style);
        if let Some(right_border) = right_border {
            output.write_char(Vec2::new(right_border, 0), top_right, self.style);
        }
        if let Some(bottom_border) = bottom_border {
            output.write_char(Vec2::new(0, bottom_border), bottom_left, self.style);
        }
        if let (Some(right_border), Some(bottom_border)) = (right_border, bottom_border) {
            output.write_char(
                Vec2::new(right_border, bottom_border),
                bottom_right,
                self.style,
            );
        }

        let (top, left, right, bottom) = self.sides;

        // Write both sides
        for y in 1..output_size.y.saturating_sub(1) {
            output.write_char(Vec2::new(0, y), left, self.style);
            if let Some(right_border) = right_border {
                output.write_char(Vec2::new(right_border, y), right, self.style);
            }
        }

        // Get the title width, is lazy because only when one of the top title and bottom title is
        // aligned to the center or right is this needed.
        let mut title_width = crate::util::Lazy::new(|| {
            let mut width: u16 = 0;
            let _ = element.title(&mut crate::util::WriteCharsFn(|c| {
                width = width.saturating_add(c.width().unwrap_or(0) as u16);
                Ok(())
            }));
            width
        });

        let available_width = output_size.x.saturating_sub(2);

        // Get the position where the title starts.
        let mut get_title_start = |align| {
            1 + match align {
                Alignment::Start => 0,
                Alignment::Middle => (available_width / 2).saturating_sub(*title_width.get() / 2),
                Alignment::End => available_width.saturating_sub(*title_width.get()),
            }
        };
        let title_start_top = self.top_title_align.map(&mut get_title_start);
        let title_start_bottom = self.bottom_title_align.map(&mut get_title_start);

        // The x-offset at which the titles are currently being drawn.
        let mut offset_top = title_start_top;
        let mut offset_bottom = title_start_bottom;

        // Draw the title
        if offset_top.is_some() || offset_bottom.is_some() {
            let _ = element.title(&mut crate::util::WriteCharsFn(|c| {
                let width = match c.width() {
                    Some(width) => width,
                    None => return Ok(()),
                } as u16;

                if let Some(offset) = &mut offset_top {
                    let after = offset.checked_add(width).ok_or(fmt::Error)?;
                    if Some(after) > right_border {
                        return Err(fmt::Error);
                    }
                    output.write_char(Vec2::new(*offset, 0), c, self.title_style);
                    *offset = after;
                }

                if let (Some(offset), Some(y)) = (&mut offset_bottom, bottom_border) {
                    let after = offset.checked_add(width).ok_or(fmt::Error)?;
                    if Some(after) > right_border {
                        return Err(fmt::Error);
                    }
                    output.write_char(Vec2::new(*offset, y), c, self.title_style);
                    *offset = after;
                }

                Ok(())
            }));
        }

        // Write top and bottom borders, not overwriting the title
        for x in 1..output_size.x.saturating_sub(1) {
            if title_start_top.map_or(true, |start| x < start || x >= offset_top.unwrap()) {
                output.write_char(Vec2::new(x, 0), top, self.style);
            }
            if let Some(y) = bottom_border {
                if title_start_bottom.map_or(true, |start| x < start || x >= offset_bottom.unwrap())
                {
                    output.write_char(Vec2::new(x, y), bottom, self.style);
                }
            }
        }
    }
    fn width<E: Element>(&self, element: E, height: Option<u16>) -> (u16, u16) {
        let (min, max) = element.width(height);
        let extra_width = if self.padding { 4 } else { 2 };
        (
            min.saturating_add(extra_width),
            max.saturating_add(extra_width),
        )
    }
    fn height<E: Element>(&self, element: E, width: Option<u16>) -> (u16, u16) {
        let (min, max) = element.height(width);
        (min.saturating_add(2), max.saturating_add(2))
    }
    fn handle<E: Element<Event = Event>>(
        &self,
        element: E,
        input: Input,
        events: &mut dyn Events<Event>,
    ) {
        let input = match input {
            Input::Key(key) => Some(Input::Key(key)),
            Input::Mouse(mouse) => (|| {
                let xborder = if self.padding { 2 } else { 1 };

                if mouse.at.x.saturating_add(xborder) >= mouse.size.x
                    || mouse.at.y.saturating_add(1) >= mouse.size.y
                {
                    return None;
                }
                Some(Input::Mouse(Mouse {
                    at: Vec2::new(mouse.at.x.checked_sub(xborder)?, mouse.at.y.checked_sub(1)?),
                    size: Vec2::new(
                        mouse.size.x.checked_sub(if self.padding { 4 } else { 2 })?,
                        mouse.size.y.checked_sub(2)?,
                    ),
                    ..mouse
                }))
            })(),
        };
        if let Some(input) = input {
            element.handle(input, events);
        }
    }
}
