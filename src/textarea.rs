//!
//! A text-area with text-styling abilities.
//!
use crate::_private::NonExhaustive;
use crate::event::{ReadOnly, TextOutcome};
use crate::fill::Fill;
use crate::text::graphemes::RopeGlyphIter;
use crate::text::textarea_core::{TextAreaCore, TextPosition, TextRange};
use crossterm::event::KeyModifiers;
#[allow(unused_imports)]
use log::debug;
use rat_event::util::MouseFlags;
use rat_event::{ct_event, flow, HandleEvent, MouseOnly, Regular};
use rat_focus::{FocusFlag, HasFocusFlag};
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::{layout_scroll, Scroll, ScrollArea, ScrollState};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Stylize;
use ratatui::style::Style;
use ratatui::widgets::{Block, StatefulWidget, StatefulWidgetRef, Widget, WidgetRef};
use ropey::{Rope, RopeSlice};
use std::cmp::{max, min};
use std::fmt::Debug;
use std::ops::{Range, RangeBounds};

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
///
#[derive(Debug, Default, Clone)]
pub struct TextArea<'a> {
    block: Option<Block<'a>>,
    hscroll: Option<Scroll<'a>>,
    h_max_offset: Option<usize>,
    h_overscroll: Option<usize>,
    vscroll: Option<Scroll<'a>>,

    show_ctrl: bool,

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
    pub value: TextAreaCore,

    /// Horizontal scroll
    pub hscroll: ScrollState,
    pub vscroll: ScrollState,

    /// internal clipboard
    pub clip: String,

    /// Helper for mouse.
    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
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

    /// Show the control characters 0x00-0x1F.
    pub fn show_ctrl(mut self, show: bool) -> Self {
        self.show_ctrl = show;
        self
    }
}

impl<'a> StatefulWidgetRef for TextArea<'a> {
    type State = TextAreaState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

impl<'a> StatefulWidget for TextArea<'a> {
    type State = TextAreaState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

fn render_ref(widget: &TextArea<'_>, area: Rect, buf: &mut Buffer, state: &mut TextAreaState) {
    state.area = area;

    let (hscroll_area, vscroll_area, inner_area) = layout_scroll(
        area,
        widget.block.as_ref(),
        widget.hscroll.as_ref(),
        widget.vscroll.as_ref(),
    );
    state.inner = inner_area;
    if let Some(h_max_offset) = widget.h_max_offset {
        state.hscroll.set_max_offset(h_max_offset);
    }
    if let Some(h_overscroll) = widget.h_overscroll {
        state.hscroll.set_overscroll_by(Some(h_overscroll));
    }
    state.hscroll.set_page_len(state.inner.width as usize);
    state
        .vscroll
        .set_max_offset(state.line_len().saturating_sub(state.inner.height as usize));
    state.vscroll.set_page_len(state.inner.height as usize);

    widget.block.render_ref(area, buf);
    if let Some(hscroll) = widget.hscroll.as_ref() {
        hscroll.render_ref(hscroll_area, buf, &mut state.hscroll);
    }
    if let Some(vscroll) = widget.vscroll.as_ref() {
        vscroll.render_ref(vscroll_area, buf, &mut state.vscroll);
    }

    let area = state.inner;

    let select_style = if let Some(select_style) = widget.select_style {
        select_style
    } else {
        Style::default().on_yellow()
    };
    let style = widget.style;

    Fill::new().style(style).render(area, buf);

    let selection = state.selection();
    let mut styles = Vec::new();

    let mut line_iter = state.value.lines_at(state.vscroll.offset());
    let (ox, oy) = state.offset();
    for row in 0..area.height {
        // text-index
        let ty = oy + row as usize;

        if let Some(line) = line_iter.next() {
            // screen col
            let mut col = 0u16;
            // text col
            let mut tx = ox;

            let mut glyph_iter = RopeGlyphIter::new(line);
            glyph_iter.set_offset(ox);
            glyph_iter.set_show_ctrl(widget.show_ctrl);

            'line: for d in glyph_iter {
                if d.len > 0 {
                    let mut style = style;
                    // text-styles
                    styles.clear();
                    state.styles_at((tx, ty), &mut styles);
                    for idx in &styles {
                        let Some(s) = widget.text_style.get(*idx) else {
                            panic!("invalid style nr: {}", idx);
                        };
                        style = style.patch(*s);
                    }
                    // selection
                    if selection.contains_pos((tx, ty)) {
                        style = style.patch(select_style);
                    };

                    let cell = buf.get_mut(area.x + col, area.y + row);
                    cell.set_symbol(d.glyph.as_ref());
                    cell.set_style(style);
                    col += 1;
                    if col >= area.width {
                        break 'line;
                    }

                    for _ in 1..d.len {
                        let cell = buf.get_mut(area.x + col, area.y + row);
                        cell.reset();
                        cell.set_style(style);
                        col += 1;
                        if col >= area.width {
                            break 'line;
                        }
                    }

                    tx += 1;
                }
            }
        }
    }
}

impl Default for TextAreaState {
    fn default() -> Self {
        let mut s = Self {
            focus: Default::default(),
            area: Default::default(),
            inner: Default::default(),
            mouse: Default::default(),
            value: TextAreaCore::default(),
            hscroll: Default::default(),
            non_exhaustive: NonExhaustive,
            vscroll: Default::default(),
            clip: Default::default(),
        };
        s.hscroll.set_max_offset(255);
        s.hscroll.set_overscroll_by(Some(16384));
        s
    }
}

impl HasFocusFlag for TextAreaState {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.area
    }

