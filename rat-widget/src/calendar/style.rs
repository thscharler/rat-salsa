use crate::_private::NonExhaustive;
use ratatui::style::Style;
use ratatui::widgets::Block;

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
