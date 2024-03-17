//!
//! A menu.
//!
//! Supports hot-keys with '_' in the item text. The keys are trigger with Ctrl or the plain
//! key if the menu has focus.
//!

use crate::util::{clamp_opt, next_opt, prev_opt, span_width};
use crate::widget::ActionTrigger;
use crate::ControlUI;
use crate::{DefaultKeys, HandleCrossterm, Input, MouseOnly};
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
#[allow(unused_imports)]
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::prelude::{Modifier, Span, Style, Widget};
use ratatui::text::{Line, Text};
use ratatui::widgets::StatefulWidget;
use std::cell::Cell;
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

/// Menu ops.
#[derive(Debug)]
pub enum InputRequest {
    Prev,
    Next,
    Action,
    KeySelect(char),
    KeyAction(char),
    MouseSelect(usize),
    MouseAction(usize, u64),
}

/// State for the menu.
#[derive(Debug)]
pub struct MenuLineState<A> {
    pub focus: Cell<bool>,
    pub area: Vec<Rect>,
    pub key: Vec<char>,
    pub trigger: ActionTrigger,
    pub select: Option<usize>,
    pub len: usize,
    pub action: Vec<A>,
}

impl<A> Default for MenuLineState<A> {
    fn default() -> Self {
        Self {
            focus: Cell::new(false),
            key: Default::default(),
            trigger: Default::default(),
            select: Some(0),
            len: Default::default(),
            area: Default::default(),
            action: Default::default(),
        }
    }
}

impl<A: Copy, E> Input<ControlUI<A, E>> for MenuLineState<A> {
    type Request = InputRequest;

    fn perform(&mut self, req: Self::Request) -> ControlUI<A, E> {
        match req {
            InputRequest::Prev => {
                self.trigger.reset();
                self.select = prev_opt(self.select);
                ControlUI::Change
            }
            InputRequest::Next => {
                self.trigger.reset();
                self.select = next_opt(self.select, self.len);
                ControlUI::Change
            }
            InputRequest::Action => {
                if let Some(i) = self.select {
                    ControlUI::Run(self.action[i])
                } else {
                    ControlUI::NoChange
                }
            }
            InputRequest::KeySelect(cc) => 'f: {
                for (i, k) in self.key.iter().enumerate() {
                    if cc == *k {
                        self.trigger.reset();
                        self.select = Some(i);
                        break 'f ControlUI::Change;
                    }
                }
                ControlUI::Continue
            }
            InputRequest::KeyAction(cc) => 'f: {
                for (i, k) in self.key.iter().enumerate() {
                    if cc == *k {
                        self.trigger.reset();
                        self.select = Some(i);
                        break 'f ControlUI::Run(self.action[i]);
                    }
                }
                ControlUI::Continue
            }
            InputRequest::MouseSelect(i) => {
                if self.select == Some(i) {
                    ControlUI::NoChange
                } else {
                    self.trigger.reset();
                    self.select = Some(i);
                    ControlUI::Change
                }
            }
            InputRequest::MouseAction(i, timeout) => {
                if self.select == Some(i) {
                    if self.trigger.pull(timeout) {
                        ControlUI::Run(self.action[i])
                    } else {
                        ControlUI::NoChange
                    }
                } else {
                    self.trigger.reset();
                    ControlUI::NoChange
                }
            }
        }
    }
}

impl<A: Copy, E> HandleCrossterm<ControlUI<A, E>> for MenuLineState<A> {
    fn handle(&mut self, event: &Event, _: DefaultKeys) -> ControlUI<A, E> {
        let req = match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char(cc),
                modifiers: mm @ KeyModifiers::NONE | mm @ KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if *mm == KeyModifiers::NONE && !self.focus.get() {
                    None
                } else {
                    Some(InputRequest::KeyAction(*cc))
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => Some(InputRequest::Prev),
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => Some(InputRequest::Next),
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => Some(InputRequest::Action),

            _ => return self.handle(event, MouseOnly),
        };

        if let Some(req) = req {
            self.perform(req)
        } else {
            ControlUI::Continue
        }
    }
}

impl<A: Copy, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for MenuLineState<A> {
    fn handle(&mut self, event: &Event, _: MouseOnly) -> ControlUI<A, E> {
        let req = match event {
            Event::Mouse(
                MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column,
                    row,
                    modifiers: KeyModifiers::NONE,
                }
                | MouseEvent {
                    kind: MouseEventKind::Drag(MouseButton::Left),
                    column,
                    row,
                    modifiers: KeyModifiers::NONE,
                },
            ) => 'f: {
                for (i, r) in self.area.iter().enumerate() {
                    if r.contains(Position::new(*column, *row)) {
                        break 'f Some(InputRequest::MouseSelect(i));
                    }
                }
                None
            }
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left),
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => 'f: {
                for (i, r) in self.area.iter().enumerate() {
                    if r.contains(Position::new(*column, *row)) {
                        break 'f Some(InputRequest::MouseAction(i, 1000));
                    }
                }
                None
            }
            _ => None,
        };

        if let Some(req) = req {
            self.perform(req)
        } else {
            ControlUI::Continue
        }
    }
}

impl<'a, A> StatefulWidget for MenuLine<'a, A> {
    type State = MenuLineState<A>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut row = area.y;
        let mut col = area.x;

        state.key = self.key;
        state.len = self.menu.len();
        state.select = clamp_opt(state.select, state.len);
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

            if state.select == Some(n) {
                for v in &mut item {
                    v.style = v.style.patch(if state.focus.get() {
                        self.focus_style
                    } else {
                        self.select_style
                    })
                }
            }

            state.area.push(Rect::new(col, row, item_width, 1));

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
