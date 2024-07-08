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
        crossterm, ct_event, flow, flow_ok, or_else, util, ConsumedEvent, Dialog, FocusKeys,
        HandleEvent, MouseOnly, Outcome, Popup,
    };
    use std::cmp::max;
    use std::ops::BitOr;
    use std::path::PathBuf;

    /// Runs only the navigation events, not any editing.
    #[derive(Debug)]
    pub struct ReadOnly;

    /// Result of event handling.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum TextOutcome {
        /// The given event has not been used at all.
        NotUsed,
        /// The event has been recognized, but the result was nil.
        /// Further processing for this event may stop.
        Unchanged,
        /// The event has been recognized and there is some change
        /// due to it.
        /// Further processing for this event may stop.
        /// Rendering the ui is advised.
        Changed,
        /// Text content has changed.
        TextChanged,
    }

    impl ConsumedEvent for TextOutcome {
        fn is_consumed(&self) -> bool {
            *self != TextOutcome::NotUsed
        }
    }

    impl BitOr for TextOutcome {
        type Output = TextOutcome;

        fn bitor(self, rhs: Self) -> Self::Output {
            max(self, rhs)
        }
    }

    // Useful for converting most navigation/edit results.
    impl From<bool> for TextOutcome {
        fn from(value: bool) -> Self {
            if value {
                TextOutcome::Changed
            } else {
                TextOutcome::Unchanged
            }
        }
    }

    impl From<TextOutcome> for Outcome {
        fn from(value: TextOutcome) -> Self {
            match value {
                TextOutcome::NotUsed => Outcome::NotUsed,
                TextOutcome::Unchanged => Outcome::Unchanged,
                TextOutcome::Changed => Outcome::Changed,
                TextOutcome::TextChanged => Outcome::Changed,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub enum FileOutcome {
        /// The given event has not been used at all.
        NotUsed,
        /// The event has been recognized, but the result was nil.
        /// Further processing for this event may stop.
        Unchanged,
        /// The event has been recognized and there is some change
        /// due to it.
        /// Further processing for this event may stop.
        /// Rendering the ui is advised.
        Changed,
        /// Cancel
        Cancel,
        /// Ok
        Ok(PathBuf),
    }

    impl From<FileOutcome> for Outcome {
        fn from(value: FileOutcome) -> Self {
            match value {
                FileOutcome::NotUsed => Outcome::NotUsed,
                FileOutcome::Unchanged => Outcome::Unchanged,
                FileOutcome::Changed => Outcome::Changed,
                FileOutcome::Ok(_) => Outcome::Changed,
                FileOutcome::Cancel => Outcome::Changed,
            }
        }
    }

    impl From<Outcome> for FileOutcome {
        fn from(value: Outcome) -> Self {
            match value {
                Outcome::NotUsed => FileOutcome::NotUsed,
                Outcome::Unchanged => FileOutcome::Unchanged,
                Outcome::Changed => FileOutcome::Changed,
            }
        }
    }

    // Useful for converting most navigation/edit results.
    impl From<bool> for FileOutcome {
        fn from(value: bool) -> Self {
            if value {
                FileOutcome::Changed
            } else {
                FileOutcome::Unchanged
            }
        }
    }

    impl ConsumedEvent for FileOutcome {
        fn is_consumed(&self) -> bool {
            !matches!(self, FileOutcome::NotUsed)
        }
    }

    pub use rat_ftable::event::{DoubleClick, DoubleClickOutcome, EditKeys, EditOutcome};
    pub use rat_scrolled::event::ScrollOutcome;
}

/// Module for focus-handling functionality.
/// For details see [rat-focus](https://docs.rs/rat-focus)
pub mod focus {
    pub use rat_focus::{
        match_focus, on_gained, on_lost, Focus, FocusFlag, HasFocus, HasFocusFlag, ZRect,
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

// --- widget modules here --- (alphabetical)

pub mod button;
pub mod calendar;
pub mod date_input;
pub mod edit_list;
pub mod file_dialog;
pub mod fill;
pub mod input;
pub mod list;
pub mod masked_input;
pub mod menubar;
pub mod menuline;
pub mod msgdialog;
pub mod number_input;
pub mod popup_menu;
pub mod statusline;

/// Scrolled widget and viewports.
pub mod scrolled {
    pub use rat_scrolled::{
        layout_scroll, view, viewport, Scroll, ScrollArea, ScrollState, ScrollStyle,
        ScrollbarPolicy,
    };
}

/// F-Table
pub mod table {
    pub use rat_ftable::{
        edit, selection, textdata, FTable, FTableContext, FTableState, FTableStyle, TableData,
        TableDataIter, TableSelection,
    };
}

pub mod textarea;
mod util;

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
