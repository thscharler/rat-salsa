//!
//! Date-input using [chrono](https://docs.rs/chrono/latest/chrono/)
//!

use crate::_private::NonExhaustive;
use crate::event::{ReadOnly, TextOutcome};
use crate::input::TextInputStyle;
use crate::masked_input::{MaskedInput, MaskedInputState};
use chrono::format::{Fixed, Item, Numeric, Pad, StrftimeItems};
use chrono::{Datelike, Days, Local, Months, NaiveDate};
#[allow(unused_imports)]
use log::debug;
use rat_event::{ct_event, HandleEvent, MouseOnly, Regular};
use rat_focus::{FocusFlag, HasFocusFlag};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, StatefulWidget, StatefulWidgetRef};
use std::fmt;
use std::fmt::Debug;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

/// Widget for dates.
#[derive(Debug, Default, Clone)]
pub struct DateInput<'a> {
    widget: MaskedInput<'a>,
}

/// State & event-handling.
/// Use `DateInputState::new(_pattern_)` to set the date pattern.
#[derive(Debug, Clone)]
pub struct DateInputState {
    /// Uses MaskedInputState for the actual functionality.
    pub widget: MaskedInputState,
    /// The chrono format pattern.
    pattern: String,
    /// Locale
    locale: chrono::Locale,

    pub non_exhaustive: NonExhaustive,
}

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

    /// Set the combined style.
    #[inline]
    pub fn styles(mut self, style: TextInputStyle) -> Self {
        self.widget = self.widget.styles(style);
        self
    }

    /// Base text style.
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

    /// Style for selection
    #[inline]
    pub fn select_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.select_style(style);
        self
    }

    /// Style for the invalid indicator.
    #[inline]
    pub fn invalid_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.invalid_style(style);
        self
    }

    /// Block
    #[inline]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.widget = self.widget.block(block);
        self
    }
}

impl<'a> StatefulWidgetRef for DateInput<'a> {
    type State = DateInputState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget.render_ref(area, buf, &mut state.widget);
    }
}

impl<'a> StatefulWidget for DateInput<'a> {
    type State = DateInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget.render(area, buf, &mut state.widget);
    }
}

impl Default for DateInputState {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            pattern: Default::default(),
            locale: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl DateInputState {
    /// New state.
    // xxx
    pub fn new() -> Self {
        Self::default()
    }

    // xxx
    pub fn named(name: &str) -> Self {
        Self {
            widget: MaskedInputState::named(name),
            ..Default::default()
        }
    }

    /// New state with a chrono date pattern.
    // xxx
    pub fn with_pattern<S: AsRef<str>>(mut self, pattern: S) -> Result<Self, fmt::Error> {
        self.set_format(pattern)?;
        Ok(self)
    }

    /// New state with a localized chrono date pattern.
    // xxx
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
    // xxx
    #[inline]
    pub fn format(&self) -> &str {
        self.pattern.as_str()
    }

    /// chrono locale.
    // xxx
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
    // xxx
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

    /// Renders the widget in invalid style.
    // xxx
    #[inline]
    pub fn set_invalid(&mut self, invalid: bool) {
        self.widget.invalid = invalid;
    }

    /// Renders the widget in invalid style.
    // xxx
    #[inline]
    pub fn get_invalid(&self) -> bool {
        self.widget.invalid
    }

    /// Reset to empty.
    // xxx
    #[inline]
    pub fn clear(&mut self) {
        self.widget.clear();
    }

    /// Offset shown.
    // xxx
    #[inline]
    pub fn offset(&self) -> usize {
        self.widget.offset()
    }

    /// Offset shown. This is corrected if the cursor wouldn't be visible.
    // xxx
    #[inline]
    pub fn set_offset(&mut self, offset: usize) {
        self.widget.set_offset(offset)
    }

    /// Cursor position
    // xxx
    #[inline]
    pub fn cursor(&self) -> usize {
        self.widget.cursor()
    }

    /// Set the cursor position, reset selection.
    // xxx
    #[inline]
    pub fn set_cursor(&mut self, cursor: usize, extend_selection: bool) -> bool {
        self.widget.set_cursor(cursor, extend_selection)
    }

    /// Place cursor at decimal separator, if any. 0 otherwise.
    // xxx
    #[inline]
    pub fn set_default_cursor(&mut self) {
        self.widget.set_default_cursor()
    }

    /// Selection anchor.
    // xxx
    #[inline]
    pub fn anchor(&self) -> usize {
        self.widget.anchor()
    }

    /// Create a default value according to the mask.
    #[inline]
    pub fn default_value(&self) -> String {
        self.widget.default_value()
    }

    /// Parses the text according to the given pattern.
    // xxx
    #[inline]
    pub fn value(&self) -> Result<NaiveDate, chrono::ParseError> {
        NaiveDate::parse_from_str(self.widget.compact_value().as_str(), self.pattern.as_str())
    }

    /// Set the date value.
    // xxx
    #[inline]
    pub fn set_value(&mut self, date: NaiveDate) {
        let v = date.format(self.pattern.as_str()).to_string();
        self.widget.set_value(v);
    }

    /// Empty
    // xxx
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.widget.is_empty()
    }

