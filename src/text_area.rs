//!
//! A text-area with text-styling abilities.
//! And undo + clipboard support.
//!

use crate::_private::NonExhaustive;
use crate::event::TextOutcome;
use crate::text_core::TextCore;
use crate::text_store::text_rope::TextRope;
use crate::text_store::TextStore;
use crate::undo_buffer::{UndoBuffer, UndoEntry};
use crate::{upos_type, Glyph, Grapheme, TextError, TextPosition, TextRange};
use rat_event::util::MouseFlags;
use rat_focus::{FocusFlag, HasFocusFlag, Navigation};
use rat_scrolled::{Scroll, ScrollState};
use ratatui::layout::Rect;
use ratatui::prelude::Style;
use ratatui::widgets::Block;
use ropey::{Rope, RopeSlice};
use std::borrow::Cow;
use std::ops::Range;

/// Text area widget.
///
/// Backend used is [ropey](https://docs.rs/ropey/latest/ropey/), so large
/// texts are no problem. Editing time increases with the number of
/// styles applied. Everything below a million styles should be fine.
///
/// For emoji support this uses
/// [unicode_display_width](https://docs.rs/unicode-display-width/latest/unicode_display_width/index.html)
/// which helps with those double-width emojis. Input of emojis
/// strongly depends on the terminal. It may or may not work.
/// And even with display there are sometimes strange glitches
/// that I haven't found yet.
///
/// Keyboard and mouse are implemented for crossterm, but it should be
/// easy to extend to other event-types. Every interaction is available
/// as function on the state.
///
/// Scrolling doesn't depend on the cursor, but the editing and move
/// functions take care that the cursor stays visible.
///
/// Wordwrap is not available. For display only use
/// [Paragraph](https://docs.rs/ratatui/latest/ratatui/widgets/struct.Paragraph.html), as
/// for editing: why?
///
/// You can directly access the underlying Rope for readonly purposes, and
/// conversion from/to byte/char positions are available. That should probably be
/// enough to write a parser that generates some styling.
///
/// The cursor must set externally on the ratatui Frame as usual.
/// [screen_cursor](TextAreaState::screen_cursor) gives you the correct value.
/// There is the inverse too [set_screen_cursor](TextAreaState::set_screen_cursor)
/// For more interactions you can use [from_screen_col](TextAreaState::from_screen_col),
/// and [to_screen_col](TextAreaState::to_screen_col). They calculate everything,
/// even in the presence of more complex graphemes and those double-width emojis.
#[derive(Debug, Default, Clone)]
pub struct TextArea<'a> {
    block: Option<Block<'a>>,
    hscroll: Option<Scroll<'a>>,
    h_max_offset: Option<usize>,
    h_overscroll: Option<usize>,
    vscroll: Option<Scroll<'a>>,

    style: Style,
    focus_style: Option<Style>,
    select_style: Option<Style>,
    text_style: Vec<Style>,
}

/// Combined style for the widget.
#[derive(Debug, Clone)]
pub struct TextAreaStyle {
    pub style: Style,
    pub focus: Option<Style>,
    pub select: Option<Style>,
    pub non_exhaustive: NonExhaustive,
}

/// State & event handling.
#[derive(Debug)]
pub struct TextAreaState {
    /// Current focus state.
    pub focus: FocusFlag,
    /// Complete area.
    pub area: Rect,
    /// Area inside the borders.
    pub inner: Rect,

    /// Text edit core
    pub value: TextCore<TextRope>,

    /// Horizontal scroll
    pub hscroll: ScrollState,
    pub vscroll: ScrollState,

