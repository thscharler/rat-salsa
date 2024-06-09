//!
//! Implements a Row and a Cell struct that are compatible to ratatui.
//! You only need these if you use preformatted data.
//!

use crate::_private::NonExhaustive;
use crate::{FTableContext, TableData};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Style, Text};
use ratatui::style::Styled;
use ratatui::widgets::Widget;

/// Internal impl for TableData using pre-rendered Cells.
#[derive(Debug, Default, Clone)]
pub(crate) struct TextTableData<'a> {
    pub(crate) rows: Vec<Row<'a>>,
}

/// Rows of the table.
#[derive(Debug, Clone)]
pub struct Row<'a> {
    pub cells: Vec<Cell<'a>>,
    pub top_margin: u16,
    pub height: u16,
    pub bottom_margin: u16,
    pub style: Option<Style>,

    pub non_exhaustive: NonExhaustive,
}

/// A single cell of the table.
#[derive(Debug, Clone)]
pub struct Cell<'a> {
    pub content: Text<'a>,
    pub style: Option<Style>,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> TableData<'a> for TextTableData<'a> {
    fn rows(&self) -> usize {
        self.rows.len()
    }

    fn row_height(&self, r: usize) -> u16 {
        if let Some(row) = self.rows.get(r) {
            row.top_margin + row.height + row.bottom_margin
        } else {
            0
        }
    }

    fn row_style(&self, r: usize) -> Option<Style> {
        if let Some(row) = self.rows.get(r) {
            row.style
        } else {
            None
        }
    }

    fn render_cell(&self, _ctx: &FTableContext, c: usize, r: usize, area: Rect, buf: &mut Buffer) {
        if let Some(row) = self.rows.get(r) {
            if let Some(cell) = row.cell(c) {
                if let Some(style) = cell.style {
                    buf.set_style(area, style);
                }
                cell.content.clone().render(area, buf);
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
        self.style.unwrap_or_default()
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.style = Some(style.into());
        self
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
        let mut s = Self {
            cells: cells.into_iter().map(|v| v.into()).collect(),
            height: 1,
            ..Default::default()
        };
        // content heigth
        if let Some(height) = s.cells.iter().map(|v| v.content.height()).max() {
            s.height = height as u16;
        }
        s
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
    pub fn style(mut self, style: Option<Style>) -> Self {
        self.style = style;
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
        self.style.unwrap_or_default()
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = Some(style.into());
        self
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
    pub fn style(mut self, style: Option<Style>) -> Self {
        self.style = style;
        self
    }
}
