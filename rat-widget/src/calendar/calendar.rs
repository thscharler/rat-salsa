use crate::calendar::event::CalOutcome;
use crate::calendar::selection::{NoSelection, RangeSelection, SingleSelection};
use crate::calendar::{CalendarSelection, MonthState};
use chrono::{Datelike, Days, Local, Months, NaiveDate, Weekday};
use rat_event::ConsumedEvent;
use rat_focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_reloc::RelocatableState;
use ratatui::layout::Rect;
use std::array;
use std::cell::RefCell;
use std::rc::Rc;

/// How should [move_to_current] behave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomePolicy {
    /// Set the primary month to the fixed index in the calendar.
    Index(usize),
    /// Behave like a yearly calendar. Sets the primary month to
    /// the current month.
    Year,
}

impl Default for HomePolicy {
    fn default() -> Self {
        Self::Index(0)
    }
}

/// A struct that contains an array of MonthState to get a true calendar
/// behaviour. There is no Widget for this, the exact layout is left
/// to the user.
#[derive(Clone)]
pub struct CalendarState<const N: usize, Selection> {
    /// Step-size when navigating outside the displayed
    /// calendar.
    ///
    /// You can do a rolling calendar which goes back 1 month
    /// or a yearly calendar which goes back 12 months.
    /// Or any other value you like.
    ///
    /// If you set step to 0 this kind of navigation is
    /// deactivated.
    ///
    /// Default is 1.
    step: usize,

    /// Behavior of move_to_current.
    home: HomePolicy,

    /// Primary month.
    /// Only this month will take part in Tab navigation.
    /// The other months will be mouse-reachable only.
    /// You can use arrow-keys to navigate the months though.
    primary_focus: usize,

    /// Months.
    pub months: [MonthState<Selection>; N],

    pub selection: Rc<RefCell<Selection>>,

    /// Calendar focus
    pub focus: FocusFlag,
}

impl<const N: usize, Selection> Default for CalendarState<N, Selection>
where
    Selection: Default,
{
    fn default() -> Self {
        let selection = Rc::new(RefCell::new(Selection::default()));

        Self {
            step: 1,
            months: array::from_fn(|_| {
                let mut state = MonthState::new();
                state.selection = selection.clone();
                state
            }),
            selection,
            primary_focus: Default::default(),
            focus: Default::default(),
            home: Default::default(),
        }
    }
}

impl<const N: usize, Selection> HasFocus for CalendarState<N, Selection> {
    fn build(&self, builder: &mut FocusBuilder) {
        let tag = builder.start(self);
        for (i, v) in self.months.iter().enumerate() {
            if i == self.primary_focus {
                builder.widget(v); // regular widget
            } else {
                builder.append_flags(v.focus(), v.area(), v.area_z(), Navigation::Leave)
            }
        }
        builder.end(tag);
    }

    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        Rect::default()
    }
}

impl<const N: usize, Selection> RelocatableState for CalendarState<N, Selection> {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        for w in &mut self.months {
            w.relocate(shift, clip);
        }
    }
}

impl<const N: usize, Selection> CalendarState<N, Selection> {
    pub fn new() -> Self
    where
        Selection: Default,
    {
        Self::default()
    }

    /// Step size in months when scrolling/leaving the
    /// displayed range.
    pub fn set_step(&mut self, step: usize) {
        assert!(step < self.months.len());
        self.step = step;
    }

    /// Step size in months when scrolling/leaving the
    /// displayed range.
    pub fn step(&self) -> usize {
        self.step
    }

    /// How should move_to_current() work.
    pub fn set_home_policy(&mut self, home: HomePolicy) {
        if let HomePolicy::Index(idx) = home {
            assert!(idx < self.months.len());
        }
        self.home = home;
    }

    pub fn home_policy(&mut self) -> HomePolicy {
        self.home
    }

    /// Set the primary month for the calendar.
    ///
    /// The primary month will be focused with Tab navigation.
    /// It can be changed by clicking on another month or
    /// by key-navigation.
    pub fn set_primary_focus(&mut self, primary: usize) {
        assert!(primary < self.months.len());
        self.primary_focus = primary;
    }

