//!
//! Button widget.
//!
use crate::_private::NonExhaustive;
use rat_focus::{FocusFlag, HasFocusFlag};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Span, StatefulWidget, Style, Text};
use ratatui::widgets::Block;

use crate::event::{FocusKeys, MouseOnly};
pub use rat_input::button::{ButtonOutcome, ButtonStyle};
use rat_input::event::HandleEvent;

/// Button widget.
#[derive(Debug, Default, Clone)]
pub struct Button<'a> {
    widget: rat_input::button::Button<'a>,
}

/// Button state.
#[derive(Debug, Clone)]
pub struct ButtonState {
    /// Button state
    pub widget: rat_input::button::ButtonState,
    /// Data for focus handling.
    pub focus: FocusFlag,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> Button<'a> {
    /// New button.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set all styles.
    #[inline]
    pub fn styles(mut self, styles: ButtonStyle) -> Self {
        self.widget = self.widget.styles(styles);
        self
    }

    /// Set style.
    #[inline]
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.style(style);
        self
    }

    /// Style when focused.
    #[inline]
    pub fn focus_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.focus_style(style);
        self
    }

    /// Style when the button is clicked but not yet released.
    #[inline]
    pub fn armed_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.armed_style(style);
        self
    }

    /// Text
    #[inline]
    pub fn text(mut self, text: impl Into<Text<'a>>) -> Self {
        self.widget = self.widget.text(text);
        self
    }

    /// Block
    #[inline]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.widget = self.widget.block(block);
        self
    }
}

impl<'a> From<&'a str> for Button<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            widget: value.into(),
        }
    }
}

impl<'a> From<String> for Button<'a> {
    fn from(value: String) -> Self {
        Self {
            widget: value.into(),
        }
    }
}

impl<'a> From<Span<'a>> for Button<'a> {
    fn from(value: Span<'a>) -> Self {
        Self {
            widget: value.into(),
        }
    }
}

impl<'a, const N: usize> From<[Span<'a>; N]> for Button<'a> {
    fn from(value: [Span<'a>; N]) -> Self {
        Self {
            widget: value.into(),
        }
    }
}

impl<'a> From<Vec<Span<'a>>> for Button<'a> {
    fn from(value: Vec<Span<'a>>) -> Self {
        Self {
            widget: value.into(),
        }
    }
}

impl<'a> From<Line<'a>> for Button<'a> {
    fn from(value: Line<'a>) -> Self {
        Self {
            widget: value.into(),
        }
    }
}

impl<'a> StatefulWidget for Button<'a> {
    type State = ButtonState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget
            .focused(state.is_focused())
            .render(area, buf, &mut state.widget)
    }
}

impl Default for ButtonState {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            focus: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl HasFocusFlag for ButtonState {
    #[inline]
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    #[inline]
    fn area(&self) -> Rect {
        self.widget.area
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, ButtonOutcome> for ButtonState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> ButtonOutcome {
        if self.is_focused() {
            self.widget.handle(event, FocusKeys)
        } else {
            self.widget.handle(event, MouseOnly)
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, ButtonOutcome> for ButtonState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> ButtonOutcome {
        self.widget.handle(event, MouseOnly)
    }
}
