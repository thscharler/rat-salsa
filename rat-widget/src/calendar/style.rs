use crate::_private::NonExhaustive;
use rat_scrolled::block_style::BlockStyle;
use ratatui::style::Style;
use ratatui::widgets::Block;
#[cfg(feature = "serde")]
use serde_derive::{Deserialize, Serialize};

/// Composite style for the calendar.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    #[cfg_attr(feature = "serde", serde(skip))]
    pub block: Option<Block<'static>>,
    /// Button border
    pub block_style: Option<BlockStyle>,

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
            block_style: None,
            non_exhaustive: NonExhaustive,
        }
    }
}
