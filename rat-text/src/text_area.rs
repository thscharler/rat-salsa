//!
//! A text-area widget with text-styling abilities.
//! And undo + clipboard support.
//!

use crate::_private::NonExhaustive;
use crate::clipboard::{Clipboard, global_clipboard};
use crate::event::{ReadOnly, TextOutcome};
#[allow(deprecated)]
use crate::glyph::Glyph;
use crate::glyph2::{GlyphIter2, TextWrap2};
use crate::text_area::text_area_op::*;
use crate::text_core::TextCore;
use crate::text_core::core_op::*;
use crate::text_store::TextStore;
use crate::text_store::text_rope::TextRope;
use crate::undo_buffer::{UndoBuffer, UndoEntry, UndoVec};
use crate::{HasScreenCursor, TextError, TextPosition, TextRange, TextStyle, ipos_type, upos_type};
use crossterm::event::KeyModifiers;
use rat_event::util::MouseFlags;
use rat_event::{HandleEvent, MouseOnly, Regular, ct_event, flow};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_reloc::{RelocatableState, relocate_area, relocate_dark_offset, relocate_pos_tuple};
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::{Scroll, ScrollArea, ScrollAreaState, ScrollState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Rect, Size};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, StatefulWidget};
use ropey::Rope;
use std::borrow::Cow;
use std::cmp::{max, min};
use std::collections::HashMap;
use std::ops::Range;

pub mod text_area_op;

/// Text area widget.
///
/// [Example](https://github.com/thscharler/rat-salsa/blob/master/rat-text/examples/textarea2.rs)
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
/// Text wrapping is available, hard line-breaks at the right margin,
/// or decent word-wrapping.
///
/// You can directly access the underlying Rope for readonly purposes, and
/// conversion from/to byte/char positions are available. That should probably be
/// enough to write a parser that generates some styling.
///
/// The cursor must set externally on the ratatui Frame as usual.
/// [screen_cursor](TextAreaState::screen_cursor) gives you the correct value.
/// There is the inverse too [set_screen_cursor](TextAreaState::set_screen_cursor)
/// For more interactions you can use [screen_to_pos](TextAreaState::screen_to_pos),
/// and [pos_to_screen](TextAreaState::pos_to_screen). They calculate everything,
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

    text_wrap: Option<TextWrap>,

    style: Style,
    focus_style: Option<Style>,
    select_style: Option<Style>,
    text_style: HashMap<usize, Style>,
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
    /// Rendered dimension. This may differ from (inner.width, inner.height)
    /// if the text area has been relocated/clipped. This holds the
    /// original rendered dimension before any relocation/clipping.
    pub rendered: Size,
    /// Cursor position on the screen.
    pub screen_cursor: Option<(u16, u16)>,

    /// Horizontal scroll.
    /// When text-break is active this value is ignored.
    /// __read+write__
    pub hscroll: ScrollState,
    /// Vertical offset.
    /// __read+write__
    pub vscroll: ScrollState,
    /// When text-break is active, this is the grapheme-offset
    /// into the first visible text-row where the display
    /// actually starts.
    /// __read+write__ but it's not advised.
    pub sub_row_offset: upos_type,
    /// Dark offset due to clipping.
    /// __read only__ secondary offset due to clipping.
    pub dark_offset: (u16, u16),
    /// The scroll offset will be adjusted to display
    /// the cursor. This will be the minimal adjustment,
    /// the cursor will stay at the same screen position if
    /// it's already visible or appear at the start/end if it's not.
    /// __read+write__ use scroll_cursor_to_visible().
    pub scroll_to_cursor: bool,

    /// Text edit core
    pub value: TextCore<TextRope>,

    /// Memory-column for up/down movement.
    ///
    /// Up/down movement tries to place the cursor at this column,
    /// but might have to clip it, because the current line is too short.
    ///
    /// This is kept as a relative screen-position. It may be less
    /// than 0, if the widget has been relocated.
    pub move_col: Option<i16>,
    /// auto indent active
    pub auto_indent: bool,
    /// quote selection active
    pub auto_quote: bool,
    /// text breaking
    pub text_wrap: TextWrap,
    /// new-line bytes
    pub newline: String,
    /// tab-width
    pub tab_width: u32,
    /// expand tabs
    pub expand_tabs: bool,

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
            area: self.area,
            inner: self.inner,
            rendered: self.rendered,
            screen_cursor: self.screen_cursor,
            hscroll: self.hscroll.clone(),
            vscroll: self.vscroll.clone(),
            sub_row_offset: self.sub_row_offset,
            dark_offset: self.dark_offset,
            scroll_to_cursor: self.scroll_to_cursor,
            value: self.value.clone(),
            move_col: None,
            auto_indent: self.auto_indent,
            auto_quote: self.auto_quote,
            text_wrap: self.text_wrap,
            newline: self.newline.clone(),
            tab_width: self.tab_width,
            expand_tabs: self.expand_tabs,
            focus: FocusFlag::named(self.focus.name()),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

