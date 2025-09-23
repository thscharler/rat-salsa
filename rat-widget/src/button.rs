//!
//! Button widget.
//!
//! Render:
//! ```rust ignore
//! Button::new("Button")
//!      .styles(THEME.button_style()) //
//!      .render(b_area_1, frame.buffer_mut(), &mut state.button1);
//! ```
//!
//! Event handling:
//! ```rust ignore
//! match state.button1.handle(event, Regular) {
//!     ButtonOutcome::Pressed => {
//!         data.p1 += 1;
//!         Outcome::Changed
//!     }
//!     r => r.into(),
//! }
//! ```
//!

use crate::_private::NonExhaustive;
use crate::button::event::ButtonOutcome;
use crate::util::{block_size, revert_style};
use rat_event::util::{MouseFlags, have_keyboard_enhancement};
use rat_event::{ConsumedEvent, HandleEvent, MouseOnly, Regular, ct_event};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_reloc::{RelocatableState, relocate_area};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::BlockExt;
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{Block, StatefulWidget, Widget};
use std::thread;
use std::time::Duration;

/// Button widget.
#[derive(Debug, Default, Clone)]
pub struct Button<'a> {
    text: Text<'a>,
    style: Style,
    focus_style: Option<Style>,
    hover_style: Option<Style>,
    armed_style: Option<Style>,
    armed_delay: Option<Duration>,
    block: Option<Block<'a>>,
}

/// Composite style.
#[derive(Debug, Clone)]
pub struct ButtonStyle {
    /// Base style
    pub style: Style,
    /// Focused style
    pub focus: Option<Style>,
    /// Armed style
    pub armed: Option<Style>,
    /// Hover style
    pub hover: Option<Style>,
    /// Button border
    pub block: Option<Block<'static>>,
    /// Some terminals repaint too fast to see the click.
    /// This adds some delay when the button state goes from
    /// armed to clicked.
    pub armed_delay: Option<Duration>,

    pub non_exhaustive: NonExhaustive,
}

/// State & event-handling.
#[derive(Debug)]
pub struct ButtonState {
    /// Complete area
    /// __readonly__. renewed for each render.
    pub area: Rect,
    /// Area inside the block.
    /// __readonly__. renewed for each render.
    pub inner: Rect,

    /// Hover is enabled?
    pub hover_enabled: bool,
    /// Button has been clicked but not released yet.
    /// __used for mouse interaction__
    pub armed: bool,
    /// Some terminals repaint too fast to see the click.
    /// This adds some delay when the button state goes from
    /// armed to clicked.
    ///
    /// Default is 50ms.
    pub armed_delay: Option<Duration>,

