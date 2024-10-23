use crate::_private::NonExhaustive;
use rat_scrolled::ScrollStyle;
use ratatui::widgets::Block;

/// Clipper styles.
#[derive(Debug, Clone)]
pub struct ClipperStyle {
    pub block: Option<Block<'static>>,
    pub scroll: Option<ScrollStyle>,

    pub non_exhaustive: NonExhaustive,
}

impl Default for ClipperStyle {
    fn default() -> Self {
        Self {
            block: None,
            scroll: None,
            non_exhaustive: NonExhaustive,
        }
    }
}
