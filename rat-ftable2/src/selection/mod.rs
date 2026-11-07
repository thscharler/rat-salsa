//! Different selection models for Table.

mod cellselection;
mod noselection;
mod rowselection;
mod rowsetselection;

pub use rat_ftable::selection::CellSelection;
pub use rat_ftable::selection::NoSelection;
pub use rat_ftable::selection::RowSelection;
pub use rat_ftable::selection::RowSetSelection;
