#![doc = include_str!("../readme.md")]

mod cellselection;
mod noselection;
mod rowselection;
mod rowsetselection;
mod table;

pub mod textdata;
mod util;

use crate::textdata::Row;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;

/// Render-context for rendering a table-cell.
#[derive(Debug)]
pub struct FTableContext {
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
    /// Row style if any.
    pub row_style: Option<Style>,
    /// Selection style if any.
    pub select_style: Option<Style>,

    /// Spacing after the cell. It's guaranteed that this
    /// is writeable in the buffer given to render_cell.
    pub space_area: Rect,

    pub non_exhaustive: NonExhaustive,
}

///
/// Trait for accessing the table-data by the FTable.
///
/// This trait is suitable if the underlying data is random access.
pub trait TableData<'a> {
    /// Size of the data.
    fn rows(&self) -> usize;

    /// Header can be obtained from here.
    /// Alternative to setting on FTable.
    fn header(&self) -> Option<Row<'a>> {
        None
    }

    /// Footer can be obtained from here.
    /// Alternative to setting on FTable.
    fn footer(&self) -> Option<Row<'a>> {
        None
    }

    /// Row height.
    #[allow(unused_variables)]
    fn row_height(&self, row: usize) -> u16 {
        1
    }

    /// Row style.
    #[allow(unused_variables)]
    fn row_style(&self, row: usize) -> Option<Style> {
        None
    }

    /// Column constraints.
    fn widths(&self) -> Vec<Constraint> {
        Vec::default()
    }

    /// Render the cell given by column/row.
    ///
    /// * ctx - a lot of context data.
    /// * full_area - area of the cell as defined.
    /// * area - area of the cell currently visible.
    fn render_cell(
        &self,
        ctx: &FTableContext,
        column: usize,
        row: usize,
        area: Rect,
        buf: &mut Buffer,
    );
}

/// Trait for accessing the table-data by the FTable.
///
/// This trait is suitable if the underlying data is an iterator.
/// It uses internal iteration which allows much more leeway with
/// borrowing & lifetimes.
///
pub trait TableDataIter<'a> {
    /// Returns the number of rows, if known.
    ///
    /// If they are not known, all items will be iterated to
    /// calculate things as the length of the table. Which will
    /// be slower if you have many items.
    fn rows(&self) -> Option<usize>;

    /// Header can be obtained from here.
    /// Alternative to setting on FTable.
    fn header(&self) -> Option<Row<'a>> {
        None
    }

    /// Footer can be obtained from here.
    /// Alternative to setting on FTable.
    fn footer(&self) -> Option<Row<'a>> {
        None
    }

    /// Skips to the nth item, returns true if such an item exists.
    /// nth(0) == next()
    fn nth(&mut self, n: usize) -> bool;

    /// Reads the next item, returns true if such an item exists.
    fn next(&mut self) -> bool {
        self.nth(0)
    }

    /// Row height for the current item.
    fn row_height(&self) -> u16 {
        1
    }

    /// Row style for the current line.
    fn row_style(&self) -> Option<Style> {
        None
    }

    /// Column constraints.
    fn widths(&self) -> Vec<Constraint> {
        Vec::default()
    }

    /// Render the cell for the current line.
    ///
    /// * ctx - a lot of context data.
    /// * full_area - area of the cell as defined.
    /// * area - area of the cell currently visible.
    fn render_cell(&self, ctx: &FTableContext, column: usize, area: Rect, buf: &mut Buffer);
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

use crate::_private::NonExhaustive;
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
        crossterm, ct_event, flow, flow_ok, util, ConsumedEvent, FocusKeys, HandleEvent, MouseOnly,
        Outcome,
    };

    /// Just checks for double-click on the table.
    ///
    /// Events for this event-map must be processed *before* calling
    /// any other event-handling routines for the same table.
    /// Otherwise, the regular event-handling might interfere with
    /// recognition of double-clicks by consuming the first click.
    #[derive(Debug, Default)]
    pub struct DoubleClick;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum DoubleClickOutcome {
        /// The given event has not been used at all.
        NotUsed,
        /// The event has been recognized, but the result was nil.
        /// Further processing for this event may stop.
        Unchanged,
        /// The event has been recognized and there is some change
        /// due to it.
        /// Further processing for this event may stop.
        /// Rendering the ui is advised.
        Changed,
        /// Double click found. Contains (column, row)
        ClickClick(usize, usize),
    }

    impl From<DoubleClickOutcome> for Outcome {
        fn from(value: DoubleClickOutcome) -> Self {
            match value {
                DoubleClickOutcome::NotUsed => Outcome::NotUsed,
                DoubleClickOutcome::Unchanged => Outcome::Unchanged,
                DoubleClickOutcome::Changed => Outcome::Changed,
                DoubleClickOutcome::ClickClick(_, _) => Outcome::Changed,
            }
        }
    }

    impl ConsumedEvent for DoubleClickOutcome {
        fn is_consumed(&self) -> bool {
            !matches!(self, DoubleClickOutcome::NotUsed)
        }
    }

    /// Activates editing behaviour in addition to the normal
    /// table event handling.
    #[derive(Debug, Default)]
    pub struct EditKeys;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum EditOutcome {
        /// The given event has not been used at all.
        NotUsed,
        /// The event has been recognized, but the result was nil.
        /// Further processing for this event may stop.
        Unchanged,
        /// The event has been recognized and there is some change
        /// due to it.
        /// Further processing for this event may stop.
        /// Rendering the ui is advised.
        Changed,
        /// Insert at the selection.
        Insert,
        /// Remove the selection.
        Remove,
        /// Edit the selection.
        Edit,
        /// Append after last row.
        Append,
    }

    impl From<Outcome> for EditOutcome {
        fn from(value: Outcome) -> Self {
            match value {
                Outcome::NotUsed => EditOutcome::NotUsed,
                Outcome::Unchanged => EditOutcome::Unchanged,
                Outcome::Changed => EditOutcome::Changed,
            }
        }
    }

    impl From<EditOutcome> for Outcome {
        fn from(value: EditOutcome) -> Self {
            match value {
                EditOutcome::NotUsed => Outcome::NotUsed,
                EditOutcome::Unchanged => Outcome::Unchanged,
                EditOutcome::Changed => Outcome::Changed,
                EditOutcome::Insert => Outcome::Unchanged,
                EditOutcome::Remove => Outcome::Unchanged,
                EditOutcome::Edit => Outcome::Unchanged,
                EditOutcome::Append => Outcome::Unchanged,
            }
        }
    }

    impl ConsumedEvent for EditOutcome {
        fn is_consumed(&self) -> bool {
            !matches!(self, EditOutcome::NotUsed)
        }
    }
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
