#![doc = include_str!("../readme.md")]
//
#![allow(clippy::collapsible_else_if)]

use ratatui::layout::{Alignment, Rect};

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
/// This enum is for use in a widget that then uses PopupCore
/// internally. Expose Placement to the users of your widget
/// to let them define a popup placement. Convert the Placement
/// to a PopupConstraint internally when forwarding this
/// to PopupCore.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Placement {
    /// Use the render-area for the popup as is.
    #[default]
    None,
    /// Place above the given area.
    Above,
    /// Place below the given area:
    Below,
    /// Place left of the given area.
    Left,
    /// Place right of the given area.
    Right,
    /// Above or below dependent on available space. Aligned left.
    AboveOrBelow,
    /// Below or above dependent on available space. Aligned left.
    BelowOrAbove,
    /// Use the render-area for the popup, but place it at position (x,y).
    Position(u16, u16),
}

impl Placement {
    pub fn into_constraint(self, alignment: Alignment, rel_area: Rect) -> PopupConstraint {
        match self {
            Placement::None => PopupConstraint::None,
            Placement::Above => PopupConstraint::Above(alignment, rel_area),
            Placement::Below => PopupConstraint::Below(alignment, rel_area),
            Placement::Left => PopupConstraint::Left(alignment, rel_area),
            Placement::Right => PopupConstraint::Right(alignment, rel_area),
            Placement::AboveOrBelow => PopupConstraint::AboveOrBelow(alignment, rel_area),
            Placement::BelowOrAbove => PopupConstraint::BelowOrAbove(alignment, rel_area),
            Placement::Position(x, y) => PopupConstraint::Position(x, y),
        }
    }
}

/// Placement relative to the main-widget area + the main-widget area.
///
/// The render() call for PopupCore will use the size of
/// the area given to render() as the size of the popup and
/// ignore the position.
///
/// It will calculate the position of the popup using these
/// constraints.
///
/// If you build a widget that uses a PopupCore internally you
/// will rather use Placement as a parameter for your widget.
/// You can construct the PopupConstraint when rendering
/// your widget and set it in PopupCore.
///
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PopupConstraint {
    /// Use the render-area for the popup as is.
    #[default]
    None,
    /// Synonym for AboveLeft
    Above(Alignment, Rect),
    /// Synonym for BelowLeft
    Below(Alignment, Rect),
    /// Synonym for LeftTop
    Left(Alignment, Rect),
    /// Synonym for RightTop
    Right(Alignment, Rect),
    /// Above or below dependent on available space. Aligned left.
    AboveOrBelow(Alignment, Rect),
    /// Below or above dependent on available space. Aligned left.
    BelowOrAbove(Alignment, Rect),
    /// Use the render-area for the popup, but place it at position (x,y).
    Position(u16, u16),
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
