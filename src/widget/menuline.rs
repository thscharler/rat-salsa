//!
//! A menu.
//!
//! Supports hot-keys with '_' in the item text. The keys are trigger with Ctrl or the plain
//! key if the menu has focus.
//!

use crate::util::span_width;
use crate::widget::MouseFlags;
use crate::{ct_event, ControlUI, FocusFlag, HasFocusFlag, SingleSelection};
use crate::{DefaultKeys, HandleCrossterm, MouseOnly};
use crossterm::event::Event;
#[allow(unused_imports)]
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::prelude::{Modifier, Span, Style, Widget};
use ratatui::text::{Line, Text};
use ratatui::widgets::StatefulWidget;
use std::fmt::Debug;

/// Menu
#[derive(Debug)]
pub struct MenuLine<'a, A> {
    pub style: Style,
    pub title_style: Style,
    pub select_style: Style,
    pub focus_style: Style,
    pub title: Span<'a>,
    pub key: Vec<char>,
    pub menu: Vec<Vec<Span<'a>>>,
    pub action: Vec<A>,
}

/// Combined styles.
#[derive(Debug, Default)]
pub struct MenuStyle {
    pub style: Style,
    pub title: Style,
    pub select: Style,
    pub focus: Style,
}

impl<'a, A> Default for MenuLine<'a, A> {
    fn default() -> Self {
        Self {
            style: Default::default(),
            title_style: Default::default(),
            select_style: Default::default(),
            focus_style: Default::default(),
            title: Default::default(),
            key: vec![],
            menu: vec![],
            action: vec![],
        }
    }
}

impl<'a, A> MenuLine<'a, A> {
    /// New
    pub fn new() -> Self {
        Default::default()
    }

    /// Combined style.
    pub fn style(mut self, styles: MenuStyle) -> Self {
        self.style = styles.style;
        self.title_style = styles.title;
        self.select_style = styles.select;
        self.focus_style = styles.focus;
        self
    }

    /// Base style.
    pub fn base_style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Menu-title style.
    pub fn title_style(mut self, style: impl Into<Style>) -> Self {
        self.title_style = style.into();
        self
    }

    /// Selection
    pub fn select_style(mut self, style: impl Into<Style>) -> Self {
        self.select_style = style.into();
        self
    }

    /// Selection + Focus
    pub fn select_style_focus(mut self, style: impl Into<Style>) -> Self {
        self.focus_style = style.into();
        self
    }

    /// Title text.
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Span::from(title);
        self
    }

    /// Add item.
    pub fn add(mut self, menu_item: &'a str, action: A) -> Self {
        let (key, item) = menu_span(menu_item);
        self.key.push(key);
        self.menu.push(item);
        self.action.push(action);
        self
    }
}

impl<'a, A> StatefulWidget for MenuLine<'a, A> {
    type State = MenuLineState<A>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut row = area.y;
        let mut col = area.x;

        state.area = area;
        state.key = self.key;
        if let Some(selected) = state.select.selected() {
            state.select.select_clamped(selected, self.menu.len());
        }
        state.action = self.action;

        let mut text = Text::default();
        let mut line = Line::default();

        if !self.title.content.is_empty() {
            let title_width = self.title.width() as u16;

            line.spans.push(self.title.style(self.title_style));
            line.spans.push(" ".into());

            col += title_width + 1;
        }

        for (n, mut item) in self.menu.into_iter().enumerate() {
            let item_width = span_width(&item);
            if col + item_width > area.x + area.width {
                text.lines.push(line);
                line = Line::default();

                row += 1;
                col = area.x;
            }

            if state.select.selected() == Some(n) {
                for v in &mut item {
                    v.style = v.style.patch(if state.focus.get() {
                        self.focus_style
                    } else {
                        self.select_style
                    })
                }
            }

            state.areas.push(Rect::new(col, row, item_width, 1));

            line.spans.extend(item);
            line.spans.push(" ".into());

            col += item_width + 1;
        }
        text.lines.push(line);

        text.style = self.style;
        text.render(area, buf);
    }
}

fn menu_span(txt: &str) -> (char, Vec<Span<'_>>) {
    let mut key = char::default();
    let mut menu = Vec::new();

    let mut it = txt.split('_');
    if let Some(p) = it.next() {
        if !p.is_empty() {
            menu.push(Span::from(p));
        }
    }

    for t in it {
        let mut cit = t.char_indices();
        // mark first char
        cit.next();
        if let Some((i, _)) = cit.next() {
            let (t0, t1) = t.split_at(i);

            key = t0.chars().next().expect("char");
            key = key.to_lowercase().next().expect("char");

            menu.push(Span::styled(t0, Style::from(Modifier::UNDERLINED)));
            menu.push(Span::from(t1));
        } else {
            menu.push(Span::from(t));
        }
    }

    (key, menu)
}

/// State for the menu.
#[derive(Debug)]
pub struct MenuLineState<A> {
    /// Focus
    pub focus: FocusFlag,
    pub area: Rect,
    pub areas: Vec<Rect>,
    pub key: Vec<char>,
    pub action: Vec<A>,
    pub select: SingleSelection,
    pub mouse: MouseFlags,
}