/// Text breaking.
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub enum TextWrap {
    /// Don't break, shift text to the left.
    #[default]
    Shift,
    /// Hard break at the right border.
    Hard,
    /// Wraps the text at word boundaries.
    ///
    /// The parameter gives an area before the right border where
    /// breaks are preferred. The first space that falls in this
    /// region will break. Otherwise, the last space before will be
    /// used, or the word will be hard-wrapped.
    ///
    /// Space is the word-separator. Words will be broken if they
    /// contain a hyphen, a soft-hyphen or a zero-width-space.
    Word(u16),
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
    pub fn styles(mut self, styles: TextStyle) -> Self {
        self.style = styles.style;
        if styles.focus.is_some() {
            self.focus_style = styles.focus;
        }
        if styles.select.is_some() {
            self.select_style = styles.select;
        }
        if let Some(border_style) = styles.border_style {
            self.block = self.block.map(|v| v.border_style(border_style));
        }
        self.block = self.block.map(|v| v.style(self.style));
        if styles.block.is_some() {
            self.block = styles.block;
        }
        if let Some(styles) = styles.scroll {
            self.hscroll = self.hscroll.map(|v| v.styles(styles.clone()));
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
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Selection style.
    pub fn select_style(mut self, style: Style) -> Self {
        self.select_style = Some(style);
        self
    }

    /// Indexed text-style.
    ///
    /// Use [TextAreaState::add_style()] to refer a text range to
    /// one of these styles.
    pub fn text_style_idx(mut self, idx: usize, style: Style) -> Self {
        self.text_style.insert(idx, style);
        self
    }

    /// List of text-styles.
    ///
    /// Use [TextAreaState::add_style()] to refer a text range to
    /// one of these styles.
    pub fn text_style<T: IntoIterator<Item = Style>>(mut self, styles: T) -> Self {
        for (i, s) in styles.into_iter().enumerate() {
            self.text_style.insert(i, s);
        }
        self
    }

    /// Map of style_id -> text_style.
    ///
    /// Use [TextAreaState::add_style()] to refer a text range to
    /// one of these styles.
    pub fn text_style_map<T: Into<Style>>(mut self, styles: HashMap<usize, T>) -> Self {
        for (i, s) in styles.into_iter() {
            self.text_style.insert(i, s.into());
        }
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

    /// Set the text wrapping.
    pub fn text_wrap(mut self, wrap: TextWrap) -> Self {
        self.text_wrap = Some(wrap);
        self
    }

    /// Maximum offset the horizontal scrollbar.
    ///
    /// This widget doesn't try to find a correct maximum value
    /// to show with the horizontal scroll bar, but uses this
    /// fixed value instead. This is the maximum offset that can
    /// be reached by using the scrollbar.
    ///
    /// Finding the maximum line length for a text is rather
    /// expensive, so this widget doesn't even try.
    ///
    /// This doesn't limit the column that can be reached with
    /// cursor positioning, just what can be done via the scrollbar.
    ///
    /// See [self.set_horizontal_overscroll]
    ///
    /// Default is 255.
    pub fn set_horizontal_max_offset(mut self, offset: usize) -> Self {
        self.h_max_offset = Some(offset);
        self
    }

    /// Maximum overscroll that can be reached by using the horizontal
    /// scrollbar and dragging beyond the area of the widget.
    ///
    /// See [self.set_horizontal_max_offset]
    ///
    /// Default is 16384.
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

impl<'a> StatefulWidget for &TextArea<'a> {
    type State = TextAreaState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
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
    state.screen_cursor = None;
    if let Some(text_wrap) = widget.text_wrap {
        state.text_wrap = text_wrap;
    }

    let style = widget.style;
    let focus_style = if let Some(focus_style) = widget.focus_style {
        focus_style
    } else {
        style
    };
    let select_style = if let Some(select_style) = widget.select_style {
        select_style
    } else {
        Style::default().black().on_yellow()
    };
    let (style, select_style) = if state.is_focused() {
        (
            style.patch(focus_style),
            style.patch(focus_style).patch(select_style),
        )
    } else {
        (style, style.patch(select_style))
    };

    // sync scroll and cursor
    state.area = area;
    state.screen_cursor = None;
    state.inner = ScrollArea::new()
        .block(widget.block.as_ref())
        .h_scroll(widget.hscroll.as_ref())
        .v_scroll(widget.vscroll.as_ref())
        .inner(area, Some(&state.hscroll), Some(&state.vscroll));
    state.rendered = state.inner.as_size();

    if let TextWrap::Hard | TextWrap::Word(_) = state.text_wrap {
        state.hscroll.set_max_offset(0);
        state.hscroll.set_overscroll_by(None);
    } else {
        if let Some(h_max_offset) = widget.h_max_offset {
            state.hscroll.set_max_offset(h_max_offset);
        }
        if let Some(h_overscroll) = widget.h_overscroll {
            state.hscroll.set_overscroll_by(Some(h_overscroll));
        }
    }
    state.hscroll.set_page_len(state.inner.width as usize);

    if let TextWrap::Hard | TextWrap::Word(_) = state.text_wrap {
        state
            .vscroll
            .set_max_offset(state.len_lines().saturating_sub(1) as usize);
    } else {
        state.vscroll.set_max_offset(
            state
                .len_lines()
                .saturating_sub(state.inner.height as upos_type) as usize,
        );
    }
    state.vscroll.set_page_len(state.inner.height as usize);

    if state.scroll_to_cursor {
        state.scroll_to_pos(state.cursor());
    }

    // scroll + background
    ScrollArea::new()
        .block(widget.block.as_ref())
        .h_scroll(widget.hscroll.as_ref())
        .v_scroll(widget.vscroll.as_ref())
        .style(style)
        .render(
            area,
            buf,
            &mut ScrollAreaState::new()
                .h_scroll(&mut state.hscroll)
                .v_scroll(&mut state.vscroll),
        );

    if state.inner.width == 0 || state.inner.height == 0 {
        // noop
        return;
    }
    if state.vscroll.offset() > state.value.len_lines() as usize {
        // noop
        return;
    }

    let (shift_left, sub_row_offset, start_row) = state.clean_offset();
    let page_rows = start_row
        ..min(
            start_row + state.inner.height as upos_type,
            state.value.len_lines(),
        );
    let page_bytes = state
        .try_bytes_at_range(TextRange::new(
            (sub_row_offset, page_rows.start),
            (0, page_rows.end),
        ))
        .expect("valid_rows");
    // let mut screen_cursor = None;
    let selection = state.selection();
    let mut styles = Vec::new();

    for g in state
        .glyphs2(shift_left, sub_row_offset, page_rows)
        .expect("valid_offset")
    {
        // relative screen-pos of the glyph
        let screen_pos = g.screen_pos();

        if screen_pos.1 >= state.inner.height {
            break;
        }

        if g.screen_width() > 0 {
            let mut style = style;
            // text-styles
            state
                .value
                .styles_at_page(g.text_bytes().start, page_bytes.clone(), &mut styles);
            for style_nr in &styles {
                if let Some(s) = widget.text_style.get(style_nr) {
                    style = style.patch(*s);
                }
            }
            // selection
            if selection.contains_pos(g.pos()) {
                style = style.patch(select_style);
            };

            // render glyph
            if let Some(cell) =
                buf.cell_mut((state.inner.x + screen_pos.0, state.inner.y + screen_pos.1))
            {
                cell.set_symbol(g.glyph());
                cell.set_style(style);
            }
            // clear the reset of the cells to avoid interferences.
            for d in 1..g.screen_width() {
                if let Some(cell) = buf.cell_mut((
                    state.inner.x + screen_pos.0 + d,
                    state.inner.y + screen_pos.1,
                )) {
                    cell.reset();
                    cell.set_style(style);
                }
            }
        }
    }

    state.screen_cursor = state.pos_to_screen(state.cursor());
    // state.screen_cursor = screen_cursor.map(|v| (state.inner.x + v.0, state.inner.y + v.1));
}

impl Default for TextAreaState {
    fn default() -> Self {
        #[cfg(windows)]
        const LINE_ENDING: &str = "\r\n";

        #[cfg(not(windows))]
        const LINE_ENDING: &str = "\n";

        let mut s = Self {
            area: Default::default(),
            inner: Default::default(),
            rendered: Default::default(),
            screen_cursor: Default::default(),
            hscroll: Default::default(),
            vscroll: Default::default(),
            sub_row_offset: 0,
            dark_offset: Default::default(),
            scroll_to_cursor: Default::default(),
            value: TextCore::new(Some(Box::new(UndoVec::new(99))), Some(global_clipboard())),
            move_col: Default::default(),
            auto_indent: true,
            auto_quote: true,
            text_wrap: TextWrap::Shift,
            newline: LINE_ENDING.to_string(),
            tab_width: 8,
            expand_tabs: true,
            focus: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        };
        s.hscroll.set_max_offset(255);
        s.hscroll.set_overscroll_by(Some(16384));
        s
    }
}

impl HasFocus for TextAreaState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
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
        self.newline = br.into();
    }

    /// Line ending used for insert.
    #[inline]
    pub fn newline(&self) -> &str {
        &self.newline
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
    pub fn set_tab_width(&mut self, tabs: u32) {
        self.tab_width = tabs;
    }

    /// Tab-width
    #[inline]
    pub fn tab_width(&self) -> u32 {
        self.tab_width
    }

    /// Expand tabs to spaces. Only for new inputs.
    #[inline]
    pub fn set_expand_tabs(&mut self, expand: bool) {
        self.expand_tabs = expand;
    }

    /// Expand tabs to spaces. Only for new inputs.
    #[inline]
    pub fn expand_tabs(&self) -> bool {
        self.expand_tabs
    }

    /// Show glyphs for control characters.
    #[inline]
    pub fn set_show_ctrl(&mut self, show_ctrl: bool) {
        self.value.set_glyph_ctrl(show_ctrl);
    }

    /// Show glyphs for control characters.
    pub fn show_ctrl(&self) -> bool {
        self.value.glyph_ctrl()
    }

    /// Show glyphs for text-wrapping.
    /// Shows inserted line-breaks, zero-width space (U+200B) or the soft hyphen (U+00AD).
    #[inline]
    pub fn set_wrap_ctrl(&mut self, wrap_ctrl: bool) {
        self.value.set_wrap_ctrl(wrap_ctrl);
    }

    /// Show glyphs for text-wrapping.
    /// Shows inserted line-breaks, zero-width space (U+200B) or the soft hyphen (U+00AD).
    pub fn wrap_ctrl(&self) -> bool {
        self.value.wrap_ctrl()
    }

    /// Text wrapping mode.
    ///
    /// * TextWrap::Shift means no wrapping.
    /// * TextWrap::Hard hard-wraps at the right border.
    /// * TextWrap::Word(n) does word wrapping.
    ///   n gives the size of the break-region close to the right border.
    ///   The first space that falls in this region is taken as a break.
    ///   If that is not possible this will break at the last space before.
    ///   If there is no space it will hard-wrap the word.
    ///
    ///   Space is used as word separator. Hyphen will be used to break
    ///   a word, and soft-hyphen and zero-width-space will be recognized too.
    pub fn set_text_wrap(&mut self, text_wrap: TextWrap) {
        self.text_wrap = text_wrap;
    }

    /// Text wrapping.
    pub fn text_wrap(&self) -> TextWrap {
        self.text_wrap
    }

    /// Extra column information for cursor movement.
    ///
    /// The cursor position is capped to the current line length, so if you
    /// move up one row, you might end at a position left of the current column.
    /// If you move up once more you want to return to the original position.
    /// That's what is stored here.
    ///
    /// This stores the relative screen-column, it may be less than 0
    /// if the widget has been relocated.
    #[inline]
    pub fn set_move_col(&mut self, col: Option<i16>) {
        self.move_col = col;
    }

    /// Extra column information for cursor movement.
    #[inline]
    pub fn move_col(&mut self) -> Option<i16> {
        self.move_col
    }
}

impl TextAreaState {
    /// Clipboard used.
    /// Default is to use the global_clipboard().
    #[inline]
    pub fn set_clipboard(&mut self, clip: Option<impl Clipboard + 'static>) {
        match clip {
            None => self.value.set_clipboard(None),
            Some(v) => self.value.set_clipboard(Some(Box::new(v))),
        }
    }

    /// Clipboard used.
    /// Default is to use the global_clipboard().
    #[inline]
    pub fn clipboard(&self) -> Option<&dyn Clipboard> {
        self.value.clipboard()
    }

    /// Copy selection to clipboard.
    #[inline]
    pub fn copy_to_clip(&mut self) -> bool {
        let Some(clip) = self.value.clipboard() else {
            return false;
        };

        _ = clip.set_string(self.selected_text().as_ref());
        false
    }

    /// Cut selection to clipboard.
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

    /// Paste from clipboard.
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
    /// Set the undo buffer.
    ///
    /// Default is an undo-buffer with 99 undoes.
    ///
    /// Adjacent edits will be merged automatically into a single undo.
    /// (Type multiple characters, they will be undone in one go.)
    #[inline]
    pub fn set_undo_buffer(&mut self, undo: Option<impl UndoBuffer + 'static>) {
        match undo {
            None => self.value.set_undo_buffer(None),
            Some(v) => self.value.set_undo_buffer(Some(Box::new(v))),
        }
    }

    /// Access the undo buffer.
    #[inline]
    pub fn undo_buffer(&self) -> Option<&dyn UndoBuffer> {
        self.value.undo_buffer()
    }

    /// Access the undo buffer.
    #[inline]
    pub fn undo_buffer_mut(&mut self) -> Option<&mut dyn UndoBuffer> {
        self.value.undo_buffer_mut()
    }

    /// Begin a sequence of changes that should be undone in one go.
    ///
    /// Call begin_undo_seq(), then call any edit-functions. When
    /// you are done call end_undo_seq(). Any changes will be undone
    /// in a single step.
    #[inline]
    pub fn begin_undo_seq(&mut self) {
        self.value.begin_undo_seq()
    }

    /// End a sequence of changes that should be undone in one go.
    #[inline]
    pub fn end_undo_seq(&mut self) {
        self.value.end_undo_seq()
    }

    /// Get all recent replay recordings. This log can be sent
    /// to a second TextAreaState and can be applied with replay_log().
    ///
    /// There are some [caveats](UndoBuffer::enable_replay_log).
    #[inline]
    pub fn recent_replay_log(&mut self) -> Vec<UndoEntry> {
        self.value.recent_replay_log()
    }

    /// Apply the replay recording.
    ///
    /// There are some [caveats](UndoBuffer::enable_replay_log).
    #[inline]
    pub fn replay_log(&mut self, replay: &[UndoEntry]) {
        self.value.replay_log(replay)
    }

    /// Do one undo.
    #[inline]
    pub fn undo(&mut self) -> bool {
        self.value.undo()
    }

    /// Do one redo.
    #[inline]
    pub fn redo(&mut self) -> bool {
        self.value.redo()
    }
}

impl TextAreaState {
    /// Set and replace all styles.
    ///
    /// The ranges are byte-ranges into the text. There is no
    /// verification that the ranges fit the text.
    ///
    /// Each byte-range maps to an index into the styles set
    /// with the widget.
    ///
    /// Any style-idx that don't have a match there are just
    /// ignored. You can use this to store other range based information.
    /// The ranges are corrected during edits, no need to recalculate
    /// everything after each keystroke.
    ///
    /// But this is only a very basic correction based on
    /// insertions and deletes. If you use this for syntax-highlighting
    /// you probably need to rebuild the styles.
    #[inline]
    pub fn set_styles(&mut self, styles: Vec<(Range<usize>, usize)>) {
        self.value.set_styles(styles);
    }

    /// Set and replace all styles.
    ///
    /// The ranges are TextRanges into the text.
    /// Each byte-range maps to an index into the styles set
    /// with the widget.
    ///
    /// Any style-idx that don't have a match there are just
    /// ignored. You can use this to store other range based information.
    /// The ranges are corrected during edits, no need to recalculate
    /// everything after each keystroke.
    #[inline]
    pub fn set_range_styles(&mut self, styles: Vec<(TextRange, usize)>) -> Result<(), TextError> {
        let mut mapped = Vec::with_capacity(styles.len());
        for (r, s) in styles {
            let rr = self.value.bytes_at_range(r)?;
            mapped.push((rr, s));
        }
        self.value.set_styles(mapped);
        Ok(())
    }

    /// Add a style for a byte-range.
    ///
    /// The style-idx refers to one of the styles set with the widget.
    /// Missing styles are just ignored.
    #[inline]
    pub fn add_style(&mut self, range: Range<usize>, style: usize) {
        self.value.add_style(range, style);
    }

    /// Add a style for a [TextRange]. The style-nr refers to one
    /// of the styles set with the widget.
    /// Missing styles are just ignored.
    #[inline]
    pub fn add_range_style(
        &mut self,
        range: impl Into<TextRange>,
        style: usize,
    ) -> Result<(), TextError> {
        let r = self.value.bytes_at_range(range.into())?;
        self.value.add_style(r, style);
        Ok(())
    }

    /// Remove the exact byte-range and style.
    #[inline]
    pub fn remove_style(&mut self, range: Range<usize>, style: usize) {
        self.value.remove_style(range, style);
    }

    /// Remove all ranges for the given style.
    #[inline]
    pub fn remove_style_fully(&mut self, style: usize) {
        self.value.remove_style_fully(style);
    }

    /// Remove the exact TextRange and style.
    #[inline]
    pub fn remove_range_style(
        &mut self,
        range: impl Into<TextRange>,
        style: usize,
    ) -> Result<(), TextError> {
        let r = self.value.bytes_at_range(range.into())?;
        self.value.remove_style(r, style);
        Ok(())
    }

    /// Find all styles that touch the given range.
    pub fn styles_in(&self, range: Range<usize>, buf: &mut Vec<(Range<usize>, usize)>) {
        self.value.styles_in(range, buf)
    }

    /// Find all styles that touch the given range.
    pub fn styles_in_match(
        &self,
        range: Range<usize>,
        style: usize,
        buf: &mut Vec<(Range<usize>, usize)>,
    ) {
        self.value.styles_in_match(range, style, buf);
    }

    /// All styles active at the given position.
    #[inline]
    pub fn styles_at(&self, byte_pos: usize, buf: &mut Vec<(Range<usize>, usize)>) {
        self.value.styles_at(byte_pos, buf)
    }

    /// Check if the given style applies at the position and
    /// return the complete range for the style.
    #[inline]
    pub fn styles_at_match(&self, byte_pos: usize, style: usize) -> Option<Range<usize>> {
        self.value.styles_at_match(byte_pos, style)
    }

    /// Check if the given style applies at the position and
    /// return the complete range for the style.
    #[inline]
    #[allow(deprecated)]
    #[deprecated(since = "1.3.0", note = "use styles_at_match() instead")]
    pub fn style_match(&self, byte_pos: usize, style: usize) -> Option<Range<usize>> {
        self.value.style_match(byte_pos, style)
    }

    /// List of all styles.
    #[inline]
    pub fn styles(&self) -> Option<impl Iterator<Item = (Range<usize>, usize)> + '_> {
        self.value.styles()
    }
}

impl TextAreaState {
    /// Current offset for scrolling.
    #[inline]
    pub fn offset(&self) -> (usize, usize) {
        (self.hscroll.offset(), self.vscroll.offset())
    }

    /// Set the offset for scrolling.
    ///
    /// The offset uses usize, but it shouldn't exceed (u32::MAX, u32::MAX).
    /// This is due to the internal ScrollState that only knows usize.
    #[inline]
    pub fn set_offset(&mut self, offset: (usize, usize)) -> bool {
        self.scroll_to_cursor = false;
        let c = self.hscroll.set_offset(offset.0);
        let r = self.vscroll.set_offset(offset.1);
        r || c
    }

    /// Scrolling uses the offset() for the start of the displayed text.
    /// When text-wrapping is active, this is not enough. This gives
    /// an extra offset into the first row where rendering should start.
    ///
    /// You probably don't want to set this value. Use set_cursor()
    /// instead, it will automatically scroll to make the cursor visible.
    ///
    /// If you really want, pos_to_line_start() can help to find the
    /// start-position of a visual row. The x-value of the returned
    /// TextPosition is a valid value for this function.
    ///
    pub fn set_sub_row_offset(&mut self, sub_row_offset: upos_type) -> bool {
        self.scroll_to_cursor = false;
        let old = self.sub_row_offset;
        self.sub_row_offset = sub_row_offset;
        sub_row_offset != old
    }

    /// Returns the extra offset into the first row where rendering
    /// starts. This is only valid if text-wrapping is active.
    ///
    /// Returns the index of the column.
    pub fn sub_row_offset(&self) -> upos_type {
        self.sub_row_offset
    }

    /// This returns the triple (hscroll.offset, sub_row_offset, vscroll.offset )
    /// all trimmed to upos_type. sub_row_offset will only have a value if
    /// there is some text-wrapping active. hscroll.offset will only have
    /// an offset if there is *no* text-wrapping active.
    fn clean_offset(&self) -> (upos_type, upos_type, upos_type) {
        let ox = self.hscroll.offset as upos_type;
        let mut oy = self.vscroll.offset as upos_type;

        // reset invalid offset
        if oy >= self.len_lines() {
            oy = 0;
        }

        match self.text_wrap {
            TextWrap::Shift => (ox, 0, oy),
            TextWrap::Hard | TextWrap::Word(_) => {
                // sub_row_offset can be any value. limit somewhat.
                if let Ok(max_col) = self.try_line_width(oy) {
                    (0, min(self.sub_row_offset, max_col), oy)
                } else {
                    (0, 0, oy)
                }
            }
        }
    }

    /// Cursor position.
    #[inline]
    pub fn cursor(&self) -> TextPosition {
        self.value.cursor()
    }

    /// Set the cursor position and scroll the cursor to a visible offset.
    #[inline]
    pub fn set_cursor(&mut self, cursor: impl Into<TextPosition>, extend_selection: bool) -> bool {
        self.scroll_cursor_to_visible();
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

    /// Set the selection, anchor and cursor are capped to a valid value.
    /// Scrolls the cursor to a visible position.
    #[inline]
    pub fn set_selection(
        &mut self,
        anchor: impl Into<TextPosition>,
        cursor: impl Into<TextPosition>,
    ) -> bool {
        self.scroll_cursor_to_visible();
        self.value.set_selection(anchor.into(), cursor.into())
    }

    /// Select all.
    /// Scrolls the cursor to a visible position.
    #[inline]
    pub fn select_all(&mut self) -> bool {
        self.scroll_cursor_to_visible();
        self.value.select_all()
    }

    /// Selected text.
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

    /// Access the underlying rope.
    #[inline]
    pub fn rope(&self) -> &Rope {
        self.value.text().rope()
    }

    /// Copy of the text-value.
    #[inline]
    pub fn text(&self) -> String {
        self.value.text().string()
    }

    /// Text slice as `Cow<str>`. Uses a byte range.
    ///
    /// Panics for an invalid range.
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
    ///
    /// Panics for an invalid range.
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

    /// Length in bytes.
    #[inline]
    pub fn len_bytes(&self) -> usize {
        self.value.len_bytes()
    }

    /// Line width as grapheme count.
    ///
    /// Panics for an invalid row.
    #[inline]
    pub fn line_width(&self, row: upos_type) -> upos_type {
        self.try_line_width(row).expect("valid_row")
    }

    /// Line width as grapheme count.
    #[inline]
    pub fn try_line_width(&self, row: upos_type) -> Result<upos_type, TextError> {
        self.value.line_width(row)
    }

    /// Line as `Cow<str>`.
    /// This contains the \n at the end.
    ///
    /// Panics for an invalid row.
    #[inline]
    pub fn line_at(&self, row: upos_type) -> Cow<'_, str> {
        self.value.line_at(row).expect("valid_row")
    }

    /// Line as `Cow<str>`.
    /// This contains the \n at the end.
    #[inline]
    pub fn try_line_at(&self, row: upos_type) -> Result<Cow<'_, str>, TextError> {
        self.value.line_at(row)
    }

    /// Iterate over text-lines, starting at offset.
    ///
    /// Panics for an invalid row.
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

    /// Iterator for the glyphs of the lines in range.
    /// Glyphs here a grapheme + display length.
    #[inline]
    #[allow(deprecated)]
    #[deprecated(since = "1.1.0", note = "discontinued api")]
    pub fn glyphs(
        &self,
        rows: Range<upos_type>,
        screen_offset: u16,
        screen_width: u16,
    ) -> impl Iterator<Item = Glyph<'_>> {
        self.value
            .glyphs(rows, screen_offset, screen_width, self.tab_width as u16)
            .expect("valid_rows")
    }

    /// Iterator for the glyphs of the lines in range.
    /// Glyphs here a grapheme + display length.
    #[inline]
    #[allow(deprecated)]
    #[deprecated(since = "1.1.0", note = "discontinued api")]
    pub fn try_glyphs(
        &self,
        rows: Range<upos_type>,
        screen_offset: u16,
        screen_width: u16,
    ) -> Result<impl Iterator<Item = Glyph<'_>>, TextError> {
        self.value
            .glyphs(rows, screen_offset, screen_width, self.tab_width as u16)
    }

    /// Grapheme iterator for a given line.
    /// This contains the \n at the end.
    ///
    /// Panics for an invalid row.
    #[inline]
    pub fn line_graphemes(&self, row: upos_type) -> <TextRope as TextStore>::GraphemeIter<'_> {
        self.value.line_graphemes(row).expect("valid_row")
    }

    /// Grapheme iterator for a given line.
    /// This contains the \n at the end.
    #[inline]
    pub fn try_line_graphemes(
        &self,
        row: upos_type,
    ) -> Result<<TextRope as TextStore>::GraphemeIter<'_>, TextError> {
        self.value.line_graphemes(row)
    }

    /// Get a cursor over all the text with the current position set at pos.
    ///
    /// Panics for an invalid pos.
    #[inline]
    pub fn text_graphemes(
        &self,
        pos: impl Into<TextPosition>,
    ) -> <TextRope as TextStore>::GraphemeIter<'_> {
        self.value.text_graphemes(pos.into()).expect("valid_pos")
    }

    /// Get a cursor over all the text with the current position set at pos.
    #[inline]
    pub fn try_text_graphemes(
        &self,
        pos: impl Into<TextPosition>,
    ) -> Result<<TextRope as TextStore>::GraphemeIter<'_>, TextError> {
        self.value.text_graphemes(pos.into())
    }

    /// Get a cursor over the text-range the current position set at pos.
    ///
    /// Panics for an invalid pos.
    #[inline]
    pub fn graphemes(
        &self,
        range: impl Into<TextRange>,
        pos: impl Into<TextPosition>,
    ) -> <TextRope as TextStore>::GraphemeIter<'_> {
        self.value
            .graphemes(range.into(), pos.into())
            .expect("valid_args")
    }

    /// Get a cursor over the text-range the current position set at pos.
    #[inline]
    pub fn try_graphemes(
        &self,
        range: impl Into<TextRange>,
        pos: impl Into<TextPosition>,
    ) -> Result<<TextRope as TextStore>::GraphemeIter<'_>, TextError> {
        self.value.graphemes(range.into(), pos.into())
    }

    /// Grapheme position to byte position.
    /// This is the (start,end) position of the single grapheme after pos.
    ///
    /// Panics for an invalid pos.
    #[inline]
    pub fn byte_at(&self, pos: impl Into<TextPosition>) -> Range<usize> {
        self.value.byte_at(pos.into()).expect("valid_pos")
    }

    /// Grapheme position to byte position.
    /// This is the (start,end) position of the single grapheme after pos.
    #[inline]
    pub fn try_byte_at(&self, pos: impl Into<TextPosition>) -> Result<Range<usize>, TextError> {
        self.value.byte_at(pos.into())
    }

    /// Grapheme range to byte range.
    ///
    /// Panics for an invalid range.
    #[inline]
    pub fn bytes_at_range(&self, range: impl Into<TextRange>) -> Range<usize> {
        self.value
            .bytes_at_range(range.into())
            .expect("valid_range")
    }

    /// Grapheme range to byte range.
    #[inline]
    pub fn try_bytes_at_range(
        &self,
        range: impl Into<TextRange>,
    ) -> Result<Range<usize>, TextError> {
        self.value.bytes_at_range(range.into())
    }

    /// Byte position to grapheme position.
    /// Returns the position that contains the given byte index.
    ///
    /// Panics for an invalid byte pos.
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
    ///
    /// Panics for an invalid range.
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
    ///
    /// Resets all internal state.
    #[inline]
    pub fn set_text<S: AsRef<str>>(&mut self, s: S) {
        self.scroll_to_cursor = false;
        self.vscroll.set_offset(0);
        self.hscroll.set_offset(0);
        self.set_sub_row_offset(0);
        self.set_move_col(None);

        self.value.set_text(TextRope::new_text(s.as_ref()));
    }

    /// Set the text value as a Rope.
    /// Resets all internal state.
    #[inline]
    pub fn set_rope(&mut self, r: Rope) {
        self.scroll_to_cursor = false;
        self.vscroll.set_offset(0);
        self.hscroll.set_offset(0);
        self.set_sub_row_offset(0);
        self.set_move_col(None);

        self.value.set_text(TextRope::new_rope(r));
    }

    /// Insert a character at the cursor position.
    /// Removes the selection and inserts the char.
    ///
    /// You can insert a tab with this. But it will not
    /// indent the current selection. It will expand
    /// the tab though. Use insert_tab() for this.
    ///
    /// You can insert a new-line with this. But it will
    /// not do an auto-indent.
    /// Use insert_new_line() for this.
    pub fn insert_char(&mut self, c: char) -> bool {
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
                let sel = self.selection();
                insert_quotes(&mut self.value, sel, c).expect("valid_selection");
                self.scroll_cursor_to_visible();
                return true;
            }
        }

        self.value
            .remove_str_range(self.selection())
            .expect("valid_selection");

        let pos = self.cursor();

        // insert missing newline
        if pos.x == 0
            && pos.y != 0
            && (pos.y == self.len_lines() || pos.y == self.len_lines().saturating_sub(1))
            && !self.value.text().has_final_newline()
        {
            let anchor = self.value.anchor();
            let cursor = self.value.cursor();
            self.value
                .insert_str(pos, &self.newline)
                .expect("valid_cursor");
            self.value.set_selection(anchor, cursor);
        }

        if c == '\n' {
            self.value
                .insert_str(pos, &self.newline)
                .expect("valid_cursor");
        } else if c == '\t' {
            insert_tab(&mut self.value, pos, self.expand_tabs, self.tab_width)
                .expect("valid_cursor");
        } else {
            self.value.insert_char(pos, c).expect("valid_cursor");
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
                indent(self, self.tab_width);
                true
            } else {
                false
            }
        } else {
            let pos = self.cursor();
            insert_tab(&mut self.value, pos, self.expand_tabs, self.tab_width)
                .expect("valid_cursor");
            self.scroll_cursor_to_visible();

            true
        }
    }

    /// Dedent the selected text by tab-width. If there is no
    /// selection this does nothing.
    ///
    /// This can be deactivated with auto_indent=false.
    pub fn insert_backtab(&mut self) -> bool {
        if self.has_selection() {
            dedent(self, self.tab_width);
            true
        } else {
            false
        }
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
            .insert_str(self.cursor(), &self.newline)
            .expect("valid_cursor");

        // insert leading spaces
        if self.auto_indent {
            auto_indent(self);
        }

        self.scroll_cursor_to_visible();
        true
    }

    /// Deletes the given range.
    ///
    /// Panics for an invalid range.
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
    #[inline]
    pub fn duplicate_text(&mut self) -> bool {
        duplicate_text(self)
    }

    /// Deletes the current line.
    /// Returns true if there was any real change.
    #[inline]
    pub fn delete_line(&mut self) -> bool {
        delete_line(self)
    }

    /// Deletes the next char or the current selection.
    /// Returns true if there was any real change.
    #[inline]
    pub fn delete_next_char(&mut self) -> bool {
        self.scroll_cursor_to_visible();
        delete_next_char(self)
    }

    /// Deletes the previous char or the selection.
    /// Returns true if there was any real change.
    #[inline]
    pub fn delete_prev_char(&mut self) -> bool {
        self.scroll_cursor_to_visible();
        delete_prev_char(self)
    }

    /// Find the start of the next word. If the position is at the start
    /// or inside a word, the same position is returned.
    ///
    /// Panics for an invalid pos.
    #[inline]
    pub fn next_word_start(&self, pos: impl Into<TextPosition>) -> TextPosition {
        next_word_start(&self.value, pos.into()).expect("valid_pos")
    }

    /// Find the start of the next word. If the position is at the start
    /// or inside a word, the same position is returned.
    #[inline]
    pub fn try_next_word_start(
        &self,
        pos: impl Into<TextPosition>,
    ) -> Result<TextPosition, TextError> {
        next_word_start(&self.value, pos.into())
    }

    /// Find the end of the next word. Skips whitespace first, then goes on
    /// until it finds the next whitespace.
    ///
    /// Panics for an invalid pos.
    #[inline]
    pub fn next_word_end(&self, pos: impl Into<TextPosition>) -> TextPosition {
        next_word_end(&self.value, pos.into()).expect("valid_pos")
    }

    /// Find the end of the next word. Skips whitespace first, then goes on
    /// until it finds the next whitespace.
    #[inline]
    pub fn try_next_word_end(
        &self,
        pos: impl Into<TextPosition>,
    ) -> Result<TextPosition, TextError> {
        next_word_end(&self.value, pos.into())
    }

    /// Find the start of the prev word. Skips whitespace first, then goes on
    /// until it finds the next whitespace.
    ///
    /// Attention: start/end are mirrored here compared to next_word_start/next_word_end,
    /// both return start<=end!
    ///
    /// Panics for an invalid range.
    #[inline]
    pub fn prev_word_start(&self, pos: impl Into<TextPosition>) -> TextPosition {
        prev_word_start(&self.value, pos.into()).expect("valid_pos")
    }

    /// Find the start of the prev word. Skips whitespace first, then goes on
    /// until it finds the next whitespace.
    ///
    /// Attention: start/end are mirrored here compared to next_word_start/next_word_end,
    /// both return start<=end!
    #[inline]
    pub fn try_prev_word_start(
        &self,
        pos: impl Into<TextPosition>,
    ) -> Result<TextPosition, TextError> {
        prev_word_start(&self.value, pos.into())
    }

    /// Find the end of the previous word. Word is everything that is not whitespace.
    /// Attention: start/end are mirrored here compared to next_word_start/next_word_end,
    /// both return start<=end!
    ///
    /// Panics for an invalid range.
    #[inline]
    pub fn prev_word_end(&self, pos: impl Into<TextPosition>) -> TextPosition {
        prev_word_end(&self.value, pos.into()).expect("valid_pos")
    }

    /// Find the end of the previous word. Word is everything that is not whitespace.
    /// Attention: start/end are mirrored here compared to next_word_start/next_word_end,
    /// both return start<=end!
    #[inline]
    pub fn try_prev_word_end(
        &self,
        pos: impl Into<TextPosition>,
    ) -> Result<TextPosition, TextError> {
        prev_word_end(&self.value, pos.into())
    }

    /// Is the position at a word boundary?
    ///
    /// Panics for an invalid range.
    #[inline]
    pub fn is_word_boundary(&self, pos: impl Into<TextPosition>) -> bool {
        is_word_boundary(&self.value, pos.into()).expect("valid_pos")
    }

    /// Is the position at a word boundary?
    #[inline]
    pub fn try_is_word_boundary(&self, pos: impl Into<TextPosition>) -> Result<bool, TextError> {
        is_word_boundary(&self.value, pos.into())
    }

    /// Find the start of the word at pos.
    /// Returns pos if the position is not inside a word.
    ///
    /// Panics for an invalid range.
    #[inline]
    pub fn word_start(&self, pos: impl Into<TextPosition>) -> TextPosition {
        word_start(&self.value, pos.into()).expect("valid_pos")
    }

    /// Find the start of the word at pos.
    /// Returns pos if the position is not inside a word.
    #[inline]
    pub fn try_word_start(&self, pos: impl Into<TextPosition>) -> Result<TextPosition, TextError> {
        word_start(&self.value, pos.into())
    }

    /// Find the end of the word at pos.
    /// Returns pos if the position is not inside a word.
    ///
    /// Panics for an invalid range.
    #[inline]
    pub fn word_end(&self, pos: impl Into<TextPosition>) -> TextPosition {
        word_end(&self.value, pos.into()).expect("valid_pos")
    }

    /// Find the end of the word at pos.
    /// Returns pos if the position is not inside a word.
    #[inline]
    pub fn try_word_end(&self, pos: impl Into<TextPosition>) -> Result<TextPosition, TextError> {
        word_end(&self.value, pos.into())
    }

    /// Delete the next word. This alternates deleting the whitespace between words and
    /// the words themselves.
    ///
    /// If there is a selection, removes only the selected text.
    #[inline]
    pub fn delete_next_word(&mut self) -> bool {
        delete_next_word(self)
    }

    /// Deletes the previous word. This alternates deleting the whitespace
    /// between words and the words themselves.
    ///
    /// If there is a selection, removes only the selected text.
    #[inline]
    pub fn delete_prev_word(&mut self) -> bool {
        delete_prev_word(self)
    }

    /// Search for a regex.
    ///
    /// Uses match_style for highlighting the matches.
    /// This doesn't change the cursor/selection, use [move_to_next_match] or
    /// [move_to_prev_match] for this.
    ///
    /// Return
    ///
    /// Returns true if the search found anything.
    ///
    pub fn search(&mut self, re: &str) -> Result<bool, TextError> {
        match search(self, re, MATCH_STYLE) {
            Ok(r) => Ok(r),
            Err(_) => Err(TextError::InvalidSearch),
        }
    }

    /// Move to the next match.
    pub fn move_to_next_match(&mut self) -> bool {
        move_to_next_match(self, MATCH_STYLE)
    }

    /// Move to the next match.
    pub fn move_to_prev_match(&mut self) -> bool {
        move_to_prev_match(self, MATCH_STYLE)
    }

    /// Clear the search.
    pub fn clear_search(&mut self) {
        clear_search(self, MATCH_STYLE)
    }

    /// Move the cursor left. Scrolls the cursor to visible.
    /// Returns true if there was any real change.
    #[inline]
    pub fn move_left(&mut self, n: u16, extend_selection: bool) -> bool {
        move_left(self, n, extend_selection)
    }

    /// Move the cursor right. Scrolls the cursor to visible.
    /// Returns true if there was any real change.
    #[inline]
    pub fn move_right(&mut self, n: u16, extend_selection: bool) -> bool {
        move_right(self, n, extend_selection)
    }

    /// Move the cursor up. Scrolls the cursor to visible.
    /// Returns true if there was any real change.
    #[inline]
    pub fn move_up(&mut self, n: u16, extend_selection: bool) -> bool {
        move_up(self, n, extend_selection)
    }

    /// Move the cursor down. Scrolls the cursor to visible.
    /// Returns true if there was any real change.
    #[inline]
    pub fn move_down(&mut self, n: u16, extend_selection: bool) -> bool {
        move_down(self, n, extend_selection)
    }

    /// Move the cursor to the start of the line.
    /// Scrolls the cursor to visible.
    /// Returns true if there was any real change.
    #[inline]
    pub fn move_to_line_start(&mut self, extend_selection: bool) -> bool {
        move_to_line_start(self, extend_selection)
    }

    /// Move the cursor to the end of the line. Scrolls to visible, if
    /// necessary.
    /// Returns true if there was any real change.
    #[inline]
    pub fn move_to_line_end(&mut self, extend_selection: bool) -> bool {
        move_to_line_end(self, extend_selection)
    }

    /// Move the cursor to the document start.
    #[inline]
    pub fn move_to_start(&mut self, extend_selection: bool) -> bool {
        move_to_start(self, extend_selection)
    }

    /// Move the cursor to the document end.
    #[inline]
    pub fn move_to_end(&mut self, extend_selection: bool) -> bool {
        move_to_end(self, extend_selection)
    }

    /// Move the cursor to the start of the visible area.
    #[inline]
    pub fn move_to_screen_start(&mut self, extend_selection: bool) -> bool {
        move_to_screen_start(self, extend_selection)
    }

    /// Move the cursor to the end of the visible area.
    #[inline]
    pub fn move_to_screen_end(&mut self, extend_selection: bool) -> bool {
        move_to_screen_end(self, extend_selection)
    }

    /// Move the cursor to the next word.
    #[inline]
    pub fn move_to_next_word(&mut self, extend_selection: bool) -> bool {
        move_to_next_word(self, extend_selection)
    }

    /// Move the cursor to the previous word.
    #[inline]
    pub fn move_to_prev_word(&mut self, extend_selection: bool) -> bool {
        move_to_prev_word(self, extend_selection)
    }
}

