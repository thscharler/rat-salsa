use crate::event::Outcome;
use crate::{FTableState, TableSelection};
use rat_event::{ct_event, FocusKeys, HandleEvent, MouseOnly};
use std::cmp::min;

/// Select a single cell in the table.
///
/// This one supports cell + column + row selection.
#[derive(Debug, Default, Clone)]
pub struct CellSelection {
    /// Selected cell.
    pub lead_cell: Option<(usize, usize)>,
}

impl TableSelection for CellSelection {
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
    pub fn select_clamped(&mut self, select: (usize, usize), maximum: (usize, usize)) -> bool {
        let old_cell = self.lead_cell;

        let col = if select.0 <= maximum.0 {
            select.0
        } else {
            maximum.0
        };
        let row = if select.1 <= maximum.1 {
            select.1
        } else {
            maximum.1
        };

        self.lead_cell = Some((col, row));

        old_cell != self.lead_cell
    }

    /// Select the next row, clamp between 0 and maximum.
    pub fn next_row(&mut self, n: usize, maximum: usize) -> bool {
        let old_cell = self.lead_cell;
        self.lead_cell = match self.lead_cell {
            None => Some((0, 0)),
            Some((scol, srow)) => Some((scol, min(srow + n, maximum))),
        };
        old_cell != self.lead_cell
    }

    /// Select the previous row, clamp between 0 and maximum.
    pub fn prev_row(&mut self, n: usize) -> bool {
        let old_cell = self.lead_cell;
        self.lead_cell = match self.lead_cell {
            None => Some((0, 0)),
            Some((scol, srow)) => Some((scol, if srow >= n { srow - n } else { 0 })),
        };
        old_cell != self.lead_cell
    }

    /// Select the next column, clamp between 0 and maximum.
    pub fn next_column(&mut self, n: usize, maximum: usize) -> bool {
        let old_cell = self.lead_cell;
        self.lead_cell = match self.lead_cell {
            None => Some((0, 0)),
            Some((scol, srow)) => Some((min(scol + n, maximum), srow)),
        };
        old_cell != self.lead_cell
    }

    /// Select the previous row, clamp between 0 and maximum.
    pub fn prev_column(&mut self, n: usize) -> bool {
        let old_cell = self.lead_cell;
        self.lead_cell = match self.lead_cell {
            None => Some((0, 0)),
            Some((scol, srow)) => Some((if scol >= n { scol - n } else { 0 }, srow)),
        };
        old_cell != self.lead_cell
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for FTableState<CellSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> Outcome {
        let res = match event {
            ct_event!(keycode press Down) => {
                let r = self
                    .selection
                    .next_row(1, self.rows.saturating_sub(1))
                    .into();
                self.scroll_to_selected();
                r
            }
            ct_event!(keycode press Up) => {
                let r = self.selection.prev_row(1).into();
                self.scroll_to_selected();
                r
            }
            ct_event!(keycode press CONTROL-Down) | ct_event!(keycode press End) => {
                let r = self
                    .selection
                    .select_row(Some(self.rows.saturating_sub(1)))
                    .into();
                self.scroll_to_selected();
                r
            }
            ct_event!(keycode press CONTROL-Up) | ct_event!(keycode press Home) => {
                let r = self.selection.select_row(Some(0)).into();
                self.scroll_to_selected();
                r
            }
            ct_event!(keycode press PageUp) => {
                let r = self
                    .selection
                    .prev_row(self.vertical_page().saturating_sub(1))
                    .into();
                self.scroll_to_selected();
                r
            }
            ct_event!(keycode press PageDown) => {
                let r = self
                    .selection
                    .next_row(
                        self.vertical_page().saturating_sub(1),
                        self.rows.saturating_sub(1),
                    )
                    .into();
                self.scroll_to_selected();
                r
            }
            ct_event!(keycode press Right) => {
                let r = self
                    .selection
                    .next_column(1, self.columns.saturating_sub(1))
                    .into();
                self.scroll_to_selected();
                r
            }
            ct_event!(keycode press Left) => {
                let r = self.selection.prev_column(1).into();
                self.scroll_to_selected();
                r
            }
            ct_event!(keycode press CONTROL-Right) | ct_event!(keycode press SHIFT-End) => {
                let r = self
                    .selection
                    .select_column(Some(self.columns.saturating_sub(1)))
                    .into();
                self.scroll_to_selected();
                r
            }
            ct_event!(keycode press CONTROL-Left) | ct_event!(keycode press SHIFT-Home) => {
                let r = self.selection.select_column(Some(0)).into();
                self.scroll_to_selected();
                r
            }
            _ => Outcome::NotUsed,
        };

        if res == Outcome::NotUsed {
            self.handle(event, MouseOnly)
        } else {
            res
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for FTableState<CellSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        match event {
            ct_event!(mouse any for m) if self.mouse.drag(self.table_area, m) => {
                let new_cell = self.cell_at_drag((m.column, m.row));
                let r = self
                    .selection
                    .select_clamped(
                        new_cell,
                        (self.columns.saturating_sub(1), self.rows.saturating_sub(1)),
                    )
                    .into();
                self.scroll_to_selected();
                r
            }
            ct_event!(scroll down for column,row) => {
                if self.area.contains((*column, *row).into()) {
                    self.scroll_down(self.table_area.height as usize / 10)
                        .into()
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(scroll up for column, row) => {
                if self.area.contains((*column, *row).into()) {
                    self.scroll_up(self.table_area.height as usize / 10).into()
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(scroll ALT down for column,row) => {
                if self.area.contains((*column, *row).into()) {
                    self.scroll_right(1).into()
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(scroll ALT up for column, row) => {
                if self.area.contains((*column, *row).into()) {
                    self.scroll_left(1).into()
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(mouse down Left for column, row) => {
                let pos = (*column, *row);
                if self.area.contains(pos.into()) {
                    if let Some(new_cell) = self.cell_at_clicked(pos) {
                        self.selection
                            .select_clamped(
                                new_cell,
                                (self.columns.saturating_sub(1), self.rows.saturating_sub(1)),
                            )
                            .into()
                    } else {
                        Outcome::NotUsed
                    }
                } else {
                    Outcome::NotUsed
                }
            }

            _ => Outcome::NotUsed,
        }
    }
}

/// Handle all events.
/// Table events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events(
    state: &mut FTableState<CellSelection>,
    focus: bool,
    event: &crossterm::event::Event,
) -> Outcome {
    if focus {
        state.handle(event, FocusKeys)
    } else {
        state.handle(event, MouseOnly)
    }
}

/// Handle only mouse-events.
pub fn handle_mouse_events(
    state: &mut FTableState<CellSelection>,
    event: &crossterm::event::Event,
) -> Outcome {
    state.handle(event, MouseOnly)
}
