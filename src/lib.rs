#![doc = include_str!("../readme.md")]

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
pub trait HandleEvent<Event, KeyMap, R: UsedEvent> {
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
pub trait UsedEvent {
    fn used_event(&self) -> bool;
}

impl<V, E> UsedEvent for Result<V, E>
where
    V: UsedEvent,
{
    fn used_event(&self) -> bool {
        match self {
            Ok(v) => v.used_event(),
            Err(_) => true,
        }
    }
}

impl<V> UsedEvent for Option<V>
where
    V: UsedEvent,
{
    fn used_event(&self) -> bool {
        match self {
            Some(v) => v.used_event(),
            None => true,
        }
    }
}
