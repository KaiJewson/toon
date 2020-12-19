//! Data structures representing terminals.
//!
//! Toon uses [`unicode_width`] to get the number of cells a character takes up in a terminal. This
//! isn't a perfect solution - for example, 👨‍👨‍👧‍👦, a family emoji, cannot be
//! displayed in Toon as [`unicode_width`] measures it as 8 columns wide. However, it is the same as
//! what most terminals do and is simple to understand and implement.

use std::cmp::Ordering;
use std::iter;

use smartstring::{LazyCompact, SmartString};
use unicode_width::UnicodeWidthChar;

use crate::{Cursor, Output, Style, Vec2};

/// A terminal state.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Buffer {
    /// The grid of characters on the terminal.
    pub grid: Grid,
    /// The cursor on the terminal.
    pub cursor: Option<Cursor>,
}

impl Buffer {
    /// Reset the buffer. This clears the grid and removes the cursor.
    pub fn reset(&mut self) {
        self.grid.clear();
        self.cursor = None;
    }
}

impl From<Grid> for Buffer {
    fn from(grid: Grid) -> Self {
        Self { grid, cursor: None }
    }
}

impl Output for Buffer {
    fn size(&self) -> Vec2<u16> {
        self.grid.size()
    }
    fn write_char(&mut self, pos: Vec2<u16>, c: char, style: Style) {
        self.grid.write_char(pos, c, style)
    }
    fn set_cursor(&mut self, cursor: Option<Cursor>) {
        self.cursor = cursor;
    }
}

/// The grid of characters on a terminal.
///
/// A grid is a list of [`Line`]s that are all the same length.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Grid {
    width: u16,
    // invariant: length <= u16::MAX, the width of each line is the width above
    lines: Vec<Line>,
}

impl Grid {
    /// Create a new grid with all empty cells.
    #[must_use]
    pub fn new(size: impl Into<Vec2<u16>>) -> Self {
        let size = size.into();

        let mut this = Self::default();
        this.resize_width(size.x);
        this.resize_height(size.y);
        this
    }

    /// Get the number of columns in the grid.
    #[must_use]
    pub fn width(&self) -> u16 {
        self.width
    }
    /// Get the number of rows in the grid.
    #[must_use]
    pub fn height(&self) -> u16 {
        self.lines.len() as u16
    }

    /// Get the rows of the grid.
    #[must_use]
    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    /// Resize the grid's width.
    ///
    /// All new cells will be empty. If resizing a line cuts off a double cell, that double cell
    /// becomes a space.
    pub fn resize_width(&mut self, new_width: u16) {
        self.width = new_width;

        for line in &mut self.lines {
            line.resize(new_width);
        }
    }

    /// Resize the grid's height, using an anchor line. Lines will be removed from the bottom until
    /// the anchor line is reached, and then they will be removed from the top to avoid removing
    /// the anchor line. Adding lines will as usual add them to the bottom. This matches the
    /// behaviour of resizing terminals, where the anchor is the line that the cursor is on.
    ///
    /// All new cells will be empty.
    pub fn resize_height_with_anchor(&mut self, new_height: u16, anchor_line: u16) {
        match usize::from(new_height).cmp(&self.lines.len()) {
            Ordering::Greater => self.resize_height(new_height),
            Ordering::Equal => {}
            Ordering::Less => {
                let new_height = usize::from(new_height);
                let anchor_line = usize::from(anchor_line);

                if new_height > anchor_line {
                    self.lines.truncate(new_height);
                } else {
                    let after_anchor = anchor_line + 1;
                    self.lines.truncate(after_anchor);
                    self.lines
                        .drain(0..after_anchor - new_height)
                        .for_each(drop);
                }
            }
        }
    }

    /// Resize the grid's height from the bottom of the grid.
    ///
    /// All new cells will be empty.
    pub fn resize_height(&mut self, new_height: u16) {
        let width = self.width;
        self.lines
            .resize_with(usize::from(new_height), || Line::new(width));
    }

