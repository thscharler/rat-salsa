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
        /// Popup has been hidden after a focus change was detected.
        ///
        HiddenFocus,
        /// Popup has been hidden.
        /// There is the need to transfer the focus elsewhere.
        Hidden,
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
                PopupOutcome::Hidden => Outcome::Changed,
                PopupOutcome::HiddenFocus => Outcome::Changed,
            }
        }
    }
}

/// Placement relative a target rect.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Placement {
    /// Use the render-area for the popup.
    #[default]
    None,
    /// Place the popup above the given area.
    /// Set an extra offset for the x-position.
    AboveLeft(Rect),
    /// Place the popup above the given area.
    /// Set an extra offset for the x-position.
    AboveCenter(Rect),
    /// Place the popup above the given area.
    /// Set an extra offset for the x-position.
    AboveRight(Rect),
    /// Place the popup to the left of the given area.
    /// Set an extra offset for the y-position.
    LeftTop(Rect),
    /// Place the popup to the left of the given area.
    /// Set an extra offset for the y-position.
    LeftMiddle(Rect),
    /// Place the popup to the left of the given area.
    /// Set an extra offset for the y-position.
    LeftBottom(Rect),
    /// Place the popup to the right of the given area.
    /// Set an extra offset for the y-position.
    RightTop(Rect),
    /// Place the popup to the right of the given area.
    /// Set an extra offset for the y-position.
    RightMiddle(Rect),
    /// Place the popup to the right of the given area.
    /// Set an extra offset for the y-position.
    RightBottom(Rect),
    /// Place the popup below the given area.
    /// Set an extra offset for the x-position.
    BelowLeft(Rect),
    /// Place the popup below the given area.
    /// Set an extra offset for the x-position.
    BelowCenter(Rect),
    /// Place the popup below the given area.
    /// Set an extra offset for the x-position.
    BelowRight(Rect),
    /// Use the render-area for the popup.
    /// But for the x,y use this position, shifted.
    Position(u16, u16),
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
