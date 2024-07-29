#![doc = include_str!("../readme.md")]
#![allow(clippy::collapsible_else_if)]

mod scroll;

pub use scroll::{layout_scroll, Scroll, ScrollArea, ScrollState, ScrollStyle, ScrollbarType};

pub mod event {
    use rat_event::{ConsumedEvent, Outcome};

    /// Result of event-handling for a scroll.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum ScrollOutcome {
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
        /// Scroll delta.
        Up(usize),
        /// Scroll delta.
        Down(usize),
        /// Scroll delta.
        Left(usize),
        /// Scroll delta.
        Right(usize),
        /// Absolute position.
        VPos(usize),
        /// Absolute position.
        HPos(usize),
    }

    impl ConsumedEvent for ScrollOutcome {
        fn is_consumed(&self) -> bool {
            !matches!(self, ScrollOutcome::NotUsed)
        }
    }

    impl From<ScrollOutcome> for Outcome {
        fn from(value: ScrollOutcome) -> Self {
            match value {
                ScrollOutcome::NotUsed => Outcome::Continue,
                ScrollOutcome::Unchanged => Outcome::Unchanged,
                ScrollOutcome::Changed => Outcome::Changed,
                ScrollOutcome::Up(_) => Outcome::Changed,
                ScrollOutcome::Down(_) => Outcome::Changed,
                ScrollOutcome::Left(_) => Outcome::Changed,
                ScrollOutcome::Right(_) => Outcome::Changed,
                ScrollOutcome::VPos(_) => Outcome::Changed,
                ScrollOutcome::HPos(_) => Outcome::Changed,
            }
        }
    }
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
