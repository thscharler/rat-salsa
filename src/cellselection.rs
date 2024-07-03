use crate::event::Outcome;
use crate::{FTableState, TableSelection};
use rat_event::{ct_event, flow, FocusKeys, HandleEvent, MouseOnly};
use rat_focus::HasFocusFlag;
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::ScrollArea;
use std::cmp::{max, min};

/// Select a single cell in the table.
///
/// This one supports cell + column + row selection.
#[derive(Debug, Default, Clone)]
pub struct CellSelection {
    /// Selected cell.
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

    /// Clear the selection.
    #[inline]
    pub fn clear(&mut self) {
        self.lead_cell = None;
    }

    /// Selected cell.
    pub fn selected(&self) -> Option<(usize, usize)> {
        self.lead_cell
    }

    #[inline]
    pub fn has_selection(&mut self) -> bool {
        self.lead_cell.is_some()
    }

    /// Select a cell.
    pub fn select_cell(&mut self, select: Option<(usize, usize)>) -> bool {
        let old_cell = self.lead_cell;
        self.lead_cell = select;
        old_cell != self.lead_cell
    }

    /// Select a row. Column stays the same.
    pub fn select_row(&mut self, select: Option<usize>) -> bool {
        let old_cell = self.lead_cell;
        self.lead_cell = match self.lead_cell {
            None => select.map(|v| (0, v)),
            Some((scol, _)) => select.map(|v| (scol, v)),
        };
        old_cell != self.lead_cell
    }

    /// Select a column, row stays the same.
    pub fn select_column(&mut self, select: Option<usize>) -> bool {
        let old_cell = self.lead_cell;
        self.lead_cell = match self.lead_cell {
            None => select.map(|v| (v, 0)),
            Some((_, srow)) => select.map(|v| (v, srow)),
        };
        old_cell != self.lead_cell
    }

    /// Select a cell, clamp between 0 and maximum.
    pub fn move_to(&mut self, select: (usize, usize), maximum: (usize, usize)) -> bool {
        let c = self.move_to_col(select.0, maximum.0);
        let r = self.move_to_row(select.1, maximum.1);
        c || r
    }

    /// Select a column. Row stays the same.
    pub fn move_to_col(&mut self, col: usize, maximum: usize) -> bool {
        let old = self.lead_cell;
        let col = min(col, maximum);
        self.lead_cell = self
            .lead_cell
            .map_or(Some((col, 0)), |(_, srow)| Some((col, srow)));
        old != self.lead_cell
    }

    /// Select a row. Column stays the same.
    pub fn move_to_row(&mut self, row: usize, maximum: usize) -> bool {
        let old = self.lead_cell;
        let row = min(row, maximum);
        self.lead_cell = self
            .lead_cell
            .map_or(Some((0, row)), |(scol, _)| Some((scol, row)));
        old != self.lead_cell
    }

    /// Select the next row, clamp between 0 and maximum.
    pub fn move_down(&mut self, n: usize, maximum: usize) -> bool {
        let old_cell = self.lead_cell;
        self.lead_cell = match self.lead_cell {
            None => Some((0, 0)),
            Some((scol, srow)) => Some((scol, min(srow + n, maximum))),
        };
        old_cell != self.lead_cell
    }

    /// Select the previous row, clamp between 0 and maximum.
    pub fn move_up(&mut self, n: usize, maximum: usize) -> bool {
        let old_cell = self.lead_cell;
        self.lead_cell = match self.lead_cell {
            None => Some((0, maximum)),
            Some((scol, srow)) => Some((scol, srow.saturating_sub(n))),
        };
        old_cell != self.lead_cell
    }

    /// Select the next column, clamp between 0 and maximum.
    pub fn move_right(&mut self, n: usize, maximum: usize) -> bool {
        let old_cell = self.lead_cell;
        self.lead_cell = match self.lead_cell {
            None => Some((0, 0)),
            Some((scol, srow)) => Some((min(scol + n, maximum), srow)),
        };
        old_cell != self.lead_cell
    }

    /// Select the previous row, clamp between 0 and maximum.
    pub fn move_left(&mut self, n: usize, maximum: usize) -> bool {
        let old_cell = self.lead_cell;
        self.lead_cell = match self.lead_cell {
            None => Some((maximum, 0)),
            Some((scol, srow)) => Some((scol.saturating_sub(n), srow)),
        };
        old_cell != self.lead_cell
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for FTableState<CellSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> Outcome {
        let res = if self.is_focused() {
            match event {
                ct_event!(keycode press Up) => self.move_up(1).into(),
                ct_event!(keycode press Down) => self.move_down(1).into(),
                ct_event!(keycode press CONTROL-Up) | ct_event!(keycode press CONTROL-Home) => {
                    self.move_to_row(0).into()
                }
                ct_event!(keycode press CONTROL-Down) | ct_event!(keycode press CONTROL-End) => {
                    self.move_to_row(self.rows.saturating_sub(1)).into()
                }

                ct_event!(keycode press PageUp) => self
                    .move_up(max(1, self.page_len().saturating_sub(1)))
                    .into(),
                ct_event!(keycode press PageDown) => self
                    .move_down(max(1, self.page_len().saturating_sub(1)))
                    .into(),

                ct_event!(keycode press Left) => self.move_left(1).into(),
                ct_event!(keycode press Right) => self.move_right(1).into(),
                ct_event!(keycode press CONTROL-Left) | ct_event!(keycode press Home) => {
                    self.move_to_col(0).into()
                }
                ct_event!(keycode press CONTROL-Right) | ct_event!(keycode press End) => {
                    self.move_to_col(self.columns.saturating_sub(1)).into()
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

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for FTableState<CellSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        flow!(match event {
            ct_event!(mouse any for m) if self.mouse.drag(self.table_area, m) => {
                self.move_to(self.cell_at_drag((m.column, m.row))).into()
            }
            ct_event!(mouse down Left for column, row) => {
                if self.area.contains((*column, *row).into()) {
                    if let Some(new_cell) = self.cell_at_clicked((*column, *row)) {
                        self.move_to(new_cell).into()
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
            ScrollOutcome::Up(v) => self.scroll_up(v),
            ScrollOutcome::Down(v) => self.scroll_down(v),
            ScrollOutcome::VPos(v) => self.scroll_to_row(v),
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

        Outcome::Unchanged
    }
}

/// Handle all events.
/// Table events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events(
    state: &mut FTableState<CellSelection>,
    focus: bool,
    event: &crossterm::event::Event,
) -> Outcome {
    state.focus.set(focus);
    state.handle(event, FocusKeys)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(
    state: &mut FTableState<CellSelection>,
    event: &crossterm::event::Event,
) -> Outcome {
    state.handle(event, MouseOnly)
}
