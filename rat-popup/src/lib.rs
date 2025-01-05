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
    /// Place the popup above the given area. Aligned left.
    AboveLeft,
    /// Place the popup above the given area. Aligned centered.
    AboveCenter,
    /// Place the popup above the given area. Aligned right.
    AboveRight,
    /// Place the popup to the left of the given area. Aligned to the top.
    LeftTop,
    /// Place the popup to the left of the given area. Aligned in the middle.
    LeftMiddle,
    /// Place the popup to the left of the given area. Aligned to the bottom.
    LeftBottom,
    /// Place the popup to the right of the given area. Aligned to the top.
    RightTop,
    /// Place the popup to the right of the given area. Aligned in the middle.
    RightMiddle,
    /// Place the popup to the right of the given area. Aligned to the bottom.
    RightBottom,
    /// Place the popup below the given area. Aligned left.
    BelowLeft,
    /// Place the popup below the given area. Aligned centered.
    BelowCenter,
    /// Place the popup below the given area. Aligned right.
    BelowRight,
    /// Place above. Aligned left.
    Above,
    /// Place below: Aligned right.
    Below,
    /// Place left. Aligned top.
    Left,
    /// Place right. Aligned top.
    Right,
    /// Above or below dependent on available space. Aligned left.
    AboveOrBelow,
    /// Below or above dependent on available space. Aligned left.
    BelowOrAbove,
    /// Use the render-area for the popup, but place it at position (x,y).
    Position(u16, u16),
}

impl Placement {
    pub fn into_constraint(self, rel_area: Rect) -> PopupConstraint {
        match self {
            Placement::None => PopupConstraint::None,
            Placement::AboveLeft => PopupConstraint::AboveLeft(rel_area),
            Placement::AboveCenter => PopupConstraint::AboveCenter(rel_area),
            Placement::AboveRight => PopupConstraint::AboveRight(rel_area),
            Placement::LeftTop => PopupConstraint::LeftTop(rel_area),
            Placement::LeftMiddle => PopupConstraint::LeftMiddle(rel_area),
            Placement::LeftBottom => PopupConstraint::LeftBottom(rel_area),
            Placement::RightTop => PopupConstraint::RightTop(rel_area),
            Placement::RightMiddle => PopupConstraint::RightMiddle(rel_area),
            Placement::RightBottom => PopupConstraint::RightBottom(rel_area),
            Placement::BelowLeft => PopupConstraint::BelowLeft(rel_area),
            Placement::BelowCenter => PopupConstraint::BelowCenter(rel_area),
            Placement::BelowRight => PopupConstraint::BelowRight(rel_area),
            Placement::Above => PopupConstraint::Above(rel_area),
            Placement::Below => PopupConstraint::Below(rel_area),
            Placement::Left => PopupConstraint::Left(rel_area),
            Placement::Right => PopupConstraint::Right(rel_area),
            Placement::AboveOrBelow => PopupConstraint::AboveOrBelow(rel_area),
            Placement::BelowOrAbove => PopupConstraint::BelowOrAbove(rel_area),
            Placement::Position(x, y) => PopupConstraint::Position(x, y),
        }
    }
}

/// Placement relative to the widget area + the widget area.
///
/// The render() call for PopupCore will only use the size of
/// the area given to the render call as the size of the popup.
/// It will calculate the position of the popup given one of
/// these constraints.
///
/// If you build a widget that uses a PopupCore internally you
/// will rather use Placement as a parameter
///
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PopupConstraint {
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
    /// Place above. Aligned left.
    Above(Rect),
    /// Place below: Aligned right.
    Below(Rect),
    /// Place left. Aligned top.
    Left(Rect),
    /// Place right. Aligned top.
    Right(Rect),
    /// Above or below dependent on available space. Aligned left.
    AboveOrBelow(Rect),
    /// Below or above dependent on available space. Aligned left.
    BelowOrAbove(Rect),
    /// Use the render-area for the popup, but place it at position (x,y).
    Position(u16, u16),
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
