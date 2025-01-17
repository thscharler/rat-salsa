//!
//! A text-area widget with text-styling abilities.
//! And undo + clipboard support.
//!

use crate::_private::NonExhaustive;
use crate::clipboard::{global_clipboard, Clipboard};
use crate::event::{ReadOnly, TextOutcome};
use crate::grapheme::{Glyph, Grapheme};
use crate::text_core::TextCore;
use crate::text_store::text_rope::TextRope;
use crate::text_store::TextStore;
use crate::undo_buffer::{UndoBuffer, UndoEntry, UndoVec};
use crate::{
    ipos_type, upos_type, Cursor, HasScreenCursor, TextError, TextPosition, TextRange, TextStyle,
};
use crossterm::event::KeyModifiers;
use rat_event::util::MouseFlags;
use rat_event::{ct_event, flow, HandleEvent, MouseOnly, Regular};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_reloc::{relocate_area, relocate_dark_offset, RelocatableState};
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::{Scroll, ScrollArea, ScrollAreaState, ScrollState};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::StatefulWidgetRef;
use ratatui::widgets::{Block, StatefulWidget};
use ropey::Rope;
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
/// For more interactions you can use [screen_to_col](TextAreaState::screen_to_col),
/// and [try_col_to_screen](TextAreaState::try_col_to_screen). They calculate everything,
/// even in the presence of more complex graphemes and those double-width emojis.
///
/// # Stateful
/// This widget implements [`StatefulWidget`], you can use it with
/// [`TextAreaState`] to handle common actions.
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

/// State & event handling.
#[derive(Debug)]
pub struct TextAreaState {
    /// The whole area with block.
    /// __read only__ renewed with each render.
    pub area: Rect,
    /// Area inside a possible block.
    /// __read only__ renewed with each render.
    pub inner: Rect,

    /// Horizontal scroll
    /// __read+write__
    pub hscroll: ScrollState,
    /// Vertical offset
    /// __read+write__
    pub vscroll: ScrollState,
    /// Dark offset due to clipping.
    /// __read only__ secondary offset due to clipping.
    pub dark_offset: (u16, u16),

    /// Text edit core
    pub value: TextCore<TextRope>,

    /// movement column
    pub move_col: Option<upos_type>,
    /// auto indent active
    pub auto_indent: bool,
    /// quote selection active
    pub auto_quote: bool,

    /// Current focus state.
    pub focus: FocusFlag,

    /// Mouse selection in progress.
    /// __read+write__
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
            auto_indent: self.auto_indent,
            auto_quote: self.auto_quote,
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
            dark_offset: (0, 0),
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
    pub fn styles_opt(self, styles: Option<TextStyle>) -> Self {
        if let Some(styles) = styles {
            self.styles(styles)
        } else {
            self
        }
    }

    /// Set the combined style.
    #[inline]
    pub fn styles(mut self, style: TextStyle) -> Self {
        self.style = style.style;
        if style.focus.is_some() {
            self.focus_style = style.focus;
        }
        if style.select.is_some() {
            self.select_style = style.select;
        }
        if style.block.is_some() {
            self.block = style.block;
        }
        if let Some(styles) = style.scroll {
            self.hscroll = self.hscroll.map(|v| v.styles(styles));
            self.vscroll = self.vscroll.map(|v| v.styles(styles));
        }
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

#[cfg(feature = "unstable-widget-ref")]
impl<'a> StatefulWidgetRef for TextArea<'a> {
    type State = TextAreaState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_text_area(self, area, buf, state);
    }
}

impl StatefulWidget for TextArea<'_> {
    type State = TextAreaState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_text_area(&self, area, buf, state);
    }
}

fn render_text_area(
    widget: &TextArea<'_>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut TextAreaState,
) {
    state.area = area;

    let sa = ScrollArea::new()
        .block(widget.block.as_ref())
        .h_scroll(widget.hscroll.as_ref())
        .v_scroll(widget.vscroll.as_ref());
    state.inner = sa.inner(area, Some(&state.hscroll), Some(&state.vscroll));

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

    let inner = state.inner;

    if inner.width == 0 || inner.height == 0 {
        // noop
        return;
    }

    let select_style = if let Some(select_style) = widget.select_style {
        select_style
    } else {
        Style::default().black().on_yellow()
    };
    let style = widget.style;

    // set base style
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.reset();
                cell.set_style(style);
            }
        }
    }

    sa.render(
        area,
        buf,
        &mut ScrollAreaState::new()
            .h_scroll(&mut state.hscroll)
            .v_scroll(&mut state.vscroll),
    );

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
        .try_bytes_at_range(TextRange::new((0, page_rows.start), (0, page_rows.end)))
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
            if let Some(cell) = buf.cell_mut((inner.x + screen_pos.0, inner.y + screen_pos.1)) {
                cell.set_symbol(g.glyph());
                cell.set_style(style);
            }
            // clear the reset of the cells to avoid interferences.
            for d in 1..g.screen_width() {
                if let Some(cell) =
                    buf.cell_mut((inner.x + screen_pos.0 + d, inner.y + screen_pos.1))
                {
                    cell.reset();
                    cell.set_style(style);
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
            value: TextCore::new(Some(Box::new(UndoVec::new(99))), Some(global_clipboard())),
            hscroll: Default::default(),
            non_exhaustive: NonExhaustive,
            vscroll: Default::default(),
            move_col: None,
            auto_indent: true,
            auto_quote: true,
            dark_offset: (0, 0),
        };
        s.hscroll.set_max_offset(255);
        s.hscroll.set_overscroll_by(Some(16384));
        s
    }
}

