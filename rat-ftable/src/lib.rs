#![doc = include_str!("../readme.md")]

mod cellselection;
pub mod edit;
mod noselection;
mod rowselection;
mod rowsetselection;
mod table;
pub mod textdata;
mod util;

use crate::textdata::Row;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Rect};
use ratatui_core::style::Style;

/// Render-context for rendering a table-cell.
#[derive(Debug)]
pub struct TableContext {
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
    /// Total area for the current row.
    pub row_area: Rect,

    /// Construct with `..Default::default()`
    pub non_exhaustive: NonExhaustive,
}

///
/// Trait for accessing the table-data by the Table.
///
/// This trait is suitable if the underlying data is some sort
/// of vec/slice.
pub trait TableData<'a> {
    /// Size of the data.
    fn rows(&self) -> usize;

    /// Header can be obtained from here.
    /// Alternative to setting on Table.
    fn header(&self) -> Option<Row<'a>> {
        None
    }

    /// Footer can be obtained from here.
    /// Alternative to setting on Table.
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
    /// * ctx - a lot of context data.
    fn render_cell(
        &self,
        ctx: &TableContext,
        column: usize,
        row: usize,
        area: Rect,
        buf: &mut Buffer,
    );
}

impl<'a> TableData<'a> for Box<dyn TableData<'a> + 'a> {
    fn rows(&self) -> usize {
        (**self).rows()
    }

    fn header(&self) -> Option<Row<'a>> {
        (**self).header()
    }

    fn footer(&self) -> Option<Row<'a>> {
        (**self).footer()
    }

    fn row_height(&self, row: usize) -> u16 {
        (**self).row_height(row)
    }

    fn row_style(&self, row: usize) -> Option<Style> {
        (**self).row_style(row)
    }

    fn widths(&self) -> Vec<Constraint> {
        (**self).widths()
    }

    fn render_cell(
        &self,
        ctx: &TableContext,
        column: usize,
        row: usize,
        area: Rect,
        buf: &mut Buffer,
    ) {
        (**self).render_cell(ctx, column, row, area, buf)
    }
}

/// Trait for accessing the table-data by the Table.
///
/// This trait is suitable if the underlying data is an iterator.
pub trait TableDataIter<'a> {
    /// StatefulWidgetRef needs a clone of the iterator for every render.
    /// For StatefulWidget this is not needed at all. So this defaults to
    /// None and warns at runtime.
    fn cloned(&self) -> Option<Box<dyn TableDataIter<'a> + 'a>> {
        None
    }

    /// Returns the number of rows, if known.
    ///
    /// If they are not known, all items will be iterated to
    /// calculate things as the length of the table. Which will
    /// be slower if you have many items.
    ///
    /// See [Table::no_row_count]
    fn rows(&self) -> Option<usize>;

    /// Header can be obtained from here.
    /// Alternative to setting on Table.
    fn header(&self) -> Option<Row<'a>> {
        None
    }

    /// Footer can be obtained from here.
    /// Alternative to setting on Table.
    fn footer(&self) -> Option<Row<'a>> {
        None
    }

    /// Skips to the nth item, returns true if such an item exists.
    /// nth(0) == next()
    fn nth(&mut self, n: usize) -> bool;

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
    /// * ctx - a lot of context data.
    fn render_cell(&self, ctx: &TableContext, column: usize, area: Rect, buf: &mut Buffer);
}

/// Trait for the different selection models used by Table.
pub trait TableSelection {
    /// Number of rowss selected.
    fn count(&self) -> usize;

    /// Row is selected. This can be separate from `is_selected_cell`.
    fn is_selected_row(&self, row: usize) -> bool;

    /// Column is selected. This can be separate from `is_selected_cell`.
    fn is_selected_column(&self, column: usize) -> bool;

    /// Specific cell is selected.
    fn is_selected_cell(&self, column: usize, row: usize) -> bool;

    /// Selection lead, or the sole selected index.
    fn lead_selection(&self) -> Option<(usize, usize)>;
}

use crate::_private::NonExhaustive;

pub use table::{handle_doubleclick_events, Table, TableState, TableStyle};

/// Different selection models for Table.
pub mod selection {
    pub use crate::cellselection::CellSelection;
    pub mod cellselection {
        pub use crate::cellselection::{handle_events, handle_mouse_events};
    }
    pub use crate::noselection::NoSelection;
    pub mod noselection {
        pub use crate::noselection::{handle_events, handle_mouse_events};
    }
    pub use crate::rowselection::RowSelection;
    pub mod rowselection {
        pub use crate::rowselection::{handle_events, handle_mouse_events};
    }
    pub use crate::rowsetselection::RowSetSelection;
    pub mod rowsetselection {
        pub use crate::rowsetselection::{handle_events, handle_mouse_events};
    }
}

