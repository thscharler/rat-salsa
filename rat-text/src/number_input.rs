//!
//! Number input widget
//!

use crate::_private::NonExhaustive;
use crate::clipboard::Clipboard;
use crate::event::{ReadOnly, TextOutcome};
use crate::text_input_mask::{MaskedInput, MaskedInputState};
use crate::undo_buffer::{UndoBuffer, UndoEntry};
use crate::{upos_type, HasScreenCursor, TextError, TextFocusGained, TextFocusLost, TextStyle};
use format_num_pattern::{NumberFmtError, NumberFormat, NumberSymbols};
use rat_event::{HandleEvent, MouseOnly, Regular};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_reloc::RelocatableState;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{StatefulWidget, Style};
use ratatui::widgets::Block;
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::StatefulWidgetRef;
use std::fmt::{Debug, Display, LowerExp};
use std::ops::Range;
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

    /// Set the combined style.
    #[inline]
    pub fn styles(mut self, style: TextStyle) -> Self {
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

    #[inline]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.widget = self.widget.block(block);
        self
    }

    /// Focus behaviour
    #[inline]
    pub fn on_focus_gained(mut self, of: TextFocusGained) -> Self {
        self.widget = self.widget.on_focus_gained(of);
        self
    }

    /// Focus behaviour
    #[inline]
    pub fn on_focus_lost(mut self, of: TextFocusLost) -> Self {
        self.widget = self.widget.on_focus_lost(of);
        self
    }
}

#[cfg(feature = "unstable-widget-ref")]
impl<'a> StatefulWidgetRef for NumberInput<'a> {
    type State = NumberInputState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget.render_ref(area, buf, &mut state.widget);
    }
}

impl StatefulWidget for NumberInput<'_> {
    type State = NumberInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget.render(area, buf, &mut state.widget);
    }
}

impl Default for NumberInputState {
    fn default() -> Self {
        let mut s = Self {
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

impl HasFocus for NumberInputState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    #[inline]
    fn focus(&self) -> FocusFlag {
        self.widget.focus.clone()
    }

    #[inline]
    fn area(&self) -> Rect {
        self.widget.area
    }

    fn navigable(&self) -> Navigation {
        self.widget.navigable()
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
        Self {
            widget: MaskedInputState::named(name),
            ..Default::default()
        }
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

    /// The next edit operation will overwrite the current content
    /// instead of adding text. Any move operations will cancel
    /// this overwrite.
    #[inline]
    pub fn set_overwrite(&mut self, overwrite: bool) {
        self.widget.overwrite = overwrite;
    }

    /// Will the next edit operation overwrite the content?
    #[inline]
    pub fn overwrite(&self) -> bool {
        self.widget.overwrite
    }
}

impl NumberInputState {
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

impl NumberInputState {
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

impl NumberInputState {
    /// Set and replace all styles.
    #[inline]
    pub fn set_styles(&mut self, styles: Vec<(Range<usize>, usize)>) {
        self.widget.set_styles(styles);
    }

    /// Add a style for a byter-range. The style-nr refers to one
    /// of the styles set with the widget.
    #[inline]
    pub fn add_style(&mut self, range: Range<usize>, style: usize) {
        self.widget.add_style(range, style);
    }

    /// Add a style for a `Range<upos_type>` to denote the cells.
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
        self.widget.remove_style(range, style);
    }

    /// Remove the exact `Range<upos_type>` and style.
    #[inline]
    pub fn remove_range_style(
        &mut self,
        range: Range<upos_type>,
        style: usize,
    ) -> Result<(), TextError> {
        self.widget.remove_range_style(range, style)
    }

    /// Find all styles that touch the given range.
    pub fn styles_in(&self, range: Range<usize>, buf: &mut Vec<(Range<usize>, usize)>) {
        self.widget.styles_in(range, buf)
    }

    /// All styles active at the given position.
    #[inline]
    pub fn styles_at(&self, byte_pos: usize, buf: &mut Vec<(Range<usize>, usize)>) {
        self.widget.styles_at(byte_pos, buf)
    }

    /// Check if the given style applies at the position and
    /// return the complete range for the style.
    #[inline]
    pub fn style_match(&self, byte_pos: usize, style: usize) -> Option<Range<usize>> {
        self.widget.style_match(byte_pos, style)
    }

    /// List of all styles.
    #[inline]
    pub fn styles(&self) -> Option<impl Iterator<Item = (Range<usize>, usize)> + '_> {
        self.widget.styles()
    }
}

impl NumberInputState {
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

impl NumberInputState {
    /// Empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.widget.is_empty()
    }

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

impl NumberInputState {
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

impl HasScreenCursor for NumberInputState {
    /// The current text cursor as an absolute screen position.
    #[inline]
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        self.widget.screen_cursor()
    }
}

impl RelocatableState for NumberInputState {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.widget.relocate(shift, clip);
    }
}

impl NumberInputState {
    /// Converts a grapheme based position to a screen position
    /// relative to the widget area.
    #[inline]
    pub fn col_to_screen(&self, pos: upos_type) -> Option<u16> {
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
