//!
//! Some utility functions that pop up all the time.
//!

use ratatui::layout::Rect;

/// Which row of the given contains the position.
/// This uses only the vertical components of the given areas.
///
/// You might want to limit calling this functions when the full
/// position is inside your target rect.
pub fn row_at_clicked(areas: &[Rect], y_pos: u16) -> Option<usize> {
    for (i, r) in areas.iter().enumerate() {
        if y_pos >= r.top() && y_pos < r.bottom() {
            return Some(i);
        }
    }
    None
}

/// Column at given position.
/// This uses only the horizontal components of the given areas.
///
/// You might want to limit calling this functions when the full
/// position is inside your target rect.
pub fn column_at_clicked(areas: &[Rect], x_pos: u16) -> Option<usize> {
    for (i, r) in areas.iter().enumerate() {
        if x_pos >= r.left() && x_pos < r.right() {
            return Some(i);
        }
    }
    None
}

/// Find a row position when dragging with the mouse. This uses positions
/// outside the given areas to estimate an invisible row that could be meant
/// by the mouse position. It uses the heuristic `1 row == 1 item` for simplicity’s
/// sake.
///
/// Rows outside the bounds are returned as Err(isize), rows inside as Ok(usize).
pub fn row_at_drag(encompassing: Rect, areas: &[Rect], y_pos: u16) -> Result<usize, isize> {
    if let Some(row) = row_at_clicked(areas, y_pos) {
        return Ok(row);
    }

    // assume row-height=1 for outside the box.
    if y_pos < encompassing.top() {
        Err(y_pos as isize - encompassing.top() as isize)
    } else {
        if let Some(last) = areas.last() {
            Err(y_pos as isize - last.bottom() as isize + 1)
        } else {
            Err(y_pos as isize - encompassing.top() as isize)
        }
    }
}

/// Column when dragging. Can go outside the area.
pub fn column_at_drag(encompassing: Rect, areas: &[Rect], x_pos: u16) -> Result<usize, isize> {
    if let Some(column) = column_at_clicked(areas, x_pos) {
        return Ok(column);
    }

    // change by 1 column if outside the box
    if x_pos < encompassing.left() {
        Err(x_pos as isize - encompassing.left() as isize)
    } else {
        if let Some(last) = areas.last() {
            Err(x_pos as isize - last.right() as isize + 1)
        } else {
            Err(x_pos as isize - encompassing.left() as isize)
        }
    }
}
