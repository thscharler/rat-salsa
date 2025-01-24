use crate::calendar::calendar::CalendarState;
use crate::calendar::event::CalOutcome;
use crate::calendar::{CalendarSelection, MonthState};
use chrono::{NaiveDate, Weekday};
use rat_event::ConsumedEvent;
use rat_event::{ct_event, flow, HandleEvent, MouseOnly, Regular};
use rat_focus::HasFocus;
use std::ops::RangeInclusive;

#[derive(Debug, Default, Clone)]
pub struct RangeSelection {
    anchor: Option<NaiveDate>,
    lead: Option<NaiveDate>,
}

impl CalendarSelection for RangeSelection {
    fn clear(&mut self) {
        self.anchor = None;
        self.lead = None;
    }

    fn is_selected(&self, date: NaiveDate) -> bool {
        if let Some(lead) = self.lead {
            if let Some(anchor) = self.anchor {
                if lead > anchor {
                    date >= anchor && date <= lead
                } else {
                    date >= lead && date <= anchor
                }
            } else {
                unreachable!()
            }
        } else {
            false
        }
    }

    fn lead_selection(&self) -> Option<NaiveDate> {
        self.lead
    }
}

impl RangeSelection {
    /// Select a week. Any date in a week will do.
    pub fn select_week(&mut self, date: NaiveDate, extend: bool) -> bool {
        let old = (self.anchor, self.lead);

        let start = date.week(Weekday::Mon).first_day();
        let end = date.week(Weekday::Mon).last_day();

        if extend {
            if let Some(anchor) = self.anchor {
                if start < anchor {
                    if end < anchor {
                        self.lead = Some(start);
                    } else {
                        self.lead = Some(start);
                        self.anchor = Some(end);
                    }
                } else {
                    self.lead = Some(end);
                }
            } else {
                self.anchor = Some(start);
                self.lead = Some(end);
            }
        } else {
            self.anchor = Some(start);
            self.lead = Some(end);
        }

        old != (self.anchor, self.lead)
    }

    /// Select a date.
    pub fn select_day(&mut self, date: NaiveDate, extend: bool) -> bool {
        let old = (self.anchor, self.lead);

        if extend {
            self.lead = Some(date);
        } else {
            self.anchor = Some(date);
            self.lead = Some(date);
        }

        old != (self.anchor, self.lead)
    }

    /// Select range as (anchor, lead).
    pub fn select(&mut self, selection: (NaiveDate, NaiveDate)) -> bool {
        let old = (self.anchor, self.lead);

        self.anchor = Some(selection.0);
        self.lead = Some(selection.1);

        old != (self.anchor, self.lead)
    }

    /// Selection as (anchor, lead)
    pub fn selected(&self) -> Option<(NaiveDate, NaiveDate)> {
        if let Some(anchor) = self.anchor {
            if let Some(lead) = self.lead {
                Some((anchor, lead))
            } else {
                unreachable!()
            }
        } else {
            None
        }
    }

    /// Selection as date-range.
    pub fn selected_range(&self) -> Option<RangeInclusive<NaiveDate>> {
        if let Some(anchor) = self.anchor {
            if let Some(lead) = self.lead {
                if lead > anchor {
                    Some(anchor..=lead)
                } else {
                    Some(lead..=anchor)
                }
            } else {
                unreachable!()
            }
        } else {
            None
        }
    }
}

impl HandleEvent<crossterm::event::Event, Regular, CalOutcome> for MonthState<RangeSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> CalOutcome {
        if self.is_focused() {
            flow!(match event {
                ct_event!(keycode press Up) => self.prev_day(7, false),
                ct_event!(keycode press Down) => self.next_day(7, false),
                ct_event!(keycode press Left) => self.prev_day(1, false),
                ct_event!(keycode press Right) => self.next_day(1, false),
                ct_event!(keycode press ALT-Up) => self.prev_week(1, false),
                ct_event!(keycode press ALT-Down) => self.next_week(1, false),
                ct_event!(keycode press SHIFT-Up) => self.prev_day(7, true),
                ct_event!(keycode press SHIFT-Down) => self.next_day(7, true),
                ct_event!(keycode press SHIFT-Left) => self.prev_day(1, true),
                ct_event!(keycode press SHIFT-Right) => self.next_day(1, true),
                ct_event!(keycode press ALT_SHIFT-Up) => self.prev_week(1, true),
                ct_event!(keycode press ALT_SHIFT-Down) => self.next_week(1, true),
                _ => CalOutcome::Continue,
            })
        }

        self.handle(event, MouseOnly)
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, CalOutcome> for MonthState<RangeSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> CalOutcome {
        match event {
            ct_event!(mouse drag Left for x, y) | ct_event!(mouse down Left for x, y) => {
                if let Some(sel) = self.mouse.item_at(&self.area_weeks, *x, *y) {
                    self.select_week(sel, false)
                } else if let Some(sel) = self.mouse.item_at(&self.area_days, *x, *y) {
                    self.select_day(sel, false)
                } else {
                    CalOutcome::Continue
                }
            }
            _ => CalOutcome::Continue,
        }
    }
}

impl<const N: usize> HandleEvent<crossterm::event::Event, Regular, CalOutcome>
    for CalendarState<N, RangeSelection>
{
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> CalOutcome {
        let mut r = 'f: {
            for month in &mut self.months {
                let r = month.handle(event, Regular);
                if r.is_consumed() {
                    //todo: change to on selected
                    self.focus_lead();
                    break 'f r;
                }
            }
            CalOutcome::Continue
        };

        r = r.or_else(|| {
            if self.is_focused() {
                match event {
                    ct_event!(keycode press PageUp) => self.shift_back(self.step()),
                    ct_event!(keycode press PageDown) => self.shift_forward(self.step()),
                    ct_event!(keycode press Up) => self.prev_day(7, false),
                    ct_event!(keycode press Down) => self.next_day(7, false),
                    ct_event!(keycode press Left) => self.prev_day(1, false),
                    ct_event!(keycode press Right) => self.next_day(1, false),
                    ct_event!(keycode press Home) => self.move_to_today(),
                    ct_event!(keycode press ALT-Up) => self.prev_week(1, false),
                    ct_event!(keycode press ALT-Down) => self.next_week(1, false),
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
    for CalendarState<N, RangeSelection>
{
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> CalOutcome {
        for i in 0..self.months.len() {
            if self.months[i].gained_focus() {
                self.set_primary_focus(i);
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
