//! more blue, in multiple colors.

use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Style;
use ratatui_core::widgets::StatefulWidget;

#[derive(Debug)]
pub struct Blue {
    style: Style,
    focus_style: Style,
}

impl Blue {
    pub fn new() -> Self {
        Self {
            style: Style::new().on_blue(),
            focus_style: Style::new().on_light_blue(),
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn focus_style(mut self, style: Style) -> Self {
        self.focus_style = style;
        self
    }
}

impl StatefulWidget for Blue {
    type State = BlueState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        if state.focus.is_focused() {
            buf.set_style(area, self.focus_style);
        } else {
            buf.set_style(area, self.style);
        }
    }
}

#[derive(Debug, Default)]
pub struct BlueState {
    pub area: Rect,
    pub focus: FocusFlag,
}

impl BlueState {
    pub fn new() -> Self {
        Self {
            area: Default::default(),
            focus: FocusFlag::new().with_name("blue"),
        }
    }

    pub fn named(name: &str) -> Self {
        Self {
            area: Default::default(),
            focus: FocusFlag::new().with_name(name),
        }
    }
}

impl HasFocus for BlueState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }
}
