//! Selection models for the table.

pub mod cell;
pub mod none;
pub mod row;
pub mod rowset;

/// Trait for the different selection models used by Table.
pub trait TableSelection {
    /// Clear the selection.
    fn clear(&mut self);

    /// Row is selected. This can be separate from `is_selected_cell`.
    fn is_selected_row(&self, row: usize) -> bool;

    /// Column is selected. This can be separate from `is_selected_cell`.
    fn is_selected_column(&self, column: usize) -> bool;

    /// Specific cell is selected.
    fn is_selected_cell(&self, column: usize, row: usize) -> bool;

    /// Selection lead, or the sole selected index.
    fn lead_selection(&self) -> Option<(usize, usize)>;

    /// Validate the selected row against the number of rows.
    #[allow(unused_variables)]
    fn validate_rows(&mut self, rows: usize) {}

    /// Validate the selected column against the number of columns.
    #[allow(unused_variables)]
    fn validate_cols(&mut self, cols: usize) {}

    /// Correct the selection for added items.
    #[allow(unused_variables)]
    fn items_added(&mut self, pos: usize, n: usize) {}

    /// Correct the selection for removed items.
    #[allow(unused_variables)]
    fn items_removed(&mut self, pos: usize, n: usize, rows: usize) {}
}
