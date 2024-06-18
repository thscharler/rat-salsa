#![doc = include_str!("../readme.md")]
#![allow(clippy::collapsible_else_if)]

pub mod button;
pub mod calender;
pub mod date_input;
pub mod edit_table;
pub mod input;
pub mod list;
pub mod masked_input;
pub mod menuline;
pub mod number_input;
pub mod table;
pub mod textarea;

pub use pure_rust_locales::Locale;

/// Module for focus-handling functionality.
/// For details see [rat-focus](https://docs.rs/rat-focus)
pub mod focus {
    pub use rat_focus::{
        match_focus, on_gained, on_lost, Focus, FocusFlag, HasFocus, HasFocusFlag, ZRect,
    };
}

pub mod scrolled {
    pub use rat_scrolled::{
        HScrollPosition, Inner, ScrollbarPolicy, Scrolled, ScrolledState, ScrolledStyle,
        ScrollingState, ScrollingWidget, VScrollPosition, View, ViewState, Viewport, ViewportState,
    };
}

pub mod event {
    pub use rat_ftable::event::{DoubleClick, DoubleClickOutcome, EditKeys, EditOutcome};
    pub use rat_input::event::{
        crossterm, ct_event, flow, flow_ok, util, ConsumedEvent, FocusKeys, HandleEvent, MouseOnly,
        Outcome, ReadOnly, TextOutcome,
    };
    pub use rat_scrolled::event::ScrollOutcome;
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

pub mod fill {
    pub use rat_input::fill::Fill;
}

mod _private {
    // todo: remvoe
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