    /// Primary month.
    pub fn primary(&self) -> usize {
        self.primary_focus
    }

    /// Set the start-date for the calendar.
    /// Will set each month-state to a consecutive month.
    pub fn set_start_date(&mut self, mut date: NaiveDate) {
        for month in &mut self.months {
            month.set_start_date(date);
            date = date + Months::new(1);
        }
    }

    /// Start-date of the calendar.
    pub fn start_date(&self) -> NaiveDate {
        self.months[0].start_date()
    }

    /// End-date of the calendar.
    pub fn end_date(&self) -> NaiveDate {
        self.months[self.months.len() - 1].end_date()
    }

    /// Changes the start-date for each month.
    /// Doesn't change any selection.
    pub fn scroll_forward(&mut self, n: usize) -> CalOutcome {
        if n == 0 {
            return CalOutcome::Continue;
        }

        // change start dates
        let mut start = self.months[0].start_date() + Months::new(n as u32);

        for i in 0..self.months.len() {
            self.months[i].set_start_date(start);
            start = start + Months::new(1);
        }

        CalOutcome::Changed
    }

    /// Changes the start-date for each month.
    /// Doesn't change any selection.
    pub fn scroll_back(&mut self, n: usize) -> CalOutcome {
        if n == 0 {
            return CalOutcome::Continue;
        }

        // change start dates
        let mut start = self.months[0].start_date() - Months::new(n as u32);

        for i in 0..self.months.len() {
            self.months[i].set_start_date(start);
            start = start + Months::new(1);
        }

        CalOutcome::Changed
    }
}

impl<const N: usize, Selection> CalendarState<N, Selection>
where
    Selection: CalendarSelection,
{
    pub(super) fn focus_lead(&mut self) -> CalOutcome {
        let Some(lead) = self.selection.lead_selection() else {
            return CalOutcome::Continue;
        };

        let mut r = CalOutcome::Continue;

        if self.is_focused() {
            for (i, month) in self.months.iter().enumerate() {
                if lead >= month.start_date() && lead <= month.end_date() {
                    if self.primary_focus != i {
                        r = CalOutcome::Changed;
                    }
                    self.primary_focus = i;
                    month.focus.set(true);
                } else {
                    month.focus.set(false);
                }
            }
        }

        r
    }
}

impl<const N: usize> CalendarState<N, NoSelection> {}

impl<const N: usize> CalendarState<N, SingleSelection> {
    /// Move all selections back by step.
    pub fn shift_back(&mut self, n: usize) -> CalOutcome {
        if self.step == 0 {
            return CalOutcome::Continue;
        }

        let mut r = CalOutcome::Continue;

        let date = self.selection.borrow().selected();
        if let Some(date) = date {
            if self
                .selection
                .borrow_mut()
                .select(date - Months::new(n as u32))
            {
                r = CalOutcome::Selected
            }
        }
        let date = self.selection.borrow().selected();
        if let Some(date) = date {
            if date < self.start_date() {
                r = r.max(self.scroll_back(self.step));
            }
        }
        r.max(self.focus_lead())
    }

    /// Move all selections forward by step
    pub fn shift_forward(&mut self, n: usize) -> CalOutcome {
        if self.step == 0 {
            return CalOutcome::Continue;
        }

        let mut r = CalOutcome::Continue;

        let date = self.selection.borrow().selected();
        if let Some(date) = date {
            if self
                .selection
                .borrow_mut()
                .select(date + Months::new(n as u32))
            {
                r = CalOutcome::Selected
            }
        }
        let date = self.selection.borrow().selected();
        if let Some(date) = date {
            if date < self.start_date() {
                r = r.max(self.scroll_forward(self.step));
            }
        }
        r.max(self.focus_lead())
    }

    pub fn move_to_current(&mut self) -> CalOutcome {
        let current = Local::now().date_naive();

        let mut r = CalOutcome::Changed;

        if self.selection.borrow_mut().select(current) {
            r = CalOutcome::Selected;
        }
        match self.home {
            HomePolicy::Index(primary) => {
                self.primary_focus = primary;
                self.set_start_date(current - Months::new(primary as u32));
                self.focus_lead();
            }
            HomePolicy::Year => {
                let month = current.month0();
                self.primary_focus = month as usize;
                self.set_start_date(current - Months::new(month));
                self.focus_lead();
            }
        }

        r
    }

