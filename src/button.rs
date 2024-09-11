//!
//! A nice Button widget.
//!

use crate::_private::NonExhaustive;
use crate::util::revert_style;
use rat_event::{ct_event, ConsumedEvent, HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::{FocusFlag, HasFocusFlag};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::prelude::BlockExt;
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, StatefulWidget, Widget};
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::{StatefulWidgetRef, WidgetRef};

/// Button widget.
#[derive(Debug, Default, Clone)]
pub struct Button<'a> {
    text: Text<'a>,
    style: Style,
    focus_style: Option<Style>,
    armed_style: Option<Style>,
    block: Option<Block<'a>>,
}

/// Composite style.
#[derive(Debug, Clone, Copy)]
pub struct ButtonStyle {
    pub style: Style,
    pub focus: Option<Style>,
    pub armed: Option<Style>,
    pub non_exhaustive: NonExhaustive,
}

/// State & event-handling.
#[derive(Debug, Clone)]
pub struct ButtonState {
    /// Complete area
    /// __readonly__. renewed for each render.
    pub area: Rect,
    /// Area inside the block.
    /// __readonly__. renewed for each render.
    pub inner: Rect,

    /// Button has been clicked but not released yet.
    /// __used for mouse interaction__
    pub armed: bool,

    /// Current focus state.
    /// __read+write__
    pub focus: FocusFlag,

    pub non_exhaustive: NonExhaustive,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            focus: Default::default(),
            armed: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a> Button<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set all styles.
    #[inline]
    pub fn styles(mut self, styles: ButtonStyle) -> Self {
        self.style = styles.style;
        self.focus_style = styles.focus;
        self.armed_style = styles.armed;
        self
    }

    /// Set the base-style.
    #[inline]
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Style when focused.
    #[inline]
    pub fn focus_style(mut self, style: impl Into<Style>) -> Self {
        self.focus_style = Some(style.into());
        self
    }

    /// Style when clicked but not released.
    #[inline]
    pub fn armed_style(mut self, style: impl Into<Style>) -> Self {
        self.armed_style = Some(style.into());
        self
    }

    /// Button text.
    #[inline]
    pub fn text(mut self, text: impl Into<Text<'a>>) -> Self {
        self.text = text.into();
        self
    }

    /// Block.
    #[inline]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> From<&'a str> for Button<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            text: Text::from(value).centered(),
            ..Default::default()
        }
    }
}

impl<'a> From<String> for Button<'a> {
    fn from(value: String) -> Self {
        Self {
            text: Text::from(value).centered(),
            ..Default::default()
        }
    }
}

impl<'a> From<Span<'a>> for Button<'a> {
    fn from(value: Span<'a>) -> Self {
        Self {
            text: Text::from(value).centered(),
            ..Default::default()
        }
    }
}

impl<'a, const N: usize> From<[Span<'a>; N]> for Button<'a> {
    fn from(value: [Span<'a>; N]) -> Self {
        let mut text = Text::default();
        for value in value {
            text.push_span(value);
        }
        Self {
            text: text.centered(),
            ..Default::default()
        }
    }
}

impl<'a> From<Vec<Span<'a>>> for Button<'a> {
    fn from(value: Vec<Span<'a>>) -> Self {
        let mut text = Text::default();
        for value in value {
            text.push_span(value);
        }
        Self {
            text: text.centered(),
            ..Default::default()
        }
    }
}

impl<'a> From<Line<'a>> for Button<'a> {
    fn from(value: Line<'a>) -> Self {
        Self {
            text: Text::from(value).centered(),
            ..Default::default()
        }
    }
}

#[cfg(feature = "unstable-widget-ref")]
impl<'a> StatefulWidgetRef for Button<'a> {
    type State = ButtonState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.inner = self.block.inner_if_some(area);

        let focus_style = if let Some(focus_style) = self.focus_style {
            focus_style
        } else {
            revert_style(self.style)
        };
        let armed_style = if let Some(armed_style) = self.armed_style {
            armed_style
        } else {
            revert_style(self.style)
        };

        self.block.render_ref(area, buf);

        if state.armed {
            buf.set_style(state.inner, armed_style);
        } else {
            if state.focus.get() {
                buf.set_style(state.inner, focus_style);
            } else {
                buf.set_style(state.inner, self.style);
            }
        }

