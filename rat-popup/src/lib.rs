#![doc = include_str!("../readme.md")]
//
#![allow(clippy::collapsible_else_if)]

use ratatui_core::layout::{Alignment, Rect};

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

/// Placement of the popup.
///
/// Use this enum for your widgets API.
///
/// Then convert the Placement to a [PopupConstraint] and
/// set the constraint with the PopupCore.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Placement {
    /// Use the render-area for the popup as is.
    #[default]
    None,
    /// Place above the main widget area.
    Above,
    /// Place below the main widget area.
    Below,
    /// Place left of the main widget area.
    Left,
    /// Place right of the main widget area.
    Right,
    /// Above or below the main widget dependent on available space.
    AboveOrBelow,
    /// Below or above the main widget dependent on available space.
    BelowOrAbove,
    /// Place the popup at this position.
    Position(u16, u16),
}

impl Placement {
    /// Convert the placement to a PopupConstraint.
    /// Needs an extra alignment and the area of the main widget.
    ///
    pub fn into_constraint(self, alignment: Alignment, relative_to_area: Rect) -> PopupConstraint {
        match self {
            Placement::None => PopupConstraint::None,
            Placement::Above => PopupConstraint::Above(alignment, relative_to_area),
            Placement::Below => PopupConstraint::Below(alignment, relative_to_area),
            Placement::Left => PopupConstraint::Left(alignment, relative_to_area),
            Placement::Right => PopupConstraint::Right(alignment, relative_to_area),
            Placement::AboveOrBelow => PopupConstraint::AboveOrBelow(alignment, relative_to_area),
            Placement::BelowOrAbove => PopupConstraint::BelowOrAbove(alignment, relative_to_area),
            Placement::Position(x, y) => PopupConstraint::Position(x, y),
        }
    }
}

/// Placement constraint for PopupCore.
///
/// This defines a position relative to the main widget,
/// plus an alignment and the main widget area.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PopupConstraint {
    /// Use the render-area for the popup as is.
    #[default]
    None,
    /// Place above the main widget area.
    Above(Alignment, Rect),
    /// Place below the main widget area.
    Below(Alignment, Rect),
    /// Place left of the main widget area.
    /// Alignment::Left == Top, Alignment::Center == Middle, Alignment::Right == Bottom
    Left(Alignment, Rect),
    /// Place right of the main widget area.
    /// Alignment::Left == Top, Alignment::Center == Middle, Alignment::Right == Bottom
    Right(Alignment, Rect),
    /// Above or below the main widget dependent on available space.
    AboveOrBelow(Alignment, Rect),
    /// Below or above the main widget dependent on available space.
    BelowOrAbove(Alignment, Rect),
    /// Place the popup at this position.
    Position(u16, u16),
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
