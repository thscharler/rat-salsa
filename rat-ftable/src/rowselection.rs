use crate::event::TableOutcome;
use crate::{TableSelection, TableState};
use rat_event::{ConsumedEvent, HandleEvent, MouseOnly, Regular, ct_event};
use rat_focus::HasFocus;
use rat_scrolled::ScrollAreaState;
use rat_scrolled::event::ScrollOutcome;
use ratatui_crossterm::crossterm::event::Event;
use std::cmp::{max, min};

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
    /// Scrolls the selection instead of the offset.
    pub scroll_selected: bool,
}

impl TableSelection for RowSelection {
    fn count(&self) -> usize {
        if self.lead_row.is_some() { 1 } else { 0 }
    }

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

    fn validate_rows(&mut self, rows: usize) {
        if let Some(lead_row) = self.lead_row {
            if rows == 0 {
                self.lead_row = None;
            } else if lead_row >= rows {
                self.lead_row = Some(rows - 1);
            }
        }
    }

    fn validate_cols(&mut self, _cols: usize) {}

    /// Update the state to match adding items.
    #[allow(clippy::collapsible_if)]
    fn items_added(&mut self, pos: usize, n: usize) {
        if let Some(lead_row) = self.lead_row {
            if lead_row > pos {
                self.lead_row = Some(lead_row.saturating_add(n));
            }
        }
    }

    /// Update the state to match removing items.
    fn items_removed(&mut self, pos: usize, n: usize, rows: usize) {
        if let Some(lead_row) = self.lead_row {
            if rows == 0 {
                self.lead_row = None;
            } else if lead_row == pos && lead_row + n >= rows {
                self.lead_row = Some(rows.saturating_sub(1))
            } else if lead_row > pos {
                self.lead_row = Some(lead_row.saturating_sub(n).min(pos));
            }
        }
    }
}

impl RowSelection {
    /// New selection.
    pub fn new() -> RowSelection {
        Self::default()
    }

    /// Clear the selection.
    pub fn clear(&mut self) {
        self.lead_row = None;
    }

    /// Scroll selection instead of offset.
    pub fn scroll_selected(&self) -> bool {
        self.scroll_selected
    }

    /// Scroll selection instead of offset.
    pub fn set_scroll_selected(&mut self, scroll: bool) {
        self.scroll_selected = scroll;
    }

    /// The current selected row.
    pub fn selected(&self) -> Option<usize> {
        self.lead_row
    }

    /// Has some selection.
    pub fn has_selection(&self) -> bool {
        self.lead_row.is_some()
    }

    /// Select a row.
    /// This function doesn't care if the given row actually exists in the table.
    pub fn select(&mut self, select: Option<usize>) -> bool {
        let old_row = self.lead_row;
        self.lead_row = select;
        old_row != self.lead_row
    }

    /// Select the given row, limit between 0 and maximum.
    pub fn move_to(&mut self, select: usize, maximum: usize) -> bool {
        let old_row = self.lead_row;
        self.lead_row = Some(min(select, maximum));
        old_row != self.lead_row
    }

    /// Select the next row, cap at maximum.
    pub fn move_down(&mut self, n: usize, maximum: usize) -> bool {
        let old_row = self.lead_row;
        self.lead_row = Some(self.lead_row.map_or(0, |v| min(v + n, maximum)));
        old_row != self.lead_row
    }

    /// Select the previous row.
    pub fn move_up(&mut self, n: usize, maximum: usize) -> bool {
        let old_row = self.lead_row;
        self.lead_row = Some(self.lead_row.map_or(maximum, |v| v.saturating_sub(n)));
        old_row != self.lead_row
    }
}

impl HandleEvent<Event, Regular, TableOutcome> for TableState<RowSelection> {
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
                ct_event!(keycode press CONTROL-Up)
                | ct_event!(keycode press CONTROL-Home)
                | ct_event!(keycode press Home) => {
                    if self.move_to(0) {
                        TableOutcome::Selected
                    } else {
                        TableOutcome::Unchanged
                    }
                }
                ct_event!(keycode press CONTROL-Down)
                | ct_event!(keycode press CONTROL-End)
                | ct_event!(keycode press End) => {
                    if self.move_to(self.rows.saturating_sub(1)) {
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

impl HandleEvent<Event, MouseOnly, TableOutcome> for TableState<RowSelection> {
    fn handle(&mut self, event: &Event, _keymap: MouseOnly) -> TableOutcome {
        let mut r = match event {
            ct_event!(mouse any for m) if self.mouse.drag(self.table_area, m) => {
                if self.move_to(self.row_at_drag((m.column, m.row))) {
                    TableOutcome::Selected
                } else {
                    TableOutcome::Unchanged
                }
            }
            ct_event!(mouse down Left for column, row) => {
                if self.table_area.contains((*column, *row).into()) {
                    if let Some(new_row) = self.row_at_clicked((*column, *row)) {
                        if self.move_to(new_row) {
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
                    if self.selection.scroll_selected() {
                        if self.move_up(1) {
                            TableOutcome::Selected
                        } else {
                            TableOutcome::Unchanged
                        }
                    } else {
                        if self.scroll_up(v) {
                            TableOutcome::Changed
                        } else {
                            TableOutcome::Unchanged
                        }
                    }
                }
                ScrollOutcome::Down(v) => {
                    if self.selection.scroll_selected() {
                        if self.move_down(1) {
                            TableOutcome::Selected
                        } else {
                            TableOutcome::Unchanged
                        }
                    } else {
                        if self.scroll_down(v) {
                            TableOutcome::Changed
                        } else {
                            TableOutcome::Unchanged
                        }
                    }
                }
                ScrollOutcome::VPos(v) => {
                    if self.selection.scroll_selected {
                        if self.move_to(self.remap_offset_selection(v)) {
                            TableOutcome::Selected
                        } else {
                            TableOutcome::Unchanged
                        }
                    } else {
                        if self.set_row_offset(self.vscroll.limited_offset(v)) {
                            TableOutcome::Changed
                        } else {
                            TableOutcome::Unchanged
                        }
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
    state: &mut TableState<RowSelection>,
    focus: bool,
    event: &Event,
) -> TableOutcome {
    state.focus.set(focus);
    state.handle(event, Regular)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(state: &mut TableState<RowSelection>, event: &Event) -> TableOutcome {
    state.handle(event, MouseOnly)
}