    /// Current focus state.
    /// __read+write__
    pub focus: FocusFlag,

    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            focus: None,
            armed: None,
            hover: None,
            block: None,
            armed_delay: None,
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
    pub fn styles_opt(self, styles: Option<ButtonStyle>) -> Self {
        if let Some(styles) = styles {
            self.styles(styles)
        } else {
            self
        }
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
        if styles.armed_delay.is_some() {
            self.armed_delay = styles.armed_delay;
        }
        if styles.hover.is_some() {
            self.hover_style = styles.hover;
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

    /// Some terminals repaint too fast to see the click.
    /// This adds some delay when the button state goes from
    /// armed to clicked.
    pub fn armed_delay(mut self, delay: Duration) -> Self {
        self.armed_delay = Some(delay);
        self
    }

    /// Style for hover over the button.
    pub fn hover_style(mut self, style: impl Into<Style>) -> Self {
        self.hover_style = Some(style.into());
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

impl<'a> StatefulWidget for &Button<'a> {
    type State = ButtonState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

impl StatefulWidget for Button<'_> {
    type State = ButtonState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

fn render_ref(widget: &Button<'_>, area: Rect, buf: &mut Buffer, state: &mut ButtonState) {
    state.area = area;
    state.inner = widget.block.inner_if_some(area);
    state.armed_delay = widget.armed_delay;
    state.hover_enabled = widget.hover_style.is_some();

    let style = widget.style;
    let focus_style = if let Some(focus_style) = widget.focus_style {
        focus_style
    } else {
        revert_style(style)
    };
    let armed_style = if let Some(armed_style) = widget.armed_style {
        armed_style
    } else {
        if state.is_focused() {
            revert_style(focus_style)
        } else {
            revert_style(style)
        }
    };

    if let Some(block) = &widget.block {
        block.render(area, buf);
    } else {
        buf.set_style(area, style);
    }

    if state.mouse.hover.get() && widget.hover_style.is_some() {
        buf.set_style(state.inner, widget.hover_style.expect("style"))
    } else if state.is_focused() {
        buf.set_style(state.inner, focus_style);
    }

    if state.armed {
        let armed_area = Rect::new(
            state.inner.x + 1,
            state.inner.y,
            state.inner.width.saturating_sub(2),
            state.inner.height,
        );
        buf.set_style(armed_area, style.patch(armed_style));
    }

    let h = widget.text.height() as u16;
    let r = state.inner.height.saturating_sub(h) / 2;
    let area = Rect::new(state.inner.x, state.inner.y + r, state.inner.width, h);
    (&widget.text).render(area, buf);
}

impl Clone for ButtonState {
    fn clone(&self) -> Self {
        Self {
            area: self.area,
            inner: self.inner,
            hover_enabled: false,
            armed: self.armed,
            armed_delay: self.armed_delay,
            focus: FocusFlag::named(self.focus.name()),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl Default for ButtonState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            inner: Default::default(),
            hover_enabled: false,
            armed: false,
            armed_delay: None,
            focus: Default::default(),
            mouse: Default::default(),
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

    pub fn clear_areas(&mut self) {
        self.area = Rect::default();
        self.inner = Rect::default();
    }
}

impl HasFocus for ButtonState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

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

pub(crate) mod event {
    use rat_event::{ConsumedEvent, Outcome};

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
}

impl HandleEvent<crossterm::event::Event, Regular, ButtonOutcome> for ButtonState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: Regular) -> ButtonOutcome {
        let r = if self.is_focused() {
            // Release keys may not be available.
            if have_keyboard_enhancement() {
                match event {
                    ct_event!(keycode press Enter) | ct_event!(key press ' ') => {
                        self.armed = true;
                        ButtonOutcome::Changed
                    }
                    ct_event!(keycode release Enter) | ct_event!(key release ' ') => {
                        if self.armed {
                            if let Some(delay) = self.armed_delay {
                                thread::sleep(delay);
                            }
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
                match event {
                    ct_event!(keycode press Enter) | ct_event!(key press ' ') => {
                        ButtonOutcome::Pressed
                    }
                    _ => ButtonOutcome::Continue,
                }
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
                    if self.armed {
                        self.armed = false;
                        ButtonOutcome::Pressed
                    } else {
                        ButtonOutcome::Continue
                    }
                } else {
                    if self.armed {
                        self.armed = false;
                        ButtonOutcome::Changed
                    } else {
                        ButtonOutcome::Continue
                    }
                }
            }
            ct_event!(mouse any for m) if self.mouse.hover(self.area, m) => ButtonOutcome::Changed,
            _ => ButtonOutcome::Continue,
        }
    }
}

/// Check event-handling for this hot-key and do Regular key-events otherwise.
impl HandleEvent<crossterm::event::Event, crossterm::event::KeyEvent, ButtonOutcome>
    for ButtonState
{
    fn handle(
        &mut self,
        event: &crossterm::event::Event,
        hotkey: crossterm::event::KeyEvent,
    ) -> ButtonOutcome {
        use crossterm::event::Event;

        let r = match event {
            Event::Key(key) => {
                // Release keys may not be available.
                if have_keyboard_enhancement() {
                    if hotkey.code == key.code && hotkey.modifiers == key.modifiers {
                        if key.kind == crossterm::event::KeyEventKind::Press {
                            self.armed = true;
                            ButtonOutcome::Changed
                        } else if key.kind == crossterm::event::KeyEventKind::Release {
                            if self.armed {
                                if let Some(delay) = self.armed_delay {
                                    thread::sleep(delay);
                                }
                                self.armed = false;
                                ButtonOutcome::Pressed
                            } else {
                                // single key release happen more often than not.
                                ButtonOutcome::Unchanged
                            }
                        } else {
                            ButtonOutcome::Continue
                        }
                    } else {
                        ButtonOutcome::Continue
                    }
                } else {
                    if hotkey.code == key.code && hotkey.modifiers == key.modifiers {
                        if key.kind == crossterm::event::KeyEventKind::Press {
                            ButtonOutcome::Pressed
                        } else {
                            ButtonOutcome::Continue
                        }
                    } else {
                        ButtonOutcome::Continue
                    }
                }
            }
            _ => ButtonOutcome::Continue,
        };

        r.or_else(|| self.handle(event, Regular))
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
