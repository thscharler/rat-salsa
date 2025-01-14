//!
//! Render a month of a calendar.
//! Can be localized with a chrono::Locale.
//!

use crate::_private::NonExhaustive;
use crate::calendar::event::CalOutcome;
use crate::util::{block_size, revert_style};
use chrono::{Datelike, Days, Months, NaiveDate, Weekday};
use rat_event::util::MouseFlagsN;
use rat_event::{ct_event, flow, ConsumedEvent, HandleEvent, MouseOnly, Regular};
use rat_focus::{Focus, FocusBuilder, FocusFlag, HasFocus};
use rat_reloc::{relocate_area, relocate_areas, RelocatableState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::block::Title;
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::StatefulWidgetRef;
use ratatui::widgets::{Block, StatefulWidget, Widget};
use std::cmp::max;
use std::collections::HashMap;
use std::fmt::Debug;

/// Renders a month.
#[derive(Debug, Default, Clone)]
pub struct Month<'a> {
    /// Start date of the month.
    start_date: Option<NaiveDate>,

    /// Base style.
    style: Style,
    /// Title style.
    title_style: Option<Style>,
    /// Title align.
    title_align: Alignment,
    /// Week number style.
    week_style: Option<Style>,
    /// Week day style.
    weekday_style: Option<Style>,
    /// Default day style.
    day_style: Option<Style>,
    /// Styling for a single date.
    day_styles: Option<&'a HashMap<NaiveDate, Style>>,
    /// Selection
    select_style: Option<Style>,
    /// Focus
    focus_style: Option<Style>,
    /// Selection
    day_selection: bool,
    week_selection: bool,
    show_weekdays: bool,

    /// Block
    block: Option<Block<'a>>,

    /// Locale
    loc: chrono::Locale,
}

/// Composite style for the calendar.
#[derive(Debug, Clone)]
pub struct MonthStyle {
    pub style: Style,
    pub title: Option<Style>,
    pub week: Option<Style>,
    pub weekday: Option<Style>,
    pub day: Option<Style>,
    pub select: Option<Style>,
    pub focus: Option<Style>,
    pub block: Option<Block<'static>>,
    pub non_exhaustive: NonExhaustive,
}

/// State & event-handling.
#[derive(Debug)]
pub struct MonthState {
    /// Total area.
    /// __readonly__. renewed for each render.
    pub area: Rect,
    /// Area inside the border.
    /// __readonly__. renewed for each render.
    pub inner: Rect,
    /// Area for the days of the month.
    /// __readonly__. renewed for each render.
    pub area_days: [Rect; 31],
    /// Area for the week numbers.
    /// __readonly__. renewed for each render.
    pub area_weeks: [Rect; 6],
    /// Startdate
    start_date: NaiveDate,

    /// Day selection enabled
    day_selection: bool,
    /// Week selection enabled
    week_selection: bool,

    /// Selected week
    selected_week: Option<usize>,
    /// Selected day
    selected_day: Option<usize>,

    /// Focus
    /// __read+write__
    pub focus: FocusFlag,
    /// Mouse flags
    /// __read+write__
    pub mouse: MouseFlagsN,

    pub non_exhaustive: NonExhaustive,
}