impl HasScreenCursor for TextAreaState {
    /// Cursor position on the screen.
    #[allow(clippy::question_mark)]
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        if self.is_focused() {
            if self.has_selection() {
                None
            } else {
                let Some(scr_cursor) = self.screen_cursor else {
                    return None;
                };

                if !(scr_cursor.0 >= self.inner.x
                    && scr_cursor.0 <= self.inner.right()
                    && scr_cursor.1 >= self.inner.y
                    && scr_cursor.1 < self.inner.bottom())
                {
                    return None;
                }
                Some(scr_cursor)
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
        if let Some(screen_cursor) = self.screen_cursor {
            self.screen_cursor = relocate_pos_tuple(screen_cursor, shift, clip);
        }
    }
}

impl TextAreaState {
    fn text_wrap_2(&self, shift_left: upos_type) -> (TextWrap2, upos_type, upos_type, upos_type) {
        match self.text_wrap {
            TextWrap::Shift => (
                TextWrap2::Shift,
                shift_left,
                shift_left + self.rendered.width as upos_type,
                shift_left + self.rendered.width as upos_type,
            ),
            TextWrap::Hard => (
                TextWrap2::Hard,
                0,
                self.rendered.width as upos_type,
                self.rendered.width as upos_type,
            ),
            TextWrap::Word(margin) => (
                TextWrap2::Word,
                0,
                self.rendered.width as upos_type,
                self.rendered.width.saturating_sub(margin) as upos_type,
            ),
        }
    }

