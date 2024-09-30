#[allow(unused_imports)]
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use std::cmp::min;
use std::mem;

/// Returns a new style with fg and bg swapped.
///
/// This is not the same as setting Style::reversed().
/// The latter sends special controls to the terminal,
/// the former just swaps.
pub fn revert_style(mut style: Style) -> Style {
    if style.fg.is_some() && style.bg.is_some() {
        mem::swap(&mut style.fg, &mut style.bg);
        style
    } else {
        style.black().on_white()
    }
}

/// Reset an area of the buffer.
pub fn reset_buf_area(buf: &mut Buffer, area: Rect) {
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

/// Copy a tmp buffer to another buf.
/// The tmp-buffer is offset by h_offset/v_offset.
/// Any outside area is cleared and set to empty_style.
/// Everything is clipped to the target area.
pub(crate) fn copy_buffer(
    view_area: Rect,
    mut tmp: Buffer,
    h_offset: usize,
    v_offset: usize,
    empty_style: Style,
    area: Rect,
    buf: &mut Buffer,
) {
    // copy buffer
    for (cell_offset, cell) in tmp.content.drain(..).enumerate() {
        let tmp_row = cell_offset as u16 / tmp.area.width;
        let tmp_col = cell_offset as u16 % tmp.area.width;

        if area.y + tmp_row >= v_offset as u16 && area.x + tmp_col >= h_offset as u16 {
            let row = area.y + tmp_row - v_offset as u16;
            let col = area.x + tmp_col - h_offset as u16;

            if row < area.bottom() && col < area.right() {
                if let Some(buf_cell) = buf.cell_mut((col, row)) {
                    *buf_cell = cell;
                }
            } else {
                // clip2
            }
        } else {
            // clip2
        }
    }

    // clear the rest
    let filled_left = (area.x + view_area.width).saturating_sub(h_offset as u16);
    let filled_bottom = (area.y + view_area.height).saturating_sub(v_offset as u16);

    for r in area.y..area.y + area.height {
        for c in area.x..area.x + area.width {
            if c >= filled_left || r >= filled_bottom {
                if let Some(buf_cell) = buf.cell_mut((c, r)) {
                    buf_cell.reset();
                    buf_cell.set_style(empty_style);
                }
            }
        }
    }
}
