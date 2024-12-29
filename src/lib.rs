#![doc = include_str!("../readme.md")]

/// Trait for accessing the screen-cursor.
///
/// In ratatui the screen-cursor can't be set during rendering, instead
/// it must be set with the Frame at some point.
///
/// This trait provides a method to get the screen cursor (if any)
/// for a widget.
pub trait HasScreenCursor {
    /// This returns the cursor position if
    /// - The cursor is displayed at all, and not scrolled off-screen.
    /// - The widget has some kind of input focus
    /// - other reasons
    fn screen_cursor(&self) -> Option<(u16, u16)>;
}
