//! Evolution of the table widget.
//!
//! It drops the table-data traits and replaces them
//! with separate closures. This keeps all the table-rendering
//! in one place. The actual table-data is provided as
//! an Iterator over the table-rows.
//!
//! See [rat-ftable2](https://docs.rs/rat-ftable2/)

pub use rat_ftable2::{
    LayoutWidth, Table, TableContext, TableState, TableStyle, TableVec, TableVecState, edit,
    selection,
};
