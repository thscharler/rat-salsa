//! Current status: BETA
//!
#![doc = include_str!("../readme.md")]
//
#![allow(clippy::collapsible_else_if)]

use ratatui::layout::Rect;

mod popup;

pub use popup::*;

pub mod event {
    use rat_event::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum PopupOutcome {
        /// The given event has not been used at all.
        Continue,
        /// The event has been recognized, but nothing noticeable has changed.
        /// Further processing for this event may stop.
        /// Rendering the ui is not necessary.
        Unchanged,
        /// The event has been recognized and there is some change due to it.
        /// Further processing for this event may stop.
        /// Rendering the ui is advised.
        Changed,
        /// Popup should be hidden.
        Hide,
    }

    impl ConsumedEvent for PopupOutcome {
        fn is_consumed(&self) -> bool {
            *self != PopupOutcome::Continue
        }
    }

    impl From<PopupOutcome> for Outcome {
        fn from(value: PopupOutcome) -> Self {
            match value {
                PopupOutcome::Continue => Outcome::Continue,
                PopupOutcome::Unchanged => Outcome::Unchanged,
                PopupOutcome::Changed => Outcome::Changed,
                PopupOutcome::Hide => Outcome::Changed,
            }
        }
    }
}

/// Placement relative a target rect.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Placement {
    /// Use the render-area for the popup as is.
    #[default]
    None,
    /// Place the popup above the given area. Aligned left.
    AboveLeft(Rect),
    /// Place the popup above the given area. Aligned centered.
    AboveCenter(Rect),
    /// Place the popup above the given area. Aligned right.
    AboveRight(Rect),
    /// Place the popup to the left of the given area. Aligned to the top.
    LeftTop(Rect),
    /// Place the popup to the left of the given area. Aligned in the middle.
    LeftMiddle(Rect),
    /// Place the popup to the left of the given area. Aligned to the bottom.
    LeftBottom(Rect),
    /// Place the popup to the right of the given area. Aligned to the top.
    RightTop(Rect),
    /// Place the popup to the right of the given area. Aligned in the middle.
    RightMiddle(Rect),
    /// Place the popup to the right of the given area. Aligned to the bottom.
    RightBottom(Rect),
    /// Place the popup below the given area. Aligned left.
    BelowLeft(Rect),
    /// Place the popup below the given area. Aligned centered.
    BelowCenter(Rect),
    /// Place the popup below the given area. Aligned right.
    BelowRight(Rect),
    /// Use the render-area for the popup, but place it at position (x,y).
    Position(u16, u16),
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
