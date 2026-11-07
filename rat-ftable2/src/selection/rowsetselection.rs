use crate::TableSelection;
use crate::event::TableOutcome;
use crate::table::TableState;
use rat_event::{HandleEvent, MouseOnly, Regular, ct_event, flow};
use rat_focus::HasFocus;
use rat_ftable::selection::RowSetSelection;
use rat_scrolled::ScrollAreaState;
use rat_scrolled::event::ScrollOutcome;
use ratatui_crossterm::crossterm::event::{Event, KeyModifiers};
use std::cmp::max;

impl HandleEvent<Event, Regular, TableOutcome> for TableState<RowSetSelection> {
    fn handle(&mut self, event: &Event, _: Regular) -> TableOutcome {
        let res = if self.is_focused() {
            match event {
                ct_event!(keycode press Up) => {
                    if self.move_up(1, false) {
                        TableOutcome::Selected
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press Down) => {
                    if self.move_down(1, false) {
                        TableOutcome::Selected
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press CONTROL-Up)
                | ct_event!(keycode press CONTROL-Home)
                | ct_event!(keycode press Home) => {
                    if self.move_to(0, false) {
                        TableOutcome::Selected
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press CONTROL-Down)
                | ct_event!(keycode press CONTROL-End)
                | ct_event!(keycode press End) => {
                    if self.move_to(self.rows.saturating_sub(1), false) {
                        TableOutcome::Selected
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press PageUp) => {
                    if self.move_up(max(1, self.page_len().saturating_sub(1)), false) {
                        TableOutcome::Selected
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press PageDown) => {
                    if self.move_down(max(1, self.page_len().saturating_sub(1)), false) {
                        TableOutcome::Selected
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press SHIFT-Up) => {
                    if self.move_up(1, true) {
                        TableOutcome::Selected
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press SHIFT-Down) => {
                    if self.move_down(1, true) {
                        TableOutcome::Selected
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press CONTROL_SHIFT-Up)
                | ct_event!(keycode press CONTROL_SHIFT-Home)
                | ct_event!(keycode press SHIFT-Home) => {
                    if self.move_to(0, true) {
                        TableOutcome::Selected
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press CONTROL_SHIFT-Down)
                | ct_event!(keycode press CONTROL_SHIFT-End)
                | ct_event!(keycode press SHIFT-End) => {
                    if self.move_to(self.rows.saturating_sub(1), true) {
                        TableOutcome::Selected
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press SHIFT-PageUp) => {
                    if self.move_up(max(1, self.page_len().saturating_sub(1)), true) {
                        TableOutcome::Selected
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press SHIFT-PageDown) => {
                    if self.move_down(max(1, self.page_len().saturating_sub(1)), true) {
                        TableOutcome::Selected
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

impl HandleEvent<Event, MouseOnly, TableOutcome> for TableState<RowSetSelection> {
    fn handle(&mut self, event: &Event, _: MouseOnly) -> TableOutcome {
        if self.is_obscured() {
            return TableOutcome::Continue;
        }

        flow!(match event {
            ct_event!(mouse any for m) | ct_event!(mouse any CONTROL for m)
                if self.mouse.drag(self.table_area, m)
                    || self.mouse.drag2(self.table_area, m, KeyModifiers::CONTROL) =>
            {
                if self.move_to(self.row_at_drag((m.column, m.row)), true) {
                    TableOutcome::Selected
                } else {
                    TableOutcome::Unchanged
                }
            }
            ct_event!(mouse down Left for column, row) => {
                let pos = (*column, *row);
                if self.table_area.contains(pos.into()) {
                    if let Some(new_row) = self.row_at_clicked(pos) {
                        if self.move_to(new_row, false) {
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
            ct_event!(mouse down ALT-Left for column, row) => {
                let pos = (*column, *row);
                if self.area.contains(pos.into()) {
                    if let Some(new_row) = self.row_at_clicked(pos) {
                        if self.move_to(new_row, true) {
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
            ct_event!(mouse down CONTROL-Left for column, row) => {
                let pos = (*column, *row);
                if self.area.contains(pos.into()) {
                    if let Some(new_row) = self.row_at_clicked(pos) {
                        self.retire_selection();
                        if self.selection.is_selected_row(new_row) {
                            self.selection.remove(new_row);
                        } else {
                            self.move_to(new_row, true);
                        }
                        TableOutcome::Selected
                    } else {
                        TableOutcome::Continue
                    }
                } else {
                    TableOutcome::Continue
                }
            }
            _ => TableOutcome::Continue,
        });

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
    }
}
