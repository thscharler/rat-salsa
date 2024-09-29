#![doc = include_str!("../readme.md")]
#![allow(clippy::collapsible_else_if)]

mod scroll;
mod scroll_area;

pub use scroll::{Scroll, ScrollState, ScrollStyle};
pub use scroll_area::{ScrollArea, ScrollAreaState};

pub mod event {
    use rat_event::*;

    /// Result of event-handling for a scroll.
    ///
    /// All values are in terms of offset.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum ScrollOutcome {
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
            !matches!(self, ScrollOutcome::Continue)
        }
    }

    impl From<ScrollOutcome> for Outcome {
        fn from(value: ScrollOutcome) -> Self {
            match value {
                ScrollOutcome::Continue => Outcome::Continue,
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

///
/// Behaviour of the scrollbar.
///
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarPolicy {
    /// Always renders the scrollbar recognizable as scrollbar.
    Always,

    /// If the scrollbar is not needed, it will be rendered in
    /// a 'minimized' style.
    ///
    /// If a `min_symbol` is set, the area for the scrollbar will
    /// be filled with the symbol.
    /// If a `min_style`is set, the area for the scrollbar will
    /// be set to this style. If no min_symbol is set, this will
    /// just set the style.
    ///
    /// > The scrollbar is not needed, if `max_offset == 0`.
    #[default]
    Minimize,

    /// If the scrollbar is not needed, no area is reserved for it.
    /// The widget will get the extra area.
    ///
    /// If the scrollbar is rendered combined with a block,
    /// the block still might reserve the same space for itself.
    Collapse,
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
