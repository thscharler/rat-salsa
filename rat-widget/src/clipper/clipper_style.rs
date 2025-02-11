use crate::_private::NonExhaustive;
use rat_scrolled::ScrollStyle;
use ratatui_core::layout::Alignment;
use ratatui_core::style::Style;
use ratatui_widgets::block::Block;

/// Clipper styles.
#[derive(Debug, Clone)]
pub struct ClipperStyle {
    pub style: Style,
    pub label_style: Option<Style>,
    pub label_alignment: Option<Alignment>,
    pub block: Option<Block<'static>>,
    pub scroll: Option<ScrollStyle>,
    pub non_exhaustive: NonExhaustive,
}

impl Default for ClipperStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            label_style: None,
            label_alignment: None,
            block: None,
            scroll: None,
            non_exhaustive: NonExhaustive,
        }
    }
}
