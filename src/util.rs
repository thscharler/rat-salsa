#[allow(unused_imports)]
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use std::cmp::min;
use std::mem;

/// Create a Line from the given text. The first '_' marks
/// the navigation-char.
pub(crate) fn menu_str(txt: &str) -> (Line<'_>, Option<char>) {
    let mut line = Line::default();

    let mut idx_underscore = None;
    let mut idx_navchar_start = None;
    let mut navchar = None;
    let mut idx_navchar_end = None;
    let cit = txt.char_indices();
    for (idx, c) in cit {
        if idx_underscore.is_none() && c == '_' {
            idx_underscore = Some(idx);
        } else if idx_underscore.is_some() && idx_navchar_start.is_none() {
            navchar = Some(c.to_ascii_lowercase());
            idx_navchar_start = Some(idx);
        } else if idx_navchar_start.is_some() && idx_navchar_end.is_none() {
            idx_navchar_end = Some(idx);
        }
    }
    if idx_navchar_start.is_some() && idx_navchar_end.is_none() {
        idx_navchar_end = Some(txt.len());
    }

    if let Some(idx_underscore) = idx_underscore {
        if let Some(idx_navchar_start) = idx_navchar_start {
            if let Some(idx_navchar_end) = idx_navchar_end {
                line.spans.push(Span::from(&txt[0..idx_underscore]));
                line.spans
                    .push(Span::from(&txt[idx_navchar_start..idx_navchar_end]).underlined());
                line.spans.push(Span::from(&txt[idx_navchar_end..]));

                return (line, navchar);
            }
        }
    }

    line.spans.push(Span::from(txt));

    (line, None)
}

pub(crate) fn revert_style(mut style: Style) -> Style {
    if style.fg.is_some() && style.bg.is_some() {
        mem::swap(&mut style.fg, &mut style.bg);
        style
    } else {
        style.black().on_white()
    }
}

/// Select previous.
pub(crate) fn prev_opt(select: Option<usize>, change: usize, len: usize) -> Option<usize> {
    if let Some(select) = select {
        Some(prev(select, change))
    } else {
        Some(len.saturating_sub(1))
    }
}

/// Select next.
pub(crate) fn next_opt(selected: Option<usize>, change: usize, len: usize) -> Option<usize> {
    if let Some(select) = selected {
        Some(next(select, change, len))
    } else {
        Some(0)
    }
}

/// Select previous.
pub(crate) fn prev(select: usize, change: usize) -> usize {
    select.saturating_sub(change)
}

/// Select next.
pub(crate) fn next(select: usize, change: usize, len: usize) -> usize {
    min(select + change, len.saturating_sub(1))
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
