use crate::event::TableOutcome;
use crate::{TableSelection, TableState};
use rat_event::{ConsumedEvent, HandleEvent, MouseOnly, Regular, ct_event};
use rat_focus::HasFocus;
use rat_scrolled::ScrollAreaState;
use rat_scrolled::event::ScrollOutcome;
use ratatui_crossterm::crossterm::event::Event;
use std::cmp::{max, min};

/// Select a single cell in the table.
///
/// This one supports cell + column + row selection.
#[derive(Debug, Default, Clone)]
pub struct CellSelection {
    /// Selected cell.
    pub lead_cell: Option<(usize, usize)>,
}

impl TableSelection for CellSelection {
    fn count(&self) -> usize {
        if self.lead_cell.is_some() { 1 } else { 0 }
    }

    fn is_selected_row(&self, row: usize) -> bool {
        self.lead_cell.map(|(_scol, srow)| srow) == Some(row)
    }

    fn is_selected_column(&self, column: usize) -> bool {
        self.lead_cell.map(|(scol, _srow)| scol) == Some(column)
    }

    fn is_selected_cell(&self, col: usize, row: usize) -> bool {
        self.lead_cell == Some((col, row))
    }

    fn lead_selection(&self) -> Option<(usize, usize)> {
        self.lead_cell
    }

    fn validate_rows(&mut self, rows: usize) {
        if let Some(lead_cell) = self.lead_cell {
            if rows == 0 {
                self.lead_cell = None;
            } else if lead_cell.1 >= rows {
                self.lead_cell = Some((lead_cell.0, rows - 1));
            }
        }
    }

    fn validate_cols(&mut self, cols: usize) {
        if let Some(lead_cell) = self.lead_cell {
            if cols == 0 {
                self.lead_cell = None;
            } else if lead_cell.0 >= cols {
                self.lead_cell = Some((cols - 1, lead_cell.1));
            }
        }
    }

    fn items_added(&mut self, pos: usize, n: usize) {
        if let Some(lead_cell) = self.lead_cell {
            if lead_cell.1 > pos {
                self.lead_cell = Some((lead_cell.0, lead_cell.1 + n));
            }
        }
    }

    fn items_removed(&mut self, pos: usize, n: usize, rows: usize) {
        if let Some(lead_cell) = self.lead_cell {
            if rows == 0 {
                self.lead_cell = None;
            } else if lead_cell.1 == pos && lead_cell.1 + n >= rows {
                self.lead_cell = Some((lead_cell.0, rows.saturating_sub(1)));
            } else if lead_cell.1 > pos {
                self.lead_cell = Some((lead_cell.0, lead_cell.1.saturating_sub(n).min(pos)));
            }
        }
    }
}

impl CellSelection {
    /// New
    pub fn new() -> CellSelection {
        Self::default()
    }

    /// Clear the selection.
    #[inline]
    pub fn clear(&mut self) {
        self.lead_cell = None;
    }

    /// Selected cell.
    pub fn selected(&self) -> Option<(usize, usize)> {
        self.lead_cell
    }

    #[inline]
    pub fn has_selection(&mut self) -> bool {
        self.lead_cell.is_some()
    }

    /// Select a cell.
    pub fn select_cell(&mut self, select: Option<(usize, usize)>) -> bool {
        let old_cell = self.lead_cell;
        self.lead_cell = select;
        old_cell != self.lead_cell
    }

    /// Select a row. Column stays the same.
    pub fn select_row(&mut self, select: Option<usize>) -> bool {
        let old_cell = self.lead_cell;
        self.lead_cell = match self.lead_cell {
            None => select.map(|v| (0, v)),
            Some((scol, _)) => select.map(|v| (scol, v)),
        };
        old_cell != self.lead_cell
    }

    /// Select a column, row stays the same.
    pub fn select_column(&mut self, select: Option<usize>) -> bool {
        let old_cell = self.lead_cell;
        self.lead_cell = match self.lead_cell {
            None => select.map(|v| (v, 0)),
            Some((_, srow)) => select.map(|v| (v, srow)),
        };
        old_cell != self.lead_cell
    }

    /// Select a cell, clamp between 0 and maximum.
    pub fn move_to(&mut self, select: (usize, usize), maximum: (usize, usize)) -> bool {
        let c = self.move_to_col(select.0, maximum.0);
        let r = self.move_to_row(select.1, maximum.1);
        c || r
    }

    /// Select a column. Row stays the same.
    pub fn move_to_col(&mut self, col: usize, maximum: usize) -> bool {
        let old = self.lead_cell;
        let col = min(col, maximum);
        self.lead_cell = self
            .lead_cell
            .map_or(Some((col, 0)), |(_, srow)| Some((col, srow)));
        old != self.lead_cell
    }

    /// Select a row. Column stays the same.
    pub fn move_to_row(&mut self, row: usize, maximum: usize) -> bool {
        let old = self.lead_cell;
        let row = min(row, maximum);
        self.lead_cell = self
            .lead_cell
            .map_or(Some((0, row)), |(scol, _)| Some((scol, row)));
        old != self.lead_cell
    }

    /// Select the next row, clamp between 0 and maximum.
    pub fn move_down(&mut self, n: usize, maximum: usize) -> bool {
        let old_cell = self.lead_cell;
        self.lead_cell = match self.lead_cell {
            None => Some((0, 0)),
            Some((scol, srow)) => Some((scol, min(srow + n, maximum))),
        };
        old_cell != self.lead_cell
    }