    /// Fill the cache for the given rows.
    /// Build up the complete information for the given rows.
    fn fill_cache(
        &self,
        shift_left: upos_type,
        sub_row_offset: upos_type,
        rows: Range<upos_type>,
    ) -> Result<(), TextError> {
        let (text_wrap, left_margin, right_margin, word_margin) = self.text_wrap_2(shift_left);
        self.value.fill_cache(
            self.rendered,
            sub_row_offset,
            rows,
            self.tab_width,
            text_wrap,
            self.wrap_ctrl() | self.show_ctrl(),
            left_margin,
            right_margin,
            word_margin,
        )
    }

    fn glyphs2(
        &self,
        shift_left: upos_type,
        sub_row_offset: upos_type,
        rows: Range<upos_type>,
    ) -> Result<GlyphIter2<'_, <TextRope as TextStore>::GraphemeIter<'_>>, TextError> {
        let (text_wrap, left_margin, right_margin, word_margin) = self.text_wrap_2(shift_left);
        self.value.glyphs2(
            self.rendered,
            sub_row_offset,
            rows,
            self.tab_width,
            text_wrap,
            self.wrap_ctrl() | self.show_ctrl(),
            left_margin,
            right_margin,
            word_margin,
        )
    }

    /// Find the text-position for an absolute screen-position.
    pub fn screen_to_pos(&self, scr_pos: (u16, u16)) -> Option<TextPosition> {
        let scr_pos = (
            scr_pos.0 as i16 - self.inner.x as i16,
            scr_pos.1 as i16 - self.inner.y as i16,
        );
        self.relative_screen_to_pos(scr_pos)
    }

