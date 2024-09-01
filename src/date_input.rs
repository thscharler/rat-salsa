//!
//! Date-input using [chrono](https://docs.rs/chrono/latest/chrono/)
//!

use crate::_private::NonExhaustive;
use crate::clipboard::Clipboard;
use crate::event::{ReadOnly, TextOutcome};
use crate::text_input::TextInputStyle;
use crate::text_input_mask::{MaskedInput, MaskedInputState};
use crate::undo_buffer::{UndoBuffer, UndoEntry};
use crate::{upos_type, HasScreenCursor, TextError};
use chrono::format::{Fixed, Item, Numeric, Pad, StrftimeItems};
use chrono::NaiveDate;
use rat_event::{HandleEvent, MouseOnly, Regular};
use rat_focus::{FocusFlag, HasFocusFlag, Navigation};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{StatefulWidget, Style};
use ratatui::widgets::Block;
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::StatefulWidgetRef;
use std::fmt;
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

#[cfg(feature = "unstable-widget-ref")]
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

impl HasFocusFlag for DateInputState {
    #[inline]
    fn focus(&self) -> FocusFlag {
        self.widget.focus.clone()
    }

    #[inline]
    fn area(&self) -> Rect {
        self.widget.area
    }

    #[inline]
    fn navigable(&self) -> Navigation {
        self.widget.navigable()
    }
}

impl DateInputState {
    /// New state.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn named(name: &str) -> Self {
        Self {
            widget: MaskedInputState::named(name),
            ..Default::default()
        }
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

    /// Renders the widget in invalid style.
    #[inline]
    pub fn set_invalid(&mut self, invalid: bool) {
        self.widget.invalid = invalid;
    }

    /// Renders the widget in invalid style.
    #[inline]
    pub fn get_invalid(&self) -> bool {
        self.widget.invalid
    }
}

impl DateInputState {
    /// Clipboard
    #[inline]
    pub fn set_clipboard(&mut self, clip: Option<impl Clipboard + 'static>) {
        self.widget.set_clipboard(clip);
    }

    /// Clipboard
    #[inline]
    pub fn clipboard(&self) -> Option<&dyn Clipboard> {
        self.widget.clipboard()
    }

    /// Copy to internal buffer
    #[inline]
    pub fn copy_to_clip(&mut self) -> bool {
        self.widget.copy_to_clip()
    }

    /// Cut to internal buffer
    #[inline]
    pub fn cut_to_clip(&mut self) -> bool {
        self.widget.cut_to_clip()
    }

    /// Paste from internal buffer.
    #[inline]
    pub fn paste_from_clip(&mut self) -> bool {
        self.widget.paste_from_clip()
    }
}

impl DateInputState {
    /// Set undo buffer.
    #[inline]
    pub fn set_undo_buffer(&mut self, undo: Option<impl UndoBuffer + 'static>) {
        self.widget.set_undo_buffer(undo);
    }

    /// Undo
    #[inline]
    pub fn undo_buffer(&self) -> Option<&dyn UndoBuffer> {
        self.widget.undo_buffer()
    }

    /// Undo
    #[inline]
    pub fn undo_buffer_mut(&mut self) -> Option<&mut dyn UndoBuffer> {
        self.widget.undo_buffer_mut()
    }

    /// Get all recent replay recordings.
    #[inline]
    pub fn recent_replay_log(&mut self) -> Vec<UndoEntry> {
        self.widget.recent_replay_log()
    }

    /// Apply the replay recording.
    #[inline]
    pub fn replay_log(&mut self, replay: &[UndoEntry]) {
        self.widget.replay_log(replay)
    }

    /// Undo operation
    #[inline]
    pub fn undo(&mut self) -> bool {
        self.widget.undo()
    }

    /// Redo operation
    #[inline]
    pub fn redo(&mut self) -> bool {
        self.widget.redo()
    }
}

impl DateInputState {
    /// Set and replace all styles.
    #[inline]
    pub fn set_styles(&mut self, styles: Vec<(Range<usize>, usize)>) {
        self.widget.set_styles(styles);
    }

    /// Add a style for a [TextRange]. The style-nr refers to one
    /// of the styles set with the widget.
    #[inline]
    pub fn add_style(&mut self, range: Range<usize>, style: usize) {
        self.widget.add_style(range.into(), style);
    }

    /// Add a style for a Range<upos_type> to denote the cells.
    /// The style-nr refers to one of the styles set with the widget.
    #[inline]
    pub fn add_range_style(
        &mut self,
        range: Range<upos_type>,
        style: usize,
    ) -> Result<(), TextError> {
        self.widget.add_range_style(range, style)
    }

    /// Remove the exact TextRange and style.
    #[inline]
    pub fn remove_style(&mut self, range: Range<usize>, style: usize) {
        self.widget.remove_style(range.into(), style);
    }

    /// Remove the exact Range<upos_type> and style.
    #[inline]
    pub fn remove_range_style(
        &mut self,
        range: Range<upos_type>,
        style: usize,
    ) -> Result<(), TextError> {
        self.widget.remove_range_style(range, style)
    }

    /// All styles active at the given position.
    #[inline]
    pub fn styles_at(&self, byte_pos: usize, buf: &mut Vec<usize>) {
        self.widget.styles_at(byte_pos, buf)
    }

