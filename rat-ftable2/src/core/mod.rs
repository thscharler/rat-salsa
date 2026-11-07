//! The core of the various table implementations.
//!
//! It doesn't hold any reference to the table-data, instead
//! it uses a special render function that gets the data-iterator
//! along the other parameters.

mod state;
mod widget;

pub use state::*;
pub use widget::*;