    /// Find the absolute screen-position for a text-position
    #[inline]
    pub fn pos_to_screen(&self, pos: impl Into<TextPosition>) -> Option<(u16, u16)> {
        let scr_pos = self.pos_to_relative_screen(pos.into())?;
        if scr_pos.0 + self.inner.x as i16 > 0 && scr_pos.1 + self.inner.y as i16 > 0 {
            Some((
                (scr_pos.0 + self.inner.x as i16) as u16,
                (scr_pos.1 + self.inner.y as i16) as u16,
            ))
        } else {
            None
        }
    }

    /// Return the starting position for the visible line containing the given position.
    pub fn pos_to_line_start(&self, pos: impl Into<TextPosition>) -> TextPosition {
        let pos = pos.into();
        match self.text_wrap {
            TextWrap::Shift => {
                //
                TextPosition::new(0, pos.y)
            }
            TextWrap::Hard | TextWrap::Word(_) => {
                self.fill_cache(0, 0, pos.y..min(pos.y + 1, self.len_lines()))
                    .expect("valid-row");

                let mut start_pos = TextPosition::new(0, pos.y);
                for (break_pos, _) in self.value.cache().line_break.borrow().range(
                    TextPosition::new(0, pos.y)
                        ..TextPosition::new(0, min(pos.y + 1, self.len_lines())),
                ) {
                    if pos >= start_pos && &pos <= break_pos {
                        break;
                    }
                    start_pos = TextPosition::new(break_pos.x + 1, break_pos.y);
                }

                start_pos
            }
        }
    }