    /// Select the previous row, clamp between 0 and maximum.
    pub fn move_up(&mut self, n: usize, maximum: usize) -> bool {
        let old_cell = self.lead_cell;
        self.lead_cell = match self.lead_cell {
            None => Some((0, maximum)),
            Some((scol, srow)) => Some((scol, srow.saturating_sub(n))),
        };
        old_cell != self.lead_cell
    }

    /// Select the next column, clamp between 0 and maximum.
    pub fn move_right(&mut self, n: usize, maximum: usize) -> bool {
        let old_cell = self.lead_cell;
        self.lead_cell = match self.lead_cell {
            None => Some((0, 0)),
            Some((scol, srow)) => Some((min(scol + n, maximum), srow)),
        };
        old_cell != self.lead_cell
    }

    /// Select the previous row, clamp between 0 and maximum.
    pub fn move_left(&mut self, n: usize, maximum: usize) -> bool {
        let old_cell = self.lead_cell;
        self.lead_cell = match self.lead_cell {
            None => Some((maximum, 0)),
            Some((scol, srow)) => Some((scol.saturating_sub(n), srow)),
        };
        old_cell != self.lead_cell
    }
}

impl HandleEvent<Event, Regular, TableOutcome> for TableState<CellSelection> {
    fn handle(&mut self, event: &Event, _keymap: Regular) -> TableOutcome {
        let res = if self.is_focused() {
            match event {
                ct_event!(keycode press Up) => {
                    if self.move_up(1) {
                        TableOutcome::Selected
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press Down) => {
                    if self.move_down(1) {
                        TableOutcome::Selected
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press CONTROL-Up) | ct_event!(keycode press CONTROL-Home) => {
                    if self.move_to_row(0) {
                        TableOutcome::Selected
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press CONTROL-Down) | ct_event!(keycode press CONTROL-End) => {
                    if self.move_to_row(self.rows.saturating_sub(1)) {
                        TableOutcome::Selected
                    } else {
                        TableOutcome::Unchanged
                    }
                }

                ct_event!(keycode press PageUp) => {
                    if self.move_up(max(1, self.page_len().saturating_sub(1))) {
                        TableOutcome::Selected
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press PageDown) => {
                    if self.move_down(max(1, self.page_len().saturating_sub(1))) {
                        TableOutcome::Selected
                    } else {
                        TableOutcome::Unchanged
                    }
                }

                ct_event!(keycode press Left) => {
                    if self.move_left(1) {
                        TableOutcome::Selected
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press Right) => {
                    if self.move_right(1) {
                        TableOutcome::Selected
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press CONTROL-Left) | ct_event!(keycode press Home) => {
                    if self.move_to_col(0) {
                        TableOutcome::Selected
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press CONTROL-Right) | ct_event!(keycode press End) => {
                    if self.move_to_col(self.columns.saturating_sub(1)) {
                        TableOutcome::Selected
                    } else {
                        TableOutcome::Unchanged
                    }
                }

                _ => TableOutcome::Continue,
            }
        } else {
            TableOutcome::Continue
        };

        if res == TableOutcome::Continue {
            self.handle(event, MouseOnly)
        } else {
            res
        }
    }
}

impl HandleEvent<Event, MouseOnly, TableOutcome> for TableState<CellSelection> {
    fn handle(&mut self, event: &Event, _keymap: MouseOnly) -> TableOutcome {
        let mut r = match event {
            ct_event!(mouse any for m) if self.mouse.drag(self.table_area, m) => {
                if self.move_to(self.cell_at_drag((m.column, m.row))) {
                    TableOutcome::Selected
                } else {
                    TableOutcome::Unchanged
                }
            }
            ct_event!(mouse down Left for column, row) => {
                if self.area.contains((*column, *row).into()) {
                    if let Some(new_cell) = self.cell_at_clicked((*column, *row)) {
                        if self.move_to(new_cell) {
                            TableOutcome::Selected
                        } else {
                            TableOutcome::Unchanged
                        }
                    } else {
                        TableOutcome::Continue
                    }
                } else {
                    TableOutcome::Continue
                }
            }
            _ => TableOutcome::Continue,
        };

        r = r.or_else(|| {
            let mut sas = ScrollAreaState::new()
                .area(self.inner)
                .h_scroll(&mut self.hscroll)
                .v_scroll(&mut self.vscroll);
            match sas.handle(event, MouseOnly) {
                ScrollOutcome::Up(v) => {
                    if self.scroll_up(v) {
                        TableOutcome::Changed
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ScrollOutcome::Down(v) => {
                    if self.scroll_down(v) {
                        TableOutcome::Changed
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ScrollOutcome::VPos(v) => {
                    if self.set_row_offset(self.vscroll.limited_offset(v)) {
                        TableOutcome::Changed
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ScrollOutcome::Left(v) => {
                    if self.scroll_left(v) {
                        TableOutcome::Changed
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ScrollOutcome::Right(v) => {
                    if self.scroll_right(v) {
                        TableOutcome::Changed
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ScrollOutcome::HPos(v) => {
                    if self.set_x_offset(self.hscroll.limited_offset(v)) {
                        TableOutcome::Changed
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ScrollOutcome::Continue => TableOutcome::Continue,
                ScrollOutcome::Unchanged => TableOutcome::Unchanged,
                ScrollOutcome::Changed => TableOutcome::Changed,
            }
        });
        r
    }
}

/// Handle all events.
/// Table events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events(
    state: &mut TableState<CellSelection>,
    focus: bool,
    event: &Event,
) -> TableOutcome {
    state.focus.set(focus);
    state.handle(event, Regular)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(state: &mut TableState<CellSelection>, event: &Event) -> TableOutcome {
    state.handle(event, MouseOnly)
}
