#![allow(clippy::question_mark)]
#![allow(clippy::type_complexity)]

use rat_event::{ConsumedEvent, Outcome};

mod dialog_control;
mod window_control;

pub use dialog_control::DialogStack;
pub use dialog_control::handle_dialog_stack;
pub use window_control::Window;
pub use window_control::WindowList;
pub use window_control::handle_window_list;
pub use window_control::window_frame::WindowFrame;
pub use window_control::window_frame::WindowFrameOutcome;
pub use window_control::window_frame::WindowFrameState;
pub use window_control::window_frame::WindowFrameStyle;

/// Extends rat-salsa::Control with some dialog specific options.
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
