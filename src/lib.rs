use crate::event::RepaintEvent;
use rat_widget::button::ButtonOutcome;
use rat_widget::event::{
    ConsumedEvent, DoubleClickOutcome, EditOutcome, Outcome, ScrollOutcome, TextOutcome,
};
use rat_widget::menuline::MenuOutcome;

mod framework;
mod timer;

pub use framework::{run_tui, AppContext, AppEvents, AppWidget, RenderContext, RunConfig};
pub use timer::{TimeOut, TimerDef, TimerEvent, TimerHandle};

/// Result type for most event-handling functions.
///
/// This controls the main event loop and the running of the
/// event-handling functions.
///
/// For the event-handling functions there is the [flow!] macro,
/// that is used to encapsulate calls to further event-handling
/// functions. It returns early if the result is anything but Continue.
///
/// In the event-loop Repaint and Action call out to the corresponding
/// event-handler functions. Continue and Break both wait for
/// new incoming events. Quit quits the application.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[must_use]
pub enum Control<Action> {
    /// Continue handling the current event.
    /// In the event-loop this goes on waiting for a new event.
    Continue,
    /// Break handling the current event.
    /// In the event-loop this does nothing and just waits for a new event.
    Break,
    /// Triggers a repaint in the event-loop.
    Repaint,
    /// The event-loop calls out the action-handlers to take care of it.
    Action(Action),
    /// Quit the application.
    Quit,
}

impl<Action> ConsumedEvent for Control<Action> {
    fn is_consumed(&self) -> bool {
        !matches!(self, Control::Continue)
    }
}

impl<Action> From<Outcome> for Control<Action> {
    fn from(value: Outcome) -> Self {
        match value {
            Outcome::NotUsed => Control::Continue,
            Outcome::Unchanged => Control::Break,
            Outcome::Changed => Control::Repaint,
        }
    }
}

impl<Action> From<MenuOutcome> for Control<Action> {
    fn from(value: MenuOutcome) -> Self {
        match value {
            MenuOutcome::NotUsed => Control::Continue,
            MenuOutcome::Unchanged => Control::Break,
            MenuOutcome::Changed => Control::Repaint,
            MenuOutcome::Selected(_) => Control::Repaint,
            MenuOutcome::Activated(_) => Control::Repaint,
        }
    }
}

impl<Action> From<ButtonOutcome> for Control<Action> {
    fn from(value: ButtonOutcome) -> Self {
        match value {
            ButtonOutcome::NotUsed => Control::Continue,
            ButtonOutcome::Unchanged => Control::Break,
            ButtonOutcome::Changed => Control::Repaint,
            ButtonOutcome::Pressed => Control::Repaint,
        }
    }
}

impl<Action> From<TextOutcome> for Control<Action> {
    fn from(value: TextOutcome) -> Self {
        match value {
            TextOutcome::NotUsed => Control::Continue,
            TextOutcome::Unchanged => Control::Break,
            TextOutcome::Changed => Control::Repaint,
            TextOutcome::TextChanged => Control::Repaint,
        }
    }
}

impl<Action, R> From<ScrollOutcome<R>> for Control<Action>
where
    R: Into<Control<Action>>,
{
    fn from(value: ScrollOutcome<R>) -> Self {
        match value {
            ScrollOutcome::NotUsed => Control::Continue,
            ScrollOutcome::Unchanged => Control::Break,
            ScrollOutcome::Changed => Control::Repaint,
            ScrollOutcome::Inner(v) => v.into(),
        }
    }
}

impl<Action> From<DoubleClickOutcome> for Control<Action> {
    fn from(value: DoubleClickOutcome) -> Self {
        match value {
            DoubleClickOutcome::NotUsed => Control::Continue,
            DoubleClickOutcome::Unchanged => Control::Break,
            DoubleClickOutcome::Changed => Control::Repaint,
            DoubleClickOutcome::ClickClick(_, _) => Control::Break,
        }
    }
}

impl<Action> From<EditOutcome> for Control<Action> {
    fn from(value: EditOutcome) -> Self {
        match value {
            EditOutcome::NotUsed => Control::Continue,
            EditOutcome::Unchanged => Control::Break,
            EditOutcome::Changed => Control::Repaint,
            EditOutcome::Insert => Control::Break,
            EditOutcome::Remove => Control::Break,
            EditOutcome::Edit => Control::Break,
            EditOutcome::Append => Control::Break,
            EditOutcome::Cancel => Control::Break,
            EditOutcome::Commit => Control::Break,
            EditOutcome::CommitAndAppend => Control::Break,
            EditOutcome::CommitAndEdit => Control::Break,
        }
    }
}

pub mod event {
    //!
    //! Event-handler traits and Keybindings.
    //!
    use crate::TimeOut;

    /// Gives some extra information why a repaint was triggered.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum RepaintEvent {
        /// There was a [Repaint](crate::Control::Repaint) or the change flag has been set.
        Repaint,
        /// A timer triggered this.
        Timer(TimeOut),
    }
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