impl Default for MonthStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            title: None,
            week: None,
            weekday: None,
            day: None,
            select: None,
            focus: None,
            block: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a> Month<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the starting date. This can be any date of the month.
    /// If no date is set, the start_date of the state is used.
    #[inline]
    pub fn date(mut self, s: NaiveDate) -> Self {
        self.start_date = Some(s.with_day(1).expect("day"));
        self
    }

    /// Locale for month-names, day-names.
    #[inline]
    pub fn locale(mut self, loc: chrono::Locale) -> Self {
        self.loc = loc;
        self
    }

    /// Date selection enabled
    #[inline]
    pub fn day_selection(mut self) -> Self {
        self.day_selection = true;
        self
    }

    /// Week selection enabled
    #[inline]
    pub fn week_selection(mut self) -> Self {
        self.week_selection = true;
        self
    }

    /// Show weekday titles
    #[inline]
    pub fn show_weekdays(mut self) -> Self {
        self.show_weekdays = true;
        self
    }

    /// Set the composite style.
    #[inline]
    pub fn styles(mut self, s: MonthStyle) -> Self {
        self.style = s.style;
        if s.title.is_some() {
            self.title_style = s.title;
        }
        if s.week.is_some() {
            self.week_style = s.week;
        }
        if s.weekday.is_some() {
            self.weekday_style = s.weekday;
        }
        if s.day.is_some() {
            self.day_style = s.day;
        }
        if s.select.is_some() {
            self.select_style = s.select;
        }
        if s.focus.is_some() {
            self.focus_style = s.focus;
        }
        if s.block.is_some() {
            self.block = s.block;
        }
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Style for the selected tab.
    pub fn select_style(mut self, style: Style) -> Self {
        self.select_style = Some(style);
        self
    }

    /// Style for a focused tab.
    pub fn focus_style(mut self, style: Style) -> Self {
        self.focus_style = Some(style);
        self
    }

    /// Sets the default day-style.
    #[inline]
    pub fn day_style(mut self, s: impl Into<Style>) -> Self {
        self.day_style = Some(s.into());
        self
    }

    /// Sets all the day-styles.
    #[inline]
    pub fn day_styles(mut self, styles: &'a HashMap<NaiveDate, Style>) -> Self {
        self.day_styles = Some(styles);
        self
    }

    /// Set the week number style
    #[inline]
    pub fn week_style(mut self, s: impl Into<Style>) -> Self {
        self.week_style = Some(s.into());
        self
    }

    /// Set the week day style
    #[inline]
    pub fn weekday_style(mut self, s: impl Into<Style>) -> Self {
        self.weekday_style = Some(s.into());
        self
    }

    /// Set the month-name style.
    #[inline]
    pub fn title_style(mut self, s: impl Into<Style>) -> Self {
        self.title_style = Some(s.into());
        self
    }

    /// Set the mont-name align.
    #[inline]
    pub fn title_align(mut self, a: Alignment) -> Self {
        self.title_align = a;
        self
    }

    /// Block
    #[inline]
    pub fn block(mut self, b: Block<'a>) -> Self {
        self.block = Some(b);
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Inherent width of the widget.
    #[inline]
    pub fn width(&self) -> u16 {
        8 * 3 + block_size(&self.block).width
    }

    /// Inherent height for the widget.
    /// Can vary with the number of months.
    #[inline]
    pub fn height(&self, state: &MonthState) -> u16 {
        let start_date = if let Some(start_date) = self.start_date {
            start_date
        } else {
            state.start_date
        };

        let r = MonthState::count_weeks(start_date) as u16;
        let w = if self.show_weekdays { 1 } else { 0 };
        let b = max(1, block_size(&self.block).height);
        r + w + b
    }
}

#[cfg(feature = "unstable-widget-ref")]
impl<'a> StatefulWidgetRef for Month<'a> {
    type State = MonthState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

impl StatefulWidget for Month<'_> {
    type State = MonthState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

fn render_ref(widget: &Month<'_>, area: Rect, buf: &mut Buffer, state: &mut MonthState) {
    state.area = area;
    if let Some(start_date) = widget.start_date {
        state.start_date = start_date;
    }
    state.day_selection = widget.day_selection;
    state.week_selection = widget.week_selection;

    let mut day = state.start_date;

    let focus_style = widget.focus_style.unwrap_or(revert_style(widget.style));
    let select_style = if let Some(select_style) = widget.select_style {
        if state.focus.get() {
            focus_style
        } else {
            select_style
        }
    } else {
        if state.focus.get() {
            focus_style
        } else {
            revert_style(widget.style)
        }
    };
    let day_style = widget.day_style.unwrap_or(widget.style);
    let week_style = widget.week_style.unwrap_or(widget.style);
    let weekday_style = widget.weekday_style.unwrap_or(widget.style);

    let title_style = if let Some(title_style) = widget.title_style {
        title_style
    } else {
        widget.style
    };
    let title_style = if state.is_focused() {
        title_style.patch(focus_style)
    } else {
        title_style
    };

    let block = if let Some(block) = widget.block.clone() {
        block
            .title(Title::from(
                day.format_localized("%B", widget.loc).to_string(),
            ))
            .title_style(title_style)
            .title_alignment(widget.title_align)
    } else {
        Block::new()
            .style(widget.style)
            .title(Title::from(
                day.format_localized("%B", widget.loc).to_string(),
            ))
            .title_style(title_style)
            .title_alignment(widget.title_align)
    };

    state.inner = block.inner(area);
    block.render(area, buf);

    let month = state.start_date.month();
    let mut w = 0;
    let mut x = state.inner.x;
    let mut y = state.inner.y;

    // week days
    if widget.show_weekdays {
        x += 3;
        buf.set_style(Rect::new(x, y, 3 * 7, 1), weekday_style);
        for wd in [
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
            Weekday::Sun,
        ] {
            let area = Rect::new(x, y, 2, 1).intersection(state.inner);

            let day = NaiveDate::from_weekday_of_month_opt(2024, 1, wd, 1).expect("date");
            let day_name = day.format_localized("%a", widget.loc).to_string();
            Span::from(format!("{:2} ", day_name)).render(area, buf);

            x += 3;
        }
        x = state.inner.x;
        y += 1;
    }

    // first line may omit a few days
    state.area_weeks[w] = Rect::new(x, y, 2, 1).intersection(state.inner);
    Span::from(day.format_localized("%V", widget.loc).to_string())
        .style(week_style)
        .render(state.area_weeks[w], buf);

    let week_sel = if state.selected_week == Some(w) {
        let week_bg = Rect::new(x + 3, y, 21, 1).intersection(state.inner);
        buf.set_style(week_bg, select_style);
        true
    } else {
        false
    };

    x += 3;

    for wd in [
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ] {
        if day.weekday() != wd {
            x += 3;
        } else {
            let day_style = if let Some(day_styles) = widget.day_styles {
                if let Some(day_style) = day_styles.get(&day) {
                    *day_style
                } else {
                    day_style
                }
            } else {
                day_style
            };
            let day_style = if week_sel || state.selected_day == Some(day.day0() as usize) {
                day_style.patch(select_style)
            } else {
                day_style
            };

            state.area_days[day.day0() as usize] = Rect::new(x, y, 2, 1).intersection(state.inner);

            Span::from(day.format_localized("%e", widget.loc).to_string())
                .style(day_style)
                .render(state.area_days[day.day0() as usize], buf);

            x += 3;
            day += chrono::Duration::try_days(1).expect("days");
        }
    }

    w += 1;
    x = state.inner.x;
    y += 1;

    while month == day.month() {
        state.area_weeks[w] = Rect::new(x, y, 2, 1).intersection(state.inner);
        Span::from(day.format_localized("%V", widget.loc).to_string())
            .style(week_style)
            .render(state.area_weeks[w], buf);

        let week_sel = if state.selected_week == Some(w) {
            let week_bg = Rect::new(x + 3, y, 21, 1).intersection(state.inner);
            buf.set_style(week_bg, select_style);
            true
        } else {
            false
        };

        x += 3;

        for _ in 0..7 {
            if day.month() == month {
                let day_style = if let Some(day_styles) = widget.day_styles {
                    if let Some(day_style) = day_styles.get(&day) {
                        *day_style
                    } else {
                        day_style
                    }
                } else {
                    day_style
                };
                let day_style = if week_sel || state.selected_day == Some(day.day0() as usize) {
                    day_style.patch(select_style)
                } else {
                    day_style
                };

                state.area_days[day.day0() as usize] =
                    Rect::new(x, y, 2, 1).intersection(state.inner);

                Span::from(day.format_localized("%e", widget.loc).to_string())
                    .style(day_style)
                    .render(state.area_days[day.day0() as usize], buf);

                x += 3;
                day += chrono::Duration::try_days(1).expect("days");
            } else {
                x += 3;
            }
        }

        w += 1;
        x = state.inner.x;
        y += 1;
    }
}

impl HasFocus for MonthState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.append_leaf(self);
    }

    #[inline]
    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    #[inline]
    fn area(&self) -> Rect {
        self.area
    }
}

