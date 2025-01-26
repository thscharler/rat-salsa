use crate::calendar::calendar::CalendarState;
use crate::calendar::event::CalOutcome;
use crate::calendar::{
    first_day_of_month, is_first_day_of_month, is_last_day_of_month, is_same_month, is_same_week,
    last_day_of_month, CalendarSelection, MonthState,
};
use chrono::{Datelike, Days, Months, NaiveDate, Weekday};
use rat_event::util::item_at;
use rat_event::ConsumedEvent;
use rat_event::{ct_event, flow, HandleEvent, MouseOnly, Regular};
use rat_focus::HasFocus;
use std::ops::RangeInclusive;

/// Can select a date range.
///
/// - Movement with the arrow-keys and PageUp/PageDown.
/// - Ctrl+Home moves to today.
///
/// - Ctrl+A selects the current month.
/// - Shift+Arrow extends the selection.
/// - Shift+PageUp/PageDown extends the selection by a whole month.
/// - Alt+Shift+Up/Down extends the selection by a whole week.
///
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

    fn len(&self) -> usize {
        if let Some(anchor) = self.anchor {
            if let Some(lead) = self.lead {
                (lead - anchor).num_days().unsigned_abs() as usize + 1
            } else {
                unreachable!()
            }
        } else {
            0
        }
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
    /// Select the week of the given date.
    ///
    /// If extend is used, this will extend the selection to include
    /// the new week. If the current selection doesn't cover full weeks
    /// it will be buffed up to do so afterwards.
    ///
    pub fn select_month(&mut self, date: NaiveDate, extend: bool) -> bool {
        let old = (self.anchor, self.lead);

        let new_start = first_day_of_month(date);
        let new_end = last_day_of_month(date);

        if extend {
            if let Some(mut lead) = self.lead {
                let Some(mut anchor) = self.anchor else {
                    unreachable!();
                };

                // fill out week
                if lead <= anchor {
                    if !is_first_day_of_month(lead) || !is_last_day_of_month(anchor) {
                        lead = first_day_of_month(lead);
                        anchor = last_day_of_month(anchor);
                        self.lead = Some(lead);
                        self.anchor = Some(anchor);
                    }
                } else {
                    if !is_last_day_of_month(lead) || !is_first_day_of_month(anchor) {
                        lead = last_day_of_month(lead);
                        anchor = first_day_of_month(anchor);
                        self.lead = Some(lead);
                        self.anchor = Some(anchor);
                    }
                }

                if is_same_month(lead, anchor) {
                    // A single month can change direction.
                    if lead <= anchor {
                        if new_start < anchor {
                            self.lead = Some(new_start);
                        } else {
                            self.lead = Some(new_end);
                            self.anchor = Some(lead);
                        }
                    } else {
                        if new_start > anchor {
                            self.lead = Some(new_end);
                        } else {
                            self.lead = Some(new_start);
                            self.anchor = Some(lead);
                        }
                    }
                } else {
                    // keep direction and reduce/extend in that direction.
                    if lead < anchor {
                        if new_start <= lead {
                            self.lead = Some(new_start);
                        } else if new_end <= anchor {
                            self.lead = Some(new_start);
                        } else {
                            self.lead = Some(new_end);
                        }
                    } else {
                        if new_end <= lead {
                            self.lead = Some(new_end);
                        } else if new_start >= anchor {
                            self.lead = Some(new_end);
                        } else {
                            self.lead = Some(new_start);
                        }
                    }
                }
            } else {
                self.lead = Some(new_start);
                self.anchor = Some(new_end);
            }
        } else {
            self.lead = Some(new_start);
            self.anchor = Some(new_end);
        }

        old != (self.anchor, self.lead)
    }

    /// Select the week of the given date.
    ///
    /// If extend is used, this will extend the selection to include
    /// the new week. If the current selection doesn't cover full weeks
    /// it will be buffed up to do so afterwards.
    ///
    pub fn select_week(&mut self, date: NaiveDate, extend: bool) -> bool {
        let old = (self.anchor, self.lead);

        let new_start = date.week(Weekday::Mon).first_day();
        let new_end = date.week(Weekday::Mon).last_day();

        if extend {
            if let Some(mut lead) = self.lead {
                let Some(mut anchor) = self.anchor else {
                    unreachable!();
                };

                // fill out week
                if lead <= anchor {
                    if lead.weekday() != Weekday::Mon || anchor.weekday() != Weekday::Sun {
                        lead = lead.week(Weekday::Mon).first_day();
                        anchor = anchor.week(Weekday::Mon).last_day();
                        self.lead = Some(lead);
                        self.anchor = Some(anchor);
                    }
                } else {
                    if lead.weekday() != Weekday::Sun || anchor.weekday() != Weekday::Mon {
                        lead = lead.week(Weekday::Mon).last_day();
                        anchor = anchor.week(Weekday::Mon).first_day();
                        self.lead = Some(lead);
                        self.anchor = Some(anchor);
                    }
                }

                if is_same_week(lead, anchor) {
                    // A single week can change direction.
                    if lead <= anchor {
                        if new_start < anchor {
                            self.lead = Some(new_start);
                        } else {
                            self.lead = Some(new_end);
                            self.anchor = Some(lead);
                        }
                    } else {
                        if new_start > anchor {
                            self.lead = Some(new_end);
                        } else {
                            self.lead = Some(new_start);
                            self.anchor = Some(lead);
                        }
                    }
                } else {
                    // keep direction and reduce/extend in that direction.
                    if lead < anchor {
                        if new_start <= lead {
                            self.lead = Some(new_start);
                        } else if new_end <= anchor {
                            self.lead = Some(new_start);
                        } else {
                            self.lead = Some(new_end);
                        }
                    } else {
                        if new_end <= lead {
                            self.lead = Some(new_end);
                        } else if new_start >= anchor {
                            self.lead = Some(new_end);
                        } else {
                            self.lead = Some(new_start);
                        }
                    }
                }
            } else {
                self.lead = Some(new_start);
                self.anchor = Some(new_end);
            }
        } else {
            self.lead = Some(new_start);
            self.anchor = Some(new_end);
        }

        old != (self.anchor, self.lead)
    }

    /// Select a date.
    pub fn select_day(&mut self, date: NaiveDate, extend: bool) -> bool {
        let old = (self.anchor, self.lead);

        if extend {
            self.lead = Some(date);
            if self.anchor.is_none() {
                self.anchor = Some(date);
            }
        } else {
            self.anchor = Some(date);
            self.lead = Some(date);
        }

        old != (self.anchor, self.lead)
    }

    /// Select range as (anchor, lead) pair.
    pub fn select(&mut self, selection: (NaiveDate, NaiveDate)) -> bool {
        let old = (self.anchor, self.lead);

        self.anchor = Some(selection.0);
        self.lead = Some(selection.1);

        old != (self.anchor, self.lead)
    }

    /// Selection as (anchor, lead) pair.
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
                ct_event!(keycode press Home) => self.select_day(0, false),
                ct_event!(keycode press End) => self.select_last(false),
                ct_event!(keycode press SHIFT-Home) => self.select_day(0, true),
                ct_event!(keycode press SHIFT-End) => self.select_last(true),

                ct_event!(keycode press Up) => self.prev_day(7, false),
                ct_event!(keycode press Down) => self.next_day(7, false),
                ct_event!(keycode press Left) => self.prev_day(1, false),
                ct_event!(keycode press Right) => self.next_day(1, false),
                ct_event!(keycode press SHIFT-Up) => self.prev_day(7, true),
                ct_event!(keycode press SHIFT-Down) => self.next_day(7, true),
                ct_event!(keycode press SHIFT-Left) => self.prev_day(1, true),
                ct_event!(keycode press SHIFT-Right) => self.next_day(1, true),

                ct_event!(keycode press ALT-Up) => self.prev_week(1, false),
                ct_event!(keycode press ALT-Down) => self.next_week(1, false),
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
        let mut r = match event {
            ct_event!(mouse any for m)
                if self.mouse.drag(
                    &[self.area_cal, self.area_weeknum], //
                    m,
                ) =>
            {
                if self.mouse.drag.get() == Some(0) {
                    if let Some(sel) = item_at(&self.area_days, m.column, m.row) {
                        self.select_day(sel, true)
                    } else {
                        let mut r = CalOutcome::Continue;

                        let mut above = self.area_cal;
                        above.y -= 2;
                        above.height = 3;
                        if above.contains((m.column, m.row).into()) {
                            r = self.select_day(0, true);
                        } else {
                            let mut below = self.area_cal;
                            below.y = below.bottom().saturating_sub(1);
                            below.height = 2;
                            if below.contains((m.column, m.row).into()) {
                                r = self.select_day(self.end_date().day0() as usize, true);
                            }
                        }
                        r
                    }
                } else if self.mouse.drag.get() == Some(1) {
                    if let Some(sel) = item_at(&self.area_weeks, m.column, m.row) {
                        self.select_week(sel, true)
                    } else {
                        let mut r = CalOutcome::Continue;

                        let mut above = self.area_weeknum;
                        above.y -= 2;
                        above.height = 3;
                        if above.contains((m.column, m.row).into()) {
                            r = self.select_week(0, true);
                        } else {
                            let mut below = self.area_cal;
                            below.y = below.bottom().saturating_sub(1);
                            below.height = 2;
                            if below.contains((m.column, m.row).into()) {
                                r = self.select_week(self.end_date().day0() as usize, true);
                            }
                        }
                        r
                    }
                } else {
                    CalOutcome::Continue
                }
            }
            _ => CalOutcome::Continue,
        };

        r = r.or_else(|| match event {
            ct_event!(mouse down Left for x, y) => {
                if let Some(sel) = item_at(&self.area_weeks, *x, *y) {
                    self.select_week(sel, false)
                } else if let Some(sel) = item_at(&self.area_days, *x, *y) {
                    self.select_day(sel, false)
                } else {
                    CalOutcome::Continue
                }
            }
            ct_event!(mouse down CONTROL-Left for x, y) => {
                if let Some(sel) = item_at(&self.area_weeks, *x, *y) {
                    self.select_week(sel, true)
                } else if let Some(sel) = item_at(&self.area_days, *x, *y) {
                    self.select_day(sel, true)
                } else {
                    CalOutcome::Continue
                }
            }
            _ => CalOutcome::Continue,
        });

        r
    }
}

