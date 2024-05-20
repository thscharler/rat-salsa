use crate::event::Outcome;
use crate::{FTableState, TableSelection};
use rat_event::{ct_event, FocusKeys, HandleEvent, MouseOnly};
use ratatui::layout::Position;
use std::cmp::min;
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
    pub anchor: Option<usize>,
    /// Current end of the active selection.
    pub lead: Option<usize>,
    /// Retired rows. This doesn't contain the rows
    /// between anchor and lead.
    ///
    /// You can call [RowSetSelection::transfer_lead_anchor] to
    /// add the anchor-lead range. This resets anchor and lead though.
    /// Or iterate the complete range and call [RowSetSelection::is_selected_row].
    pub selected: HashSet<usize>,
}

impl TableSelection for RowSetSelection {
    #[allow(clippy::collapsible_else_if)]
    fn is_selected_row(&self, row: usize) -> bool {
        if let Some(mut anchor) = self.anchor {
            if let Some(mut lead) = self.lead {
                if lead < anchor {
                    mem::swap(&mut lead, &mut anchor);
                }

                if row >= anchor && row <= lead {
                    return true;
                }
            }
        } else {
            if let Some(lead) = self.lead {
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
        self.lead.map(|srow| (0, srow))
    }
}

impl RowSetSelection {
    /// New selection.
    pub fn new() -> RowSetSelection {
        RowSetSelection {
            anchor: None,
            lead: None,
            selected: HashSet::new(),
        }
    }

    fn extend(&mut self, extend: bool) {
        if extend {
            if self.anchor.is_none() {
                self.anchor = self.lead;
            }
        } else {
            self.anchor = None;
            self.selected.clear();
        }
    }

    /// Select next. Maybe extend the range.
    pub fn next(&mut self, n: usize, max: usize, extend: bool) {
        self.extend(extend);
        self.lead = match self.lead {
            None => Some(0),
            Some(srow) => Some(min(srow + n, max)),
        };
    }

    /// Select next. Maybe extend the range.
    pub fn prev(&mut self, n: usize, extend: bool) {
        self.extend(extend);
        self.lead = match self.lead {
            None => Some(0),
            Some(srow) => Some(srow.saturating_sub(n)),
        };
    }

    /// Set a new lead. Maybe extend the range.
    pub fn set_lead(&mut self, lead: Option<usize>, extend: bool) {
        self.extend(extend);
        self.lead = lead;
    }

    /// Set a new lead, at the same time limit the lead to max.
    pub fn set_lead_clamped(&mut self, lead: usize, max: usize, extend: bool) {
        self.extend(extend);
        if lead <= max {
            self.lead = Some(lead);
        } else {
            self.lead = Some(max);
        }
    }

    /// Current lead.
    pub fn lead(&self) -> Option<usize> {
        self.lead
    }

    /// Current anchor.
    pub fn anchor(&self) -> Option<usize> {
        self.anchor
    }

    /// Transfers the range anchor to lead to the selection set and reset both.
    pub fn transfer_lead_anchor(&mut self) {
        Self::fill(self.anchor, self.lead, &mut self.selected);
        self.anchor = None;
        self.lead = None;
    }

    /// Set of all selected rows. Clones the retired set and adds the current anchor..lead range.
    pub fn selected(&self) -> HashSet<usize> {
        let mut selected = self.selected.clone();
        Self::fill(self.anchor, self.lead, &mut selected);
        selected
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

    /// Clear the selection.
    pub fn clear(&mut self) {
        self.anchor = None;
        self.lead = None;
        self.selected.clear();
    }

    /// Add to selection.
    pub fn add(&mut self, idx: usize) {
        self.selected.insert(idx);
    }

    /// Remove from selection. Only works for retired selections, not for the
    /// active anchor-lead range.
    pub fn remove(&mut self, idx: usize) {
        self.selected.remove(&idx);
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for FTableState<RowSetSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _: FocusKeys) -> Outcome {
        let res = {
            match event {
                ct_event!(keycode press Down) => {
                    self.selection.next(1, self.rows - 1, false);
                    self.scroll_to_selected();
                    Outcome::Changed
                }
                ct_event!(keycode press SHIFT-Down) => {
                    self.selection.next(1, self.rows - 1, true);
                    self.scroll_to_selected();
                    Outcome::Changed
                }
                ct_event!(keycode press Up) => {
                    self.selection.prev(1, false);
                    self.scroll_to_selected();
                    Outcome::Changed
                }
                ct_event!(keycode press SHIFT-Up) => {
                    self.selection.prev(1, true);
                    self.scroll_to_selected();
                    Outcome::Changed
                }
                ct_event!(keycode press CONTROL-Down) | ct_event!(keycode press End) => {
                    self.selection.set_lead(Some(self.rows - 1), false);
                    self.scroll_to_selected();
                    Outcome::Changed
                }
                ct_event!(keycode press SHIFT-End) => {
                    self.selection.set_lead(Some(self.rows - 1), true);
                    self.scroll_to_selected();
                    Outcome::Changed
                }
                ct_event!(keycode press CONTROL-Up) | ct_event!(keycode press Home) => {
                    self.selection.set_lead(Some(0), false);
                    self.scroll_to_selected();
                    Outcome::Changed
                }
                ct_event!(keycode press SHIFT-Home) => {
                    self.selection.set_lead(Some(0), true);
                    self.scroll_to_selected();
                    Outcome::Changed
                }

                ct_event!(keycode press PageUp) => {
                    self.selection.prev(self.table_area.height as usize, false);
                    self.scroll_to_selected();
                    Outcome::Changed
                }
                ct_event!(keycode press SHIFT-PageUp) => {
                    self.selection.prev(self.table_area.height as usize, true);
                    self.scroll_to_selected();
                    Outcome::Changed
                }
                ct_event!(keycode press PageDown) => {
                    self.selection
                        .next(self.table_area.height as usize, self.rows - 1, false);
                    self.scroll_to_selected();
                    Outcome::Changed
                }
                ct_event!(keycode press SHIFT-PageDown) => {
                    self.selection
                        .next(self.table_area.height as usize, self.rows - 1, true);
                    self.scroll_to_selected();
                    Outcome::Changed
                }
                _ => Outcome::NotUsed,
            }
        };

        if res == Outcome::NotUsed {
            self.handle(event, MouseOnly)
        } else {
            res
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for FTableState<RowSetSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _: MouseOnly) -> Outcome {
        match event {
            ct_event!(scroll up for column, row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.scroll_up(self.table_area.height as usize / 10);
                    Outcome::Changed
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(scroll down for column, row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.scroll_down(self.table_area.height as usize / 10);
                    Outcome::Changed
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(mouse down Left for column, row) => {
                let pos = Position::new(*column, *row);
                if self.area.contains(pos) {
                    if let Some(new_row) = self.row_at_clicked(pos) {
                        self.mouse.set_drag();
                        self.selection
                            .set_lead_clamped(new_row, self.rows - 1, false);
                        Outcome::Changed
                    } else {
                        Outcome::Unchanged
                    }
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(mouse down ALT-Left for column, row) => {
                let pos = Position::new(*column, *row);
                if self.area.contains(pos) {
                    if let Some(new_row) = self.row_at_clicked(pos) {
                        self.mouse.set_drag();
                        self.selection
                            .set_lead_clamped(new_row, self.rows - 1, true);
                        Outcome::Changed
                    } else {
                        Outcome::Unchanged
                    }
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(mouse down CONTROL-Left for column, row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    let pos = Position::new(*column, *row);
                    if let Some(new_row) = self.row_at_clicked(pos) {
                        self.mouse.set_drag();
                        self.selection.transfer_lead_anchor();
                        if self.selection.is_selected_row(new_row) {
                            self.selection.remove(new_row);
                        } else {
                            self.selection
                                .set_lead_clamped(new_row, self.rows - 1, true);
                        }
                        Outcome::Changed
                    } else {
                        Outcome::Unchanged
                    }
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(mouse drag Left for column, row)
            | ct_event!(mouse drag CONTROL-Left for column, row) => {
                if self.mouse.do_drag() {
                    let pos = Position::new(*column, *row);
                    let new_row = self.row_at_drag(pos);
                    self.selection
                        .set_lead_clamped(new_row, self.rows - 1, true);
                    self.scroll_to_selected();
                    Outcome::Changed
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(mouse moved) => {
                self.mouse.clear_drag();
                Outcome::NotUsed
            }

            _ => Outcome::NotUsed,
        }
    }
}