    /// Return the end position for the visible line containing the given position.
    pub fn pos_to_line_end(&self, pos: impl Into<TextPosition>) -> TextPosition {
        let pos = pos.into();

        self.fill_cache(0, 0, pos.y..min(pos.y + 1, self.len_lines()))
            .expect("valid-row");

        let mut end_pos = TextPosition::new(0, pos.y);
        for (break_pos, _) in self
            .value
            .cache()
            .line_break
            .borrow()
            .range(TextPosition::new(0, pos.y)..TextPosition::new(0, pos.y + 1))
        {
            if pos >= end_pos && &pos <= break_pos {
                end_pos = *break_pos;
                break;
            }
            end_pos = TextPosition::new(break_pos.x + 1, break_pos.y);
        }

        end_pos
    }

    // ensure cache is up to date for n pages.
    fn stc_fill_screen_cache(&self, scr: (upos_type, upos_type, upos_type)) {
        let y2 = scr.1 + scr.2;
        self.fill_cache(0, scr.0, scr.1..min(y2, self.len_lines()))
            .expect("valid-rows");
    }

    // requires: cache for ... pages
    fn stc_screen_row(
        &self,
        scr: (upos_type, upos_type, upos_type),
        pos: TextPosition,
    ) -> Option<upos_type> {
        if pos < TextPosition::new(scr.0, scr.1) {
            return None;
        }

        let line_breaks = self.value.cache().line_break.borrow();
        let range_start = TextPosition::new(scr.0, scr.1);
        let y2 = scr.1 + scr.2;
        let range_end = TextPosition::new(0, min(y2, self.len_lines()));

        let mut start_pos = TextPosition::new(scr.0, scr.1);
        let mut scr_row = 0;
        for (_key, cache) in line_breaks.range(range_start..range_end) {
            if pos < cache.start_pos {
                return Some(scr_row);
            }
            scr_row += 1;
            start_pos = cache.start_pos;
        }

        // very last position on the very last row without a \n
        if pos == start_pos {
            return Some(scr_row);
        }

        None
    }

    // start offset for given screen-row. only if within `scr.2` pages.
    // requires: cache for `scr.2` pages
    fn stc_sub_row_offset(
        &self,
        scr: (upos_type, upos_type, upos_type),
        mut scr_row: upos_type,
    ) -> (upos_type, upos_type) {
        let line_breaks = self.value.cache().line_break.borrow();
        let range_start = TextPosition::new(scr.0, scr.1);
        let y2 = scr.1 + scr.2;
        let range_end = TextPosition::new(0, min(y2, self.len_lines()));

        let mut start_pos = (scr.0, scr.1);
        for (_key, cache) in line_breaks.range(range_start..range_end) {
            if scr_row == 0 {
                return start_pos;
            }
            scr_row -= 1;
            start_pos = (cache.start_pos.x, cache.start_pos.y);
        }

        // actual data is shorter than expected.
        start_pos
    }