    /// Length in grapheme count.
    // xxx
    #[inline]
    pub fn len(&self) -> usize {
        self.widget.len()
    }

    /// Selection
    // xxx
    #[inline]
    pub fn has_selection(&self) -> bool {
        self.widget.has_selection()
    }

    /// Selection
    // xxx
    #[inline]
    pub fn selection(&self) -> Range<usize> {
        self.widget.selection()
    }

    /// Selection
    // xxx
    #[inline]
    pub fn set_selection(&mut self, anchor: usize, cursor: usize) -> bool {
        self.widget.set_selection(anchor, cursor)
    }

    /// Select all text.
    // xxx
    #[inline]
    pub fn select_all(&mut self) {
        self.widget.select_all();
    }

    /// Selection
    // xxx
    #[inline]
    pub fn selected_value(&self) -> &str {
        self.widget.selected_value()
    }

    /// Insert a char at the current position.
    // xxx
    #[inline]
    pub fn insert_char(&mut self, c: char) -> bool {
        self.widget.insert_char(c)
    }

    /// Remove the selected range. The text will be replaced with the default value
    /// as defined by the mask.
    // xxx
    #[inline]
    pub fn delete_range(&mut self, range: Range<usize>) -> bool {
        self.widget.delete_range(range)
    }

    /// Deletes the next word.
    #[inline]
    pub fn delete_next_word(&mut self) -> bool {
        self.widget.delete_next_word()
    }

    /// Deletes the given range.
    #[inline]
    pub fn delete_prev_word(&mut self) -> bool {
        self.widget.delete_prev_word()
    }

    /// Delete the char before the cursor.
    // xxx
    #[inline]
    pub fn delete_prev_char(&mut self) -> bool {
        self.widget.delete_prev_char()
    }

    /// Delete the char after the cursor.
    // xxx
    #[inline]
    pub fn delete_next_char(&mut self) -> bool {
        self.widget.delete_next_char()
    }

    #[inline]
    pub fn move_to_next_word(&mut self, extend_selection: bool) -> bool {
        self.widget.move_to_next_word(extend_selection)
    }

    #[inline]
    pub fn move_to_prev_word(&mut self, extend_selection: bool) -> bool {
        self.widget.move_to_next_word(extend_selection)
    }

    /// Move to the next char.
    // xxx
    #[inline]
    pub fn move_to_next(&mut self, extend_selection: bool) -> bool {
        self.widget.move_to_next(extend_selection)
    }

    /// Move to the previous char.
    // xxx
    #[inline]
    pub fn move_to_prev(&mut self, extend_selection: bool) -> bool {
        self.widget.move_to_prev(extend_selection)
    }

    /// Start of line
    // xxx
    #[inline]
    pub fn move_to_line_start(&mut self, extend_selection: bool) -> bool {
        self.widget.move_to_line_start(extend_selection)
    }

    /// End of line
    // xxx
    #[inline]
    pub fn move_to_line_end(&mut self, extend_selection: bool) -> bool {
        self.widget.move_to_line_end(extend_selection)
    }

    /// Converts a grapheme based position to a screen position
    /// relative to the widget area.
    // xxx
    pub fn to_screen_col(&self, pos: usize) -> Option<u16> {
        self.widget.to_screen_col(pos)
    }

    /// Converts from a widget relative screen coordinate to a grapheme index.
    /// x is the relative screen position.
    // xxx
    pub fn from_screen_col(&self, x: i16) -> usize {
        self.widget.from_screen_col(x)
    }

    /// Set the cursor position from a screen position relative to the origin
    /// of the widget. This value can be negative, which selects a currently
    /// not visible position and scrolls to it.
    // xxx
    #[inline]
    pub fn set_screen_cursor(&mut self, cursor: i16, extend_selection: bool) -> bool {
        self.widget.set_screen_cursor(cursor, extend_selection)
    }

    /// Screen position of the cursor for rendering.
    // xxx
    #[inline]
    pub fn screen_cursor(&self) -> Option<(u16, u16)> {
        self.widget.screen_cursor()
    }
}