    // use secondary focus keys.
    fn primary_keys(&self) -> bool {
        false
    }
}

impl TextAreaState {
    /// New State.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear everything.
    #[inline]
    pub fn clear(&mut self) -> bool {
        self.value.clear()
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
    pub fn value(&self) -> String {
        self.value.value()
    }

    /// Set the text value.
    /// Resets all internal state.
    #[inline]
    pub fn set_value<S: AsRef<str>>(&mut self, s: S) {
        self.vscroll.set_offset(0);
        self.hscroll.set_offset(0);

        self.value.set_value(s);
    }

    /// Set the text value as a Rope.
    /// Resets all internal state.
    #[inline]
    pub fn set_rope(&mut self, s: Rope) {
        self.vscroll.set_offset(0);
        self.hscroll.set_offset(0);

        self.value.set_rope(s);
    }

    /// Borrow the rope
    #[inline]
    pub fn rope(&self) -> &Rope {
        self.value.rope()
    }

    /// Text value
    #[inline]
    pub fn text_slice(&self, range: impl Into<TextRange>) -> Option<RopeSlice<'_>> {
        self.value.text_slice(range.into())
    }

    /// Text as Bytes iterator.
    #[inline]
    pub fn bytes(&self) -> impl Iterator<Item = u8> + '_ {
        self.value.bytes()
    }

    /// Value as Bytes iterator.
    pub fn byte_slice<R>(&self, byte_range: R) -> RopeSlice<'_>
    where
        R: RangeBounds<usize>,
    {
        self.value.byte_slice(byte_range)
    }

    /// Text as Bytes iterator.
    #[inline]
    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.value.chars()
    }

    /// Value as Chars iterator.
    pub fn char_slice<R>(&self, char_range: R) -> RopeSlice<'_>
    where
        R: RangeBounds<usize>,
    {
        self.value.char_slice(char_range)
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
    pub fn set_tab_width(&mut self, tabs: u16) {
        self.value.set_tab_width(tabs);
    }

    /// Tab-width
    #[inline]
    pub fn tab_width(&self) -> u16 {
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

    /// Empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Line count.
    #[inline]
    pub fn line_len(&self) -> usize {
        self.value.len_lines()
    }

    /// Line width as grapheme count.
    #[inline]
    pub fn line_width(&self, n: usize) -> Option<usize> {
        self.value.line_width(n)
    }

    /// Line as RopeSlice.
    /// This contains the \n at the end.
    pub fn line_at(&self, n: usize) -> Option<RopeSlice<'_>> {
        self.value.line_at(n)
    }

    /// Iterate over text-lines, starting at offset.
    #[inline]
    pub fn lines_at(&self, n: usize) -> impl Iterator<Item = RopeSlice<'_>> {
        self.value.lines_at(n)
    }

    /// Iterator for the glyphs of a given line.
    /// Glyphs here a grapheme + display length.
    /// This covers multi-column graphemes as well as tabs (with varying width).
    /// This contains the \n at the end.
    pub fn line_glyphs(&self, n: usize) -> Option<RopeGlyphIter<'_>> {
        self.value.line_glyphs(n)
    }

    /// Grapheme iterator for a given line.
    /// This contains the \n at the end.
    #[inline]
    pub fn line_graphemes(&self, n: usize) -> Option<impl Iterator<Item = RopeSlice<'_>>> {
        self.value.line_graphemes(n)
    }

    /// Char iterator for a given line.
    /// This contains the \n at the end.
    #[inline]
    pub fn line_chars(&self, n: usize) -> Option<impl Iterator<Item = char> + '_> {
        self.value.line_chars(n)
    }

    /// Byte iterator for a given line.
    /// This contains the \n at the end.
    #[inline]
    pub fn line_bytes(&self, n: usize) -> Option<impl Iterator<Item = u8> + '_> {
        self.value.line_bytes(n)
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
    pub fn selected_value(&self) -> Option<RopeSlice<'_>> {
        self.value.text_slice(self.value.selection())
    }

    /// Clear all set styles.
    #[inline]
    pub fn clear_styles(&mut self) {
        self.value.clear_styles();
    }

    /// Add a style for a [TextRange]. The style-nr refers to one
    /// of the styles set with the widget.
    #[inline]
    pub fn add_style(&mut self, range: impl Into<TextRange>, style: usize) {
        self.value.add_style(range.into(), style);
    }

    /// Remove the exact TextRange and style.
    #[inline]
    pub fn remove_style(&mut self, range: impl Into<TextRange>, style: usize) {
        self.value.remove_style(range.into(), style);
    }

    /// All styles active at the given position.
    #[inline]
    pub fn styles_at(&self, pos: impl Into<TextPosition>, buf: &mut Vec<usize>) {
        self.value.styles_at(pos.into(), buf)
    }

    /// Convert a byte position to a text area position.
    /// Uses grapheme based column indexes.
    #[inline]
    pub fn byte_pos(&self, byte: usize) -> Option<TextPosition> {
        self.value.byte_pos(byte)
    }

    /// Convert a text area position to a byte range.
    /// Uses grapheme based column indexes.
    /// Returns (byte-start, byte-end) of the grapheme at the given position.
    #[inline]
    pub fn byte_at(&self, pos: impl Into<TextPosition>) -> Option<Range<usize>> {
        self.value.byte_at(pos.into())
    }

    /// Convert a char position to a text area position.
    /// Uses grapheme based column indexes.
    #[inline]
    pub fn char_pos(&self, byte: usize) -> Option<TextPosition> {
        self.value.char_pos(byte)
    }

    /// Convert a text area position to a char position.
    /// Uses grapheme based column indexes.
    #[inline]
    pub fn char_at(&self, pos: impl Into<TextPosition>) -> Option<usize> {
        self.value.char_at(pos.into())
    }

    /// Insert a character at the cursor position.
    /// Removes the selection and inserts the char.
    pub fn insert_char(&mut self, c: char) -> bool {
        if self.value.has_selection() {
            self.value.remove_range(self.value.selection());
        }
        if c == '\n' {
            self.value.insert_newline(self.value.cursor());
        } else if c == '\t' {
            self.value.insert_tab(self.value.cursor());
        } else {
            self.value.insert_char(self.value.cursor(), c);
        }
        self.scroll_cursor_to_visible();
        true
    }

    /// Insert a character at the cursor position.
    /// Removes the selection and inserts the char.
    pub fn insert_tab(&mut self) -> bool {
        if self.value.has_selection() {
            self.value.remove_range(self.value.selection());
        }
        self.value.insert_tab(self.value.cursor());
        self.scroll_cursor_to_visible();
        true
    }

    /// Insert text at the cursor position.
    /// Removes the selection and inserts the text.
    pub fn insert_str(&mut self, t: impl AsRef<str>) -> bool {
        let t = t.as_ref();
        if self.value.has_selection() {
            self.value.remove_range(self.value.selection());
        }
        self.value.insert_str(self.value.cursor(), t);
        self.scroll_cursor_to_visible();
        true
    }

    /// Insert a line break at the cursor position.
    pub fn insert_newline(&mut self) -> bool {
        if self.value.has_selection() {
            self.value.remove_range(self.value.selection());
        }
        self.value.insert_newline(self.value.cursor());
        self.scroll_cursor_to_visible();
        true
    }

    /// Deletes the given range.
    pub fn delete_range(&mut self, range: impl Into<TextRange>) -> bool {
        let range = range.into();
        if !range.is_empty() {
            self.value.remove_range(range);
            self.scroll_cursor_to_visible();
            true
        } else {
            false
        }
    }

    /// Deletes the next char or the current selection.
    /// Returns true if there was any real change.
    pub fn delete_next_char(&mut self) -> bool {
        if self.value.has_selection() {
            self.delete_range(self.selection())
        } else {
            let r = self.value.remove_next_char(self.value.cursor());
            let s = self.scroll_cursor_to_visible();
            r || s
        }
    }

    /// Deletes the previous char or the selection.
    /// Returns true if there was any real change.
    pub fn delete_prev_char(&mut self) -> bool {
        if self.value.has_selection() {
            self.delete_range(self.selection())
        } else {
            let r = self.value.remove_prev_char(self.value.cursor());
            let s = self.scroll_cursor_to_visible();
            r || s
        }
    }

    /// Find the start of the next word. Word is everything that is not whitespace.
    pub fn next_word_start(&self, pos: impl Into<TextPosition>) -> Option<TextPosition> {
        let mut char_pos = self.char_at(pos.into())?;

        let chars_after = self.value.char_slice(char_pos..);
        let mut it = chars_after.chars_at(0);
        loop {
            let Some(c) = it.next() else {
                break;
            };
            if !c.is_whitespace() {
                break;
            }
            char_pos += 1;
        }

        self.char_pos(char_pos)
    }

    /// Find the end of the next word. Skips whitespace first, then goes on
    /// until it finds the next whitespace.
    pub fn next_word_end(&self, pos: impl Into<TextPosition>) -> Option<TextPosition> {
        let mut char_pos = self.char_at(pos.into())?;

        let chars_after = self.value.char_slice(char_pos..);
        let mut it = chars_after.chars_at(0);
        let mut init = true;
        loop {
            let Some(c) = it.next() else {
                break;
            };

            if init {
                if !c.is_whitespace() {
                    init = false;
                }
            } else {
                if c.is_whitespace() {
                    break;
                }
            }

            char_pos += 1;
        }

        self.char_pos(char_pos)
    }

    /// Find prev word. Skips whitespace first.
    /// Attention: start/end are mirrored here compared to next_word_start/next_word_end,
    /// both return start<=end!
    pub fn prev_word_start(&self, pos: impl Into<TextPosition>) -> Option<TextPosition> {
        let mut char_pos = self.char_at(pos.into())?;

        let chars_before = self.value.char_slice(..char_pos);
        let mut it = chars_before.chars_at(chars_before.len_chars());
        let mut init = true;
        loop {
            let Some(c) = it.prev() else {
                break;
            };

            if init {
                if !c.is_whitespace() {
                    init = false;
                }
            } else {
                if c.is_whitespace() {
                    break;
                }
            }

            char_pos -= 1;
        }

        self.char_pos(char_pos)
    }

    /// Find the end of the previous word. Word is everything that is not whitespace.
    /// Attention: start/end are mirrored here compared to next_word_start/next_word_end,
    /// both return start<=end!
    pub fn prev_word_end(&self, pos: impl Into<TextPosition>) -> Option<TextPosition> {
        let mut char_pos = self.char_at(pos.into())?;

        let chars_before = self.value.char_slice(..char_pos);
        let mut it = chars_before.chars_at(chars_before.len_chars());
        loop {
            let Some(c) = it.prev() else {
                break;
            };
            if !c.is_whitespace() {
                break;
            }
            char_pos -= 1;
        }

        self.char_pos(char_pos)
    }

    /// Is the position at a word boundary?
    pub fn is_word_boundary(&self, pos: impl Into<TextPosition>) -> bool {
        let char_pos = self.char_at(pos.into()).expect("valid_selection");
        let mut it = self.char_slice(..).chars_at(char_pos);
        if let Some(c0) = it.prev() {
            it.next();
            if let Some(c1) = it.next() {
                c0.is_whitespace() && !c1.is_whitespace()
                    || !c0.is_whitespace() && c1.is_whitespace()
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Find the start of the word at pos.
    pub fn word_start(&self, pos: impl Into<TextPosition>) -> Option<TextPosition> {
        let mut char_pos = self.char_at(pos.into())?;

        let chars_before = self.value.char_slice(..char_pos);
        let mut it = chars_before.chars_at(chars_before.len_chars());
        loop {
            let Some(c) = it.prev() else {
                break;
            };
            if c.is_whitespace() {
                break;
            }
            char_pos -= 1;
        }

        self.char_pos(char_pos)
    }

    /// Find the end of the word at pos.
    pub fn word_end(&self, pos: impl Into<TextPosition>) -> Option<TextPosition> {
        let mut char_pos = self.char_at(pos.into())?;

        let chars_after = self.value.char_slice(char_pos..);
        let mut it = chars_after.chars_at(0);
        loop {
            let Some(c) = it.next() else {
                break;
            };
            if c.is_whitespace() {
                break;
            }
            char_pos += 1;
        }

        self.char_pos(char_pos)
    }

    /// Delete the next word. This alternates deleting the whitespace between words and
    /// the words themselves.
    pub fn delete_next_word(&mut self) -> bool {
        if self.value.has_selection() {
            self.delete_range(self.value.selection())
        } else {
            let cursor = self.value.cursor();
            let start = self.next_word_start(cursor).expect("valid_cursor");

            let range = if cursor == start {
                let end = self.next_word_end(cursor).expect("valid_cursor");
                TextRange::new(cursor, end)
            } else {
                TextRange::new(cursor, start)
            };

            self.delete_range(range)
        }
    }

    /// Deletes the previous word. This alternates deleting the whitespace between words and
    /// the words themselves.
    pub fn delete_prev_word(&mut self) -> bool {
        if self.value.has_selection() {
            self.delete_range(self.value.selection())
        } else {
            let cursor = self.value.cursor();
            let end = self.prev_word_end(cursor).expect("valid_cursor");

            let range = if cursor == end {
                let start = self.prev_word_start(cursor).expect("valid_cursor");
                TextRange::new(start, cursor)
            } else {
                TextRange::new(end, cursor)
            };

            self.delete_range(range)
        }
    }

    /// Move the cursor left. Scrolls the cursor to visible.
    /// Returns true if there was any real change.
    pub fn move_left(&mut self, n: usize, extend_selection: bool) -> bool {
        let mut cursor = self.value.cursor();

        if cursor.x == 0 {
            if cursor.y > 0 {
                cursor.y = cursor.y.saturating_sub(1);
                let Some(c_line_width) = self.value.line_width(cursor.y) else {
                    panic!(
                        "invalid_cursor: {:?} value {:?}",
                        (cursor.x, cursor.y),
                        self.value
                    );
                };
                cursor.x = c_line_width;
            }
        } else {
            cursor.x = cursor.x.saturating_sub(n);
        }

        self.value.set_move_col(Some(cursor.x));
        let c = self.value.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor right. Scrolls the cursor to visible.
    /// Returns true if there was any real change.
    pub fn move_right(&mut self, n: usize, extend_selection: bool) -> bool {
        let mut cursor = self.value.cursor();
        let Some(c_line_width) = self.value.line_width(cursor.y) else {
            panic!(
                "invalid_cursor: {:?} value {:?}",
                (cursor.x, cursor.y),
                self.value
            );
        };

        if cursor.x == c_line_width {
            if cursor.y + 1 < self.value.len_lines() {
                cursor.y += 1;
                cursor.x = 0;
            }
        } else {
            cursor.x = min(cursor.x + n, c_line_width)
        }

        self.value.set_move_col(Some(cursor.x));
        let c = self.value.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor up. Scrolls the cursor to visible.
    /// Returns true if there was any real change.
    pub fn move_up(&mut self, n: usize, extend_selection: bool) -> bool {
        let mut cursor = self.value.cursor();
        let Some(c_line_width) = self.value.line_width(cursor.y) else {
            panic!(
                "invalid_cursor: {:?} value {:?}",
                (cursor.x, cursor.y),
                self.value
            );
        };

        cursor.y = cursor.y.saturating_sub(n);
        if let Some(xx) = self.value.move_col() {
            cursor.x = min(xx, c_line_width);
        } else {
            cursor.x = min(cursor.x, c_line_width);
        }

        let c = self.value.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor down. Scrolls the cursor to visible.
    /// Returns true if there was any real change.
    pub fn move_down(&mut self, n: usize, extend_selection: bool) -> bool {
        let mut cursor = self.value.cursor();
        let Some(c_line_width) = self.value.line_width(cursor.y) else {
            panic!(
                "invalid_cursor: {:?} value {:?}",
                (cursor.x, cursor.y),
                self.value
            );
        };

        cursor.y = min(cursor.y + n, self.value.len_lines() - 1);
        if let Some(xx) = self.value.move_col() {
            cursor.x = min(xx, c_line_width);
        } else {
            cursor.x = min(cursor.x, c_line_width);
        }

        let c = self.value.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor to the start of the line.
    /// Scrolls the cursor to visible.
    /// Returns true if there was any real change.
    pub fn move_to_line_start(&mut self, extend_selection: bool) -> bool {
        let mut cursor = self.value.cursor();

        cursor.x = 'f: {
            if cursor.x > 0 {
                let Some(line) = self.value.line_graphemes(cursor.y) else {
                    panic!(
                        "invalid_cursor: {:?} value {:?}",
                        (cursor.x, cursor.y),
                        self.value
                    );
                };
                for (idx, ch) in line.enumerate() {
                    if ch.as_str() != Some(" ") {
                        if cursor.x != idx {
                            break 'f idx;
                        } else {
                            break 'f 0;
                        }
                    }
                }
            }
            0
        };

        self.value.set_move_col(Some(cursor.x));
        let c = self.value.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor to the end of the line. Scrolls to visible, if
    /// necessary.
    /// Returns true if there was any real change.
    pub fn move_to_line_end(&mut self, extend_selection: bool) -> bool {
        let mut cursor = self.value.cursor();
        let Some(c_line_width) = self.value.line_width(cursor.y) else {
            panic!(
                "invalid_cursor: {:?} value {:?}",
                (cursor.x, cursor.y),
                self.value
            );
        };

        cursor.x = c_line_width;

        self.value.set_move_col(Some(cursor.x));
        let c = self.value.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor to the document start.
    pub fn move_to_start(&mut self, extend_selection: bool) -> bool {
        let cursor = TextPosition::new(0, 0);

        let c = self.value.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor to the document end.
    pub fn move_to_end(&mut self, extend_selection: bool) -> bool {
        let len = self.value.len_lines();

        let cursor = TextPosition::new(0, len - 1);

        let c = self.value.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor to the start of the visible area.
    pub fn move_to_screen_start(&mut self, extend_selection: bool) -> bool {
        let (ox, oy) = self.offset();

        let cursor = TextPosition::new(ox, oy);

        let c = self.value.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor to the end of the visible area.
    pub fn move_to_screen_end(&mut self, extend_selection: bool) -> bool {
        let (ox, oy) = self.offset();
        let len = self.value.len_lines();

        let cursor = TextPosition::new(ox, min(oy + self.vertical_page() - 1, len - 1));

        let c = self.value.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor to the next word.
    pub fn move_to_next_word(&mut self, extend_selection: bool) -> bool {
        let cursor = self.value.cursor();

        let word = self.next_word_end(cursor).expect("valid_cursor");

        let c = self.value.set_cursor(word, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor to the previous word.
    pub fn move_to_prev_word(&mut self, extend_selection: bool) -> bool {
        let cursor = self.value.cursor();

        let word = self.prev_word_start(cursor).expect("valid_cursor");

        let c = self.value.set_cursor(word, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Converts from a widget relative screen coordinate to a line.
    /// It limits its result to a valid row.
    pub fn from_screen_row(&self, scy: i16) -> usize {
        let (_, oy) = self.offset();

        if scy < 0 {
            oy.saturating_sub((scy as isize).abs() as usize)
        } else if scy as u16 >= self.area.height {
            min(oy + scy as usize, self.line_len().saturating_sub(1))
        } else {
            let cy = oy + scy as usize;
            if cy < self.line_len() {
                cy
            } else {
                self.line_len().saturating_sub(1)
            }
        }
    }

    /// Converts from a widget relative screen coordinate to a grapheme index.
    /// It limits its result to a valid column.
    ///
    /// * row is a row-index into the value, not a screen-row. It can be calculated
    ///   with from_screen_row().
    /// * x is the relative screen position.
    pub fn from_screen_col(&self, row: usize, scx: i16) -> usize {
        let (ox, _) = self.offset();

        if scx < 0 {
            ox.saturating_sub((scx as isize).abs() as usize)
        } else if scx as u16 >= self.area.width {
            min(ox + scx as usize, self.line_width(row).expect("valid_row"))
        } else {
            let scx = scx as u16;

            let mut line = self.line_glyphs(row).expect("valid_row");
            line.set_offset(ox);

            let mut cx = 0;
            let mut testx = 0;
            for g in line {
                if scx >= testx && scx < testx + g.len as u16 {
                    break;
                }
                testx += g.len as u16;
                cx += 1;
            }

            min(ox + cx, self.line_width(row).expect("valid_row"))
        }
    }

    /// Converts a grapheme based position to a screen position
    /// relative to the widget area.
    pub fn to_screen_col(&self, pos: impl Into<TextPosition>) -> Option<u16> {
        let pos = pos.into();
        let (ox, _) = self.offset();

        let mut line = self.line_glyphs(pos.y)?;
        line.set_offset(ox);

        let sx = line.take(pos.x - ox).map(|v| v.len).sum::<usize>();

        Some(sx as u16)
    }

    /// Cursor position on the screen.
    pub fn screen_cursor(&self) -> Option<(u16, u16)> {
        if self.is_focused() {
            let cursor = self.value.cursor();
            let (ox, oy) = self.offset();

            if cursor.y < oy {
                None
            } else if cursor.y >= oy + self.inner.height as usize {
                None
            } else {
                let sy = cursor.y - oy;
                if cursor.x < ox {
                    None
                } else if cursor.x > ox + self.inner.width as usize {
                    None
                } else {
                    let sx = self.to_screen_col(cursor).expect("valid_cursor");

                    Some((self.inner.x + sx, self.inner.y + sy as u16))
                }
            }
        } else {
            None
        }
    }

    /// Set the cursor position from screen coordinates.
    ///
    /// The cursor positions are relative to the inner rect.
    /// They may be negative too, this allows setting the cursor
    /// to a position that is currently scrolled away.
    pub fn set_screen_cursor(&mut self, cursor: (i16, i16), extend_selection: bool) -> bool {
        let (scx, scy) = (cursor.0, cursor.1);

        let cy = self.from_screen_row(scy);
        let cx = self.from_screen_col(cy, scx);

        let c = self.set_cursor(TextPosition::new(cx, cy), extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Set the cursor position from screen coordinates,
    /// rounds the position to the next word start/end.
    ///
    /// The cursor positions are relative to the inner rect.
    /// They may be negative too, this allows setting the cursor
    /// to a position that is currently scrolled away.
    pub fn set_screen_cursor_words(&mut self, cursor: (i16, i16), extend_selection: bool) -> bool {
        let (scx, scy) = (cursor.0, cursor.1);
        let anchor = self.anchor();

        let cy = self.from_screen_row(scy);
        let cx = self.from_screen_col(cy, scx);
        let cursor = TextPosition::new(cx, cy);

        let cursor = if cursor < anchor {
            if let Some(word) = self.word_start(cursor) {
                word
            } else {
                cursor
            }
        } else {
            if let Some(word) = self.word_end(cursor) {
                word
            } else {
                cursor
            }
        };

        // extend anchor
        if !self.is_word_boundary(anchor) {
            if cursor < anchor {
                if let Some(anchor) = self.word_end(anchor) {
                    self.set_cursor(anchor, false);
                }
            } else {
                if let Some(anchor) = self.word_start(anchor) {
                    self.set_cursor(anchor, false);
                }
            }
        }

        let c = self.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }
}

impl TextAreaState {
    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is shorter than the length of the content by whatever fills the last page.
    /// This is the base for the scrollbar content_length.
    pub fn vertical_max_offset(&self) -> usize {
        self.vscroll.max_offset()
    }

    /// Current vertical offset.
    pub fn vertical_offset(&self) -> usize {
        self.vscroll.offset()
    }

    /// Vertical page-size at the current offset.
    pub fn vertical_page(&self) -> usize {
        self.vscroll.page_len()
    }

    /// Suggested scroll per scroll-event.
    pub fn vertical_scroll(&self) -> usize {
        self.vscroll.scroll_by()
    }

    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is currently set to usize::MAX.
    pub fn horizontal_max_offset(&self) -> usize {
        self.hscroll.max_offset()
    }

    /// Current horizontal offset.
    pub fn horizontal_offset(&self) -> usize {
        self.hscroll.offset()
    }

    /// Horizontal page-size at the current offset.
    pub fn horizontal_page(&self) -> usize {
        self.hscroll.page_len()
    }

    /// Suggested scroll per scroll-event.
    pub fn horizontal_scroll(&self) -> usize {
        self.hscroll.scroll_by()
    }

    /// Change the vertical offset.
    ///
    /// Due to overscroll it's possible that this is an invalid offset for the widget.
    /// The widget must deal with this situation.
    ///
    /// The widget returns true if the offset changed at all.
    #[allow(unused_assignments)]
    pub fn set_vertical_offset(&mut self, row_offset: usize) -> bool {
        self.vscroll.set_offset(row_offset)
    }

    /// Change the horizontal offset.
    ///
    /// Due to overscroll it's possible that this is an invalid offset for the widget.
    /// The widget must deal with this situation.
    ///
    /// The widget returns true if the offset changed at all.
    #[allow(unused_assignments)]
    pub fn set_horizontal_offset(&mut self, col_offset: usize) -> bool {
        self.hscroll.set_offset(col_offset)
    }

    /// Scroll to position.
    pub fn scroll_to_row(&mut self, pos: usize) -> bool {
        self.vscroll.set_offset(pos)
    }

    /// Scroll to position.
    pub fn scroll_to_col(&mut self, pos: usize) -> bool {
        self.hscroll.set_offset(pos)
    }

    /// Scrolling
    pub fn scroll_up(&mut self, delta: usize) -> bool {
        self.vscroll.scroll_up(delta)
    }

    /// Scrolling
    pub fn scroll_down(&mut self, delta: usize) -> bool {
        self.vscroll.scroll_down(delta)
    }

    /// Scrolling
    pub fn scroll_left(&mut self, delta: usize) -> bool {
        self.hscroll.scroll_left(delta)
    }

    /// Scrolling
    pub fn scroll_right(&mut self, delta: usize) -> bool {
        self.hscroll.scroll_right(delta)
    }
}

impl TextAreaState {
    /// Scroll that the cursor is visible.
    /// All move-fn do this automatically.
    pub fn scroll_cursor_to_visible(&mut self) -> bool {
        let old_offset = self.offset();

        let cursor = self.value.cursor();
        let (ox, oy) = self.offset();

        let noy = if cursor.y < oy {
            cursor.y
        } else if cursor.y >= oy + self.inner.height as usize {
            cursor.y.saturating_sub(self.inner.height as usize - 1)
        } else {
            oy
        };

        let nox = if cursor.x < ox {
            cursor.x
        } else if cursor.x >= ox + self.inner.width as usize {
            cursor.x.saturating_sub(self.inner.width as usize)
        } else {
            ox
        };

        self.set_offset((nox, noy));

        self.offset() != old_offset
    }
}

impl HandleEvent<crossterm::event::Event, Regular, TextOutcome> for TextAreaState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: Regular) -> TextOutcome {
        let mut r = if self.is_focused() {
            match event {
                ct_event!(key press c)
                | ct_event!(key press SHIFT-c)
                | ct_event!(key press CONTROL_ALT-c) => self.insert_char(*c).into(),
                ct_event!(keycode press Tab) => {
                    if !self.focus.gained() {
                        self.insert_tab().into()
                    } else {
                        TextOutcome::Unchanged
                    }
                }
                ct_event!(keycode press Enter) => self.insert_newline().into(),
                ct_event!(keycode press Backspace) => self.delete_prev_char().into(),
                ct_event!(keycode press Delete) => self.delete_next_char().into(),
                ct_event!(keycode press CONTROL-Backspace) => self.delete_prev_word().into(),
                ct_event!(keycode press CONTROL-Delete) => self.delete_next_word().into(),
                ct_event!(key press CONTROL-'y') => {
                    self.clip = self
                        .selected_value()
                        .map(|v| v.to_string())
                        .unwrap_or_default();
                    TextOutcome::Unchanged
                }
                ct_event!(key press CONTROL-'p') => {
                    self.insert_str(&self.clip.clone());
                    TextOutcome::Changed
                }
                ct_event!(key press CONTROL-'z') => self.value.undo().into(),
                ct_event!(key press CONTROL_SHIFT-'Z') => self.value.redo().into(),

                ct_event!(key release _)
                | ct_event!(key release SHIFT-_)
                | ct_event!(key release CONTROL_ALT-_)
                | ct_event!(keycode release Tab)
                | ct_event!(keycode release Enter)
                | ct_event!(keycode release Backspace)
                | ct_event!(keycode release Delete)
                | ct_event!(keycode release CONTROL-Backspace)
                | ct_event!(keycode release CONTROL-Delete)
                | ct_event!(key release  CONTROL-'y')
                | ct_event!(key release  CONTROL-'p') => TextOutcome::Unchanged,
                _ => TextOutcome::NotUsed,
            }
        } else {
            TextOutcome::NotUsed
        };
        // remap to TextChanged
        if r == TextOutcome::Changed {
            r = TextOutcome::TextChanged;
        }

        if r == TextOutcome::NotUsed {
            r = self.handle(event, ReadOnly);
        }
        r
    }
}

impl HandleEvent<crossterm::event::Event, ReadOnly, TextOutcome> for TextAreaState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: ReadOnly) -> TextOutcome {
        let mut r = if self.is_focused() {
            match event {
                ct_event!(keycode press Left) => self.move_left(1, false).into(),
                ct_event!(keycode press Right) => self.move_right(1, false).into(),
                ct_event!(keycode press Up) => self.move_up(1, false).into(),
                ct_event!(keycode press Down) => self.move_down(1, false).into(),
                ct_event!(keycode press PageUp) => self.move_up(self.vertical_page(), false).into(),
                ct_event!(keycode press PageDown) => {
                    self.move_down(self.vertical_page(), false).into()
                }
                ct_event!(keycode press Home) => self.move_to_line_start(false).into(),
                ct_event!(keycode press End) => self.move_to_line_end(false).into(),
                ct_event!(keycode press CONTROL-Left) => self.move_to_prev_word(false).into(),
                ct_event!(keycode press CONTROL-Right) => self.move_to_next_word(false).into(),
                ct_event!(keycode press CONTROL-Up) => false.into(),
                ct_event!(keycode press CONTROL-Down) => false.into(),
                ct_event!(keycode press CONTROL-PageUp) => self.move_to_screen_start(false).into(),
                ct_event!(keycode press CONTROL-PageDown) => self.move_to_screen_end(false).into(),
                ct_event!(keycode press CONTROL-Home) => self.move_to_start(false).into(),
                ct_event!(keycode press CONTROL-End) => self.move_to_end(false).into(),

                ct_event!(keycode press ALT-Left) => self.scroll_left(1).into(),
                ct_event!(keycode press ALT-Right) => self.scroll_right(1).into(),
                ct_event!(keycode press ALT-Up) => self.scroll_up(1).into(),
                ct_event!(keycode press ALT-Down) => self.scroll_down(1).into(),
                ct_event!(keycode press ALT-PageUp) => {
                    self.scroll_up(max(self.vertical_page() / 2, 1)).into()
                }
                ct_event!(keycode press ALT-PageDown) => {
                    self.scroll_down(max(self.vertical_page() / 2, 1)).into()
                }
                ct_event!(keycode press ALT_SHIFT-PageUp) => {
                    self.scroll_left(max(self.horizontal_page() / 5, 1)).into()
                }
                ct_event!(keycode press ALT_SHIFT-PageDown) => {
                    self.scroll_right(max(self.horizontal_page() / 5, 1)).into()
                }

                ct_event!(keycode press SHIFT-Left) => self.move_left(1, true).into(),
                ct_event!(keycode press SHIFT-Right) => self.move_right(1, true).into(),
                ct_event!(keycode press SHIFT-Up) => self.move_up(1, true).into(),
                ct_event!(keycode press SHIFT-Down) => self.move_down(1, true).into(),
                ct_event!(keycode press SHIFT-PageUp) => {
                    self.move_up(self.vertical_page(), true).into()
                }
                ct_event!(keycode press SHIFT-PageDown) => {
                    self.move_down(self.vertical_page(), true).into()
                }
                ct_event!(keycode press SHIFT-Home) => self.move_to_line_start(true).into(),
                ct_event!(keycode press SHIFT-End) => self.move_to_line_end(true).into(),
                ct_event!(keycode press CONTROL_SHIFT-Left) => self.move_to_prev_word(true).into(),
                ct_event!(keycode press CONTROL_SHIFT-Right) => self.move_to_next_word(true).into(),
                ct_event!(key press CONTROL-'a') => self.select_all().into(),

                ct_event!(keycode release Left)
                | ct_event!(keycode release Right)
                | ct_event!(keycode release Up)
                | ct_event!(keycode release Down)
                | ct_event!(keycode release PageUp)
                | ct_event!(keycode release PageDown)
                | ct_event!(keycode release Home)
                | ct_event!(keycode release End)
                | ct_event!(keycode release CONTROL-Left)
                | ct_event!(keycode release CONTROL-Right)
                | ct_event!(keycode release CONTROL-Up)
                | ct_event!(keycode release CONTROL-Down)
                | ct_event!(keycode release CONTROL-PageUp)
                | ct_event!(keycode release CONTROL-PageDown)
                | ct_event!(keycode release CONTROL-Home)
                | ct_event!(keycode release CONTROL-End)
                | ct_event!(keycode release ALT-Left)
                | ct_event!(keycode release ALT-Right)
                | ct_event!(keycode release ALT-Up)
                | ct_event!(keycode release ALT-Down)
                | ct_event!(keycode release ALT-PageUp)
                | ct_event!(keycode release ALT-PageDown)
                | ct_event!(keycode release ALT_SHIFT-PageUp)
                | ct_event!(keycode release ALT_SHIFT-PageDown)
                | ct_event!(keycode release SHIFT-Left)
                | ct_event!(keycode release SHIFT-Right)
                | ct_event!(keycode release SHIFT-Up)
                | ct_event!(keycode release SHIFT-Down)
                | ct_event!(keycode release SHIFT-PageUp)
                | ct_event!(keycode release SHIFT-PageDown)
                | ct_event!(keycode release SHIFT-Home)
                | ct_event!(keycode release SHIFT-End)
                | ct_event!(keycode release CONTROL_SHIFT-Left)
                | ct_event!(keycode release CONTROL_SHIFT-Right)
                | ct_event!(key release CONTROL-'a') => TextOutcome::Unchanged,
                _ => TextOutcome::NotUsed,
            }
        } else {
            TextOutcome::NotUsed
        };

        if r == TextOutcome::NotUsed {
            r = self.handle(event, MouseOnly);
        }
        r
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, TextOutcome> for TextAreaState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> TextOutcome {
        flow!(match event {
            ct_event!(mouse any for m) if self.mouse.drag(self.inner, m) => {
                let cx = m.column as i16 - self.inner.x as i16;
                let cy = m.row as i16 - self.inner.y as i16;
                self.set_screen_cursor((cx, cy), true).into()
            }
            ct_event!(mouse any for m) if self.mouse.drag2(self.inner, m, KeyModifiers::ALT) => {
                let cx = m.column as i16 - self.inner.x as i16;
                let cy = m.row as i16 - self.inner.y as i16;
                self.set_screen_cursor_words((cx, cy), true).into()
            }
            ct_event!(mouse any for m) if self.mouse.doubleclick(self.inner, m) => {
                let ty = self.from_screen_row(m.row as i16 - self.inner.y as i16);
                let tx = self.from_screen_col(ty, m.column as i16 - self.inner.x as i16);
                let test = TextPosition::new(tx, ty);
                if let Some(b0) = self.word_start(test) {
                    if let Some(b1) = self.word_end(test) {
                        self.set_selection(TextRange::new(b0, b1)).into()
                    } else {
                        TextOutcome::Unchanged
                    }
                } else {
                    TextOutcome::Unchanged
                }
            }
            ct_event!(mouse down Left for column,row) => {
                if self.inner.contains((*column, *row).into()) {
                    let cx = (column - self.inner.x) as i16;
                    let cy = (row - self.inner.y) as i16;
                    self.set_screen_cursor((cx, cy), false).into()
                } else {
                    TextOutcome::NotUsed
                }
            }
            ct_event!(mouse down CONTROL-Left for column,row) => {
                if self.inner.contains((*column, *row).into()) {
                    let cx = (column - self.inner.x) as i16;
                    let cy = (row - self.inner.y) as i16;
                    self.set_screen_cursor((cx, cy), true).into()
                } else {
                    TextOutcome::NotUsed
                }
            }
            ct_event!(mouse down ALT-Left for column,row) => {
                if self.inner.contains((*column, *row).into()) {
                    let cx = (column - self.inner.x) as i16;
                    let cy = (row - self.inner.y) as i16;
                    self.set_screen_cursor_words((cx, cy), true).into()
                } else {
                    TextOutcome::NotUsed
                }
            }
            _ => TextOutcome::NotUsed,
        });

        let r = match ScrollArea(self.inner, Some(&mut self.hscroll), Some(&mut self.vscroll))
            .handle(event, MouseOnly)
        {
            ScrollOutcome::Up(v) => self.scroll_up(v),
            ScrollOutcome::Down(v) => self.scroll_down(v),
            ScrollOutcome::Left(v) => self.scroll_left(v),
            ScrollOutcome::Right(v) => self.scroll_right(v),
            ScrollOutcome::VPos(v) => self.set_vertical_offset(v),
            ScrollOutcome::HPos(v) => self.set_horizontal_offset(v),
            _ => false,
        };
        if r {
            return TextOutcome::Changed;
        }

        TextOutcome::NotUsed
    }
}

/// Handle all events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events(
    state: &mut TextAreaState,
    focus: bool,
    event: &crossterm::event::Event,
) -> TextOutcome {
    state.focus.set(focus);
    state.handle(event, Regular)
}

/// Handle only navigation events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_readonly_events(
    state: &mut TextAreaState,
    focus: bool,
    event: &crossterm::event::Event,
) -> TextOutcome {
    state.focus.set(focus);
    state.handle(event, ReadOnly)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(
    state: &mut TextAreaState,
    event: &crossterm::event::Event,
) -> TextOutcome {
    state.handle(event, MouseOnly)
}
