use crate::_private::NonExhaustive;
use ratatui::style::Style;
use ratatui::widgets::Block;

/// Composite style for the calendar.
#[derive(Debug, Clone)]
pub struct CalendarStyle {
    /// Base style.
    pub style: Style,
    /// Block.
    pub block: Option<Block<'static>>,
    pub border_style: Option<Style>,
    pub title_style: Option<Style>,
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

    pub non_exhaustive: NonExhaustive,
}

impl Default for CalendarStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            block: Default::default(),
            border_style: Default::default(),
            title_style: Default::default(),
            title: Default::default(),
            weeknum: Default::default(),
            weekday: Default::default(),
            day: Default::default(),
            select: Default::default(),
            focus: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}
