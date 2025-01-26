use crate::calendar::event::CalOutcome;
use crate::calendar::{CalendarSelection, CalendarState, MonthState};
use chrono::NaiveDate;
use rat_event::{ct_event, ConsumedEvent, HandleEvent, MouseOnly, Regular};
use rat_focus::HasFocus;

/// No selection model
///
/// Can scroll the calendar with Up/Down, PageUp/PageDown though.
/// Ctrl+Home moves to today.
///
#[derive(Debug, Default, Clone)]
pub struct NoSelection;

impl CalendarSelection for NoSelection {
    fn count(&self) -> usize {
        0
    }

    fn is_selected(&self, _: NaiveDate) -> bool {
        false
    }

    fn lead_selection(&self) -> Option<NaiveDate> {
        None
    }
}

impl HandleEvent<crossterm::event::Event, Regular, CalOutcome> for MonthState<NoSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> CalOutcome {
        self.handle(event, MouseOnly)
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, CalOutcome> for MonthState<NoSelection> {
    fn handle(&mut self, _event: &crossterm::event::Event, _qualifier: MouseOnly) -> CalOutcome {
        CalOutcome::Continue
    }
}

impl<const N: usize> HandleEvent<crossterm::event::Event, Regular, CalOutcome>
    for CalendarState<N, NoSelection>
{
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> CalOutcome {
        let mut r = 'f: {
            for month in &mut self.months {
                let r = month.handle(event, Regular);
                if r == CalOutcome::Selected {
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
                    ct_event!(keycode press Up) => self.prev_month(1),
                    ct_event!(keycode press Down) => self.next_month(1),
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
    for CalendarState<N, NoSelection>
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
