#![doc = include_str!("../readme.md")]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::assigning_clones)]

pub use pure_rust_locales::Locale;

pub mod event {
    //!
    //! Event-handler traits and Keybindings.
    //!

    pub use rat_event::{
        crossterm, ct_event, flow, try_flow, util, ConsumedEvent, Dialog, DoubleClick, HandleEvent,
        MouseOnly, Outcome, Popup, Regular,
    };

    pub use crate::file_dialog::event::FileOutcome;
    pub use crate::tabbed::event::TabbedOutcome;
    pub use rat_ftable::event::{DoubleClickOutcome, EditKeys, EditOutcome};
    pub use rat_scrolled::event::ScrollOutcome;
    pub use rat_text::event::{ReadOnly, TextOutcome};
}

/// Module for focus-handling functionality.
/// For details see [rat-focus](https://docs.rs/rat-focus)
pub mod focus {
    pub use rat_focus::{
        match_focus, on_gained, on_lost, ContainerFlag, Focus, FocusFlag, HasFocus, HasFocusFlag,
        Navigation, ZRect,
    };
}

/// Some functions that calculate more complicate layouts.
pub mod layout {
    mod layout_dialog;
    mod layout_edit;
    mod layout_grid;

    pub use layout_dialog::{layout_dialog, LayoutDialog};
    pub use layout_edit::{layout_edit, EditConstraint, LayoutEdit, LayoutEditIterator};
    pub use layout_grid::{layout_grid, layout_middle};
}

/// Scroll attribute and event-handling.
pub mod scrolled {
    pub use rat_scrolled::{
        layout_scroll, Scroll, ScrollArea, ScrollState, ScrollStyle, ScrollbarType,
    };
}

/// Text editing core functionality and utilities.
pub mod text {
    pub use rat_text::clipboard;
    pub use rat_text::core;
    pub use rat_text::undo_buffer;
    pub use rat_text::{
        ipos_type, upos_type, Cursor, Glyph, Grapheme, TextError, TextPosition, TextRange,
    };
}

pub mod util;

// --- widget modules here --- (alphabetical)

pub mod button;
pub mod calendar;
pub mod date_input {
    pub use rat_text::date_input::{
        handle_events, handle_mouse_events, handle_readonly_events, DateInput, DateInputState,
    };
}
pub mod file_dialog;
pub mod fill;
pub mod list;
pub mod line_number {
    pub use rat_text::line_number::{LineNumberState, LineNumberStyle, LineNumbers};
}
pub mod menubar;
pub mod menuline;
pub mod msgdialog;
pub mod number_input {
    pub use rat_text::number_input::{
        handle_events, handle_mouse_events, handle_readonly_events, NumberInput, NumberInputState,
    };
}
pub mod paragraph;
pub mod popup_menu;
pub mod splitter;
pub mod statusline;
/// F-Table
pub mod table {
    pub use rat_ftable::{
        edit, selection, textdata, Table, TableContext, TableData, TableDataIter, TableSelection,
        TableState, TableStyle,
    };
}
pub mod tabbed;
/// Text-Input
pub mod text_input {
    pub use rat_text::text_input::{
        handle_events, handle_mouse_events, handle_readonly_events, TextInput, TextInputState,
        TextInputStyle,
    };
}
/// Text-Input with mask.
pub mod text_input_mask {
    pub use rat_text::text_input_mask::{
        handle_events, handle_mouse_events, handle_readonly_events, MaskedInput, MaskedInputState,
    };
}
/// Text-Area.
pub mod textarea {
    pub use rat_text::text_area::{
        handle_events, handle_mouse_events, handle_readonly_events, TextArea, TextAreaState,
        TextAreaStyle,
    };
}
pub mod view;
pub mod viewport;

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
