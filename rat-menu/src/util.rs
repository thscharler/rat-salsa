use ratatui::layout::{Rect, Size};
use ratatui::prelude::BlockExt;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Padding};
use std::mem;

/// Returns a new style with fg and bg swapped.
///
/// This is not the same as setting Style::reversed().
/// The latter sends special controls to the terminal,
/// the former just swaps.
pub(crate) fn revert_style(mut style: Style) -> Style {
    mem::swap(&mut style.fg, &mut style.bg);
    style
}

pub(crate) fn get_block_size(block: &Option<Block>) -> Size {
    let area = Rect::new(0, 0, 20, 20);
    let inner = block.inner_if_some(area);
    Size {
        width: (inner.left() - area.left()) + (area.right() - inner.right()),
        height: (inner.top() - area.top()) + (area.bottom() - inner.bottom()),
    }
}

pub(crate) fn get_block_padding(block: &Option<Block>) -> Padding {
    let area = Rect::new(0, 0, 20, 20);
    let inner = block.inner_if_some(area);
    Padding {
        left: inner.left() - area.left(),
        right: area.right() - inner.right(),
        top: inner.top() - area.top(),
        bottom: area.bottom() - inner.bottom(),
    }
}