impl RelocatableState for MonthState {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.area = relocate_area(self.area, shift, clip);
        self.inner = relocate_area(self.inner, shift, clip);
        relocate_areas(&mut self.area_days, shift, clip);
        relocate_areas(&mut self.area_weeks, shift, clip);
    }
}

impl Clone for MonthState {
    fn clone(&self) -> Self {
        Self {
            area: self.area,
            inner: self.inner,
            area_days: self.area_days.clone(),
            area_weeks: self.area_weeks.clone(),
            start_date: self.start_date,
            day_selection: self.day_selection,
            week_selection: self.week_selection,
            selected_week: self.selected_week,
            selected_day: self.selected_day,
            focus: FocusFlag::named(self.focus.name()),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl Default for MonthState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            inner: Default::default(),
            area_days: [Rect::default(); 31],
            area_weeks: [Rect::default(); 6],
            start_date: Default::default(),
            day_selection: false,
            week_selection: false,
            selected_week: Default::default(),
            selected_day: Default::default(),
            focus: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl MonthState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn named(name: &str) -> Self {
        Self {
            focus: FocusFlag::named(name),
            ..Self::default()
        }
    }

    /// Sets the start-date of the calendar. You can set every date, it
    /// will always be changed to the first of the month.
    ///
    /// Setting this will be useless if the date is set with the Month widget.
    pub fn set_start_date(&mut self, date: NaiveDate) -> bool {
        let old_value = self.start_date;
        self.start_date = date.with_day(1).expect("date");
        old_value != self.start_date
    }

    /// Start date of this month. Will always be the first.
    pub fn start_date(&self) -> NaiveDate {
        self.start_date
    }

    /// Removes all selection.
    pub fn clear_selection(&mut self) {
        self.selected_week = None;
        self.selected_day = None;
    }

    /// Select a week.
    pub fn select_week(&mut self, n: Option<usize>) {
        self.selected_week = n;
        self.selected_day = None;
    }

    /// Select a week by date
    /// Returns true if the date is valid for this month.
    /// If false it doesn't change the selection.
    pub fn select_week_by_date(&mut self, d: Option<NaiveDate>) -> bool {
        self.selected_day = None;
        if let Some(d) = d {
            if d.year() == self.start_date.year() {
                if let Some(w) = self.date_as_week(d) {
                    self.selected_week = Some(w);
                    true
                } else {
                    self.selected_week = None;
                    false
                }
            } else {
                self.selected_week = None;
                false
            }
        } else {
            self.selected_week = None;
            true
        }
    }

    /// Selected week
    pub fn selected_week(&mut self) -> Option<usize> {
        self.selected_week
    }

    /// Selected week
    pub fn selected_week_as_date(&mut self) -> Option<NaiveDate> {
        self.selected_week.map(|v| self.week_day(v))
    }

    /// Select a day
    pub fn select_day(&mut self, n: Option<usize>) {
        self.selected_day = n;
        self.selected_week = None;
    }

    /// Select by date.
    /// Returns true if the date is valid for this month.
    /// If false it doesn't change the selection.
    pub fn select_date(&mut self, d: Option<NaiveDate>) -> bool {
        self.selected_week = None;
        if let Some(d) = d {
            if d.year() == self.start_date.year() && d.month() == self.start_date.month() {
                self.selected_day = Some(d.day0() as usize);
                true
            } else {
                self.selected_day = None;
                false
            }
        } else {
            self.selected_day = None;
            true
        }
    }

    /// Selected day
    pub fn selected_day(&mut self) -> Option<usize> {
        self.selected_day
    }

    /// Selected day
    pub fn selected_day_as_date(&mut self) -> Option<NaiveDate> {
        self.selected_day.map(|v| self.month_day(v))
    }

    /// Select previous day.
    pub fn prev_day(&mut self, n: usize) -> bool {
        if let Some(sel) = self.selected_week {
            let week_day = self.week_day(sel);
            if week_day < self.start_date {
                self.selected_day = Some(0);
            } else {
                self.selected_day = Some(week_day.day0() as usize);
            }
            self.selected_week = None;
        }

        if let Some(sel) = self.selected_day {
            if sel >= n {
                self.selected_day = Some(sel - n);
                true
            } else {
                false
            }
        } else {
            let mut d = 30;
            loop {
                if self.start_date.with_day0(d).is_some() {
                    break;
                }
                d -= 1;
            }
            self.selected_day = Some(d as usize);
            true
        }
    }

    /// Select next day.
    pub fn next_day(&mut self, n: usize) -> bool {
        if let Some(sel) = self.selected_week {
            let week_day = self.week_day(sel);
            if week_day < self.start_date {
                self.selected_day = Some(0);
            } else {
                self.selected_day = Some(week_day.day0() as usize);
            }
            self.selected_week = None;
        }

        if let Some(sel) = self.selected_day {
            if self.start_date.with_day0(sel as u32 + n as u32).is_some() {
                self.selected_day = Some(sel + n);
                true
            } else {
                false
            }
        } else {
            self.selected_day = Some(0);
            true
        }
    }

    /// Select previous week.
    pub fn prev_week(&mut self, n: usize) -> bool {
        if let Some(sel) = self.selected_day {
            self.selected_week = self.month_day_as_week(sel);
            self.selected_day = None;
        }
        if let Some(sel) = self.selected_week {
            if sel >= n {
                self.selected_week = Some(sel - n);
                true
            } else {
                false
            }
        } else {
            let mut d = 30;
            loop {
                if self.start_date.with_day0(d).is_some() {
                    break;
                }
                d -= 1;
            }
            self.selected_week = self.month_day_as_week(d as usize);
            true
        }
    }

    /// Select next week.
    pub fn next_week(&mut self, n: usize) -> bool {
        if let Some(sel) = self.selected_day {
            self.selected_week = self.month_day_as_week(sel);
            self.selected_day = None;
        }
        if let Some(sel) = self.selected_week {
            let sel_day = self.week_day(sel);
            let new_day = sel_day + chrono::Duration::try_days(7 * n as i64).expect("days");
            if self.start_date.month() == new_day.month() {
                self.selected_week = self.month_day_as_week(new_day.day0() as usize);
                true
            } else {
                false
            }
        } else {
            self.selected_week = Some(0);
            true
        }
    }

    /// Monday of the nth displayed week
    pub fn week_day(&self, n: usize) -> NaiveDate {
        let mut day = self.start_date;
        while day.weekday() != Weekday::Mon {
            day -= chrono::Duration::try_days(1).expect("days");
        }
        day += chrono::Duration::try_days(7 * n as i64).expect("days");
        day
    }

    /// Date of the nth displayed date
    pub fn month_day(&self, n: usize) -> NaiveDate {
        let mut day = self.start_date;
        day += chrono::Duration::try_days(n as i64).expect("days");
        day
    }

    /// Week of the nth displayed date
    pub fn month_day_as_week(&self, n: usize) -> Option<usize> {
        if let Some(day) = self.start_date.with_day0(n as u32) {
            self.date_as_week(day)
        } else {
            None
        }
    }

    /// Week of the given date
    pub fn date_as_week(&self, d: NaiveDate) -> Option<usize> {
        let mut day = self.start_date;
        let month = day.month();
        let mut w = 0;

        while month == day.month() {
            if day.week(Weekday::Mon).days().contains(&d) {
                return Some(w);
            }
            day += chrono::Duration::try_days(7).expect("days");
            w += 1;
        }
        // last week might be next month
        let week = day.week(Weekday::Mon);
        if week.first_day().month() == month {
            if week.days().contains(&d) {
                return Some(w);
            }
        }
        None
    }

    /// Nr of weeks in this month.
    pub fn week_len(&self) -> usize {
        Self::count_weeks(self.start_date)
    }

    /// Nr of weeks for the given month
    pub fn count_weeks(day: NaiveDate) -> usize {
        let mut day = day.with_day0(0).expect("date");
        let month = day.month();

        let mut weeks = 1;
        for weekday in [
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
            Weekday::Sun,
        ] {
            // run through first week
            if day.weekday() == weekday {
                day += chrono::Duration::try_days(1).expect("days");
            }
        }
        // count mondays
        while month == day.month() {
            weeks += 1;
            day += chrono::Duration::try_days(7).expect("days");
        }

        weeks
    }
}

pub(crate) mod event {
    use chrono::NaiveDate;
    use rat_event::{ConsumedEvent, Outcome};

    /// Result of event handling.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum CalOutcome {
        /// The given event has not been used at all.
        Continue,
        /// The event has been recognized, but the result was nil.
        /// Further processing for this event may stop.
        Unchanged,
        /// The event has been recognized and there is some change
        /// due to it.
        /// Further processing for this event may stop.
        /// Rendering the ui is advised.
        Changed,
        /// Week selected. This is Monday of the selected week.
        Week(NaiveDate),
        /// Day selected.
        /// Selected tab should be closed.
        Day(NaiveDate),
    }

    impl ConsumedEvent for CalOutcome {
        fn is_consumed(&self) -> bool {
            *self != CalOutcome::Continue
        }
    }

    // Useful for converting most navigation/edit results.
    impl From<bool> for CalOutcome {
        fn from(value: bool) -> Self {
            if value {
                CalOutcome::Changed
            } else {
                CalOutcome::Unchanged
            }
        }
    }

    impl From<Outcome> for CalOutcome {
        fn from(value: Outcome) -> Self {
            match value {
                Outcome::Continue => CalOutcome::Continue,
                Outcome::Unchanged => CalOutcome::Unchanged,
                Outcome::Changed => CalOutcome::Changed,
            }
        }
    }

    impl From<CalOutcome> for Outcome {
        fn from(value: CalOutcome) -> Self {
            match value {
                CalOutcome::Continue => Outcome::Continue,
                CalOutcome::Unchanged => Outcome::Unchanged,
                CalOutcome::Changed => Outcome::Changed,
                CalOutcome::Week(_) => Outcome::Changed,
                CalOutcome::Day(_) => Outcome::Changed,
            }
        }
    }
}

impl HandleEvent<crossterm::event::Event, Regular, CalOutcome> for MonthState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> CalOutcome {
        if self.is_focused() {
            flow!(match event {
                ct_event!(keycode press Up) => {
                    if !self.day_selection {
                        return CalOutcome::Continue;
                    }
                    if self.prev_day(7) {
                        CalOutcome::Day(self.selected_day_as_date().expect("day"))
                    } else {
                        CalOutcome::Continue
                    }
                }
                ct_event!(keycode press Down) => {
                    if !self.day_selection {
                        return CalOutcome::Continue;
                    }
                    if self.next_day(7) {
                        CalOutcome::Day(self.selected_day_as_date().expect("day"))
                    } else {
                        CalOutcome::Continue
                    }
                }
                ct_event!(keycode press Left) => {
                    if !self.day_selection {
                        return CalOutcome::Continue;
                    }
                    if self.prev_day(1) {
                        CalOutcome::Day(self.selected_day_as_date().expect("day"))
                    } else {
                        CalOutcome::Continue
                    }
                }
                ct_event!(keycode press Right) => {
                    if !self.day_selection {
                        return CalOutcome::Continue;
                    }
                    if self.next_day(1) {
                        CalOutcome::Day(self.selected_day_as_date().expect("day"))
                    } else {
                        CalOutcome::Continue
                    }
                }
                ct_event!(keycode press ALT-Up) => {
                    if !self.week_selection {
                        return CalOutcome::Continue;
                    }
                    if self.prev_week(1) {
                        CalOutcome::Week(self.selected_week_as_date().expect("week"))
                    } else {
                        CalOutcome::Continue
                    }
                }
                ct_event!(keycode press ALT-Down) => {
                    if !self.week_selection {
                        return CalOutcome::Continue;
                    }
                    if self.next_week(1) {
                        CalOutcome::Week(self.selected_week_as_date().expect("week"))
                    } else {
                        CalOutcome::Continue
                    }
                }
                _ => CalOutcome::Continue,
            })
        }

