use ratatui::prelude::Style;
use ratatui::style::Stylize;
use std::mem;

pub(crate) fn revert_style(mut style: Style) -> Style {
    if style.fg.is_some() && style.bg.is_some() {
        mem::swap(&mut style.fg, &mut style.bg);
        style
    } else {
        style.black().on_white()
    }
}
