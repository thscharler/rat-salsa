//!
//! Checkbox widget.
//!
//! Can use a third optional/defaulted state too.
//!
//! ```rust ignore
//! use rat_widget::checkbox::{Checkbox, CheckboxState};
//! use ratatui::widgets::StatefulWidget;
//!
//! Checkbox::new()
//!     .text("Carrots 🥕")
//!     .default_settable()
//!     .styles(THEME.checkbox_style())
//!     .render(layout[1][1], frame.buffer_mut(), &mut state.c1);
//!
//! Checkbox::new()
//!     .text("Potatoes 🥔\nTomatoes 🍅")
//!     .default_settable()
//!     .styles(THEME.checkbox_style())
//!     .render(layout[1][2], frame.buffer_mut(), &mut state.c2);
//! ```
//!
use crate::_private::NonExhaustive;
use crate::checkbox::event::CheckOutcome;
use crate::util::{block_size, revert_style};
use rat_event::util::MouseFlags;
use rat_event::{HandleEvent, MouseOnly, Regular, ct_event};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_reloc::{RelocatableState, relocate_area};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::BlockExt;
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::text::Text;
use ratatui::widgets::Block;
use ratatui::widgets::{StatefulWidget, Widget};
use std::cmp::max;
use unicode_segmentation::UnicodeSegmentation;

/// Enum controling the behaviour of the Checkbox.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CheckboxCheck {
    SingleClick,
    #[default]
    DoubleClick,
}

/// Checkbox widget.
#[derive(Debug, Clone)]
pub struct Checkbox<'a> {
    text: Text<'a>,

    // Check state override.
    checked: Option<bool>,
    default: Option<bool>,

    true_str: Span<'a>,
    false_str: Span<'a>,

    behave_check: CheckboxCheck,

    style: Style,
    focus_style: Option<Style>,
    block: Option<Block<'a>>,
}

/// Composite style.
#[derive(Debug, Clone)]
pub struct CheckboxStyle {
    /// Base style.
    pub style: Style,
    /// Focused style
    pub focus: Option<Style>,
    /// Border
    pub block: Option<Block<'static>>,

    /// Display text for 'true'
    pub true_str: Option<Span<'static>>,
    /// Display text for 'false'
    pub false_str: Option<Span<'static>>,

    pub behave_check: Option<CheckboxCheck>,

    pub non_exhaustive: NonExhaustive,
}

/// State.
#[derive(Debug)]
pub struct CheckboxState {
    /// Complete area
    /// __read only__. renewed for each render.
    pub area: Rect,
    /// Area inside the block.
    /// __read only__. renewed for each render.
    pub inner: Rect,
    /// Area of the check mark.
    /// __read only__. renewed for each render.
    pub check_area: Rect,
    /// Area for the text.
    /// __read only__. renewed for each render.
    pub text_area: Rect,
    /// Behaviour for check.
    /// __read only__. renewed for each render.
    pub behave_check: CheckboxCheck,

    /// Checked state.
    /// __read+write__
    pub checked: bool,

    /// Default state.
    /// __read+write__ Maybe overriden by a default set for the widget.
    pub default: bool,

    /// Current focus state.
    /// __read+write__
    pub focus: FocusFlag,

    /// Mouse helper
    /// __read+write__
    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

pub(crate) mod event {
    use rat_event::{ConsumedEvent, Outcome};

    /// Result value for event-handling.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CheckOutcome {
        /// The given event was not handled at all.
        Continue,
        /// The event was handled, no repaint necessary.
        Unchanged,
        /// The event was handled, repaint necessary.
        Changed,
        /// Checkbox has been checked or unchecked.
        Value,
    }

    impl ConsumedEvent for CheckOutcome {
        fn is_consumed(&self) -> bool {
            *self != CheckOutcome::Continue
        }
    }

