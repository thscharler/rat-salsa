#![doc = include_str!("../readme.md")]

use std::cmp::{max, min};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

pub mod crossterm;
pub mod util;

/// Default key bindings when focused.
/// Also handles all events that are MouseOnly.
#[derive(Debug, Default)]
pub struct FocusKeys;

/// Handler for mouse events only.
/// * Useful when writing a new key binding.
/// * FocusKeys should only be handled by the focused widget,
///   while mouse events are not bound by the focus.
#[derive(Debug, Default)]
pub struct MouseOnly;

///
/// A very broad trait for an event handler for widgets.
///
/// As widget types are only short-lived, this trait should be implemented
/// for the state type. Thereby it can modify any state, and it can
/// return an arbitrary result, that fits the widget.
///
pub trait HandleEvent<Event, KeyMap, R: ConsumedEvent> {
    /// Handle an event.
    ///
    /// * self - Should be the widget state.
    /// * event - Event
    /// * keymap - Which keymapping. Predefined are FocusKeys and MouseOnly.
    fn handle(&mut self, event: &Event, keymap: KeyMap) -> R;
}

/// When composing several widgets, the minimum information from the outcome
/// of the inner widget is, whether it used & consumed the event.
///
/// This allows shortcutting the event-handling and prevents double
/// interactions with the event. The inner widget can special case an event,
/// and the fallback behaviour on the outer layer should not run too.
pub trait ConsumedEvent {
    fn is_consumed(&self) -> bool;
}

impl<V, E> ConsumedEvent for Result<V, E>
where
    V: ConsumedEvent,
{
    fn is_consumed(&self) -> bool {
        match self {
            Ok(v) => v.is_consumed(),
            Err(_) => true,
        }
    }
}

impl<V> ConsumedEvent for Option<V>
where
    V: ConsumedEvent,
{
    fn is_consumed(&self) -> bool {
        match self {
            Some(v) => v.is_consumed(),
            None => true,
        }
    }
}

/// A baseline Outcome for event-handling.
///
/// A widget can define its own, if it has more things to report.
/// It would be nice of the widget though, if its outcome would be
/// convertible to this baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Outcome {
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
}

impl ConsumedEvent for Outcome {
    fn is_consumed(&self) -> bool {
        *self != Outcome::NotUsed
    }
}

impl BitOr for Outcome {
    type Output = Outcome;

    fn bitor(self, rhs: Self) -> Self::Output {
        max(self, rhs)
    }
}

impl BitAnd for Outcome {
    type Output = Outcome;

    fn bitand(self, rhs: Self) -> Self::Output {
        min(self, rhs)
    }
}

impl BitOrAssign for Outcome {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.bitor(rhs);
    }
}

impl BitAndAssign for Outcome {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = self.bitand(rhs);
    }
}

impl From<bool> for Outcome {
    fn from(value: bool) -> Self {
        if value {
            Outcome::Changed
        } else {
            Outcome::Unchanged
        }
    }
}
