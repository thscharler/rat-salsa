//! Extra widgets for inline editing in a table.
//!
//! Extra keys while viewing are
//! * Insert - Insert a row and start the editor widget.
//! * Delete - Delete row.
//! * Enter - Start editor widget.
//! * Double-Click - Start editor widget.
//! * Down - Append after the last row and start the editor widget.
//!
//! Keys while editing are
//! * Esc - Cancel editing.
//! * Enter - Commit current edit and edit next/append a row.
//! * Up/Down - Commit current edit.
use rat_focus::HasFocus;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

pub mod table;
pub mod vec;

/// StatefulWidget alike trait.
///
/// This one takes a slice of areas for all the cells in the table,
/// and renders all input widgets as it needs.
pub trait TableEditor {
    /// State associated with the stateful widget.
    type State: TableEditorState;

    /// Standard render call, but with added areas for each cell.
    fn render(&self, area: Rect, cell_areas: &[Rect], buf: &mut Buffer, state: &mut Self::State);
}

/// Trait for the editor widget state
pub trait TableEditorState: HasFocus {
    type Context<'a>: Clone;
    type Data: Clone;
    type Err;

    /// Create default data.
    fn new_edit_data(&self, ctx: Self::Context<'_>) -> Result<Self::Data, Self::Err>;

    /// Set editing data.
    fn set_edit_data(&mut self, data: &Self::Data, ctx: Self::Context<'_>)
        -> Result<(), Self::Err>;

    /// Copy the editor state back to the data.
    fn get_edit_data(
        &mut self,
        data: &mut Self::Data,
        ctx: Self::Context<'_>,
    ) -> Result<(), Self::Err>;

    /// Is this some empty state?
    fn is_empty(&self) -> bool;

    /// Returns the currently focused column.
    fn focused_col(&self) -> Option<usize>;
}

/// Editing mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    View,
    Edit,
    Insert,
}