#[allow(clippy::len_without_is_empty)]
impl<A> MenuLineState<A> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.action.len()
    }

    pub fn selected(&self) -> Option<usize> {
        self.select.selected()
    }

    pub fn select(&mut self, select: Option<usize>) {
        self.select.select(select);
    }

    pub fn select_by_key(&mut self, cc: char) {
        let cc = cc.to_ascii_lowercase();
        for (i, k) in self.key.iter().enumerate() {
            if cc == *k {
                self.select.select(Some(i));
                break;
            }
        }
    }

    pub fn item_at(&self, pos: Position) -> Option<usize> {
        for (i, r) in self.areas.iter().enumerate() {
            if r.contains(pos) {
                return Some(i);
            }
        }
        None
    }

    pub fn next(&mut self) {
        self.select.next(1, self.len());
    }

    pub fn prev(&mut self) {
        self.select.prev(1);
    }

    pub fn action(&self) -> Option<A>
    where
        A: Copy,
    {
        match self.select.selected() {
            Some(i) => self.action.get(i).copied(),
            None => None,
        }
    }
}

impl<A> Default for MenuLineState<A> {
    fn default() -> Self {
        Self {
            focus: Default::default(),
            key: Default::default(),
            mouse: Default::default(),
            select: Default::default(),
            areas: Default::default(),
            action: Default::default(),
            area: Default::default(),
        }
    }
}

impl<A> HasFocusFlag for MenuLineState<A> {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.area
    }
}

/// React to ctrl + menu shortcut.
#[derive(Debug)]
pub struct HotKeyCtrl;

impl<A: Copy, E> HandleCrossterm<ControlUI<A, E>, HotKeyCtrl> for MenuLineState<A> {
    fn handle(&mut self, event: &Event, _: HotKeyCtrl) -> ControlUI<A, E> {
        match event {
            ct_event!(key press CONTROL-cc) => {
                self.select_by_key(*cc);
                match self.action() {
                    Some(a) => ControlUI::Run(a),
                    None => ControlUI::Continue,
                }
            }
            _ => ControlUI::Continue,
        }
    }
}

/// React to alt + menu shortcut.
#[derive(Debug)]
pub struct HotKeyAlt;

impl<A: Copy + Debug, E: Debug> HandleCrossterm<ControlUI<A, E>, HotKeyAlt> for MenuLineState<A> {
    fn handle(&mut self, event: &Event, _: HotKeyAlt) -> ControlUI<A, E> {
        match event {
            ct_event!(key press ALT-cc) => {
                self.select_by_key(*cc);
                match self.action() {
                    Some(a) => ControlUI::Run(a),
                    None => ControlUI::Continue,
                }
            }
            _ => ControlUI::Continue,
        }
    }
}

impl<A: Copy, E> HandleCrossterm<ControlUI<A, E>, DefaultKeys> for MenuLineState<A> {
    fn handle(&mut self, event: &Event, _: DefaultKeys) -> ControlUI<A, E> {
        let res = if self.is_focused() {
            match event {
                ct_event!(key press cc) => {
                    self.select_by_key(*cc);
                    match self.action() {
                        Some(a) => ControlUI::Run(a),
                        None => ControlUI::Continue,
                    }
                }
                ct_event!(keycode press Left) => {
                    self.prev();
                    ControlUI::Change
                }
                ct_event!(keycode press Right) => {
                    self.next();
                    ControlUI::Change
                }
                ct_event!(keycode press Home) => {
                    self.select(Some(0));
                    ControlUI::Change
                }
                ct_event!(keycode press End) => {
                    self.select(Some(self.len() - 1));
                    ControlUI::Change
                }
                ct_event!(keycode press Enter) => match self.action() {
                    Some(a) => ControlUI::Run(a),
                    None => ControlUI::Continue,
                },
                _ => ControlUI::Continue,
            }
        } else {
            ControlUI::Continue
        };

        res.or_else(|| self.handle(event, MouseOnly))
    }
}

impl<A: Copy, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for MenuLineState<A> {
    fn handle(&mut self, event: &Event, _: MouseOnly) -> ControlUI<A, E> {
        match event {
            ct_event!(mouse down Left for col, row) => {
                if let Some(i) = self.item_at(Position::new(*col, *row)) {
                    self.mouse.set_drag();
                    self.select(Some(i));
                    ControlUI::Change
                } else {
                    ControlUI::Continue
                }
            }
            ct_event!(mouse drag Left for col, row) => {
                if self.mouse.do_drag() {
                    if let Some(i) = self.item_at(Position::new(*col, *row)) {
                        self.mouse.set_drag();
                        self.select(Some(i));
                        ControlUI::Change
                    } else {
                        ControlUI::Continue
                    }
                } else {
                    ControlUI::Continue
                }
            }
            ct_event!(mouse up Left for col,row) => {
                let idx = self.item_at(Position::new(*col, *row));
                if self.selected() == idx && self.mouse.pull_trigger(500) {
                    match self.action() {
                        Some(a) => ControlUI::Run(a),
                        None => ControlUI::Continue,
                    }
                } else {
                    ControlUI::Continue
                }
            }
            ct_event!(mouse moved) => {
                self.mouse.clear_drag();
                ControlUI::Continue
            }
            _ => ControlUI::Continue,
        }
    }
}