impl HasFocusFlag for DateInputState {
    #[inline]
    fn focus(&self) -> FocusFlag {
        self.widget.focus.clone()
    }

    #[inline]
    fn area(&self) -> Rect {
        self.widget.area
    }
}

/// Add convenience keys.
/// * `h` - today
/// * `a` - January, 1st
/// * `e` - December, 31st
/// * `l` - first of last month
/// * `L` - last of last month
/// * `m` - first of this month
/// * `M` - last of this month
/// * `n` - first of next month
/// * `N` - last of next month
/// * `j` - add month
/// * `k` - subtract month
/// * `J` - add year
/// * `K` - subtract year
///
/// Calls handle(FocusKeys) afterwards.
#[derive(Debug)]
pub struct ConvenientKeys;

impl HandleEvent<crossterm::event::Event, ConvenientKeys, TextOutcome> for DateInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: ConvenientKeys) -> TextOutcome {
        let r = if self.is_focused() {
            match event {
                ct_event!(key press 'h') => {
                    self.set_value(Local::now().date_naive());
                    TextOutcome::Changed
                }
                ct_event!(key press 'l') => {
                    let date = Local::now()
                        .date_naive()
                        .checked_sub_months(Months::new(1))
                        .expect("month")
                        .with_day(1)
                        .expect("day");
                    self.set_value(date);
                    TextOutcome::Changed
                }
                ct_event!(key press SHIFT-'L') => {
                    let date = Local::now()
                        .date_naive()
                        .with_day(1)
                        .expect("month")
                        .checked_sub_days(Days::new(1))
                        .expect("day");
                    self.set_value(date);
                    TextOutcome::Changed
                }

                ct_event!(key press 'm') => {
                    let date = Local::now().date_naive().with_day(1).expect("day");
                    self.set_value(date);
                    TextOutcome::Changed
                }
                ct_event!(key press SHIFT-'M') => {
                    let date = Local::now()
                        .date_naive()
                        .checked_add_months(Months::new(1))
                        .expect("month")
                        .with_day(1)
                        .expect("day")
                        .checked_sub_days(Days::new(1))
                        .expect("day");
                    self.set_value(date);
                    TextOutcome::Changed
                }

                ct_event!(key press 'n') => {
                    let date = Local::now()
                        .date_naive()
                        .checked_add_months(Months::new(1))
                        .expect("month")
                        .with_day(1)
                        .expect("day");
                    self.set_value(date);
                    TextOutcome::Changed
                }
                ct_event!(key press SHIFT-'N') => {
                    let date = Local::now()
                        .date_naive()
                        .checked_add_months(Months::new(2))
                        .expect("month")
                        .with_day(1)
                        .expect("day")
                        .checked_sub_days(Days::new(1))
                        .expect("day");
                    self.set_value(date);
                    TextOutcome::Changed
                }

                ct_event!(key press 'j') => {
                    if let Ok(date) = self.value() {
                        let date = date.checked_add_months(Months::new(1)).expect("month");
                        self.set_value(date);
                    }
                    TextOutcome::Changed
                }
                ct_event!(key press SHIFT-'J') => {
                    if let Ok(date) = self.value() {
                        let date = date.with_year(date.year() + 1).expect("year");
                        self.set_value(date);
                    }
                    TextOutcome::Changed
                }

                ct_event!(key press 'k') => {
                    if let Ok(date) = self.value() {
                        let date = date.checked_sub_months(Months::new(1)).expect("month");
                        self.set_value(date);
                    }
                    TextOutcome::Changed
                }
                ct_event!(key press SHIFT-'K') => {
                    if let Ok(date) = self.value() {
                        let date = date.with_year(date.year() - 1).expect("year");
                        self.set_value(date);
                    }
                    TextOutcome::Changed
                }

                ct_event!(key press 'a'|'b') => {
                    if let Ok(date) = self.value() {
                        let date = date.with_month(1).expect("month").with_day(1).expect("day");
                        self.set_value(date);
                    }
                    TextOutcome::Changed
                }
                ct_event!(key press 'e') => {
                    if let Ok(date) = self.value() {
                        let date = date
                            .with_month(12)
                            .expect("month")
                            .with_day(31)
                            .expect("day");
                        self.set_value(date);
                    }
                    TextOutcome::Changed
                }
                _ => TextOutcome::Continue,
            }
        } else {
            TextOutcome::Continue
        };

        if r == TextOutcome::Continue {
            self.handle(event, Regular)
        } else {
            r
        }
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
