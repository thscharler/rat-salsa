//! Text input with an input mask.
//!
//! * Can do the usual insert/delete/move operations.
//! * Text selection with keyboard + mouse
//! * Scrolls with the cursor.
//! * Modes for focus and valid.
//! * Info-overlay for sub-fields without value.
//! * Localization with [format_num_pattern::NumberSymbols]
//!
//! * Accepts an input mask:
//!   * `0`: can enter digit, display as 0
//!   * `9`: can enter digit, display as space
//!   * `#`: digit, plus or minus sign, display as space
//!   * `-`: sign
//!   * `+`: sign, positive is '+', negative is '-', not localized.
//!   * `.` and `,`: decimal and grouping separators
//!
//!   * `H`: must enter a hex digit, display as 0
//!   * `h`: can enter a hex digit, display as space
//!   * `O`: must enter an octal digit, display as 0
//!   * `o`: can enter an octal digit, display as space
//!   * `D`: must enter a decimal digit, display as 0
//!   * `d`: can enter a decimal digit, display as space
//!
//!   * `l`: can enter letter, display as space
//!   * `a`: can enter letter or digit, display as space
//!   * `c`: can enter character or space, display as space
//!   * `_`: anything, display as space
//!
//!   * `<space>`, `:`, `;`, `/`: separator characters move the cursor when entered.
//!   * `\`: escapes the following character and uses it as a separator.
//!   * all other ascii characters a reserved.
//!   * other unicode characters can be used as separators without escaping.
//!
//! * Accepts a display overlay used instead of the default chars of the input mask.
//!
//! ```rust ignore
//! use ratatui::widgets::StatefulWidget;
//! use rat_input::masked_input::{MaskedInput, MaskedInputState};
//!
//! let date_focused = false;
//! let creditcard_focused = true;
//! let area = Rect::default();
//! let buf = Buffer::default();
//!
//! let mut date_state = MaskedInputState::new();
//! date_state.set_mask("99/99/9999")?;
//! date_state.set_display_mask("mm/dd/yyyy");
//!
//! let w_date = MaskedInput::default();
//! w_date.render(area, &mut buf, &mut date_state);
//! if date_focused {
//!     frame.set_cursor(date_state.cursor.x, date_state.cursor.y);
//! }
//!
//! let mut creditcard_state = MaskedInputState::new();
//! creditcard_state.set_mask("dddd dddd dddd dddd")?;
//!
//! let w_creditcard = MaskedInput::default();
//! w_creditcard.render(area, &mut buf, &mut creditcard_state);
//! if creditcard_focused {
//!     frame.set_cursor(creditcard_state.cursor.x, creditcard_state.cursor.y);
//! }
//!
//! ```
//!
//! Event handling by calling the freestanding fn [handle_events].
//! There's [handle_mouse_events] if you want to override the default key bindings but keep
//! the mouse behaviour.

use crate::_private::NonExhaustive;
use format_num_pattern::NumberSymbols;
use rat_event::util::Outcome;
use rat_event::{FocusKeys, HandleEvent, MouseOnly};
use rat_focus::{FocusFlag, HasFocusFlag};
pub use rat_input::masked_input::MaskedInputStyle;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::prelude::Style;
use ratatui::widgets::{Block, StatefulWidget, StatefulWidgetRef};
use std::fmt;
use std::ops::Range;

#[derive(Debug, Default, Clone)]
pub struct MaskedInput<'a> {
    widget: rat_input::masked_input::MaskedInput<'a>,
}

#[derive(Debug, Clone)]
pub struct MaskedInputState {
    pub widget: rat_input::masked_input::MaskedInputState,
    pub focus: FocusFlag,
    pub valid: bool,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> MaskedInput<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the compact form, if the focus is not with this widget.
    #[inline]
    pub fn show_compact(mut self, show_compact: bool) -> Self {
        self.widget = self.widget.show_compact(show_compact);
        self
    }

    /// Set the combined style.
    #[inline]
    pub fn styles(mut self, style: MaskedInputStyle) -> Self {
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
}

impl<'a> StatefulWidget for MaskedInput<'a> {
    type State = MaskedInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget
            .focused(state.is_focused())
            .valid(state.valid)
            .render(area, buf, &mut state.widget)
    }
}

