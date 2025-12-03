use rat_event::{ConsumedEvent, Outcome};

/// Result of event handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TabbedOutcome {
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
    /// Tab selection changed.
    Select(usize),
    /// Selected tab should be closed.
    Close(usize),
}

impl ConsumedEvent for TabbedOutcome {
    fn is_consumed(&self) -> bool {
        *self != TabbedOutcome::Continue
    }
}

// Useful for converting most navigation/edit results.
impl From<bool> for TabbedOutcome {
    fn from(value: bool) -> Self {
        if value {
            TabbedOutcome::Changed
        } else {
            TabbedOutcome::Unchanged
        }
    }
}

impl From<Outcome> for TabbedOutcome {
    fn from(value: Outcome) -> Self {
        match value {
            Outcome::Continue => TabbedOutcome::Continue,
            Outcome::Unchanged => TabbedOutcome::Unchanged,
            Outcome::Changed => TabbedOutcome::Changed,
        }
    }
}

impl From<TabbedOutcome> for Outcome {
    fn from(value: TabbedOutcome) -> Self {
        match value {
            TabbedOutcome::Continue => Outcome::Continue,
            TabbedOutcome::Unchanged => Outcome::Unchanged,
            TabbedOutcome::Changed => Outcome::Changed,
            TabbedOutcome::Select(_) => Outcome::Changed,
            TabbedOutcome::Close(_) => Outcome::Changed,
        }
    }
}
