#![doc = include_str!("../readme.md")]
#![allow(clippy::collapsible_else_if)]

pub mod edit_table;
pub mod list;
pub mod menubar;
pub mod number_input;
pub mod popup_menu;
pub mod table;
pub mod textarea;
mod util;

pub use pure_rust_locales::Locale;

/// Event-handling traits and types.
pub mod event {
    pub use rat_ftable::event::{DoubleClick, DoubleClickOutcome, EditKeys, EditOutcome};
    pub use rat_input::event::{
        crossterm, ct_event, flow, flow_ok, util, ConsumedEvent, Dialog, FocusKeys, HandleEvent,
        MouseOnly, Outcome, Popup, ReadOnly, TextOutcome,
    };
    pub use rat_scrolled::event::ScrollOutcome;
}

/// Module for focus-handling functionality.
/// For details see [rat-focus](https://docs.rs/rat-focus)
pub mod focus {
    pub use rat_focus::{
        match_focus, on_gained, on_lost, Focus, FocusFlag, HasFocus, HasFocusFlag, ZRect,
    };
}

/// Layout calculation.
pub mod layout {
    pub use rat_input::layout::{
        layout_dialog, layout_edit, layout_grid, layout_middle, EditConstraint, LayoutDialog,
        LayoutEdit, LayoutEditIterator,
    };
}

// --- widget modules here --- (alphabetical)

/// Button widget.
pub mod button {
    pub use rat_input::button::{Button, ButtonOutcome, ButtonState, ButtonStyle};
}

/// Calendar month widget.
pub mod calendar {
    pub use rat_input::calendar::{Month, MonthState, MonthStyle};
}

/// Date input using chrono.
pub mod date_input {
    pub use rat_input::date_input::{ConvenientKeys, DateInput, DateInputState};
}

/// Fill an area with a Style and a symbol.
pub mod fill {
    pub use rat_input::fill::Fill;
}

/// TextInput
pub mod input {
    pub use rat_input::input::{core, TextInput, TextInputState, TextInputStyle};
}

/// Textinput with an input mask
pub mod masked_input {
    pub use rat_input::masked_input::{core, MaskedInput, MaskedInputState, MaskedInputStyle};
}

/// Menu as a single Text-line.
pub mod menuline {
    pub use rat_input::menuline::{MenuLine, MenuLineState, MenuOutcome, MenuStyle};
}

/// Basic message dialog.
pub mod msgdialog {
    pub use rat_input::msgdialog::{MsgDialog, MsgDialogState, MsgDialogStyle};
}

/// Scrolled widget and viewports.
pub mod scrolled {
    pub use rat_scrolled::{
        HScrollPosition, Inner, ScrollbarPolicy, Scrolled, ScrolledState, ScrolledStyle,
        ScrollingState, ScrollingWidget, VScrollPosition, View, ViewState, Viewport, ViewportState,
    };
}

/// Statusbar.
pub mod statusline {
    pub use rat_input::statusline::{StatusLine, StatusLineState};
}

mod _private {
    // todo: remvoe
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
