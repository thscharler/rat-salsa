//!
//! Render a month of a calendar.
//!
use crate::_private::NonExhaustive;
use chrono::NaiveDate;
use rat_focus::{FocusFlag, HasFocusFlag};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{StatefulWidget, Style};

pub use rat_input::calendar::MonthStyle;

#[derive(Debug, Default)]
pub struct Month {
    widget: rat_input::calendar::Month,
}

#[derive(Debug, Clone)]
pub struct MonthState {
    pub widget: rat_input::calendar::MonthState,
    pub focus: FocusFlag,

    pub non_exhaustive: NonExhaustive,
}

impl Month {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the starting date.
    #[inline]
    pub fn date(mut self, s: NaiveDate) -> Self {
        self.widget = self.widget.date(s);
        self
    }

    #[inline]
    pub fn locale(mut self, loc: chrono::Locale) -> Self {
        self.widget = self.widget.locale(loc);
        self
    }

    /// Set the composite style.
    #[inline]
    pub fn style(mut self, s: MonthStyle) -> Self {
        self.widget = self.widget.style(s);
        self
    }

    /// Sets a closure that is called to calculate the day style.
    #[inline]
    pub fn day_style(mut self, s: Box<dyn Fn(NaiveDate) -> Style>) -> Self {
        self.widget = self.widget.day_style(s);
        self
    }

    /// Set the week number style
    #[inline]
    pub fn week_style(mut self, s: impl Into<Style>) -> Self {
        self.widget = self.widget.week_style(s);
        self
    }

    /// Set the month-name style.
    #[inline]
    pub fn title_style(mut self, s: impl Into<Style>) -> Self {
        self.widget = self.widget.title_style(s);
        self
    }

    /// Required width for the widget.
    #[inline]
    pub fn width(&self) -> usize {
        self.widget.width()
    }

    /// Required height for the widget. Varies.
    #[inline]
    pub fn height(&self) -> usize {
        self.widget.height()
    }
}

impl StatefulWidget for Month {
    type State = MonthState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget.render(area, buf, &mut state.widget)
    }
}

impl Default for MonthState {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            focus: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl HasFocusFlag for MonthState {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.widget.area
    }
}
