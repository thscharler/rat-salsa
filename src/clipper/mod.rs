//!
//! An alternative view widget.
//!
//! The extra requirement is that you can create a Layout that
//! contains the bounds of all widgets that can be rendered.
//!
//! It works in 4 phases:
//!
//! * Create the layout. The layout can be stored long-term
//!   and needs to be rebuilt only if your widget layout changes.
//!
//! ```
//! ```
//!
//! * With the scroll offset the visible area for this layout is
//!   calculated. Starting from that an extended visible area
//!   is computed, that contains the bounds for all
//!   visible/partially visible widgets.
//!
//! ```
//! ```
//!
//! * The widgets are rendered to that buffer.
//!
//! ```
//! ```
//!
//! * The last step clips and copies the buffer to the frame buffer.
//!
//! ```
//! ```
//!
//! __StatefulWidget__
//!
//! For this to work with StatefulWidgets they must cooperate
//! by implementing the [RelocatableState](crate::relocate::RelocatableState)
//! trait. With this trait the widget can clip/hide all areas that
//! it stores in its state.
//!
//! __See__
//!
//! [example](https://github.com/thscharler/rat-widget/blob/master/examples/clipper1.rs)
//!

#[allow(clippy::module_inception)]
mod clipper;
mod clipper_layout;

pub use crate::commons::AreaHandle;
pub use clipper::*;
pub use clipper_layout::*;
