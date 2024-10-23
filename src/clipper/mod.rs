//!
//! An alternative view widget.
//!
//! > The extra requirement for this one is that you can create
//! > a Layout that defines the bounds of all widgets that can
//! > be rendered.
//!
//! It works in 4 phases:
//!
//! * Create the layout. The layout can be stored long-term
//!   and needs to be rebuilt only if your widget layout changes.
//!
//!   > __Note__: add() returns a handle for the area. Can be used later
//!   > to refer to the stored area.
//!
//!     ```rust ignore
//!         if state.layout.is_empty() {
//!             let mut cl = ClipperLayout::new();
//!             for i in 0..100 {
//!                 let handle = cl.add(Rect::new(10, i*11, 15, 10));
//!                 state.handles[i] = handle;
//!             }
//!         }
//!     ```
//!
//! * The given area plus the current scroll offset define the
//!   view area. With the view area a temporary buffer is created
//!   that is big enough to fit all widgets that are at least
//!   partially visible.
//!
//!     ```rust ignore
//!         let clipper = Clipper::new()
//!             .block(Block::bordered())
//!             .hscroll(Scroll::new())
//!             .vscroll(Scroll::new());
//!
//!         let mut clip_buf = clipper
//!             .layout(state.layout.clone())
//!             .into_buffer(l2[1], &mut state.clipper);
//!     ```
//!
//! * The widgets are rendered to that buffer.
//!
//!   Either ad hoc
//!     ```rust ignore
//!         let v_area = clip_buf.layout_area(state.handles[i]);
//!         let w_area = Rect::new(5, v_area.y, 5, 1);
//!         clip_buf.render_widget(Span::from(format!("{:?}:", i)), w_area);
//!     ```
//!   or by referring to a handle
//!     ```rust ignore
//!         clip_buf.render_stateful_handle(
//!             TextInputMock::default()
//!                 .sample(format!("{:?}", state.hundred_areas[i]))
//!                 .style(THEME.limegreen(0))
//!                 .focus_style(THEME.limegreen(2)),
//!             state.handles[i],
//!             &mut state.widget_states[i],
//!         );
//!     ```
//!
//! * The last step clips and copies the buffer to the frame buffer.
//!
//!     ```rust ignore
//!         clip_buf
//!             .into_widget()
//!             .render(l2[1], frame.buffer_mut(), &mut state.clipper);
//!     ```
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
