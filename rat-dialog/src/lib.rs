#![allow(clippy::question_mark)]
#![allow(clippy::type_complexity)]

mod dialog_stack;
mod window;

pub use dialog_stack::DialogStack;
pub use window::{Window, WindowState};

pub mod event {
    pub use crate::dialog_stack::DialogControl;
    pub use crate::window::WindowOutcome;
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
