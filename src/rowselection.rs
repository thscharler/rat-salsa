use crate::event::Outcome;
use crate::{FTableState, TableSelection};
use rat_event::{ct_event, FocusKeys, HandleEvent, MouseOnly};
use ratatui::layout::Position;
use std::cmp::min;

/// Allows selecting a single row of the table.
///
/// This is the right one if you want a list-style selection
/// for your table.
///
/// This one only supports row-selection.
#[derive(Debug, Default, Clone)]
pub struct RowSelection {
    /// Selected row.
    pub lead_row: Option<usize>,
}

impl TableSelection for RowSelection {
    fn is_selected_row(&self, row: usize) -> bool {
        self.lead_row == Some(row)
    }

    fn is_selected_column(&self, _column: usize) -> bool {
        false
    }

    fn is_selected_cell(&self, _column: usize, _row: usize) -> bool {
        false
    }

    fn lead_selection(&self) -> Option<(usize, usize)> {
        self.lead_row.map(|v| (0, v))
    }
}

impl RowSelection {
    /// New selection.
    pub fn new() -> RowSelection {
        Self::default()
    }

    /// The current selected row.
    pub fn selected(&self) -> Option<usize> {
        self.lead_row
    }

    /// Select a row.
    pub fn select(&mut self, select: Option<usize>) {
        self.lead_row = select;
    }

    /// Select a row, clamp between 0 and maximum.
    pub fn select_clamped(&mut self, select: usize, maximum: usize) {
        if select <= maximum {
            self.lead_row = Some(select);
        } else {
            self.lead_row = Some(maximum);
        }
    }

    /// Select the next row, clamp between 0 and maximum.
    pub fn next(&mut self, n: usize, maximum: usize) {
        self.lead_row = match self.lead_row {
            None => Some(0),
            Some(v) => Some(min(v + n, maximum)),
        };
    }

    /// Select the previous row, clamp between 0 and maximum.
    pub fn prev(&mut self, n: usize) {
        self.lead_row = match self.lead_row {
            None => Some(0),
            Some(v) => {
                if v >= n {
                    Some(v - n)
                } else {
                    Some(0)
                }
            }
        };
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for FTableState<RowSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> Outcome {
        let res = match event {
            ct_event!(keycode press Down) => {
                self.selection.next(1, self.rows - 1);
                self.scroll_to_selected();
                Outcome::Changed
            }
            ct_event!(keycode press Up) => {
                self.selection.prev(1);
                self.scroll_to_selected();
                Outcome::Changed
            }
            ct_event!(keycode press CONTROL-Down) | ct_event!(keycode press End) => {
                self.selection.select(Some(self.rows - 1));
                self.scroll_to_selected();
                Outcome::Changed
            }
            ct_event!(keycode press CONTROL-Up) | ct_event!(keycode press Home) => {
                self.selection.select(Some(0));
                self.scroll_to_selected();
                Outcome::Changed
            }
            ct_event!(keycode press PageUp) => {
                self.selection.prev(self.table_area.height as usize);
                self.scroll_to_selected();
                Outcome::Changed
            }
            ct_event!(keycode press PageDown) => {
                self.selection
                    .next(self.table_area.height as usize, self.rows - 1);
                self.scroll_to_selected();
                Outcome::Changed
            }
            ct_event!(keycode press Right) => {
                self.scroll_right(1);
                Outcome::Changed
            }
            ct_event!(keycode press Left) => {
                self.scroll_left(1);
                Outcome::Changed
            }
            ct_event!(keycode press CONTROL-Right) | ct_event!(keycode press SHIFT-End) => {
                self.set_horizontal_offset(self.max_col_offset);
                Outcome::Changed
            }
            ct_event!(keycode press CONTROL-Left) | ct_event!(keycode press SHIFT-Home) => {
                self.set_horizontal_offset(0);
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

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for FTableState<RowSelection> {
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
                    if let Some(new_row) = self.row_at_clicked(pos) {
                        self.mouse.set_drag();
                        self.selection.select_clamped(new_row, self.rows - 1);
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
                    let new_row = self.row_at_drag(pos);
                    self.mouse.set_drag();
                    self.selection.select_clamped(new_row, self.rows - 1);
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