impl HasFocus for TextAreaState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.append_leaf(self);
    }

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

    /// Sets auto-indent on new-line.
    #[inline]
    pub fn set_auto_indent(&mut self, indent: bool) {
        self.auto_indent = indent;
    }

    /// Activates 'add quotes to selection'.
    #[inline]
    pub fn set_auto_quote(&mut self, quote: bool) {
        self.auto_quote = quote;
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
    #[inline]
    pub fn set_clipboard(&mut self, clip: Option<impl Clipboard + 'static>) {
        match clip {
            None => self.value.set_clipboard(None),
            Some(v) => self.value.set_clipboard(Some(Box::new(v))),
        }
    }

    /// Clipboard
    #[inline]
    pub fn clipboard(&self) -> Option<&dyn Clipboard> {
        self.value.clipboard()
    }

    /// Copy to internal buffer
    #[inline]
    pub fn copy_to_clip(&mut self) -> bool {
        let Some(clip) = self.value.clipboard() else {
            return false;
        };

        _ = clip.set_string(self.selected_text().as_ref());
        false
    }

    /// Cut to internal buffer
    #[inline]
    pub fn cut_to_clip(&mut self) -> bool {
        let Some(clip) = self.value.clipboard() else {
            return false;
        };

        match clip.set_string(self.selected_text().as_ref()) {
            Ok(_) => self.delete_range(self.selection()),
            Err(_) => false,
        }
    }

    /// Paste from internal buffer.
    #[inline]
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
    #[inline]
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

    /// Begin a sequence of changes that should be undone in one go.
    #[inline]
    pub fn begin_undo_seq(&mut self) {
        self.value.begin_undo_seq()
    }

    /// End a sequence of changes that should be undone in one go.
    #[inline]
    pub fn end_undo_seq(&mut self) {
        self.value.end_undo_seq()
    }

    /// Get all recent replay recordings.
    #[inline]
    pub fn recent_replay_log(&mut self) -> Vec<UndoEntry> {
        self.value.recent_replay_log()
    }

    /// Apply the replay recording.
    #[inline]
    pub fn replay_log(&mut self, replay: &[UndoEntry]) {
        self.value.replay_log(replay)
    }

    /// Undo operation
    #[inline]
    pub fn undo(&mut self) -> bool {
        self.value.undo()
    }

    /// Redo operation
    #[inline]
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
        self.value.add_style(range, style);
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
        self.value.remove_style(range, style);
    }

    /// Remove the exact TextRange and style.
    #[inline]
    pub fn remove_range_style(&mut self, range: TextRange, style: usize) -> Result<(), TextError> {
        let r = self.value.bytes_at_range(range)?;
        self.value.remove_style(r, style);
        Ok(())
    }

    /// Find all styles that touch the given range.
    pub fn styles_in(&self, range: Range<usize>, buf: &mut Vec<(Range<usize>, usize)>) {
        self.value.styles_in(range, buf)
    }

    /// All styles active at the given position.
    #[inline]
    pub fn styles_at(&self, byte_pos: usize, buf: &mut Vec<(Range<usize>, usize)>) {
        self.value.styles_at(byte_pos, buf)
    }

    /// Check if the given style applies at the position and
    /// return the complete range for the style.
    #[inline]
    pub fn style_match(&self, byte_pos: usize, style: usize) -> Option<Range<usize>> {
        self.value.style_match(byte_pos, style)
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

    /// Text slice as `Cow<str>`. Uses a byte range.
    #[inline]
    pub fn str_slice_byte(&self, range: Range<usize>) -> Cow<'_, str> {
        self.value.str_slice_byte(range).expect("valid_range")
    }

    /// Text slice as `Cow<str>`. Uses a byte range.
    #[inline]
    pub fn try_str_slice_byte(&self, range: Range<usize>) -> Result<Cow<'_, str>, TextError> {
        self.value.str_slice_byte(range)
    }

    /// Text slice as `Cow<str>`
    #[inline]
    pub fn str_slice(&self, range: impl Into<TextRange>) -> Cow<'_, str> {
        self.value.str_slice(range.into()).expect("valid_range")
    }

    /// Text slice as `Cow<str>`
    #[inline]
    pub fn try_str_slice(&self, range: impl Into<TextRange>) -> Result<Cow<'_, str>, TextError> {
        self.value.str_slice(range.into())
    }

    /// Line count.
    #[inline]
    pub fn len_lines(&self) -> upos_type {
        self.value.len_lines()
    }

    /// Line width as grapheme count.
    #[inline]
    pub fn line_width(&self, row: upos_type) -> upos_type {
        self.value.line_width(row).expect("valid_row")
    }

    /// Line width as grapheme count.
    #[inline]
    pub fn try_line_width(&self, row: upos_type) -> Result<upos_type, TextError> {
        self.value.line_width(row)
    }

    /// Line as RopeSlice.
    /// This contains the \n at the end.
    #[inline]
    pub fn line_at(&self, row: upos_type) -> Cow<'_, str> {
        self.value.line_at(row).expect("valid_row")
    }

    /// Line as RopeSlice.
    /// This contains the \n at the end.
    #[inline]
    pub fn try_line_at(&self, row: upos_type) -> Result<Cow<'_, str>, TextError> {
        self.value.line_at(row)
    }

    /// Iterate over text-lines, starting at offset.
    #[inline]
    pub fn lines_at(&self, row: upos_type) -> impl Iterator<Item = Cow<'_, str>> {
        self.value.lines_at(row).expect("valid_row")
    }

    /// Iterate over text-lines, starting at offset.
    #[inline]
    pub fn try_lines_at(
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
    ) -> impl Iterator<Item = Glyph<'_>> {
        self.value
            .glyphs(rows, screen_offset, screen_width)
            .expect("valid_rows")
    }

    // Iterator for the glyphs of the lines in range.
    /// Glyphs here a grapheme + display length.
    #[inline]
    pub fn try_glyphs(
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
    pub fn line_graphemes(&self, row: upos_type) -> impl Iterator<Item = Grapheme<'_>> {
        self.value.line_graphemes(row).expect("valid_row")
    }

    /// Grapheme iterator for a given line.
    /// This contains the \n at the end.
    #[inline]
    pub fn try_line_graphemes(
        &self,
        row: upos_type,
    ) -> Result<impl Iterator<Item = Grapheme<'_>>, TextError> {
        self.value.line_graphemes(row)
    }

    /// Get a cursor over all the text with the current position set at pos.
    #[inline]
    pub fn text_graphemes(&self, pos: TextPosition) -> impl Cursor<Item = Grapheme<'_>> {
        self.value.text_graphemes(pos).expect("valid_pos")
    }

    /// Get a cursor over all the text with the current position set at pos.
    #[inline]
    pub fn try_text_graphemes(
        &self,
        pos: TextPosition,
    ) -> Result<impl Cursor<Item = Grapheme<'_>>, TextError> {
        self.value.text_graphemes(pos)
    }

    /// Get a cursor over the text-range the current position set at pos.
    #[inline]
    pub fn graphemes(
        &self,
        range: TextRange,
        pos: TextPosition,
    ) -> impl Cursor<Item = Grapheme<'_>> {
        self.value.graphemes(range, pos).expect("valid_args")
    }

    /// Get a cursor over the text-range the current position set at pos.
    #[inline]
    pub fn try_graphemes(
        &self,
        range: TextRange,
        pos: TextPosition,
    ) -> Result<impl Cursor<Item = Grapheme<'_>>, TextError> {
        self.value.graphemes(range, pos)
    }

    /// Grapheme position to byte position.
    /// This is the (start,end) position of the single grapheme after pos.
    #[inline]
    pub fn byte_at(&self, pos: TextPosition) -> Range<usize> {
        self.value.byte_at(pos).expect("valid_pos")
    }

    /// Grapheme position to byte position.
    /// This is the (start,end) position of the single grapheme after pos.
    #[inline]
    pub fn try_byte_at(&self, pos: TextPosition) -> Result<Range<usize>, TextError> {
        self.value.byte_at(pos)
    }

    /// Grapheme range to byte range.
    #[inline]
    pub fn try_bytes_at_range(&self, range: TextRange) -> Result<Range<usize>, TextError> {
        self.value.bytes_at_range(range)
    }

    /// Grapheme range to byte range.
    #[inline]
    pub fn bytes_at_range(&self, range: TextRange) -> Range<usize> {
        self.value.bytes_at_range(range).expect("valid_range")
    }

    /// Byte position to grapheme position.
    /// Returns the position that contains the given byte index.
    #[inline]
    pub fn byte_pos(&self, byte: usize) -> TextPosition {
        self.value.byte_pos(byte).expect("valid_pos")
    }

    /// Byte position to grapheme position.
    /// Returns the position that contains the given byte index.
    #[inline]
    pub fn try_byte_pos(&self, byte: usize) -> Result<TextPosition, TextError> {
        self.value.byte_pos(byte)
    }

    /// Byte range to grapheme range.
    #[inline]
    pub fn byte_range(&self, bytes: Range<usize>) -> TextRange {
        self.value.byte_range(bytes).expect("valid_range")
    }

    /// Byte range to grapheme range.
    #[inline]
    pub fn try_byte_range(&self, bytes: Range<usize>) -> Result<TextRange, TextError> {
        self.value.byte_range(bytes)
    }
}

