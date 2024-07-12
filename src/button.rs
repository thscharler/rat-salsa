//!
//! A button widget.
//!

use crate::_private::NonExhaustive;
use crate::util::revert_style;
use rat_event::{ct_event, ConsumedEvent, HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::{FocusFlag, HasFocusFlag};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::BlockExt;
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, StatefulWidget, StatefulWidgetRef, WidgetRef};

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

/// State data & event-handling.
#[derive(Debug, Clone)]
pub struct ButtonState {
    /// Current focus state.
    pub focus: FocusFlag,
    /// Complete area
    pub area: Rect,
    /// Inner area.
    pub inner_area: Rect,

    /// Button has been clicked but not released yet.
    pub armed: bool,

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

impl<'a> StatefulWidgetRef for Button<'a> {
    type State = ButtonState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

impl<'a> StatefulWidget for Button<'a> {
    type State = ButtonState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

fn render_ref(widget: &Button<'_>, area: Rect, buf: &mut Buffer, state: &mut ButtonState) {
    state.area = area;
    state.inner_area = widget.block.inner_if_some(area);

    widget.block.render_ref(area, buf);

    let focus_style = if let Some(focus_style) = widget.focus_style {
        focus_style
    } else {
        revert_style(widget.style)
    };
    let armed_style = if let Some(armed_style) = widget.armed_style {
        armed_style
    } else {
        revert_style(widget.style)
    };

    if state.armed {
        buf.set_style(state.inner_area, armed_style);
    } else {
        if state.focus.get() {
            buf.set_style(state.inner_area, focus_style);
        } else {
            buf.set_style(state.inner_area, widget.style);
        }
    }

    let layout = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(widget.text.height() as u16),
        Constraint::Fill(1),
    ])
    .split(state.inner_area);

    widget.text.render_ref(layout[1], buf);
}

impl ButtonState {
    pub fn new() -> Self {
        Self {
            focus: Default::default(),
            area: Default::default(),
            inner_area: Default::default(),
            armed: false,
            non_exhaustive: NonExhaustive,
        }
    }

    /// Renders the widget in focused style.
    ///
    /// This flag is not used for event-handling.
    #[inline]
    pub fn set_focused(&mut self, focus: bool) {
        self.focus.set(focus);
    }

    /// Renders the widget in focused style.
    ///
    /// This flag is not used for event-handling.
    #[inline]
    pub fn is_focused(&mut self) -> bool {
        self.focus.get()
    }
}

impl HasFocusFlag for ButtonState {
    #[inline]
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    #[inline]
    fn area(&self) -> Rect {
        self.area
    }
}

impl Default for ButtonState {
    fn default() -> Self {
        Self {
            focus: Default::default(),
            area: Default::default(),
            inner_area: Default::default(),
            armed: false,
            non_exhaustive: NonExhaustive,
        }
    }
}

/// Result value for event-handling.
///
/// Adds `Pressed` to the general Outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonOutcome {
    /// The given event was not handled at all.
    NotUsed,
    /// The event was handled, no repaint necessary.
    Unchanged,
    /// The event was handled, repaint necessary.
    Changed,
    /// Button has been pressed.
    Pressed,
}

impl ConsumedEvent for ButtonOutcome {
    fn is_consumed(&self) -> bool {
        *self != ButtonOutcome::NotUsed
    }
}

impl From<ButtonOutcome> for Outcome {
    fn from(value: ButtonOutcome) -> Self {
        match value {
            ButtonOutcome::NotUsed => Outcome::NotUsed,
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
                _ => ButtonOutcome::NotUsed,
            }
        } else {
            ButtonOutcome::NotUsed
        };

        if r == ButtonOutcome::NotUsed {
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
                    ButtonOutcome::NotUsed
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
                        ButtonOutcome::NotUsed
                    }
                }
            }
            _ => ButtonOutcome::NotUsed,
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
