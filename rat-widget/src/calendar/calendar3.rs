use crate::calendar::{CalendarSelection, CalendarState, CalendarStyle, Month};
use chrono::NaiveDate;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Direction, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, StatefulWidget};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem;

///
/// Calendar with 3 month on display.
///
/// Take this as sample for your own fancy calendar.
///
/// ```rust
/// # use std::collections::HashMap;
/// # use chrono::Local;
/// # use pure_rust_locales::Locale;
/// # use ratatui::buffer::Buffer;
/// # use ratatui::layout::{Alignment, Direction};
/// # use ratatui::prelude::Rect;
/// # use ratatui::widgets::{Block, StatefulWidget};
/// # use rat_widget::calendar::{Calendar3, CalendarState, CalendarStyle, TodayPolicy};
/// # use rat_widget::calendar::selection::SingleSelection;
///
/// # let mut buf = Buffer::empty(Rect::new(0, 0, 24, 24));
///
/// let mut style = CalendarStyle::default();
///
/// let mut day_styles = HashMap::default();
///
/// let mut state = CalendarState::<3, SingleSelection>::new();
/// state.set_step(1);
/// state.set_primary_idx(1);
/// state.set_today_policy(TodayPolicy::Index(1));
/// state.set_start_date(Local::now().date_naive());
///
/// Calendar3::new()
///         .direction(Direction::Vertical)
///         .locale(Locale::default())
///         .styles(style)
///         .title_align(Alignment::Left)
///         .day_styles(&day_styles)
///         .show_weekdays()
///         .block(Block::bordered())
///         .render(Rect::new(0,0,24,24), &mut buf, &mut state);
///
/// ```
///
#[derive(Debug, Default)]
pub struct Calendar3<'a, Selection> {
    direction: Direction,
    months: [Month<'a, Selection>; 3],
    phantom: PhantomData<Selection>,
}

impl<'a, Selection> Calendar3<'a, Selection> {
    pub fn new() -> Self
    where
        Selection: Default,
    {
        Self::default()
    }

    /// Locale for month-names, day-names.
    #[inline]
    pub fn locale(mut self, loc: chrono::Locale) -> Self {
        for i in 0..3 {
            self.months[i] = mem::take(&mut self.months[i]).locale(loc);
        }
        self
    }

    #[inline]
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Show weekday titles
    #[inline]
    pub fn show_weekdays(mut self) -> Self {
        for i in 0..3 {
            self.months[i] = mem::take(&mut self.months[i]).show_weekdays();
        }
        self
    }

    /// Set the composite style.
    #[inline]
    pub fn styles(mut self, s: CalendarStyle) -> Self {
        for i in 0..3 {
            self.months[i] = mem::take(&mut self.months[i]).styles(s.clone());
        }
        self
    }

    /// Style for the selected tab.
    pub fn select_style(mut self, style: Style) -> Self {
        for i in 0..3 {
            self.months[i] = mem::take(&mut self.months[i]).select_style(style);
        }
        self
    }

    /// Style for a focused tab.
    pub fn focus_style(mut self, style: Style) -> Self {
        for i in 0..3 {
            self.months[i] = mem::take(&mut self.months[i]).focus_style(style);
        }
        self
    }

    /// Sets the default day-style.
    #[inline]
    pub fn day_style(mut self, s: impl Into<Style>) -> Self {
        let s = s.into();
        for i in 0..3 {
            self.months[i] = mem::take(&mut self.months[i]).day_style(s);
        }
        self
    }

    /// Sets all the day-styles.
    #[inline]
    pub fn day_styles(mut self, styles: &'a HashMap<NaiveDate, Style>) -> Self {
        for i in 0..3 {
            self.months[i] = mem::take(&mut self.months[i]).day_styles(styles);
        }
        self
    }

    /// Set the week number style
    #[inline]
    pub fn week_style(mut self, s: impl Into<Style>) -> Self {
        let s = s.into();
        for i in 0..3 {
            self.months[i] = mem::take(&mut self.months[i]).week_style(s);
        }
        self
    }

    /// Set the week day style
    #[inline]
    pub fn weekday_style(mut self, s: impl Into<Style>) -> Self {
        let s = s.into();
        for i in 0..3 {
            self.months[i] = mem::take(&mut self.months[i]).weekday_style(s);
        }
        self
    }

    /// Set the month-name style.
    #[inline]
    pub fn title_style(mut self, s: impl Into<Style>) -> Self {
        let s = s.into();
        for i in 0..3 {
            self.months[i] = mem::take(&mut self.months[i]).title_style(s);
        }
        self
    }

    /// Set the mont-name align.
    #[inline]
    pub fn title_align(mut self, a: Alignment) -> Self {
        for i in 0..3 {
            self.months[i] = mem::take(&mut self.months[i]).title_align(a);
        }
        self
    }

    /// Block
    #[inline]
    pub fn block(mut self, b: Block<'a>) -> Self {
        for i in 0..3 {
            self.months[i] = mem::take(&mut self.months[i]).block(b.clone());
        }
        self
    }
}

impl<Selection> StatefulWidget for Calendar3<'_, Selection>
where
    Selection: CalendarSelection,
{
    type State = CalendarState<3, Selection>;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        match self.direction {
            Direction::Horizontal => {
                let width = self.months[0].width();
                let height = self
                    .months
                    .iter()
                    .enumerate()
                    .map(|(i, v)| v.height(&state.months[i]))
                    .max()
                    .expect("height");

                let mut area = Rect::new(area.x, area.y, width, height);
                for i in 0..3 {
                    mem::take(&mut self.months[i]).render(area, buf, &mut state.months[i]);
                    area.x += area.width + 2;
                }
            }
            Direction::Vertical => {
                let width = self.months[0].width();

                let mut area = Rect::new(area.x, area.y, width, 0);
                for i in 0..3 {
                    area.height = self.months[i].height(&state.months[0]);
                    mem::take(&mut self.months[i]).render(area, buf, &mut state.months[i]);
                    area.y += area.height + 1;
                }
            }
        }
    }
}