    /// Get the grid's contents as a vector of strings.
    ///
    /// This is mostly useful in tests.
    #[must_use]
    pub fn contents(&self) -> Vec<String> {
        self.lines.iter().map(Line::contents).collect()
    }

    /// Clear the grid.
    pub fn clear(&mut self) {
        for line in &mut self.lines {
            line.clear();
        }
    }
}

impl Output for Grid {
    fn size(&self) -> Vec2<u16> {
        Vec2::new(self.width, self.height())
    }
    fn write_char(&mut self, pos: Vec2<u16>, c: char, style: Style) {
        if let Some(line) = self.lines.get_mut(usize::from(pos.y)) {
            line.write_char(Vec2::new(pos.x, 0), c, style);
        }
    }
    fn set_cursor(&mut self, _cursor: Option<Cursor>) {}
}

/// A line of cells in a terminal.
///
/// A line is a list of [`Cell`]s with the guarantee that each cell is 1 or 2
/// columns wide and that double-width cells will be followed by continuation cells.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Line {
    // invariant: length <= u16::MAX, double cells must be followed by continuation cells
    cells: Vec<Cell>,
}

impl Line {
    /// Create a new line with all empty cells.
    #[must_use]
    pub fn new(len: u16) -> Self {
        let mut this = Self::default();
        this.resize(len);
        this
    }

    /// Get the number of cells in the line.
    #[must_use]
    pub fn len(&self) -> u16 {
        self.cells.len() as u16
    }

    /// Get whether the line is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Get the cells in the line.
    #[must_use]
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Resize the line.
    ///
    /// All new cells will be empty. If resizing the line cuts off a double cell, that double cell
    /// becomes a space.
    pub fn resize(&mut self, new_len: u16) {
        self.cells.resize(
            usize::from(new_len),
            Cell(CellInner::Char {
                contents: SmartString::from(" "),
                double: false,
                style: Style::default(),
            }),
        );

        if let Some(Cell(CellInner::Char {
            contents,
            double: double @ true,
            ..
        })) = self.cells.last_mut()
        {
            *contents = SmartString::from(" ");
            *double = false;
        }
    }

    /// Get the contents of the line as a string.
    ///
    /// This is mostly useful in tests.
    #[must_use]
    pub fn contents(&self) -> String {
        self.cells.iter().filter_map(Cell::contents).collect()
    }

    /// Clear the line.
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.0 = CellInner::Char {
                contents: " ".into(),
                double: false,
                style: Style::default(),
            };
        }
    }
}

impl Output for Line {
    fn size(&self) -> Vec2<u16> {
        Vec2::new(self.len(), 1)
    }
    fn write_char(&mut self, pos: Vec2<u16>, c: char, style: Style) {
        if pos.y != 0 || c == '\0' {
            return;
        }

        let x = usize::from(pos.x);

        match c.width() {
            Some(0) => {
                if let Some(Cell(CellInner::Char { contents, .. })) = self.cells.get_mut(x) {
                    contents.push(c);
                }
            }
            Some(1) => {
                let cell = match self.cells.get_mut(x) {
                    Some(cell) => cell,
                    None => return,
                };
                let old_cell = std::mem::replace(
                    &mut cell.0,
                    CellInner::Char {
                        contents: iter::once(c).collect(),
                        double: false,
                        style,
                    },
                );

                match old_cell {
                    CellInner::Char {
                        double: true,
                        style: old_style,
                        ..
                    } => {
                        self.cells[x + 1].0 = CellInner::Char {
                            contents: SmartString::from(" "),
                            double: false,
                            style: old_style,
                        };
                    }
                    CellInner::Char { .. } => {}
                    CellInner::Continuation => match &mut self.cells[x - 1].0 {
                        CellInner::Char {
                            contents,
                            double: double @ true,
                            ..
                        } => {
                            *contents = SmartString::from(" ");
                            *double = false;
                        }
                        _ => unreachable!(),
                    },
                }
            }
            Some(2) => {
                let second_cell = match self.cells.get_mut(x + 1) {
                    Some(cell) => &mut cell.0,
                    None => return,
                };

                let old_second = std::mem::replace(second_cell, CellInner::Continuation);
                let old_first = std::mem::replace(
                    &mut self.cells[x].0,
                    CellInner::Char {
                        contents: iter::once(c).collect(),
                        double: true,
                        style,
                    },
                );

                if let CellInner::Continuation = old_first {
                    match &mut self.cells[x - 1].0 {
                        CellInner::Char {
                            contents, double, ..
                        } => {
                            *contents = SmartString::from(" ");
                            *double = false;
                        }
                        _ => unreachable!(),
                    }
                }
                if let CellInner::Char {
                    double: true,
                    style: old_style,
                    ..
                } = old_second
                {
                    self.cells[x + 2].0 = CellInner::Char {
                        contents: SmartString::from(" "),
                        double: false,
                        style: old_style,
                    };
                }
            }
            Some(_) => unreachable!(),
            None => {}
        };
    }

