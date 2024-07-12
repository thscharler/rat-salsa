use crate::event::Outcome;
use crate::{TableSelection, TableState};
use crossterm::event::KeyModifiers;
use rat_event::{ct_event, flow, HandleEvent, MouseOnly, Regular};
use rat_focus::HasFocusFlag;
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::ScrollArea;
use std::cmp::{max, min};
use std::collections::HashSet;
use std::mem;

/// Allows selection an active range of rows.
///
/// The current range can be retired to a set of selected rows,
/// and a new range be started. This allows multiple interval
/// selection and deselection of certain rows.
///
/// This one only supports row-selection.
#[derive(Debug, Default, Clone)]
pub struct RowSetSelection {
    /// Start of the active selection.
    pub anchor_row: Option<usize>,
    /// Current end of the active selection.
    pub lead_row: Option<usize>,
    /// Retired rows. This doesn't contain the rows
    /// between anchor and lead.
    ///
    /// You can call [RowSetSelection::retire_selection] to
    /// add the anchor-lead range. This resets anchor and lead though.
    /// Or iterate the complete range and call [RowSetSelection::is_selected_row].
    pub selected: HashSet<usize>,
}

impl TableSelection for RowSetSelection {
    #[allow(clippy::collapsible_else_if)]
    fn is_selected_row(&self, row: usize) -> bool {
        if let Some(mut anchor) = self.anchor_row {
            if let Some(mut lead) = self.lead_row {
                if lead < anchor {
                    mem::swap(&mut lead, &mut anchor);
                }
                if row >= anchor && row <= lead {
                    return true;
                }
            }
        } else {
            if let Some(lead) = self.lead_row {
                if row == lead {
                    return true;
                }
            }
        }

        self.selected.contains(&row)
    }

    fn is_selected_column(&self, _column: usize) -> bool {
        false
    }

    fn is_selected_cell(&self, _column: usize, _row: usize) -> bool {
        false
    }

    fn lead_selection(&self) -> Option<(usize, usize)> {
        self.lead_row.map(|srow| (0, srow))
    }
}

impl RowSetSelection {
    /// New selection.
    pub fn new() -> RowSetSelection {
        RowSetSelection {
            anchor_row: None,
            lead_row: None,
            selected: HashSet::new(),
        }
    }

    /// Clear the selection.
    pub fn clear(&mut self) {
        self.anchor_row = None;
        self.lead_row = None;
        self.selected.clear();
    }

    /// Current lead.
    pub fn lead(&self) -> Option<usize> {
        self.lead_row
    }

    /// Current anchor.
    pub fn anchor(&self) -> Option<usize> {
        self.anchor_row
    }

    /// Set of all selected rows. Clones the retired set and adds the current anchor..lead range.
    pub fn selected(&self) -> HashSet<usize> {
        let mut selected = self.selected.clone();
        Self::fill(self.anchor_row, self.lead_row, &mut selected);
        selected
    }

    /// Has some selection.
    pub fn has_selection(&self) -> bool {
        self.lead_row.is_some() || !self.selected.is_empty()
    }

    /// Set a new lead. Maybe extend the range.
    pub fn set_lead(&mut self, lead: Option<usize>, extend: bool) -> bool {
        let old_selection = (self.anchor_row, self.lead_row);
        self.extend(extend);
        self.lead_row = lead;
        old_selection != (self.anchor_row, self.lead_row)
    }

    /// Transfers the range anchor to lead to the selection set and reset both.
    pub fn retire_selection(&mut self) {
        Self::fill(self.anchor_row, self.lead_row, &mut self.selected);
        self.anchor_row = None;
        self.lead_row = None;
    }

    /// Add to selection. Only works for retired selections, not for the
    /// active anchor-lead range.
    pub fn add(&mut self, idx: usize) {
        self.selected.insert(idx);
    }

    /// Remove from selection. Only works for retired selections, not for the
    /// active anchor-lead range.
    pub fn remove(&mut self, idx: usize) {
        self.selected.remove(&idx);
    }

    /// Set a new lead, at the same time limit the lead to max.
    pub fn move_to(&mut self, lead: usize, max: usize, extend: bool) -> bool {
        let old_selection = (self.anchor_row, self.lead_row);
        self.extend(extend);
        if lead <= max {
            self.lead_row = Some(lead);
        } else {
            self.lead_row = Some(max);
        }
        old_selection != (self.anchor_row, self.lead_row)
    }

    /// Select next. Maybe extend the range.
    pub fn move_down(&mut self, n: usize, maximum: usize, extend: bool) -> bool {
        let old_selection = (self.anchor_row, self.lead_row);
        self.extend(extend);
        self.lead_row = Some(self.lead_row.map_or(0, |v| min(v + n, maximum)));
        old_selection != (self.anchor_row, self.lead_row)
    }

    /// Select next. Maybe extend the range.
    pub fn move_up(&mut self, n: usize, maximum: usize, extend: bool) -> bool {
        let old_selection = (self.anchor_row, self.lead_row);
        self.extend(extend);
        self.lead_row = Some(self.lead_row.map_or(maximum, |v| v.saturating_sub(n)));
        old_selection != (self.anchor_row, self.lead_row)
    }

