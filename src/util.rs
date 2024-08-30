use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Style;
use ratatui::style::Stylize;
use std::mem;

pub(crate) fn revert_style(mut style: Style) -> Style {
    if style.fg.is_some() && style.bg.is_some() {
        mem::swap(&mut style.fg, &mut style.bg);
        style
    } else {
        style.black().on_white()
    }
}

/// Move a tmp-buffer to a target.
/// All cells in the tmp-buffer are reset to defaults.
///
/// * tmp: Temporary buffer
/// * h_offset: Left shift of the tmp-buffer.
/// * view_area: clipped area in the target buffer.
/// * buf: Target buffer
pub(crate) fn transfer_buffer(tmp: &mut Buffer, h_offset: u16, view_area: Rect, buf: &mut Buffer) {
    // copy buffer
    for (cell_offset, cell) in tmp.content.iter_mut().enumerate() {
        let tmp_row = cell_offset as u16 / tmp.area.width;
        let tmp_col = cell_offset as u16 % tmp.area.width;

        let cell = mem::take(cell);

        // ensure tmp_col-h_offset doesn't underflow.
        if tmp_col >= h_offset {
            let buf_row = view_area.y + tmp_row;
            let buf_col = view_area.x + tmp_col - h_offset;

            if view_area.contains((buf_col, buf_row).into()) {
                if let Some(buf_cell) = buf.cell_mut((buf_col, buf_row)) {
                    *buf_cell = cell
                }
            }
        }
    }
}
