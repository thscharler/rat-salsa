use crate::event::Outcome;
use crate::{FTableState, TableSelection};
use rat_event::{ct_event, FocusKeys, HandleEvent, MouseOnly};
use ratatui::layout::Position;
use std::cmp::max;

#[derive(Debug, Default, Clone)]
pub struct NoSelection;

impl TableSelection for NoSelection {
    fn is_selected_row(&self, _row: usize) -> bool {
        false
    }

    fn is_selected_column(&self, _column: usize) -> bool {
        false
    }

    fn is_selected_cell(&self, _column: usize, _row: usize) -> bool {
        false
    }

    fn lead_selection(&self) -> Option<(usize, usize)> {
        None
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for FTableState<NoSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> Outcome {
        let res = match event {
            ct_event!(keycode press Down) => {
                self.scroll_down(1);
                Outcome::Changed
            }
            ct_event!(keycode press Up) => {
                self.scroll_up(1);
                Outcome::Changed
            }
            ct_event!(keycode press CONTROL-Down) | ct_event!(keycode press End) => {
                self.set_row_offset(self.max_row_offset);
                Outcome::Changed
            }
            ct_event!(keycode press CONTROL-Up) | ct_event!(keycode press Home) => {
                self.set_row_offset(0);
                Outcome::Changed
            }
            ct_event!(keycode press PageUp) => {
                self.scroll_up(self.row_page_len);
                Outcome::Changed
            }
            ct_event!(keycode press PageDown) => {
                self.scroll_down(self.row_page_len);
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
                self.set_column_offset(self.max_col_offset);
                Outcome::Changed
            }
            ct_event!(keycode press CONTROL-Left) | ct_event!(keycode press SHIFT-Home) => {
                self.set_column_offset(0);
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

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for FTableState<NoSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        match event {
            ct_event!(scroll down for column,row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.scroll_down(max(self.row_page_len / 10, 1));
                    Outcome::Changed
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(scroll up for column, row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.scroll_up(max(self.row_page_len / 10, 1));
                    Outcome::Changed
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(scroll ALT down for column,row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.scroll_down(1);
                    Outcome::Changed
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(scroll ALT up for column, row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.scroll_up(1);
                    Outcome::Changed
                } else {
                    Outcome::NotUsed
                }
            }
            _ => Outcome::NotUsed,
        }
    }
}
