use crate::_private::NonExhaustive;
use crate::calendar::event::CalOutcome;
use crate::calendar::selection::{NoSelection, RangeSelection, SingleSelection};
use crate::calendar::style::CalendarStyle;
use crate::calendar::CalendarSelection;
use crate::util::{block_size, revert_style};
use chrono::{Datelike, Days, Months, NaiveDate, Weekday};
use rat_event::util::MouseFlagsN;
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_reloc::{relocate_area, relocate_areas, RelocatableState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::Style;
use ratatui::text::Span;
use ratatui::widgets::block::Title;
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::StatefulWidgetRef;
use ratatui::widgets::{Block, StatefulWidget, Widget};
use std::cell::RefCell;
use std::cmp::max;
use std::collections::HashMap;
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

    /// Show month name
    show_month: bool,
    /// Show Weekdays above
    show_weekdays: bool,

    /// Block
    block: Option<Block<'a>>,

    /// Locale
    loc: chrono::Locale,

    phantom: PhantomData<Selection>,
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
    /// Area of the calendar.
    pub area_cal: Rect,
    /// Area for the days of the month.
    /// __readonly__. renewed for each render.
    pub area_days: [Rect; 31],
    /// Area for all the week numbers.
    pub area_weeknum: Rect,
    /// Area for the week numbers.
    /// __readonly__. renewed for each render.
    pub area_weeks: [Rect; 6],

    /// Startdate
    start_date: NaiveDate,

    /// Selection
    pub selection: Rc<RefCell<Selection>>,

    /// Set to the container-focus if part of a container.
    /// __read+write__
    pub container: Option<FocusFlag>,

    /// Focus
    /// __read+write__
    pub focus: FocusFlag,
    /// Mouse flags
    /// __read+write__
    pub mouse: MouseFlagsN,

    pub non_exhaustive: NonExhaustive,
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
            show_month: true,
            show_weekdays: true,
            block: Default::default(),
            loc: Default::default(),
            phantom: PhantomData,
        }
    }
}

impl<'a, Selection> Month<'a, Selection> {
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

