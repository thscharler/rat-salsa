use crate::_private::NonExhaustive;
use rat_scrolled::ScrollStyle;
use ratatui::widgets::Block;

/// All styles for a xview.
#[derive(Debug)]
pub struct ViewStyle {
    pub block: Option<Block<'static>>,
    pub scroll: Option<ScrollStyle>,

    pub non_exhaustive: NonExhaustive,
}

impl Default for ViewStyle {
    fn default() -> Self {
        Self {
            block: None,
            scroll: None,
            non_exhaustive: NonExhaustive,
        }
    }
}