impl<'a> StatefulWidgetRef for MaskedInput<'a> {
    type State = MaskedInputState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget
            .clone()
            .focused(state.is_focused())
            .valid(state.valid)
            .render(area, buf, &mut state.widget)
    }
}

impl Default for MaskedInputState {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            focus: Default::default(),
            valid: true,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl HasFocusFlag for MaskedInputState {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.widget.area
    }
}

impl MaskedInputState {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn new_with_symbols(sym: NumberSymbols) -> Self {
        Self {
            widget: rat_input::masked_input::MaskedInputState::new_with_symbols(sym),
            ..Default::default()
        }
    }

    /// Reset to empty.
    #[inline]
    pub fn reset(&mut self) {
        self.widget.reset()
    }

    /// Offset shown.
    #[inline]
    pub fn offset(&self) -> usize {
        self.widget.offset()
    }

    /// Offset shown. This is corrected if the cursor wouldn't be visible.
    #[inline]
    pub fn set_offset(&mut self, offset: usize) {
        self.widget.set_offset(offset)
    }

    /// Set the cursor position, reset selection.
    #[inline]
    pub fn set_cursor(&mut self, cursor: usize, extend_selection: bool) {
        self.widget.set_cursor(cursor, extend_selection)
    }

    /// Place cursor at decimal separator, if any. 0 otherwise.
    #[inline]
    pub fn set_default_cursor(&mut self) {
        self.widget.set_default_cursor()
    }

    /// Cursor position
    #[inline]
    pub fn cursor(&self) -> usize {
        self.widget.cursor()
    }

    /// Set the display mask. This text is used for parts that have
    /// no valid input yet. Part means consecutive characters of the
    /// input mask with the same mask type.
    ///
    /// There is a default representation for each mask type if this
    /// is not set.
    ///
    /// If the length differs from the mask, the difference will be
    /// ignored / filled with defaults.
    #[inline]
    pub fn set_display_mask<S: Into<String>>(&mut self, s: S) {
        self.widget.set_display_mask(s)
    }

    /// Display mask.
    #[inline]
    pub fn display_mask(&self) -> String {
        self.widget.display_mask()
    }

    /// Set the input mask. This overwrites the display mask and the value
    /// with a default representation of the mask.
    ///
    /// The result value contains all punctuation and
    /// the value given as 'display' below. See [compact_value()](rat_input::masked_input::MaskedInputState::compact_value).
    ///
    /// * `0`: can enter digit, display as 0
    /// * `9`: can enter digit, display as space
    /// * `#`: digit, plus or minus sign, display as space
    /// * `+`: sign. display '+' for positive
    /// * `-`: sign. display ' ' for positive
    /// * `.` and `,`: decimal and grouping separators
    ///
    /// * `H`: must enter a hex digit, display as 0
    /// * `h`: can enter a hex digit, display as space
    /// * `O`: must enter an octal digit, display as 0
    /// * `o`: can enter an octal digit, display as space
    /// * `D`: must enter a decimal digit, display as 0
    /// * `d`: can enter a decimal digit, display as space
    ///
    /// * `l`: can enter letter, display as space
    /// * `a`: can enter letter or digit, display as space
    /// * `c`: can enter character or space, display as space
    /// * `_`: anything, display as space
    ///
    /// * `:` `;` `-` `/`: separator characters move the cursor when entered.
    /// * `\`: escapes the following character and uses it as a separator.
    /// * all other ascii characters a reserved.
    ///
    /// Inspired by <https://support.microsoft.com/en-gb/office/control-data-entry-formats-with-input-masks-e125997a-7791-49e5-8672-4a47832de8da>
    #[inline]
    pub fn set_mask<S: AsRef<str>>(&mut self, s: S) -> Result<(), fmt::Error> {
        self.widget.set_mask(s)
    }

    /// Display mask.
    #[inline]
    pub fn mask(&self) -> String {
        self.widget.mask()
    }

