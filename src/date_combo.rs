use crate::calendar::MonthState;
use rat_popup::{Placement, PopupCore, PopupCoreState};
use rat_scrolled::Scroll;
use rat_text::date_input::{DateInput, DateInputState};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Block;

#[derive(Debug, Default, Clone)]
pub struct DateCombo<'a> {
    style: Style,
    button_style: Option<Style>,
    select_style: Option<Style>,
    focus_style: Option<Style>,
    invalid_style: Option<Style>,
    block: Option<Block<'a>>,

    popup_placement: Placement,
    popup_boundary: Option<Rect>,
    popup_block: Option<Block<'a>>,
    popup_style: Option<Style>,
}

#[derive(Debug, Default, Clone)]
pub struct DateComboState {
    pub input: DateInputState,
    pub popup: PopupCoreState,
    pub month: MonthState,
}

impl<'a> DateCombo<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn focus_style(mut self, style: Style) -> Self {
        self.focus_style = Some(style);
        self
    }

    pub fn select_style(mut self, style: Style) -> Self {
        self.select_style = Some(style);
        self
    }

    pub fn invalid_style(mut self, style: Style) -> Self {
        self.invalid_style = Some(style);
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

#[derive(Debug)]
pub struct RenderDateCombo<'a> {
    input: DateInput<'a>,
}