    impl From<CheckOutcome> for Outcome {
        fn from(value: CheckOutcome) -> Self {
            match value {
                CheckOutcome::Continue => Outcome::Continue,
                CheckOutcome::Unchanged => Outcome::Unchanged,
                CheckOutcome::Changed => Outcome::Changed,
                CheckOutcome::Value => Outcome::Changed,
            }
        }
    }
}

impl Default for CheckboxStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            focus: Default::default(),
            block: Default::default(),
            true_str: Default::default(),
            false_str: Default::default(),
            behave_check: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl Default for Checkbox<'_> {
    fn default() -> Self {
        Self {
            text: Default::default(),
            checked: Default::default(),
            default: Default::default(),
            true_str: Span::from("[\u{2713}]"),
            false_str: Span::from("[ ]"),
            behave_check: Default::default(),
            style: Default::default(),
            focus_style: Default::default(),
            block: Default::default(),
        }
    }
}

impl<'a> Checkbox<'a> {
    /// New.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set all styles.
    pub fn styles(mut self, styles: CheckboxStyle) -> Self {
        self.style = styles.style;
        if styles.focus.is_some() {
            self.focus_style = styles.focus;
        }
        if let Some(block) = styles.block {
            self.block = Some(block);
        }
        if let Some(true_str) = styles.true_str {
            self.true_str = true_str;
        }
        if let Some(false_str) = styles.false_str {
            self.false_str = false_str;
        }
        if let Some(check) = styles.behave_check {
            self.behave_check = check;
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

    /// Button text.
    #[inline]
    pub fn text(mut self, text: impl Into<Text<'a>>) -> Self {
        self.text = text.into();
        self
    }

    /// Checked state. If set overrides the value from the state.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    /// Default state. If set overrides the value from the state.
    pub fn default_(mut self, default: bool) -> Self {
        self.default = Some(default);
        self
    }

    /// Block.
    #[inline]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Text for true
    pub fn true_str(mut self, str: Span<'a>) -> Self {
        self.true_str = str;
        self
    }

    /// Text for false
    pub fn false_str(mut self, str: Span<'a>) -> Self {
        self.false_str = str;
        self
    }

    /// Sets the behaviour for selecting from the list.
    pub fn behave_check(mut self, check: CheckboxCheck) -> Self {
        self.behave_check = check;
        self
    }

    /// Length of the check
    fn check_len(&self) -> u16 {
        max(
            self.true_str.content.graphemes(true).count(),
            self.false_str.content.graphemes(true).count(),
        ) as u16
    }

    /// Inherent width.
    pub fn width(&self) -> u16 {
        let chk_len = self.check_len();
        let txt_len = self.text.width() as u16;

        chk_len + 1 + txt_len + block_size(&self.block).width
    }

    /// Inherent height.
    pub fn height(&self) -> u16 {
        self.text.height() as u16 + block_size(&self.block).height
    }
}

impl<'a> StatefulWidget for &Checkbox<'a> {
    type State = CheckboxState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

impl StatefulWidget for Checkbox<'_> {
    type State = CheckboxState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

fn render_ref(widget: &Checkbox<'_>, area: Rect, buf: &mut Buffer, state: &mut CheckboxState) {
    state.area = area;
    state.inner = widget.block.inner_if_some(area);
    state.behave_check = widget.behave_check;

    let chk_len = widget.check_len();
    state.check_area = Rect::new(state.inner.x, state.inner.y, chk_len, 1);
    state.text_area = Rect::new(
        state.inner.x + chk_len + 1,
        state.inner.y,
        state.inner.width.saturating_sub(chk_len + 1),
        state.inner.height,
    );

    if let Some(checked) = widget.checked {
        state.checked = checked;
    }
    if let Some(default) = widget.default {
        state.default = default;
    }

    let style = widget.style;
    let focus_style = if let Some(focus_style) = widget.focus_style {
        style.patch(focus_style)
    } else {
        revert_style(style)
    };

    if let Some(block) = &widget.block {
        block.render(area, buf);
        if state.focus.get() {
            buf.set_style(state.inner, focus_style);
        }
    } else {
        if state.focus.get() {
            buf.set_style(state.inner, focus_style);
        } else {
            buf.set_style(state.inner, widget.style);
        }
    }

    let cc = if state.checked {
        &widget.true_str
    } else {
        &widget.false_str
    };
    cc.render(state.check_area, buf);
    (&widget.text).render(state.text_area, buf);
}

impl Clone for CheckboxState {
    fn clone(&self) -> Self {
        Self {
            area: self.area,
            inner: self.inner,
            check_area: self.check_area,
            text_area: self.text_area,
            behave_check: self.behave_check,
            checked: self.checked,
            default: self.default,
            focus: FocusFlag::named(self.focus.name()),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl Default for CheckboxState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            inner: Default::default(),
            check_area: Default::default(),
            text_area: Default::default(),
            behave_check: Default::default(),
            checked: false,
            default: false,
            focus: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl HasFocus for CheckboxState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl RelocatableState for CheckboxState {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.area = relocate_area(self.area, shift, clip);
        self.inner = relocate_area(self.inner, shift, clip);
    }
}

impl CheckboxState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn named(name: &str) -> Self {
        Self {
            focus: FocusFlag::named(name),
            ..Default::default()
        }
    }

    /// Get the value.
    pub fn checked(&self) -> bool {
        self.checked
    }

    /// Set the value.
    pub fn set_checked(&mut self, checked: bool) -> bool {
        let old_value = self.checked;
        self.checked = checked;
        old_value != self.checked
    }

    /// Get the default value.
    pub fn default_(&self) -> bool {
        self.default
    }

    /// Set the default value.
    pub fn set_default(&mut self, default: bool) -> bool {
        let old_value = self.default;
        self.default = default;
        old_value != self.default
    }

    /// Get the checked value, disregarding of the default state.
    pub fn value(&self) -> bool {
        self.checked
    }

    /// Set checked value. Always sets default to false.
    pub fn set_value(&mut self, checked: bool) -> bool {
        let old_value = self.checked;
        self.checked = checked;
        old_value != self.checked
    }

    /// Flip the checkbox.
    /// If it was in default state it just switches off
    /// the default flag. Otherwise, it flips true/false.
    pub fn flip_checked(&mut self) {
        self.checked = !self.checked;
    }
}

impl HandleEvent<crossterm::event::Event, Regular, CheckOutcome> for CheckboxState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> CheckOutcome {
        let r = if self.is_focused() {
            match event {
                ct_event!(keycode press Enter) | ct_event!(key press ' ') => {
                    self.flip_checked();
                    CheckOutcome::Value
                }
                ct_event!(keycode press Backspace) | ct_event!(keycode press Delete) => {
                    self.set_value(self.default);
                    CheckOutcome::Value
                }
                _ => CheckOutcome::Continue,
            }
        } else {
            CheckOutcome::Continue
        };

        if r == CheckOutcome::Continue {
            HandleEvent::handle(self, event, MouseOnly)
        } else {
            r
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, CheckOutcome> for CheckboxState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> CheckOutcome {
        match event {
            ct_event!(mouse any for m)
                if self.behave_check == CheckboxCheck::DoubleClick
                    && self.mouse.doubleclick(self.area, m) =>
            {
                self.flip_checked();
                CheckOutcome::Value
            }
            ct_event!(mouse down Left for x,y)
                if self.behave_check == CheckboxCheck::SingleClick
                    && self.area.contains((*x, *y).into()) =>
            {
                self.flip_checked();
                CheckOutcome::Value
            }
            _ => CheckOutcome::Continue,
        }
    }
}

/// Handle all events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events(
    state: &mut CheckboxState,
    focus: bool,
    event: &crossterm::event::Event,
) -> CheckOutcome {
    state.focus.set(focus);
    HandleEvent::handle(state, event, Regular)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(
    state: &mut CheckboxState,
    event: &crossterm::event::Event,
) -> CheckOutcome {
    HandleEvent::handle(state, event, MouseOnly)
}
