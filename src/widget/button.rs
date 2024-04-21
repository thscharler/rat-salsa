//!
//! A simple button.
//!
use crate::{ct_event, FocusFlag};
use crate::{ControlUI, HasFocusFlag};
use crate::{DefaultKeys, HandleCrossterm, MouseOnly};
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

impl<A> ButtonState<A> {
    //
    pub fn action(&mut self) -> Option<A>
    where
        A: Clone,
    {
        if self.armed {
            self.armed = false;
            Some(self.action.clone())
        } else {
            None
        }
    }
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

    fn area(&self) -> Rect {
        self.area
    }
}

impl<A: Clone, E> HandleCrossterm<ControlUI<A, E>, DefaultKeys> for ButtonState<A> {
    fn handle(&mut self, event: &Event, _: DefaultKeys) -> ControlUI<A, E> {
        let res = if self.is_focused() {
            match event {
                ct_event!(keycode press Enter) => {
                    self.armed = true;
                    ControlUI::Change
                }
                ct_event!(keycode release Enter) => match self.action() {
                    Some(a) => ControlUI::Run(a),
                    None => ControlUI::NoChange,
                },
                _ => ControlUI::Continue,
            }
        } else {
            ControlUI::Continue
        };

        res.or_else(|| {
            <Self as HandleCrossterm<ControlUI<A, E>, MouseOnly>>::handle(self, event, MouseOnly)
        })
    }
}

impl<A: Clone, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for ButtonState<A> {
    fn handle(&mut self, event: &Event, _: MouseOnly) -> ControlUI<A, E> {
        let res = match event {
            ct_event!(mouse down Left for column, row)
            | ct_event!(mouse drag Left for column, row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.armed = true;
                    ControlUI::Change
                } else {
                    ControlUI::Continue
                }
            }
            ct_event!(mouse up Left for column, row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    match self.action() {
                        Some(a) => ControlUI::Run(a),
                        None => ControlUI::NoChange,
                    }
                } else {
                    ControlUI::Continue
                }
            }
            _ => ControlUI::Continue,
        };

        res
    }
}
