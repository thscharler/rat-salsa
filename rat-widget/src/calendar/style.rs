use crate::_private::NonExhaustive;
use ratatui::style::Style;
use ratatui::widgets::Block;

/// Composite style for the calendar.
#[derive(Debug, Clone)]
pub struct CalendarStyle {
    /// Base style.
    pub style: Style,
    /// Title style.
    pub title: Option<Style>,
    /// Week-number style.
    pub weeknum: Option<Style>,
    /// Weekday style.
    pub weekday: Option<Style>,
    /// Default day style.
    pub day: Option<Style>,
    /// Selection style.
    pub select: Option<Style>,
    /// Focused style.
    pub focus: Option<Style>,
    /// Block.
    pub block: Option<Block<'static>>,

    pub non_exhaustive: NonExhaustive,
}

impl Default for CalendarStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            title: None,
            weeknum: None,
            weekday: None,
            day: None,
            select: None,
            focus: None,
            block: None,
            non_exhaustive: NonExhaustive,
        }
    }
}