impl<const N: usize> HandleEvent<crossterm::event::Event, Regular, CalOutcome>
    for CalendarState<N, RangeSelection>
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
        // Enable drag for all months.
        if r.is_consumed() {
            let mut drag = None;
            for m in &self.months {
                if let Some(d) = m.mouse.drag.get() {
                    drag = Some(d);
                    break;
                }
            }
            if drag.is_some() {
                for m in &self.months {
                    m.mouse.drag.set(drag);
                }
            }
        }

        r = r.or_else(|| {
            if self.is_focused() {
                match event {
                    ct_event!(key press CONTROL-'a') => {
                        if self.select_month(self.months[self.primary_idx()].start_date(), false) {
                            CalOutcome::Selected
                        } else {
                            CalOutcome::Continue
                        }
                    }
                    ct_event!(keycode press CONTROL-Home) => self.move_to_today(),
                    ct_event!(keycode press PageUp) => {
                        self.move_to_prev(Months::new(1), Days::new(0))
                    }
                    ct_event!(keycode press PageDown) => {
                        self.move_to_next(Months::new(1), Days::new(0))
                    }
                    ct_event!(keycode press SHIFT-PageUp) => self.prev_month(1, true),
                    ct_event!(keycode press SHIFT-PageDown) => self.next_month(1, true),

                    ct_event!(keycode press Up) => self.prev_day(7, false),
                    ct_event!(keycode press Down) => self.next_day(7, false),
                    ct_event!(keycode press Left) => self.prev_day(1, false),
                    ct_event!(keycode press Right) => self.next_day(1, false),
                    ct_event!(keycode press SHIFT-Up) => self.prev_day(7, true),
                    ct_event!(keycode press SHIFT-Down) => self.next_day(7, true),
                    ct_event!(keycode press SHIFT-Left) => self.prev_day(1, true),
                    ct_event!(keycode press SHIFT-Right) => self.next_day(1, true),

                    ct_event!(keycode press ALT-Up) => self.prev_week(1, false),
                    ct_event!(keycode press ALT-Down) => self.next_week(1, false),
                    ct_event!(keycode press ALT_SHIFT-Up) => self.prev_week(1, true),
                    ct_event!(keycode press ALT_SHIFT-Down) => self.next_week(1, true),

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