    /// Return the screen_position for the given text position
    /// relative to the origin of the widget.
    ///
    /// This may be outside the visible area, if the text-area
    /// has been relocated. It may even be outside the screen,
    /// so this returns an (i16, i16) as an absolute screen position.
    ///
    /// If the text-position is outside the rendered area,
    /// this will return None.
    #[allow(clippy::explicit_counter_loop)]
    pub fn pos_to_relative_screen(&self, pos: impl Into<TextPosition>) -> Option<(i16, i16)> {
        let pos = pos.into();
        match self.text_wrap {
            TextWrap::Shift => {
                let (ox, _, oy) = self.clean_offset();

                if oy > self.len_lines() {
                    return None;
                }
                if pos.y < oy {
                    return None;
                }
                if pos.y > self.len_lines() {
                    return None;
                }
                if pos.y - oy >= self.rendered.height as u32 {
                    return None;
                }

                let screen_y = (pos.y - oy) as u16;

                let screen_x = 'f: {
                    for g in self
                        .glyphs2(ox, 0, pos.y..min(pos.y + 1, self.len_lines()))
                        .expect("valid-row")
                    {
                        if g.pos().x == pos.x {
                            break 'f g.screen_pos().0;
                        } else if g.line_break() {
                            break 'f g.screen_pos().0;
                        }
                    }
                    // last row
                    0
                };
                assert!(screen_x <= self.rendered.width);

                Some((
                    screen_x as i16 - self.dark_offset.0 as i16,
                    screen_y as i16 - self.dark_offset.1 as i16,
                ))
            }
            TextWrap::Hard | TextWrap::Word(_) => {
                let (_, sub_row_offset, oy) = self.clean_offset();

                if oy > self.len_lines() {
                    return None;
                }
                if pos.y < oy {
                    return None;
                }
                if pos.y > self.len_lines() {
                    return None;
                }

                let page = self.rendered.height as upos_type;
                let scr = (sub_row_offset, oy, page);
                self.stc_fill_screen_cache(scr);
                let (screen_y, start_pos) = if let Some(pos_row) = self.stc_screen_row(scr, pos) {
                    if pos_row >= page {
                        // beyond page
                        return None;
                    }
                    let start_pos = self.stc_sub_row_offset(scr, pos_row);
                    (pos_row, start_pos)
                } else {
                    // out of bounds
                    return None;
                };

                let screen_x = 'f: {
                    for g in self
                        .glyphs2(
                            0,
                            start_pos.0,
                            start_pos.1..min(start_pos.1 + 1, self.len_lines()),
                        )
                        .expect("valid-row")
                    {
                        if g.pos().x == pos.x {
                            break 'f g.screen_pos().0;
                        } else if g.line_break() {
                            break 'f g.screen_pos().0;
                        }
                    }
                    // no glyphs on this line
                    0
                };
                assert!(screen_x <= self.rendered.width);

                let scr = (
                    screen_x as i16 - self.dark_offset.0 as i16,
                    screen_y as i16 - self.dark_offset.1 as i16,
                );
                Some(scr)
            }
        }
    }

    /// Find the text-position for the widget-relative screen-position.
    #[allow(clippy::needless_return)]
    pub fn relative_screen_to_pos(&self, scr_pos: (i16, i16)) -> Option<TextPosition> {
        let scr_pos = (
            scr_pos.0 + self.dark_offset.0 as i16,
            scr_pos.1 + self.dark_offset.1 as i16,
        );

        match self.text_wrap {
            TextWrap::Shift => {
                let (ox, _, oy) = self.clean_offset();

                if oy >= self.len_lines() {
                    return None;
                }

                if scr_pos.1 < 0 {
                    // before the first visible line. fall back to col 0.
                    return Some(TextPosition::new(
                        0,
                        oy.saturating_add_signed(scr_pos.1 as ipos_type),
                    ));
                } else if (oy + scr_pos.1 as upos_type) >= self.len_lines() {
                    // after the last visible line. fall back to col 0.
                    return Some(TextPosition::new(0, self.len_lines().saturating_sub(1)));
                }

                let pos_y = oy + scr_pos.1 as upos_type;

                if scr_pos.0 < 0 {
                    return Some(TextPosition::new(
                        ox.saturating_add_signed(scr_pos.0 as ipos_type),
                        pos_y,
                    ));
                } else if scr_pos.0 as u16 >= self.rendered.width {
                    return Some(TextPosition::new(
                        min(ox + scr_pos.0 as upos_type, self.line_width(pos_y)),
                        pos_y,
                    ));
                } else {
                    let mut start_pos = TextPosition::new(0, pos_y);
                    for g in self
                        .glyphs2(ox, 0, pos_y..min(pos_y + 1, self.len_lines()))
                        .expect("valid-position")
                    {
                        if g.contains_screen_x(scr_pos.0 as u16) {
                            return Some(TextPosition::new(g.pos().x, pos_y));
                        }
                        start_pos = g.pos();
                    }
                    Some(start_pos)
                }
            }
            TextWrap::Hard | TextWrap::Word(_) => {
                let (_, sub_row_offset, oy) = self.clean_offset();

                if oy >= self.len_lines() {
                    return None;
                }

                if scr_pos.1 < 0 {
                    // Guess a starting position for an alternate screen that
                    // would contain the given screen-position.
                    // By locating our actual offset within that screen we can
                    // calculate the correct screen-position for that alternate
                    // screen. And then find the correct text-position for
                    // that again.

                    // estimate start row
                    let ry = oy.saturating_add_signed(scr_pos.1 as ipos_type - 1);

                    self.fill_cache(0, 0, ry..oy).expect("valid-rows");

                    let n_start_pos = 'f: {
                        let line_break = self.value.cache().line_break.borrow();
                        let start_range = TextPosition::new(0, ry);
                        let end_range = TextPosition::new(sub_row_offset, oy);

                        let mut nrows = scr_pos.1.unsigned_abs();
                        for (_break_pos, cache) in line_break.range(start_range..end_range).rev() {
                            if nrows == 0 {
                                break 'f cache.start_pos;
                            }
                            nrows -= 1;
                        }
                        TextPosition::new(0, ry)
                    };

                    // find the exact col
                    if scr_pos.0 < 0 {
                        return Some(n_start_pos);
                    }

                    let min_row = n_start_pos.y;
                    let max_row = min(n_start_pos.y + 1, self.len_lines());
                    for g in self
                        .glyphs2(0, n_start_pos.x, min_row..max_row)
                        .expect("valid-rows")
                    {
                        if g.contains_screen_x(scr_pos.0 as u16) {
                            return Some(g.pos());
                        }
                    }

                    // beyond the last line
                    return Some(n_start_pos);
                } else {
                    let scr_pos = (max(0, scr_pos.0) as u16, scr_pos.1 as u16);

                    // row-0 equals the current offset. done.
                    let n_start_pos = if scr_pos.1 == 0 {
                        TextPosition::new(sub_row_offset, oy)
                    } else {
                        // start at the offset and find the screen-position.
                        self.fill_cache(
                            0,
                            sub_row_offset,
                            oy..min(oy + scr_pos.1 as upos_type, self.len_lines()),
                        )
                        .expect("valid-rows");

                        'f: {
                            let text_range = self.value.cache().line_break.borrow();
                            let start_range = TextPosition::new(sub_row_offset, oy);
                            let end_range = TextPosition::new(0, self.len_lines());

                            let mut nrows = scr_pos.1 - 1;
                            let mut start_pos = TextPosition::new(sub_row_offset, oy);
                            for (_break_pos, cache) in text_range.range(start_range..end_range) {
                                if nrows == 0 {
                                    break 'f cache.start_pos;
                                }
                                start_pos = cache.start_pos;
                                nrows -= 1;
                            }
                            start_pos
                        }
                    };

                    let min_row = n_start_pos.y;
                    let max_row = min(n_start_pos.y + 1, self.len_lines());
                    for g in self
                        .glyphs2(0, n_start_pos.x, min_row..max_row)
                        .expect("valid-rows")
                    {
                        if g.contains_screen_x(scr_pos.0) {
                            return Some(g.pos());
                        }
                    }

                    // beyond the last line
                    return Some(n_start_pos);
                }
            }
        }
    }
}

impl TextAreaState {
    /// Converts from a widget relative screen coordinate to a line.
    /// It limits its result to a valid row.
    #[deprecated(since = "1.1.0", note = "replaced by relative_screen_to_pos()")]
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
    #[allow(deprecated)]
    #[deprecated(since = "1.1.0", note = "replaced by relative_screen_to_pos()")]
    pub fn screen_to_col(&self, row: upos_type, scx: i16) -> upos_type {
        self.try_screen_to_col(row, scx).expect("valid_row")
    }

    /// Converts from a widget relative screen coordinate to a grapheme index.
    /// It limits its result to a valid column.
    ///
    /// * row is a row-index into the value, not a screen-row. It can be calculated
    ///   with screen_to_row().
    /// * x is the relative screen position.
    #[allow(deprecated)]
    #[deprecated(since = "1.1.0", note = "replaced by relative_screen_to_pos()")]
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
    #[deprecated(since = "1.1.0", note = "replaced by pos_to_relative_screen()")]
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
    #[allow(deprecated)]
    #[deprecated(since = "1.1.0", note = "replaced by pos_to_relative_screen()")]
    pub fn col_to_screen(&self, pos: impl Into<TextPosition>) -> Option<u16> {
        self.try_col_to_screen(pos).expect("valid_pos")
    }

    /// Converts a grapheme based position to a screen position
    /// relative to the widget area.
    #[allow(deprecated)]
    #[deprecated(since = "1.1.0", note = "replaced by pos_to_relative_screen()")]
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
        let Some(cursor) = self.relative_screen_to_pos(cursor) else {
            return false;
        };
        if let Some(scr_cursor) = self.pos_to_relative_screen(cursor) {
            self.set_move_col(Some(scr_cursor.0));
        }
        self.set_cursor(cursor, extend_selection)
    }

    /// Set the cursor position from screen coordinates,
    /// rounds the position to the next word start/end.
    ///
    /// The cursor positions are relative to the inner rect.
    /// They may be negative too, this allows setting the cursor
    /// to a position that is currently scrolled away.
    pub fn set_screen_cursor_words(&mut self, cursor: (i16, i16), extend_selection: bool) -> bool {
        let Some(cursor) = self.relative_screen_to_pos(cursor) else {
            return false;
        };

        let anchor = self.anchor();
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

        self.set_cursor(cursor, extend_selection)
    }
}

impl TextAreaState {
    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is set to `len_lines - page_size` for shift-mode,
    /// and to `len_lines` for any wrapping-mode.
    pub fn vertical_max_offset(&self) -> usize {
        self.vscroll.max_offset()
    }

    /// Current vertical offset.
    pub fn vertical_offset(&self) -> usize {
        self.vscroll.offset()
    }

    /// Rendered height of the widget.
    pub fn vertical_page(&self) -> usize {
        self.vscroll.page_len()
    }

    /// Suggested scroll per scroll-event.
    pub fn vertical_scroll(&self) -> usize {
        self.vscroll.scroll_by()
    }

    /// Maximum horizontal offset.
    ///
    /// This is set to 0 when text-wrapping is active.
    /// Otherwise, it can be set manually, but is always
    /// ignored by all scroll-functions. This widget
    /// doesn't try to find an overall text-width.
    ///
    /// It __will__ be used to render the scrollbar though.
    pub fn horizontal_max_offset(&self) -> usize {
        self.hscroll.max_offset()
    }

    /// Current horizontal offset.
    pub fn horizontal_offset(&self) -> usize {
        self.hscroll.offset()
    }

    /// Rendered width of the text-area.
    pub fn horizontal_page(&self) -> usize {
        self.hscroll.page_len()
    }

    /// Suggested scroll-by per scroll-event.
    pub fn horizontal_scroll(&self) -> usize {
        self.hscroll.scroll_by()
    }

    /// Change the vertical offset.
    /// There is no limit to this offset.
    ///
    /// Return
    ///
    /// `true` if the offset changed at all.
    pub fn set_vertical_offset(&mut self, row_offset: usize) -> bool {
        self.scroll_to_cursor = false;
        self.sub_row_offset = 0;
        self.vscroll.set_offset(row_offset)
    }

