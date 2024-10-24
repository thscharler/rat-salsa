//!
//! Render a month of a calendar.
//! Can be localized with a chrono::Locale.
//!

use crate::_private::NonExhaustive;
use crate::calendar::event::CalOutcome;
use crate::util::{block_size, revert_style};
use chrono::{Datelike, NaiveDate, Weekday};
use rat_event::util::MouseFlagsN;
use rat_event::{ct_event, flow, HandleEvent, MouseOnly, Regular};
use rat_focus::{FocusFlag, HasFocus};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::block::Title;
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::StatefulWidgetRef;
use ratatui::widgets::{Block, StatefulWidget, Widget};
use std::collections::HashMap;
use std::fmt::Debug;

/// Renders a month.
#[derive(Debug, Default, Clone)]
pub struct Month<'a> {
    /// Start date of the month.
    start_date: NaiveDate,

    /// Base style.
    style: Style,
    /// Title style.
    title_style: Option<Style>,
    /// Title align.
    title_align: Alignment,
    /// Week number style.
    week_style: Option<Style>,
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
    pub day: Option<Style>,
    pub select: Option<Style>,
    pub focus: Option<Style>,
    pub block: Option<Block<'static>>,
    pub non_exhaustive: NonExhaustive,
}

/// State & event-handling.
#[derive(Debug, Clone)]
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
    /// __readonly__. renewed for each render.
    pub start_date: NaiveDate,

    /// Day selection enabled
    /// __readonly__. renewed for each render.
    day_selection: bool,
    /// Week selection enabled
    /// __readonly__. renewed for each render.
    week_selection: bool,

    /// Selected week
    pub selected_week: Option<usize>,
    /// Selected day
    pub selected_day: Option<usize>,

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
            title: Default::default(),
            week: Default::default(),
            day: Default::default(),
            select: Default::default(),
            focus: Default::default(),
            block: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a> Month<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the starting date.
    /// This can be any date of the month.
    #[inline]
    pub fn date(mut self, s: NaiveDate) -> Self {
        self.start_date = s.with_day(1).expect("day");
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
    pub fn height(&self) -> u16 {
        let r = MonthState::count_weeks(self.start_date) as u16;
        r + block_size(&self.block).height
    }
}

#[cfg(feature = "unstable-widget-ref")]
impl<'a> StatefulWidgetRef for Month<'a> {
    type State = MonthState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

impl<'a> StatefulWidget for Month<'a> {
    type State = MonthState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

fn render_ref(widget: &Month<'_>, area: Rect, buf: &mut Buffer, state: &mut MonthState) {
    state.area = area;
    state.start_date = widget.start_date;
    state.day_selection = widget.day_selection;
    state.week_selection = widget.week_selection;

    let mut day = widget.start_date;

    let title_style = if let Some(title_style) = widget.title_style {
        title_style
    } else {
        widget.style
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
            .title(Title::from(
                day.format_localized("%B", widget.loc).to_string(),
            ))
            .title_style(title_style)
            .title_alignment(widget.title_align)
    };

    buf.set_style(area, widget.style);
    state.inner = block.inner(area);
    block.render(area, buf);

    let focus_style = if let Some(focus_style) = widget.focus_style {
        focus_style
    } else {
        revert_style(widget.style)
    };
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

    let day_style = if let Some(day_style) = widget.day_style {
        day_style
    } else {
        widget.style
    };
    let week_style = if let Some(week_style) = widget.week_style {
        week_style
    } else {
        widget.style
    };

    let month = widget.start_date.month();
    let mut w = 0;
    let mut x = state.inner.x;
    let mut y = state.inner.y;

    // first line may omit a few days
    state.area_weeks[w] = Rect::new(x, y, 2, 1).intersection(state.inner);
    Span::from(day.format_localized("%W", widget.loc).to_string())
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
        Span::from(day.format_localized("%W", widget.loc).to_string())
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
    #[inline]
    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    #[inline]
    fn area(&self) -> Rect {
        self.area
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

        let mut w = 0;

        while month == day.month() {
            day += chrono::Duration::try_days(7).expect("days");
            w += 1;
        }
        // last week might be next month
        let week = day.week(Weekday::Mon);
        if week.first_day().month() != month {
            w -= 1;
        }

        w
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
        /// Month in a list of months selected.
        Month(usize),
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
                CalOutcome::Month(_) => Outcome::Changed,
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

impl HandleEvent<crossterm::event::Event, Regular, CalOutcome> for &mut [MonthState] {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> CalOutcome {
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
                                    CalOutcome::Month(i - 1)
                                } else {
                                    CalOutcome::Continue
                                }
                            } else {
                                CalOutcome::Continue
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
                                    CalOutcome::Month(i + 1)
                                } else {
                                    CalOutcome::Continue
                                }
                            } else {
                                CalOutcome::Continue
                            }
                        }
                        ct_event!(keycode press Left) => {
                            if !self[i].day_selection {
                                return CalOutcome::Continue;
                            }
                            if i > 0 {
                                self[i].select_day(None);
                                self[i - 1].select_day(None);
                                if self[i - 1].prev_day(1) {
                                    CalOutcome::Month(i - 1)
                                } else {
                                    CalOutcome::Continue
                                }
                            } else {
                                CalOutcome::Continue
                            }
                        }
                        ct_event!(keycode press Right) => {
                            if !self[i].day_selection {
                                return CalOutcome::Continue;
                            }
                            if i + 1 < self.len() {
                                self[i].select_day(None);
                                self[i + 1].select_day(None);
                                if self[i + 1].next_day(1) {
                                    CalOutcome::Month(i + 1)
                                } else {
                                    CalOutcome::Continue
                                }
                            } else {
                                CalOutcome::Continue
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
                                    CalOutcome::Month(i - 1)
                                } else {
                                    CalOutcome::Continue
                                }
                            } else {
                                CalOutcome::Continue
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
                                    CalOutcome::Month(i + 1)
                                } else {
                                    CalOutcome::Continue
                                }
                            } else {
                                CalOutcome::Continue
                            }
                        }
                        _ => CalOutcome::Continue,
                    },
                    r => r,
                };

                return r;
            }
        }

        for i in 0..self.len() {
            let month = &mut self[i];
            if !month.is_focused() {
                flow!(match month.handle(event, MouseOnly) {
                    CalOutcome::Week(d) => {
                        if month.selected_week == Some(0) {
                            if i > 0 {
                                self[i - 1].select_week_by_date(Some(d));
                            }
                        } else if month.selected_week == Some(month.week_len() - 1) {
                            if i < self.len() {
                                self[i + 1].select_week_by_date(Some(d));
                            }
                        }
                        CalOutcome::Week(d)
                    }
                    r => {
                        r
                    }
                });
            }
        }

        CalOutcome::Continue
    }
}
