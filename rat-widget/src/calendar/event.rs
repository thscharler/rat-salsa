use rat_event::{ConsumedEvent, Outcome};

/// Result of event handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CalOutcome {
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
    /// The selection has changed.
    Selected,
}

impl ConsumedEvent for CalOutcome {
    fn is_consumed(&self) -> bool {
        *self != CalOutcome::Continue
    }
}

impl From<Outcome> for CalOutcome {
    fn from(value: Outcome) -> Self {
        match value {
            Outcome::Continue => CalOutcome::Continue,
            Outcome::Unchanged => CalOutcome::Unchanged,
            Outcome::Changed => CalOutcome::Changed,
        }
    }
}

impl From<CalOutcome> for Outcome {
    fn from(value: CalOutcome) -> Self {
        match value {
            CalOutcome::Continue => Outcome::Continue,
            CalOutcome::Unchanged => Outcome::Unchanged,
            CalOutcome::Changed => Outcome::Changed,
            CalOutcome::Selected => Outcome::Changed,
        }
    }
}
