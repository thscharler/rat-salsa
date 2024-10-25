//!
//! Button widget.
//!

use crate::_private::NonExhaustive;
use crate::util::{block_size, revert_style};
use rat_event::{ct_event, ConsumedEvent, HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::{FocusFlag, HasFocus};
use rat_reloc::{relocate_area, RelocatableState};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::BlockExt;
use ratatui::style::Style;
use ratatui::text::Text;
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::StatefulWidgetRef;
use ratatui::widgets::{Block, StatefulWidget, Widget};

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
#[derive(Debug, Clone)]
pub struct ButtonStyle {
    pub style: Style,
    pub focus: Option<Style>,
    pub armed: Option<Style>,
    pub block: Option<Block<'static>>,
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
            focus: None,
            armed: None,
            block: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a> Button<'a> {
    pub fn new(text: impl Into<Text<'a>>) -> Self {
        Self::default().text(text)
    }

    /// Set all styles.
    #[inline]
    pub fn styles(mut self, styles: ButtonStyle) -> Self {
        self.style = styles.style;
        if styles.focus.is_some() {
            self.focus_style = styles.focus;
        }
        if styles.armed.is_some() {
            self.armed_style = styles.armed;
        }
        if let Some(block) = styles.block {
            self.block = Some(block);
        }
        self.block = self.block.map(|v| v.style(self.style));
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
        self.text = text.into().centered();
        self
    }

    /// Left align button text.
    pub fn left_aligned(mut self) -> Self {
        self.text = self.text.left_aligned();
        self
    }

    /// Right align button text.
    pub fn right_aligned(mut self) -> Self {
        self.text = self.text.right_aligned();
        self
    }

    /// Block.
    #[inline]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Inherent width.
    pub fn width(&self) -> u16 {
        self.text.width() as u16 + block_size(&self.block).width
    }

    /// Inherent height.
    pub fn height(&self) -> u16 {
        self.text.height() as u16 + block_size(&self.block).height
    }
}

#[cfg(feature = "unstable-widget-ref")]
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
    state.inner = widget.block.inner_if_some(area);

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

    widget.block.render(area, buf);

    if state.armed {
        buf.set_style(state.inner, armed_style);
    } else if state.focus.get() {
        buf.set_style(state.inner, focus_style);
    }

    let h = widget.text.height() as u16;
    let r = state.inner.height.saturating_sub(h) / 2;
    let area = Rect::new(state.inner.x, state.inner.y + r, state.inner.width, h);

    (&widget.text).render(area, buf);
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
            ..Default::default()
        }
    }
}

impl HasFocus for ButtonState {
    #[inline]
    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    #[inline]
    fn area(&self) -> Rect {
        self.area
    }
}

impl RelocatableState for ButtonState {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.area = relocate_area(self.area, shift, clip);
        self.inner = relocate_area(self.inner, shift, clip);
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