    /// Select previous day.
    pub fn prev_day(&mut self, n: usize) -> CalOutcome {
        let base_start = self.start_date();
        let base_end = self.end_date();

        let new_date = if let Some(date) = self.selection.lead_selection() {
            if date >= base_start && date <= base_end {
                date - Days::new(n as u64)
            } else if date < base_start {
                self.start_date()
            } else {
                self.end_date()
            }
        } else {
            self.end_date()
        };

        let mut r = CalOutcome::Continue;

        if new_date >= base_start && new_date <= base_end {
            if self.selection.borrow_mut().select(new_date) {
                r = CalOutcome::Selected;
            }
        } else if self.step > 0 {
            r = r.max(self.scroll_back(self.step));
            if self.selection.borrow_mut().select(new_date) {
                r = r.max(CalOutcome::Selected);
            }
        }

        if r.is_consumed() {
            self.focus_lead();
        }

        r
    }

    /// Select previous week.
    pub fn next_day(&mut self, n: usize) -> CalOutcome {
        let base_start = self.start_date();
        let base_end = self.end_date();

        let new_date = if let Some(date) = self.selection.lead_selection() {
            if date >= base_start && date <= base_end {
                date + Days::new(n as u64)
            } else if date < base_start {
                self.start_date()
            } else {
                self.end_date()
            }
        } else {
            self.start_date()
        };

        let mut r = CalOutcome::Continue;

        if new_date >= base_start && new_date <= base_end {
            if self.selection.borrow_mut().select(new_date) {
                r = CalOutcome::Selected;
            }
        } else if self.step > 0 {
            r = self.scroll_forward(self.step);
            if self.selection.borrow_mut().select(new_date) {
                r = r.max(CalOutcome::Selected);
            }
        }

        if r.is_consumed() {
            self.focus_lead();
        }

        r
    }
}

impl<const N: usize> CalendarState<N, RangeSelection> {
    /// Move all selections back by step.
    pub fn shift_back(&mut self, n: usize) -> CalOutcome {
        if self.step == 0 {
            return CalOutcome::Continue;
        }

        let mut r = CalOutcome::Continue;

        let date = self.selection.borrow().selected();
        if let Some(date) = date {
            if self.selection.borrow_mut().select((
                date.0 - Months::new(n as u32),
                date.1 - Months::new(n as u32),
            )) {
                r = CalOutcome::Selected;
            }
        }
        let date = self.selection.borrow().selected();
        if let Some(date) = date {
            if date.0 < self.start_date() {
                r = r.max(self.scroll_back(self.step));
            }
        }
        r.max(self.focus_lead())
    }

    /// Move all selections forward by step
    pub fn shift_forward(&mut self, n: usize) -> CalOutcome {
        if self.step == 0 {
            return CalOutcome::Continue;
        }

        let mut r = CalOutcome::Continue;

        let date = self.selection.borrow().selected();
        if let Some(date) = date {
            if self.selection.borrow_mut().select((
                date.0 + Months::new(n as u32),
                date.1 + Months::new(n as u32),
            )) {
                r = CalOutcome::Selected;
            }
        }
        let date = self.selection.borrow().selected();
        if let Some(date) = date {
            if date.0 < self.start_date() {
                r = r.max(self.scroll_forward(self.step));
            }
        }

        r.max(self.focus_lead())
    }

    pub fn move_to_current(&mut self) -> CalOutcome {
        let current = Local::now().date_naive();

        let mut r = CalOutcome::Changed;

        if self.selection.borrow_mut().select_day(current, false) {
            r = CalOutcome::Selected;
        }
        match self.home {
            HomePolicy::Index(primary) => {
                self.primary_focus = primary;
                self.set_start_date(current - Months::new(primary as u32));
                self.focus_lead();
            }
            HomePolicy::Year => {
                let month = current.month0();
                self.primary_focus = month as usize;
                self.set_start_date(current - Months::new(month));
                self.focus_lead();
            }
        }

        r
    }

