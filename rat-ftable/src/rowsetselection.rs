use crate::event::TableOutcome;
use crate::{TableSelection, TableState};
use rat_event::{HandleEvent, MouseOnly, Regular, ct_event, flow};
use rat_focus::HasFocus;
use rat_scrolled::ScrollAreaState;
use rat_scrolled::event::ScrollOutcome;
use ratatui_crossterm::crossterm::event::{Event, KeyModifiers};
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
    fn count(&self) -> usize {
        let n = if let Some(anchor) = self.anchor_row {
            if let Some(lead) = self.lead_row {
                lead.abs_diff(anchor) + 1
            } else {
                0
            }
        } else {
            0
        };

        n + self.selected.len()
    }

    #[allow(clippy::collapsible_else_if)]
    #[allow(clippy::collapsible_if)]
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

    fn validate_rows(&mut self, rows: usize) {
        if rows == 0 {
            self.lead_row = None;
            self.anchor_row = None;
            self.selected.clear();
        } else {
            if let Some(lead_row) = self.lead_row {
                if lead_row >= rows {
                    self.lead_row = Some(rows - 1);
                }
            }
            if let Some(anchor_row) = self.anchor_row {
                if anchor_row >= rows {
                    self.anchor_row = Some(rows - 1);
                }
            }
            self.selected.retain(|v| *v < rows);
        }
    }

    fn validate_cols(&mut self, _cols: usize) {}

    fn items_added(&mut self, pos: usize, n: usize) {
        if let Some(lead_row) = self.lead_row {
            if lead_row > pos {
                self.lead_row = Some(lead_row + n);
            }
        }
        if let Some(anchor_row) = self.anchor_row {
            if anchor_row > pos {
                self.anchor_row = Some(anchor_row + n);
            }
        }
        let corr = self.selected.extract_if(|v| *v > pos).collect::<Vec<_>>();
        for v in corr {
            self.selected.insert(v + n);
        }
    }

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
        if let Some(anchor_row) = self.anchor_row {
            if rows == 0 {
                self.anchor_row = None;
            } else if anchor_row == pos && anchor_row + n >= rows {
                self.anchor_row = Some(rows.saturating_sub(1))
            } else if anchor_row > pos {
                self.anchor_row = Some(anchor_row.saturating_sub(n).min(pos));
            }
        }
        let corr = self.selected.extract_if(|v| *v > pos).collect::<Vec<_>>();
        for v in corr {
            if rows == 0 {
                // removed
            } else if v == pos && v + n >= rows {
                self.selected.insert(rows.saturating_sub(1));
            } else if v > pos {
                self.selected.insert(v.saturating_sub(n).min(pos));
            }
        }
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

/// Handle all events.
/// Table events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events(
    state: &mut TableState<RowSetSelection>,
    focus: bool,
    event: &Event,
) -> TableOutcome {
    state.focus.set(focus);
    state.handle(event, Regular)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(state: &mut TableState<RowSetSelection>, event: &Event) -> TableOutcome {
    state.handle(event, MouseOnly)
}
