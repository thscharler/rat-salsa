//! Eventhandling.

pub use rat_event::*;

/// Result value for event-handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TableOutcome {
    /// The given event was not handled at all.
    Continue,
    /// The event was handled, no repaint necessary.
    Unchanged,
    /// Header column clicked.
    Header(usize),
    /// The event was handled, repaint necessary.
    Changed,
    /// The selection has changed.
    Selected,
}

impl ConsumedEvent for TableOutcome {
    fn is_consumed(&self) -> bool {
        *self != TableOutcome::Continue
    }
}

impl From<TableOutcome> for Outcome {
    fn from(value: TableOutcome) -> Self {
        match value {
            TableOutcome::Continue => Outcome::Continue,
            TableOutcome::Unchanged => Outcome::Unchanged,
            TableOutcome::Changed => Outcome::Changed,
            TableOutcome::Selected => Outcome::Changed,
            TableOutcome::Header(_) => Outcome::Unchanged,
        }
    }
}

/// Result type for double-click event-handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DoubleClickOutcome {
    /// The given event has not been used at all.
    Continue,
    /// The event has been recognized, but the result was nil.
    /// Further processing for this event may stop.
    Unchanged,
    /// The event has been recognized and there is some change
    /// due to it.
    /// Further processing for this event may stop.
    /// Rendering the ui is advised.
    Changed,
    /// Double click occurred. Contains (column, row)
    ClickClick(usize, usize),
}

impl From<DoubleClickOutcome> for Outcome {
    fn from(value: DoubleClickOutcome) -> Self {
        match value {
            DoubleClickOutcome::Continue => Outcome::Continue,
            DoubleClickOutcome::Unchanged => Outcome::Unchanged,
            DoubleClickOutcome::Changed => Outcome::Changed,
            DoubleClickOutcome::ClickClick(_, _) => Outcome::Changed,
        }
    }
}

impl From<Outcome> for DoubleClickOutcome {
    fn from(value: Outcome) -> Self {
        match value {
            Outcome::Continue => DoubleClickOutcome::Continue,
            Outcome::Unchanged => DoubleClickOutcome::Unchanged,
            Outcome::Changed => DoubleClickOutcome::Changed,
        }
    }
}

impl ConsumedEvent for DoubleClickOutcome {
    fn is_consumed(&self) -> bool {
        !matches!(self, DoubleClickOutcome::Continue)
    }
}
