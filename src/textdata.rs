//!
//! Implements a Row and a Cell struct that are compatible to ratatui.
//! You only need these if you use preformatted data.
//!

use crate::TableData;
use crate::_private::NonExhaustive;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Style, Text};
use ratatui::style::Styled;
use ratatui::widgets::WidgetRef;

/// Internal impl for TableData using prerendered Cells.
#[derive(Debug, Default, Clone)]
pub(crate) struct TextTableData<'a> {
    pub(crate) columns: usize,
    pub(crate) rows: Vec<Row<'a>>,
}

/// Rows of the table.
#[derive(Debug, Clone)]
pub struct Row<'a> {
    pub cells: Vec<Cell<'a>>,
    pub top_margin: u16,
    pub height: u16,
    pub bottom_margin: u16,
    pub style: Style,

    pub non_exhaustive: NonExhaustive,
}

/// A single cell of the table.
#[derive(Debug, Clone)]
pub struct Cell<'a> {
    pub content: Text<'a>,
    pub style: Style,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> TableData<'a> for TextTableData<'a> {
    fn size(&self) -> (usize, usize) {
        (self.columns, self.rows.len())
    }

    fn row_height(&self, r: usize) -> u16 {
        if let Some(row) = self.rows.get(r) {
            row.top_margin + row.height + row.bottom_margin
        } else {
            0
        }
    }

    fn row_style(&self, r: usize) -> Style {
        if let Some(row) = self.rows.get(r) {
            row.style
        } else {
            Style::default()
        }
    }

    fn render_cell(&self, c: usize, r: usize, area: Rect, buf: &mut Buffer) {
        if let Some(row) = self.rows.get(r) {
            buf.set_style(area, row.style);
            if let Some(cell) = row.cell(c) {
                buf.set_style(area, cell.style);
                cell.content.render_ref(area, buf);
            }
        }
    }
}

impl<'a> Default for Row<'a> {
    fn default() -> Self {
        Self {
            cells: Default::default(),
            top_margin: 0,
            height: 0,
            bottom_margin: 0,
            style: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a> Styled for Row<'a> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        self.style(style)
    }
}

impl<'a, Item> FromIterator<Item> for Row<'a>
where
    Item: Into<Cell<'a>>,
{
    fn from_iter<T: IntoIterator<Item = Item>>(cells: T) -> Self {
        Self::new(cells)
    }
}

impl<'a> Row<'a> {
    /// New row of data cells.
    pub fn new<T>(cells: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<Cell<'a>>,
    {
        Self {
            cells: cells.into_iter().map(|v| v.into()).collect(),
            height: 1,
            ..Default::default()
        }
    }

    /// Set the data cells for the row.
    pub fn cells<T>(mut self, cells: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<Cell<'a>>,
    {
        self.cells = cells.into_iter().map(Into::into).collect();
        self
    }

    /// Set the row-height.
    #[inline]
    pub fn height(mut self, height: u16) -> Self {
        self.height = height;
        self
    }

    /// Add some margin.
    pub fn top_margin(mut self, margin: u16) -> Self {
        self.top_margin = margin;
        self
    }

    /// Add some margin.
    pub fn bottom_margin(mut self, margin: u16) -> Self {
        self.bottom_margin = margin;
        self
    }

    /// Rowstyle.
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    /// Access to the cell.
    pub fn cell<'b: 'a>(&'b self, c: usize) -> Option<&'a Cell<'a>> {
        if let Some(t) = self.cells.get(c) {
            Some(t)
        } else {
            None
        }
    }
}

impl<'a> Default for Cell<'a> {
    fn default() -> Self {
        Self {
            content: Default::default(),
            style: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a, T> From<T> for Cell<'a>
where
    T: Into<Text<'a>>,
{
    fn from(value: T) -> Self {
        Self {
            content: value.into(),
            style: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a> Styled for Cell<'a> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self {
        self.style(style)
    }
}

impl<'a> Cell<'a> {
    /// New Cell.
    pub fn new<T>(content: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        Self {
            content: content.into(),
            style: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }

    /// Set the cell content.
    pub fn content<T>(mut self, content: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        self.content = content.into();
        self
    }

    /// Cell style.
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }
}
