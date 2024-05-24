//! Text input with an input mask.

use crate::_private::NonExhaustive;
use crate::{ControlUI, ValidFlag};
use crate::{DefaultKeys, FrameWidget, HandleCrossterm, MouseOnly};
use crate::{FocusFlag, HasFocusFlag, HasValidFlag};
use format_num_pattern::NumberSymbols;
#[allow(unused_imports)]
use log::debug;
use rat_input::masked_input;
use rat_input::masked_input::{MaskedInput, MaskedInputState, MaskedInputStyle};
use ratatui::layout::{Position, Rect};
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::Frame;
use std::fmt;
use std::ops::Range;

/// Text input widget with input mask.
#[derive(Debug)]
pub struct MaskedInputExt<'a> {
    widget: MaskedInput<'a>,
}

impl<'a> Default for MaskedInputExt<'a> {
    fn default() -> Self {
        Self {
            widget: Default::default(),
        }
    }
}

impl<'a> MaskedInputExt<'a> {
    /// Show the compact form, if the focus is not with this widget.
    pub fn show_compact(mut self, show_compact: bool) -> Self {
        self.widget = self.widget.show_compact(show_compact);
        self
    }

    /// Set the combined style.
    pub fn styles(mut self, style: MaskedInputStyle) -> Self {
        self.widget = self.widget.styles(style);
        self
    }

    /// Base text style.
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.style(style);
        self
    }

    /// Style when focused.
    pub fn focus_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.focus_style(style);
        self
    }

    /// Style for selection
    pub fn select_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.select_style(style);
        self
    }

    /// Style for the invalid indicator.
    pub fn invalid_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.invalid_style(style);
        self
    }

    /// Style for the invalid indicator.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.widget = self.widget.block(block);
        self
    }
}

impl<'a> FrameWidget for MaskedInputExt<'a> {
    type State = MaskedInputExtState;

    #[allow(clippy::vec_init_then_push)]
    fn render(mut self, frame: &mut Frame<'_>, area: Rect, state: &mut Self::State) {
        state.area = area;

        self.widget = self
            .widget
            .focused(state.is_focused())
            .valid(state.is_valid());

        frame.render_stateful_widget(self.widget, area, &mut state.widget);

        if let Some(Position { x, y }) = state.widget.screen_cursor() {
            frame.set_cursor(x, y);
        }
    }
}

/// State of the input-mask.
#[derive(Debug, Clone)]
pub struct MaskedInputExtState {
    ///
    pub widget: MaskedInputState,
    /// Focus
    pub focus: FocusFlag,
    /// Area
    pub area: Rect,
    /// Valid.
    pub valid: ValidFlag,
    ///
    pub non_exhaustive: NonExhaustive,
}

