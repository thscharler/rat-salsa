#![doc = include_str!("../readme.md")]
#![allow(clippy::collapsible_else_if)]

pub mod button;
pub mod calender;
pub mod date_input;
pub mod ftable;
pub mod input;
pub mod list;
pub mod masked_input;
pub mod menuline;
pub mod textarea;

pub use pure_rust_locales::Locale;

pub mod focus {
    pub use rat_focus::{match_focus, on_gained, on_lost, Focus, FocusFlag, HasFocusFlag};
}

pub mod scrolled {
    pub use rat_scrolled::{
        HScrollPosition, ScrollbarPolicy, Scrolled, ScrolledState, ScrollingState, ScrollingWidget,
        VScrollPosition, View, ViewState, Viewport, ViewportState,
    };
}

pub mod event {
    pub use rat_input::event::{
        crossterm, ct_event, util, ConsumedEvent, FocusKeys, HandleEvent, MouseOnly, Outcome,
        ReadOnly, TextOutcome,
    };
}

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
