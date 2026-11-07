use crate::event::TableOutcome;
use crate::table::TableState;
use rat_event::{ConsumedEvent, HandleEvent, MouseOnly, Regular, ct_event};
use rat_focus::HasFocus;
use rat_ftable::selection::CellSelection;
use rat_scrolled::ScrollAreaState;
use rat_scrolled::event::ScrollOutcome;
use ratatui_crossterm::crossterm::event::Event;
use std::cmp::max;

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
        if self.is_obscured() {
            return TableOutcome::Continue;
        }

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
                } else if self.header_area.contains((*column, *row).into()) {
                    if let Some(col) = self.column_at_clicked((*column, *row)) {
                        TableOutcome::Header(col)
                    } else {
                        TableOutcome::Continue
                    }
                } else if self.footer_area.contains((*column, *row).into()) {
                    if let Some(col) = self.column_at_clicked((*column, *row)) {
                        TableOutcome::Footer(col)
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
