use crate::event::Outcome;
use crate::{FTableState, TableSelection};
use rat_event::{ct_event, FocusKeys, HandleEvent, MouseOnly};
use ratatui::layout::Position;
use std::cmp::min;

/// Select a single cell in the table.
///
/// This one supports cell + column + row selection.
#[derive(Debug, Default, Clone)]
pub struct CellSelection {
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

    /// Selected cell.
    pub fn selected(&self) -> Option<(usize, usize)> {
        self.lead_cell
    }

    /// Select a cell.
    pub fn select_cell(&mut self, select: Option<(usize, usize)>) {
        self.lead_cell = select;
    }

    /// Select a row. Column stays the same.
    pub fn select_row(&mut self, select: Option<usize>) {
        self.lead_cell = match self.lead_cell {
            None => select.map(|v| (0, v)),
            Some((scol, _)) => select.map(|v| (scol, v)),
        }
    }

    /// Select a column, row stays the same.
    pub fn select_column(&mut self, select: Option<usize>) {
        self.lead_cell = match self.lead_cell {
            None => select.map(|v| (v, 0)),
            Some((_, srow)) => select.map(|v| (v, srow)),
        }
    }

    /// Select a cell, clamp between 0 and maximum.
    pub fn select_clamped(&mut self, select: (usize, usize), maximum: (usize, usize)) {
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

        self.lead_cell = Some((col, row))
    }

    /// Select the next row, clamp between 0 and maximum.
    pub fn next_row(&mut self, n: usize, maximum: usize) {
        self.lead_cell = match self.lead_cell {
            None => Some((0, 0)),
            Some((scol, srow)) => Some((scol, min(srow + n, maximum))),
        };
    }

    /// Select the previous row, clamp between 0 and maximum.
    pub fn prev_row(&mut self, n: usize) {
        self.lead_cell = match self.lead_cell {
            None => Some((0, 0)),
            Some((scol, srow)) => Some((scol, if srow >= n { srow - n } else { 0 })),
        };
    }

    /// Select the next column, clamp between 0 and maximum.
    pub fn next_column(&mut self, n: usize, maximum: usize) {
        self.lead_cell = match self.lead_cell {
            None => Some((0, 0)),
            Some((scol, srow)) => Some((min(scol + n, maximum), srow)),
        };
    }

    /// Select the previous row, clamp between 0 and maximum.
    pub fn prev_column(&mut self, n: usize) {
        self.lead_cell = match self.lead_cell {
            None => Some((0, 0)),
            Some((scol, srow)) => Some((if scol >= n { scol - n } else { 0 }, srow)),
        };
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for FTableState<CellSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> Outcome {
        let res = match event {
            ct_event!(keycode press Down) => {
                self.selection.next_row(1, self.rows - 1);
                self.scroll_to_selected();
                Outcome::Changed
            }
            ct_event!(keycode press Up) => {
                self.selection.prev_row(1);
                self.scroll_to_selected();
                Outcome::Changed
            }
            ct_event!(keycode press CONTROL-Down) | ct_event!(keycode press End) => {
                self.selection.select_row(Some(self.rows - 1));
                self.scroll_to_selected();
                Outcome::Changed
            }
            ct_event!(keycode press CONTROL-Up) | ct_event!(keycode press Home) => {
                self.selection.select_row(Some(0));
                self.scroll_to_selected();
                Outcome::Changed
            }
            ct_event!(keycode press PageUp) => {
                self.selection.prev_row(self.table_area.height as usize);
                self.scroll_to_selected();
                Outcome::Changed
            }
            ct_event!(keycode press PageDown) => {
                self.selection
                    .next_row(self.table_area.height as usize, self.rows - 1);
                self.scroll_to_selected();
                Outcome::Changed
            }
            ct_event!(keycode press Right) => {
                self.selection.next_column(1, self.columns - 1);
                self.scroll_to_selected();
                Outcome::Changed
            }
            ct_event!(keycode press Left) => {
                self.selection.prev_column(1);
                self.scroll_to_selected();
                Outcome::Changed
            }
            ct_event!(keycode press CONTROL-Right) | ct_event!(keycode press SHIFT-End) => {
                self.selection.select_column(Some(self.columns - 1));
                self.scroll_to_selected();
                Outcome::Changed
            }
            ct_event!(keycode press CONTROL-Left) | ct_event!(keycode press SHIFT-Home) => {
                self.selection.select_column(Some(0));
                self.scroll_to_selected();
                Outcome::Changed
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
            ct_event!(scroll down for column,row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.scroll_down(self.table_area.height as usize / 10);
                    Outcome::Changed
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(scroll up for column, row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.scroll_up(self.table_area.height as usize / 10);
                    Outcome::Changed
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(scroll ALT down for column,row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.scroll_right(1);
                    Outcome::Changed
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(scroll ALT up for column, row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.scroll_left(1);
                    Outcome::Changed
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(mouse down Left for column, row) => {
                let pos = Position::new(*column, *row);
                if self.area.contains(pos) {
                    if let Some(new_cell) = self.cell_at_clicked(pos) {
                        self.mouse.set_drag();
                        self.selection
                            .select_clamped(new_cell, (self.columns - 1, self.rows - 1));
                        Outcome::Changed
                    } else {
                        Outcome::Unchanged
                    }
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(mouse drag Left for column, row) => {
                if self.mouse.do_drag() {
                    let pos = Position::new(*column, *row);
                    let new_cell = self.cell_at_drag(pos);
                    self.selection
                        .select_clamped(new_cell, (self.columns - 1, self.rows - 1));
                    self.scroll_to_selected();
                    Outcome::Changed
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(mouse moved) => {
                self.mouse.clear_drag();
                Outcome::NotUsed
            }

            _ => Outcome::NotUsed,
        }
    }
}
