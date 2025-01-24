use crate::calendar::event::CalOutcome;
use crate::calendar::{CalendarSelection, CalendarState, MonthState};
use chrono::NaiveDate;
use log::debug;
use rat_event::{HandleEvent, MouseOnly, Regular};
use rat_focus::HasFocus;

#[derive(Debug, Default, Clone)]
pub struct NoSelection;

impl CalendarSelection for NoSelection {
    fn clear(&mut self) {}

    fn len(&self) -> usize {
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
        self.handle(event, MouseOnly)
    }
}

impl<const N: usize> HandleEvent<crossterm::event::Event, MouseOnly, CalOutcome>
    for CalendarState<N, NoSelection>
{
    fn handle(&mut self, _event: &crossterm::event::Event, _qualifier: MouseOnly) -> CalOutcome {
        for i in 0..self.months.len() {
            if self.months[i].gained_focus() {
                debug!("no gained {}", i);
                self.set_primary_idx(i);
                break;
            }
        }

        CalOutcome::Continue
    }
}
