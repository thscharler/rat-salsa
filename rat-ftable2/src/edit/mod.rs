//!
//! Table-widget and row-editor.
//!

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;

mod state;
mod widget;

pub use state::*;
pub use widget::*;

/// This trait renders the row-editor widgets.
///
/// It gets a slice of areas for all the cells in the table,
/// and renders all input widgets as it needs.
pub trait TableEditor<T> {
    /// State associated with the stateful widget.
    type State: TableEditorState<T>;

    /// Standard render call, but with added areas for each cell.
    /// The area is the area of the full row.
    fn render(
        &self, //
        area: Rect,
        cell_areas: &[Rect],
        buf: &mut Buffer,
        state: &mut Self::State,
    );
}

///
/// Trait for the editor widget state.
///
#[allow(unused_variables)]
pub trait TableEditorState<T> {
    /// Some external context.
    type Context<'a>;
    /// Error type.
    type Err;

    /// Create a fresh row value with all the defaults.
    fn create_value<'a>(&self, ctx: &'a Self::Context<'a>) -> Result<T, Self::Err>;

    /// Set the current row value for the editor.
    fn set_value<'a>(&mut self, value: T, ctx: &'a Self::Context<'a>) -> Result<(), Self::Err>;

    /// Return the current row value from the editor.
    /// None results will remove the table row.
    fn value<'a>(&mut self, ctx: &'a Self::Context<'a>) -> Result<Option<T>, Self::Err>;

    /// Returns the currently focused column.
    /// Used to scroll the column to fully visible.
    fn focused_col(&self) -> Option<usize> {
        None
    }

    /// Start editing the current value.
    /// If a specific column should be focused initially it is given as
    /// the column index.
    fn start<'a>(
        &mut self,
        col: Option<usize>,
        ctx: &'a Self::Context<'a>,
    ) -> Result<(), Self::Err> {
        Ok(())
    }

    /// Stop editing the current value.
    /// If necessary the editor can do some cleanup here.
    fn stop<'a>(&mut self, ctx: &'a Self::Context<'a>) -> Result<(), Self::Err> {
        Ok(())
    }
}

/// Editing mode.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    #[default]
    View,
    Edit,
    Insert,
}
