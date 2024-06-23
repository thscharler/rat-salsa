//!
//! Draws a simple line menu.
//!
//! If the render area has more than one line, this will
//! linebreak if necessary.
//!
//! ## Navigation keys
//! If you give plain-text strings as items, the underscore
//! designates a navigation key. If you hit the key, the matching
//! item is selected. On the second hit, the matching item is
//! activated.
//!
use crate::_private::NonExhaustive;
use crate::event::{FocusKeys, HandleEvent, MouseOnly};
#[allow(unused_imports)]
use log::debug;
use rat_focus::{FocusFlag, HasFocusFlag};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Line;
use ratatui::style::Style;
use ratatui::widgets::{StatefulWidget, StatefulWidgetRef};

pub use rat_input::menuline::{MenuOutcome, MenuStyle};

///
/// Menu widget.
///
/// If the text exceeds the area width it wraps around.
///
#[derive(Debug, Default, Clone)]
pub struct RMenuLine<'a> {
    widget: rat_input::menuline::MenuLine<'a>,
}

///
/// State for the menu widget
///
#[derive(Debug, Clone)]
pub struct RMenuLineState {
    /// State of the inner widget.
    pub widget: rat_input::menuline::MenuLineState,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> RMenuLine<'a> {
    /// New
    pub fn new() -> Self {
        Default::default()
    }

    /// Title text.
    #[inline]
    pub fn title(mut self, title: impl Into<Line<'a>>) -> Self {
        self.widget = self.widget.title(title);
        self
    }

    /// Add a formatted item.
    /// The navchar is optional, any markup for it is your problem.
    pub fn add(mut self, item: Line<'a>, navchar: Option<char>) -> Self {
        self.widget = self.widget.add(item, navchar);
        self
    }

    /// Add item.
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn add_str(mut self, menu_item: &'a str) -> Self {
        self.widget = self.widget.add_str(menu_item);
        self
    }

    /// Combined style.
    #[inline]
    pub fn styles(mut self, styles: MenuStyle) -> Self {
        self.widget = self.widget.styles(styles);
        self
    }

    /// Base style.
    #[inline]
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.style(style);
        self
    }

    /// Menu-title style.
    #[inline]
    pub fn title_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.title_style(style);
        self
    }

    /// Selection
    #[inline]
    pub fn select_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.select_style(style);
        self
    }

    /// Selection + Focus
    #[inline]
    pub fn focus_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.focus_style(style);
        self
    }
}

impl<'a> StatefulWidgetRef for RMenuLine<'a> {
    type State = RMenuLineState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget.render_ref(area, buf, &mut state.widget)
    }
}

impl<'a> StatefulWidget for RMenuLine<'a> {
    type State = RMenuLineState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget.render(area, buf, &mut state.widget)
    }
}

impl Default for RMenuLineState {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

#[allow(clippy::len_without_is_empty)]
impl RMenuLineState {
    /// New
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of items.
    #[inline]
    pub fn len(&self) -> usize {
        self.widget.len()
    }

    /// Any items.
    pub fn is_empty(&self) -> bool {
        self.widget.is_empty()
    }

    /// Select
    #[inline]
    pub fn select(&mut self, select: Option<usize>) -> bool {
        self.widget.select(select)
    }

    /// Selected index
    #[inline]
    pub fn selected(&self) -> Option<usize> {
        self.widget.selected
    }

    /// Previous item.
    #[inline]
    pub fn prev(&mut self) -> bool {
        self.widget.prev()
    }

    /// Next item.
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> bool {
        self.widget.next()
    }

    /// Select by hotkey
    #[inline]
    pub fn navigate(&mut self, c: char) -> MenuOutcome {
        self.widget.navigate(c)
    }

    /// Select item at position
    #[inline]
    pub fn select_at(&mut self, pos: (u16, u16)) -> bool {
        self.widget.select_at(pos)
    }

    /// Item at position.
    #[inline]
    pub fn item_at(&self, pos: (u16, u16)) -> Option<usize> {
        self.widget.item_at(pos)
    }
}

impl HasFocusFlag for RMenuLineState {
    /// Focus flag.
    fn focus(&self) -> &FocusFlag {
        &self.widget.focus
    }

    /// Focus area.
    fn area(&self) -> Rect {
        self.widget.area
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, MenuOutcome> for RMenuLineState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> MenuOutcome {
        if self.is_focused() {
            self.widget.handle(event, FocusKeys)
        } else {
            self.widget.handle(event, MouseOnly)
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, MenuOutcome> for RMenuLineState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> MenuOutcome {
        self.widget.handle(event, MouseOnly)
    }
}
