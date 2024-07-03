use crate::event::Outcome;
use crate::{FTableState, TableSelection};
use rat_event::{ct_event, flow, FocusKeys, HandleEvent, MouseOnly};
use rat_focus::HasFocusFlag;
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::ScrollArea;
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

    fn scroll_selected(&self) -> bool {
        self.scroll_selected
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

impl HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for FTableState<RowSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> Outcome {
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
                ct_event!(keycode press CONTROL-Left) => self.scroll_to_col(0).into(),
                ct_event!(keycode press CONTROL-Right) => {
                    self.scroll_to_col(self.col_max_offset()).into()
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
        flow!(match event {
            ct_event!(mouse any for m) if self.mouse.drag(self.table_area, m) => {
                self.move_to(self.row_at_drag((m.column, m.row))).into()
            }
            ct_event!(mouse down Left for column, row) => {
                if self.table_area.contains((*column, *row).into()) {
                    if let Some(new_row) = self.row_at_clicked((*column, *row)) {
                        self.move_to(new_row).into()
                    } else {
                        Outcome::NotUsed
                    }
                } else {
                    Outcome::NotUsed
                }
            }

            _ => Outcome::NotUsed,
        });

        let r = match ScrollArea(self.inner, Some(&mut self.hscroll), Some(&mut self.vscroll))
            .handle(event, MouseOnly)
        {
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
                    self.scroll_to_row(v)
                }
            }
            ScrollOutcome::Left(v) => self.scroll_left(v),
            ScrollOutcome::Right(v) => self.scroll_right(v),
            ScrollOutcome::HPos(v) => self.scroll_to_col(v),

            ScrollOutcome::NotUsed => false,
            ScrollOutcome::Unchanged => false,
            ScrollOutcome::Changed => true,
        };
        if r {
            return Outcome::Changed;
        }

        Outcome::NotUsed
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
    state.focus.set(focus);
    state.handle(event, FocusKeys)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(
    state: &mut FTableState<RowSelection>,
    event: &crossterm::event::Event,
) -> Outcome {
    state.handle(event, MouseOnly)
}
