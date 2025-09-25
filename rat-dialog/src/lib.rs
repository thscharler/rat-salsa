#![allow(clippy::question_mark)]
#![allow(clippy::type_complexity)]

mod dialog_control;
mod window_control;

pub use dialog_control::{DialogControl, DialogStack, handle_dialog_stack};
pub use window_control::{WindowControl, WindowList, handle_window_list};
