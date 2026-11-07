//! Different selection models for Table.

mod cellselection;
mod noselection;
mod rowselection;
mod rowsetselection;

pub use cellselection::CellSelection;
pub use noselection::NoSelection;
pub use rowselection::RowSelection;
pub use rowsetselection::RowSetSelection;
