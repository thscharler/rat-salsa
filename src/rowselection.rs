use crate::event::Outcome;
use crate::{FTableState, TableSelection};
use rat_event::{ct_event, FocusKeys, HandleEvent, MouseOnly};
use rat_focus::HasFocusFlag;
use rat_scrolled::ScrollingState;
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

    /// Clear the selection. Locked state is removed and
    /// lead_row set to None.
    pub fn clear(&mut self) {
        self.lead_row = None;
    }

    /// Scroll selection instead of offset.
    pub fn set_scroll_selected(&mut self, scroll: bool) {
        self.scroll_selected = scroll;
    }

    /// Scroll selection instead of offset.
    pub fn scroll_selected(&self) -> bool {
        self.scroll_selected
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
    pub fn select(&mut self, select: Option<usize>) -> bool {
        let old_row = self.lead_row;
        self.lead_row = select;
        old_row != self.lead_row
    }

    /// Select a row, clamp between 0 and maximum.
    pub fn select_clamped(&mut self, select: usize, maximum: usize) -> bool {
        let old_row = self.lead_row;
        self.lead_row = Some(min(select, maximum));
        old_row != self.lead_row
    }

    /// Select the next row, clamp between 0 and maximum.
    pub fn next(&mut self, n: usize, maximum: usize) -> bool {
        let old_row = self.lead_row;
        self.lead_row = Some(self.lead_row.map_or(0, |v| min(v + n, maximum)));
        old_row != self.lead_row
    }

    /// Select the previous row, clamp between 0 and maximum.
    pub fn prev(&mut self, n: usize) -> bool {
        let old_row = self.lead_row;
        self.lead_row = Some(self.lead_row.map_or(0, |v| v.saturating_sub(n)));
        old_row != self.lead_row
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for FTableState<RowSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> Outcome {
        let res = if self.is_focused() {
            match event {
                ct_event!(keycode press Down) => {
                    let r = self.selection.next(1, self.rows.saturating_sub(1)).into();
                    self.scroll_to_selected();
                    r
                }
                ct_event!(keycode press Up) => {
                    let r = self.selection.prev(1).into();
                    self.scroll_to_selected();
                    r
                }
                ct_event!(keycode press CONTROL-Down) | ct_event!(keycode press End) => {
                    let r = self
                        .selection
                        .select(Some(self.rows.saturating_sub(1)))
                        .into();
                    self.scroll_to_selected();
                    r
                }
                ct_event!(keycode press CONTROL-Up) | ct_event!(keycode press Home) => {
                    let r = self.selection.select(Some(0)).into();
                    self.scroll_to_selected();
                    r
                }
                ct_event!(keycode press PageUp) => {
                    let r = self
                        .selection
                        .prev(self.vertical_page().saturating_sub(1))
                        .into();
                    self.scroll_to_selected();
                    r
                }
                ct_event!(keycode press PageDown) => {
                    let r = self
                        .selection
                        .next(
                            self.vertical_page().saturating_sub(1),
                            self.rows.saturating_sub(1),
                        )
                        .into();
                    self.scroll_to_selected();
                    r
                }
                ct_event!(keycode press Right) => self.scroll_right(1).into(),
                ct_event!(keycode press Left) => self.scroll_left(1).into(),
                ct_event!(keycode press CONTROL-Right) | ct_event!(keycode press SHIFT-End) => {
                    self.set_horizontal_offset(self.max_col_offset).into()
                }
                ct_event!(keycode press CONTROL-Left) | ct_event!(keycode press SHIFT-Home) => {
                    self.set_horizontal_offset(0).into()
                }
                _ => Outcome::NotUsed,
            }
        } else {
            Outcome::NotUsed
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
            ct_event!(mouse any for m) if self.mouse.drag(self.table_area, m) => {
                let pos = (m.column, m.row);
                let new_row = self.row_at_drag(pos);
                let r = self
                    .selection
                    .select_clamped(new_row, self.rows.saturating_sub(1))
                    .into();
                self.scroll_to_selected();
                r
            }
            ct_event!(scroll down for column,row) => {
                if self.area.contains((*column, *row).into()) {
                    if self.selection.scroll_selected {
                        let r = self.selection.next(1, self.rows.saturating_sub(1));
                        self.scroll_to_selected();
                        r.into()
                    } else {
                        self.scroll_down(self.table_area.height as usize / 10)
                            .into()
                    }
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(scroll up for column, row) => {
                if self.area.contains((*column, *row).into()) {
                    if self.selection.scroll_selected {
                        let r = self.selection.prev(1);
                        self.scroll_to_selected();
                        r.into()
                    } else {
                        self.scroll_up(self.table_area.height as usize / 10).into()
                    }
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
                    if let Some(new_row) = self.row_at_clicked(pos) {
                        self.selection
                            .select_clamped(new_row, self.rows.saturating_sub(1))
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
    state: &mut FTableState<RowSelection>,
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
    state: &mut FTableState<RowSelection>,
    event: &crossterm::event::Event,
) -> Outcome {
    state.handle(event, MouseOnly)
}
