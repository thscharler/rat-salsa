mod cellselection;
mod noselection;
mod rowselection;
mod table;
pub mod textdata;
pub mod util;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

/// Trait for accessing the table-data by the FTable.
pub trait TableData<'a> {
    /// Size of the data.
    fn size(&self) -> (usize, usize);

    /// Row height.
    fn row_height(&self, row: usize) -> u16;

    /// Row style.
    fn row_style(&self, row: usize) -> Style;

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
    pub use crate::cellselection::CellSelection;
    pub use crate::noselection::NoSelection;
    pub use crate::rowselection::RowSelection;
}

pub mod event {
    pub use rat_event::{FocusKeys, HandleEvent, MouseOnly};

    /// Result type for event-handling. Used by widgets in this crate.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Outcome {
        /// The given event was not handled at all.
        NotUsed,
        /// The event was handled, no repaint necessary.
        Unchanged,
        /// The event was handled, repaint necessary.
        Changed,
    }
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
