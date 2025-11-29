//!
//! Date-input widget using [chrono](https://docs.rs/chrono/latest/chrono/)
//!

use crate::_private::NonExhaustive;
use crate::event::{ReadOnly, TextOutcome};
use crate::text_input_mask::{MaskedInput, MaskedInputState};
use crate::{
    TextFocusGained, TextFocusLost, TextStyle, TextTab, derive_text_widget,
    derive_text_widget_state,
};
use chrono::NaiveDate;
use chrono::format::{Fixed, Item, Numeric, Pad, StrftimeItems};
use rat_event::{HandleEvent, MouseOnly, Regular};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::StatefulWidget;
use std::fmt;
use unicode_segmentation::UnicodeSegmentation;

/// Widget for dates.
///
/// # Stateful
/// This widget implements [`StatefulWidget`], you can use it with
/// [`DateInputState`] to handle common actions.
#[derive(Debug, Default, Clone)]
pub struct DateInput<'a> {
    widget: MaskedInput<'a>,
}

/// State & event-handling.
/// Use [DateInputState::with_pattern] to set the date pattern.
#[derive(Debug, Clone)]
pub struct DateInputState {
    /// Area of the widget
    /// __read only__ renewed with each render.
    pub area: Rect,
    /// Area inside the block.
    /// __read only__ renewed with each render.
    pub inner: Rect,
    /// Uses MaskedInputState for the actual functionality.
    pub widget: MaskedInputState,
    /// The chrono format pattern.
    pattern: String,
    /// Locale
    locale: chrono::Locale,

    pub non_exhaustive: NonExhaustive,
}

derive_text_widget!(DateInput<'a>);

impl<'a> DateInput<'a> {
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
    pub fn width(&self, state: &DateInputState) -> u16 {
        state.widget.mask.len() as u16 + 1
    }

    /// Preferred width.
    pub fn height(&self) -> u16 {
        1
    }
}

impl<'a> StatefulWidget for &DateInput<'a> {
    type State = DateInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        (&self.widget).render(area, buf, &mut state.widget);

        state.area = state.widget.area;
        state.inner = state.widget.inner;
    }
}

impl StatefulWidget for DateInput<'_> {
    type State = DateInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget.render(area, buf, &mut state.widget);

        state.area = state.widget.area;
        state.inner = state.widget.inner;
    }
}

derive_text_widget_state!(DateInputState);

