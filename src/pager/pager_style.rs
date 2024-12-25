use crate::_private::NonExhaustive;
use ratatui::layout::Alignment;
use ratatui::prelude::Style;
use ratatui::widgets::Block;

/// All styles for a pager.
#[derive(Debug, Clone)]
pub struct PagerStyle {
    pub style: Style,
    pub label_style: Option<Style>,
    pub label_alignment: Option<Alignment>,
    pub nav: Option<Style>,
    pub divider: Option<Style>,
    pub title: Option<Style>,
    pub block: Option<Block<'static>>,
    pub non_exhaustive: NonExhaustive,
}

impl Default for PagerStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            label_style: None,
            label_alignment: None,
            nav: None,
            divider: None,
            title: None,
            block: None,
            non_exhaustive: NonExhaustive,
        }
    }
}