    /// Check if the given style applies at the position and
    /// return the complete range for the style.
    #[inline]
    pub fn style_match(&self, byte_pos: usize, style: usize) -> Option<Range<usize>> {
        self.widget.style_match(byte_pos, style.into())
    }

    /// List of all styles.
    #[inline]
    pub fn styles(&self) -> Option<impl Iterator<Item = (Range<usize>, usize)> + '_> {
        self.widget.styles()
    }
}

impl DateInputState {
    /// Offset shown.
    #[inline]
    pub fn offset(&self) -> upos_type {
        self.widget.offset()
    }

    /// Offset shown. This is corrected if the cursor wouldn't be visible.
    #[inline]
    pub fn set_offset(&mut self, offset: upos_type) {
        self.widget.set_offset(offset)
    }

    /// Cursor position
    #[inline]
    pub fn cursor(&self) -> upos_type {
        self.widget.cursor()
    }

    /// Set the cursor position, reset selection.
    #[inline]
    pub fn set_cursor(&mut self, cursor: upos_type, extend_selection: bool) -> bool {
        self.widget.set_cursor(cursor, extend_selection)
    }

    /// Place cursor at some sensible position according to the mask.
    #[inline]
    pub fn set_default_cursor(&mut self) {
        self.widget.set_default_cursor()
    }

    /// Selection anchor.
    #[inline]
    pub fn anchor(&self) -> upos_type {
        self.widget.anchor()
    }

    /// Selection
    #[inline]
    pub fn has_selection(&self) -> bool {
        self.widget.has_selection()
    }

    /// Selection
    #[inline]
    pub fn selection(&self) -> Range<upos_type> {
        self.widget.selection()
    }

    /// Selection
    #[inline]
    pub fn set_selection(&mut self, anchor: upos_type, cursor: upos_type) -> bool {
        self.widget.set_selection(anchor, cursor)
    }

    /// Select all text.
    #[inline]
    pub fn select_all(&mut self) {
        self.widget.select_all();
    }

    /// Selection
    #[inline]
    pub fn selected_text(&self) -> &str {
        self.widget.selected_text()
    }
}

impl DateInputState {
    /// Empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.widget.is_empty()
    }

    /// Parses the text according to the given pattern.
    #[inline]
    pub fn value(&self) -> Result<NaiveDate, chrono::ParseError> {
        NaiveDate::parse_from_str(self.widget.text(), self.pattern.as_str())
    }

    /// Length in grapheme count.
    #[inline]
    pub fn len(&self) -> upos_type {
        self.widget.len()
    }

    /// Length as grapheme count.
    #[inline]
    pub fn line_width(&self) -> upos_type {
        self.widget.line_width()
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

    /// Insert a char at the current position.
    #[inline]
    pub fn insert_char(&mut self, c: char) -> bool {
        self.widget.insert_char(c)
    }

    /// Remove the selected range. The text will be replaced with the default value
    /// as defined by the mask.
    #[inline]
    pub fn delete_range(&mut self, range: Range<upos_type>) -> bool {
        self.widget.delete_range(range)
    }

    /// Remove the selected range. The text will be replaced with the default value
    /// as defined by the mask.
    #[inline]
    pub fn try_delete_range(&mut self, range: Range<upos_type>) -> Result<bool, TextError> {
        self.widget.try_delete_range(range)
    }
}

impl DateInputState {
    /// Delete the char after the cursor.
    #[inline]
    pub fn delete_next_char(&mut self) -> bool {
        self.widget.delete_next_char()
    }

    /// Delete the char before the cursor.
    #[inline]
    pub fn delete_prev_char(&mut self) -> bool {
        self.widget.delete_prev_char()
    }

    /// Move to the next char.
    #[inline]
    pub fn move_right(&mut self, extend_selection: bool) -> bool {
        self.widget.move_right(extend_selection)
    }

    /// Move to the previous char.
    #[inline]
    pub fn move_left(&mut self, extend_selection: bool) -> bool {
        self.widget.move_left(extend_selection)
    }

    /// Start of line
    #[inline]
    pub fn move_to_line_start(&mut self, extend_selection: bool) -> bool {
        self.widget.move_to_line_start(extend_selection)
    }

    /// End of line
    #[inline]
    pub fn move_to_line_end(&mut self, extend_selection: bool) -> bool {
        self.widget.move_to_line_end(extend_selection)
    }
}

impl HasScreenCursor for DateInputState {
    /// The current text cursor as an absolute screen position.
    #[inline]
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        self.widget.screen_cursor()
    }
}

impl DateInputState {
    /// Converts a grapheme based position to a screen position
    /// relative to the widget area.
    #[inline]
    pub fn col_to_screen(&self, pos: upos_type) -> u16 {
        self.widget.col_to_screen(pos)
    }

    /// Converts from a widget relative screen coordinate to a grapheme index.
    /// x is the relative screen position.
    #[inline]
    pub fn screen_to_col(&self, scx: i16) -> upos_type {
        self.widget.screen_to_col(scx)
    }

    /// Set the cursor position from a screen position relative to the origin
    /// of the widget. This value can be negative, which selects a currently
    /// not visible position and scrolls to it.
    #[inline]
    pub fn set_screen_cursor(&mut self, cursor: i16, extend_selection: bool) -> bool {
        self.widget.set_screen_cursor(cursor, extend_selection)
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
