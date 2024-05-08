//!
//! A simple button.
//!
use crate::FocusFlag;
use crate::_private::NonExhaustive;
use crate::{ControlUI, HasFocusFlag};
use crate::{DefaultKeys, HandleCrossterm, MouseOnly};
use crossterm::event::Event;
use rat_input::button::{Button, ButtonOutcome, ButtonState, ButtonStyle};
use rat_input::events::HandleEvent;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::StatefulWidget;
use std::fmt::Debug;

/// Simple button.
#[derive(Debug)]
pub struct ButtonExt<'a> {
    widget: Button<'a>,
}

impl<'a> Default for ButtonExt<'a> {
    fn default() -> Self {
        Self {
            widget: Default::default(),
        }
    }
}

impl<'a> ButtonExt<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn styles(mut self, styles: ButtonStyle) -> Self {
        self.widget = self.widget.styles(styles);
        self
    }

    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.style(style);
        self
    }

    pub fn focus_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.focus_style(style);
        self
    }

    pub fn armed_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.armed_style(style);
        self
    }
}

impl<'a> From<&'a str> for ButtonExt<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            widget: Button::from(value),
        }
    }
}

impl<'a> From<String> for ButtonExt<'a> {
    fn from(value: String) -> Self {
        Self {
            widget: Button::from(value),
        }
    }
}

impl<'a> From<Span<'a>> for ButtonExt<'a> {
    fn from(value: Span<'a>) -> Self {
        Self {
            widget: Button::from(value),
        }
    }
}

impl<'a, const N: usize> From<[Span<'a>; N]> for ButtonExt<'a> {
    fn from(value: [Span<'a>; N]) -> Self {
        Self {
            widget: Button::from(value),
        }
    }
}

impl<'a> From<Vec<Span<'a>>> for ButtonExt<'a> {
    fn from(value: Vec<Span<'a>>) -> Self {
        Self {
            widget: Button::from(value),
        }
    }
}

impl<'a> From<Line<'a>> for ButtonExt<'a> {
    fn from(value: Line<'a>) -> Self {
        Self {
            widget: Button::from(value),
        }
    }
}

/// Button state.
#[derive(Debug)]
pub struct ButtonExtState {
    pub widget: ButtonState,
    pub focus: FocusFlag,

    pub non_exhaustive: NonExhaustive,
}

impl Default for ButtonExtState {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            focus: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl ButtonExtState {}

impl<'a> StatefulWidget for ButtonExt<'a> {
    type State = ButtonExtState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget.render(area, buf, &mut state.widget)
    }
}

impl HasFocusFlag for ButtonExtState {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.widget.area
    }
}

impl<E> HandleCrossterm<ControlUI<bool, E>, DefaultKeys> for ButtonExtState {
    fn handle(&mut self, event: &Event, _: DefaultKeys) -> ControlUI<bool, E> {
        if self.is_focused() {
            match self.widget.handle(event, rat_input::events::FocusKeys) {
                ButtonOutcome::NotUsed => ControlUI::Continue,
                ButtonOutcome::Unchanged => ControlUI::NoChange,
                ButtonOutcome::Changed => ControlUI::Change,
                ButtonOutcome::Pressed => ControlUI::Run(true),
            }
        } else {
            match self.widget.handle(event, rat_input::events::MouseOnly) {
                ButtonOutcome::NotUsed => ControlUI::Continue,
                ButtonOutcome::Unchanged => ControlUI::NoChange,
                ButtonOutcome::Changed => ControlUI::Change,
                ButtonOutcome::Pressed => ControlUI::Run(true),
            }
        }
    }
}

impl<E> HandleCrossterm<ControlUI<bool, E>, MouseOnly> for ButtonExtState {
    fn handle(&mut self, event: &Event, _: MouseOnly) -> ControlUI<bool, E> {
        match self.widget.handle(event, rat_input::events::MouseOnly) {
            ButtonOutcome::NotUsed => ControlUI::Continue,
            ButtonOutcome::Unchanged => ControlUI::NoChange,
            ButtonOutcome::Changed => ControlUI::Change,
            ButtonOutcome::Pressed => ControlUI::Run(true),
        }
    }
}