    fn set_cursor(&mut self, _cursor: Option<Cursor>) {}
}

/// A cell in a terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell(CellInner);

impl Cell {
    /// Get the kind of the cell.
    #[must_use]
    pub fn kind(&self) -> CellKind<'_> {
        match &self.0 {
            CellInner::Char {
                contents,
                double,
                style,
            } => CellKind::Char {
                contents: &contents,
                double: *double,
                style: *style,
            },
            CellInner::Continuation => CellKind::Continuation,
        }
    }

    /// Get whether the cell is a character cell.
    #[must_use]
    pub fn is_char(&self) -> bool {
        matches!(self.0, CellInner::Char { .. })
    }

    /// Get whether the cell is a continuation cell.
    #[must_use]
    pub fn is_continuation(&self) -> bool {
        matches!(self.0, CellInner::Continuation)
    }

    /// Get whether the cell is part of a double-width character.
    #[must_use]
    pub fn is_double(&self) -> bool {
        matches!(self.0, CellInner::Char { double: true, .. } | CellInner::Continuation)
    }

    /// Get the contents of the cell, if it is not a continuation cell.
    #[must_use]
    pub fn contents(&self) -> Option<&str> {
        match &self.0 {
            CellInner::Char { contents, .. } => Some(&**contents),
            _ => None,
        }
    }

    /// Get the style of cell, if it is not a continuation cell.
    #[must_use]
    pub fn style(&self) -> Option<Style> {
        match &self.0 {
            CellInner::Char { style, .. } => Some(*style),
            _ => None,
        }
    }
}

/// A kind of cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellKind<'a> {
    /// The cell contains a 1-2 width character followed by any number of zero-width characters.
    Char {
        /// The characters in the cell. The first character will be 1-2 columns wide, and the rest
        /// of the characters will be zero columns wide. This will contain no control characters.
        contents: &'a str,
        /// Whether the cell is double-width (that is, the width of the first character of
        /// `contents` as well as all of `contents` is 2).
        ///
        /// If a cell is double width the next cell will be a `Continuation`.
        double: bool,
        /// The style of the cell.
        style: Style,
    },
    /// The cell is a continuation of the previous double-width cell.
    Continuation,
}

/// A cell in a terminal. See `CellKind` above for more info on each variant.
#[derive(Debug, Clone, PartialEq, Eq)]
enum CellInner {
    Char {
        /// Since there are many cells, this is stored as a smart string which avoids too much heap
        /// allocation.
        contents: SmartString<LazyCompact>,
        double: bool,
        style: Style,
    },
    Continuation,
}

