//!
//! Widgets that render different window-frames.
//!
//! All widgets have a widget_area in their state, that is the
//! area where you can render the window content.
//!
//! * MacFrame
//!
//! Frame with Mac like min/max/close buttons. Moveable and resizable.
//!
//! * WindowFrame
//!
//! Classic TUI window style. Double-click the title to maximize.
//! Moveable and resizable.
//!

mod frame_state;
mod mac_frame;
mod window_frame;

pub use frame_state::*;
pub use mac_frame::*;
pub use window_frame::*;