impl Default for MaskedInputExtState {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            focus: Default::default(),
            area: Default::default(),
            valid: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, DefaultKeys> for MaskedInputExtState
where
    E: From<fmt::Error>,
{
    fn handle(&mut self, event: &crossterm::event::Event, _: DefaultKeys) -> ControlUI<A, E> {
        match masked_input::handle_events(&mut self.widget, self.focus.get(), event) {
            rat_input::event::Outcome::Changed => ControlUI::Change,
            rat_input::event::Outcome::Unchanged => ControlUI::NoChange,
            rat_input::event::Outcome::NotUsed => ControlUI::Continue,
        }
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for MaskedInputExtState
where
    E: From<fmt::Error>,
{
    fn handle(&mut self, event: &crossterm::event::Event, _: MouseOnly) -> ControlUI<A, E> {
        match masked_input::handle_mouse_events(&mut self.widget, event) {
            rat_input::event::Outcome::Changed => ControlUI::Change,
            rat_input::event::Outcome::Unchanged => ControlUI::NoChange,
            rat_input::event::Outcome::NotUsed => ControlUI::Continue,
        }
    }
}

impl MaskedInputExtState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_symbols(sym: NumberSymbols) -> Self {
        Self {
            widget: MaskedInputState::new_with_symbols(sym),
            ..Self::default()
        }
    }

    /// Reset to empty.
    pub fn reset(&mut self) {
        self.widget.reset();
    }

    /// Offset shown.
    pub fn offset(&self) -> usize {
        self.widget.offset()
    }

    /// Offset shown. This is corrected if the cursor wouldn't be visible.
    pub fn set_offset(&mut self, offset: usize) {
        self.widget.set_offset(offset);
    }

    /// Set the cursor position, reset selection.
    pub fn set_cursor(&mut self, cursor: usize, extend_selection: bool) {
        self.widget.set_cursor(cursor, extend_selection);
    }

    /// Place cursor at decimal separator, if any. 0 otherwise.
    pub fn set_default_cursor(&mut self) {
        self.widget.set_default_cursor();
    }

    /// Cursor position
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
    pub fn set_display_mask<S: Into<String>>(&mut self, s: S) {
        self.widget.set_display_mask(s);
    }

    /// Display mask.
    pub fn display_mask(&self) -> String {
        self.widget.display_mask()
    }

    /// Set the input mask. This overwrites the display mask and the value
    /// with a default representation of the mask.
    ///
    /// The result value contains all punctuation and
    /// the value given as 'display' below. See [compact_value()](MaskedInputState::compact_value).
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
    pub fn set_mask<S: AsRef<str>>(&mut self, s: S) -> Result<(), fmt::Error> {
        self.widget.set_mask(s)
    }

    /// Display mask.
    pub fn mask(&self) -> String {
        self.widget.mask()
    }

    /// Mask with some debug information.
    pub fn debug_mask(&self) -> String {
        self.widget.debug_mask()
    }

    /// Set symbols for number display.
    ///
    /// These are only used for rendering and to map user input.
    /// The value itself uses ".", "," and "-".
    pub fn set_num_symbols(&mut self, sym: NumberSymbols) {
        self.widget.set_num_symbols(sym);
    }

    /// Set the value.
    ///
    /// No checks if the value conforms to the mask.
    /// If the value is too short it will be filled with space.
    /// if the value is too long it will be truncated.
    pub fn set_value<S: Into<String>>(&mut self, s: S) {
        self.widget.set_value(s);
    }

    /// Value with all punctuation and default values according to the mask type.
    pub fn value(&self) -> &str {
        self.widget.value()
    }

    /// Value split along any separators
    pub fn value_parts(&self) -> Vec<String> {
        self.widget.value_parts()
    }

    /// Value without optional whitespace and grouping separators. Might be easier to parse.
    pub fn compact_value(&self) -> String {
        self.widget.compact_value()
    }

    /// Value.
    pub fn as_str(&self) -> &str {
        self.widget.as_str()
    }

    ///
    pub fn is_empty(&self) -> bool {
        self.widget.is_empty()
    }

    /// Length in grapheme count.
    pub fn len(&self) -> usize {
        self.widget.len()
    }

    /// Selection
    pub fn has_selection(&self) -> bool {
        self.widget.has_selection()
    }

    /// Selection
    pub fn set_selection(&mut self, anchor: usize, cursor: usize) {
        self.widget.set_selection(anchor, cursor);
    }

    /// Selection
    pub fn select_all(&mut self) {
        self.widget.select_all();
    }

    /// Selection
    pub fn selection(&self) -> Range<usize> {
        self.widget.selection()
    }

    /// Selection
    pub fn selection_str(&self) -> &str {
        self.widget.selection_str()
    }

    /// Set the cursor position from a visual position relative to the origin.
    pub fn set_visual_cursor(&mut self, rpos: isize, extend_selection: bool) {
        self.widget.set_visual_cursor(rpos, extend_selection);
    }

    /// Previous word boundary.
    pub fn prev_word_boundary(&self) -> usize {
        self.widget.prev_word_boundary()
    }

    /// Next word boundary.
    pub fn next_word_boundary(&self) -> usize {
        self.widget.next_word_boundary()
    }

    /// Move to the next char.
    pub fn move_to_next(&mut self, extend_selection: bool) {
        self.widget.move_to_next(extend_selection);
    }

    /// Move to the previous char.
    pub fn move_to_prev(&mut self, extend_selection: bool) {
        self.widget.move_to_prev(extend_selection);
    }

    /// Insert a char at the current position.
    pub fn insert_char(&mut self, c: char) {
        self.widget.insert_char(c)
    }

    /// Remove the selected range. The text will be replaced with the default value
    /// as defined by the mask.
    pub fn remove_selection(&mut self, selection: Range<usize>) {
        self.widget.remove_selection(selection)
    }

    /// Delete the char before the cursor.
    pub fn delete_prev_char(&mut self) {
        self.widget.delete_prev_char()
    }

    /// Delete the char after the cursor.
    pub fn delete_next_char(&mut self) {
        self.widget.delete_next_char()
    }
}

impl HasFocusFlag for MaskedInputExtState {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl HasValidFlag for MaskedInputExtState {
    fn valid(&self) -> &ValidFlag {
        &self.valid
    }
}