    /// Select previous day.
    pub fn prev_day(&mut self, n: usize, extend: bool) -> CalOutcome {
        let base_start = self.start_date();
        let base_end = self.end_date();

        let new_date = if let Some(date) = self.selection.lead_selection() {
            if date >= base_start && date <= base_end {
                date - Days::new(n as u64)
            } else if date < base_start {
                self.start_date()
            } else {
                self.end_date()
            }
        } else {
            self.end_date()
        };

        let mut r = CalOutcome::Continue;

        if new_date >= base_start && new_date <= base_end {
            if self.selection.borrow_mut().select_day(new_date, extend) {
                r = CalOutcome::Selected;
            }
        } else if self.step > 0 {
            r = self.scroll_back(self.step);
            if self.selection.borrow_mut().select_day(new_date, extend) {
                r = CalOutcome::Selected;
            }
        }

        if r.is_consumed() {
            self.focus_lead();
        }

        r
    }

    /// Select previous week.
    pub fn next_day(&mut self, n: usize, extend: bool) -> CalOutcome {
        let base_start = self.start_date();
        let base_end = self.end_date();

        let new_date = if let Some(date) = self.selection.lead_selection() {
            if date >= base_start && date <= base_end {
                date + Days::new(n as u64)
            } else if date < base_start {
                self.start_date()
            } else {
                self.end_date()
            }
        } else {
            self.start_date()
        };

        let mut r = CalOutcome::Continue;

        if new_date >= base_start && new_date <= base_end {
            if self.selection.borrow_mut().select_day(new_date, extend) {
                r = CalOutcome::Selected;
            }
        } else if self.step > 0 {
            r = self.scroll_forward(self.step);
            if self.selection.borrow_mut().select_day(new_date, extend) {
                r = CalOutcome::Selected;
            }
        }

        if r.is_consumed() {
            self.focus_lead();
        }

        r
    }

    /// Select previous week.
    pub fn prev_week(&mut self, n: usize, extend: bool) -> CalOutcome {
        let base_start = self.start_date();
        let base_end = self.end_date();

        let mut r = CalOutcome::Continue;

        if let Some(date) = self.selection.lead_selection() {
            let new_date = if date >= base_start && date <= base_end || self.step != 0 {
                date - Days::new(7 * n as u64)
            } else if date < base_start {
                self.start_date()
            } else {
                self.end_date()
            };

            let new_date_end = new_date.week(Weekday::Mon).last_day();

            if new_date_end >= base_start && new_date_end <= base_end {
                if self.selection.borrow_mut().select_week(new_date, extend) {
                    r = CalOutcome::Selected;
                }
            } else if self.step > 0 {
                r = self.scroll_back(self.step);
                if self.selection.borrow_mut().select_week(new_date, extend) {
                    r = CalOutcome::Selected;
                }
            }
        } else {
            let new_date = self.end_date();
            if self.selection.borrow_mut().select_week(new_date, extend) {
                r = CalOutcome::Selected;
            }
        }

        if r.is_consumed() {
            self.focus_lead();
        }

        r
    }

    /// Select previous week.
    pub fn next_week(&mut self, n: usize, extend: bool) -> CalOutcome {
        let base_start = self.start_date();
        let base_end = self.end_date();

        let new_date = if let Some(date) = self.selection.lead_selection() {
            let date_end = date.week(Weekday::Mon).last_day();
            if date_end >= base_start && date_end <= base_end || self.step > 0 {
                date + Days::new(7 * n as u64)
            } else if date_end < base_start {
                self.start_date()
            } else {
                self.end_date()
            }
        } else {
            self.start_date()
        };

        let mut r = CalOutcome::Continue;

        if new_date >= base_start && new_date <= base_end {
            if self.selection.borrow_mut().select_week(new_date, extend) {
                r = CalOutcome::Selected;
            }
        } else if self.step > 0 {
            r = self.scroll_forward(self.step);
            if self.selection.borrow_mut().select_week(new_date, extend) {
                r = CalOutcome::Selected;
            }
        }

        if r.is_consumed() {
            self.focus_lead();
        }

        r
    }
}