    /// Mask with some debug information.
    #[inline]
    pub fn debug_mask(&self) -> String {
        self.widget.debug_mask()
    }

    /// Set symbols for number display.
    ///
    /// These are only used for rendering and to map user input.
    /// The value itself uses ".", "," and "-".
    #[inline]
    pub fn set_num_symbols(&mut self, sym: NumberSymbols) {
        self.widget.set_num_symbols(sym)
    }

    /// Create a default value according to the mask.
    #[inline]
    pub fn default_value(&self) -> String {
        self.widget.default_value()
    }

    /// Set the value.
    ///
    /// No checks if the value conforms to the mask.
    /// If the value is too short it will be filled with space.
    /// if the value is too long it will be truncated.
    #[inline]
    pub fn set_value<S: Into<String>>(&mut self, s: S) {
        self.widget.set_value(s)
    }

    /// Value with all punctuation and default values according to the mask type.
    #[inline]
    pub fn value(&self) -> &str {
        self.widget.value()
    }

    /// Value split along any separators
    #[inline]
    pub fn value_parts(&self) -> Vec<String> {
        self.widget.value_parts()
    }

    /// Value without optional whitespace and grouping separators. Might be easier to parse.
    #[inline]
    pub fn compact_value(&self) -> String {
        self.widget.compact_value()
    }

    /// Value.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.widget.as_str()
    }

    ///
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.widget.is_empty()
    }

    /// Length in grapheme count.
    #[inline]
    pub fn len(&self) -> usize {
        self.widget.len()
    }

    /// Selection
    #[inline]
    pub fn has_selection(&self) -> bool {
        self.widget.has_selection()
    }

    /// Selection
    #[inline]
    pub fn set_selection(&mut self, anchor: usize, cursor: usize) {
        self.widget.set_selection(anchor, cursor)
    }

    /// Selection
    #[inline]
    pub fn select_all(&mut self) {
        self.widget.select_all()
    }

    /// Selection
    #[inline]
    pub fn selection(&self) -> Range<usize> {
        self.widget.selection()
    }

    /// Selection
    #[inline]
    pub fn selection_str(&self) -> &str {
        self.widget.selection_str()
    }

    /// Set the cursor position from a visual position relative to the origin.
    #[inline]
    pub fn set_visual_cursor(&mut self, rpos: isize, extend_selection: bool) -> bool {
        self.widget.set_visual_cursor(rpos, extend_selection)
    }

    /// The current text cursor as an absolute screen position.
    #[inline]
    pub fn screen_cursor(&self) -> Option<Position> {
        self.widget.screen_cursor()
    }

    /// Previous word boundary.
    #[inline]
    pub fn prev_word_boundary(&self) -> usize {
        self.widget.prev_word_boundary()
    }

    /// Next word boundary.
    #[inline]
    pub fn next_word_boundary(&self) -> usize {
        self.widget.next_word_boundary()
    }

    /// Move to the next char.
    #[inline]
    pub fn move_to_next(&mut self, extend_selection: bool) {
        self.widget.move_to_next(extend_selection)
    }

    /// Move to the previous char.
    #[inline]
    pub fn move_to_prev(&mut self, extend_selection: bool) {
        self.widget.move_to_prev(extend_selection)
    }

    /// Insert a char at the current position.
    #[inline]
    pub fn insert_char(&mut self, c: char) {
        self.widget.insert_char(c)
    }

    /// Remove the selected range. The text will be replaced with the default value
    /// as defined by the mask.
    #[inline]
    pub fn remove_selection(&mut self, selection: Range<usize>) {
        self.widget.remove_selection(selection)
    }

    /// Delete the char before the cursor.
    #[inline]
    pub fn delete_prev_char(&mut self) {
        self.widget.delete_prev_char()
    }

    /// Delete the char after the cursor.
    #[inline]
    pub fn delete_next_char(&mut self) {
        self.widget.delete_next_char()
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for MaskedInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> Outcome {
        if self.is_focused() {
            self.widget.handle(event, FocusKeys)
        } else {
            self.widget.handle(event, MouseOnly)
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for MaskedInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        self.widget.handle(event, MouseOnly)
    }
}
