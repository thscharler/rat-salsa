//!
//! rat-dialog contains two structs that manage TUI windows.
//!
//! * [DialogStack]
//!
//! A pure stack of modal windows.
//!
//! * [WindowList]
//!
//! A list of modeless windows.
//!
//! There are also some window-decoration widgets that can handle
//! the moving/resizing part. See [decorations](crate::decorations)
//!

#![allow(clippy::question_mark)]
#![allow(clippy::type_complexity)]

use rat_event::{ConsumedEvent, Outcome};

pub mod decorations;

mod dialog_stack;
mod window_list;

pub use dialog_stack::DialogStack;
pub use dialog_stack::handle_dialog_stack;
pub use window_list::Window;
pub use window_list::WindowFrameOutcome;
pub use window_list::WindowList;
pub use window_list::handle_window_list;

/// Result of event-handling for [DialogStack] and [WindowList].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[must_use]
pub enum WindowControl<Event> {
    /// Continue with event-handling.
    /// In the event-loop this waits for the next event.
    Continue,
    /// Break event-handling without repaint.
    /// In the event-loop this waits for the next event.
    Unchanged,
    /// Break event-handling and repaints/renders the application.
    /// In the event-loop this calls `render`.
    Changed,
    /// Return back some application event.
    Event(Event),
    /// Close the dialog
    Close(Event),
}

// Replace WindowControl with this

// pub trait WindowClose {
//     fn is_close(&self) -> bool;
// }

impl<Event> ConsumedEvent for WindowControl<Event> {
    fn is_consumed(&self) -> bool {
        !matches!(self, WindowControl::Continue)
    }
}

impl<Event, T: Into<Outcome>> From<T> for WindowControl<Event> {
    fn from(value: T) -> Self {
        let r = value.into();
        match r {
            Outcome::Continue => WindowControl::Continue,
            Outcome::Unchanged => WindowControl::Unchanged,
            Outcome::Changed => WindowControl::Changed,
        }
    }
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