#[cfg(test)]
#[test]
fn test_line() {
    use std::convert::TryFrom;
    use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

    fn assert_invariants(line: &Line) {
        assert!(u16::try_from(line.cells.len()).is_ok());

        let mut continuation = false;
        for (x, cell) in line.cells.iter().enumerate() {
            if continuation {
                if let Some(contents) = cell.contents() {
                    panic!(
                        "Cell at {} is not a continuation, contains {:?} ({:?})",
                        x,
                        contents,
                        contents.as_bytes()
                    );
                }
                continuation = false;
            } else {
                match cell.kind() {
                    CellKind::Char {
                        contents, double, ..
                    } => {
                        assert!(!contents.is_empty());

                        let width = if double { 2 } else { 1 };
                        assert_eq!(contents.width(), width);
                        assert_eq!(contents.chars().next().unwrap().width().unwrap(), width);

                        continuation = double;
                    }
                    CellKind::Continuation => panic!("Cell at {} is a continuation!", x),
                }
            }
        }

        if continuation {
            panic!("Last cell is a double!");
        }
    }

    let mut line = Line::new(0);
    assert_invariants(&line);

    line.resize(5);
    assert_invariants(&line);
    assert_eq!(line.len(), 5);
    assert_eq!(line.contents(), "     ");

    // Drawing a double width character
    line.write_char(Vec2::new(0, 0), '😊', Style::default());
    assert_invariants(&line);
    assert_eq!(line.contents(), "😊   ");

    // Drawing a single width character next to a double width character
    line.write_char(Vec2::new(2, 0), 'a', Style::default());
    assert_invariants(&line);
    assert_eq!(line.contents(), "😊a  ");

    // Drawing a double width character over a double width & single width character
    line.write_char(Vec2::new(1, 0), '😊', Style::default());
    assert_invariants(&line);
    assert_eq!(line.contents(), " 😊  ");

    // Drawing a double width character over a double width character
    line.write_char(Vec2::new(0, 0), '😊', Style::default());
    assert_invariants(&line);
    assert_eq!(line.contents(), "😊   ");

    // Drawing a double width character at the edge doesn't do anything
    line.write_char(Vec2::new(4, 0), '😊', Style::default());
    assert_invariants(&line);
    assert_eq!(line.contents(), "😊   ");

    // Drawing a single width character over a double width character
    line.write_char(Vec2::new(1, 0), 'a', Style::default());
    assert_invariants(&line);
    assert_eq!(line.contents(), " a   ");

    // Drawing a double width character next to a single width character
    line.write_char(Vec2::new(2, 0), '😊', Style::default());
    assert_invariants(&line);
    assert_eq!(line.contents(), " a😊 ");

    // Resizing the line and removing the double-width char
    line.resize(3);
    assert_invariants(&line);
    assert_eq!(line.contents(), " a ");
}

#[cfg(test)]
#[test]
fn test_resize_anchor() {
    use crate::output::Ext as _;

    let mut grid = Grid::new((1, 3));

    grid.write((0, 0), "0", Style::default());
    grid.write((0, 1), "1", Style::default());
    grid.write((0, 2), "2", Style::default());

    grid.resize_height_with_anchor(2, 2);

    assert_eq!(grid.lines().len(), 2);
    assert_eq!(grid.lines()[0].cells()[0].contents(), Some("1"));
    assert_eq!(grid.lines()[1].cells()[0].contents(), Some("2"));

    grid.resize_height_with_anchor(1, 0);

    assert_eq!(grid.lines().len(), 1);
    assert_eq!(grid.lines()[0].cells()[0].contents(), Some("1"));

    grid.resize_height_with_anchor(5, 3);

    assert_eq!(grid.lines().len(), 5);
    assert_eq!(grid.lines()[0].cells()[0].contents(), Some("1"));
    for i in 1..5 {
        assert_eq!(
            grid.lines()[usize::from(i)].cells()[0].contents(),
            Some(" ")
        );
        grid.write_char(
            Vec2::new(0, i),
            i.to_string().chars().next().unwrap(),
            Style::default(),
        );
    }

    grid.resize_height_with_anchor(3, 3);

    assert_eq!(grid.lines().len(), 3);
    assert_eq!(grid.lines()[0].cells()[0].contents(), Some("1"));
    assert_eq!(grid.lines()[1].cells()[0].contents(), Some("2"));
    assert_eq!(grid.lines()[2].cells()[0].contents(), Some("3"));
}