impl Default for DateInputState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            inner: Default::default(),
            widget: Default::default(),
            pattern: Default::default(),
            locale: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl DateInputState {
    /// New state.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn named(name: &str) -> Self {
        let mut z = Self::default();
        z.widget.focus = z.widget.focus.with_name(name);
        z
    }

    /// New state with a chrono date pattern.
    pub fn with_pattern<S: AsRef<str>>(mut self, pattern: S) -> Result<Self, fmt::Error> {
        self.set_format(pattern)?;
        Ok(self)
    }

    /// New state with a localized chrono date pattern.
    #[inline]
    pub fn with_loc_pattern<S: AsRef<str>>(
        mut self,
        pattern: S,
        locale: chrono::Locale,
    ) -> Result<Self, fmt::Error> {
        self.set_format_loc(pattern, locale)?;
        Ok(self)
    }

    /// chrono format string.
    #[inline]
    pub fn format(&self) -> &str {
        self.pattern.as_str()
    }

    /// chrono locale.
    #[inline]
    pub fn locale(&self) -> chrono::Locale {
        self.locale
    }

    /// chrono format string.
    ///
    /// generates a mask according to the format and overwrites whatever
    /// set_mask() did.
    #[inline]
    pub fn set_format<S: AsRef<str>>(&mut self, pattern: S) -> Result<(), fmt::Error> {
        self.set_format_loc(pattern, chrono::Locale::default())
    }

    /// chrono format string.
    ///
    /// generates a mask according to the format and overwrites whatever
    /// set_mask() did.
    #[inline]
    pub fn set_format_loc<S: AsRef<str>>(
        &mut self,
        pattern: S,
        locale: chrono::Locale,
    ) -> Result<(), fmt::Error> {
        let mut mask = String::new();
        let items = StrftimeItems::new_with_locale(pattern.as_ref(), locale)
            .parse()
            .map_err(|_| fmt::Error)?;
        for t in &items {
            match t {
                Item::Literal(s) => {
                    for c in s.graphemes(true) {
                        mask.push('\\');
                        mask.push_str(c);
                    }
                }
                Item::OwnedLiteral(s) => {
                    for c in s.graphemes(true) {
                        mask.push('\\');
                        mask.push_str(c);
                    }
                }
                Item::Space(s) => {
                    for c in s.graphemes(true) {
                        mask.push_str(c);
                    }
                }
                Item::OwnedSpace(s) => {
                    for c in s.graphemes(true) {
                        mask.push_str(c);
                    }
                }
                Item::Numeric(v, Pad::None | Pad::Space) => match v {
                    Numeric::Year | Numeric::IsoYear => mask.push_str("9999"),
                    Numeric::YearDiv100
                    | Numeric::YearMod100
                    | Numeric::IsoYearDiv100
                    | Numeric::IsoYearMod100
                    | Numeric::Month
                    | Numeric::Day
                    | Numeric::WeekFromSun
                    | Numeric::WeekFromMon
                    | Numeric::IsoWeek
                    | Numeric::Hour
                    | Numeric::Hour12
                    | Numeric::Minute
                    | Numeric::Second => mask.push_str("99"),
                    Numeric::NumDaysFromSun | Numeric::WeekdayFromMon => mask.push('9'),
                    Numeric::Ordinal => mask.push_str("999"),
                    Numeric::Nanosecond => mask.push_str("999999999"),
                    Numeric::Timestamp => mask.push_str("###########"),
                    _ => return Err(fmt::Error),
                },
                Item::Numeric(v, Pad::Zero) => match v {
                    Numeric::Year | Numeric::IsoYear => mask.push_str("0000"),
                    Numeric::YearDiv100
                    | Numeric::YearMod100
                    | Numeric::IsoYearDiv100
                    | Numeric::IsoYearMod100
                    | Numeric::Month
                    | Numeric::Day
                    | Numeric::WeekFromSun
                    | Numeric::WeekFromMon
                    | Numeric::IsoWeek
                    | Numeric::Hour
                    | Numeric::Hour12
                    | Numeric::Minute
                    | Numeric::Second => mask.push_str("00"),
                    Numeric::NumDaysFromSun | Numeric::WeekdayFromMon => mask.push('0'),
                    Numeric::Ordinal => mask.push_str("000"),
                    Numeric::Nanosecond => mask.push_str("000000000"),
                    Numeric::Timestamp => mask.push_str("#0000000000"),
                    _ => return Err(fmt::Error),
                },
                Item::Fixed(v) => match v {
                    Fixed::ShortMonthName => mask.push_str("___"),
                    Fixed::LongMonthName => mask.push_str("_________"),
                    Fixed::ShortWeekdayName => mask.push_str("___"),
                    Fixed::LongWeekdayName => mask.push_str("________"),
                    Fixed::LowerAmPm => mask.push_str("__"),
                    Fixed::UpperAmPm => mask.push_str("__"),
                    Fixed::Nanosecond => mask.push_str(".#########"),
                    Fixed::Nanosecond3 => mask.push_str(".###"),
                    Fixed::Nanosecond6 => mask.push_str(".######"),
                    Fixed::Nanosecond9 => mask.push_str(".#########"),
                    Fixed::TimezoneName => mask.push_str("__________"),
                    Fixed::TimezoneOffsetColon | Fixed::TimezoneOffset => mask.push_str("+##:##"),
                    Fixed::TimezoneOffsetDoubleColon => mask.push_str("+##:##:##"),
                    Fixed::TimezoneOffsetTripleColon => mask.push_str("+##"),
                    Fixed::TimezoneOffsetColonZ | Fixed::TimezoneOffsetZ => return Err(fmt::Error),
                    Fixed::RFC2822 => {
                        // 01 Jun 2016 14:31:46 -0700
                        return Err(fmt::Error);
                    }
                    Fixed::RFC3339 => {
                        // not supported, for now
                        return Err(fmt::Error);
                    }
                    _ => return Err(fmt::Error),
                },
                Item::Error => return Err(fmt::Error),
            }
        }

        self.locale = locale;
        self.pattern = pattern.as_ref().to_string();
        self.widget.set_mask(mask)?;
        Ok(())
    }
}

impl DateInputState {
    /// Reset to empty.
    #[inline]
    pub fn clear(&mut self) {
        self.widget.clear();
    }

    /// Set the date value.
    #[inline]
    pub fn set_value(&mut self, date: NaiveDate) {
        let v = date.format(self.pattern.as_str()).to_string();
        self.widget.set_text(v);
    }

    /// Parses the text according to the given pattern.
    #[inline]
    pub fn value(&self) -> Result<NaiveDate, chrono::ParseError> {
        NaiveDate::parse_from_str(self.widget.text(), self.pattern.as_str())
    }
}

impl HandleEvent<crossterm::event::Event, Regular, TextOutcome> for DateInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: Regular) -> TextOutcome {
        self.widget.handle(event, Regular)
    }
}

impl HandleEvent<crossterm::event::Event, ReadOnly, TextOutcome> for DateInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: ReadOnly) -> TextOutcome {
        self.widget.handle(event, ReadOnly)
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, TextOutcome> for DateInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> TextOutcome {
        self.widget.handle(event, MouseOnly)
    }
}

/// Handle all events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events(
    state: &mut DateInputState,
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
    state: &mut DateInputState,
    focus: bool,
    event: &crossterm::event::Event,
) -> TextOutcome {
    state.widget.focus.set(focus);
    state.handle(event, ReadOnly)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(
    state: &mut DateInputState,
    event: &crossterm::event::Event,
) -> TextOutcome {
    HandleEvent::handle(state, event, MouseOnly)
}