    /// Change the horizontal offset.
    ///
    /// There is no limit to this offset. If there is text-wrapping
    /// this offset will be ignored.
    ///
    /// Return
    ///
    /// `true` if the offset changed at all.
    pub fn set_horizontal_offset(&mut self, col_offset: usize) -> bool {
        self.scroll_to_cursor = false;
        self.hscroll.set_offset(col_offset)
    }

    /// Scrolls to make the given position visible.
    pub fn scroll_to_pos(&mut self, pos: impl Into<TextPosition>) -> bool {
        let old_offset = self.clean_offset();

        let pos = pos.into();

        'f: {
            match self.text_wrap {
                TextWrap::Shift => {
                    let (ox, _, oy) = old_offset;

                    let height = self.rendered.height as upos_type;
                    let width = self.rendered.width as upos_type;
                    let width = if self.show_ctrl() || self.wrap_ctrl() {
                        width.saturating_sub(1)
                    } else {
                        width
                    };

                    let noy = if pos.y < oy.saturating_sub(height) {
                        pos.y.saturating_sub(height * 4 / 10)
                    } else if pos.y < oy {
                        pos.y
                    } else if pos.y >= oy + 2 * height {
                        pos.y.saturating_sub(height * 6 / 10)
                    } else if pos.y >= oy + height {
                        pos.y.saturating_sub(height.saturating_sub(1))
                    } else {
                        oy
                    };

                    let nox = if pos.x < ox {
                        pos.x
                    } else if pos.x >= ox + width {
                        pos.x.saturating_sub(width) + 1
                    } else {
                        ox
                    };

                    self.set_offset((nox as usize, noy as usize));
                    self.set_sub_row_offset(0);
                }
                TextWrap::Hard | TextWrap::Word(_) => {
                    let (_ox, sub_row_offset, oy) = old_offset;
                    let page = self.rendered.height as upos_type;

                    // on visible or close by
                    let scr = (0, oy.saturating_sub(page), 3 * page);
                    self.stc_fill_screen_cache(scr);
                    if let Some(off_row) =
                        self.stc_screen_row(scr, TextPosition::new(sub_row_offset, oy))
                    {
                        if let Some(pos_row) = self.stc_screen_row(scr, pos) {
                            if pos_row < off_row && pos_row >= off_row.saturating_sub(page) {
                                let noff_row = pos_row;
                                let (nsub_row_offset, noy) = self.stc_sub_row_offset(scr, noff_row);
                                self.set_offset((0, noy as usize));
                                self.set_sub_row_offset(nsub_row_offset);
                                break 'f;
                            } else if pos_row >= off_row + page && pos_row < off_row + 2 * page {
                                let noff_row = pos_row.saturating_sub(page.saturating_sub(1));
                                let (nsub_row_offset, noy) = self.stc_sub_row_offset(scr, noff_row);
                                self.set_offset((0, noy as usize));
                                self.set_sub_row_offset(nsub_row_offset);
                                break 'f;
                            } else if pos_row >= off_row && pos_row < off_row + page {
                                break 'f;
                            }
                        }
                    }

                    // long jump. center position.
                    let alt_scr = (0, pos.y.saturating_sub(page), 3 * page);
                    self.stc_fill_screen_cache(alt_scr);
                    if let Some(alt_scr_row) = self.stc_screen_row(alt_scr, pos) {
                        let noff_row = alt_scr_row.saturating_sub(page * 5 / 10);
                        let (nsub_row_offset, noy) = self.stc_sub_row_offset(alt_scr, noff_row);
                        self.set_offset((0, noy as usize));
                        self.set_sub_row_offset(nsub_row_offset);
                    } else {
                        self.set_offset((0, pos.y as usize));
                        self.set_sub_row_offset(0);
                    }
                }
            }
        }

        old_offset != self.clean_offset()
    }

    /// Scrolls to make the given row visible.
    ///
    /// Adjusts the offset just enough to make this happen.
    /// Does nothing if the position is already visible.
    ///
    /// Return
    ///
    /// `true` if the offset changed.
    pub fn scroll_to_row(&mut self, pos: usize) -> bool {
        self.scroll_to_cursor = false;

        match self.text_wrap {
            TextWrap::Shift => self.vscroll.scroll_to_pos(pos),
            TextWrap::Hard | TextWrap::Word(_) => {
                self.vscroll.set_offset(self.vscroll.limited_offset(pos))
            }
        }
    }

    /// Scroll to make the given column visible.
    ///
    /// This scroll-offset is ignored if there is any text-wrapping.
    ///
    /// Return
    ///
    /// `true` if the offset changed.
    pub fn scroll_to_col(&mut self, pos: usize) -> bool {
        self.scroll_to_cursor = false;
        self.hscroll.set_offset(pos)
    }

    /// Scroll up by `delta` rows.
    ///
    /// Return
    ///
    /// `true` if the offset changes.
    pub fn scroll_up(&mut self, delta: usize) -> bool {
        if let Some(pos) = self.relative_screen_to_pos((0, -(delta as i16))) {
            self.sub_row_offset = pos.x;
            self.vscroll.set_offset(pos.y as usize);
            true
        } else {
            false
        }
    }

    /// Scroll down by `delta` rows.
    ///
    /// Return
    ///
    /// `true` if the offset changes.
    pub fn scroll_down(&mut self, delta: usize) -> bool {
        if let Some(pos) = self.relative_screen_to_pos((0, delta as i16)) {
            self.sub_row_offset = pos.x;
            self.vscroll.set_offset(pos.y as usize);
            true
        } else {
            false
        }
    }

    /// Scroll left by `delta` columns.
    ///
    /// This ignores the max_offset, as that is never correct anyway.
    ///
    /// __Return__
    ///
    /// `true` if the offset changes.
    ///
    /// TODO: Does nothing if there is any text-wrapping.
    pub fn scroll_left(&mut self, delta: usize) -> bool {
        self.hscroll
            .set_offset(self.hscroll.offset.saturating_add(delta))
    }

    /// Scroll right by `delta` columns.
    ///
    /// This ignores the max_offset, as that is never correct anyway.
    ///
    /// __Return__
    ///
    /// `true`if the offset changes.
    ///
    /// TODO: Does nothing if there is any text-wrapping.
    pub fn scroll_right(&mut self, delta: usize) -> bool {
        self.hscroll
            .set_offset(self.hscroll.offset.saturating_sub(delta))
    }

    #[deprecated(since = "1.3.0", note = "not useful as is")]
    pub fn scroll_sub_row_offset(&mut self, col: upos_type) -> bool {
        if let Ok(max_col) = self.try_line_width(self.offset().1 as upos_type) {
            self.sub_row_offset = min(col as upos_type, max_col);
        } else {
            self.sub_row_offset = 0;
        }
        true
    }
}

impl TextAreaState {
    /// Scroll that the cursor is visible.
    ///
    /// This positioning happens with the next render.
    ///
    /// All move-fn do this automatically.
    pub fn scroll_cursor_to_visible(&mut self) {
        self.scroll_to_cursor = true;
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

/// Style-id for search matches.
pub const MATCH_STYLE: usize = 100_001;

impl HandleEvent<crossterm::event::Event, ReadOnly, TextOutcome> for TextAreaState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: ReadOnly) -> TextOutcome {
        let mut r = if self.is_focused() {
            match event {
                ct_event!(keycode press Left) => self.move_left(1, false).into(),
                ct_event!(keycode press Right) => self.move_right(1, false).into(),
                ct_event!(keycode press Up) => self.move_up(1, false).into(),
                ct_event!(keycode press Down) => self.move_down(1, false).into(),
                ct_event!(keycode press PageUp) => {
                    self.move_up(self.vertical_page() as u16, false).into()
                }
                ct_event!(keycode press PageDown) => {
                    self.move_down(self.vertical_page() as u16, false).into()
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
                    self.move_up(self.vertical_page() as u16, true).into()
                }
                ct_event!(keycode press SHIFT-PageDown) => {
                    self.move_down(self.vertical_page() as u16, true).into()
                }
                ct_event!(keycode press SHIFT-Home) => self.move_to_line_start(true).into(),
                ct_event!(keycode press SHIFT-End) => self.move_to_line_end(true).into(),
                ct_event!(keycode press CONTROL_SHIFT-Left) => self.move_to_prev_word(true).into(),
                ct_event!(keycode press CONTROL_SHIFT-Right) => self.move_to_next_word(true).into(),
                ct_event!(keycode press CONTROL_SHIFT-Home) => self.move_to_start(true).into(),
                ct_event!(keycode press CONTROL_SHIFT-End) => self.move_to_end(true).into(),
                ct_event!(key press CONTROL-'a') => self.select_all().into(),
                ct_event!(key press CONTROL-'c') => self.copy_to_clip().into(),

                ct_event!(keycode press F(3)) => self.move_to_next_match().into(),
                ct_event!(keycode press SHIFT-F(3)) => self.move_to_prev_match().into(),

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
                if let Some(test) = self.screen_to_pos((m.column, m.row)) {
                    let start = self.word_start(test);
                    let end = self.word_end(test);
                    self.set_selection(start, end).into()
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
                    TextOutcome::Continue
                }
            }
            ct_event!(mouse down SHIFT-Left for column,row) => {
                if self.inner.contains((*column, *row).into()) {
                    let cx = (column - self.inner.x) as i16;
                    let cy = (row - self.inner.y) as i16;
                    self.set_screen_cursor((cx, cy), true).into()
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
            ScrollOutcome::VPos(v) => self.scroll_to_row(v),
            ScrollOutcome::HPos(v) => self.scroll_to_col(v),
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
