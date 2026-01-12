use crate::event::TableOutcome;
use crate::{TableSelection, TableState};
use rat_event::{HandleEvent, MouseOnly, Regular, ct_event};
use rat_focus::HasFocus;
use rat_scrolled::ScrollAreaState;
use rat_scrolled::event::ScrollOutcome;
use ratatui_crossterm::crossterm::event::Event;
use std::cmp::max;

/// Doesn't do any selection for the table.
///
/// But it implements scrolling via mouse and keyboard.
#[derive(Debug, Default, Clone)]
pub struct NoSelection;

impl TableSelection for NoSelection {
    fn count(&self) -> usize {
        0
    }

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

    fn validate_rows(&mut self, _rows: usize) {}

    fn validate_cols(&mut self, _cols: usize) {}

    fn items_added(&mut self, _pos: usize, _n: usize) {}

    fn items_removed(&mut self, _pos: usize, _n: usize, _rows: usize) {}
}

impl HandleEvent<Event, Regular, TableOutcome> for TableState<NoSelection> {
    fn handle(&mut self, event: &Event, _keymap: Regular) -> TableOutcome {
        let res = if self.is_focused() {
            match event {
                ct_event!(keycode press Up) => {
                    if self.scroll_up(1) {
                        TableOutcome::Changed
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press Down) => {
                    if self.scroll_down(1) {
                        TableOutcome::Changed
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press CONTROL-Up)
                | ct_event!(keycode press CONTROL-Home)
                | ct_event!(keycode press Home) => {
                    if self.scroll_to_row(0) {
                        TableOutcome::Changed
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press CONTROL-Down)
                | ct_event!(keycode press CONTROL-End)
                | ct_event!(keycode press End) => {
                    if self.scroll_to_row(self.rows.saturating_sub(1)) {
                        TableOutcome::Changed
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press PageUp) => {
                    if self.scroll_up(max(1, self.page_len().saturating_sub(1))) {
                        TableOutcome::Changed
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press PageDown) => {
                    if self.scroll_down(max(1, self.page_len().saturating_sub(1))) {
                        TableOutcome::Changed
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press Left) => {
                    if self.scroll_left(1) {
                        TableOutcome::Changed
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press Right) => {
                    if self.scroll_right(1) {
                        TableOutcome::Changed
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press CONTROL-Left) => {
                    if self.scroll_to_x(0) {
                        TableOutcome::Changed
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press CONTROL-Right) => {
                    if self.scroll_to_x(self.x_max_offset()) {
                        TableOutcome::Changed
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

impl HandleEvent<Event, MouseOnly, TableOutcome> for TableState<NoSelection> {
    fn handle(&mut self, event: &Event, _keymap: MouseOnly) -> TableOutcome {
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
                if self.set_row_offset(v) {
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
                if self.set_x_offset(v) {
                    TableOutcome::Changed
                } else {
                    TableOutcome::Unchanged
                }
            }
            ScrollOutcome::Continue => TableOutcome::Continue,
            ScrollOutcome::Unchanged => TableOutcome::Unchanged,
            ScrollOutcome::Changed => TableOutcome::Changed,
        }
    }
}

/// Handle all events.
/// Table events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events(
    state: &mut TableState<NoSelection>,
    focus: bool,
    event: &Event,
) -> TableOutcome {
    state.focus.set(focus);
    state.handle(event, Regular)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(state: &mut TableState<NoSelection>, event: &Event) -> TableOutcome {
    state.handle(event, MouseOnly)
}
