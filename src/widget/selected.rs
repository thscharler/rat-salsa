use crate::util::{next_opt, prev_opt};
use std::collections::HashSet;
use std::fmt::Debug;
use std::mem;

/// Trait for using a selection.
///
pub trait Selection {
    /// Is selected.
    fn is_selected(&self, n: usize) -> bool;

    /// Selection lead.
    fn lead_selection(&self) -> Option<usize>;
}

// -----------------------------------------------------------------------
// -----------------------------------------------------------------------

/// NoSelection
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NoSelection;

impl Selection for NoSelection {
    fn is_selected(&self, _n: usize) -> bool {
        false
    }

    fn lead_selection(&self) -> Option<usize> {
        None
    }
}

// -----------------------------------------------------------------------
// -----------------------------------------------------------------------

/// Single element selection.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct SingleSelection {
    pub selected: Option<usize>,
}

impl Selection for SingleSelection {
    fn is_selected(&self, n: usize) -> bool {
        self.selected == Some(n)
    }

    fn lead_selection(&self) -> Option<usize> {
        self.selected
    }
}

impl SingleSelection {
    pub fn new() -> SingleSelection {
        SingleSelection { selected: None }
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn select(&mut self, select: Option<usize>) {
        self.selected = select;
    }

    pub fn next(&mut self, n: usize, max: usize) {
        self.selected = next_opt(self.selected, n, max);
    }

    pub fn prev(&mut self, n: usize) {
        self.selected = prev_opt(self.selected, n);
    }
}

// -----------------------------------------------------------------------
// -----------------------------------------------------------------------

/// List selection
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct SetSelection {
    pub anchor: Option<usize>,
    pub lead: Option<usize>,
    pub selected: HashSet<usize>,
}

impl Selection for SetSelection {
    fn is_selected(&self, n: usize) -> bool {
        if let Some(mut anchor) = self.anchor {
            if let Some(mut lead) = self.lead {
                if lead < anchor {
                    mem::swap(&mut lead, &mut anchor);
                }

                if n >= lead && n <= anchor {
                    return true;
                }
            }
        } else {
            if let Some(lead) = self.lead {
                if n == lead {
                    return true;
                }
            }
        }

        self.selected.contains(&n)
    }

    fn lead_selection(&self) -> Option<usize> {
        self.lead
    }
}

impl SetSelection {
    pub fn new() -> SetSelection {
        SetSelection {
            anchor: None,
            lead: None,
            selected: HashSet::new(),
        }
    }

    pub fn next(&mut self, n: usize, max: usize, extend: bool) {
        if extend && self.anchor.is_none() {
            self.anchor = self.lead;
        } else {
            self.anchor = None;
        }
        self.lead = next_opt(self.lead, n, max);
    }

    pub fn prev(&mut self, n: usize, extend: bool) {
        if extend && self.anchor.is_none() {
            self.anchor = self.lead;
        } else {
            self.anchor = None;
        }
        self.lead = prev_opt(self.lead, n);
    }

    pub fn set_lead(&mut self, lead: Option<usize>, extend: bool) {
        if extend && self.anchor.is_none() {
            self.anchor = self.lead;
        }
        self.lead = lead;
    }

    pub fn lead(&self) -> Option<usize> {
        self.lead
    }

    pub fn anchor(&self) -> Option<usize> {
        self.anchor
    }

    pub fn transfer_lead_anchor(&mut self) {
        self.fill(&mut self.selected);
        self.anchor = None;
        // keep lead
    }

    fn fill(&self, selection: &mut HashSet<usize>) {
        if let Some(mut anchor) = self.anchor {
            if let Some(mut lead) = self.lead {
                if lead < anchor {
                    mem::swap(&mut lead, &mut anchor);
                }

                for n in lead..=anchor {
                    selection.add(n);
                }
            }
        } else {
            if let Some(lead) = self.lead {
                selection.add(lead);
            }
        }
    }

    pub fn clear(&mut self) {
        self.anchor = None;
        self.lead = None;
        self.selected.clear();
    }

    pub fn add(&mut self, idx: usize) {
        self.selected.insert(idx);
    }

    pub fn remove(&mut self, idx: usize) {
        self.selected.remove(&idx);
    }
}

// -----------------------------------------------------------------------
// -----------------------------------------------------------------------
