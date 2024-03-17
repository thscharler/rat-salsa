//!
//! A simple button.
//!
use crate::lib_focus::HasArea;
use crate::FocusFlag;
use crate::{ControlUI, HasFocusFlag};
use crate::{DefaultKeys, HandleCrossterm, Input, MouseOnly};
use crossterm::event::Event;
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
        Self {
            text: Line::from(value),
            ..Default::default()
        }
    }
}

impl<'a, A: Default> From<String> for Button<'a, A> {
    fn from(value: String) -> Self {
        Self {
            text: Line::from(value),
            ..Default::default()
        }
    }
}

impl<'a, A: Default> From<Span<'a>> for Button<'a, A> {
    fn from(value: Span<'a>) -> Self {
        Self {
            text: Line::from(value),
            ..Default::default()
        }
    }
}

impl<'a, A: Default, const N: usize> From<[Span<'a>; N]> for Button<'a, A> {
    fn from(value: [Span<'a>; N]) -> Self {
        Self {
            text: Line::from(Vec::from(value)),
            ..Default::default()
        }
    }
}

impl<'a, A: Default> From<Vec<Span<'a>>> for Button<'a, A> {
    fn from(value: Vec<Span<'a>>) -> Self {
        Self {
            text: Line::from(value),
            ..Default::default()
        }
    }
}

impl<'a, A: Default> From<Line<'a>> for Button<'a, A> {
    fn from(value: Line<'a>) -> Self {
        Self {
            text: value,
            ..Default::default()
        }
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

/// Button actions.
#[derive(Debug)]
pub enum InputRequest {
    Arm,
    Action,
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

impl<A> HasFocusFlag for ButtonState<A> {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }
}

impl<A> HasArea for ButtonState<A> {
    fn area(&self) -> Rect {
        self.area
    }
}

impl<A: Clone, E> HandleCrossterm<ControlUI<A, E>, DefaultKeys> for ButtonState<A> {
    fn handle(&mut self, event: &Event, _: DefaultKeys) -> ControlUI<A, E> {
        use crossterm::event::*;

        let req = match event {
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => 'f: {
                if !self.focus.get() {
                    break 'f None;
                }
                Some(InputRequest::Arm)
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Release,
                ..
            }) => 'f: {
                if !self.focus.get() {
                    break 'f None;
                }
                Some(InputRequest::Action)
            }
            _ => return self.handle(event, MouseOnly),
        };

        if let Some(req) = req {
            self.perform(req)
        } else {
            ControlUI::Continue
        }
    }
}

impl<A: Clone, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for ButtonState<A> {
    fn handle(&mut self, event: &Event, _: MouseOnly) -> ControlUI<A, E> {
        use crossterm::event::*;

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
            ) => {
                if self.area.contains(Position::new(*column, *row)) {
                    Some(InputRequest::Arm)
                } else {
                    None
                }
            }
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left),
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => {
                if self.area.contains(Position::new(*column, *row)) {
                    Some(InputRequest::Action)
                } else {
                    None
                }
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

impl<A: Clone, E> Input<ControlUI<A, E>> for ButtonState<A> {
    type Request = InputRequest;

    fn perform(&mut self, action: Self::Request) -> ControlUI<A, E> {
        match action {
            InputRequest::Arm => {
                self.armed = true;
                ControlUI::Change
            }
            InputRequest::Action => {
                if self.armed {
                    self.armed = false;
                    ControlUI::Run(self.action.clone())
                } else {
                    ControlUI::Continue
                }
            }
        }
    }
}
