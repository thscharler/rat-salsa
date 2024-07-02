//!
//! Render a month of a calendar.
//! Can be localized with a chrono::Locale.
//!

use crate::_private::NonExhaustive;
use chrono::{Datelike, NaiveDate, Weekday};
use rat_focus::{FocusFlag, HasFocusFlag};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{StatefulWidget, StatefulWidgetRef, Widget};
use std::fmt::{Debug, Formatter};

/// Renders a month.
pub struct Month {
    /// Title style.
    title_style: Style,
    /// Week number style.
    week_style: Style,
    /// Styling for a single date.
    day_style: Box<dyn Fn(NaiveDate) -> Style>,
    /// Start date of the month.
    start_date: NaiveDate,
    /// Locale
    loc: chrono::Locale,
}

/// Composite style for the calendar.
pub struct MonthStyle {
    pub title_style: Style,
    pub week_style: Style,
    pub day_style: Box<dyn Fn(NaiveDate) -> Style>,
    pub non_exhaustive: NonExhaustive,
}

/// Month state.
#[derive(Debug, Clone)]
pub struct MonthState {
    /// Current focus state.
    pub focus: FocusFlag,
    /// Total area.
    pub area: Rect,
    /// Area for the month name.
    pub area_month: Rect,
    /// Area for the days of the month.
    pub area_days: [Rect; 31],
    /// Area for the week numbers.
    pub weeks: [Rect; 6],

    pub non_exhaustive: NonExhaustive,
}

impl Default for MonthStyle {
    fn default() -> Self {
        Self {
            title_style: Default::default(),
            week_style: Default::default(),
            day_style: Box::new(|_| Style::default()),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl MonthState {
    pub fn new() -> Self {
        Self {
            focus: Default::default(),
            area: Default::default(),
            area_month: Default::default(),
            area_days: [Rect::default(); 31],
            weeks: [Rect::default(); 6],
            non_exhaustive: NonExhaustive,
        }
    }

    /// Renders the widget in focused style.
    ///
    /// This flag is not used for event-handling.
    #[inline]
    pub fn set_focused(&mut self, focus: bool) {
        self.focus.focus.set(focus);
    }

    /// Renders the widget in focused style.
    ///
    /// This flag is not used for event-handling.
    #[inline]
    pub fn is_focused(&mut self) -> bool {
        self.focus.focus.get()
    }
}

impl Default for MonthState {
    fn default() -> Self {
        Self {
            focus: Default::default(),
            area: Default::default(),
            area_month: Default::default(),
            area_days: [Rect::default(); 31],
            weeks: [Rect::default(); 6],
            non_exhaustive: NonExhaustive,
        }
    }
}

impl Debug for MonthStyle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MonthStyle")
            .field("title_style", &self.title_style)
            .field("week_style", &self.week_style)
            .field("day_style", &"... dyn Fn ...")
            .finish()
    }
}

impl Default for Month {
    fn default() -> Self {
        Self {
            title_style: Default::default(),
            week_style: Default::default(),
            day_style: Box::new(|_| Style::default()),
            start_date: Default::default(),
            loc: Default::default(),
        }
    }
}

impl Debug for Month {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Month")
            .field("title_style", &self.title_style)
            .field("week_style", &self.week_style)
            .field("day_style", &"dyn Fn()")
            .field("start_date", &self.start_date)
            .finish()
    }
}

impl Month {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the starting date.
    #[inline]
    pub fn date(mut self, s: NaiveDate) -> Self {
        self.start_date = s;
        self
    }

    #[inline]
    pub fn locale(mut self, loc: chrono::Locale) -> Self {
        self.loc = loc;
        self
    }

    /// Set the composite style.
    #[inline]
    pub fn style(mut self, s: MonthStyle) -> Self {
        self.title_style = s.title_style;
        self.week_style = s.week_style;
        self.day_style = s.day_style;
        self
    }