    fn extend(&mut self, extend: bool) {
        if extend {
            if self.anchor_row.is_none() {
                self.anchor_row = self.lead_row;
            }
        } else {
            self.anchor_row = None;
            self.selected.clear();
        }
    }

    #[allow(clippy::collapsible_else_if)]
    fn fill(anchor: Option<usize>, lead: Option<usize>, selection: &mut HashSet<usize>) {
        if let Some(mut anchor) = anchor {
            if let Some(mut lead) = lead {
                if lead < anchor {
                    mem::swap(&mut lead, &mut anchor);
                }

                for n in anchor..=lead {
                    selection.insert(n);
                }
            }
        } else {
            if let Some(lead) = lead {
                selection.insert(lead);
            }
        }
    }
}

impl HandleEvent<crossterm::event::Event, Regular, Outcome> for TableState<RowSetSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _: Regular) -> Outcome {
        let res = if self.is_focused() {
            match event {
                ct_event!(keycode press Up) => self.move_up(1, false).into(),
                ct_event!(keycode press Down) => self.move_down(1, false).into(),
                ct_event!(keycode press CONTROL-Up)
                | ct_event!(keycode press CONTROL-Home)
                | ct_event!(keycode press Home) => self.move_to(0, false).into(),
                ct_event!(keycode press CONTROL-Down)
                | ct_event!(keycode press CONTROL-End)
                | ct_event!(keycode press End) => {
                    self.move_to(self.rows.saturating_sub(1), false).into()
                }
                ct_event!(keycode press PageUp) => self
                    .move_up(max(1, self.page_len().saturating_sub(1)), false)
                    .into(),
                ct_event!(keycode press PageDown) => self
                    .move_down(max(1, self.page_len().saturating_sub(1)), false)
                    .into(),

                ct_event!(keycode press SHIFT-Up) => self.move_up(1, true).into(),
                ct_event!(keycode press SHIFT-Down) => self.move_down(1, true).into(),
                ct_event!(keycode press CONTROL_SHIFT-Up)
                | ct_event!(keycode press CONTROL_SHIFT-Home)
                | ct_event!(keycode press SHIFT-Home) => self.move_to(0, true).into(),
                ct_event!(keycode press CONTROL_SHIFT-Down)
                | ct_event!(keycode press CONTROL_SHIFT-End)
                | ct_event!(keycode press SHIFT-End) => {
                    self.move_to(self.rows.saturating_sub(1), true).into()
                }
                ct_event!(keycode press SHIFT-PageUp) => self
                    .move_up(max(1, self.page_len().saturating_sub(1)), true)
                    .into(),
                ct_event!(keycode press SHIFT-PageDown) => self
                    .move_down(max(1, self.page_len().saturating_sub(1)), true)
                    .into(),

                ct_event!(keycode press Left) => self.scroll_left(1).into(),
                ct_event!(keycode press Right) => self.scroll_right(1).into(),
                ct_event!(keycode press CONTROL-Left) => self.scroll_to_x(0).into(),
                ct_event!(keycode press CONTROL-Right) => {
                    self.scroll_to_x(self.x_max_offset()).into()
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

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for TableState<RowSetSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _: MouseOnly) -> Outcome {
        flow!(match event {
            ct_event!(mouse any for m) | ct_event!(mouse any CONTROL for m)
                if self.mouse.drag(self.table_area, m)
                    || self.mouse.drag2(self.table_area, m, KeyModifiers::CONTROL) =>
            {
                self.move_to(self.row_at_drag((m.column, m.row)), true)
                    .into()
            }
            ct_event!(mouse down Left for column, row) => {
                let pos = (*column, *row);
                if self.table_area.contains(pos.into()) {
                    if let Some(new_row) = self.row_at_clicked(pos) {
                        self.move_to(new_row, false).into()
                    } else {
                        Outcome::NotUsed
                    }
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(mouse down ALT-Left for column, row) => {
                let pos = (*column, *row);
                if self.area.contains(pos.into()) {
                    if let Some(new_row) = self.row_at_clicked(pos) {
                        self.move_to(new_row, true).into()
                    } else {
                        Outcome::NotUsed
                    }
                } else {
                    Outcome::NotUsed
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
                        Outcome::Changed
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
            ScrollOutcome::VPos(v) => self.set_row_offset(v),
            ScrollOutcome::Left(v) => self.scroll_left(v),
            ScrollOutcome::Right(v) => self.scroll_right(v),
            ScrollOutcome::HPos(v) => self.set_x_offset(v),

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
    state: &mut TableState<RowSetSelection>,
    focus: bool,
    event: &crossterm::event::Event,
) -> Outcome {
    state.focus.set(focus);
    state.handle(event, Regular)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(
    state: &mut TableState<RowSetSelection>,
    event: &crossterm::event::Event,
) -> Outcome {
    state.handle(event, MouseOnly)
}
