#![doc = include_str!("../readme.md")]

pub mod crossterm;

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
pub trait HandleEvent<Event, KeyMap, R> {
    /// Handle an event.
    ///
    /// * self - Should be the widget state.
    /// * event - Event
    /// * keymap - Which keymapping. Predefined are FocusKeys and MouseOnly.
    fn handle(&mut self, event: &Event, keymap: KeyMap) -> R;
}
