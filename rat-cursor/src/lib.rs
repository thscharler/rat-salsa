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

/// Returns the screen_cursor for the first widget that returns one.
#[inline(always)]
pub fn screen_cursor<const N: usize>(list: [&dyn HasScreenCursor; N]) -> Option<(u16, u16)> {
    for v in list {
        if let Some(v) = v.screen_cursor() {
            return Some(v);
        }
    }
    None
}

/// Create the implementation of HasScreenCursor for the
/// given list of struct members.
#[macro_export]
macro_rules! impl_screen_cursor {
    ($($n:ident),* for $ty:ty) => {
        impl HasScreenCursor for $ty {
            fn screen_cursor(&self) -> Option<(u16, u16)> {
                use $crate::screen_cursor;
                screen_cursor([
                    $(&self.$n),*
                ])
            }
        }
    };
}
