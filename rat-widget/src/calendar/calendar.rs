use crate::_private::NonExhaustive;
use crate::calendar::event::CalOutcome;
use crate::calendar::selection::{NoSelection, RangeSelection, SingleSelection};
use crate::calendar::{CalendarSelection, MonthState};
use crate::text::HasScreenCursor;
use chrono::{Datelike, Days, Local, Months, NaiveDate, Weekday};
use rat_event::ConsumedEvent;
use rat_focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_reloc::RelocatableState;
use ratatui::layout::Rect;
use std::array;
use std::cell::RefCell;
use std::ops::RangeInclusive;
use std::rc::Rc;

/// How should `move_to_today()` behave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodayPolicy {
    /// Set the current date at the given month index and fill
    /// out the rest accordingly.
    Index(usize),
    /// Behave like a yearly calendar. Sets the calendar to the
    /// current year and focuses on the month of the current date.
    Year,
}

impl Default for TodayPolicy {
    fn default() -> Self {
        Self::Index(0)
    }
}

///
/// A struct that contains an array of MonthState to get a true calendar
/// behaviour. There is no Widget for this, the exact layout is left
/// to the user. This takes care of all the rest.
///
#[derive(Debug, Clone)]
pub struct CalendarState<const N: usize, Selection> {
    /// Complete area
    /// __read only__. renewed for each render.
    pub area: Rect,
    /// Area inside the block.
    /// __read only__. renewed for each render.
    pub inner: Rect,
    /// Area inside the block.
    /// __read only__. renewed for each render.
    #[deprecated(since = "2.3.0", note = "use inner instead")]
    pub widget_area: Rect,
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

    /// Behavior of move_to_today.
    home: TodayPolicy,

    /// Primary month.
    /// Only this month will take part in Tab navigation.
    /// The other months will be mouse-reachable only.
    /// You can use arrow-keys to navigate the months though.
    primary_idx: usize,

    /// Months.
    pub months: [MonthState<Selection>; N],

    /// Selection model.
    pub selection: Rc<RefCell<Selection>>,

    /// Calendar focus
    pub focus: FocusFlag,

    pub non_exhaustive: NonExhaustive,
}

