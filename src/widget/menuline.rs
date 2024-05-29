//!
//! A menu.
//!
//! Supports hot-keys with '_' in the item text. The keys are trigger with Ctrl or the plain
//! key if the menu has focus.
//!
use crate::_private::NonExhaustive;
use crate::{ControlUI, FocusFlag, HasFocusFlag};
use crate::{DefaultKeys, HandleCrossterm};
use crossterm::event::Event;
#[allow(unused_imports)]
use log::debug;
use rat_input::event::{FocusKeys, HandleEvent, MouseOnly};
use rat_input::menuline::{MenuLine, MenuLineState, MenuOutcome};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::prelude::Style;
use ratatui::widgets::StatefulWidget;
use std::fmt::Debug;

pub use rat_input::menuline::{HotKeyAlt, HotKeyCtrl, MenuStyle};

/// Menu
#[derive(Debug)]
pub struct MenuLineExt<'a, A> {
    widget: MenuLine<'a>,
    action: Vec<A>,
}

impl<'a, A> Default for MenuLineExt<'a, A> {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            action: vec![],
        }
    }
}

impl<'a, A> MenuLineExt<'a, A> {
    /// New
    pub fn new() -> Self {
        Default::default()
    }

    /// Combined style.
    pub fn styles(mut self, styles: MenuStyle) -> Self {
        self.widget = self.widget.styles(styles);
        self
    }

    /// Base style.
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.style(style);
        self
    }

    /// Menu-title style.
    pub fn title_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.title_style(style);
        self
    }

    /// Selection
    pub fn select_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.select_style(style);
        self
    }

    /// Selection + Focus
    pub fn select_style_focus(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.select_style_focus(style);
        self
    }

    /// Title text.
    pub fn title(mut self, title: &'a str) -> Self {
        self.widget = self.widget.title(title);
        self
    }

    /// Add item.
    pub fn add(mut self, menu_item: &'a str, action: A) -> Self {
        self.widget = self.widget.add(menu_item);
        self.action.push(action);
        self
    }
}

impl<'a, A> StatefulWidget for MenuLineExt<'a, A> {
    type State = MenuLineExtState<A>;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.action = self.action;

        self.widget = self.widget.focused(state.is_focused());
        self.widget.render(area, buf, &mut state.widget);
    }
}

/// State for the menu.
#[derive(Debug)]
pub struct MenuLineExtState<A> {
    pub widget: MenuLineState,
    /// Focus
    pub focus: FocusFlag,
    pub area: Rect,
    pub action: Vec<A>,
    pub non_exhaustive: NonExhaustive,
}

#[allow(clippy::len_without_is_empty)]
impl<A> MenuLineExtState<A> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.widget.len()
    }

    pub fn selected(&self) -> Option<usize> {
        self.widget.selected()
    }

    pub fn select(&mut self, select: Option<usize>) {
        self.widget.select(select);
    }

    pub fn select_by_key(&mut self, cc: char) {
        self.widget.select_by_key(cc);
    }

    pub fn item_at(&self, pos: Position) -> Option<usize> {
        self.widget.item_at(pos)
    }

    pub fn next(&mut self) {
        self.widget.next()
    }

    pub fn prev(&mut self) {
        self.widget.prev()
    }

    pub fn action(&self) -> Option<A>
    where
        A: Copy,
    {
        match self.widget.selected {
            Some(i) => self.action.get(i).copied(),
            None => None,
        }
    }
}

impl<A> Default for MenuLineExtState<A> {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            focus: Default::default(),
            action: Default::default(),
            area: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<A> HasFocusFlag for MenuLineExtState<A> {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.area
    }
}

/// React to ctrl + menu shortcut.

impl<A: Copy, E> HandleCrossterm<ControlUI<A, E>, HotKeyCtrl> for MenuLineExtState<A> {
    fn handle(&mut self, event: &Event, _: HotKeyCtrl) -> ControlUI<A, E> {
        match self.widget.handle(event, HotKeyCtrl) {
            MenuOutcome::NotUsed => ControlUI::Continue,
            MenuOutcome::Unchanged => ControlUI::NoChange,
            MenuOutcome::Changed => ControlUI::Change,
            MenuOutcome::Selected(_) => ControlUI::Change,
            MenuOutcome::Activated(i) => match self.action.get(i) {
                Some(v) => ControlUI::Run(*v),
                None => ControlUI::Continue,
            },
        }
    }
}

/// React to alt + menu shortcut.
impl<A: Copy + Debug, E: Debug> HandleCrossterm<ControlUI<A, E>, HotKeyAlt>
    for MenuLineExtState<A>
{
    fn handle(&mut self, event: &Event, _: HotKeyAlt) -> ControlUI<A, E> {
        match self.widget.handle(event, HotKeyAlt) {
            MenuOutcome::NotUsed => ControlUI::Continue,
            MenuOutcome::Unchanged => ControlUI::NoChange,
            MenuOutcome::Changed => ControlUI::Change,
            MenuOutcome::Selected(_) => ControlUI::Change,
            MenuOutcome::Activated(i) => match self.action.get(i) {
                Some(v) => ControlUI::Run(*v),
                None => ControlUI::Continue,
            },
        }
    }
}

impl<A: Copy, E> HandleCrossterm<ControlUI<A, E>, DefaultKeys> for MenuLineExtState<A> {
    fn handle(&mut self, event: &Event, _: DefaultKeys) -> ControlUI<A, E> {
        if self.is_focused() {
            match self.widget.handle(event, FocusKeys) {
                MenuOutcome::NotUsed => ControlUI::Continue,
                MenuOutcome::Unchanged => ControlUI::NoChange,
                MenuOutcome::Changed => ControlUI::Change,
                MenuOutcome::Selected(_) => ControlUI::Change,
                MenuOutcome::Activated(i) => match self.action.get(i) {
                    Some(v) => ControlUI::Run(*v),
                    None => ControlUI::Continue,
                },
            }
        } else {
            match self.widget.handle(event, MouseOnly) {
                MenuOutcome::NotUsed => ControlUI::Continue,
                MenuOutcome::Unchanged => ControlUI::NoChange,
                MenuOutcome::Changed => ControlUI::Change,
                MenuOutcome::Selected(_) => ControlUI::Change,
                MenuOutcome::Activated(i) => match self.action.get(i) {
                    Some(v) => ControlUI::Run(*v),
                    None => ControlUI::Continue,
                },
            }
        }
    }
}

impl<A: Copy, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for MenuLineExtState<A> {
    fn handle(&mut self, event: &Event, _: MouseOnly) -> ControlUI<A, E> {
        match self.widget.handle(event, MouseOnly) {
            MenuOutcome::NotUsed => ControlUI::Continue,
            MenuOutcome::Unchanged => ControlUI::NoChange,
            MenuOutcome::Changed => ControlUI::Change,
            MenuOutcome::Selected(_) => ControlUI::Change,
            MenuOutcome::Activated(i) => match self.action.get(i) {
                Some(v) => ControlUI::Run(*v),
                None => ControlUI::Continue,
            },
        }
    }
}
