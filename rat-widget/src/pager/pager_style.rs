use crate::_private::NonExhaustive;
use ratatui::layout::Alignment;
use ratatui::style::Style;
use ratatui::widgets::Block;

/// All styles for a pager.
#[derive(Debug, Clone)]
pub struct PagerStyle {
    pub style: Style,
    pub label_style: Option<Style>,
    pub label_alignment: Option<Alignment>,
    pub navigation: Option<Style>,
    pub title: Option<Style>,
    pub block: Option<Block<'static>>,
    pub next_page_mark: Option<&'static str>,
    pub prev_page_mark: Option<&'static str>,
    pub first_page_mark: Option<&'static str>,
    pub last_page_mark: Option<&'static str>,
    pub non_exhaustive: NonExhaustive,
}

impl Default for PagerStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            label_style: None,
            label_alignment: None,
            navigation: None,
            title: None,
            block: None,
            next_page_mark: None,
            prev_page_mark: None,
            first_page_mark: None,
            last_page_mark: None,
            non_exhaustive: NonExhaustive,
        }
    }
}