impl TextAreaState {
    /// Clear everything.
    #[inline]
    pub fn clear(&mut self) -> bool {
        if !self.is_empty() {
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
    ///
    /// This insert makes no special actions when encountering
    /// a new-line or tab. Use insert_newline and insert_tab for
    /// this.
    pub fn insert_char(&mut self, c: char) -> bool {
        let mut insert = true;
        if self.has_selection() {
            if self.auto_quote
                && (c == '\''
                    || c == '"'
                    || c == '`'
                    || c == '<'
                    || c == '['
                    || c == '('
                    || c == '{')
            {
                self.value
                    .insert_quotes(self.selection(), c)
                    .expect("valid_selection");
                insert = false;
            } else {
                self.value
                    .remove_str_range(self.selection())
                    .expect("valid_selection");
            }
        }

        if insert {
            if c == '\n' {
                self.value
                    .insert_newline(self.cursor())
                    .expect("valid_cursor");
            } else if c == '\t' {
                self.value.insert_tab(self.cursor()).expect("valid_cursor");
            } else {
                self.value
                    .insert_char(self.cursor(), c)
                    .expect("valid_cursor");
            }
        }

        self.scroll_cursor_to_visible();

        true
    }

    /// Inserts tab at the current position. This respects the
    /// tab-width set.
    ///
    /// If there is a text-selection the text-rows will be indented instead.
    /// This can be deactivated with auto_indent=false.
    pub fn insert_tab(&mut self) -> bool {
        if self.has_selection() {
            if self.auto_indent {
                let sel = self.selection();
                let indent = " ".repeat(self.tab_width() as usize);

                self.value.begin_undo_seq();
                for r in sel.start.y..=sel.end.y {
                    self.value
                        .insert_str(TextPosition::new(0, r), &indent)
                        .expect("valid_row");
                }
                self.value.end_undo_seq();

                true
            } else {
                false
            }
        } else {
            self.value.insert_tab(self.cursor()).expect("valid_cursor");
            self.scroll_cursor_to_visible();

            true
        }
    }

    /// Unindents the selected text by tab-width. If there is no
    /// selection this does nothing.
    ///
    /// This can be deactivated with auto_indent=false.
    pub fn insert_backtab(&mut self) -> bool {
        let sel = self.selection();

        self.value.begin_undo_seq();
        for r in sel.start.y..=sel.end.y {
            let mut idx = 0;
            let g_it = self
                .value
                .graphemes(TextRange::new((0, r), (0, r + 1)), TextPosition::new(0, r))
                .expect("valid_range")
                .take(self.tab_width() as usize);
            for g in g_it {
                if g != " " && g != "\t" {
                    break;
                }
                idx += 1;
            }

            self.value
                .remove_str_range(TextRange::new((0, r), (idx, r)))
                .expect("valid_range");
        }
        self.value.end_undo_seq();

        true
    }

    /// Insert text at the cursor position.
    /// Removes the selection and inserts the text.
    pub fn insert_str(&mut self, t: impl AsRef<str>) -> bool {
        let t = t.as_ref();
        if self.has_selection() {
            self.value
                .remove_str_range(self.selection())
                .expect("valid_selection");
        }
        self.value
            .insert_str(self.cursor(), t)
            .expect("valid_cursor");
        self.scroll_cursor_to_visible();
        true
    }

    /// Insert a line break at the cursor position.
    ///
    /// If auto_indent is set the new line starts with the same
    /// indent as the current.
    pub fn insert_newline(&mut self) -> bool {
        if self.has_selection() {
            self.value
                .remove_str_range(self.selection())
                .expect("valid_selection");
        }
        self.value
            .insert_newline(self.cursor())
            .expect("valid_cursor");

        // insert leading spaces
        if self.auto_indent {
            let cursor = self.cursor();
            if cursor.y > 0 {
                let mut blanks = String::new();
                for g in self.line_graphemes(cursor.y - 1) {
                    if g == " " || g == "\t" {
                        blanks.push_str(g.grapheme());
                    } else {
                        break;
                    }
                }
                if !blanks.is_empty() {
                    self.value
                        .insert_str(cursor, &blanks)
                        .expect("valid_cursor");
                }
            }
        }

        self.scroll_cursor_to_visible();
        true
    }

    /// Deletes the given range.
    #[inline]
    pub fn delete_range(&mut self, range: impl Into<TextRange>) -> bool {
        self.try_delete_range(range).expect("valid_range")
    }

    /// Deletes the given range.
    #[inline]
    pub fn try_delete_range(&mut self, range: impl Into<TextRange>) -> Result<bool, TextError> {
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
        if self.has_selection() {
            let sel_range = self.selection();
            if !sel_range.is_empty() {
                let v = self.str_slice(sel_range).to_string();
                self.value
                    .insert_str(sel_range.end, &v)
                    .expect("valid_selection");
                true
            } else {
                false
            }
        } else {
            let pos = self.cursor();
            let row_range = TextRange::new((0, pos.y), (0, pos.y + 1));
            let v = self.str_slice(row_range).to_string();
            self.value
                .insert_str(row_range.start, &v)
                .expect("valid_cursor");
            true
        }
    }

    /// Deletes the current line.
    /// Returns true if there was any real change.
    pub fn delete_line(&mut self) -> bool {
        let pos = self.cursor();
        if pos.y + 1 < self.len_lines() {
            self.delete_range(TextRange::new((0, pos.y), (0, pos.y + 1)))
        } else {
            let width = self.line_width(pos.y);
            self.delete_range(TextRange::new((0, pos.y), (width, pos.y)))
        }
    }

    /// Deletes the next char or the current selection.
    /// Returns true if there was any real change.
    pub fn delete_next_char(&mut self) -> bool {
        if self.has_selection() {
            self.delete_range(self.selection())
        } else {
            let r = self
                .value
                .remove_next_char(self.cursor())
                .expect("valid_cursor");
            let s = self.scroll_cursor_to_visible();

            r || s
        }
    }

    /// Deletes the previous char or the selection.
    /// Returns true if there was any real change.
    pub fn delete_prev_char(&mut self) -> bool {
        if self.has_selection() {
            self.delete_range(self.selection())
        } else {
            let r = self
                .value
                .remove_prev_char(self.cursor())
                .expect("valid_cursor");
            let s = self.scroll_cursor_to_visible();

            r || s
        }
    }

    /// Find the start of the next word. If the position is at the start
    /// or inside a word, the same position is returned.
    pub fn next_word_start(&self, pos: impl Into<TextPosition>) -> TextPosition {
        self.value.next_word_start(pos.into()).expect("valid_pos")
    }

    /// Find the start of the next word. If the position is at the start
    /// or inside a word, the same position is returned.
    pub fn try_next_word_start(
        &self,
        pos: impl Into<TextPosition>,
    ) -> Result<TextPosition, TextError> {
        self.value.next_word_start(pos.into())
    }

    /// Find the end of the next word. Skips whitespace first, then goes on
    /// until it finds the next whitespace.
    pub fn next_word_end(&self, pos: impl Into<TextPosition>) -> TextPosition {
        self.value.next_word_end(pos.into()).expect("valid_pos")
    }

    /// Find the end of the next word. Skips whitespace first, then goes on
    /// until it finds the next whitespace.
    pub fn try_next_word_end(
        &self,
        pos: impl Into<TextPosition>,
    ) -> Result<TextPosition, TextError> {
        self.value.next_word_end(pos.into())
    }

    /// Find the start of the prev word. Skips whitespace first, then goes on
    /// until it finds the next whitespace.
    ///
    /// Attention: start/end are mirrored here compared to next_word_start/next_word_end,
    /// both return start<=end!
    pub fn prev_word_start(&self, pos: impl Into<TextPosition>) -> TextPosition {
        self.value.prev_word_start(pos.into()).expect("valid_pos")
    }

    /// Find the start of the prev word. Skips whitespace first, then goes on
    /// until it finds the next whitespace.
    ///
    /// Attention: start/end are mirrored here compared to next_word_start/next_word_end,
    /// both return start<=end!
    pub fn try_prev_word_start(
        &self,
        pos: impl Into<TextPosition>,
    ) -> Result<TextPosition, TextError> {
        self.value.prev_word_start(pos.into())
    }

    /// Find the end of the previous word. Word is everything that is not whitespace.
    /// Attention: start/end are mirrored here compared to next_word_start/next_word_end,
    /// both return start<=end!
    pub fn prev_word_end(&self, pos: impl Into<TextPosition>) -> TextPosition {
        self.value.prev_word_end(pos.into()).expect("valid_pos")
    }

    /// Find the end of the previous word. Word is everything that is not whitespace.
    /// Attention: start/end are mirrored here compared to next_word_start/next_word_end,
    /// both return start<=end!
    pub fn try_prev_word_end(
        &self,
        pos: impl Into<TextPosition>,
    ) -> Result<TextPosition, TextError> {
        self.value.prev_word_end(pos.into())
    }

    /// Is the position at a word boundary?
    pub fn is_word_boundary(&self, pos: impl Into<TextPosition>) -> bool {
        self.value.is_word_boundary(pos.into()).expect("valid_pos")
    }

    /// Is the position at a word boundary?
    pub fn try_is_word_boundary(&self, pos: impl Into<TextPosition>) -> Result<bool, TextError> {
        self.value.is_word_boundary(pos.into())
    }

    /// Find the start of the word at pos.
    /// Returns pos if the position is not inside a word.
    pub fn word_start(&self, pos: impl Into<TextPosition>) -> TextPosition {
        self.value.word_start(pos.into()).expect("valid_pos")
    }

    /// Find the start of the word at pos.
    /// Returns pos if the position is not inside a word.
    pub fn try_word_start(&self, pos: impl Into<TextPosition>) -> Result<TextPosition, TextError> {
        self.value.word_start(pos.into())
    }

    /// Find the end of the word at pos.
    /// Returns pos if the position is not inside a word.
    pub fn word_end(&self, pos: impl Into<TextPosition>) -> TextPosition {
        self.value.word_end(pos.into()).expect("valid_pos")
    }

    /// Find the end of the word at pos.
    /// Returns pos if the position is not inside a word.
    pub fn try_word_end(&self, pos: impl Into<TextPosition>) -> Result<TextPosition, TextError> {
        self.value.word_end(pos.into())
    }

    /// Delete the next word. This alternates deleting the whitespace between words and
    /// the words themselves.
    pub fn delete_next_word(&mut self) -> bool {
        if self.has_selection() {
            self.delete_range(self.selection())
        } else {
            let cursor = self.cursor();

            let start = self.next_word_start(cursor);
            if start != cursor {
                self.delete_range(cursor..start)
            } else {
                let end = self.next_word_end(cursor);
                self.delete_range(cursor..end)
            }
        }
    }

    /// Deletes the previous word. This alternates deleting the whitespace
    /// between words and the words themselves.
    pub fn delete_prev_word(&mut self) -> bool {
        if self.has_selection() {
            self.delete_range(self.selection())
        } else {
            let cursor = self.cursor();

            // delete to beginning of line?
            let till_line_start = if cursor.x != 0 {
                self.graphemes(TextRange::new((0, cursor.y), cursor), cursor)
                    .rev_cursor()
                    .all(|v| v.is_whitespace())
            } else {
                false
            };

            if till_line_start {
                self.delete_range(TextRange::new((0, cursor.y), cursor))
            } else {
                let end = self.prev_word_end(cursor);
                if end != cursor {
                    self.delete_range(end..cursor)
                } else {
                    let start = self.prev_word_start(cursor);
                    self.delete_range(start..cursor)
                }
            }
        }
    }

    /// Move the cursor left. Scrolls the cursor to visible.
    /// Returns true if there was any real change.
    pub fn move_left(&mut self, n: upos_type, extend_selection: bool) -> bool {
        let mut cursor = self.cursor();

        if cursor.x == 0 {
            if cursor.y > 0 {
                cursor.y = cursor.y.saturating_sub(1);
                cursor.x = self.line_width(cursor.y);
            }
        } else {
            cursor.x = cursor.x.saturating_sub(n);
        }

        self.set_move_col(Some(cursor.x));
        let c = self.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor right. Scrolls the cursor to visible.
    /// Returns true if there was any real change.
    pub fn move_right(&mut self, n: upos_type, extend_selection: bool) -> bool {
        let mut cursor = self.cursor();

        let c_line_width = self.line_width(cursor.y);
        if cursor.x == c_line_width {
            if cursor.y + 1 < self.len_lines() {
                cursor.y += 1;
                cursor.x = 0;
            }
        } else {
            cursor.x = min(cursor.x + n, c_line_width)
        }

        self.set_move_col(Some(cursor.x));
        let c = self.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor up. Scrolls the cursor to visible.
    /// Returns true if there was any real change.
    pub fn move_up(&mut self, n: upos_type, extend_selection: bool) -> bool {
        let mut cursor = self.cursor();

        cursor.y = cursor.y.saturating_sub(n);
        let c_line_width = self.line_width(cursor.y);
        if let Some(move_col) = self.move_col() {
            cursor.x = min(move_col, c_line_width);
        } else {
            cursor.x = min(cursor.x, c_line_width);
        }

        let c = self.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor down. Scrolls the cursor to visible.
    /// Returns true if there was any real change.
    pub fn move_down(&mut self, n: upos_type, extend_selection: bool) -> bool {
        let mut cursor = self.cursor();

        cursor.y = min(cursor.y + n, self.len_lines() - 1);
        let c_line_width = self.line_width(cursor.y);
        if let Some(move_col) = self.move_col() {
            cursor.x = min(move_col, c_line_width);
        } else {
            cursor.x = min(cursor.x, c_line_width);
        }

        let c = self.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor to the start of the line.
    /// Scrolls the cursor to visible.
    /// Returns true if there was any real change.
    pub fn move_to_line_start(&mut self, extend_selection: bool) -> bool {
        let mut cursor = self.cursor();

        cursor.x = 'f: {
            for (idx, g) in self.line_graphemes(cursor.y).enumerate() {
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
        let c = self.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor to the end of the line. Scrolls to visible, if
    /// necessary.
    /// Returns true if there was any real change.
    pub fn move_to_line_end(&mut self, extend_selection: bool) -> bool {
        let mut cursor = self.cursor();

        cursor.x = self.line_width(cursor.y);

        self.set_move_col(Some(cursor.x));
        let c = self.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor to the document start.
    pub fn move_to_start(&mut self, extend_selection: bool) -> bool {
        let cursor = TextPosition::new(0, 0);

        let c = self.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor to the document end.
    pub fn move_to_end(&mut self, extend_selection: bool) -> bool {
        let len = self.len_lines();

        let cursor = TextPosition::new(0, len - 1);

        let c = self.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor to the start of the visible area.
    pub fn move_to_screen_start(&mut self, extend_selection: bool) -> bool {
        let (ox, oy) = self.offset();

        let cursor = TextPosition::new(ox as upos_type, oy as upos_type);

        let c = self.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor to the end of the visible area.
    pub fn move_to_screen_end(&mut self, extend_selection: bool) -> bool {
        let (ox, oy) = self.offset();
        let (ox, oy) = (ox as upos_type, oy as upos_type);
        let len = self.len_lines();

        let cursor =
            TextPosition::new(ox, min(oy + self.vertical_page() as upos_type - 1, len - 1));

        let c = self.set_cursor(cursor, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor to the next word.
    pub fn move_to_next_word(&mut self, extend_selection: bool) -> bool {
        let cursor = self.cursor();

        let word = self.next_word_end(cursor);

        let c = self.set_cursor(word, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move the cursor to the previous word.
    pub fn move_to_prev_word(&mut self, extend_selection: bool) -> bool {
        let cursor = self.cursor();

        let word = self.prev_word_start(cursor);

        let c = self.set_cursor(word, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }
}

impl HasScreenCursor for TextAreaState {
    /// Cursor position on the screen.
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        if self.is_focused() {
            let cursor = self.cursor();
            let (ox, oy) = self.offset();
            let (ox, oy) = (ox as upos_type, oy as upos_type);

            if cursor.y < oy {
                None
            } else if cursor.y >= oy + (self.inner.height + self.dark_offset.1) as upos_type {
                None
            } else {
                if cursor.x < ox {
                    None
                } else if cursor.x > ox + (self.inner.width + self.dark_offset.0) as upos_type {
                    None
                } else {
                    let sy = self.row_to_screen(cursor);
                    let sx = self.col_to_screen(cursor);

                    if let Some((sx, sy)) = sx.iter().zip(sy.iter()).next() {
                        Some((self.inner.x + *sx, self.inner.y + *sy))
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        }
    }
}

impl RelocatableState for TextAreaState {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        // clip offset for some corrections.
        self.dark_offset = relocate_dark_offset(self.inner, shift, clip);
        self.area = relocate_area(self.area, shift, clip);
        self.inner = relocate_area(self.inner, shift, clip);
    }
}

impl TextAreaState {
    /// Converts from a widget relative screen coordinate to a line.
    /// It limits its result to a valid row.
    pub fn screen_to_row(&self, scy: i16) -> upos_type {
        let (_, oy) = self.offset();
        let oy = oy as upos_type + self.dark_offset.1 as upos_type;

        if scy < 0 {
            oy.saturating_sub((scy as ipos_type).unsigned_abs())
        } else if scy as u16 >= (self.inner.height + self.dark_offset.1) {
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
        self.try_screen_to_col(row, scx).expect("valid_row")
    }

    /// Converts from a widget relative screen coordinate to a grapheme index.
    /// It limits its result to a valid column.
    ///
    /// * row is a row-index into the value, not a screen-row. It can be calculated
    ///   with screen_to_row().
    /// * x is the relative screen position.
    pub fn try_screen_to_col(&self, row: upos_type, scx: i16) -> Result<upos_type, TextError> {
        let (ox, _) = self.offset();

        let ox = ox as upos_type + self.dark_offset.0 as upos_type;

        if scx < 0 {
            Ok(ox.saturating_sub((scx as ipos_type).unsigned_abs()))
        } else if scx as u16 >= (self.inner.width + self.dark_offset.0) {
            Ok(min(ox + scx as upos_type, self.line_width(row)))
        } else {
            let scx = scx as u16;

            let line = self.try_glyphs(
                row..row + 1,
                ox as u16,
                self.inner.width + self.dark_offset.0,
            )?;

            let mut col = ox;
            for g in line {
                if scx < g.screen_pos().0 + g.screen_width() {
                    break;
                }
                col = g.pos().x + 1;
            }
            Ok(col)
        }
    }

    /// Converts the row of the position to a screen position
    /// relative to the widget area.
    pub fn row_to_screen(&self, pos: impl Into<TextPosition>) -> Option<u16> {
        let pos = pos.into();
        let (_, oy) = self.offset();

        if pos.y < oy as upos_type {
            return None;
        }

        let screen_y = pos.y - oy as upos_type;

        if screen_y >= self.dark_offset.1 as upos_type {
            Some(screen_y as u16 - self.dark_offset.1)
        } else {
            None
        }
    }

    /// Converts a grapheme based position to a screen position
    /// relative to the widget area.
    pub fn col_to_screen(&self, pos: impl Into<TextPosition>) -> Option<u16> {
        self.try_col_to_screen(pos).expect("valid_pos")
    }

    /// Converts a grapheme based position to a screen position
    /// relative to the widget area.
    pub fn try_col_to_screen(
        &self,
        pos: impl Into<TextPosition>,
    ) -> Result<Option<u16>, TextError> {
        let pos = pos.into();
        let (ox, _) = self.offset();

        if pos.x < ox as upos_type {
            return Ok(None);
        }

        let line = self.try_glyphs(
            pos.y..pos.y + 1,
            ox as u16,
            self.inner.width + self.dark_offset.0,
        )?;
        let mut screen_x = 0;
        for g in line {
            if g.pos().x == pos.x {
                break;
            }
            screen_x = g.screen_pos().0 + g.screen_width();
        }

        if screen_x >= self.dark_offset.0 {
            Ok(Some(screen_x - self.dark_offset.0))
        } else {
            Ok(None)
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

        let cy = self.screen_to_row(scy);
        let cx = self.screen_to_col(cy, scx);
        let cursor = TextPosition::new(cx, cy);

        let cursor = if cursor < anchor {
            self.word_start(cursor)
        } else {
            self.word_end(cursor)
        };

        // extend anchor
        if !self.is_word_boundary(anchor) {
            if cursor < anchor {
                self.set_cursor(self.word_end(anchor), false);
            } else {
                self.set_cursor(self.word_start(anchor), false);
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

        let cursor = self.cursor();
        let (ox, oy) = self.offset();
        let (ox, oy) = (ox as upos_type, oy as upos_type);

        let noy = if cursor.y < oy {
            cursor.y
        } else if cursor.y >= oy + (self.inner.height + self.dark_offset.1) as upos_type {
            cursor
                .y
                .saturating_sub((self.inner.height + self.dark_offset.1) as upos_type - 1)
        } else {
            oy
        };

        let nox = if cursor.x < ox {
            cursor.x
        } else if cursor.x >= ox + (self.inner.width + self.dark_offset.0) as upos_type {
            cursor
                .x
                .saturating_sub((self.inner.width + self.dark_offset.0) as upos_type)
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
                ct_event!(keycode press SHIFT-BackTab) => {
                    // ignore tab from focus
                    tc(if !self.focus.gained() {
                        self.insert_backtab()
                    } else {
                        false
                    })
                }
                ct_event!(keycode press Enter) => tc(self.insert_newline()),
                ct_event!(keycode press Backspace) => tc(self.delete_prev_char()),
                ct_event!(keycode press Delete) => tc(self.delete_next_char()),
                ct_event!(keycode press CONTROL-Backspace)
                | ct_event!(keycode press ALT-Backspace) => tc(self.delete_prev_word()),
                ct_event!(keycode press CONTROL-Delete) | ct_event!(keycode press ALT-Delete) => {
                    tc(self.delete_next_word())
                }
                ct_event!(key press CONTROL-'x') => tc(self.cut_to_clip()),
                ct_event!(key press CONTROL-'v') => tc(self.paste_from_clip()),
                ct_event!(key press CONTROL-'d') => tc(self.duplicate_text()),
                ct_event!(key press CONTROL-'y') => tc(self.delete_line()),
                ct_event!(key press CONTROL-'z') => tc(self.undo()),
                ct_event!(key press CONTROL_SHIFT-'Z') => tc(self.redo()),

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
                ct_event!(key press CONTROL-'c') => self.copy_to_clip().into(),

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
                | ct_event!(key release CONTROL-'a')
                | ct_event!(key release CONTROL-'c') => TextOutcome::Unchanged,
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
                let start = self.word_start(test);
                let end = self.word_end(test);
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

        let mut sas = ScrollAreaState::new()
            .area(self.inner)
            .h_scroll(&mut self.hscroll)
            .v_scroll(&mut self.vscroll);
        let r = match sas.handle(event, MouseOnly) {
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
