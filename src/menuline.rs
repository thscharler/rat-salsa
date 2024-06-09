//!
//! A simple menu. No submenus.
//!
//! Supports hot-keys with '_' in the item text.
//!

use crate::_private::NonExhaustive;
use crate::event::{FocusKeys, HandleEvent, MouseOnly};
#[allow(unused_imports)]
use log::debug;
use rat_focus::{FocusFlag, HasFocusFlag};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::StatefulWidget;

pub use rat_input::menuline::{HotKeyAlt, HotKeyCtrl, MenuOutcome, MenuStyle};

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
    /// Focus flag.
    pub focus: FocusFlag,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> RMenuLine<'a> {
    /// New
    pub fn new() -> Self {
        Default::default()
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
    pub fn select_style_focus(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.select_style_focus(style);
        self
    }

    /// Title text.
    #[inline]
    pub fn title(mut self, title: &'a str) -> Self {
        self.widget = self.widget.title(title);
        self
    }

    /// Add item.
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, menu_item: &'a str) -> Self {
        self.widget = self.widget.add(menu_item);
        self
    }
}

impl<'a> StatefulWidget for RMenuLine<'a> {
    type State = RMenuLineState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget
            .focused(state.is_focused())
            .render(area, buf, &mut state.widget)
    }
}

impl Default for RMenuLineState {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            focus: Default::default(),
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

    /// Selected index
    #[inline]
    pub fn selected(&self) -> Option<usize> {
        self.widget.selected
    }

    /// Select
    #[inline]
    pub fn select(&mut self, select: Option<usize>) -> bool {
        self.widget.select(select)
    }

    /// Select by hotkey
    #[inline]
    pub fn select_by_key(&mut self, cc: char) -> bool {
        self.widget.select_by_key(cc)
    }

    /// Item at position.
    #[inline]
    pub fn item_at(&self, pos: (u16, u16)) -> Option<usize> {
        self.widget.item_at(pos)
    }

    /// Next item.
    #[inline]
    pub fn next(&mut self) -> bool {
        self.widget.next()
    }

    /// Previous item.
    #[inline]
    pub fn prev(&mut self) -> bool {
        self.widget.prev()
    }
}

impl HasFocusFlag for RMenuLineState {
    /// Focus flag.
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    /// Focus area.
    fn area(&self) -> Rect {
        self.widget.area
    }
}

impl HandleEvent<crossterm::event::Event, HotKeyCtrl, MenuOutcome> for RMenuLineState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: HotKeyCtrl) -> MenuOutcome {
        self.widget.handle(event, HotKeyCtrl)
    }
}

impl HandleEvent<crossterm::event::Event, HotKeyAlt, MenuOutcome> for RMenuLineState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: HotKeyAlt) -> MenuOutcome {
        self.widget.handle(event, HotKeyAlt)
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
