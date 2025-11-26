#![doc = include_str!("../readme.md")]
//
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::assigning_clones)]
#![allow(clippy::question_mark)]
#![allow(clippy::uninlined_format_args)]

pub mod event {
    //!
    //! Event-handler traits and Keybindings.
    //!
    //! See [rat-event](https://docs.rs/rat-event/latest/rat_event/)
    //!
    pub use rat_event::*;

    pub use crate::button::event::ButtonOutcome;
    pub use crate::calendar::event::CalOutcome;
    pub use crate::checkbox::event::CheckOutcome;
    pub use crate::choice::event::ChoiceOutcome;
    pub use crate::combobox::event::ComboboxOutcome;
    pub use crate::file_dialog::event::FileOutcome;
    pub use crate::form::event::FormOutcome;
    pub use crate::radio::event::RadioOutcome;
    pub use crate::slider::event::SliderOutcome;
    pub use crate::tabbed::event::TabbedOutcome;
    pub use rat_ftable::event::{DoubleClickOutcome, EditOutcome, TableOutcome};
    pub use rat_menu::event::MenuOutcome;
    pub use rat_popup::event::PopupOutcome;
    pub use rat_scrolled::event::ScrollOutcome;
    pub use rat_text::event::{ReadOnly, TextOutcome};
}

/// Module for focus-handling functionality.
/// See [rat-focus](https://docs.rs/rat-focus)
pub mod focus {
    pub use rat_focus::{
        Focus, FocusBuilder, FocusFlag, HasFocus, Navigation, handle_focus, impl_has_focus,
        match_focus, on_gained, on_lost,
    };
}

/// Layout calculations apart from ratatui/Layout.
pub mod layout;

/// Trait for relocatable widgets.
/// See also [rat-reloc](https://docs.rs/rat-reloc/latest/rat_reloc/)
pub mod reloc {
    pub use rat_reloc::{
        RelocatableState, impl_relocatable_state, relocate_area, relocate_areas, relocate_position,
        relocate_positions,
    };
}

/// Scroll attribute and event-handling.
/// See [rat-scrolled](https://docs.rs/rat-scrolled/latest/rat_scrolled/)
pub mod scrolled {
    pub use rat_scrolled::{
        SCROLLBAR_DOUBLE_HORIZONTAL, SCROLLBAR_DOUBLE_VERTICAL, SCROLLBAR_HORIZONTAL,
        SCROLLBAR_VERTICAL, Scroll, ScrollArea, ScrollAreaState, ScrollState, ScrollStyle,
        ScrollSymbols, ScrollbarPolicy,
    };
}

/// Text editing core functionality and utilities.
pub mod text {
    pub use rat_text::clipboard;
    pub use rat_text::core;
    pub use rat_text::undo_buffer;
    pub use rat_text::{
        Cursor, Grapheme, HasScreenCursor, Locale, TextError, TextFocusGained, TextFocusLost,
        TextPosition, TextRange, TextStyle, impl_screen_cursor, ipos_type, screen_cursor,
        upos_type,
    };
}

// --- widget modules here --- (alphabetical)

pub mod button;
pub mod calendar;
pub mod checkbox;
pub mod choice;
pub mod clipper;
#[cfg(feature = "color_input")]
pub mod color_input;
pub mod combobox;
pub mod date_input;
pub mod dialog_frame;
pub mod file_dialog;
pub mod form;
pub mod hover;
pub mod line_number;
pub mod list;
pub mod menu;
pub mod msgdialog;
pub mod number_input;
pub mod paired;
pub mod paragraph;
pub mod popup;
pub mod radio;
pub mod range_op;
pub mod shadow;
pub mod slider;
pub mod splitter;
pub mod statusline;
pub mod statusline_stacked;
pub mod tabbed;
pub mod table;
pub mod text_input;
pub mod text_input_mask;
pub mod textarea;
pub mod util;
pub mod view;

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