        self.handle(event, MouseOnly)
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, CalOutcome> for MonthState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> CalOutcome {
        match event {
            ct_event!(mouse drag Left for x, y) | ct_event!(mouse down Left for x, y) => {
                if let Some(sel) = self.mouse.item_at(&self.area_weeks, *x, *y) {
                    if !self.week_selection {
                        return CalOutcome::Continue;
                    }
                    self.select_week(Some(sel));
                    CalOutcome::Week(self.week_day(sel))
                } else if let Some(sel) = self.mouse.item_at(&self.area_days, *x, *y) {
                    if !self.day_selection {
                        return CalOutcome::Continue;
                    }
                    self.select_day(Some(sel));
                    CalOutcome::Day(self.month_day(sel))
                } else {
                    CalOutcome::Continue
                }
            }

            _ => CalOutcome::Continue,
        }
    }
}

fn scroll_up_month_list(months: &mut [MonthState], delta: u32) {
    // change start dates
    let mut start = months[0]
        .start_date
        .checked_sub_months(Months::new(delta))
        .expect("date");
    for i in 0..months.len() {
        months[i].start_date = start;
        start = start.checked_add_months(Months::new(1)).expect("date");
    }
}

fn scroll_down_month_list(months: &mut [MonthState], delta: u32) {
    // change start dates
    let mut start = months[0]
        .start_date
        .checked_add_months(Months::new(delta))
        .expect("date");
    for i in 0..months.len() {
        months[i].start_date = start;
        start = start.checked_add_months(Months::new(1)).expect("date");
    }
}

/// Multi-month event-handling.
///
/// The parameter gives the shift of the calendar in months, if
/// the newly selected date is not visible with the current settings.
///
/// This also needs the Focus, as it changes the focused month if necessary.
pub struct MultiMonth<'a>(pub &'a Focus, pub u32);