    /// Sets a closure that is called to calculate the day style.
    #[inline]
    pub fn day_style(mut self, s: Box<dyn Fn(NaiveDate) -> Style>) -> Self {
        self.day_style = s;
        self
    }

    /// Set the week number style
    #[inline]
    pub fn week_style(mut self, s: impl Into<Style>) -> Self {
        self.week_style = s.into();
        self
    }

    /// Set the month-name style.
    #[inline]
    pub fn title_style(mut self, s: impl Into<Style>) -> Self {
        self.title_style = s.into();
        self
    }

    /// Required width for the widget.
    #[inline]
    pub fn width(&self) -> usize {
        8 * 3
    }

    /// Required height for the widget. Varies.
    #[inline]
    pub fn height(&self) -> usize {
        let mut r = 0;
        let mut day = self.start_date;
        let month = day.month();

        // i'm sure you can calculate this better.
        for wd in [
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
            Weekday::Sun,
        ] {
            if day.weekday() == wd {
                day += chrono::Duration::try_days(1).expect("days");
            }
        }
        r += 1;
        while month == day.month() {
            day += chrono::Duration::try_days(7).expect("days");
            r += 1;
        }

        r
    }
}

impl StatefulWidgetRef for Month {
    type State = MonthState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

impl StatefulWidget for Month {
    type State = MonthState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

fn render_ref(widget: &Month, area: Rect, buf: &mut Buffer, state: &mut MonthState) {
    let mut day = widget.start_date;
    let month = widget.start_date.month();

    state.area = area;

    let mut w = 0;
    let mut x = area.x;
    let mut y = area.y;

    let day_style = widget.day_style.as_ref();

    let mut w_month = Text::default();

    let w_title = Line::styled(
        day.format_localized("%B", widget.loc).to_string(),
        widget.title_style,
    );
    state.area_month = Rect::new(x, y, w_title.width() as u16, 1);
    w_month.lines.push(w_title);
    y += 1;

    // first line may omit a few days
    let mut w_week = Line::default();
    let w_weeknum =
        Span::from(day.format_localized("%U", widget.loc).to_string()).style(widget.week_style);
    state.weeks[w] = Rect::new(x, y, w_weeknum.width() as u16, 1);
    w_week.spans.push(w_weeknum);
    w_week.spans.push(" ".into());
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
            w_week.spans.push("   ".into());
            x += 3;
        } else {
            let w_date = Span::from(day.format_localized("%e", widget.loc).to_string())
                .style(day_style(day));
            state.area_days[day.day0() as usize] = Rect::new(x, y, w_date.width() as u16, 1);
            w_week.spans.push(w_date);
            w_week.spans.push(" ".into());
            x += 3;

            day += chrono::Duration::try_days(1).expect("days");
        }
    }
    w_month.lines.push(w_week);

    y += 1;
    x = area.x;
    w += 1;

    while month == day.month() {
        let mut w_week = Line::default();
        let w_weeknum =
            Span::from(day.format_localized("%U", widget.loc).to_string()).style(widget.week_style);
        state.weeks[w] = Rect::new(x, y, w_weeknum.width() as u16, 1);
        w_week.spans.push(w_weeknum);
        w_week.spans.push(" ".into());
        x += 3;

        for _ in 0..7 {
            if day.month() == month {
                let w_date = Span::from(day.format_localized("%e", widget.loc).to_string())
                    .style(day_style(day));
                state.area_days[day.day0() as usize] = Rect::new(x, y, w_date.width() as u16, 1);
                w_week.spans.push(w_date);
                w_week.spans.push(" ".into());
                x += 3;

                day += chrono::Duration::try_days(1).expect("days");
            } else {
                w_week.spans.push("   ".into());
                x += 3;
            }
        }
        w_month.lines.push(w_week);

        y += 1;
        x = area.x;
        w += 1;
    }

    w_month.render(area, buf);
}

impl HasFocusFlag for MonthState {
    #[inline]
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    #[inline]
    fn area(&self) -> Rect {
        self.area
    }
}
