//!
//! An alternative view widget.
//!
//! > The extra requirement for this one is that you can create
//! > a Layout that defines the bounds of all widgets that can
//! > be rendered.
//!
//! It works in 4 phases:
//!
//! ```rust no_run
//!     # use rat_widget::clipper::{Clipper, ClipperState};
//!     # use rat_widget::checkbox::{Checkbox, CheckboxState};
//!     # use ratatui::prelude::*;
//!     # use rat_focus::{FocusFlag, HasFocus};
//!     # use rat_widget::layout::GenericLayout;
//!     #
//!     # let l2 = [Rect::ZERO, Rect::ZERO];
//!     # struct State {
//!     #      check_states: Vec<CheckboxState>,
//!     #      clipper: ClipperState<FocusFlag>
//!     #  }
//!     # let mut state = State {
//!     #      clipper: Default::default(),
//!     #      check_states: Vec::default()
//!     #  };
//!     # let mut buf = Buffer::default();
//!
//!     /// Create the layout. The layout can be stored long-term
//!     /// and needs to be rebuilt only if your widget layout changes.
//!
//!     let clipper = Clipper::new();
//!     let layout_size = clipper.layout_size(l2[1], &mut state.clipper);
//!
//!     if !state.clipper.valid_layout(layout_size) {
//!         let mut cl = GenericLayout::new();
//!         for i in 0..100 {
//!             cl.add(state.check_states[i].focus(),
//!                 Rect::new(10, i as u16 *11, 15, 10),
//!                 None,
//!                 Rect::default()
//!             );
//!         }
//!         state.clipper.set_layout(cl);
//!     }
//!
//!     /// The given area plus the current scroll offset define the
//!     /// view area. With the view area a temporary buffer is created
//!     /// that is big enough to fit all widgets that are at least
//!     /// partially visible.
//!
//!     let mut clip_buf = clipper
//!         .into_buffer(l2[1], &mut state.clipper);
//!
//!     ///
//!     /// The widgets are rendered to that buffer.
//!     ///
//!     for i in 0..100 {
//!         // refer by handle
//!         clip_buf.render(
//!             state.check_states[i].focus(),
//!             || {
//!                 Checkbox::new()
//!                 .text(format!("{:?}", i))
//!             },
//!             &mut state.check_states[i],
//!         );
//!     }
//!
//!     ///
//!     /// The last step clips and copies the buffer to the frame buffer.
//!     ///
//!
//!     clip_buf
//!         .into_widget()
//!         .render(l2[1], &mut buf, &mut state.clipper);
//!
//! ```
//!
//! __StatefulWidget__
//!
//! For this to work with StatefulWidgets they must cooperate
//! by implementing the [RelocatableState](crate::reloc::RelocatableState)
//! trait. With this trait the widget can clip/hide all areas that
//! it stores in its state.
//!
//! __See__
//!
//! [example](https://github.com/thscharler/rat-widget/blob/master/examples/clipper1.rs)
//!

#[allow(clippy::module_inception)]
mod clipper;
mod clipper_style;

pub use clipper::*;
pub use clipper_style::*;
