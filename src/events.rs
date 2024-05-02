#[derive(Debug, Default)]
pub struct DefaultKeys;

#[derive(Debug, Default)]
pub struct MouseOnly;

///
/// A very broad trait for a event handler function.
///
pub trait HandleEvent<Event, KeyMap, R> {
    /// Handle an event.
    ///
    /// * self - Should be the widget state.
    /// * event - Event
    /// * focus - The widget has the input focus
    /// * keymap - Which keymapping. Predefined are DefaultKeys and MouseOnly.
    fn handle(&mut self, event: &Event, focus: bool, keymap: KeyMap) -> R;
}

/// Result value for event-handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The given event was not handled at all.
    NotUsed,
    /// The event was handled, no repaint necessary.
    Unchanged,
    /// The event was handled, repaint necessary.
    Changed,
}