    /// Show month title
    #[inline]
    pub fn show_show_month(mut self) -> Self {
        self.show_month = true;
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
        if state.is_container_focused() || state.is_focused() {
            focus_style
        } else {
            select_style
        }
    } else {
        if state.is_container_focused() || state.is_focused() {
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

    let block = if widget.show_month {
        if let Some(block) = widget.block.clone() {
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
        }
    } else {
        if let Some(block) = widget.block.clone() {
            block
        } else {
            Block::new().style(widget.style)
        }
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

    // reset areas
    for i in 0..31 {
        state.area_days[i] = Rect::default();
    }
    for i in 0..6 {
        state.area_weeks[i] = Rect::default();
    }
    state.area_cal = Rect::new(x + 3, y, 7 * 3, state.week_len() as u16);
    state.area_weeknum = Rect::new(x, y, 3, state.week_len() as u16);

    // first line may omit a few days
    state.area_weeks[w] = Rect::new(x, y, 2, 1).intersection(state.inner);
    Span::from(day.format_localized("%V", widget.loc).to_string())
        .style(week_style)
        .render(state.area_weeks[w], buf);

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
            let day_style = calc_day_style(widget, state, day, day_style, select_style);
            state.area_days[day.day0() as usize] = Rect::new(x, y, 2, 1).intersection(state.inner);

            Span::from(day.format_localized("%e", widget.loc).to_string())
                .style(day_style)
                .render(state.area_days[day.day0() as usize], buf);

            if wd != Weekday::Sun && state.selection.is_selected(day + Days::new(1)) {
                let mut gap_area = state.area_days[day.day0() as usize];
                gap_area.x += 2;
                gap_area.width = 1;
                Span::from(" ").style(day_style).render(gap_area, buf);
            }

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
            if day.month() == month {
                let day_style = calc_day_style(widget, state, day, day_style, select_style);

                state.area_days[day.day0() as usize] =
                    Rect::new(x, y, 2, 1).intersection(state.inner);

                Span::from(day.format_localized("%e", widget.loc).to_string())
                    .style(day_style)
                    .render(state.area_days[day.day0() as usize], buf);

                if wd != Weekday::Sun && state.selection.is_selected(day + Days::new(1)) {
                    let mut gap_area = state.area_days[day.day0() as usize];
                    gap_area.x += 2;
                    gap_area.width = 1;
                    Span::from(" ").style(day_style).render(gap_area, buf);
                }

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

fn calc_day_style<Selection: CalendarSelection>(
    widget: &Month<'_, Selection>,
    state: &mut MonthState<Selection>,
    day: NaiveDate,
    day_style: Style,
    select_style: Style,
) -> Style {
    let day_style = if let Some(day_styles) = widget.day_styles {
        if let Some(day_style) = day_styles.get(&day) {
            *day_style
        } else {
            day_style
        }
    } else {
        day_style
    };

    if (state.is_container_focused() || state.is_focused())
        && state.selection.len() > 1
        && state.selection.lead_selection() == Some(day)
    {
        day_style.patch(revert_style(select_style))
    } else if state.selection.is_selected(day) {
        day_style.patch(select_style)
    } else {
        day_style
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
            area_cal: self.area_cal.clone(),
            area_days: self.area_days.clone(),
            area_weeknum: self.area_weeknum.clone(),
            area_weeks: self.area_weeks.clone(),
            start_date: self.start_date,
            selection: self.selection.clone(),
            container: self.container.clone(),
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
            area_cal: Default::default(),
            area_days: Default::default(),
            area_weeknum: Default::default(),
            area_weeks: Default::default(),
            start_date: Default::default(),
            selection: Default::default(),
            container: Default::default(),
            focus: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<Selection> MonthState<Selection> {
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

    fn in_range(&self, date: NaiveDate) -> bool {
        date >= self.start_date() && date <= self.end_date()
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

    // is there a container for this month?
    fn is_container_focused(&self) -> bool {
        self.container
            .as_ref()
            .map(|v| v.is_focused())
            .unwrap_or(false)
    }
}

impl<Selection> MonthState<Selection>
where
    Selection: CalendarSelection,
{
    /// Removes all selection.
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }
}

impl MonthState<NoSelection> {}

impl MonthState<SingleSelection> {
    /// Select a day
    pub fn select_day(&mut self, n: usize) -> CalOutcome {
        if let Some(date) = self.start_date.with_day0(n as u32) {
            if self.selection.borrow_mut().select(date) {
                CalOutcome::Selected
            } else {
                CalOutcome::Continue
            }
        } else {
            CalOutcome::Continue
        }
    }

    /// Select a day
    pub fn select_last(&mut self) -> CalOutcome {
        let date = self.end_date();
        if self.selection.borrow_mut().select(date) {
            CalOutcome::Selected
        } else {
            CalOutcome::Continue
        }
    }

    /// Select by date.
    /// Returns true if the date is valid for this month.
    /// If false it doesn't change the selection.
    pub fn select_date(&mut self, d: NaiveDate) -> bool {
        let start = self.start_date;
        if d.year() == start.year() && d.month() == start.month() {
            self.selection.borrow_mut().select(d)
        } else {
            false
        }
    }

    /// Lead selected day
    pub fn selected_date(&self) -> Option<NaiveDate> {
        self.selection.lead_selection()
    }

    /// Select previous day.
    pub fn prev_day(&mut self, n: usize) -> CalOutcome {
        let base_start = self.start_date();
        let base_end = self.end_date();

        let date = if let Some(date) = self.selection.lead_selection() {
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

        if self.in_range(date) {
            if self.selection.borrow_mut().select(date) {
                CalOutcome::Selected
            } else {
                CalOutcome::Continue
            }
        } else {
            CalOutcome::Continue
        }
    }

    /// Select previous day.
    pub fn next_day(&mut self, n: usize) -> CalOutcome {
        let base_start = self.start_date();
        let base_end = self.end_date();

        let date = if let Some(date) = self.selection.lead_selection() {
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

        if self.in_range(date) {
            if self.selection.borrow_mut().select(date) {
                CalOutcome::Selected
            } else {
                CalOutcome::Continue
            }
        } else {
            CalOutcome::Continue
        }
    }
}

impl MonthState<RangeSelection> {
    /// Select a week.
    pub fn select_week(&mut self, n: usize, extend: bool) -> CalOutcome {
        if n < self.week_len() {
            let date = self.start_date() + Days::new(7 * n as u64);
            if self.selection.borrow_mut().select_week(date, extend) {
                CalOutcome::Selected
            } else {
                CalOutcome::Continue
            }
        } else {
            CalOutcome::Continue
        }
    }

    /// Select a day
    pub fn select_day(&mut self, n: usize, extend: bool) -> CalOutcome {
        if let Some(date) = self.start_date.with_day0(n as u32) {
            if self.selection.borrow_mut().select_day(date, extend) {
                CalOutcome::Selected
            } else {
                CalOutcome::Continue
            }
        } else {
            CalOutcome::Continue
        }
    }

    /// Select a day
    pub fn select_last(&mut self, extend: bool) -> CalOutcome {
        let date = self.end_date();
        if self.selection.borrow_mut().select_day(date, extend) {
            CalOutcome::Selected
        } else {
            CalOutcome::Continue
        }
    }

    /// Select a week by date
    /// Returns true if the date is valid for this month.
    /// If false it doesn't change the selection.
    pub fn select_week_by_date(&mut self, date: NaiveDate, extend: bool) -> bool {
        let base = self.start_date;

        let start = date.week(Weekday::Mon).first_day();
        let end = date.week(Weekday::Mon).last_day();

        if (start.year() == base.year() && start.month() == base.month())
            || (end.year() == base.year() && end.month() == base.month())
        {
            self.selection.borrow_mut().select_week(start, extend)
        } else {
            false
        }
    }

    /// Select by date.
    /// Returns true if the date is valid for this month.
    /// If false it doesn't change the selection.
    pub fn select_date(&mut self, d: NaiveDate, extend: bool) -> bool {
        let base = self.start_date;
        if d.year() == base.year() && d.month() == base.month() {
            self.selection.borrow_mut().select_day(d, extend)
        } else {
            false
        }
    }

    /// Lead selected day
    pub fn selected_date(&self) -> Option<NaiveDate> {
        self.selection.lead_selection()
    }

    /// Select previous day.
    pub fn prev_day(&mut self, n: usize, extend: bool) -> CalOutcome {
        let base_start = self.start_date();
        let base_end = self.end_date();

        let date = if let Some(date) = self.selection.lead_selection() {
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

        if self.in_range(date) {
            if self.selection.borrow_mut().select_day(date, extend) {
                CalOutcome::Selected
            } else {
                CalOutcome::Continue
            }
        } else {
            CalOutcome::Continue
        }
    }

    /// Select previous day.
    pub fn next_day(&mut self, n: usize, extend: bool) -> CalOutcome {
        let base_start = self.start_date();
        let base_end = self.end_date();

        let date = if let Some(date) = self.selection.lead_selection() {
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

        if self.in_range(date) {
            if self.selection.borrow_mut().select_day(date, extend) {
                CalOutcome::Selected
            } else {
                CalOutcome::Continue
            }
        } else {
            CalOutcome::Continue
        }
    }

    /// Select previous week.
    pub fn prev_week(&mut self, n: usize, extend: bool) -> CalOutcome {
        let base_start = self.start_date();
        let base_end = self.end_date();

        if let Some(date) = self.selection.lead_selection() {
            let new_date = if date >= base_start && date <= base_end {
                date - Days::new(7 * n as u64)
            } else if date < base_start {
                self.start_date()
            } else {
                self.end_date()
            };
            let new_date_end = new_date.week(Weekday::Mon).last_day();
            if new_date_end >= base_start && new_date_end <= base_end {
                if self.selection.borrow_mut().select_week(new_date, extend) {
                    CalOutcome::Selected
                } else {
                    CalOutcome::Continue
                }
            } else {
                CalOutcome::Continue
            }
        } else {
            let new_date = self.end_date();
            if self.selection.borrow_mut().select_week(new_date, extend) {
                CalOutcome::Selected
            } else {
                CalOutcome::Continue
            }
        }
    }

    /// Select previous day.
    pub fn next_week(&mut self, n: usize, extend: bool) -> CalOutcome {
        let start = self.start_date();
        let end = self.end_date();

        let new_date = if let Some(date) = self.selection.lead_selection() {
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
            if self.selection.borrow_mut().select_week(new_date, extend) {
                CalOutcome::Selected
            } else {
                CalOutcome::Continue
            }
        } else {
            CalOutcome::Continue
        }
    }
}
