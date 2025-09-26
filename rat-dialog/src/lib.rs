#![allow(clippy::question_mark)]
#![allow(clippy::type_complexity)]

mod dialog_control;
mod window_control;

pub use dialog_control::{DialogControl, DialogStack, handle_dialog_stack};
pub use window_control::window_frame::{
    WindowFrame, WindowFrameOutcome, WindowFrameState, WindowFrameStyle,
};
pub use window_control::{Window, WindowControl, WindowList, handle_window_list};

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
