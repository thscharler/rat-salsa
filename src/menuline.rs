use crate::tui::libui::util::{clamp_opt, next_opt, prev_opt, span_width};
use crate::tui::libui::{ControlUI, HandleEvent};
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
use std::time::{Duration, SystemTime};

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

#[derive(Debug, Default)]
pub struct MenuStyle {
    pub style: Style,
    pub title: Style,
    pub select: Style,
    pub focus: Style,
}

#[allow(dead_code)]
impl<'a, A> MenuLine<'a, A> {
    pub fn new() -> Self {
        Self {
            style: Default::default(),
            title_style: Default::default(),
            select_style: Default::default(),
            focus_style: Default::default(),
            title: Default::default(),
            key: Vec::new(),
            menu: Vec::new(),
            action: Vec::new(),
        }
    }

    pub fn style(mut self, styles: MenuStyle) -> Self {
        self.style = styles.style;
        self.title_style = styles.title;
        self.select_style = styles.select;
        self.focus_style = styles.focus;
        self
    }

    pub fn base_style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    pub fn title_style(mut self, style: impl Into<Style>) -> Self {
        self.title_style = style.into();
        self
    }

    pub fn select_style(mut self, style: impl Into<Style>) -> Self {
        self.select_style = style.into();
        self
    }

    pub fn select_style_focus(mut self, style: impl Into<Style>) -> Self {
        self.focus_style = style.into();
        self
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Span::from(title);
        self
    }

    pub fn add(mut self, menu_item: &'a str, action: A) -> Self {
        let (key, item) = menu_span(menu_item);
        self.key.push(key);
        self.menu.push(item);
        self.action.push(action);
        self
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MenuLineState<A> {
    pub focus: Cell<bool>,
    pub area: Vec<Rect>,
    pub key: Vec<char>,
    pub armed: u8,
    pub armed_time: SystemTime,
    pub select: Option<usize>,
    pub len: usize,
    pub action: Vec<A>,
}

impl<A> Default for MenuLineState<A> {
    fn default() -> Self {
        Self {
            focus: Cell::new(false),
            key: Vec::new(),
            armed: 0,
            armed_time: SystemTime::UNIX_EPOCH,
            select: Some(0),
            len: 0,
            area: Vec::new(),
            action: Vec::new(),
        }
    }
}

impl<A: Copy + Debug, E> HandleEvent<A, E> for MenuLineState<A> {
    fn handle(&mut self, evt: &Event) -> ControlUI<A, E> {
        match evt {
            Event::Key(KeyEvent {
                code: KeyCode::Char(cc),
                modifiers: mm @ KeyModifiers::NONE | mm @ KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if *mm == KeyModifiers::NONE && !self.focus.get() {
                    ControlUI::Continue
                } else {
                    'v: {
                        for (i, k) in self.key.iter().enumerate() {
                            if cc == k {
                                self.armed = 0;
                                self.select = Some(i);
                                break 'v ControlUI::Action(self.action[i]);
                            }
                        }
                        ControlUI::Continue
                    }
                }
            }
            Event::Key(KeyEvent {
                code: cc,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => match cc {
                KeyCode::Left => {
                    self.armed = 0;
                    self.select = prev_opt(self.select);
                    ControlUI::Changed
                }
                KeyCode::Right => {
                    self.armed = 0;
                    self.select = next_opt(self.select, self.len);
                    ControlUI::Changed
                }
                KeyCode::Enter => {
                    if let Some(i) = self.select {
                        ControlUI::Action(self.action[i])
                    } else {
                        ControlUI::Unchanged
                    }
                }
                _ => ControlUI::Continue,
            },

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
                        if self.select == Some(i) {
                            break 'f ControlUI::Changed;
                        } else {
                            self.armed = 0;
                            self.select = Some(i);
                            break 'f ControlUI::Changed;
                        }
                    }
                }
                ControlUI::Continue
            }
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left),
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => 'f: {
                for (i, r) in self.area.iter().enumerate() {
                    if r.contains(Position::new(*column, *row)) && self.select == Some(i) {
                        self.armed += 1;
                        if self.armed == 1 {
                            self.armed_time = SystemTime::now();
                        }

                        if self.armed == 1 {
                            break 'f ControlUI::Unchanged;
                        }
                        if self.armed_time.elapsed().expect("timeout") > Duration::from_millis(1000)
                        {
                            self.armed = 0;
                            break 'f ControlUI::Unchanged;
                        }
                        self.armed = 0;

                        break 'f ControlUI::Action(self.action[i]);
                    }
                }
                ControlUI::Continue
            }
            _ => ControlUI::Continue,
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

fn menu_span(txt: &str) -> (char, Vec<Span>) {
    let mut key = char::default();
    let mut menu = Vec::new();

    let mut it = txt.split('_');
    if let Some(p) = it.next() {
        if p.len() > 0 {
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
