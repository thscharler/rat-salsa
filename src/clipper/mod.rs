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
//!     # use rat_widget::clipper::{Clipper, AreaHandle, ClipperLayout, ClipperState};
//!     # use rat_widget::checkbox::{Checkbox, CheckboxState};
//!     # use ratatui::prelude::*;
//!     #
//!     # let l2 = [Rect::ZERO, Rect::ZERO];
//!     # struct State {
//!     #      layout: ClipperLayout,
//!     #      handles: Vec<AreaHandle>,
//!     #      check_states: Vec<CheckboxState>,
//!     #      clipper: ClipperState
//!     #  }
//!     # let mut state = State {
//!     #      layout: ClipperLayout::new(1),
//!     #      handles: Vec::default(),
//!     #      clipper: Default::default(),
//!     #      check_states: Vec::default()
//!     #  };
//!     # let mut buf = Buffer::default();
//!     ///
//!     /// Create the layout. The layout can be stored long-term
//!     /// and needs to be rebuilt only if your widget layout changes.
//!     ///
//!     ///> __Note__: add() returns a handle for the area. Can be used later
//!     ///> to refer to the stored area.
//!
//!     if state.layout.is_empty() {
//!         let mut cl = ClipperLayout::new(1);
//!         for i in 0..100 {
//!             let handle = cl.add(&[Rect::new(10, i*11, 15, 10)]);
//!             state.handles[i as usize] = handle;
//!         }
//!     }
//!
//!     /// The given area plus the current scroll offset define the
//!     /// view area. With the view area a temporary buffer is created
//!     /// that is big enough to fit all widgets that are at least
//!     /// partially visible.
//!
//!     let clipper = Clipper::new();
//!
//!     let mut clip_buf = clipper
//!         .layout(state.layout.clone())
//!         .into_buffer(l2[1], &mut state.clipper);
//!
//!     ///
//!     /// The widgets are rendered to that buffer.
//!     ///
//!     for i in 0..100 {
//!         // create a new area
//!         let v_area = clip_buf.layout().layout_handle(state.handles[i])[0];
//!         let w_area = Rect::new(5, v_area.y, 5, 1);
//!         clip_buf.render_widget(Span::from(format!("{:?}:", i)), w_area);
//!
//!         // refer by handle
//!         clip_buf.render_stateful_handle(
//!             Checkbox::new()
//!                 .text(format!("{:?}", state.handles[i])),
//!             state.handles[i],
//!             0,
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
mod clipper_style;

pub use crate::commons::AreaHandle;
pub use clipper::*;
pub use clipper_style::*;
