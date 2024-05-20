#![doc = include_str!("../readme.md")]

pub mod button;
pub mod calender;
pub mod date_input;
pub mod ftable;
pub mod input;
pub mod list;
pub mod masked_input;
pub mod menuline;

pub mod layout {
    pub use rat_input::layout_dialog::{layout_dialog, LayoutDialog};
    pub use rat_input::layout_edit::{layout_edit, EditConstraint, LayoutEdit, LayoutEditIterator};
}

pub mod msgdialog {
    pub use rat_input::msgdialog::{MsgDialog, MsgDialogState, MsgDialogStyle};
}

pub mod statusline {
    pub use rat_input::statusline::{StatusLine, StatusLineState};
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