/// Eventhandling.
pub mod event {
    pub use rat_event::*;

    /// Result value for event-handling.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[non_exhaustive]
    pub enum TableOutcome {
        /// The given event was not handled at all.
        Continue,
        /// The event was handled, no repaint necessary.
        Unchanged,
        /// The event was handled, repaint necessary.
        Changed,
        /// The selection has changed.
        Selected,
    }

    impl ConsumedEvent for TableOutcome {
        fn is_consumed(&self) -> bool {
            *self != TableOutcome::Continue
        }
    }

    impl From<TableOutcome> for Outcome {
        fn from(value: TableOutcome) -> Self {
            match value {
                TableOutcome::Continue => Outcome::Continue,
                TableOutcome::Unchanged => Outcome::Unchanged,
                TableOutcome::Changed => Outcome::Changed,
                TableOutcome::Selected => Outcome::Changed,
            }
        }
    }

    /// Result type for double-click event-handling.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum DoubleClickOutcome {
        /// The given event has not been used at all.
        Continue,
        /// The event has been recognized, but the result was nil.
        /// Further processing for this event may stop.
        Unchanged,
        /// The event has been recognized and there is some change
        /// due to it.
        /// Further processing for this event may stop.
        /// Rendering the ui is advised.
        Changed,
        /// Double click occurred. Contains (column, row)
        ClickClick(usize, usize),
    }

    impl From<DoubleClickOutcome> for Outcome {
        fn from(value: DoubleClickOutcome) -> Self {
            match value {
                DoubleClickOutcome::Continue => Outcome::Continue,
                DoubleClickOutcome::Unchanged => Outcome::Unchanged,
                DoubleClickOutcome::Changed => Outcome::Changed,
                DoubleClickOutcome::ClickClick(_, _) => Outcome::Changed,
            }
        }
    }

    impl From<Outcome> for DoubleClickOutcome {
        fn from(value: Outcome) -> Self {
            match value {
                Outcome::Continue => DoubleClickOutcome::Continue,
                Outcome::Unchanged => DoubleClickOutcome::Unchanged,
                Outcome::Changed => DoubleClickOutcome::Changed,
            }
        }
    }

    impl ConsumedEvent for DoubleClickOutcome {
        fn is_consumed(&self) -> bool {
            !matches!(self, DoubleClickOutcome::Continue)
        }
    }

    /// Result type for the [edit](crate::edit) widgets.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum EditOutcome {
        /// The given event has not been used at all.
        Continue,
        /// The event has been recognized, but the result was nil.
        /// Further processing for this event may stop.
        Unchanged,
        /// The event has been recognized and there is some change
        /// due to it.
        /// Further processing for this event may stop.
        /// Rendering the ui is advised.
        Changed,
        /// Cancel an ongoing edit.
        Cancel,
        /// Commit the current edit.
        Commit,
        /// Commit the edit, append a new line.
        CommitAndAppend,
        /// Commit the edit, edit next line.
        CommitAndEdit,
        /// Insert an item at the selection.
        Insert,
        /// Remove the item at the selection.
        Remove,
        /// Edit the item at the selection.
        Edit,
        /// Append an item after last row.
        /// Might want to start the edit too.
        Append,
    }

    impl From<Outcome> for EditOutcome {
        fn from(value: Outcome) -> Self {
            match value {
                Outcome::Continue => EditOutcome::Continue,
                Outcome::Unchanged => EditOutcome::Unchanged,
                Outcome::Changed => EditOutcome::Changed,
            }
        }
    }

    impl From<EditOutcome> for Outcome {
        fn from(value: EditOutcome) -> Self {
            match value {
                EditOutcome::Continue => Outcome::Continue,
                EditOutcome::Unchanged => Outcome::Unchanged,
                EditOutcome::Changed => Outcome::Changed,
                EditOutcome::Insert => Outcome::Unchanged,
                EditOutcome::Remove => Outcome::Unchanged,
                EditOutcome::Edit => Outcome::Unchanged,
                EditOutcome::Append => Outcome::Unchanged,
                EditOutcome::Cancel => Outcome::Unchanged,
                EditOutcome::Commit => Outcome::Unchanged,
                EditOutcome::CommitAndAppend => Outcome::Unchanged,
                EditOutcome::CommitAndEdit => Outcome::Unchanged,
            }
        }
    }

    impl ConsumedEvent for EditOutcome {
        fn is_consumed(&self) -> bool {
            !matches!(self, EditOutcome::Continue)
        }
    }
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
