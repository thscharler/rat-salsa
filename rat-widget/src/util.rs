//!
//! Small helpers.
//!
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Rect, Size};
use ratatui_core::style::Style;
use ratatui_core::widgets::Widget;
use ratatui_widgets::block::{Block, BlockExt, Padding};
use std::{fmt, mem};

/// Union the areas, but regard only non-empty ones.
///
/// This can help if you want to union two non-adjacent areas.
pub fn union_non_empty(area1: Rect, area2: Rect) -> Rect {
    if area1.is_empty() && area2.is_empty() {
        Rect::new(area1.x, area1.y, 0, 0)
    } else if area1.is_empty() {
        area2
    } else if area2.is_empty() {
        area1
    } else {
        area1.union(area2)
    }
}

/// Union the areas, but regard only non-empty ones.
///
/// This can help if you want to union two non-adjacent areas.
pub fn union_all_non_empty(areas: &[Rect]) -> Rect {
    let mut area = Rect::default();
    for a in areas {
        if area.is_empty() {
            if !a.is_empty() {
                area = *a;
            }
        } else {
            if !a.is_empty() {
                area = area.union(*a);
            }
        }
    }
    area
}

/// Returns a new style with fg and bg swapped.
///
/// This is not the same as setting Style::reversed().
/// The latter sends special controls to the terminal,
/// the former just swaps.
pub fn revert_style(mut style: Style) -> Style {
    if style.fg.is_some() || style.bg.is_some() {
        mem::swap(&mut style.fg, &mut style.bg);
        style
    } else {
        style.black().on_white()
    }
}

/// Fallback for select style.
pub fn fallback_select_style(style: Style) -> Style {
    if style.fg.is_some() || style.bg.is_some() {
        style
    } else {
        style.underlined()
    }
}

/// Reset an area of the buffer.
pub fn reset_buf_area(area: Rect, buf: &mut Buffer) {
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.reset();
            }
        }
    }
}

/// Fill the given area of the buffer.
pub fn fill_buf_area(buf: &mut Buffer, area: Rect, symbol: &str, style: impl Into<Style>) {
    let style = style.into();

    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.reset();
                cell.set_symbol(symbol);
                cell.set_style(style);
            }
        }
    }
}

pub fn rect_dbg(area: Rect) -> String {
    use fmt::Write;
    let mut buf = String::new();
    _ = write!(buf, "{}:{}+{}+{}", area.x, area.y, area.width, area.height);
    buf
}

pub(crate) const DOUBLE_VERTICAL_SINGLE_LEFT: &str = "\u{2562}";
pub(crate) const DOUBLE_VERTICAL_SINGLE_RIGHT: &str = "\u{255F}";
pub(crate) const THICK_VERTICAL_SINGLE_LEFT: &str = "\u{2528}";
pub(crate) const THICK_VERTICAL_SINGLE_RIGHT: &str = "\u{2520}";

/// Get the padding the block imposes as Padding.
pub fn block_padding(block: &Option<Block<'_>>) -> Padding {
    let area = Rect::new(0, 0, 20, 20);
    let inner = block.inner_if_some(area);
    Padding {
        left: inner.left() - area.left(),
        right: area.right() - inner.right(),
        top: inner.top() - area.top(),
        bottom: area.bottom() - inner.bottom(),
    }
}

/// Get the padding the block imposes as Padding.
pub fn block_padding2(block: &Block<'_>) -> Padding {
    let area = Rect::new(0, 0, 20, 20);
    let inner = block.inner(area);
    Padding {
        left: inner.left() - area.left(),
        right: area.right() - inner.right(),
        top: inner.top() - area.top(),
        bottom: area.bottom() - inner.bottom(),
    }
}

/// Get the padding the block imposes as a Size.
pub fn block_size(block: &Option<Block<'_>>) -> Size {
    let area = Rect::new(0, 0, 20, 20);
    let inner = block.inner_if_some(area);
    Size {
        width: (inner.left() - area.left()) + (area.right() - inner.right()),
        height: (inner.top() - area.top()) + (area.bottom() - inner.bottom()),
    }
}

pub(crate) fn block_left(block: &Block<'_>) -> String {
    let area = Rect::new(0, 0, 3, 3);
    let mut buf = Buffer::empty(area);
    block.clone().render(area, &mut buf);
    buf.cell((0, 1)).expect("cell").symbol().into()
}

pub(crate) fn block_right(block: &Block<'_>) -> String {
    let area = Rect::new(0, 0, 3, 3);
    let mut buf = Buffer::empty(area);
    block.clone().render(area, &mut buf);
    buf.cell((2, 1)).expect("cell").symbol().into()
}

#[allow(dead_code)]
pub(crate) fn block_top(block: &Block<'_>) -> String {
    let area = Rect::new(0, 0, 3, 3);
    let mut buf = Buffer::empty(area);
    block.clone().render(area, &mut buf);
    buf.cell((1, 0)).expect("cell").symbol().into()
}

#[allow(dead_code)]
pub(crate) fn block_bottom(block: &Block<'_>) -> String {
    let area = Rect::new(0, 0, 3, 3);
    let mut buf = Buffer::empty(area);
    block.clone().render(area, &mut buf);
    buf.cell((1, 2)).expect("cell").symbol().into()
}
