#![doc = include_str!("../readme.md")]

pub mod core;
mod derive;
pub mod edit;
pub mod event;
pub mod selection;
mod style;
mod table;
mod util;
mod vec;

use crate::_private::NonExhaustive;
use ratatui_core::layout::Rect;
use ratatui_core::style::Style;
pub use style::TableStyle;
pub use table::{Table, TableState};
pub use vec::{TableVec, TableVecState};

/// Width for the table-layout.
#[derive(Debug, Default)]
pub enum LayoutWidth {
    /// Fixed width. Columns are layout for this size.
    Length(u16),
    /// Width is the sum of the columns. If the width is
    /// larger than the area, horizontal scrolling is used.
    ///
    /// Note: Constraint::Fill is useless for this.
    /// Such columns will assume a width of 10.
    Columns,
    /// Fit the columns to the table-area.
    #[default]
    Area,
}

/// Render-context for rendering a table-cell.
#[derive(Debug)]
pub struct RenderContext<'a> {
    /// Current row.
    pub row: usize,

    /// Focus flag is set.
    pub focus: bool,

    /// Cell is selected.
    pub selected_cell: bool,
    /// Row of the cell is selected.
    pub selected_row: bool,
    /// Column of the cell is selected.
    pub selected_column: bool,

    /// Base style
    pub style: Style,
    /// Selection style if any.
    pub select_style: Option<Style>,

    /// Column layout for all columns.
    /// Only the x-position and width are set,
    /// y-position and height are 0.
    pub column_layout: &'a [Rect],
    /// Spacing after the current cell.
    /// Will be empty when calling row-rendering.
    pub space_area: Rect,
    /// Total area for the current row.
    pub row_area: Rect,

    /// Construct with `..Default::default()`
    pub non_exhaustive: NonExhaustive,
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
