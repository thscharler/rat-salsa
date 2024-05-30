use crate::_private::NonExhaustive;
use rat_focus::{FocusFlag, HasFocusFlag};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::StatefulWidget;

use crate::event::{FocusKeys, HandleEvent, MouseOnly};
pub use rat_input::menuline::{HotKeyAlt, HotKeyCtrl, MenuOutcome, MenuStyle};

#[derive(Debug, Default, Clone)]
pub struct MenuLine<'a> {
    widget: rat_input::menuline::MenuLine<'a>,
}

#[derive(Debug, Clone)]
pub struct MenuLineState {
    pub widget: rat_input::menuline::MenuLineState,
    pub focus: FocusFlag,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> MenuLine<'a> {
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

// impl<'a> StatefulWidgetRef for MenuLine<'a> {
//     type State = MenuLineState;
//
//     fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
//         self.widget
//             .clone()
//             .focused(state.is_focused())
//             .render(area, buf, &mut state.widget)
//     }
// }

impl<'a> StatefulWidget for MenuLine<'a> {
    type State = MenuLineState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget
            .focused(state.is_focused())
            .render(area, buf, &mut state.widget)
    }
}

impl Default for MenuLineState {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            focus: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

#[allow(clippy::len_without_is_empty)]
impl MenuLineState {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.widget.len()
    }

    #[inline]
    pub fn selected(&self) -> Option<usize> {
        self.widget.selected
    }

    #[inline]
    pub fn select(&mut self, select: Option<usize>) {
        self.widget.select(select)
    }

    #[inline]
    pub fn select_by_key(&mut self, cc: char) {
        self.widget.select_by_key(cc)
    }

    #[inline]
    pub fn item_at(&self, pos: (u16, u16)) -> Option<usize> {
        self.widget.item_at(pos)
    }

    #[inline]
    pub fn next(&mut self) {
        self.widget.next()
    }

    #[inline]
    pub fn prev(&mut self) {
        self.widget.prev()
    }
}

impl HasFocusFlag for MenuLineState {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.widget.area
    }
}

impl HandleEvent<crossterm::event::Event, HotKeyCtrl, MenuOutcome> for MenuLineState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: HotKeyCtrl) -> MenuOutcome {
        self.widget.handle(event, HotKeyCtrl)
    }
}

impl HandleEvent<crossterm::event::Event, HotKeyAlt, MenuOutcome> for MenuLineState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: HotKeyAlt) -> MenuOutcome {
        self.widget.handle(event, HotKeyAlt)
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, MenuOutcome> for MenuLineState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> MenuOutcome {
        if self.is_focused() {
            self.widget.handle(event, FocusKeys)
        } else {
            self.widget.handle(event, MouseOnly)
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, MenuOutcome> for MenuLineState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> MenuOutcome {
        self.widget.handle(event, MouseOnly)
    }
}
