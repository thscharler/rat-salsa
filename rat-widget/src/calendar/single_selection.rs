use crate::calendar::event::CalOutcome;
use crate::calendar::{CalendarSelection, CalendarState, MonthState};
use chrono::NaiveDate;
use rat_event::util::item_at;
use rat_event::{ConsumedEvent, HandleEvent, MouseOnly, Regular, ct_event, flow};
use rat_focus::HasFocus;

/// Can select a single date.
///
/// Movement with the arrow keys and PageUp/PageDown.
/// Ctrl+Home moves to today.
///
#[derive(Debug, Default, Clone)]
pub struct SingleSelection {
    selected: Option<NaiveDate>,
}

impl CalendarSelection for SingleSelection {
    fn count(&self) -> usize {
        if self.selected.is_some() { 1 } else { 0 }
    }

    fn is_selected(&self, date: NaiveDate) -> bool {
        self.selected == Some(date)
    }

    fn lead_selection(&self) -> Option<NaiveDate> {
        self.selected
    }
}

impl SingleSelection {
    pub fn clear(&mut self) {
        self.selected = None;
    }

    pub fn select(&mut self, date: NaiveDate) -> bool {
        let old = self.selected;
        self.selected = Some(date);
        old != self.selected
    }

    pub fn selected(&self) -> Option<NaiveDate> {
        self.selected
    }
}

impl HandleEvent<crossterm::event::Event, Regular, CalOutcome> for MonthState<SingleSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> CalOutcome {
        if self.is_focused() {
            flow!(match event {
                ct_event!(keycode press Home) => self.select_day(0),
                ct_event!(keycode press End) => self.select_last(),
                ct_event!(keycode press Up) => self.prev_day(7),
                ct_event!(keycode press Down) => self.next_day(7),
                ct_event!(keycode press Left) => self.prev_day(1),
                ct_event!(keycode press Right) => self.next_day(1),
                _ => CalOutcome::Continue,
            })
        }

        self.handle(event, MouseOnly)
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, CalOutcome> for MonthState<SingleSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> CalOutcome {
        match event {
            ct_event!(mouse drag Left for x, y) | ct_event!(mouse down Left for x, y) => {
                if let Some(sel) = item_at(&self.area_days, *x, *y) {
                    self.select_day(sel)
                } else {
                    CalOutcome::Continue
                }
            }

            _ => CalOutcome::Continue,
        }
    }
}

impl<const N: usize> HandleEvent<crossterm::event::Event, Regular, CalOutcome>
    for CalendarState<N, SingleSelection>
{
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> CalOutcome {
        let mut r = 'f: {
            for month in &mut self.months {
                let r = month.handle(event, Regular);
                if r.is_consumed() {
                    self.focus_lead();
                    break 'f r;
                }
            }
            CalOutcome::Continue
        };

        r = r.or_else(|| {
            if self.is_focused() {
                match event {
                    ct_event!(keycode press CONTROL-Home) => self.move_to_today(),
                    ct_event!(keycode press PageUp) => self.prev_month(1),
                    ct_event!(keycode press PageDown) => self.next_month(1),
                    ct_event!(keycode press Up) => self.prev_day(7),
                    ct_event!(keycode press Down) => self.next_day(7),
                    ct_event!(keycode press Left) => self.prev_day(1),
                    ct_event!(keycode press Right) => self.next_day(1),

                    _ => CalOutcome::Continue,
                }
            } else {
                CalOutcome::Continue
            }
        });

        r.or_else(|| self.handle(event, MouseOnly))
    }
}

impl<const N: usize> HandleEvent<crossterm::event::Event, MouseOnly, CalOutcome>
    for CalendarState<N, SingleSelection>
{
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> CalOutcome {
        for i in 0..self.months.len() {
            if self.months[i].gained_focus() {
                self.set_primary_idx(i);
                break;
            }
        }

        let all_areas = self
            .months
            .iter()
            .map(|v| v.area)
            .reduce(|v, w| v.union(w))
            .unwrap_or_default();
        match event {
            ct_event!(scroll up for x,y) if all_areas.contains((*x, *y).into()) => {
                self.scroll_back(self.step())
            }
            ct_event!(scroll down for x,y) if all_areas.contains((*x, *y).into()) => {
                self.scroll_forward(self.step())
            }
            _ => CalOutcome::Continue,
        }
    }
}
