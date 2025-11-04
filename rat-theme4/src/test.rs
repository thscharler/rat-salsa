use crate::palettes::IMPERIAL;
use crate::{SalsaTheme, WidgetStyle, create_empty};
use rat_widget::scrolled::ScrollStyle;

pub fn ff() {
    let th = create_empty("f", IMPERIAL);

    let s: ScrollStyle = th.style(WidgetStyle::SCROLL);
    let c = th.p().normal_contrast(th.p().green[3]);
}
