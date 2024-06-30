#![doc = include_str!("../readme.md")]

mod inner;
mod scrolled;
mod util;
pub mod view;
pub mod viewport;

pub use scrolled::{
    layout_scroll, layout_scroll_inner, Scroll, ScrollArea, ScrollState, ScrollbarPolicy,
    ScrolledStyle,
};

pub mod event {
    use rat_event::{ConsumedEvent, Outcome};

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
        /// Scroll delta when using HandleEvent for area scrolling.
        /// (∆col, ∆row)
        Delta(isize, isize),
        /// Offset should change to this value.
        Offset(usize),
    }

    impl ConsumedEvent for ScrollOutcome {
        fn is_consumed(&self) -> bool {
            !matches!(self, ScrollOutcome::NotUsed)
        }
    }

    impl From<ScrollOutcome> for Outcome {
        fn from(value: ScrollOutcome) -> Self {
            match value {
                ScrollOutcome::NotUsed => Outcome::NotUsed,
                ScrollOutcome::Unchanged => Outcome::Unchanged,
                ScrollOutcome::Changed => Outcome::Changed,
                ScrollOutcome::Offset(_) => Outcome::Changed,
                ScrollOutcome::Delta(_, _) => Outcome::Changed,
            }
        }
    }
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