    /// Helper for mouse.
    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

impl Clone for TextAreaState {
    fn clone(&self) -> Self {
        Self {
            focus: FocusFlag::named(self.focus.name()),
            area: self.area,
            inner: self.inner,
            value: self.value.clone(),
            hscroll: self.hscroll.clone(),
            vscroll: self.vscroll.clone(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl Default for TextAreaStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            focus: None,
            select: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a> TextArea<'a> {
    /// New widget.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the combined style.
    #[inline]
    pub fn styles(mut self, style: TextAreaStyle) -> Self {
        self.style = style.style;
        self.focus_style = style.focus;
        self.select_style = style.select;
        self
    }

    /// Base style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Style when focused.
    pub fn focus_style(mut self, style: Style) -> Self {
        self.focus_style = Some(style);
        self
    }

    /// Selection style.
    pub fn select_style(mut self, style: Style) -> Self {
        self.select_style = Some(style);
        self
    }

    /// List of text-styles.
    ///
    /// Use [TextAreaState::add_style()] to refer a text range to
    /// one of these styles.
    pub fn text_style<T: IntoIterator<Item = Style>>(mut self, styles: T) -> Self {
        self.text_style = styles.into_iter().collect();
        self
    }

    /// Block.
    #[inline]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set both scrollbars.
    pub fn scroll(mut self, scroll: Scroll<'a>) -> Self {
        self.hscroll = Some(scroll.clone().override_horizontal());
        self.vscroll = Some(scroll.override_vertical());
        self
    }

    /// Set the horizontal scrollbar.
    pub fn hscroll(mut self, scroll: Scroll<'a>) -> Self {
        self.hscroll = Some(scroll.override_horizontal());
        self
    }

    /// Set a maximum horizontal offset that will be used even
    /// if there is no horizontal scrollbar set.
    ///
    /// This widget doesn't try to find a maximum text-length for
    /// all lines.
    ///
    /// Default is 255
    pub fn set_horizontal_max_offset(mut self, offset: usize) -> Self {
        self.h_max_offset = Some(offset);
        self
    }

    /// Set a horizontal overscroll that will be used even if
    /// there is no horizontal scrollbar set.
    ///
    /// Default is 16384
    pub fn set_horizontal_overscroll(mut self, overscroll: usize) -> Self {
        self.h_overscroll = Some(overscroll);
        self
    }

    /// Set the vertical scrollbar.
    pub fn vscroll(mut self, scroll: Scroll<'a>) -> Self {
        self.vscroll = Some(scroll.override_vertical());
        self
    }
}

impl Default for TextAreaState {
    fn default() -> Self {
        let mut s = Self {
            focus: Default::default(),
            area: Default::default(),
            inner: Default::default(),
            mouse: Default::default(),
            value: TextCore::default(),
            hscroll: Default::default(),
            non_exhaustive: NonExhaustive,
            vscroll: Default::default(),
        };
        s.hscroll.set_max_offset(255);
        s.hscroll.set_overscroll_by(Some(16384));
        s
    }
}

impl HasFocusFlag for TextAreaState {
    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn navigable(&self) -> Navigation {
        Navigation::Reach
    }
}

impl TextAreaState {
    /// New State.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// New state with a focus name.
    #[inline]
    pub fn named(name: &str) -> Self {
        Self {
            focus: FocusFlag::named(name),
            ..Default::default()
        }
    }

    /// Sets the line ending used for insert.
    /// There is no auto-detection or conversion done for set_value().
    ///
    /// Caution: If this doesn't match the line ending used in the value, you
    /// will get a value with mixed line endings.
    #[inline]
    pub fn set_newline(&mut self, br: impl Into<String>) {
        self.value.set_newline(br.into());
    }

    /// Line ending used for insert.
    #[inline]
    pub fn newline(&self) -> &str {
        self.value.newline()
    }

    /// Set tab-width.
    #[inline]
    pub fn set_tab_width(&mut self, tabs: upos_type) {
        self.value.set_tab_width(tabs);
    }

    /// Tab-width
    #[inline]
    pub fn tab_width(&self) -> upos_type {
        self.value.tab_width()
    }

    /// Expand tabs to spaces. Only for new inputs.
    #[inline]
    pub fn set_expand_tabs(&mut self, expand: bool) {
        self.value.set_expand_tabs(expand);
    }

    /// Expand tabs to spaces. Only for new inputs.
    #[inline]
    pub fn expand_tabs(&self) -> bool {
        self.value.expand_tabs()
    }

    /// Set undo buffer.
    pub fn set_undo_buffer(&mut self, undo: impl UndoBuffer + 'static) {
        self.value.set_undo_buffer(Box::new(undo));
    }

    /// Undo
    #[inline]
    pub fn undo_buffer(&self) -> Option<&dyn UndoBuffer> {
        self.value.undo_buffer()
    }

    /// Undo
    #[inline]
    pub fn undo_buffer_mut(&mut self) -> Option<&mut dyn UndoBuffer> {
        self.value.undo_buffer_mut()
    }

    /// Get all recent replay recordings.
    pub fn recent_replay_log(&mut self) -> Vec<UndoEntry> {
        self.value.recent_replay_log()
    }

    /// Apply the replay recording.
    pub fn replay_log(&mut self, replay: &[UndoEntry]) -> Result<(), TextError> {
        self.value.replay_log(replay)
    }

    /// Clear everything.
    #[inline]
    pub fn clear(&mut self) -> bool {
        if !self.value.is_empty() {
            self.value.clear();
            true
        } else {
            false
        }
    }

    /// Empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Current offset for scrolling.
    #[inline]
    pub fn offset(&self) -> (usize, usize) {
        (self.hscroll.offset(), self.vscroll.offset())
    }

    /// Set the offset for scrolling.
    #[inline]
    pub fn set_offset(&mut self, offset: (usize, usize)) -> bool {
        let c = self.hscroll.set_offset(offset.0);
        let r = self.vscroll.set_offset(offset.1);
        r || c
    }

    /// Cursor position.
    #[inline]
    pub fn cursor(&self) -> TextPosition {
        self.value.cursor()
    }

    /// Set the cursor position.
    /// This doesn't scroll the cursor to a visible position.
    /// Use [TextAreaState::scroll_cursor_to_visible()] for that.
    #[inline]
    pub fn set_cursor(&mut self, cursor: impl Into<TextPosition>, extend_selection: bool) -> bool {
        self.value.set_cursor(cursor.into(), extend_selection)
    }

    /// Selection anchor.
    #[inline]
    pub fn anchor(&self) -> TextPosition {
        self.value.anchor()
    }

    /// Text value
    #[inline]
    pub fn text(&self) -> String {
        self.value.text().string()
    }

    /// Set the text value.
    /// Resets all internal state.
    #[inline]
    pub fn set_text<S: AsRef<str>>(&mut self, s: S) {
        self.vscroll.set_offset(0);
        self.hscroll.set_offset(0);

        self.value.set_text(TextRope::new_text(s.as_ref()));
    }

    /// Set the text value as a Rope.
    /// Resets all internal state.
    #[inline]
    pub fn set_rope(&mut self, r: Rope) {
        self.vscroll.set_offset(0);
        self.hscroll.set_offset(0);

        self.value.set_text(TextRope::new_rope(r));
    }

    /// Borrow the rope
    #[inline]
    pub fn rope(&self) -> &Rope {
        self.value.text().rope()
    }

    /// Text slice as RopeSlice
    #[inline]
    pub fn rope_slice(&self, range: impl Into<TextRange>) -> Result<RopeSlice<'_>, TextError> {
        self.value.text().rope_slice(range.into())
    }

    /// Text slice as Cow<str>
    #[inline]
    pub fn str_slice(&self, range: impl Into<TextRange>) -> Result<Cow<'_, str>, TextError> {
        self.value.str_slice(range.into())
    }

    /// Line count.
    #[inline]
    pub fn line_len(&self) -> upos_type {
        self.value.len_lines()
    }

    /// Line width as grapheme count.
    #[inline]
    pub fn line_width(&self, row: upos_type) -> Result<upos_type, TextError> {
        self.value.line_width(row)
    }

    /// Line as RopeSlice.
    /// This contains the \n at the end.
    pub fn line_at(&self, row: upos_type) -> Result<Cow<'_, str>, TextError> {
        self.value.line_at(row)
    }

