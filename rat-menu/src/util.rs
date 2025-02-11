use ratatui_core::style::{Style, Stylize};
use std::mem;

/// Returns a new style with fg and bg swapped.
///
/// This is not the same as setting Style::reversed().
/// The latter sends special controls to the terminal,
/// the former just swaps.
pub(crate) fn revert_style(mut style: Style) -> Style {
    if style.fg.is_some() && style.bg.is_some() {
        mem::swap(&mut style.fg, &mut style.bg);
        style
    } else {
        style.black().on_white()
    }
}

/// Fallback for select style.
pub(crate) fn fallback_select_style(style: Style) -> Style {
    if style.fg.is_some() || style.bg.is_some() {
        style
    } else {
        style.underlined()
    }
}
