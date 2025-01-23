//!
//! Render a month of a calendar.
//! Can be localized with a chrono::Locale.
//!

use crate::_private::NonExhaustive;
use crate::calendar::event::CalOutcome;
use crate::util::{block_size, revert_style};
use chrono::{Datelike, Days, Local, Months, NaiveDate, Weekday};
use log::debug;
use rat_event::util::MouseFlagsN;
use rat_event::{ct_event, flow, ConsumedEvent, HandleEvent, MouseOnly, Regular};
use rat_focus::{Focus, FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_reloc::{relocate_area, relocate_areas, RelocatableState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::block::Title;
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::StatefulWidgetRef;
use ratatui::widgets::{Block, StatefulWidget, Widget};
use std::array;
use std::cell::RefCell;
use std::cmp::max;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

/// Renders a month.
#[derive(Debug, Clone)]
pub struct Month<'a, Selection> {
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
    /// Show Weekdays above
    show_weekdays: bool,

    /// Block
    block: Option<Block<'a>>,

    /// Locale
    loc: chrono::Locale,

    phantom: PhantomData<Selection>,
}

/// Composite style for the calendar.
#[derive(Debug, Clone)]
pub struct CalendarStyle {
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
pub struct MonthState<Selection = SingleSelection> {
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

    /// Selected week
    selection: Selection,

    /// Focus
    /// __read+write__
    pub focus: FocusFlag,
    /// Mouse flags
    /// __read+write__
    pub mouse: MouseFlagsN,

    pub non_exhaustive: NonExhaustive,
}

impl Default for CalendarStyle {
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

/// Selection model for a calendar.
pub trait CalendarSelection {
    /// Clear all selections.
    fn clear(&mut self);

    /// Retires the current anchor+lead keeping the range selected.
    /// Starts a new anchor+lead.
    fn retire(&mut self);

    /// Is the given day selected.
    fn is_selected_day(&self, date: NaiveDate) -> bool;

    /// Select a single day, maybe extending a range.
    /// Removes any anchor/lead for months and weeks.
    fn select_day(&mut self, date: NaiveDate, extend: bool) -> bool;

    /// Selection lead, or the sole selected day.
    fn lead_day(&self) -> Option<NaiveDate>;

    /// Is the given week selected.
    fn is_selected_week(&self, date: NaiveDate) -> bool;

    /// Select a single week, maybe extending a range.
    /// Removes any anchor/lead for days and months.
    fn select_week(&mut self, date: NaiveDate, extend: bool) -> bool;

    /// Selection lead, or the sole selected week.
    fn lead_week(&self) -> Option<NaiveDate>;

    /// Is the given month selected.
    fn is_selected_month(&self, date: NaiveDate) -> bool;

    /// Select a single month, maybe extending a range.
    /// Removes any anchor/lead for days and weeks.
    fn select_month(&mut self, date: NaiveDate, extend: bool) -> bool;

    /// Selection lead, or the sole selected month.
    fn lead_month(&self) -> Option<NaiveDate>;

    /// Returns the overall lead, wether day, week or month.
    fn lead(&self) -> Option<NaiveDate>;
}

/// Basic single selection.
#[derive(Debug, Default, Clone)]
pub struct SingleSelection {
    day: Option<NaiveDate>,
    week: Option<NaiveDate>,
    month: Option<NaiveDate>,
}

impl CalendarSelection for SingleSelection {
    fn clear(&mut self) {
        self.day = None;
        self.week = None;
        self.month = None;
    }

    fn retire(&mut self) {
        // noop
    }

    fn is_selected_day(&self, date: NaiveDate) -> bool {
        if let Some(day) = self.day {
            date == day
        } else if let Some(week) = self.week {
            date.week(Weekday::Mon).first_day() == week
        } else if let Some(month) = self.month {
            date.with_day(1).expect("date") == month
        } else {
            false
        }
    }

    fn select_day(&mut self, date: NaiveDate, _extend: bool) -> bool {
        let old = (self.day, self.week, self.month);
        self.day = Some(date);
        self.week = None;
        self.month = None;
        old != (self.day, self.week, self.month)
    }

    fn lead_day(&self) -> Option<NaiveDate> {
        self.day
    }

    fn is_selected_week(&self, date: NaiveDate) -> bool {
        if let Some(week) = self.week {
            date.week(Weekday::Mon).first_day() == week
        } else if let Some(month) = self.month {
            date.with_day(1).expect("date") == month
        } else {
            false
        }
    }

    fn select_week(&mut self, date: NaiveDate, _: bool) -> bool {
        let old = (self.day, self.week, self.month);
        self.day = None;
        self.week = Some(date.week(Weekday::Mon).first_day());
        self.month = None;
        old != (self.day, self.week, self.month)
    }

    fn lead_week(&self) -> Option<NaiveDate> {
        self.week
    }

    fn is_selected_month(&self, date: NaiveDate) -> bool {
        if let Some(month) = self.month {
            date.with_day(1).expect("date") == month
        } else {
            false
        }
    }

    fn select_month(&mut self, date: NaiveDate, _: bool) -> bool {
        let old = (self.day, self.week, self.month);
        self.day = None;
        self.week = None;
        self.month = Some(date.with_day(1).expect("date"));
        old != (self.day, self.week, self.month)
    }

    fn lead_month(&self) -> Option<NaiveDate> {
        self.month
    }

    fn lead(&self) -> Option<NaiveDate> {
        self.day.or(self.week).or(self.month)
    }
}

impl<T: CalendarSelection> CalendarSelection for Rc<RefCell<T>> {
    fn clear(&mut self) {
        self.borrow_mut().clear();
    }

    fn retire(&mut self) {
        self.borrow_mut().retire();
    }

    fn is_selected_day(&self, date: NaiveDate) -> bool {
        self.borrow().is_selected_day(date)
    }

    fn select_day(&mut self, date: NaiveDate, extend: bool) -> bool {
        self.borrow_mut().select_day(date, extend)
    }

    fn lead_day(&self) -> Option<NaiveDate> {
        self.borrow().lead_day()
    }

    fn is_selected_week(&self, date: NaiveDate) -> bool {
        self.borrow().is_selected_week(date)
    }

    fn select_week(&mut self, date: NaiveDate, extend: bool) -> bool {
        self.borrow_mut().select_week(date, extend)
    }

    fn lead_week(&self) -> Option<NaiveDate> {
        self.borrow().lead_week()
    }

    fn is_selected_month(&self, date: NaiveDate) -> bool {
        self.borrow().is_selected_month(date)
    }

    fn select_month(&mut self, date: NaiveDate, extend: bool) -> bool {
        self.borrow_mut().select_month(date, extend)
    }

    fn lead_month(&self) -> Option<NaiveDate> {
        self.borrow().lead_month()
    }

    fn lead(&self) -> Option<NaiveDate> {
        self.borrow().lead()
    }
}

impl<'a, Selection> Default for Month<'a, Selection> {
    fn default() -> Self {
        Self {
            start_date: None,
            style: Default::default(),
            title_style: Default::default(),
            title_align: Default::default(),
            week_style: Default::default(),
            weekday_style: Default::default(),
            day_style: Default::default(),
            day_styles: Default::default(),
            select_style: Default::default(),
            focus_style: Default::default(),
            show_weekdays: false,
            block: Default::default(),
            loc: Default::default(),
            phantom: PhantomData,
        }
    }
}

impl<'a, Selection> Month<'a, Selection>
where
    Selection: CalendarSelection,
{
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

    /// Show weekday titles
    #[inline]
    pub fn show_weekdays(mut self) -> Self {
        self.show_weekdays = true;
        self
    }

    /// Set the composite style.
    #[inline]
    pub fn styles(mut self, s: CalendarStyle) -> Self {
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
    pub fn height(&self, state: &MonthState<Selection>) -> u16 {
        let start_date = if let Some(start_date) = self.start_date {
            start_date
        } else {
            state.start_date
        };

        let r = MonthState::<Selection>::count_weeks(start_date) as u16;
        let w = if self.show_weekdays { 1 } else { 0 };
        let b = max(1, block_size(&self.block).height);
        r + w + b
    }
}

#[cfg(feature = "unstable-widget-ref")]
impl<'a, Selection> StatefulWidgetRef for Month<'a, Selection>
where
    Selection: CalendarSelection,
{
    type State = MonthState<Selection>;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

impl<Selection> StatefulWidget for Month<'_, Selection>
where
    Selection: CalendarSelection,
{
    type State = MonthState<Selection>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

fn render_ref<Selection: CalendarSelection>(
    widget: &Month<'_, Selection>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut MonthState<Selection>,
) {
    state.area = area;
    if let Some(start_date) = widget.start_date {
        state.start_date = start_date;
    }

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

    let month = day.month();
    let mut w = 0;
    let mut x = state.inner.x;
    let mut y = state.inner.y;

    // week days
    if widget.show_weekdays {
        let mut week_0 = day.week(Weekday::Mon).first_day();

        x += 3;
        buf.set_style(Rect::new(x, y, 3 * 7, 1), weekday_style);
        for _ in 0..7 {
            let area = Rect::new(x, y, 2, 1).intersection(state.inner);

            let day_name = week_0.format_localized("%a", widget.loc).to_string();
            Span::from(format!("{:2} ", day_name)).render(area, buf);

            x += 3;
            week_0 = week_0 + Days::new(1);
        }
        x = state.inner.x;
        y += 1;
    }

    // first line may omit a few days
    state.area_weeks[w] = Rect::new(x, y, 2, 1).intersection(state.inner);
    Span::from(day.format_localized("%V", widget.loc).to_string())
        .style(week_style)
        .render(state.area_weeks[w], buf);

    let week_sel = if state.selection.is_selected_week(day) {
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
            let day_style = if week_sel || state.selection.is_selected_day(day) {
                day_style.patch(select_style)
            } else {
                day_style
            };

            state.area_days[day.day0() as usize] = Rect::new(x, y, 2, 1).intersection(state.inner);

            Span::from(day.format_localized("%e", widget.loc).to_string())
                .style(day_style)
                .render(state.area_days[day.day0() as usize], buf);

            x += 3;
            day = day + Days::new(1);
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

        let week_sel = if state.selection.is_selected_week(day) {
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
                let day_style = if week_sel || state.selection.is_selected_day(day) {
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
                day = day + Days::new(1);
            } else {
                x += 3;
            }
        }

        w += 1;
        x = state.inner.x;
        y += 1;
    }
}

impl<Selection> HasFocus for MonthState<Selection> {
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

impl<Selection> RelocatableState for MonthState<Selection> {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.area = relocate_area(self.area, shift, clip);
        self.inner = relocate_area(self.inner, shift, clip);
        relocate_areas(&mut self.area_days, shift, clip);
        relocate_areas(&mut self.area_weeks, shift, clip);
    }
}

impl<Selection> Clone for MonthState<Selection>
where
    Selection: Clone,
{
    fn clone(&self) -> Self {
        Self {
            area: self.area,
            inner: self.inner,
            area_days: self.area_days.clone(),
            area_weeks: self.area_weeks.clone(),
            start_date: self.start_date,
            selection: self.selection.clone(),
            focus: FocusFlag::named(self.focus.name()),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<Selection> Default for MonthState<Selection>
where
    Selection: Default,
{
    fn default() -> Self {
        Self {
            area: Default::default(),
            inner: Default::default(),
            area_days: [Rect::default(); 31],
            area_weeks: [Rect::default(); 6],
            start_date: Default::default(),
            selection: Default::default(),
            focus: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<Selection> MonthState<Selection>
where
    Selection: CalendarSelection,
{
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

    /// End date of this month.
    pub fn end_date(&self) -> NaiveDate {
        self.start_date + Months::new(1) - Days::new(1)
    }

    /// Removes all selection.
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    /// Select a week.
    pub fn select_week(&mut self, n: usize) -> bool {
        if n < self.week_len() {
            let date = self.start_date() + Days::new(7 * n as u64);
            self.selection.select_week(date, false)
        } else {
            false
        }
    }

    /// Selected week
    pub fn selected_week(&self) -> Option<usize> {
        if let Some(week) = self.selection.lead_week() {
            let mut test = self.start_date.week(Weekday::Mon).first_day();
            for n in 0..6 {
                if week == test {
                    return Some(n);
                }
                test = test + Days::new(7);
            }
            None
        } else {
            None
        }
    }

    /// Select a week by date
    /// Returns true if the date is valid for this month.
    /// If false it doesn't change the selection.
    pub fn select_week_by_date(&mut self, date: NaiveDate) -> bool {
        let start = self.start_date;

        let new_date = date.week(Weekday::Mon).first_day();
        let new_date_end = date.week(Weekday::Mon).last_day();

        if (new_date.year() == start.year() && new_date.month() == start.month())
            || (new_date_end.year() == start.year() && new_date_end.month() == start.month())
        {
            self.selection.select_week(new_date, false)
        } else {
            false
        }
    }

    /// Selected week
    pub fn selected_week_as_date(&self) -> Option<NaiveDate> {
        self.selection.lead_week()
    }

    /// Select a day
    pub fn select_day(&mut self, n: usize) -> bool {
        if let Some(date) = self.start_date.with_day0(n as u32) {
            return self.selection.select_day(date, false);
        }
        false
    }

    /// Selected day
    pub fn selected_day(&self) -> Option<usize> {
        if let Some(day) = self.selection.lead_day() {
            Some(day.day0() as usize)
        } else {
            None
        }
    }

    /// Select by date.
    /// Returns true if the date is valid for this month.
    /// If false it doesn't change the selection.
    pub fn select_date(&mut self, d: NaiveDate) -> bool {
        let start = self.start_date;
        if d.year() == start.year() && d.month() == start.month() {
            self.selection.select_day(d, false);
            return true;
        }
        false
    }

    /// Selected day
    pub fn selected_day_as_date(&self) -> Option<NaiveDate> {
        self.selection.lead_day()
    }

    /// Select previous day.
    pub fn prev_day(&mut self, n: usize) -> bool {
        let start = self.start_date();
        let end = self.end_date();
        let date = self.selection.lead();

        let new_date = if let Some(date) = date {
            if date >= start && date <= end {
                date - Days::new(n as u64)
            } else if date < start {
                self.start_date()
            } else {
                self.end_date()
            }
        } else {
            self.end_date()
        };

        if new_date >= start && new_date <= end {
            self.selection.select_day(new_date, false)
        } else {
            false
        }
    }

    /// Select next day.
    /// Doesn't let the selection get out of the month.
    /// If the current selection is outside the month it
    /// will set a new date within.
    pub fn next_day(&mut self, n: usize) -> bool {
        let start = self.start_date();
        let end = self.end_date();
        let date = self.selection.lead();

        let new_date = if let Some(date) = date {
            if date >= start && date <= end {
                date + Days::new(n as u64)
            } else if date < start {
                self.start_date()
            } else {
                self.end_date()
            }
        } else {
            self.start_date()
        };

        if new_date >= start && new_date <= end {
            self.selection.select_day(new_date, false)
        } else {
            false
        }
    }

    /// Select previous week.
    pub fn prev_week(&mut self, n: usize) -> bool {
        let start = self.start_date();
        let end = self.end_date();
        let date = self.selection.lead();

        if let Some(date) = date {
            let new_date = if date >= start && date <= end {
                date - Days::new(7 * n as u64)
            } else if date < start {
                self.start_date()
            } else {
                self.end_date()
            };
            let new_date_end = new_date.week(Weekday::Mon).last_day();
            if new_date_end >= start && new_date_end <= end {
                self.selection.select_week(new_date, false)
            } else {
                false
            }
        } else {
            let new_date = self.end_date();
            self.selection.select_week(new_date, false)
        }
    }

    /// Select next week.
    pub fn next_week(&mut self, n: usize) -> bool {
        let start = self.start_date();
        let end = self.end_date();
        let date = self.selection.lead();

        let new_date = if let Some(date) = date {
            let date_end = date.week(Weekday::Mon).last_day();
            if date_end >= start && date_end <= end {
                date + Days::new(7 * n as u64)
            } else if date_end < start {
                self.start_date()
            } else {
                self.end_date()
            }
        } else {
            self.start_date()
        };

        if new_date >= start && new_date <= end {
            self.selection.select_week(new_date, false)
        } else {
            false
        }
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
                day = day + Days::new(1);
            }
        }
        // count mondays
        while month == day.month() {
            weeks += 1;
            day = day + Days::new(7);
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
        /// The selection has changed. This contains the new
        /// lead date.
        Selected(NaiveDate),
        /// Month selected. This is the first of the selected month.
        Month(NaiveDate),
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
                CalOutcome::Selected(_) => Outcome::Changed,
                CalOutcome::Month(_) => Outcome::Changed,
            }
        }
    }
}

impl<Selection> HandleEvent<crossterm::event::Event, Regular, CalOutcome> for MonthState<Selection>
where
    Selection: CalendarSelection,
{
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> CalOutcome {
        if self.is_focused() {
            flow!(match event {
                ct_event!(keycode press Up) => {
                    if self.prev_day(7) {
                        CalOutcome::Day(self.selected_day_as_date().expect("day"))
                    } else {
                        CalOutcome::Continue
                    }
                }
                ct_event!(keycode press Down) => {
                    if self.next_day(7) {
                        CalOutcome::Day(self.selected_day_as_date().expect("day"))
                    } else {
                        CalOutcome::Continue
                    }
                }
                ct_event!(keycode press Left) => {
                    if self.prev_day(1) {
                        CalOutcome::Day(self.selected_day_as_date().expect("day"))
                    } else {
                        CalOutcome::Continue
                    }
                }
                ct_event!(keycode press Right) => {
                    if self.next_day(1) {
                        CalOutcome::Day(self.selected_day_as_date().expect("day"))
                    } else {
                        CalOutcome::Continue
                    }
                }
                ct_event!(keycode press ALT-Up) => {
                    if self.prev_week(1) {
                        CalOutcome::Week(self.selected_week_as_date().expect("week"))
                    } else {
                        CalOutcome::Continue
                    }
                }
                ct_event!(keycode press ALT-Down) => {
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

impl<Selection> HandleEvent<crossterm::event::Event, MouseOnly, CalOutcome>
    for MonthState<Selection>
where
    Selection: CalendarSelection,
{
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> CalOutcome {
        match event {
            ct_event!(mouse drag Left for x, y) | ct_event!(mouse down Left for x, y) => {
                if let Some(sel) = self.mouse.item_at(&self.area_weeks, *x, *y) {
                    if self.select_week(sel) {
                        CalOutcome::Week(self.selected_week_as_date().expect("week"))
                    } else {
                        CalOutcome::Unchanged
                    }
                } else if let Some(sel) = self.mouse.item_at(&self.area_days, *x, *y) {
                    if self.select_day(sel) {
                        CalOutcome::Day(self.selected_day_as_date().expect("day"))
                    } else {
                        CalOutcome::Unchanged
                    }
                } else {
                    CalOutcome::Continue
                }
            }

            _ => CalOutcome::Continue,
        }
    }
}

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
pub struct CalendarState<const N: usize, Selection = SingleSelection> {
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
    pub months: [MonthState<Rc<RefCell<Selection>>>; N],

    pub selection: Rc<RefCell<Selection>>,

    /// Calendar focus
    pub focus: FocusFlag,

    inner_focus: Option<Focus>,
}

impl<const N: usize, Selection> Default for CalendarState<N, Selection>
where
    Selection: CalendarSelection + Default,
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
            inner_focus: Default::default(),
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

impl<const N: usize, Selection> CalendarState<N, Selection>
where
    Selection: CalendarSelection + Default,
{
    pub fn new() -> Self {
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

    /// Sets the starting primary month for the calendar.
    pub fn set_home_policy(&mut self, home: HomePolicy) {
        if let HomePolicy::Index(idx) = home {
            assert!(idx < self.months.len());
        }
        self.home = home;
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
    pub fn scroll_forward(&mut self, n: usize) -> bool {
        if n == 0 {
            return false;
        }

        // change start dates
        let mut start = self.months[0].start_date() + Months::new(n as u32);

        for i in 0..self.months.len() {
            self.months[i].set_start_date(start);
            start = start + Months::new(1);
        }

        true
    }

    /// Changes the start-date for each month.
    /// Doesn't change any selection.
    pub fn scroll_back(&mut self, n: usize) -> bool {
        if n == 0 {
            return false;
        }

        // change start dates
        let mut start = self.months[0].start_date() - Months::new(n as u32);

        for i in 0..self.months.len() {
            self.months[i].set_start_date(start);
            start = start + Months::new(1);
        }

        true
    }

    fn focus_lead(&mut self) {
        let Some(lead) = self.selection.lead() else {
            return;
        };
        if self.is_focused() {
            for (i, month) in self.months.iter().enumerate() {
                if lead >= month.start_date() && lead <= month.end_date() {
                    self.primary_focus = i;
                    month.focus.set(true);
                } else {
                    month.focus.set(false);
                }
            }
        }
    }

    /// Move all selections back by step.
    pub fn shift_back(&mut self, n: usize)
    where
        Selection: Debug,
    {
        if self.step == 0 {
            return;
        }

        if let Some(lead) = self.selection.lead_day() {
            self.selection
                .select_day(lead - Months::new(n as u32), false);
        }
        if let Some(lead) = self.selection.lead_week() {
            self.selection
                .select_week(lead - Months::new(n as u32), false);
        }
        if let Some(lead) = self.selection.lead_month() {
            self.selection
                .select_month(lead - Months::new(n as u32), false);
        }

        if let Some(lead) = self.selection.lead() {
            if lead < self.start_date() {
                self.scroll_back(self.step);
            }
        }
        self.focus_lead();
    }

    /// Move all selections forward by step
    pub fn shift_forward(&mut self, n: usize)
    where
        Selection: Debug,
    {
        if self.step == 0 {
            return;
        }

        if let Some(lead) = self.selection.lead_day() {
            self.selection
                .select_day(lead + Months::new(n as u32), false);
        }
        if let Some(lead) = self.selection.lead_week() {
            self.selection
                .select_week(lead + Months::new(n as u32), false);
        }
        if let Some(lead) = self.selection.lead_month() {
            self.selection
                .select_month(lead + Months::new(n as u32), false);
        }

        if let Some(lead) = self.selection.lead() {
            if lead > self.end_date() {
                self.scroll_forward(self.step);
            }
        }
        self.focus_lead();
    }

    /// Select previous week.
    pub fn prev_week(&mut self, n: usize) -> bool {
        let start = self.start_date();
        let end = self.end_date();
        let date = self.selection.lead();
        debug!("lead {:?}", date);

        let r = if let Some(date) = date {
            let new_date = if date >= start && date <= end || self.step != 0 {
                date - Days::new(7 * n as u64)
            } else if date < start {
                self.start_date()
            } else {
                self.end_date()
            };
            let new_date_end = new_date.week(Weekday::Mon).last_day();
            if new_date_end >= start && new_date_end <= end {
                self.selection.select_week(new_date, false)
            } else if self.step > 0 {
                self.scroll_back(self.step);
                self.selection.select_week(new_date, false)
            } else {
                false
            }
        } else {
            let new_date = self.end_date();
            self.selection.select_week(new_date, false)
        };

        if r {
            self.focus_lead();
        }

        r
    }

    /// Select previous week.
    pub fn next_week(&mut self, n: usize) -> bool {
        let start = self.start_date();
        let end = self.end_date();
        let date = self.selection.lead();
        debug!("lead {:?}", date);

        let new_date = if let Some(date) = date {
            let date_end = date.week(Weekday::Mon).last_day();
            if date_end >= start && date_end <= end || self.step > 0 {
                date + Days::new(7 * n as u64)
            } else if date_end < start {
                self.start_date()
            } else {
                self.end_date()
            }
        } else {
            debug!("next_week {}", self.start_date());
            self.start_date()
        };

        let r = if new_date >= start && new_date <= end {
            self.selection.select_week(new_date, false)
        } else if self.step > 0 {
            self.scroll_forward(self.step);
            self.selection.select_week(new_date, false)
        } else {
            false
        };

        if r {
            self.focus_lead();
        }

        r
    }

    pub fn move_to_current(&mut self) -> bool {
        let current = Local::now().date_naive();

        let r = self.selection.select_day(current, false);
        match self.home {
            HomePolicy::Index(primary) => {
                self.primary_focus = primary;
                self.set_start_date(current - Months::new(primary as u32));
                self.focus_lead();
            }
            HomePolicy::Year => {
                let month = current.month();
                self.primary_focus = month as usize;
                self.set_start_date(current - Months::new(month));
                self.focus_lead();
            }
        }

        r
    }

    /// Select previous day.
    pub fn prev_day(&mut self, n: usize) -> bool {
        let start = self.start_date();
        let end = self.end_date();
        let date = self.selection.lead();

        let new_date = if let Some(date) = date {
            if date >= start && date <= end {
                date - Days::new(n as u64)
            } else if date < start {
                self.start_date()
            } else {
                self.end_date()
            }
        } else {
            self.end_date()
        };

        let r = if new_date >= start && new_date <= end {
            self.selection.select_day(new_date, false)
        } else if self.step > 0 {
            self.scroll_back(self.step);
            self.selection.select_day(new_date, false)
        } else {
            false
        };

        if r {
            self.focus_lead();
        }

        r
    }

    /// Select previous week.
    pub fn next_day(&mut self, n: usize) -> bool {
        let start = self.start_date();
        let end = self.end_date();
        let date = self.selection.lead();

        let new_date = if let Some(date) = date {
            if date >= start && date <= end {
                date + Days::new(n as u64)
            } else if date < start {
                self.start_date()
            } else {
                self.end_date()
            }
        } else {
            self.start_date()
        };

        let r = if new_date >= start && new_date <= end {
            self.selection.select_day(new_date, false)
        } else if self.step > 0 {
            self.scroll_forward(self.step);
            self.selection.select_day(new_date, false)
        } else {
            false
        };

        if r {
            self.focus_lead();
        }

        r
    }

    fn rebuild_inner_focus(&mut self) {
        let mut builder = FocusBuilder::new(self.inner_focus.take());
        self.build(&mut builder);
        self.inner_focus = Some(builder.build());
    }
}

impl<const N: usize, Selection> HandleEvent<crossterm::event::Event, Regular, CalOutcome>
    for CalendarState<N, Selection>
where
    Selection: CalendarSelection + Default + Debug,
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
                self.rebuild_inner_focus();

                match event {
                    ct_event!(keycode press PageUp) => {
                        self.shift_back(self.step);
                        //CalOutcome::Selected(self.selection.lead().unwrap())
                        CalOutcome::Changed
                    }
                    ct_event!(keycode press PageDown) => {
                        self.shift_forward(self.step);
                        CalOutcome::Changed
                    }
                    ct_event!(keycode press Up) => {
                        self.prev_day(7);
                        CalOutcome::Changed
                    }
                    ct_event!(keycode press Down) => {
                        self.next_day(7);
                        CalOutcome::Changed
                    }
                    ct_event!(keycode press Left) => {
                        self.prev_day(1);
                        CalOutcome::Changed
                    }
                    ct_event!(keycode press Right) => {
                        self.next_day(1);
                        CalOutcome::Changed
                    }
                    ct_event!(keycode press Home) => {
                        self.move_to_current();
                        CalOutcome::Changed
                    }
                    ct_event!(keycode press ALT-Up) => {
                        self.prev_week(1);
                        CalOutcome::Changed
                    }
                    ct_event!(keycode press ALT-Down) => {
                        self.next_week(1);
                        CalOutcome::Changed
                    }
                    _ => CalOutcome::Continue,
                }
            } else {
                CalOutcome::Continue
            }
        });

        r.or_else(|| {
            let all_areas = self
                .months
                .iter()
                .map(|v| v.area)
                .reduce(|v, w| v.union(w))
                .unwrap_or_default();
            match event {
                ct_event!(scroll up for x,y) if all_areas.contains((*x, *y).into()) => {
                    self.scroll_back(self.step);
                    CalOutcome::Changed
                }
                ct_event!(scroll down for x,y) if all_areas.contains((*x, *y).into()) => {
                    self.scroll_forward(self.step);
                    CalOutcome::Changed
                }
                _ => CalOutcome::Continue,
            }
        });

        r
    }
}

#[allow(clippy::needless_range_loop)]
pub fn scroll_up_month_list<Selection: CalendarSelection>(
    months: &mut [MonthState<Selection>],
    delta: u32,
) {
    // change start dates
    let mut start = months[0].start_date() - Months::new(delta);
    for i in 0..months.len() {
        let d = months[i].selected_day();
        let w = months[i].selected_week();

        months[i].set_start_date(start);

        months[i].clear_selection();
        if let Some(d) = d {
            months[i].select_day(d);
        }
        if let Some(w) = w {
            months[i].select_week(w);
        }

        start = start + Months::new(1);
    }
}

#[allow(clippy::needless_range_loop)]
pub fn scroll_down_month_list<Selection: CalendarSelection>(
    months: &mut [MonthState<Selection>],
    delta: u32,
) {
    // change start dates
    let mut start = months[0].start_date() + Months::new(delta);
    for i in 0..months.len() {
        let d = months[i].selected_day();
        let w = months[i].selected_week();

        months[i].set_start_date(start);

        months[i].clear_selection();
        if let Some(d) = d {
            months[i].select_day(d);
        }
        if let Some(w) = w {
            months[i].select_week(w);
        }

        start = start + Months::new(1);
    }
}

/// Multi-month event-handling.
///
/// The parameter gives the shift of the calendar in months, if
/// the newly selected date is not visible with the current settings.
///
/// This also needs the Focus, as it changes the focused month if necessary.
pub struct MultiMonth<'a>(pub &'a Focus, pub u32);

impl<Selection> HandleEvent<crossterm::event::Event, MultiMonth<'_>, CalOutcome>
    for &mut [MonthState<Selection>]
where
    Selection: CalendarSelection,
{
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
                                self[i - 1].select_week_by_date(date);
                            }
                            if i + 1 < self.len() {
                                self[i + 1].select_week_by_date(date);
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
                            ct_event!(keycode press PageUp) => {
                                if i > 0 {
                                    if let Some(date) = self[i]
                                        .selected_day_as_date()
                                        .or_else(|| self[i].selected_week_as_date())
                                    {
                                        let new_date = date - Months::new(1);
                                        self[i].clear_selection();
                                        self[i - 1].select_date(new_date);
                                        arg.0.focus(&self[i - 1]);
                                        CalOutcome::Day(new_date)
                                    } else {
                                        // ?? what is this. invalid day??
                                        CalOutcome::Continue
                                    }
                                } else {
                                    if let Some(date) = self[i]
                                        .selected_day_as_date()
                                        .or_else(|| self[i].selected_week_as_date())
                                    {
                                        let new_date = date - Months::new(1);
                                        scroll_up_month_list(self, arg.1);
                                        // date may be anywhere (even nowhere) after the shift.
                                        for i in 0..self.len() {
                                            self[i].select_date(new_date);
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
                            ct_event!(keycode press PageDown) => {
                                if i + 1 < self.len() {
                                    if let Some(date) = self[i]
                                        .selected_day_as_date()
                                        .or_else(|| self[i].selected_week_as_date())
                                    {
                                        let new_date = date + Months::new(1);
                                        self[i].clear_selection();
                                        self[i + 1].select_date(new_date);
                                        arg.0.focus(&self[i + 1]);
                                        CalOutcome::Day(new_date)
                                    } else {
                                        CalOutcome::Continue
                                    }
                                } else {
                                    if let Some(date) = self[i]
                                        .selected_day_as_date()
                                        .or_else(|| self[i].selected_week_as_date())
                                    {
                                        let new_date = date + Months::new(1);
                                        scroll_down_month_list(self, arg.1);
                                        // date may be anywhere (even nowhere) after the shift.
                                        for i in 0..self.len() {
                                            self[i].select_date(new_date);
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
                            ct_event!(keycode press Up) => {
                                if i > 0 {
                                    if let Some(date) = self[i].selected_day_as_date() {
                                        let new_date = date - Days::new(7);
                                        self[i].clear_selection();
                                        self[i - 1].select_date(new_date);
                                        arg.0.focus(&self[i - 1]);
                                        CalOutcome::Day(new_date)
                                    } else {
                                        // ?? what is this. invalid day??
                                        CalOutcome::Continue
                                    }
                                } else {
                                    if let Some(date) = self[i].selected_day_as_date() {
                                        let new_date = date - Days::new(7);
                                        scroll_up_month_list(self, arg.1);
                                        // date may be anywhere (even nowhere) after the shift.
                                        for i in 0..self.len() {
                                            self[i].select_date(new_date);
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
                                if i + 1 < self.len() {
                                    if let Some(date) = self[i].selected_day_as_date() {
                                        let new_date = date + Days::new(7);
                                        self[i].clear_selection();
                                        self[i + 1].select_date(new_date);
                                        arg.0.focus(&self[i + 1]);
                                        CalOutcome::Day(new_date)
                                    } else {
                                        CalOutcome::Continue
                                    }
                                } else {
                                    if let Some(date) = self[i].selected_day_as_date() {
                                        let new_date = date + Days::new(7);
                                        scroll_down_month_list(self, arg.1);
                                        // date may be anywhere (even nowhere) after the shift.
                                        for i in 0..self.len() {
                                            self[i].select_date(new_date);
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
                                if i > 0 {
                                    let prev_day = self[i].start_date - Days::new(1);

                                    self[i].clear_selection();
                                    self[i - 1].select_date(prev_day);
                                    arg.0.focus(&self[i - 1]);

                                    CalOutcome::Day(prev_day)
                                } else {
                                    let new_date = self[i].start_date - Days::new(1);
                                    scroll_up_month_list(self, 1);
                                    // date may be anywhere (even nowhere) after the shift.
                                    for i in 0..self.len() {
                                        self[i].select_date(new_date);
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
                                if i + 1 < self.len() {
                                    let next_day = self[i].start_date + Months::new(1);

                                    self[i].clear_selection();
                                    self[i + 1].select_date(next_day);
                                    arg.0.focus(&self[i + 1]);

                                    CalOutcome::Day(next_day)
                                } else {
                                    let new_date = self[i].start_date + Months::new(1);
                                    scroll_down_month_list(self, 1);
                                    // date may be anywhere (even nowhere) after the shift.
                                    for i in 0..self.len() {
                                        self[i].select_date(new_date);
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
                                if i > 0 {
                                    if let Some(date) = self[i].selected_week_as_date() {
                                        let new_date = date - Days::new(7);

                                        self[i].clear_selection();
                                        self[i - 1].clear_selection();
                                        self[i].select_week_by_date(new_date);
                                        self[i - 1].select_week_by_date(new_date);
                                        arg.0.focus(&self[i - 1]);
                                        CalOutcome::Week(new_date)
                                    } else {
                                        // ?? invalid date
                                        unreachable!()
                                    }
                                } else {
                                    if let Some(date) = self[i].selected_week_as_date() {
                                        let new_date = date - Days::new(7);
                                        scroll_up_month_list(self, arg.1);
                                        // date may be anywhere (even nowhere) after the shift.
                                        for i in 0..self.len() {
                                            if self[i].select_week_by_date(new_date) {
                                                arg.0.focus(&self[i]);
                                                break;
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
                                if i + 1 < self.len() {
                                    if let Some(date) = self[i].selected_week_as_date() {
                                        let new_date = date + Days::new(7);

                                        self[i].clear_selection();
                                        self[i + 1].clear_selection();
                                        self[i].select_week_by_date(new_date);
                                        self[i + 1].select_week_by_date(new_date);
                                        arg.0.focus(&self[i + 1]);
                                        CalOutcome::Week(new_date)
                                    } else {
                                        // ?? invalid date
                                        CalOutcome::Continue
                                    }
                                } else {
                                    if let Some(date) = self[i].selected_week_as_date() {
                                        let new_date = date + Days::new(7);
                                        scroll_down_month_list(self, arg.1);
                                        // date may be anywhere (even nowhere) after the shift.
                                        for i in 0..self.len() {
                                            if self[i].select_week_by_date(new_date) {
                                                arg.0.focus(&self[i]);
                                                break;
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
                            // todo:
                            // if self[i].selected_week == Some(0) {
                            //     if i > 0 {
                            //         self[i - 1].select_week_by_date(Some(d));
                            //     }
                            // } else if self[i].selected_week == Some(self[i].week_len() - 1) {
                            //     if i < self.len() {
                            //         self[i + 1].select_week_by_date(Some(d));
                            //     }
                            // }

                            // transfer focus
                            for k in 0..self.len() {
                                if self[k].is_focused() {
                                    self[k].clear_selection();
                                    arg.0.focus(&self[i]);
                                } else if i != k {
                                    self[k].clear_selection();
                                }
                            }

                            CalOutcome::Week(d)
                        }
                        CalOutcome::Day(d) => {
                            // transfer focus
                            for k in 0..self.len() {
                                if self[k].is_focused() {
                                    self[k].clear_selection();
                                    arg.0.focus(&self[i]);
                                } else if i != k {
                                    self[k].clear_selection();
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

                    'f: {
                        for m in self.iter() {
                            if let Some(d) = m.selected_week_as_date() {
                                break 'f CalOutcome::Week(d);
                            } else if let Some(d) = m.selected_day_as_date() {
                                break 'f CalOutcome::Day(d);
                            }
                        }
                        CalOutcome::Changed
                    }
                }
                ct_event!(scroll up for x,y) if all_areas.contains((*x, *y).into()) => {
                    scroll_up_month_list(self, arg.1);

                    'f: {
                        for m in self.iter() {
                            if let Some(d) = m.selected_week_as_date() {
                                break 'f CalOutcome::Week(d);
                            } else if let Some(d) = m.selected_day_as_date() {
                                break 'f CalOutcome::Day(d);
                            }
                        }
                        CalOutcome::Changed
                    }
                }
                _ => CalOutcome::Continue,
            }
        } else {
            CalOutcome::Continue
        };

        max(r, max(s, t))
    }
}
