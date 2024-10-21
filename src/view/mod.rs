#[allow(clippy::module_inception)]
mod view;
mod viewport;

use crate::_private::NonExhaustive;
use rat_scrolled::ScrollStyle;
use ratatui::style::Style;
use ratatui::widgets::Block;
pub use view::{View, ViewState};
pub use viewport::{Viewport, ViewportState};

#[derive(Debug)]
pub struct ViewStyle {
    pub style: Style,
    pub block: Option<Block<'static>>,
    pub scroll: Option<ScrollStyle>,

    pub non_exhaustive: NonExhaustive,
}

impl Default for ViewStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            block: None,
            scroll: None,
            non_exhaustive: NonExhaustive,
        }
    }
}