    /// Iterate over text-lines, starting at offset.
    #[inline]
    pub fn lines_at(
        &self,
        row: upos_type,
    ) -> Result<impl Iterator<Item = Cow<'_, str>>, TextError> {
        self.value.lines_at(row)
    }

    /// Iterator for the glyphs of a given line.
    /// Glyphs here a grapheme + display length.
    /// This covers multi-column graphemes as well as tabs (with varying width).
    /// This contains the \n at the end.
    #[inline]
    pub fn line_glyphs(
        &self,
        row: upos_type,
        col_offset: upos_type,
    ) -> Result<impl Iterator<Item = Glyph<'_>>, TextError> {
        self.value.line_glyphs(row, col_offset)
    }

    /// Grapheme iterator for a given line.
    /// This contains the \n at the end.
    #[inline]
    pub fn line_graphemes(
        &self,
        row: upos_type,
    ) -> Result<impl Iterator<Item = Grapheme<'_>>, TextError> {
        self.value.line_graphemes(row)
    }

    /// Has a selection?
    #[inline]
    pub fn has_selection(&self) -> bool {
        self.value.has_selection()
    }

    /// Current selection.
    #[inline]
    pub fn selection(&self) -> TextRange {
        self.value.selection()
    }

    /// Set the selection.
    #[inline]
    pub fn set_selection(&mut self, range: impl Into<TextRange>) -> bool {
        self.value.set_selection(range.into())
    }

    /// Select all.
    #[inline]
    pub fn select_all(&mut self) -> bool {
        self.value.select_all()
    }

    /// Selection.
    #[inline]
    pub fn selected_value(&self) -> Result<Cow<'_, str>, TextError> {
        self.value.str_slice(self.value.selection())
    }

    /// Set and replace all styles.
    #[inline]
    pub fn set_styles(&mut self, styles: Vec<(Range<usize>, usize)>) {
        self.value.set_styles(styles);
    }

    /// Add a style for a [TextRange]. The style-nr refers to one
    /// of the styles set with the widget.
    #[inline]
    pub fn add_style(&mut self, range: Range<usize>, style: usize) {
        self.value.add_style(range.into(), style);
    }

    /// Remove the exact TextRange and style.
    #[inline]
    pub fn remove_style(&mut self, range: Range<usize>, style: usize) {
        self.value.remove_style(range.into(), style);
    }

    /// All styles active at the given position.
    #[inline]
    pub fn styles_at(&self, byte_pos: usize, buf: &mut Vec<usize>) {
        self.value.styles_at(byte_pos, buf)
    }

    /// Check if the given style applies at the position and
    /// return the complete range for the style.
    #[inline]
    pub fn style_match(&self, byte_pos: usize, style: usize) -> Option<Range<usize>> {
        self.value.style_match(byte_pos, style.into())
    }

    /// Insert a character at the cursor position.
    /// Removes the selection and inserts the char.
    pub fn insert_char(&mut self, c: char) -> TextOutcome {
        if self.value.has_selection() {
            self.value
                .remove_str_range(self.value.selection())
                .expect("valid_selection");
        }
        if c == '\n' {
            self.value
                .insert_newline(self.value.cursor())
                .expect("valid_cursor");
        } else if c == '\t' {
            self.value
                .insert_tab(self.value.cursor())
                .expect("valid_cursor");
        } else {
            self.value
                .insert_char(self.value.cursor(), c)
                .expect("valid_cursor");
        }
        self.scroll_cursor_to_visible();
        TextOutcome::TextChanged
    }

    /// Insert a character at the cursor position.
    /// Removes the selection and inserts the char.
    pub fn insert_tab(&mut self) -> TextOutcome {
        if self.value.has_selection() {
            self.value
                .remove_str_range(self.value.selection())
                .expect("valid_selection");
        }
        self.value
            .insert_tab(self.value.cursor())
            .expect("valid_cursor");
        self.scroll_cursor_to_visible();
        TextOutcome::TextChanged
    }

    /// Insert text at the cursor position.
    /// Removes the selection and inserts the text.
    pub fn insert_str(&mut self, t: impl AsRef<str>) -> TextOutcome {
        let t = t.as_ref();
        if self.value.has_selection() {
            self.value
                .remove_str_range(self.value.selection())
                .expect("valid_selection");
        }
        self.value
            .insert_str(self.value.cursor(), t)
            .expect("valid_cursor");
        self.scroll_cursor_to_visible();
        TextOutcome::TextChanged
    }

    /// Insert a line break at the cursor position.
    pub fn insert_newline(&mut self) -> TextOutcome {
        if self.value.has_selection() {
            self.value
                .remove_str_range(self.value.selection())
                .expect("valid_selection");
        }
        self.value
            .insert_newline(self.value.cursor())
            .expect("valid_cursor");

        // insert leading spaces
        let pos = self.value.cursor();
        if pos.y > 0 {
            let mut blanks = String::new();
            for c in self.value.line_graphemes(pos.y - 1).expect("valid_cursor") {
                if c.grapheme == " " || c == "\t" {
                    blanks.push_str(c.grapheme.as_ref());
                } else {
                    break;
                }
            }
            if blanks.len() > 0 {
                self.value.insert_str(pos, &blanks).expect("valid_cursor");
            }
        }

        self.scroll_cursor_to_visible();
        TextOutcome::TextChanged
    }

    /// Deletes the given range.
    pub fn delete_range(&mut self, range: impl Into<TextRange>) -> Result<TextOutcome, TextError> {
        let range = range.into();
        if !range.is_empty() {
            self.value.remove_str_range(range)?;
            self.scroll_cursor_to_visible();
            Ok(TextOutcome::TextChanged)
        } else {
            Ok(TextOutcome::Unchanged)
        }
    }

    /// Duplicates the selection or the current line.
    /// Returns TextOutcome::TextChanged if there was any real change.
    pub fn duplicate_text(&mut self) -> TextOutcome {
        if self.value.has_selection() {
            let sel_range = self.value.selection();
            if !sel_range.is_empty() {
                let v = self
                    .value
                    .str_slice(sel_range)
                    .expect("valid_selection")
                    .to_string();
                self.value
                    .insert_str(sel_range.end, &v)
                    .expect("valid_selection");
                TextOutcome::TextChanged
            } else {
                TextOutcome::Unchanged
            }
        } else {
            let pos = self.value.cursor();
            let row_range = TextRange::new((0, pos.y), (0, pos.y + 1));
            let v = self
                .value
                .str_slice(row_range)
                .expect("valid_cursor")
                .to_string();
            self.value
                .insert_str(row_range.start, &v)
                .expect("valid_cursor");
            TextOutcome::TextChanged
        }
    }

    /// Deletes the current line.
    /// Returns true if there was any real change.
    pub fn delete_line(&mut self) -> TextOutcome {
        let pos = self.value.cursor();
        if pos.y + 1 < self.value.len_lines() {
            self.delete_range(TextRange::new((0, pos.y), (0, pos.y + 1)))
                .expect("valid_cursor")
        } else {
            let width = self.value.line_width(pos.y).expect("valid_cursor");
            self.delete_range(TextRange::new((0, pos.y), (width, pos.y)))
                .expect("valid_cursor")
        }
    }

    /// Deletes the next char or the current selection.
    /// Returns true if there was any real change.
    pub fn delete_next_char(&mut self) -> TextOutcome {
        if self.value.has_selection() {
            self.delete_range(self.selection())
                .expect("valid_selection")
        } else {
            let r = self
                .value
                .remove_next_char(self.value.cursor())
                .expect("valid_cursor");
            let s = self.scroll_cursor_to_visible();

            if r {
                TextOutcome::TextChanged
            } else if s {
                TextOutcome::Changed
            } else {
                TextOutcome::Continue
            }
        }
    }

    /// Deletes the previous char or the selection.
    /// Returns true if there was any real change.
    pub fn delete_prev_char(&mut self) -> TextOutcome {
        if self.value.has_selection() {
            self.delete_range(self.selection())
                .expect("valid_selection")
        } else {
            let r = self
                .value
                .remove_prev_char(self.value.cursor())
                .expect("valid_cursor");
            let s = self.scroll_cursor_to_visible();

            if r {
                TextOutcome::TextChanged
            } else if s {
                TextOutcome::Changed
            } else {
                TextOutcome::Continue
            }
        }
    }

    pub fn scroll_cursor_to_visible(&mut self) -> bool {
        todo!()
    }
}
