#![doc = include_str!("../readme.md")]

mod cellselection;
mod noselection;
mod rowselection;
mod rowsetselection;
mod table;

pub mod textdata;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

/// Trait for accessing the table-data by the FTable.
pub trait TableData<'a> {
    /// Size of the data.
    fn size(&self) -> (usize, usize);

    /// Row height.
    #[allow(unused_variables)]
    fn row_height(&self, row: usize) -> u16 {
        1
    }

    /// Row style.
    #[allow(unused_variables)]
    fn row_style(&self, row: usize) -> Style {
        Style::default()
    }

    /// Render the cell given by column/row.
    fn render_cell(&self, column: usize, row: usize, area: Rect, buf: &mut Buffer);
}

/// Trait for the different selection models used by FTable.
pub trait TableSelection {
    /// Row is selected. This can be separate from `is_selected_cell`.
    fn is_selected_row(&self, row: usize) -> bool;

    /// Column is selected. This can be separate from `is_selected_cell`.
    fn is_selected_column(&self, column: usize) -> bool;

    /// Specific cell is selected.
    fn is_selected_cell(&self, column: usize, row: usize) -> bool;

    /// Selection lead.
    fn lead_selection(&self) -> Option<(usize, usize)>;
}

pub use table::{FTable, FTableState, FTableStyle};

pub mod selection {
    //! Different selection models for FTable.
    pub use crate::cellselection::CellSelection;
    pub use crate::noselection::NoSelection;
    pub use crate::rowselection::RowSelection;
    pub use crate::rowsetselection::RowSetSelection;
}

pub mod event {
    //! Rexported eventhandling traits.
    pub use rat_event::{
        crossterm, ct_event, util, ConsumedEvent, FocusKeys, HandleEvent, MouseOnly, Outcome,
    };
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