impl HandleEvent<crossterm::event::Event, MultiMonth<'_>, CalOutcome> for &mut [MonthState] {
    fn handle(&mut self, event: &crossterm::event::Event, arg: MultiMonth<'_>) -> CalOutcome {
        let r = 'f: {
            for i in 0..self.len() {
                let month = &mut self[i];
                if month.is_focused() {
                    let r = match month.handle(event, Regular) {
                        CalOutcome::Week(date) => {
                            for j in 0..self.len() {
                                if i != j {
                                    self[j].clear_selection();
                                }
                            }
                            if i > 0 {
                                self[i - 1].select_week_by_date(Some(date));
                            }
                            if i + 1 < self.len() {
                                self[i + 1].select_week_by_date(Some(date));
                            }
                            CalOutcome::Week(date)
                        }
                        CalOutcome::Day(date) => {
                            for j in 0..self.len() {
                                if i != j {
                                    self[j].clear_selection();
                                }
                            }
                            CalOutcome::Day(date)
                        }
                        CalOutcome::Continue => match event {
                            ct_event!(keycode press Up) => {
                                if !self[i].day_selection {
                                    return CalOutcome::Continue;
                                }
                                if i > 0 {
                                    if let Some(date) = self[i].selected_day_as_date() {
                                        let new_date =
                                            date - chrono::Duration::try_days(7).expect("days");
                                        self[i].select_day(None);
                                        self[i - 1].select_date(Some(new_date));
                                        arg.0.focus(&self[i - 1]);
                                        CalOutcome::Day(new_date)
                                    } else {
                                        // ?? what is this. invalid day??
                                        CalOutcome::Continue
                                    }
                                } else {
                                    if let Some(date) = self[i].selected_day_as_date() {
                                        let new_date =
                                            date - chrono::Duration::try_days(7).expect("days");
                                        scroll_up_month_list(self, arg.1);
                                        // date may be anywhere (even nowhere) after the shift.
                                        for i in 0..self.len() {
                                            self[i].select_date(Some(new_date));
                                        }
                                        for i in 0..self.len() {
                                            if self[i].selected_day_as_date().is_some() {
                                                arg.0.focus(&self[i]);
                                            }
                                        }
                                        CalOutcome::Day(new_date)
                                    } else {
                                        CalOutcome::Continue
                                    }
                                }
                            }
                            ct_event!(keycode press Down) => {
                                if !self[i].day_selection {
                                    return CalOutcome::Continue;
                                }
                                if i + 1 < self.len() {
                                    if let Some(date) = self[i].selected_day_as_date() {
                                        let new_date =
                                            date + chrono::Duration::try_days(7).expect("days");
                                        self[i].select_day(None);
                                        self[i + 1].select_date(Some(new_date));
                                        arg.0.focus(&self[i + 1]);
                                        CalOutcome::Day(new_date)
                                    } else {
                                        CalOutcome::Continue
                                    }
                                } else {
                                    if let Some(date) = self[i].selected_day_as_date() {
                                        let new_date =
                                            date + chrono::Duration::try_days(7).expect("days");
                                        scroll_down_month_list(self, arg.1);
                                        // date may be anywhere (even nowhere) after the shift.
                                        for i in 0..self.len() {
                                            self[i].select_date(Some(new_date));
                                        }
                                        for i in 0..self.len() {
                                            if self[i].selected_day_as_date().is_some() {
                                                arg.0.focus(&self[i]);
                                            }
                                        }
                                        CalOutcome::Day(new_date)
                                    } else {
                                        CalOutcome::Continue
                                    }
                                }
                            }
                            ct_event!(keycode press Left) => {
                                if !self[i].day_selection {
                                    return CalOutcome::Continue;
                                }
                                if i > 0 {
                                    let prev_day = self[i]
                                        .start_date
                                        .checked_sub_days(Days::new(1))
                                        .expect("date");

                                    self[i].select_day(None);
                                    self[i - 1].select_date(Some(prev_day));
                                    arg.0.focus(&self[i - 1]);

                                    CalOutcome::Day(prev_day)
                                } else {
                                    let new_date = self[i]
                                        .start_date
                                        .checked_sub_days(Days::new(1))
                                        .expect("date");
                                    scroll_up_month_list(self, 1);
                                    // date may be anywhere (even nowhere) after the shift.
                                    for i in 0..self.len() {
                                        self[i].select_date(Some(new_date));
                                    }
                                    for i in 0..self.len() {
                                        if self[i].selected_day_as_date().is_some() {
                                            arg.0.focus(&self[i]);
                                        }
                                    }
                                    CalOutcome::Day(new_date)
                                }
                            }
                            ct_event!(keycode press Right) => {
                                if !self[i].day_selection {
                                    return CalOutcome::Continue;
                                }
                                if i + 1 < self.len() {
                                    let next_day = self[i]
                                        .start_date
                                        .checked_add_months(Months::new(1))
                                        .expect("date");

                                    self[i].select_day(None);
                                    self[i + 1].select_date(Some(next_day));
                                    arg.0.focus(&self[i + 1]);

                                    CalOutcome::Day(next_day)
                                } else {
                                    let new_date = self[i]
                                        .start_date
                                        .checked_add_months(Months::new(1))
                                        .expect("date");
                                    scroll_down_month_list(self, 1);
                                    // date may be anywhere (even nowhere) after the shift.
                                    for i in 0..self.len() {
                                        self[i].select_date(Some(new_date));
                                    }
                                    for i in 0..self.len() {
                                        if self[i].selected_day_as_date().is_some() {
                                            arg.0.focus(&self[i]);
                                        }
                                    }
                                    CalOutcome::Day(new_date)
                                }
                            }
                            ct_event!(keycode press ALT-Up) => {
                                if !self[i].week_selection {
                                    return CalOutcome::Continue;
                                }
                                if i > 0 {
                                    if let Some(date) = self[i].selected_week_as_date() {
                                        let new_date =
                                            date - chrono::Duration::try_days(7).expect("days");

                                        self[i].select_week_by_date(Some(new_date));
                                        self[i - 1].select_week_by_date(Some(new_date));
                                        arg.0.focus(&self[i - 1]);
                                        CalOutcome::Week(new_date)
                                    } else {
                                        // ?? invalid date
                                        CalOutcome::Continue
                                    }
                                } else {
                                    if let Some(date) = self[i].selected_week_as_date() {
                                        let new_date =
                                            date - chrono::Duration::try_days(7).expect("days");
                                        scroll_up_month_list(self, arg.1);
                                        // date may be anywhere (even nowhere) after the shift.
                                        for i in 0..self.len() {
                                            self[i].select_week_by_date(Some(new_date));
                                        }
                                        for i in 0..self.len() {
                                            if self[i].selected_week().is_some() {
                                                arg.0.focus(&self[i]);
                                            }
                                        }
                                        CalOutcome::Week(new_date)
                                    } else {
                                        // ?? invalid date
                                        CalOutcome::Continue
                                    }
                                }
                            }
                            ct_event!(keycode press ALT-Down) => {
                                if !self[i].week_selection {
                                    return CalOutcome::Continue;
                                }
                                if i + 1 < self.len() {
                                    if let Some(date) = self[i].selected_week_as_date() {
                                        let new_date =
                                            date + chrono::Duration::try_days(7).expect("days");

                                        self[i].select_week_by_date(Some(new_date));
                                        self[i + 1].select_week_by_date(Some(new_date));
                                        arg.0.focus(&self[i + 1]);
                                        CalOutcome::Week(new_date)
                                    } else {
                                        // ?? invalid date
                                        CalOutcome::Continue
                                    }
                                } else {
                                    if let Some(date) = self[i].selected_week_as_date() {
                                        let new_date =
                                            date + chrono::Duration::try_days(7).expect("days");
                                        scroll_down_month_list(self, arg.1);
                                        // date may be anywhere (even nowhere) after the shift.
                                        for i in 0..self.len() {
                                            self[i].select_week_by_date(Some(new_date));
                                        }
                                        for i in 0..self.len() {
                                            if self[i].selected_week().is_some() {
                                                arg.0.focus(&self[i]);
                                            }
                                        }
                                        CalOutcome::Week(new_date)
                                    } else {
                                        // ?? invalid date
                                        CalOutcome::Continue
                                    }
                                }
                            }
                            _ => CalOutcome::Continue,
                        },
                        r => r,
                    };

                    break 'f r;
                }
            }

            CalOutcome::Continue
        };

        let s = 'f: {
            for i in 0..self.len() {
                if !self[i].is_focused() {
                    let r = match self[i].handle(event, MouseOnly) {
                        CalOutcome::Week(d) => {
                            if self[i].selected_week == Some(0) {
                                if i > 0 {
                                    self[i - 1].select_week_by_date(Some(d));
                                }
                            } else if self[i].selected_week == Some(self[i].week_len() - 1) {
                                if i < self.len() {
                                    self[i + 1].select_week_by_date(Some(d));
                                }
                            }

                            // transfer focus
                            for k in 0..self.len() {
                                if self[k].is_focused() {
                                    self[k].select_day(None);
                                    arg.0.focus(&self[i]);
                                } else if i != k {
                                    self[k].select_day(None);
                                }
                            }

                            CalOutcome::Week(d)
                        }
                        CalOutcome::Day(d) => {
                            // transfer focus
                            for k in 0..self.len() {
                                if self[k].is_focused() {
                                    self[k].select_day(None);
                                    arg.0.focus(&self[i]);
                                } else if i != k {
                                    self[k].select_day(None);
                                }
                            }

                            CalOutcome::Day(d)
                        }
                        r => r,
                    };
                    if r.is_consumed() {
                        break 'f r;
                    }
                }
            }
            CalOutcome::Continue
        };

        let all_areas = self.iter().map(|v| v.area).reduce(|v, w| v.union(w));
        let t = if let Some(all_areas) = all_areas {
            match event {
                ct_event!(scroll down for x,y) if all_areas.contains((*x, *y).into()) => {
                    scroll_down_month_list(self, arg.1);
                    CalOutcome::Changed
                }
                ct_event!(scroll up for x,y) if all_areas.contains((*x, *y).into()) => {
                    scroll_up_month_list(self, arg.1);
                    CalOutcome::Changed
                }
                _ => CalOutcome::Continue,
            }
        } else {
            CalOutcome::Continue
        };

        max(r, max(s, t))
    }
}
