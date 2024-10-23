//! Contains
//!
//!
//!
//!

mod clipper;
mod clipper_layout;
mod dual_pager;
mod pager_layout;
mod pager_style;
mod single_pager;
mod view;

pub use clipper::*;
pub use clipper_layout::*;
pub use dual_pager::*;
pub use pager_layout::*;
pub use pager_style::*;
pub use single_pager::*;
pub use view::*;

/// Handle for an area.
/// Can be used to get a stored area.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AreaHandle(usize);

pub(crate) mod event {
    use rat_event::{ConsumedEvent, Outcome};

    /// Result of event handling.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum PagerOutcome {
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
        /// Displayed page changed.
        Page(usize),
    }

    impl ConsumedEvent for PagerOutcome {
        fn is_consumed(&self) -> bool {
            *self != PagerOutcome::Continue
        }
    }

    // Useful for converting most navigation/edit results.
    impl From<bool> for PagerOutcome {
        fn from(value: bool) -> Self {
            if value {
                PagerOutcome::Changed
            } else {
                PagerOutcome::Unchanged
            }
        }
    }

    impl From<Outcome> for PagerOutcome {
        fn from(value: Outcome) -> Self {
            match value {
                Outcome::Continue => PagerOutcome::Continue,
                Outcome::Unchanged => PagerOutcome::Unchanged,
                Outcome::Changed => PagerOutcome::Changed,
            }
        }
    }

    impl From<PagerOutcome> for Outcome {
        fn from(value: PagerOutcome) -> Self {
            match value {
                PagerOutcome::Continue => Outcome::Continue,
                PagerOutcome::Unchanged => Outcome::Unchanged,
                PagerOutcome::Changed => Outcome::Changed,
                PagerOutcome::Page(_) => Outcome::Changed,
            }
        }
    }
}
