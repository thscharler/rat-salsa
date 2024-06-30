use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Style;

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

            if area.contains(Position::new(col, row)) {
                *buf.get_mut(col, row) = cell;
            } else {
                // clip
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
                buf.get_mut(c, r).reset();
                buf.get_mut(c, r).set_style(empty_style);
            }
        }
    }
}