        let layout = Layout::vertical([Constraint::Length(self.text.height() as u16)])
            .flex(Flex::Center)
            .split(state.inner);

        self.text.render_ref(layout[0], buf);
    }
}

impl<'a> StatefulWidget for Button<'a> {
    type State = ButtonState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.inner = self.block.inner_if_some(area);

        let focus_style = if let Some(focus_style) = self.focus_style {
            focus_style
        } else {
            revert_style(self.style)
        };
        let armed_style = if let Some(armed_style) = self.armed_style {
            armed_style
        } else {
            revert_style(self.style)
        };

        self.block.render(area, buf);

        if state.armed {
            buf.set_style(state.inner, armed_style);
        } else {
            if state.focus.get() {
                buf.set_style(state.inner, focus_style);
            } else {
                buf.set_style(state.inner, self.style);
            }
        }

        let layout = Layout::vertical([Constraint::Length(self.text.height() as u16)])
            .flex(Flex::Center)
            .split(state.inner);

        self.text.render(layout[0], buf);
    }
}

impl Default for ButtonState {
    fn default() -> Self {
        Self {
            focus: Default::default(),
            area: Default::default(),
            inner: Default::default(),
            armed: false,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl ButtonState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn named(name: &str) -> Self {
        Self {
            focus: FocusFlag::named(name),
            ..ButtonState::default()
        }
    }
}

impl HasFocusFlag for ButtonState {
    #[inline]
    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    #[inline]
    fn area(&self) -> Rect {
        self.area
    }
}

/// Result value for event-handling.
///
/// Adds `Pressed` to the general Outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonOutcome {
    /// The given event was not handled at all.
    Continue,
    /// The event was handled, no repaint necessary.
    Unchanged,
    /// The event was handled, repaint necessary.
    Changed,
    /// Button has been pressed.
    Pressed,
}

impl ConsumedEvent for ButtonOutcome {
    fn is_consumed(&self) -> bool {
        *self != ButtonOutcome::Continue
    }
}

impl From<ButtonOutcome> for Outcome {
    fn from(value: ButtonOutcome) -> Self {
        match value {
            ButtonOutcome::Continue => Outcome::Continue,
            ButtonOutcome::Unchanged => Outcome::Unchanged,
            ButtonOutcome::Changed => Outcome::Changed,
            ButtonOutcome::Pressed => Outcome::Changed,
        }
    }
}

impl HandleEvent<crossterm::event::Event, Regular, ButtonOutcome> for ButtonState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: Regular) -> ButtonOutcome {
        let r = if self.is_focused() {
            match event {
                ct_event!(keycode press Enter) => {
                    self.armed = true;
                    ButtonOutcome::Changed
                }
                ct_event!(keycode release Enter) => {
                    if self.armed {
                        self.armed = false;
                        ButtonOutcome::Pressed
                    } else {
                        // single key release happen more often than not.
                        ButtonOutcome::Unchanged
                    }
                }
                _ => ButtonOutcome::Continue,
            }
        } else {
            ButtonOutcome::Continue
        };

        if r == ButtonOutcome::Continue {
            HandleEvent::handle(self, event, MouseOnly)
        } else {
            r
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, ButtonOutcome> for ButtonState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> ButtonOutcome {
        match event {
            ct_event!(mouse down Left for column, row) => {
                if self.area.contains((*column, *row).into()) {
                    self.armed = true;
                    ButtonOutcome::Changed
                } else {
                    ButtonOutcome::Continue
                }
            }
            ct_event!(mouse up Left for column, row) => {
                if self.area.contains((*column, *row).into()) {
                    self.armed = false;
                    ButtonOutcome::Pressed
                } else {
                    if self.armed {
                        self.armed = false;
                        ButtonOutcome::Changed
                    } else {
                        ButtonOutcome::Continue
                    }
                }
            }
            _ => ButtonOutcome::Continue,
        }
    }
}

/// Handle all events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events(
    state: &mut ButtonState,
    focus: bool,
    event: &crossterm::event::Event,
) -> ButtonOutcome {
    state.focus.set(focus);
    HandleEvent::handle(state, event, Regular)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(
    state: &mut ButtonState,
    event: &crossterm::event::Event,
) -> ButtonOutcome {
    HandleEvent::handle(state, event, MouseOnly)
}
