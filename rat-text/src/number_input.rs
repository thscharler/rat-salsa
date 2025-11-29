//!
//! Number input widget
//!
use crate::_private::NonExhaustive;
use crate::event::{ReadOnly, TextOutcome};
use crate::text_input_mask::{MaskedInput, MaskedInputState};
use crate::{
    TextFocusGained, TextFocusLost, TextStyle, TextTab, derive_text_widget,
    derive_text_widget_state,
};
use format_num_pattern::{NumberFmtError, NumberFormat, NumberSymbols};
use rat_event::{HandleEvent, MouseOnly, Regular};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::StatefulWidget;
use std::fmt::{Debug, Display, LowerExp};
use std::str::FromStr;

/// NumberInput with [format_num_pattern][refFormatNumPattern] backend. A bit
/// similar to javas DecimalFormat.
///
/// # Stateful
/// This widget implements [`StatefulWidget`], you can use it with
/// [`NumberInputState`] to handle common actions.
///
/// [refFormatNumPattern]: https://docs.rs/format_num_pattern
#[derive(Debug, Default, Clone)]
pub struct NumberInput<'a> {
    widget: MaskedInput<'a>,
}

/// State & event handling.
#[derive(Debug, Clone)]
pub struct NumberInputState {
    /// Area of the widget.
    /// __read only__ renewed with each render.
    pub area: Rect,
    /// Area inside the block.
    /// __read only__ renewed with each render.
    pub inner: Rect,

    pub widget: MaskedInputState,

    /// NumberFormat pattern.
    pattern: String,
    /// Locale
    locale: format_num_pattern::Locale,
    // MaskedInput internally always works with the POSIX locale.
    // So don't be surprised, if you see that one instead of the
    // paramter locale used here.
    format: NumberFormat,

    pub non_exhaustive: NonExhaustive,
}

derive_text_widget!(NumberInput<'a>);

impl<'a> NumberInput<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the compact form, if the focus is not with this widget.
    #[inline]
    pub fn compact(mut self, compact: bool) -> Self {
        self.widget = self.widget.compact(compact);
        self
    }

    /// Preferred width.
    pub fn width(&self, state: &NumberInputState) -> u16 {
        state.widget.mask.len() as u16 + 1
    }

    /// Preferred width.
    pub fn height(&self) -> u16 {
        1
    }
}

impl<'a> StatefulWidget for &NumberInput<'a> {
    type State = NumberInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        (&self.widget).render(area, buf, &mut state.widget);

        state.area = state.widget.area;
        state.inner = state.widget.inner;
    }
}

impl StatefulWidget for NumberInput<'_> {
    type State = NumberInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget.render(area, buf, &mut state.widget);

        state.area = state.widget.area;
        state.inner = state.widget.inner;
    }
}

derive_text_widget_state!(NumberInputState);

impl Default for NumberInputState {
    fn default() -> Self {
        let mut s = Self {
            area: Default::default(),
            inner: Default::default(),
            widget: Default::default(),
            pattern: "".to_string(),
            locale: Default::default(),
            format: Default::default(),
            non_exhaustive: NonExhaustive,
        };
        _ = s.set_format("#####");
        s
    }
}

impl NumberInputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_pattern<S: AsRef<str>>(pattern: S) -> Result<Self, NumberFmtError> {
        let mut s = Self::default();
        s.set_format(pattern)?;
        Ok(s)
    }

    pub fn new_loc_pattern<S: AsRef<str>>(
        pattern: S,
        locale: format_num_pattern::Locale,
    ) -> Result<Self, NumberFmtError> {
        let mut s = Self::default();
        s.set_format_loc(pattern.as_ref(), locale)?;
        Ok(s)
    }

    pub fn named(name: &str) -> Self {
        let mut z = Self::default();
        z.widget.focus = z.widget.focus.with_name(name);
        z
    }

    pub fn with_pattern<S: AsRef<str>>(mut self, pattern: S) -> Result<Self, NumberFmtError> {
        self.set_format(pattern)?;
        Ok(self)
    }

    pub fn with_loc_pattern<S: AsRef<str>>(
        mut self,
        pattern: S,
        locale: format_num_pattern::Locale,
    ) -> Result<Self, NumberFmtError> {
        self.set_format_loc(pattern.as_ref(), locale)?;
        Ok(self)
    }

    /// [format_num_pattern] format string.
    #[inline]
    pub fn format(&self) -> &str {
        self.pattern.as_str()
    }

    /// chrono locale.
    #[inline]
    pub fn locale(&self) -> chrono::Locale {
        self.locale
    }

    /// Set format.
    pub fn set_format<S: AsRef<str>>(&mut self, pattern: S) -> Result<(), NumberFmtError> {
        self.set_format_loc(pattern, format_num_pattern::Locale::default())
    }

    /// Set format and locale.
    pub fn set_format_loc<S: AsRef<str>>(
        &mut self,
        pattern: S,
        locale: format_num_pattern::Locale,
    ) -> Result<(), NumberFmtError> {
        let sym = NumberSymbols::monetary(locale);

        self.format = NumberFormat::new(pattern.as_ref())?;
        self.widget.set_mask(pattern.as_ref())?;
        self.widget.set_num_symbols(sym);

        Ok(())
    }
}

impl NumberInputState {
    /// Parses the text as the desired value type.
    /// If the text content is empty returns None.
    pub fn value_opt<T: FromStr>(&self) -> Result<Option<T>, NumberFmtError> {
        let s = self.widget.text();
        if s.trim().is_empty() {
            Ok(None)
        } else {
            self.format.parse(s).map(|v| Some(v))
        }
    }

    /// Parses the text as the desired value type.
    pub fn value<T: FromStr>(&self) -> Result<T, NumberFmtError> {
        let s = self.widget.text();
        self.format.parse(s)
    }
}

impl NumberInputState {
    /// Reset to empty.
    #[inline]
    pub fn clear(&mut self) {
        self.widget.clear();
    }

    /// Sets the numeric value.
    pub fn set_value<T: LowerExp + Display + Debug>(
        &mut self,
        number: T,
    ) -> Result<(), NumberFmtError> {
        let s = self.format.fmt(number)?;
        self.widget.set_text(s);
        Ok(())
    }
}

impl HandleEvent<crossterm::event::Event, Regular, TextOutcome> for NumberInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: Regular) -> TextOutcome {
        self.widget.handle(event, Regular)
    }
}

impl HandleEvent<crossterm::event::Event, ReadOnly, TextOutcome> for NumberInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: ReadOnly) -> TextOutcome {
        self.widget.handle(event, ReadOnly)
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, TextOutcome> for NumberInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> TextOutcome {
        self.widget.handle(event, MouseOnly)
    }
}

/// Handle all events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events(
    state: &mut NumberInputState,
    focus: bool,
    event: &crossterm::event::Event,
) -> TextOutcome {
    state.widget.focus.set(focus);
    HandleEvent::handle(state, event, Regular)
}

/// Handle only navigation events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_readonly_events(
    state: &mut NumberInputState,
    focus: bool,
    event: &crossterm::event::Event,
) -> TextOutcome {
    state.widget.focus.set(focus);
    state.handle(event, ReadOnly)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(
    state: &mut NumberInputState,
    event: &crossterm::event::Event,
) -> TextOutcome {
    HandleEvent::handle(state, event, MouseOnly)
}
