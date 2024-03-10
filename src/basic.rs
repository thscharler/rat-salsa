//!
//! Some basic widgets.
//!

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::Style;

/// Clears an area, but with style.
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct ClearStyle {
    style: Style,
}

impl ClearStyle {
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }
}

impl Widget for ClearStyle {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                buf.get_mut(x, y).reset();
            }
        }

        buf.set_style(area, self.style);
    }
}
