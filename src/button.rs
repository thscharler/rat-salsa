use crate::focus::FocusFlag;
use crate::{ControlUI, HandleEvent};
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
#[allow(unused_imports)]
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::Widget;
use std::fmt::Debug;

/// Simple button.
#[derive(Debug)]
pub struct Button<'a, A> {
    pub text: Line<'a>,
    pub action: A,
    pub style: Style,
    pub focus_style: Style,
    pub armed_style: Style,
}

/// Composite style.
#[derive(Debug, Default)]
pub struct ButtonStyle {
    pub style: Style,
    pub focus: Style,
    pub armed: Style,
}

impl<'a, A: Default> Default for Button<'a, A> {
    fn default() -> Self {
        Self {
            text: Default::default(),
            action: Default::default(),
            style: Default::default(),
            focus_style: Default::default(),
            armed_style: Default::default(),
        }
    }
}

impl<'a, A: Default> Button<'a, A> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn action(mut self, action: A) -> Self {
        self.action = action;
        self
    }

    pub fn style(mut self, styles: ButtonStyle) -> Self {
        self.style = styles.style;
        self.armed_style = styles.armed;
        self.focus_style = styles.focus;
        self
    }

    pub fn base_style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    pub fn focus_style(mut self, style: impl Into<Style>) -> Self {
        self.focus_style = style.into();
        self
    }

    pub fn armed_style(mut self, style: impl Into<Style>) -> Self {
        self.armed_style = style.into();
        self
    }
}

impl<'a, A: Default> From<&'a str> for Button<'a, A> {
    fn from(value: &'a str) -> Self {
        let mut s = Self::default();
        s.text = Line::from(value);
        s
    }
}

impl<'a, A: Default> From<String> for Button<'a, A> {
    fn from(value: String) -> Self {
        let mut s = Self::default();
        s.text = Line::from(value);
        s
    }
}

impl<'a, A: Default> From<Span<'a>> for Button<'a, A> {
    fn from(value: Span<'a>) -> Self {
        let mut s = Self::default();
        s.text = Line::from(value);
        s
    }
}

impl<'a, A: Default, const N: usize> From<[Span<'a>; N]> for Button<'a, A> {
    fn from(value: [Span<'a>; N]) -> Self {
        let value = Vec::from(value);

        let mut s = Self::default();
        s.text = Line::from(value);
        s
    }
}

impl<'a, A: Default> From<Vec<Span<'a>>> for Button<'a, A> {
    fn from(value: Vec<Span<'a>>) -> Self {
        let mut s = Self::default();
        s.text = Line::from(value);
        s
    }
}

impl<'a, A: Default> From<Line<'a>> for Button<'a, A> {
    fn from(value: Line<'a>) -> Self {
        let mut s = Self::default();
        s.text = value;
        s
    }
}

/// Button state.
#[derive(Debug, Default)]
pub struct ButtonState<A> {
    pub focus: FocusFlag,
    pub area: Rect,
    pub armed: bool,
    pub action: A,
}

impl<'a, A> StatefulWidget for Button<'a, A> {
    type State = ButtonState<A>;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.text = self.text.patch_style(self.style);
        if state.focus.get() {
            self.text = self.text.patch_style(self.focus_style);
        }
        if state.armed {
            self.text = self.text.patch_style(self.armed_style);
        }
        state.area = area;
        state.action = self.action;

        self.text.render(area, buf);
    }
}

impl<A: Clone, E> HandleEvent<A, E> for ButtonState<A> {
    fn handle(&mut self, event: &Event) -> ControlUI<A, E> {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if self.focus.get() {
                    self.armed = true;
                    ControlUI::Changed
                } else {
                    ControlUI::Continue
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Release,
                ..
            }) => {
                if self.focus.get() {
                    if self.armed {
                        self.armed = false;
                        ControlUI::Action(self.action.clone())
                    } else {
                        ControlUI::Continue
                    }
                } else {
                    ControlUI::Continue
                }
            }

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
            ) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.armed = true;
                    ControlUI::Changed
                } else {
                    ControlUI::Continue
                }
            }
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left),
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.armed = false;
                    if let Some(action) = self.action {
                        ControlUI::Action(action)
                    } else {
                        panic!("no action")
                    }
                } else {
                    ControlUI::Continue
                }
            }

            _ => ControlUI::Continue,
        }
    }
}
