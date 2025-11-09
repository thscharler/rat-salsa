//! Table widget.
//!
//! Can be used as a drop-in replacement for the ratatui table. But
//! that's not the point of this widget.
//!
//! This widget uses the [TableData] trait instead
//! of rendering all the table-cells and putting them into a Vec.
//! This way rendering time only depends on the screen-size not on
//! the size of your data.
//!
//! There is a second trait [TableDataIter] that
//! works better if you only have an Iterator over your data.
//!
//! See [rat-ftable](https://docs.rs/rat-ftable/)
pub use rat_ftable::{
    Table, TableContext, TableData, TableDataIter, TableSelection, TableState, TableStyle, edit,
    handle_doubleclick_events, selection, textdata,
};