impl<const N: usize, Selection> Default for CalendarState<N, Selection>
where
    Selection: Default,
{
    #[allow(deprecated)]
    fn default() -> Self {
        let selection = Rc::new(RefCell::new(Selection::default()));
        let focus = FocusFlag::new();

        Self {
            area: Default::default(),
            inner: Default::default(),
            widget_area: Default::default(),
            step: 1,
            months: array::from_fn(|_| {
                let mut state = MonthState::new();
                state.selection = selection.clone();
                state.container = Some(focus.clone());
                state
            }),
            selection,
            primary_idx: Default::default(),
            focus,
            home: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<const N: usize, Selection> HasFocus for CalendarState<N, Selection> {
    fn build(&self, builder: &mut FocusBuilder) {
        let tag = builder.start(self);
        for (i, v) in self.months.iter().enumerate() {
            if i == self.primary_idx {
                builder.widget(v); // regular widget
            } else {
                builder.widget_with_flags(v.focus(), v.area(), v.area_z(), Navigation::Leave)
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

impl<const N: usize, Selection> HasScreenCursor for CalendarState<N, Selection> {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        None
    }
}

impl<const N: usize, Selection> RelocatableState for CalendarState<N, Selection> {
    #[allow(deprecated)]
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.area.relocate(shift, clip);
        self.inner.relocate(shift, clip);
        self.widget_area.relocate(shift, clip);

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

    pub fn named(name: &str) -> Self
    where
        Selection: Default,
    {
        let mut z = Self::default();
        z.focus = z.focus.with_name(name);
        z
    }

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
    pub fn set_step(&mut self, step: usize) {
        self.step = step;
    }

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
    pub fn step(&self) -> usize {
        self.step
    }

    /// How should move_to_today() work.
    pub fn set_today_policy(&mut self, home: TodayPolicy) {
        if let TodayPolicy::Index(idx) = home {
            assert!(idx < self.months.len());
        }
        self.home = home;
    }

    /// How should move_to_today() work.
    pub fn today_policy(&mut self) -> TodayPolicy {
        self.home
    }

    /// Set the primary month for the calendar.
    ///
    /// The primary month will be focused with Tab navigation.
    /// That means you can jump over all calendar months with just
    /// one Tab.
    ///
    /// Movement within the calendar is possible with the arrow-keys.
    /// This will change the primary month for future Tab navigation.
    pub fn set_primary_idx(&mut self, primary: usize) {
        assert!(primary < self.months.len());
        self.primary_idx = primary;
    }

    /// The primary month for the calendar.
    ///
    /// The primary month will be focused with Tab navigation.
    /// That means you can jump over all calendar months with just
    /// one Tab.
    ///
    /// Movement within the calendar is possible with the arrow-keys.
    /// This will change the primary month for future Tab navigation.
    pub fn primary_idx(&self) -> usize {
        self.primary_idx
    }

    /// Set the start-date for the calendar.
    /// This will set the start-date for each month.
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

    /// Changes the start-date for each month.
    /// Doesn't change any selection.
    pub fn scroll_to(&mut self, date: NaiveDate) -> CalOutcome {
        // change start dates
        let mut start = date - Months::new(self.primary_idx() as u32);

        for i in 0..self.months.len() {
            self.months[i].set_start_date(start);
            start = start + Months::new(1);
        }

        CalOutcome::Changed
    }

    /// Returns None
    pub fn screen_cursor(&self) -> Option<(u16, u16)> {
        None
    }
}

impl<const N: usize, Selection> CalendarState<N, Selection>
where
    Selection: CalendarSelection,
{
    /// Is the date selected?
    pub fn is_selected(&self, date: NaiveDate) -> bool {
        self.selection.is_selected(date)
    }

    /// Lead selection.
    pub fn lead_selection(&self) -> Option<NaiveDate> {
        self.selection.lead_selection()
    }

    pub(super) fn focus_lead(&mut self) -> CalOutcome {
        let Some(lead) = self.selection.lead_selection() else {
            return CalOutcome::Continue;
        };

        let mut r = CalOutcome::Continue;

        if self.is_focused() {
            for (i, month) in self.months.iter().enumerate() {
                if lead >= month.start_date() && lead <= month.end_date() {
                    if self.primary_idx != i {
                        r = CalOutcome::Changed;
                    }
                    self.primary_idx = i;
                    month.focus.set(true);
                } else {
                    month.focus.set(false);
                }
            }
        }

        r
    }

    pub(super) fn focus_n(&mut self, n: usize) -> CalOutcome {
        let mut r = CalOutcome::Continue;

        if self.is_focused() {
            for (i, month) in self.months.iter().enumerate() {
                if i == n {
                    if self.primary_idx != i {
                        r = CalOutcome::Changed;
                    }
                    self.primary_idx = i;
                    month.focus.set(true);
                } else {
                    month.focus.set(false);
                }
            }
        }

        r
    }
}

impl<const N: usize> CalendarState<N, NoSelection> {
    /// Move to the previous month. Scrolls the calendar
    /// if necessary.
    pub fn prev_month(&mut self, n: usize) -> CalOutcome {
        let base_start = self.start_date();
        let date = self.months[self.primary_idx].start_date();

        let prev = date - Months::new(n as u32);

        let mut r = CalOutcome::Continue;
        if prev >= base_start {
            self.focus_n(self.primary_idx - 1);
            r = CalOutcome::Changed;
        } else if self.step() > 0 {
            r = self.scroll_back(self.step());
        }

        r
    }

    /// Move to the next month. Scrolls the calendar
    /// if necessary.
    pub fn next_month(&mut self, n: usize) -> CalOutcome {
        let base_end = self.end_date();
        let date = self.months[self.primary_idx].start_date();

        let next = date + Months::new(n as u32);

        let mut r = CalOutcome::Continue;
        if next <= base_end {
            self.focus_n(self.primary_idx + 1);
            r = CalOutcome::Changed;
        } else if self.step() > 0 {
            r = self.scroll_forward(self.step());
        }
        r
    }

    /// Move the calendar to show the current date.
    ///
    /// Resets the start-dates according to TodayPolicy.
    /// Focuses the primary index and selects the current day.
    ///
    pub fn move_to_today(&mut self) -> CalOutcome {
        self.move_to(Local::now().date_naive())
    }

    pub fn move_to(&mut self, date: NaiveDate) -> CalOutcome {
        let r = CalOutcome::Changed;

        match self.home {
            TodayPolicy::Index(primary) => {
                self.primary_idx = primary;
                self.set_start_date(date - Months::new(primary as u32));
                self.focus_n(self.primary_idx);
            }
            TodayPolicy::Year => {
                let month = date.month0();
                self.primary_idx = month as usize;
                self.set_start_date(date - Months::new(month));
                self.focus_n(self.primary_idx);
            }
        }

        r
    }
}

impl<const N: usize> CalendarState<N, SingleSelection> {
    /// Clear the selection.
    pub fn clear_selection(&mut self) {
        self.selection.borrow_mut().clear();
    }

    /// Select the given date.
    /// Doesn't scroll the calendar.
    pub fn select(&mut self, date: NaiveDate) -> bool {
        self.selection.borrow_mut().select(date)
    }

    /// Selected date.
    pub fn selected(&self) -> Option<NaiveDate> {
        self.selection.borrow().selected()
    }

    /// Move the calendar to show the current date.
    ///
    /// Resets the start-dates according to TodayPolicy.
    /// Focuses the primary index and selects the current day.
    pub fn move_to_today(&mut self) -> CalOutcome {
        self.move_to(Local::now().date_naive())
    }

    /// Move the calendar to show the given date.
    ///
    /// Resets the start-dates according to TodayPolicy.
    /// Focuses the primary index and selects the current day.
    ///
    pub fn move_to(&mut self, date: NaiveDate) -> CalOutcome {
        let mut r = CalOutcome::Changed;

        if self.selection.borrow_mut().select(date) {
            r = CalOutcome::Selected;
        }
        match self.home {
            TodayPolicy::Index(primary) => {
                self.primary_idx = primary;
                self.set_start_date(date - Months::new(primary as u32));
                self.focus_lead();
            }
            TodayPolicy::Year => {
                let month = date.month0();
                self.primary_idx = month as usize;
                self.set_start_date(date - Months::new(month));
                self.focus_lead();
            }
        }

        r
    }

    /// Select previous day.
    /// Scrolls the calendar if necessary.
    pub fn prev_day(&mut self, n: usize) -> CalOutcome {
        self.prev(Months::new(0), Days::new(n as u64))
    }

    /// Select previous week.
    /// Scrolls the calendar if necessary.
    pub fn next_day(&mut self, n: usize) -> CalOutcome {
        self.next(Months::new(0), Days::new(n as u64))
    }

    /// Select the same day in the previous month.
    ///
    /// This may shift the date a bit if it's out of range of the
    /// new month.
    pub fn prev_month(&mut self, n: usize) -> CalOutcome {
        self.prev(Months::new(n as u32), Days::new(0))
    }

    /// Select the same day in the next month.
    ///
    /// This may shift the date a bit if it's out of range of the
    /// new month.
    pub fn next_month(&mut self, n: usize) -> CalOutcome {
        self.next(Months::new(n as u32), Days::new(0))
    }

    /// impl for prev_day(), prev_month()
    fn prev(&mut self, months: Months, days: Days) -> CalOutcome {
        let base_start = self.start_date();
        let base_end = self.end_date();

        let new_date = if let Some(date) = self.selection.lead_selection() {
            if date >= base_start && date <= base_end {
                date - months - days
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

    /// impl for next_day(), next_month()
    fn next(&mut self, months: Months, days: Days) -> CalOutcome {
        let base_start = self.start_date();
        let base_end = self.end_date();

        let new_date = if let Some(date) = self.selection.lead_selection() {
            if date >= base_start && date <= base_end {
                date + months + days
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
    /// Clear the selection.
    pub fn clear_selection(&mut self) {
        self.selection.borrow_mut().clear();
    }

    /// Select the full month of the date. Any date of the month will do.
    /// Can extend the selection to encompass the month.
    /// Any existing selection is buffed up to fill one month.
    pub fn select_month(&mut self, date: NaiveDate, extend: bool) -> bool {
        self.selection.borrow_mut().select_month(date, extend)
    }

    /// Select the week of the given date. Any date of the week will do.
    /// Can extend the selection to encompass the week.
    /// Any existing selection is buffed up to fill one week.
    pub fn select_week(&mut self, date: NaiveDate, extend: bool) -> bool {
        self.selection.borrow_mut().select_week(date, extend)
    }

    /// Select the given date.
    /// Can extend the selection to the given date.
    pub fn select_day(&mut self, date: NaiveDate, extend: bool) -> bool {
        self.selection.borrow_mut().select_day(date, extend)
    }

    /// Set the selection as (anchor, lead) pair.
    pub fn select(&mut self, selection: (NaiveDate, NaiveDate)) -> bool {
        self.selection.borrow_mut().select(selection)
    }

    /// Selection as (anchor, lead) pair.
    pub fn selected(&self) -> Option<(NaiveDate, NaiveDate)> {
        self.selection.borrow().selected()
    }

    /// Selection as NaiveDate range.
    pub fn selected_range(&self) -> Option<RangeInclusive<NaiveDate>> {
        self.selection.borrow().selected_range()
    }

    /// Move the calendar to today.
    ///
    /// Uses [TodayPolicy] to build the new calendar and focuses and selects
    /// the given date.
    pub fn move_to_today(&mut self) -> CalOutcome {
        self.move_to(Local::now().date_naive())
    }

    /// Move the calendar the given day.
    ///
    /// Uses [TodayPolicy] to build the new calendar and focuses and selects
    /// the given date.
    pub fn move_to(&mut self, date: NaiveDate) -> CalOutcome {
        let mut r = CalOutcome::Changed;

        if self.selection.borrow_mut().select_day(date, false) {
            r = CalOutcome::Selected;
        }
        match self.home {
            TodayPolicy::Index(primary) => {
                self.primary_idx = primary;
                self.set_start_date(date - Months::new(primary as u32));
                self.focus_lead();
            }
            TodayPolicy::Year => {
                let month = date.month0();
                self.primary_idx = month as usize;
                self.set_start_date(date - Months::new(month));
                self.focus_lead();
            }
        }

        r
    }

    /// Move the lead selection back by months/days.
    /// Clears the current selection and scrolls if necessary.
    pub fn move_to_prev(&mut self, months: Months, days: Days) -> CalOutcome {
        let base_start = self.start_date();
        let base_end = self.end_date();

        let new_date = if let Some(date) = self.selection.lead_selection() {
            if date >= base_start && date <= base_end {
                date - months - days
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
            if self.selection.borrow_mut().select_day(new_date, false) {
                r = CalOutcome::Selected;
            }
        } else if self.step > 0 {
            r = r.max(self.scroll_back(self.step));
            if self.selection.borrow_mut().select_day(new_date, false) {
                r = r.max(CalOutcome::Selected);
            }
        }

        if r.is_consumed() {
            self.focus_lead();
        }

        r
    }

    /// Move the lead selection forward by months/days.
    /// Clears the current selection and scrolls if necessary.
    pub fn move_to_next(&mut self, months: Months, days: Days) -> CalOutcome {
        let base_start = self.start_date();
        let base_end = self.end_date();

        let new_date = if let Some(date) = self.selection.lead_selection() {
            if date >= base_start && date <= base_end {
                date + months + days
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
            if self.selection.borrow_mut().select_day(new_date, false) {
                r = CalOutcome::Selected;
            }
        } else if self.step > 0 {
            r = self.scroll_forward(self.step);
            if self.selection.borrow_mut().select_day(new_date, false) {
                r = r.max(CalOutcome::Selected);
            }
        }

        if r.is_consumed() {
            self.focus_lead();
        }

        r
    }

    /// Select previous day.
    ///
    /// Can extend the selection to include the new date.
    pub fn prev_day(&mut self, n: usize, extend: bool) -> CalOutcome {
        let base_start = self.start_date();
        let base_end = self.end_date();

        let mut r = CalOutcome::Continue;

        if let Some(date) = self.selection.lead_selection() {
            let new_date = if date >= base_start && date <= base_end || self.step != 0 {
                date - Days::new(n as u64)
            } else if date < base_start {
                self.start_date()
            } else {
                self.end_date()
            };

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
        } else {
            let new_date = self.end_date();
            if self.selection.borrow_mut().select_day(new_date, extend) {
                r = CalOutcome::Selected;
            }
        }

        if r.is_consumed() {
            self.focus_lead();
        }

        r
    }

    /// Select next day.
    ///
    /// Can extend the selection to include the new date.
    pub fn next_day(&mut self, n: usize, extend: bool) -> CalOutcome {
        let base_start = self.start_date();
        let base_end = self.end_date();

        let new_date = if let Some(date) = self.selection.lead_selection() {
            if date >= base_start && date <= base_end || self.step > 0 {
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

    /// Select the previous week.
    ///
    /// When extending a selection, the current selection buffs up to fill one week.
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

    /// Select the next week.
    ///
    /// When extending a selection, the current selection buffs up to fill one week.
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

    /// Select the previous month.
    ///
    /// When extending a selection, the current selection buffs up to fill one month.
    ///
    pub fn prev_month(&mut self, n: usize, extend: bool) -> CalOutcome {
        let base_start = self.start_date();
        let base_end = self.end_date();

        let mut r = CalOutcome::Continue;

        if let Some(date) = self.selection.lead_selection() {
            let new_date = if date >= base_start && date <= base_end || self.step != 0 {
                date - Months::new(n as u32)
            } else if date < base_start {
                self.start_date()
            } else {
                self.end_date()
            };

            if new_date >= base_start && new_date <= base_end {
                if self.selection.borrow_mut().select_month(new_date, extend) {
                    r = CalOutcome::Selected;
                }
            } else if self.step > 0 {
                r = self.scroll_back(self.step);
                if self.selection.borrow_mut().select_month(new_date, extend) {
                    r = CalOutcome::Selected;
                }
            }
        } else {
            let new_date = self.end_date();
            if self.selection.borrow_mut().select_month(new_date, extend) {
                r = CalOutcome::Selected;
            }
        }

        if r.is_consumed() {
            self.focus_lead();
        }

        r
    }

    /// Select the previous month.
    ///
    /// When extending a selection, the current selection buffs up to fill one month.
    ///
    pub fn next_month(&mut self, n: usize, extend: bool) -> CalOutcome {
        let base_start = self.start_date();
        let base_end = self.end_date();

        let new_date = if let Some(date) = self.selection.lead_selection() {
            if date >= base_start && date <= base_end || self.step > 0 {
                date + Months::new(n as u32)
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
            if self.selection.borrow_mut().select_month(new_date, extend) {
                r = CalOutcome::Selected;
            }
        } else if self.step > 0 {
            r = self.scroll_forward(self.step);
            if self.selection.borrow_mut().select_month(new_date, extend) {
                r = CalOutcome::Selected;
            }
        }

        if r.is_consumed() {
            self.focus_lead();
        }

        r
    }
}
