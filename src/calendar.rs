//!
//! Render a month of a calender.
//!

use chrono::{Datelike, NaiveDate, Weekday};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Span, StatefulWidget, Text};
use ratatui::style::Style;
use ratatui::widgets::Widget;
use std::fmt::{Debug, Formatter};

/// Renders a month.
pub struct Month {
    /// Title style.
    pub title_style: Style,
    /// Week number style.
    pub week_style: Style,
    /// Styling for a single date.
    pub day_style: Box<dyn Fn(NaiveDate) -> Style>,
    /// Start date of the month.
    pub start_date: NaiveDate,
}

/// Composite style for the calendar.
pub struct MonthStyle {
    pub title_style: Style,
    pub week_style: Style,
    pub day_style: Box<dyn Fn(NaiveDate) -> Style>,
}

/// Month state.
#[derive(Debug, Default, Clone, Copy)]
pub struct MonthState {
    pub area_month: Rect,
    pub area_days: [Rect; 31],
    pub weeks: [Rect; 6],
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
    /// Sets the starting date.
    pub fn date(mut self, s: NaiveDate) -> Self {
        self.start_date = s;
        self
    }

    /// Set the composite style.
    pub fn style(mut self, s: MonthStyle) -> Self {
        self.title_style = s.title_style;
        self.week_style = s.week_style;
        self.day_style = s.day_style;
        self
    }

    /// Set the day style.
    pub fn day_style(mut self, s: Box<dyn Fn(NaiveDate) -> Style>) -> Self {
        self.day_style = s;
        self
    }

    /// Set the week number style
    pub fn week_style(mut self, s: impl Into<Style>) -> Self {
        self.week_style = s.into();
        self
    }

    /// Set the month-name style.
    pub fn title_style(mut self, s: impl Into<Style>) -> Self {
        self.title_style = s.into();
        self
    }

    /// Required width for the widget.
    pub fn width(&self) -> usize {
        8 * 3
    }

    /// Required height for the widget. Varies.
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
            for _ in 0..7 {
                day += chrono::Duration::try_days(1).expect("days");
            }
            r += 1;
        }

        r
    }
}

impl StatefulWidget for Month {
    type State = MonthState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut day = self.start_date;
        let month = self.start_date.month();

        let mut w = 0;
        let mut x = area.x;
        let mut y = area.y;

        let day_style = self.day_style.as_ref();

        let mut w_month = Text::default();

        let w_title = Line::styled(day.format("%B").to_string(), self.title_style);
        state.area_month = Rect::new(x, y, w_title.width() as u16, 1);
        w_month.lines.push(w_title);
        y += 1;

        // first line may omit a few days
        let mut w_week = Line::default();
        let w_weeknum = Span::from(day.format("%U").to_string()).style(self.week_style.clone());
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
                let w_date = Span::from(day.format("%e").to_string()).style(day_style(day));
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
            let w_weeknum = Span::from(day.format("%U").to_string()).style(self.week_style.clone());
            state.weeks[w] = Rect::new(x, y, w_weeknum.width() as u16, 1);
            w_week.spans.push(w_weeknum);
            w_week.spans.push(" ".into());
            x += 3;

            for _ in 0..7 {
                if day.month() == month {
                    let w_date = Span::from(day.format("%e").to_string()).style(day_style(day));
                    state.area_days[day.day0() as usize] =
                        Rect::new(x, y, w_date.width() as u16, 1);
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
}
