//!
//! A text-area with text-styling abilities.
//! And undo + clipboard support.
//!

use crate::_private::NonExhaustive;
use crate::clipboard::{Clipboard, LocalClipboard};
use crate::event::{ReadOnly, TextOutcome};
use crate::grapheme::{Glyph, Grapheme};
use crate::text_core::TextCore;
use crate::text_store::text_rope::TextRope;
use crate::text_store::TextStore;
use crate::undo_buffer::{UndoBuffer, UndoEntry, UndoVec};
use crate::{ipos_type, upos_type, Cursor, TextError, TextPosition, TextRange};
use crossterm::event::KeyModifiers;
use rat_event::util::MouseFlags;
use rat_event::{ct_event, flow, HandleEvent, MouseOnly, Regular};
use rat_focus::{FocusFlag, HasFocusFlag, Navigation};
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::{layout_scroll, Scroll, ScrollArea, ScrollState};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{StatefulWidget, Style, Stylize};
use ratatui::widgets::{Block, StatefulWidgetRef, WidgetRef};
use ropey::{Rope, RopeSlice};
use std::borrow::Cow;
use std::cmp::{max, min};
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

    /// movement column
    pub move_col: Option<upos_type>,

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
            move_col: None,
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
    state.vscroll.set_max_offset(
        state
            .len_lines()
            .saturating_sub(state.inner.height as upos_type) as usize,
    );
    state.vscroll.set_page_len(state.inner.height as usize);

    widget.block.render_ref(area, buf);
    if let Some(hscroll) = widget.hscroll.as_ref() {
        hscroll.render_ref(hscroll_area, buf, &mut state.hscroll);
    }
    if let Some(vscroll) = widget.vscroll.as_ref() {
        vscroll.render_ref(vscroll_area, buf, &mut state.vscroll);
    }

    let inner = state.inner;

    if inner.width == 0 || inner.height == 0 {
        // noop
        return;
    }

    let select_style = if let Some(select_style) = widget.select_style {
        select_style
    } else {
        Style::default().on_yellow()
    };
    let style = widget.style;

    // set base style
    for y in inner.top()..inner.bottom() {
        for x in inner.left()..inner.right() {
            let cell = buf.get_mut(x, y);
            cell.reset();
            cell.set_style(style);
        }
    }

    if state.vscroll.offset() > state.value.len_lines() as usize {
        return;
    }

    let (ox, oy) = state.offset();
    let page_rows = (oy as upos_type)
        ..min(
            oy as upos_type + inner.height as upos_type,
            state.value.len_lines(),
        );
    let page_bytes = state
        .bytes_at_range(TextRange::new((0, page_rows.start), (0, page_rows.end)))
        .expect("valid_rows");
    let selection = state.selection();
    let mut styles = Vec::new();

    let glyph_iter = state
        .value
        .glyphs(page_rows.clone(), ox as u16, inner.width)
        .expect("valid_offset");

    for g in glyph_iter {
        if g.screen_width() > 0 {
            let mut style = style;
            // text-styles
            styles.clear();
            state
                .value
                .styles_at_page(page_bytes.clone(), g.text_bytes().start, &mut styles);
            for style_nr in &styles {
                if let Some(s) = widget.text_style.get(*style_nr) {
                    style = style.patch(*s);
                }
            }
            // selection
            if selection.contains_pos(g.pos()) {
                style = style.patch(select_style);
            };

            // relative screen-pos of the glyph
            let screen_pos = g.screen_pos();

            // render glyph
            let cell = buf.get_mut(inner.x + screen_pos.0, inner.y + screen_pos.1);
            cell.set_symbol(g.glyph());
            cell.set_style(style);
            // clear the reset of the cells to avoid interferences.
            for d in 1..g.screen_width() {
                let cell = buf.get_mut(inner.x + screen_pos.0 + d, inner.y + screen_pos.1);
                cell.reset();
                cell.set_style(style);
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
            value: TextCore::new(
                Some(Box::new(UndoVec::new(99))),
                Some(Box::new(LocalClipboard::new())),
            ),
            hscroll: Default::default(),
            non_exhaustive: NonExhaustive,
            vscroll: Default::default(),
            move_col: None,
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

    /// Show control characters.
    #[inline]
    pub fn set_show_ctrl(&mut self, show_ctrl: bool) {
        self.value.set_glyph_ctrl(show_ctrl);
    }

    /// Show control characters.
    pub fn show_ctrl(&self) -> bool {
        self.value.glyph_ctrl()
    }

    /// Extra column information for cursor movement.
    ///
    /// The cursor position is capped to the current line length, so if you
    /// move up one row, you might end at a position left of the current column.
    /// If you move up once more you want to return to the original position.
    /// That's what is stored here.
    #[inline]
    pub fn set_move_col(&mut self, col: Option<upos_type>) {
        self.move_col = col;
    }

    /// Extra column information for cursor movement.
    #[inline]
    pub fn move_col(&mut self) -> Option<upos_type> {
        self.move_col
    }
}

impl TextAreaState {
    /// Clipboard
    pub fn set_clipboard(&mut self, clip: Option<impl Clipboard + 'static>) {
        match clip {
            None => self.value.set_clipboard(None),
            Some(v) => self.value.set_clipboard(Some(Box::new(v))),
        }
    }

    /// Clipboard
    pub fn clipboard(&self) -> Option<&dyn Clipboard> {
        self.value.clipboard()
    }

    /// Copy to internal buffer
    pub fn copy_to_clip(&mut self) -> bool {
        let Some(clip) = self.value.clipboard() else {
            return false;
        };

        _ = clip.set_string(self.selected_text().as_ref());
        false
    }

    /// Cut to internal buffer
    pub fn cut_to_clip(&mut self) -> bool {
        let Some(clip) = self.value.clipboard() else {
            return false;
        };

        match clip.set_string(self.selected_text().as_ref()) {
            Ok(_) => self
                .delete_range(self.selection())
                .expect("valid_selection"),
            Err(_) => false,
        }
    }

    /// Paste from internal buffer.
    pub fn paste_from_clip(&mut self) -> bool {
        let Some(clip) = self.value.clipboard() else {
            return false;
        };

        if let Ok(text) = clip.get_string() {
            self.insert_str(text)
        } else {
            false
        }
    }
}

impl TextAreaState {
    /// Set undo buffer.
    pub fn set_undo_buffer(&mut self, undo: Option<impl UndoBuffer + 'static>) {
        match undo {
            None => self.value.set_undo_buffer(None),
            Some(v) => self.value.set_undo_buffer(Some(Box::new(v))),
        }
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
    pub fn replay_log(&mut self, replay: &[UndoEntry]) {
        self.value.replay_log(replay)
    }

    /// Undo operation
    pub fn undo(&mut self) -> bool {
        self.value.undo()
    }

    /// Redo operation
    pub fn redo(&mut self) -> bool {
        self.value.redo()
    }
}

impl TextAreaState {
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

    /// Add a style for a [TextRange]. The style-nr refers to one
    /// of the styles set with the widget.
    #[inline]
    pub fn add_range_style(&mut self, range: TextRange, style: usize) -> Result<(), TextError> {
        let r = self.value.bytes_at_range(range)?;
        self.value.add_style(r, style);
        Ok(())
    }

    /// Remove the exact TextRange and style.
    #[inline]
    pub fn remove_style(&mut self, range: Range<usize>, style: usize) {
        self.value.remove_style(range.into(), style);
    }

    /// Remove the exact TextRange and style.
    #[inline]
    pub fn remove_range_style(&mut self, range: TextRange, style: usize) -> Result<(), TextError> {
        let r = self.value.bytes_at_range(range)?;
        self.value.remove_style(r, style);
        Ok(())
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

    /// List of all styles.
    #[inline]
    pub fn styles(&self) -> impl Iterator<Item = (Range<usize>, usize)> + '_ {
        self.value.styles().expect("styles")
    }
}

impl TextAreaState {
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
    pub fn set_selection(
        &mut self,
        anchor: impl Into<TextPosition>,
        cursor: impl Into<TextPosition>,
    ) -> bool {
        self.value.set_selection(anchor.into(), cursor.into())
    }

    /// Select all.
    #[inline]
    pub fn select_all(&mut self) -> bool {
        self.value.select_all()
    }

    /// Selection.
    #[inline]
    pub fn selected_text(&self) -> Cow<'_, str> {
        self.value
            .str_slice(self.value.selection())
            .expect("valid_selection")
    }
}

impl TextAreaState {
    /// Empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Borrow the rope
    #[inline]
    pub fn rope(&self) -> &Rope {
        self.value.text().rope()
    }

    /// Text value
    #[inline]
    pub fn text(&self) -> String {
        self.value.text().string()
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
    pub fn len_lines(&self) -> upos_type {
        self.value.len_lines()
    }

    /// Line width as grapheme count.
    #[inline]
    pub fn line_width(&self, row: upos_type) -> Result<upos_type, TextError> {
        self.value.line_width(row)
    }

    /// Line as RopeSlice.
    /// This contains the \n at the end.
    #[inline]
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

    // Iterator for the glyphs of the lines in range.
    /// Glyphs here a grapheme + display length.
    #[inline]
    pub fn glyphs(
        &self,
        rows: Range<upos_type>,
        screen_offset: u16,
        screen_width: u16,
    ) -> Result<impl Iterator<Item = Glyph<'_>>, TextError> {
        self.value.glyphs(rows, screen_offset, screen_width)
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

    /// Get a cursor over all the text with the current position set at pos.
    #[inline]
    pub fn text_graphemes(
        &self,
        pos: TextPosition,
    ) -> Result<impl Iterator<Item = Grapheme<'_>> + Cursor, TextError> {
        self.value.text_graphemes(pos)
    }

    /// Get a cursor over the text-range the current position set at pos.
    #[inline]
    pub fn graphemes(
        &self,
        range: TextRange,
        pos: TextPosition,
    ) -> Result<impl Iterator<Item = Grapheme<'_>> + Cursor, TextError> {
        self.value.graphemes(range, pos)
    }

    /// Grapheme position to byte position.
    /// This is the (start,end) position of the single grapheme after pos.
    #[inline]
    pub fn byte_at(&self, pos: TextPosition) -> Result<Range<usize>, TextError> {
        self.value.byte_at(pos)
    }

    /// Grapheme range to byte range.
    #[inline]
    pub fn bytes_at_range(&self, range: TextRange) -> Result<Range<usize>, TextError> {
        self.value.bytes_at_range(range)
    }

    /// Byte position to grapheme position.
    /// Returns the position that contains the given byte index.
    #[inline]
    pub fn byte_pos(&self, byte: usize) -> Result<TextPosition, TextError> {
        self.value.byte_pos(byte)
    }

    /// Byte range to grapheme range.
    #[inline]
    pub fn byte_range(&self, bytes: Range<usize>) -> Result<TextRange, TextError> {
        self.value.byte_range(bytes)
    }
}

impl TextAreaState {
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

    /// Insert a character at the cursor position.
    /// Removes the selection and inserts the char.
    pub fn insert_char(&mut self, c: char) -> bool {
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
        true
    }

    /// Insert a character at the cursor position.
    /// Removes the selection and inserts the char.
    pub fn insert_tab(&mut self) -> bool {
        if self.value.has_selection() {
            self.value
                .remove_str_range(self.value.selection())
                .expect("valid_selection");
        }
        self.value
            .insert_tab(self.value.cursor())
            .expect("valid_cursor");
        self.scroll_cursor_to_visible();
        true
    }

    /// Insert text at the cursor position.
    /// Removes the selection and inserts the text.
    pub fn insert_str(&mut self, t: impl AsRef<str>) -> bool {
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
        true
    }

    /// Insert a line break at the cursor position.
    pub fn insert_newline(&mut self) -> bool {
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
            for g in self.value.line_graphemes(pos.y - 1).expect("valid_cursor") {
                if g == " " || g == "\t" {
                    blanks.push_str(g.grapheme());
                } else {
                    break;
                }
            }
            if blanks.len() > 0 {
                self.value.insert_str(pos, &blanks).expect("valid_cursor");
            }
        }

        self.scroll_cursor_to_visible();
        true
    }

    /// Deletes the given range.
    pub fn delete_range(&mut self, range: impl Into<TextRange>) -> Result<bool, TextError> {
        let range = range.into();
        if !range.is_empty() {
            self.value.remove_str_range(range)?;
            self.scroll_cursor_to_visible();
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl TextAreaState {
    /// Duplicates the selection or the current line.
    /// Returns true if there was any real change.
    pub fn duplicate_text(&mut self) -> bool {
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
                true
            } else {
                false
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
            true
        }
    }

    /// Deletes the current line.
    /// Returns true if there was any real change.
    pub fn delete_line(&mut self) -> bool {
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
    pub fn delete_next_char(&mut self) -> bool {
        if self.value.has_selection() {
            self.delete_range(self.selection())
                .expect("valid_selection")
        } else {
            let r = self
                .value
                .remove_next_char(self.value.cursor())
                .expect("valid_cursor");
            let s = self.scroll_cursor_to_visible();

            r || s
        }
    }

    /// Deletes the previous char or the selection.
    /// Returns true if there was any real change.
    pub fn delete_prev_char(&mut self) -> bool {
        if self.value.has_selection() {
            self.delete_range(self.selection())
                .expect("valid_selection")
        } else {
            let r = self
                .value
                .remove_prev_char(self.value.cursor())
                .expect("valid_cursor");
            let s = self.scroll_cursor_to_visible();

            r || s
        }
    }

    /// Find the start of the next word. If the position is at the start
    /// or inside a word, the same position is returned.
    pub fn next_word_start(&self, pos: impl Into<TextPosition>) -> Result<TextPosition, TextError> {
        self.value.next_word_start(pos.into())
    }

    /// Find the end of the next word. Skips whitespace first, then goes on
    /// until it finds the next whitespace.
    pub fn next_word_end(&self, pos: impl Into<TextPosition>) -> Result<TextPosition, TextError> {
        self.value.next_word_end(pos.into())
    }

    /// Find the start of the prev word. Skips whitespace first, then goes on
    /// until it finds the next whitespace.
    ///
    /// Attention: start/end are mirrored here compared to next_word_start/next_word_end,
    /// both return start<=end!
    pub fn prev_word_start(&self, pos: impl Into<TextPosition>) -> Result<TextPosition, TextError> {
        self.value.prev_word_start(pos.into())
    }

    /// Find the end of the previous word. Word is everything that is not whitespace.
    /// Attention: start/end are mirrored here compared to next_word_start/next_word_end,
    /// both return start<=end!
    pub fn prev_word_end(&self, pos: impl Into<TextPosition>) -> Result<TextPosition, TextError> {
        self.value.prev_word_end(pos.into())
    }

    /// Is the position at a word boundary?
    pub fn is_word_boundary(&self, pos: impl Into<TextPosition>) -> Result<bool, TextError> {
        self.value.is_word_boundary(pos.into())
    }

    /// Find the start of the word at pos.
    /// Returns pos if the position is not inside a word.
    pub fn word_start(&self, pos: impl Into<TextPosition>) -> Result<TextPosition, TextError> {
        self.value.word_start(pos.into())
    }

    /// Find the end of the word at pos.
    /// Returns pos if the position is not inside a word.
    pub fn word_end(&self, pos: impl Into<TextPosition>) -> Result<TextPosition, TextError> {
        self.value.word_end(pos.into())
    }

    /// Delete the next word. This alternates deleting the whitespace between words and
    /// the words themselves.
    pub fn delete_next_word(&mut self) -> bool {
        if self.value.has_selection() {
            self.delete_range(self.value.selection())
                .expect("valid_selection")
        } else {
            let cursor = self.value.cursor();

            let start = self.next_word_start(cursor).expect("valid_cursor");
            if start != cursor {
                self.delete_range(cursor..start).expect("valid_range")
            } else {
                let end = self.next_word_end(cursor).expect("valid_cursor");
                self.delete_range(cursor..end).expect("valid_range")
            }
        }
    }

    /// Deletes the previous word. This alternates deleting the whitespace
    /// between words and the words themselves.
    pub fn delete_prev_word(&mut self) -> bool {
        if self.value.has_selection() {
            self.delete_range(self.value.selection())
                .expect("valid_selection")
        } else {
            let cursor = self.value.cursor();

            // delete to beginning of line?
            let till_line_start = if cursor.x != 0 {
                self.value
                    .graphemes(TextRange::new((0, cursor.y), cursor), cursor)
                    .expect("valid_cursor")
                    .rev_cursor()
                    .find(|v| !v.is_whitespace())
                    .is_none()
            } else {
                false
            };

            if till_line_start {
                self.delete_range(TextRange::new((0, cursor.y), cursor))
                    .expect("valid_cursor")
            } else {
                let end = self.prev_word_end(cursor).expect("valid_cursor");
                if end != cursor {
                    self.delete_range(end..cursor).expect("valid_cursor")
                } else {
                    let start = self.prev_word_start(cursor).expect("valid_cursor");
                    self.delete_range(start..cursor).expect("valid_cursor")
                }
            }
        }
    }

    /// Move the cursor left. Scrolls the cursor to visible.
    /// Returns true if there was any real change.
    pub fn move_left(&mut self, n: upos_type, extend_selection: bool) -> bool {
        let mut cursor = self.value.cursor();

        if cursor.x == 0 {
            if cursor.y > 0 {
                cursor.y = cursor.y.saturating_sub(1);
                cursor.x = self.value.line_width(cursor.y).expect("valid_cursor");
            }
        } else {
            cursor.x = cursor.x.saturating_sub(n);
        }

        self.set_move_col(Some(cursor.x));
        let c = self.value.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor right. Scrolls the cursor to visible.
    /// Returns true if there was any real change.
    pub fn move_right(&mut self, n: upos_type, extend_selection: bool) -> bool {
        let mut cursor = self.value.cursor();

        let c_line_width = self.value.line_width(cursor.y).expect("valid_cursor");
        if cursor.x == c_line_width {
            if cursor.y + 1 < self.value.len_lines() {
                cursor.y += 1;
                cursor.x = 0;
            }
        } else {
            cursor.x = min(cursor.x + n, c_line_width)
        }

        self.set_move_col(Some(cursor.x));
        let c = self.value.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor up. Scrolls the cursor to visible.
    /// Returns true if there was any real change.
    pub fn move_up(&mut self, n: upos_type, extend_selection: bool) -> bool {
        let mut cursor = self.value.cursor();

        cursor.y = cursor.y.saturating_sub(n);
        let c_line_width = self.value.line_width(cursor.y).expect("valid_cursor");
        if let Some(move_col) = self.move_col() {
            cursor.x = min(move_col, c_line_width);
        } else {
            cursor.x = min(cursor.x, c_line_width);
        }

        let c = self.value.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor down. Scrolls the cursor to visible.
    /// Returns true if there was any real change.
    pub fn move_down(&mut self, n: upos_type, extend_selection: bool) -> bool {
        let mut cursor = self.value.cursor();

        cursor.y = min(cursor.y + n, self.value.len_lines() - 1);
        let c_line_width = self.value.line_width(cursor.y).expect("valid_cursor");
        if let Some(move_col) = self.move_col() {
            cursor.x = min(move_col, c_line_width);
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
            for (idx, g) in self
                .value
                .line_graphemes(cursor.y)
                .expect("valid_cursor")
                .enumerate()
            {
                if g != " " && g != "\t" {
                    if cursor.x != idx as upos_type {
                        break 'f idx as upos_type;
                    } else {
                        break 'f 0;
                    }
                }
            }
            0
        };

        self.set_move_col(Some(cursor.x));
        let c = self.value.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor to the end of the line. Scrolls to visible, if
    /// necessary.
    /// Returns true if there was any real change.
    pub fn move_to_line_end(&mut self, extend_selection: bool) -> bool {
        let mut cursor = self.value.cursor();

        cursor.x = self.value.line_width(cursor.y).expect("valid_cursor");

        self.set_move_col(Some(cursor.x));
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

        let cursor = TextPosition::new(ox as upos_type, oy as upos_type);

        let c = self.value.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor to the end of the visible area.
    pub fn move_to_screen_end(&mut self, extend_selection: bool) -> bool {
        let (ox, oy) = self.offset();
        let (ox, oy) = (ox as upos_type, oy as upos_type);
        let len = self.value.len_lines();

        let cursor =
            TextPosition::new(ox, min(oy + self.vertical_page() as upos_type - 1, len - 1));

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
}

impl TextAreaState {
    /// Converts from a widget relative screen coordinate to a line.
    /// It limits its result to a valid row.
    pub fn screen_to_row(&self, scy: i16) -> upos_type {
        let (_, oy) = self.offset();
        let oy = oy as upos_type;

        if scy < 0 {
            oy.saturating_sub((scy as ipos_type).abs() as upos_type)
        } else if scy as u16 >= self.inner.height {
            min(oy + scy as upos_type, self.len_lines().saturating_sub(1))
        } else {
            let scy = oy + scy as upos_type;
            let len = self.len_lines();
            if scy < len {
                scy
            } else {
                len.saturating_sub(1)
            }
        }
    }

    /// Converts from a widget relative screen coordinate to a grapheme index.
    /// It limits its result to a valid column.
    ///
    /// * row is a row-index into the value, not a screen-row. It can be calculated
    ///   with screen_to_row().
    /// * x is the relative screen position.
    pub fn screen_to_col(&self, row: upos_type, scx: i16) -> upos_type {
        let (ox, _) = self.offset();
        let ox = ox as upos_type;

        if scx < 0 {
            ox.saturating_sub((scx as ipos_type).abs() as upos_type)
        } else if scx as u16 >= self.inner.width {
            min(
                ox + scx as upos_type,
                self.line_width(row).expect("valid_row"),
            )
        } else {
            let scx = scx as u16;

            let line = self
                .glyphs(row..row + 1, ox as u16, self.inner.width)
                .expect("valid_row");

            let mut col = ox;
            for g in line {
                if scx < g.screen_pos().0 + g.screen_width() {
                    break;
                }
                col = g.pos().x + 1;
            }
            col
        }
    }

    /// Converts a grapheme based position to a screen position
    /// relative to the widget area.
    pub fn col_to_screen(&self, pos: impl Into<TextPosition>) -> Result<u16, TextError> {
        let pos = pos.into();
        let (ox, _) = self.offset();

        if pos.x < ox as upos_type {
            return Ok(0);
        }

        let line = self.glyphs(pos.y..pos.y + 1, ox as u16, self.inner.width)?;
        let mut screen_x = 0;
        for g in line {
            if g.pos().x == pos.x {
                break;
            }
            screen_x = g.screen_pos().0 + g.screen_width();
        }
        Ok(screen_x)
    }

    /// Cursor position on the screen.
    pub fn screen_cursor(&self) -> Option<(u16, u16)> {
        if self.is_focused() {
            let cursor = self.value.cursor();
            let (ox, oy) = self.offset();
            let (ox, oy) = (ox as upos_type, oy as upos_type);

            if cursor.y < oy {
                None
            } else if cursor.y >= oy + self.inner.height as upos_type {
                None
            } else {
                let sy = cursor.y - oy;
                if cursor.x < ox {
                    None
                } else if cursor.x > ox + self.inner.width as upos_type {
                    None
                } else {
                    let sx = self.col_to_screen(cursor).expect("valid_cursor");

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

        let cy = self.screen_to_row(scy);
        let cx = self.screen_to_col(cy, scx);

        let c = self
            .value
            .set_cursor(TextPosition::new(cx, cy), extend_selection);
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

        let cy = self.screen_to_row(scy);
        let cx = self.screen_to_col(cy, scx);
        let cursor = TextPosition::new(cx, cy);

        let cursor = if cursor < anchor {
            self.word_start(cursor).expect("valid_cursor")
        } else {
            self.word_end(cursor).expect("valid_cursor")
        };

        // extend anchor
        if !self.is_word_boundary(anchor).expect("valid_anchor") {
            if cursor < anchor {
                self.set_cursor(self.word_end(anchor).expect("valid_anchor"), false);
            } else {
                self.set_cursor(self.word_start(anchor).expect("valid_anchor"), false);
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
        let (ox, oy) = (ox as upos_type, oy as upos_type);

        let noy = if cursor.y < oy {
            cursor.y
        } else if cursor.y >= oy + self.inner.height as upos_type {
            cursor.y.saturating_sub(self.inner.height as upos_type - 1)
        } else {
            oy
        };

        let nox = if cursor.x < ox {
            cursor.x
        } else if cursor.x >= ox + self.inner.width as upos_type {
            cursor.x.saturating_sub(self.inner.width as upos_type)
        } else {
            ox
        };

        self.set_offset((nox as usize, noy as usize));

        self.offset() != old_offset
    }
}

impl HandleEvent<crossterm::event::Event, Regular, TextOutcome> for TextAreaState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: Regular) -> TextOutcome {
        // small helper ...
        fn tc(r: bool) -> TextOutcome {
            if r {
                TextOutcome::TextChanged
            } else {
                TextOutcome::Unchanged
            }
        }

        let mut r = if self.is_focused() {
            match event {
                ct_event!(key press c)
                | ct_event!(key press SHIFT-c)
                | ct_event!(key press CONTROL_ALT-c) => tc(self.insert_char(*c)),
                ct_event!(keycode press Tab) => {
                    // ignore tab from focus
                    tc(if !self.focus.gained() {
                        self.insert_tab()
                    } else {
                        false
                    })
                }
                ct_event!(keycode press Enter) => tc(self.insert_newline()),
                ct_event!(keycode press Backspace) => tc(self.delete_prev_char()),
                ct_event!(keycode press Delete) => tc(self.delete_next_char()),
                ct_event!(keycode press CONTROL-Backspace)
                | ct_event!(keycode press ALT-Backspace) => tc(self.delete_prev_word()),
                ct_event!(keycode press CONTROL-Delete) => tc(self.delete_next_word()),
                ct_event!(key press CONTROL-'c') => tc(self.copy_to_clip()),
                ct_event!(key press CONTROL-'x') => tc(self.cut_to_clip()),
                ct_event!(key press CONTROL-'v') => tc(self.paste_from_clip()),
                ct_event!(key press CONTROL-'d') => tc(self.duplicate_text()),
                ct_event!(key press CONTROL-'y') => tc(self.delete_line()),
                ct_event!(key press CONTROL-'z') => tc(self.value.undo()),
                ct_event!(key press CONTROL_SHIFT-'Z') => tc(self.value.redo()),

                ct_event!(key release _)
                | ct_event!(key release SHIFT-_)
                | ct_event!(key release CONTROL_ALT-_)
                | ct_event!(keycode release Tab)
                | ct_event!(keycode release Enter)
                | ct_event!(keycode release Backspace)
                | ct_event!(keycode release Delete)
                | ct_event!(keycode release CONTROL-Backspace)
                | ct_event!(keycode release ALT-Backspace)
                | ct_event!(keycode release CONTROL-Delete)
                | ct_event!(key release CONTROL-'c')
                | ct_event!(key release CONTROL-'x')
                | ct_event!(key release CONTROL-'v')
                | ct_event!(key release CONTROL-'d')
                | ct_event!(key release CONTROL-'y')
                | ct_event!(key release CONTROL-'z')
                | ct_event!(key release CONTROL_SHIFT-'Z') => TextOutcome::Unchanged,
                _ => TextOutcome::Continue,
            }
        } else {
            TextOutcome::Continue
        };
        if r == TextOutcome::Continue {
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
                ct_event!(keycode press PageUp) => self
                    .move_up(self.vertical_page() as upos_type, false)
                    .into(),
                ct_event!(keycode press PageDown) => self
                    .move_down(self.vertical_page() as upos_type, false)
                    .into(),
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
                    self.move_up(self.vertical_page() as upos_type, true).into()
                }
                ct_event!(keycode press SHIFT-PageDown) => self
                    .move_down(self.vertical_page() as upos_type, true)
                    .into(),
                ct_event!(keycode press SHIFT-Home) => self.move_to_line_start(true).into(),
                ct_event!(keycode press SHIFT-End) => self.move_to_line_end(true).into(),
                ct_event!(keycode press CONTROL_SHIFT-Left) => self.move_to_prev_word(true).into(),
                ct_event!(keycode press CONTROL_SHIFT-Right) => self.move_to_next_word(true).into(),
                ct_event!(keycode press CONTROL_SHIFT-Home) => self.move_to_start(true).into(),
                ct_event!(keycode press CONTROL_SHIFT-End) => self.move_to_end(true).into(),
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
                | ct_event!(keycode release CONTROL_SHIFT-Home)
                | ct_event!(keycode release CONTROL_SHIFT-End)
                | ct_event!(key release CONTROL-'a') => TextOutcome::Unchanged,
                _ => TextOutcome::Continue,
            }
        } else {
            TextOutcome::Continue
        };

        if r == TextOutcome::Continue {
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
                let ty = self.screen_to_row(m.row as i16 - self.inner.y as i16);
                let tx = self.screen_to_col(ty, m.column as i16 - self.inner.x as i16);
                let test = TextPosition::new(tx, ty);
                let start = self.word_start(test).expect("valid_pos");
                let end = self.word_end(test).expect("valid_pos");
                self.set_selection(start, end).into()
            }
            ct_event!(mouse down Left for column,row) => {
                if self.inner.contains((*column, *row).into()) {
                    let cx = (column - self.inner.x) as i16;
                    let cy = (row - self.inner.y) as i16;
                    self.set_screen_cursor((cx, cy), false).into()
                } else {
                    TextOutcome::Continue
                }
            }
            ct_event!(mouse down CONTROL-Left for column,row) => {
                if self.inner.contains((*column, *row).into()) {
                    let cx = (column - self.inner.x) as i16;
                    let cy = (row - self.inner.y) as i16;
                    self.set_screen_cursor((cx, cy), true).into()
                } else {
                    TextOutcome::Continue
                }
            }
            ct_event!(mouse down ALT-Left for column,row) => {
                if self.inner.contains((*column, *row).into()) {
                    let cx = (column - self.inner.x) as i16;
                    let cy = (row - self.inner.y) as i16;
                    self.set_screen_cursor_words((cx, cy), true).into()
                } else {
                    TextOutcome::Continue
                }
            }
            _ => TextOutcome::Continue,
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

        TextOutcome::Continue
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
