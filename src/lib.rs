//! Current status: BETA
//!
#![doc = include_str!("../readme.md")]
//
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

    pub use rat_event::*;

    pub use crate::calendar::event::CalOutcome;
    pub use crate::file_dialog::event::FileOutcome;
    pub use crate::tabbed::event::TabbedOutcome;
    pub use rat_ftable::event::{DoubleClickOutcome, EditOutcome};
    pub use rat_menu::event::MenuOutcome;
    pub use rat_scrolled::event::ScrollOutcome;
    pub use rat_text::event::{ReadOnly, TextOutcome};
}

/// Module for focus-handling functionality.
/// For details see [rat-focus](https://docs.rs/rat-focus)
pub mod focus {
    pub use rat_focus::{
        build_focus, handle_focus, match_focus, on_gained, on_lost, rebuild_focus, ContainerFlag,
        Focus, FocusBuilder, FocusFlag, HasFocus, HasFocusFlag, Navigation, ZRect,
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
        Scroll, ScrollArea, ScrollAreaState, ScrollState, ScrollStyle, ScrollbarPolicy,
    };
}

/// Text editing core functionality and utilities.
pub mod text {
    pub use rat_text::clipboard;
    pub use rat_text::core;
    pub use rat_text::undo_buffer;
    pub use rat_text::{
        ipos_type, upos_type, Cursor, Glyph, Grapheme, HasScreenCursor, TextError, TextPosition,
        TextRange,
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
pub mod list;
pub mod line_number {
    pub use rat_text::line_number::{LineNumberState, LineNumberStyle, LineNumbers};
}
pub mod menu {
    pub use rat_menu::menubar::{MenuBarState, Menubar, MenubarLine, MenubarPopup};
    pub use rat_menu::menuline::{MenuLine, MenuLineState};
    pub use rat_menu::popup_menu::{Placement, PopupMenu, PopupMenuState};
    pub use rat_menu::{MenuItem, MenuStructure, MenuStyle, Separator, StaticMenu};

    pub mod menubar {
        pub use rat_menu::menubar::{handle_events, handle_mouse_events, handle_popup_events};
    }
    pub mod menuline {
        pub use rat_menu::menuline::{handle_events, handle_mouse_events};
    }
    pub mod popup_menu {
        pub use rat_menu::popup_menu::{handle_mouse_events, handle_popup_events};
    }
}
pub mod msgdialog;
pub mod number_input {
    pub use rat_text::number_input::{
        handle_events, handle_mouse_events, handle_readonly_events, NumberInput, NumberInputState,
    };
}
pub mod paragraph;
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
