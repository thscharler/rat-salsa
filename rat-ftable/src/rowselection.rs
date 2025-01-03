use crate::event::Outcome;
use crate::{TableSelection, TableState};
use rat_event::{ct_event, flow, HandleEvent, MouseOnly, Regular};
use rat_focus::HasFocus;
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::ScrollAreaState;
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

    /// Update the state to match adding items.
    pub fn items_added(&mut self, pos: usize, n: usize) {
        if let Some(lead_row) = self.lead_row {
            if lead_row > pos {
                self.lead_row = Some(lead_row + n);
            }
        }
    }

    /// Update the state to match removing items.
    ///
    /// This will leave the selection at 0 after the
    /// last item has been removed.
    pub fn items_removed(&mut self, pos: usize, n: usize, maximum: usize) {
        if let Some(lead_row) = self.lead_row {
            if lead_row > pos {
                self.lead_row = Some(lead_row.saturating_sub(n));
            } else if lead_row == pos && lead_row == maximum {
                self.lead_row = Some(lead_row.saturating_sub(1));
            }
        }
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

impl HandleEvent<crossterm::event::Event, Regular, Outcome> for TableState<RowSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: Regular) -> Outcome {
        let res = if self.is_focused() {
            match event {
                ct_event!(keycode press Up) => self.move_up(1).into(),
                ct_event!(keycode press Down) => self.move_down(1).into(),
                ct_event!(keycode press CONTROL-Up)
                | ct_event!(keycode press CONTROL-Home)
                | ct_event!(keycode press Home) => self.move_to(0).into(),
                ct_event!(keycode press CONTROL-Down)
                | ct_event!(keycode press CONTROL-End)
                | ct_event!(keycode press End) => self.move_to(self.rows.saturating_sub(1)).into(),

                ct_event!(keycode press PageUp) => self
                    .move_up(max(1, self.page_len().saturating_sub(1)))
                    .into(),
                ct_event!(keycode press PageDown) => self
                    .move_down(max(1, self.page_len().saturating_sub(1)))
                    .into(),

                ct_event!(keycode press Left) => self.scroll_left(1).into(),
                ct_event!(keycode press Right) => self.scroll_right(1).into(),
                ct_event!(keycode press CONTROL-Left) => self.scroll_to_x(0).into(),
                ct_event!(keycode press CONTROL-Right) => {
                    self.scroll_to_x(self.x_max_offset()).into()
                }
                _ => Outcome::Continue,
            }
        } else {
            Outcome::Continue
        };

        if res == Outcome::Continue {
            self.handle(event, MouseOnly)
        } else {
            res
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for TableState<RowSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        flow!(match event {
            ct_event!(mouse any for m) if self.mouse.drag(self.table_area, m) => {
                self.move_to(self.row_at_drag((m.column, m.row))).into()
            }
            ct_event!(mouse down Left for column, row) => {
                if self.table_area.contains((*column, *row).into()) {
                    if let Some(new_row) = self.row_at_clicked((*column, *row)) {
                        self.move_to(new_row).into()
                    } else {
                        Outcome::Continue
                    }
                } else {
                    Outcome::Continue
                }
            }

            _ => Outcome::Continue,
        });

        let mut sas = ScrollAreaState::new()
            .area(self.inner)
            .h_scroll(&mut self.hscroll)
            .v_scroll(&mut self.vscroll);
        let r = match sas.handle(event, MouseOnly) {
            ScrollOutcome::Up(v) => {
                if self.selection.scroll_selected() {
                    self.move_up(1)
                } else {
                    self.scroll_up(v)
                }
            }
            ScrollOutcome::Down(v) => {
                if self.selection.scroll_selected() {
                    self.move_down(1)
                } else {
                    self.scroll_down(v)
                }
            }
            ScrollOutcome::VPos(v) => {
                if self.selection.scroll_selected {
                    self.move_to(self.remap_offset_selection(v))
                } else {
                    self.set_row_offset(v)
                }
            }
            ScrollOutcome::Left(v) => self.scroll_left(v),
            ScrollOutcome::Right(v) => self.scroll_right(v),
            ScrollOutcome::HPos(v) => self.set_x_offset(v),

            ScrollOutcome::Continue => false,
            ScrollOutcome::Unchanged => false,
            ScrollOutcome::Changed => true,
        };
        if r {
            return Outcome::Changed;
        }

        Outcome::Continue
    }
}

/// Handle all events.
/// Table events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events(
    state: &mut TableState<RowSelection>,
    focus: bool,
    event: &crossterm::event::Event,
) -> Outcome {
    state.focus.set(focus);
    state.handle(event, Regular)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(
    state: &mut TableState<RowSelection>,
    event: &crossterm::event::Event,
) -> Outcome {
    state.handle(event, MouseOnly)
}
