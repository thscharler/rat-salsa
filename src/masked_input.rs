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
//! The visual cursor must be set separately after rendering.
//! It is accessible as [TextInputState::screen_cursor()] after rendering.
//!
//! Event handling by calling the freestanding fn [handle_events].
//! There's [handle_mouse_events] if you want to override the default key bindings but keep
//! the mouse behaviour.
//!

use crate::_private::NonExhaustive;
use crate::clipboard::Clipboard;
use crate::event::{ReadOnly, TextOutcome};
use crate::masked_input::masked::MaskedCore;
use crate::text_input::{TextInputState, TextInputStyle};
use crate::undo_buffer::{UndoBuffer, UndoEntry};
use crate::{ipos_type, upos_type, Cursor, Glyph, Grapheme, TextError};
use format_num_pattern::NumberSymbols;
use rat_event::util::MouseFlags;
use rat_event::{ct_event, HandleEvent, MouseOnly, Regular};
use rat_focus::{FocusFlag, HasFocusFlag};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{BlockExt, StatefulWidget, Style, Stylize, Widget};
use ratatui::widgets::{Block, StatefulWidgetRef};
use std::borrow::Cow;
use std::cmp::min;
use std::fmt;
use std::ops::Range;

/// Text input widget with input mask.
#[derive(Debug, Default, Clone)]
pub struct MaskedInput<'a> {
    compact: bool,
    block: Option<Block<'a>>,
    style: Style,
    focus_style: Option<Style>,
    select_style: Option<Style>,
    invalid_style: Option<Style>,
    text_style: Vec<Style>,
}

/// State & event-handling.
#[derive(Debug, Clone)]
pub struct MaskedInputState {
    /// Current focus state.
    pub focus: FocusFlag,
    /// Display as invalid.
    pub invalid: bool,

    /// The whole area with block.
    pub area: Rect,
    /// Area inside a possible block.
    pub inner: Rect,

    /// Display offset
    pub offset: upos_type,
    /// Editing core
    pub core: MaskedCore,

    /// Mouse selection in progress.
    pub mouse: MouseFlags,
    /// Construct with `..Default::default()`
    pub non_exhaustive: NonExhaustive,
}

impl<'a> MaskedInput<'a> {
    /// New
    pub fn new() -> Self {
        Self::default()
    }

    /// Show a compact form of the content without unnecessary spaces,
    /// if this widget is not focused.
    #[inline]
    pub fn compact(mut self, show_compact: bool) -> Self {
        self.compact = show_compact;
        self
    }

    /// Set the combined style.
    #[inline]
    pub fn styles(mut self, style: TextInputStyle) -> Self {
        self.style = style.style;
        self.focus_style = style.focus;
        self.select_style = style.select;
        self.invalid_style = style.invalid;
        self
    }

    /// Base text style.
    #[inline]
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Style when focused.
    #[inline]
    pub fn focus_style(mut self, style: impl Into<Style>) -> Self {
        self.focus_style = Some(style.into());
        self
    }

    /// Style for selection
    #[inline]
    pub fn select_style(mut self, style: impl Into<Style>) -> Self {
        self.select_style = Some(style.into());
        self
    }

    /// Style for the invalid indicator.
    /// This is patched onto either base_style or focus_style
    #[inline]
    pub fn invalid_style(mut self, style: impl Into<Style>) -> Self {
        self.invalid_style = Some(style.into());
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
}

impl<'a> StatefulWidgetRef for MaskedInput<'a> {
    type State = MaskedInputState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

impl<'a> StatefulWidget for MaskedInput<'a> {
    type State = MaskedInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

fn render_ref(
    widget: &MaskedInput<'_>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut MaskedInputState,
) {
    state.area = area;
    state.inner = widget.block.inner_if_some(area);

    widget.block.render(area, buf);

    let inner = state.inner;

    let focus_style = if let Some(focus_style) = widget.focus_style {
        focus_style
    } else {
        widget.style
    };
    let select_style = if let Some(select_style) = widget.select_style {
        select_style
    } else {
        Style::default().on_yellow()
    };
    let invalid_style = if let Some(invalid_style) = widget.invalid_style {
        invalid_style
    } else {
        Style::default().red()
    };

    let (style, select_style) = if state.focus.get() {
        if state.invalid {
            (
                focus_style.patch(invalid_style),
                select_style.patch(invalid_style),
            )
        } else {
            (focus_style, select_style)
        }
    } else {
        if state.invalid {
            (
                widget.style.patch(invalid_style),
                widget.style.patch(invalid_style),
            )
        } else {
            (widget.style, widget.style)
        }
    };

    // set base style
    for y in inner.top()..inner.bottom() {
        for x in inner.left()..inner.right() {
            let cell = buf.get_mut(x, y);
            cell.reset();
            cell.set_style(style);
        }
    }

    let ox = state.offset() as u16;
    // this is just a guess at the display-width
    let show_range = {
        let start = ox as upos_type;
        let end = min(start + inner.width as upos_type, state.len());
        state.bytes_at_range(start..end).expect("valid_range")
    };
    let selection = state.selection();
    let mut styles = Vec::new();

    let mut glyph_iter_regular;
    let mut glyph_iter_cond;
    let glyph_iter: &mut dyn Iterator<Item = Glyph<'_>>;
    if !state.is_focused() && widget.compact {
        glyph_iter_regular = state
            .core
            .glyphs(0..1, ox, inner.width)
            .expect("valid_offset");
        glyph_iter = &mut glyph_iter_regular;
    } else {
        glyph_iter_cond = state
            .core
            .condensed_glyphs(0..1, ox, inner.width)
            .expect("valid_offset");
        glyph_iter = &mut glyph_iter_cond;
    }

    for g in glyph_iter {
        if g.screen_width() > 0 {
            let mut style = style;
            styles.clear();
            state
                .core
                .styles_at_page(show_range.clone(), g.text_bytes().start, &mut styles);
            for style_nr in &styles {
                if let Some(s) = widget.text_style.get(*style_nr) {
                    style = style.patch(*s);
                }
            }
            // selection
            if selection.contains(&g.pos().x) {
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

impl Default for MaskedInputState {
    fn default() -> Self {
        Self {
            focus: Default::default(),
            invalid: false,
            area: Default::default(),
            inner: Default::default(),
            mouse: Default::default(),
            core: Default::default(),
            non_exhaustive: NonExhaustive,
            offset: 0,
        }
    }
}

impl HasFocusFlag for MaskedInputState {
    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }

    // todo: leavefwd
}

impl MaskedInputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn named(name: &str) -> Self {
        Self {
            focus: FocusFlag::named(name),
            ..MaskedInputState::default()
        }
    }

    /// With localized symbols for number formatting.
    #[inline]
    pub fn with_symbols(mut self, sym: NumberSymbols) -> Self {
        self.set_num_symbols(sym);
        self
    }

    /// With input mask.
    pub fn with_mask<S: AsRef<str>>(mut self, mask: S) -> Result<Self, fmt::Error> {
        self.core.set_mask(mask.as_ref())?;
        Ok(self)
    }

    /// Set symbols for number display.
    ///
    /// These are only used for rendering and to map user input.
    /// The value itself uses ".", "," and "-".
    #[inline]
    pub fn set_num_symbols(&mut self, sym: NumberSymbols) {
        self.core.set_num_symbols(sym);
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
    #[inline]
    pub fn set_mask<S: AsRef<str>>(&mut self, s: S) -> Result<(), fmt::Error> {
        self.core.set_mask(s)
    }

    /// Display mask.
    #[inline]
    pub fn mask(&self) -> String {
        self.core.mask()
    }

    /// Renders the widget in invalid style.
    #[inline]
    pub fn set_invalid(&mut self, invalid: bool) {
        self.invalid = invalid;
    }

    /// Renders the widget in invalid style.
    #[inline]
    pub fn get_invalid(&self) -> bool {
        self.invalid
    }
}

impl MaskedInputState {
    /// Clipboard
    pub fn set_clipboard(&mut self, clip: Option<impl Clipboard + 'static>) {
        match clip {
            None => self.core.set_clipboard(None),
            Some(v) => self.core.set_clipboard(Some(Box::new(v))),
        }
    }

    /// Clipboard
    pub fn clipboard(&self) -> Option<&dyn Clipboard> {
        self.core.clipboard()
    }

    /// Copy to internal buffer
    pub fn copy_to_clip(&mut self) -> bool {
        let Some(clip) = self.core.clipboard() else {
            return false;
        };

        _ = clip.set_string(self.selected_text().as_ref());

        true
    }

    /// Cut to internal buffer
    pub fn cut_to_clip(&mut self) -> bool {
        let Some(clip) = self.core.clipboard() else {
            return false;
        };

        match clip.set_string(self.selected_text().as_ref()) {
            Ok(_) => self
                .delete_range(self.selection())
                .expect("valid_selection"),
            Err(_) => true,
        }
    }

    /// Paste from internal buffer.
    pub fn paste_from_clip(&mut self) -> bool {
        let Some(clip) = self.core.clipboard() else {
            return false;
        };

        if let Ok(text) = clip.get_string() {
            for c in text.chars() {
                self.insert_char(c);
            }
            true
        } else {
            false
        }
    }
}

impl MaskedInputState {
    /// Set undo buffer.
    pub fn set_undo_buffer(&mut self, undo: Option<impl UndoBuffer + 'static>) {
        match undo {
            None => self.core.set_undo_buffer(None),
            Some(v) => self.core.set_undo_buffer(Some(Box::new(v))),
        }
    }

    /// Undo
    #[inline]
    pub fn undo_buffer(&self) -> Option<&dyn UndoBuffer> {
        self.core.undo_buffer()
    }

    /// Undo
    #[inline]
    pub fn undo_buffer_mut(&mut self) -> Option<&mut dyn UndoBuffer> {
        self.core.undo_buffer_mut()
    }

    /// Get all recent replay recordings.
    pub fn recent_replay_log(&mut self) -> Vec<UndoEntry> {
        self.core.recent_replay_log()
    }

    /// Apply the replay recording.
    pub fn replay_log(&mut self, replay: &[UndoEntry]) {
        self.core.replay_log(replay)
    }

    /// Undo operation
    pub fn undo(&mut self) -> bool {
        self.core.undo()
    }

    /// Redo operation
    pub fn redo(&mut self) -> bool {
        self.core.redo()
    }
}

impl MaskedInputState {
    /// Set and replace all styles.
    #[inline]
    pub fn set_styles(&mut self, styles: Vec<(Range<usize>, usize)>) {
        self.core.set_styles(styles);
    }

    /// Add a style for a [TextRange]. The style-nr refers to one
    /// of the styles set with the widget.
    #[inline]
    pub fn add_style(&mut self, range: Range<usize>, style: usize) {
        self.core.add_style(range.into(), style);
    }

    /// Add a style for a Range<upos_type> to denote the cells.
    /// The style-nr refers to one of the styles set with the widget.
    #[inline]
    pub fn add_range_style(
        &mut self,
        range: Range<upos_type>,
        style: usize,
    ) -> Result<(), TextError> {
        let r = self.core.bytes_at_range(range)?;
        self.core.add_style(r, style);
        Ok(())
    }

    /// Remove the exact TextRange and style.
    #[inline]
    pub fn remove_style(&mut self, range: Range<usize>, style: usize) {
        self.core.remove_style(range.into(), style);
    }

    /// Remove the exact Range<upos_type> and style.
    #[inline]
    pub fn remove_range_style(
        &mut self,
        range: Range<upos_type>,
        style: usize,
    ) -> Result<(), TextError> {
        let r = self.core.bytes_at_range(range)?;
        self.core.remove_style(r, style);
        Ok(())
    }

    /// All styles active at the given position.
    #[inline]
    pub fn styles_at(&self, byte_pos: usize, buf: &mut Vec<usize>) {
        self.core.styles_at(byte_pos, buf)
    }

    /// Check if the given style applies at the position and
    /// return the complete range for the style.
    #[inline]
    pub fn style_match(&self, byte_pos: usize, style: usize) -> Option<Range<usize>> {
        self.core.style_match(byte_pos, style.into())
    }

    /// List of all styles.
    #[inline]
    pub fn styles(&self) -> Option<impl Iterator<Item = (Range<usize>, usize)> + '_> {
        self.core.styles()
    }
}

impl MaskedInputState {
    /// Offset shown.
    #[inline]
    pub fn offset(&self) -> upos_type {
        self.offset
    }

    /// Offset shown. This is corrected if the cursor wouldn't be visible.
    #[inline]
    pub fn set_offset(&mut self, offset: upos_type) {
        self.offset = offset;
    }

    /// Cursor position.
    #[inline]
    pub fn cursor(&self) -> upos_type {
        self.core.cursor()
    }

    /// Set the cursor position, reset selection.
    #[inline]
    pub fn set_cursor(&mut self, cursor: upos_type, extend_selection: bool) -> bool {
        self.core.set_cursor(cursor, extend_selection)
    }

    /// Place cursor at the decimal separator, if any.
    /// 0 otherwise.
    #[inline]
    pub fn set_default_cursor(&mut self) {
        self.core.set_default_cursor();
    }

    /// Selection anchor.
    #[inline]
    pub fn anchor(&self) -> upos_type {
        self.core.anchor()
    }

    /// Selection.
    #[inline]
    pub fn has_selection(&self) -> bool {
        self.core.has_selection()
    }

    /// Selection.
    #[inline]
    pub fn selection(&self) -> Range<upos_type> {
        self.core.selection()
    }

    /// Selection.
    #[inline]
    pub fn set_selection(&mut self, anchor: upos_type, cursor: upos_type) -> bool {
        self.core.set_selection(anchor, cursor)
    }

    /// Selection.
    #[inline]
    pub fn select_all(&mut self) -> bool {
        self.core.select_all()
    }

    /// Selection.
    #[inline]
    pub fn selected_text(&self) -> &str {
        self.core.selected_text()
    }
}

impl MaskedInputState {
    /// Empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.core.is_empty()
    }

    /// Value with all punctuation and default values according to the mask type.
    #[inline]
    pub fn text(&self) -> &str {
        self.core.text()
    }

    /// Text slice as Cow<str>
    #[inline]
    pub fn str_slice(&self, range: Range<upos_type>) -> Result<Cow<'_, str>, TextError> {
        self.core.str_slice(range)
    }

    /// Length as grapheme count.
    #[inline]
    pub fn len(&self) -> upos_type {
        self.core.line_width()
    }

    /// Length as grapheme count.
    #[inline]
    pub fn line_width(&self) -> upos_type {
        self.core.line_width()
    }

    /// Iterator for the glyphs of the lines in range.
    /// Glyphs here a grapheme + display length.
    #[inline]
    pub fn glyphs(
        &self,
        screen_offset: u16,
        screen_width: u16,
    ) -> Result<impl Iterator<Item = Glyph<'_>>, TextError> {
        self.core.glyphs(0..1, screen_offset, screen_width)
    }

    /// Iterator for the glyphs of the lines in range.
    /// Glyphs here a grapheme + display length.
    #[inline]
    pub fn condensed_glyphs(
        &self,
        screen_offset: u16,
        screen_width: u16,
    ) -> Result<impl Iterator<Item = Glyph<'_>>, TextError> {
        self.core
            .condensed_glyphs(0..1, screen_offset, screen_width)
    }

    /// Get a cursor over all the text with the current position set at pos.
    #[inline]
    pub fn text_graphemes(
        &self,
        pos: upos_type,
    ) -> Result<impl Iterator<Item = Grapheme<'_>> + Cursor, TextError> {
        self.core.text_graphemes(pos)
    }

    /// Get a cursor over the text-range the current position set at pos.
    #[inline]
    pub fn graphemes(
        &self,
        range: Range<upos_type>,
        pos: upos_type,
    ) -> Result<impl Iterator<Item = Grapheme<'_>> + Cursor, TextError> {
        self.core.graphemes(range, pos)
    }

    /// Grapheme position to byte position.
    /// This is the (start,end) position of the single grapheme after pos.
    #[inline]
    pub fn byte_at(&self, pos: upos_type) -> Result<Range<usize>, TextError> {
        self.core.byte_at(pos)
    }

    /// Grapheme range to byte range.
    #[inline]
    pub fn bytes_at_range(&self, range: Range<upos_type>) -> Result<Range<usize>, TextError> {
        self.core.bytes_at_range(range)
    }

    /// Byte position to grapheme position.
    /// Returns the position that contains the given byte index.
    #[inline]
    pub fn byte_pos(&self, byte: usize) -> Result<upos_type, TextError> {
        self.core.byte_pos(byte)
    }

    /// Byte range to grapheme range.
    #[inline]
    pub fn byte_range(&self, bytes: Range<usize>) -> Result<Range<upos_type>, TextError> {
        self.core.byte_range(bytes)
    }
}

impl MaskedInputState {
    /// Reset to empty.
    #[inline]
    pub fn clear(&mut self) -> bool {
        if self.is_empty() {
            false
        } else {
            self.offset = 0;
            self.core.clear();
            true
        }
    }

    /// Set the value.
    ///
    /// No checks if the value conforms to the mask.
    /// If the value is too short it will be filled with space.
    /// if the value is too long it will be truncated.
    #[inline]
    pub fn set_text<S: Into<String>>(&mut self, s: S) {
        self.offset = 0;
        self.core.set_text(s);
    }

    /// Insert a char at the current position.
    #[inline]
    pub fn insert_char(&mut self, c: char) -> bool {
        if self.core.has_selection() {
            self.core
                .remove_range(self.core.selection())
                .expect("valid_selection");
        }
        self.core.advance_cursor(c);
        self.core.insert_char(c);
        self.scroll_cursor_to_visible();
        true
    }

    /// Remove the selected range. The text will be replaced with the default value
    /// as defined by the mask.
    #[inline]
    pub fn delete_range(&mut self, range: Range<upos_type>) -> Result<bool, TextError> {
        if !range.is_empty() {
            self.core.remove_range(range)?;
            self.scroll_cursor_to_visible();
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MaskedInputState {
    /// Delete the char after the cursor.
    #[inline]
    pub fn delete_next_char(&mut self) -> bool {
        if self.has_selection() {
            self.delete_range(self.selection())
                .expect("valid_selection")
        } else if self.cursor() == self.len() {
            false
        } else {
            self.core.remove_next();
            self.scroll_cursor_to_visible();
            true
        }
    }

    /// Delete the char before the cursor.
    #[inline]
    pub fn delete_prev_char(&mut self) -> bool {
        if self.has_selection() {
            self.delete_range(self.selection())
                .expect("valid_selection")
        } else if self.cursor() == 0 {
            false
        } else {
            self.core.remove_prev();
            self.scroll_cursor_to_visible();
            true
        }
    }

    /// Move to the next char.
    #[inline]
    pub fn move_right(&mut self, extend_selection: bool) -> bool {
        let c = min(self.cursor() + 1, self.len());
        let c = self.set_cursor(c, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move to the previous char.
    #[inline]
    pub fn move_left(&mut self, extend_selection: bool) -> bool {
        let c = self.cursor().saturating_sub(1);
        let c = self.set_cursor(c, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Start of line
    #[inline]
    pub fn move_to_line_start(&mut self, extend_selection: bool) -> bool {
        let c = self.set_cursor(0, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// End of line
    #[inline]
    pub fn move_to_line_end(&mut self, extend_selection: bool) -> bool {
        let c = self.len();
        let c = self.set_cursor(c, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }
}

impl MaskedInputState {
    /// Converts a grapheme based position to a screen position
    /// relative to the widget area.
    pub fn col_to_screen(&self, pos: upos_type) -> Result<u16, TextError> {
        let ox = self.offset();

        if pos < ox {
            return Ok(0);
        }

        let line = self.glyphs(ox as u16, self.inner.width)?;
        let mut screen_x = 0;
        for g in line {
            if g.pos().x >= pos {
                break;
            }
            screen_x = g.screen_pos().0 + g.screen_width();
        }
        Ok(screen_x)
    }

    /// Converts from a widget relative screen coordinate to a grapheme index.
    /// x is the relative screen position.
    pub fn screen_to_col(&self, scx: i16) -> upos_type {
        let ox = self.offset();

        if scx < 0 {
            ox.saturating_sub((scx as ipos_type).abs() as upos_type)
        } else if scx as u16 >= self.inner.width {
            min(ox + scx as upos_type, self.len())
        } else {
            let scx = scx as u16;

            let line = self.glyphs(ox as u16, self.inner.width).expect("valid_row");

            let mut col = ox;
            for g in line {
                col = g.pos().x;
                if scx < g.screen_pos().0 + g.screen_width() {
                    break;
                }
            }
            col
        }
    }

    /// Set the cursor position from a screen position relative to the origin
    /// of the widget. This value can be negative, which selects a currently
    /// not visible position and scrolls to it.
    #[inline]
    pub fn set_screen_cursor(&mut self, cursor: i16, extend_selection: bool) -> bool {
        let scx = cursor;

        let cx = self.screen_to_col(scx);

        let c = self.set_cursor(cx, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// The current text cursor as an absolute screen position.
    #[inline]
    pub fn screen_cursor(&self) -> Option<(u16, u16)> {
        if self.is_focused() {
            let cx = self.cursor();
            let ox = self.offset();

            if cx < ox {
                None
            } else if cx > ox + self.inner.width as upos_type {
                None
            } else {
                let sc = self.col_to_screen(cx).expect("valid_cursor");
                Some((self.inner.x + sc, self.inner.y))
            }
        } else {
            None
        }
    }

    /// Scrolling
    pub fn scroll_left(&mut self, delta: upos_type) -> bool {
        self.set_offset(self.offset.saturating_sub(delta));
        true
    }

    /// Scrolling
    pub fn scroll_right(&mut self, delta: upos_type) -> bool {
        self.set_offset(self.offset + delta);
        true
    }

    /// Change the offset in a way that the cursor is visible.
    pub fn scroll_cursor_to_visible(&mut self) -> bool {
        let old_offset = self.offset();

        let c = self.cursor();
        let o = self.offset();

        let no = if c < o {
            c
        } else if c >= o + self.inner.width as upos_type {
            c.saturating_sub(self.inner.width as upos_type)
        } else {
            o
        };

        self.set_offset(no);

        self.offset() != old_offset
    }
}

mod mask {
    use crate::upos_type;
    use std::fmt;
    use std::fmt::{Debug, Display, Formatter};
    use std::ops::Range;

    /// Edit direction for part of a mask.
    /// Numeric values can switch between right-to-left (integer part) and left-to-right (fraction).
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub(super) enum EditDirection {
        Ltor,
        Rtol,
    }

    /// One char of the input mask.
    #[allow(variant_size_differences)]
    #[derive(Clone, PartialEq, Eq, Default)]
    #[non_exhaustive]
    pub(super) enum Mask {
        Digit0(EditDirection),
        Digit(EditDirection),
        Numeric(EditDirection),
        DecimalSep,
        GroupingSep,
        Sign,
        Plus,
        Hex0,
        Hex,
        Oct0,
        Oct,
        Dec0,
        Dec,
        Letter,
        LetterOrDigit,
        LetterDigitSpace,
        AnyChar,
        Separator(Box<str>),
        #[default]
        None,
    }

    /// One token of the input mask.
    ///
    /// Which field of the mask does this belong to:
    /// * Number with integer part, decimal separator, fraction and sign.
    /// * Consecutive mask parts of similar type.
    ///
    /// At this cursor position, what is the main mask (right) and what is possible left of
    /// the cursor position (peek_left).
    ///
    /// Default-values for editing and display.
    #[derive(Clone, PartialEq, Eq)]
    #[non_exhaustive]
    pub(super) struct MaskToken {
        pub nr_id: u16,
        pub nr_start: upos_type,
        pub nr_end: upos_type,

        pub sec_id: u16,
        pub sec_start: upos_type,
        pub sec_end: upos_type,

        pub peek_left: Mask,
        pub right: Mask,
        pub edit: Box<str>,
    }

    impl Debug for EditDirection {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    EditDirection::Ltor => ">",
                    EditDirection::Rtol => "<",
                }
            )
        }
    }

    impl Display for Mask {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            let s = match self {
                Mask::Digit0(_) => "0",
                Mask::Digit(_) => "9",
                Mask::Numeric(_) => "#",
                Mask::DecimalSep => ".",
                Mask::GroupingSep => ",",
                Mask::Sign => "-",
                Mask::Plus => "+",
                Mask::Hex0 => "H",
                Mask::Hex => "h",
                Mask::Oct0 => "O",
                Mask::Oct => "o",
                Mask::Dec0 => "D",
                Mask::Dec => "d",
                Mask::Letter => "l",
                Mask::LetterOrDigit => "a",
                Mask::LetterDigitSpace => "c",
                Mask::AnyChar => "_",
                Mask::Separator(s) => {
                    if matches!(
                        s.as_ref(),
                        "0" | "9"
                            | "#"
                            | "."
                            | ","
                            | "-"
                            | "+"
                            | "H"
                            | "h"
                            | "O"
                            | "o"
                            | "D"
                            | "d"
                            | "l"
                            | "a"
                            | "c"
                            | "_"
                    ) {
                        write!(f, "\\")?;
                    }
                    s
                }
                Mask::None => "",
            };
            write!(f, "{}", s)
        }
    }

    impl Debug for Mask {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                Mask::Digit0(d) => {
                    write!(f, "{:?}0", d)
                }
                Mask::Digit(d) => {
                    write!(f, "{:?}9", d)
                }
                Mask::Numeric(d) => {
                    write!(f, "{:?}#", d)
                }
                Mask::DecimalSep => write!(f, "."),
                Mask::GroupingSep => write!(f, ","),
                Mask::Sign => write!(f, "-"),
                Mask::Plus => write!(f, "+"),
                Mask::Hex0 => write!(f, "H"),
                Mask::Hex => write!(f, "h"),
                Mask::Oct0 => write!(f, "O"),
                Mask::Oct => write!(f, "o"),
                Mask::Dec0 => write!(f, "D"),
                Mask::Dec => write!(f, "d"),
                Mask::Letter => write!(f, "l"),
                Mask::LetterOrDigit => write!(f, "a"),
                Mask::LetterDigitSpace => write!(f, "c"),
                Mask::AnyChar => write!(f, "_"),
                Mask::Separator(s) => {
                    if matches!(
                        s.as_ref(),
                        "0" | "9"
                            | "#"
                            | "."
                            | ","
                            | "-"
                            | "+"
                            | "H"
                            | "h"
                            | "O"
                            | "o"
                            | "D"
                            | "d"
                            | "l"
                            | "a"
                            | "c"
                            | "_"
                    ) {
                        write!(f, "\\")?;
                    }
                    write!(f, "{}", s)
                }
                Mask::None => write!(f, ""),
            }
        }
    }

    impl EditDirection {
        pub(super) fn is_ltor(&self) -> bool {
            *self == EditDirection::Ltor
        }

        pub(super) fn is_rtol(&self) -> bool {
            *self == EditDirection::Rtol
        }
    }

    impl Mask {
        /// is not editable. the last field of the mask at position txt.len() can not be edited,
        /// but it's a valid cursor position.
        pub(super) fn is_none(&self) -> bool {
            *self == Mask::None
        }

        /// is a number mask
        #[inline]
        pub(super) fn is_number(&self) -> bool {
            match self {
                Mask::Digit0(_) => true,
                Mask::Digit(_) => true,
                Mask::Numeric(_) => true,
                Mask::DecimalSep => true,
                Mask::GroupingSep => true,
                Mask::Sign => true,
                Mask::Plus => true,

                Mask::Hex0 => false,
                Mask::Hex => false,
                Mask::Oct0 => false,
                Mask::Oct => false,
                Mask::Dec0 => false,
                Mask::Dec => false,
                Mask::Letter => false,
                Mask::LetterOrDigit => false,
                Mask::LetterDigitSpace => false,
                Mask::AnyChar => false,
                Mask::Separator(_) => false,
                Mask::None => false,
            }
        }

        /// left to right editing
        #[inline]
        pub(super) fn is_ltor(&self) -> bool {
            match self {
                Mask::Digit0(d) => d.is_ltor(),
                Mask::Digit(d) => d.is_ltor(),
                Mask::Numeric(d) => d.is_ltor(),
                Mask::GroupingSep => false,
                Mask::Sign => false,
                Mask::Plus => false,
                Mask::DecimalSep => true,
                Mask::Hex0 => true,
                Mask::Hex => true,
                Mask::Oct0 => true,
                Mask::Oct => true,
                Mask::Dec0 => true,
                Mask::Dec => true,
                Mask::Letter => true,
                Mask::LetterOrDigit => true,
                Mask::LetterDigitSpace => true,
                Mask::AnyChar => true,
                Mask::Separator(_) => true,
                Mask::None => false,
            }
        }

        /// right to left editing
        #[inline]
        pub(super) fn is_rtol(&self) -> bool {
            match self {
                Mask::Digit0(d) => d.is_rtol(),
                Mask::Digit(d) => d.is_rtol(),
                Mask::Numeric(d) => d.is_rtol(),
                Mask::GroupingSep => true,
                Mask::Sign => true,
                Mask::Plus => true,
                Mask::DecimalSep => false,
                Mask::Hex0 => false,
                Mask::Hex => false,
                Mask::Oct0 => false,
                Mask::Oct => false,
                Mask::Dec0 => false,
                Mask::Dec => false,
                Mask::Letter => false,
                Mask::LetterOrDigit => false,
                Mask::LetterDigitSpace => false,
                Mask::AnyChar => false,
                Mask::Separator(_) => false,
                Mask::None => false,
            }
        }

        #[inline]
        pub(super) fn is_fraction(&self) -> bool {
            match self {
                Mask::Digit0(d) => d.is_ltor(),
                Mask::Digit(d) => d.is_ltor(),
                Mask::Numeric(d) => d.is_ltor(),
                Mask::GroupingSep => false,
                Mask::Sign => false,
                Mask::Plus => false,
                Mask::DecimalSep => false,
                Mask::Hex0 => false,
                Mask::Hex => false,
                Mask::Oct0 => false,
                Mask::Oct => false,
                Mask::Dec0 => false,
                Mask::Dec => false,
                Mask::Letter => false,
                Mask::LetterOrDigit => false,
                Mask::LetterDigitSpace => false,
                Mask::AnyChar => false,
                Mask::Separator(_) => false,
                Mask::None => false,
            }
        }

        /// which mask-types are put together into one section.
        #[inline]
        pub(super) fn section(&self) -> u8 {
            match self {
                Mask::Digit0(_) => 0,
                Mask::Digit(_) => 0,
                Mask::Numeric(_) => 0,
                Mask::GroupingSep => 0,

                Mask::Sign => 1,

                Mask::Plus => 2,

                Mask::DecimalSep => 3,

                Mask::Hex0 => 4,
                Mask::Hex => 4,

                Mask::Oct0 => 5,
                Mask::Oct => 5,

                Mask::Dec0 => 6,
                Mask::Dec => 6,

                Mask::Letter => 7,
                Mask::LetterOrDigit => 7,
                Mask::LetterDigitSpace => 7,
                Mask::AnyChar => 7,

                Mask::Separator(_) => 8,

                Mask::None => 9,
            }
        }

        /// which mask-types constitute a number
        #[inline]
        pub(super) fn number(&self) -> u8 {
            match self {
                Mask::Digit0(_) => 0,
                Mask::Digit(_) => 0,
                Mask::Numeric(_) => 0,
                Mask::GroupingSep => 0,
                Mask::Sign => 0,
                Mask::Plus => 0,
                Mask::DecimalSep => 0,

                Mask::Hex0 => 1,
                Mask::Hex => 1,
                Mask::Oct0 => 1,
                Mask::Oct => 1,
                Mask::Dec0 => 1,
                Mask::Dec => 1,
                Mask::Letter => 1,
                Mask::LetterOrDigit => 1,
                Mask::LetterDigitSpace => 1,
                Mask::AnyChar => 1,

                Mask::Separator(_) => 2,
                Mask::None => 3,
            }
        }

        // mask should overwrite instead of insert
        #[inline]
        pub(super) fn can_overwrite(&self, c: &str) -> bool {
            match self {
                Mask::Digit0(_) | Mask::Digit(_) | Mask::Numeric(_) => false,
                Mask::DecimalSep => "." == c,
                Mask::GroupingSep => false,
                Mask::Sign => "-" == c || " " == c,
                Mask::Plus => "-" == c || "+" == c || " " == c,
                Mask::Hex0 => c == "0",
                Mask::Hex => false,
                Mask::Oct0 => c == "0",
                Mask::Oct => false,
                Mask::Dec0 => c == "0",
                Mask::Dec => false,
                Mask::Letter => false,
                Mask::LetterOrDigit => false,
                Mask::LetterDigitSpace => false,
                Mask::AnyChar => false,
                Mask::Separator(sep) => sep.as_ref() == c,
                Mask::None => false,
            }
        }

        // char can be dropped
        #[inline]
        pub(super) fn can_drop(&self, c: &str) -> bool {
            match self {
                Mask::Digit0(_) => c == "0",
                Mask::Digit(_) => c == " ",
                Mask::Numeric(_) => c == " ",
                Mask::DecimalSep => false,
                Mask::Sign => false,
                Mask::Plus => false,
                Mask::GroupingSep => true,
                Mask::Hex0 => c == "0",
                Mask::Hex => c == " ",
                Mask::Oct0 => c == "0",
                Mask::Oct => c == " ",
                Mask::Dec0 => c == "0",
                Mask::Dec => c == " ",
                Mask::Letter => c == " ",
                Mask::LetterOrDigit => c == " ",
                Mask::LetterDigitSpace => c == " ",
                Mask::AnyChar => c == " ",
                Mask::Separator(_sep) => false,
                Mask::None => false,
            }
        }

        /// Get the default char for this mask.
        #[inline]
        pub(super) fn edit_value(&self) -> &str {
            match self {
                Mask::Digit0(_) => "0",
                Mask::Digit(_) => " ",
                Mask::Numeric(_) => " ",
                Mask::DecimalSep => ".",
                Mask::GroupingSep => " ", // don't show. reformat fills it in if necessary.
                Mask::Sign => " ",
                Mask::Plus => "+",
                Mask::Hex0 => "0",
                Mask::Hex => " ",
                Mask::Oct0 => "0",
                Mask::Oct => " ",
                Mask::Dec0 => "0",
                Mask::Dec => " ",
                Mask::Letter => " ",
                Mask::LetterOrDigit => " ",
                Mask::LetterDigitSpace => " ",
                Mask::AnyChar => " ",
                Mask::Separator(g) => g.as_ref(),
                Mask::None => "",
            }
        }
    }

    impl Debug for MaskToken {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "Mask #{}:{}:{}-{} {:?} | {:?}",
                self.nr_id, self.sec_id, self.sec_start, self.sec_end, self.peek_left, self.right
            )
        }
    }

    impl MaskToken {
        /// Create a string with the default edit mask.
        pub(super) fn empty_section(mask: &[MaskToken]) -> String {
            let mut buf = String::new();
            for m in mask {
                buf.push_str(&m.edit);
            }
            buf
        }
    }
}

impl HandleEvent<crossterm::event::Event, Regular, TextOutcome> for MaskedInputState {
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
                ct_event!(keycode press Backspace) => tc(self.delete_prev_char()),
                ct_event!(keycode press Delete) => tc(self.delete_next_char()),
                // ct_event!(keycode press CONTROL-Backspace) => tc(self.delete_prev_word()),
                // ct_event!(keycode press CONTROL-Delete) => tc(self.delete_next_word()),
                ct_event!(key press CONTROL-'d') => tc(self.clear()),

                ct_event!(key release _)
                | ct_event!(key release SHIFT-_)
                | ct_event!(key release CONTROL_ALT-_)
                | ct_event!(keycode release Backspace)
                | ct_event!(keycode release Delete)
                // | ct_event!(keycode release CONTROL-Backspace)
                // | ct_event!(keycode release CONTROL-Delete)
                | ct_event!(key release CONTROL-'d') => TextOutcome::Unchanged,

                _ => TextOutcome::Continue,
            }
        } else {
            TextOutcome::Continue
        };
        // remap to TextChanged
        if r == TextOutcome::Changed {
            r = TextOutcome::TextChanged;
        }

        if r == TextOutcome::Continue {
            r = self.handle(event, ReadOnly);
        }
        r
    }
}

impl HandleEvent<crossterm::event::Event, ReadOnly, TextOutcome> for MaskedInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: ReadOnly) -> TextOutcome {
        let mut r = if self.is_focused() {
            match event {
                ct_event!(keycode press Left) => self.move_left(false).into(),
                ct_event!(keycode press Right) => self.move_right(false).into(),
                // ct_event!(keycode press CONTROL-Left) => self.move_to_prev_word(false).into(),
                // ct_event!(keycode press CONTROL-Right) => self.move_to_next_word(false).into(),
                ct_event!(keycode press Home) => self.move_to_line_start(false).into(),
                ct_event!(keycode press End) => self.move_to_line_end(false).into(),
                ct_event!(keycode press SHIFT-Left) => self.move_left(true).into(),
                ct_event!(keycode press SHIFT-Right) => self.move_right(true).into(),
                // ct_event!(keycode press CONTROL_SHIFT-Left) => self.move_to_prev_word(true).into(),
                // ct_event!(keycode press CONTROL_SHIFT-Right) => self.move_to_next_word(true).into(),
                ct_event!(keycode press SHIFT-Home) => self.move_to_line_start(true).into(),
                ct_event!(keycode press SHIFT-End) => self.move_to_line_end(true).into(),
                ct_event!(key press CONTROL-'a') => self.select_all().into(),

                ct_event!(keycode release Left)
                | ct_event!(keycode release Right)
                // | ct_event!(keycode release CONTROL-Left)
                // | ct_event!(keycode release CONTROL-Right)
                | ct_event!(keycode release Home)
                | ct_event!(keycode release End)
                | ct_event!(keycode release SHIFT-Left)
                | ct_event!(keycode release SHIFT-Right)
                // | ct_event!(keycode release CONTROL_SHIFT-Left)
                // | ct_event!(keycode release CONTROL_SHIFT-Right)
                | ct_event!(keycode release SHIFT-Home)
                | ct_event!(keycode release SHIFT-End)
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

impl HandleEvent<crossterm::event::Event, MouseOnly, TextOutcome> for MaskedInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> TextOutcome {
        match event {
            ct_event!(mouse any for m) if self.mouse.drag(self.inner, m) => {
                let c = (m.column as i16) - (self.inner.x as i16);
                self.set_screen_cursor(c, true).into()
            }
            ct_event!(mouse down Left for column,row) => {
                if self.gained_focus() {
                    // don't react to the first click that's for
                    // focus. this one shouldn't demolish the selection.
                    TextOutcome::Unchanged
                } else if self.inner.contains((*column, *row).into()) {
                    let c = (column - self.inner.x) as i16;
                    self.set_screen_cursor(c, false).into()
                } else {
                    TextOutcome::Continue
                }
            }
            _ => TextOutcome::Continue,
        }
    }
}

/// Handle all events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events(
    state: &mut MaskedInputState,
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
    state: &mut TextInputState,
    focus: bool,
    event: &crossterm::event::Event,
) -> TextOutcome {
    state.focus.set(focus);
    state.handle(event, ReadOnly)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(
    state: &mut MaskedInputState,
    event: &crossterm::event::Event,
) -> TextOutcome {
    state.handle(event, MouseOnly)
}

mod masked {
    use crate::clipboard::{Clipboard, LocalClipboard};
    use crate::core::{TextCore, TextString};
    use crate::grapheme::GlyphIter;
    use crate::masked_input::mask::{EditDirection, Mask, MaskToken};
    use crate::undo_buffer::{UndoBuffer, UndoEntry, UndoVec};
    use crate::{upos_type, Cursor, Glyph, Grapheme, TextError, TextPosition, TextRange};
    use format_num_pattern::core::{clean_num, map_num};
    use format_num_pattern::{CurrencySym, NumberFormat, NumberSymbols};
    use std::borrow::Cow;
    use std::fmt;
    use std::iter::once;
    use std::ops::Range;
    use unicode_segmentation::UnicodeSegmentation;

    /// Text editing core.
    #[derive(Debug, Clone)]
    pub struct MaskedCore {
        // text
        masked: TextCore<TextString>,
        // number symbols
        sym: Option<NumberSymbols>,
        // parsed mask
        mask: Vec<MaskToken>,
    }

    impl Default for MaskedCore {
        fn default() -> Self {
            let mut value = TextCore::new(
                Some(Box::new(UndoVec::new(99))),
                Some(Box::new(LocalClipboard::new())),
            );
            value.set_glyph_line_break(false);

            Self {
                masked: value,
                sym: None,
                mask: Default::default(),
            }
        }
    }

    impl MaskedCore {
        pub fn new() -> Self {
            Self::default()
        }

        /// Set the decimal separator and other symbols.
        /// Only used for rendering and to map user input.
        /// The value itself uses "."
        pub fn set_num_symbols(&mut self, sym: NumberSymbols) {
            self.sym = Some(sym);
        }

        fn dec_sep(&self) -> char {
            if let Some(sym) = &self.sym {
                sym.decimal_sep
            } else {
                '.'
            }
        }

        fn grp_sep(&self) -> char {
            if let Some(sym) = &self.sym {
                if let Some(grp) = sym.decimal_grp {
                    grp
                } else {
                    // fallback for empty grp-char.
                    // it would be really ugly, if we couldn't keep
                    //   mask-idx == grapheme-idx
                    ' '
                }
            } else {
                ','
            }
        }

        fn neg_sym(&self) -> char {
            if let Some(sym) = &self.sym {
                sym.negative_sym
            } else {
                '-'
            }
        }

        fn pos_sym(&self) -> char {
            if let Some(sym) = &self.sym {
                sym.positive_sym
            } else {
                ' '
            }
        }

        /// Changes the mask.
        /// Resets the value to a default.
        pub fn set_mask<S: AsRef<str>>(&mut self, s: S) -> Result<(), fmt::Error> {
            self.mask = Self::parse_mask(s.as_ref())?;
            self.clear();
            Ok(())
        }

        #[allow(clippy::needless_range_loop)]
        fn parse_mask(mask_str: &str) -> Result<Vec<MaskToken>, fmt::Error> {
            let mut out = Vec::<MaskToken>::new();

            let mut start_id = 0;
            let mut id = 0;
            let mut start_nr = 0;
            let mut nr_id = 0;
            let mut last_mask = Mask::None;
            let mut dec_dir = EditDirection::Rtol;
            let mut esc = false;
            let mut idx = 0;
            for m in mask_str.graphemes(true).chain(once("")) {
                let mask = if esc {
                    esc = false;
                    Mask::Separator(Box::from(m))
                } else {
                    match m {
                        "0" => Mask::Digit0(dec_dir),
                        "9" => Mask::Digit(dec_dir),
                        "#" => Mask::Numeric(dec_dir),
                        "." => Mask::DecimalSep,
                        "," => Mask::GroupingSep,
                        "-" => Mask::Sign,
                        "+" => Mask::Plus,
                        "h" => Mask::Hex,
                        "H" => Mask::Hex0,
                        "o" => Mask::Oct,
                        "O" => Mask::Oct0,
                        "d" => Mask::Dec,
                        "D" => Mask::Dec0,
                        "l" => Mask::Letter,
                        "a" => Mask::LetterOrDigit,
                        "c" => Mask::LetterDigitSpace,
                        "_" => Mask::AnyChar,
                        "" => Mask::None,
                        " " | ";" | ":" | "/" => Mask::Separator(Box::from(m)),
                        "\\" => {
                            esc = true;
                            continue;
                        }
                        s if s.is_ascii() => return Err(fmt::Error),
                        s => Mask::Separator(Box::from(s)),
                    }
                };

                match mask {
                    Mask::Digit0(_)
                    | Mask::Digit(_)
                    | Mask::Numeric(_)
                    | Mask::GroupingSep
                    | Mask::Sign
                    | Mask::Plus => {
                        // no change
                    }
                    Mask::DecimalSep => {
                        dec_dir = EditDirection::Ltor;
                    }
                    Mask::Hex0
                    | Mask::Hex
                    | Mask::Oct0
                    | Mask::Oct
                    | Mask::Dec0
                    | Mask::Dec
                    | Mask::Letter
                    | Mask::LetterOrDigit
                    | Mask::LetterDigitSpace
                    | Mask::AnyChar
                    | Mask::Separator(_) => {
                        // reset to default number input direction
                        dec_dir = EditDirection::Rtol
                    }
                    Mask::None => {
                        // no change, doesn't matter
                    }
                }

                if matches!(mask, Mask::Separator(_)) || mask.number() != last_mask.number() {
                    for j in start_nr..idx {
                        out[j].nr_id = nr_id;
                        out[j].nr_start = start_nr as upos_type;
                        out[j].nr_end = idx as upos_type;
                    }
                    nr_id += 1;
                    start_nr = idx;
                }
                if matches!(mask, Mask::Separator(_)) || mask.section() != last_mask.section() {
                    for j in start_id..idx {
                        out[j].sec_id = id;
                        out[j].sec_start = start_id as upos_type;
                        out[j].sec_end = idx as upos_type;
                    }
                    id += 1;
                    start_id = idx;
                }

                let tok = MaskToken {
                    nr_id: 0,
                    nr_start: 0,
                    nr_end: 0,
                    sec_id: 0,
                    sec_start: 0,
                    sec_end: 0,
                    peek_left: last_mask,
                    right: mask.clone(),
                    edit: mask.edit_value().into(),
                };
                out.push(tok);

                idx += 1;
                last_mask = mask;
            }
            for j in start_nr..out.len() {
                out[j].nr_id = nr_id;
                out[j].nr_start = start_nr as upos_type;
                out[j].nr_end = mask_str.graphemes(true).count() as upos_type;
            }
            for j in start_id..out.len() {
                out[j].sec_id = id;
                out[j].sec_start = start_id as upos_type;
                out[j].sec_end = mask_str.graphemes(true).count() as upos_type;
            }

            Ok(out)
        }

        /// Return the mask.
        pub fn mask(&self) -> String {
            use std::fmt::Write;

            let mut buf = String::new();
            for t in self.mask.iter() {
                _ = write!(buf, "{}", t.right);
            }
            buf
        }
    }

    impl MaskedCore {
        /// Clipboard
        pub fn set_clipboard(&mut self, clip: Option<Box<dyn Clipboard + 'static>>) {
            self.masked.set_clipboard(clip);
        }

        /// Clipboard
        pub fn clipboard(&self) -> Option<&dyn Clipboard> {
            self.masked.clipboard()
        }
    }

    impl MaskedCore {
        /// Undo
        #[inline]
        pub fn set_undo_buffer(&mut self, undo: Option<Box<dyn UndoBuffer>>) {
            self.masked.set_undo_buffer(undo);
        }

        /// Set undo count
        #[inline]
        pub fn set_undo_count(&mut self, n: u32) {
            self.masked.set_undo_count(n);
        }

        /// Begin a sequence of changes that should be undone in one go.
        #[inline]
        pub fn begin_undo_seq(&mut self) {
            self.masked.begin_undo_seq();
        }

        /// End a sequence of changes that should be undone in one go.
        #[inline]
        pub fn end_undo_seq(&mut self) {
            self.masked.end_undo_seq();
        }

        /// Undo
        #[inline]
        pub fn undo_buffer(&self) -> Option<&dyn UndoBuffer> {
            self.masked.undo_buffer()
        }

        /// Undo
        #[inline]
        pub fn undo_buffer_mut(&mut self) -> Option<&mut dyn UndoBuffer> {
            self.masked.undo_buffer_mut()
        }

        /// Undo last.
        pub fn undo(&mut self) -> bool {
            self.masked.undo()
        }

        /// Redo last.
        pub fn redo(&mut self) -> bool {
            self.masked.redo()
        }

        /// Get last replay recording.
        pub fn recent_replay_log(&mut self) -> Vec<UndoEntry> {
            self.masked.recent_replay_log()
        }

        /// Replay a recording of changes.
        pub fn replay_log(&mut self, replay: &[UndoEntry]) {
            self.masked.replay_log(replay)
        }
    }

    impl MaskedCore {
        /// Set all styles.
        ///
        /// The ranges are byte-ranges. The usize value is the index of the
        /// actual style. Those are set with the widget.
        #[inline]
        pub fn set_styles(&mut self, new_styles: Vec<(Range<usize>, usize)>) {
            self.masked.set_styles(new_styles);
        }

        /// Add a style for the given byte-range.
        ///
        /// The usize value is the index of the actual style.
        /// Those are set at the widget.
        #[inline]
        pub fn add_style(&mut self, range: Range<usize>, style: usize) {
            self.masked.add_style(range, style);
        }

        /// Remove a style for the given byte-range.
        ///
        /// Range and style must match to be removed.
        #[inline]
        pub fn remove_style(&mut self, range: Range<usize>, style: usize) {
            self.masked.remove_style(range, style);
        }

        /// Find all values for the given position.
        ///
        /// Creates a cache for the styles in range.
        #[inline]
        pub(crate) fn styles_at_page(&self, range: Range<usize>, pos: usize, buf: &mut Vec<usize>) {
            self.masked.styles_at_page(range, pos, buf);
        }

        /// Finds all styles for the given position.
        #[inline]
        pub fn styles_at(&self, byte_pos: usize, buf: &mut Vec<usize>) {
            self.masked.styles_at(byte_pos, buf);
        }

        /// Check if the given style applies at the position and
        /// return the complete range for the style.
        #[inline]
        pub fn style_match(&self, byte_pos: usize, style: usize) -> Option<Range<usize>> {
            self.masked.style_match(byte_pos, style)
        }

        /// List of all styles.
        #[inline]
        pub fn styles(&self) -> Option<impl Iterator<Item = (Range<usize>, usize)> + '_> {
            self.masked.styles()
        }
    }

    impl MaskedCore {
        /// Set the cursor position.
        /// The value is capped to the number of text lines and
        /// the line-width for the given line.
        ///
        /// Returns true, if the cursor actually changed.
        pub fn set_cursor(&mut self, cursor: upos_type, extend_selection: bool) -> bool {
            self.masked
                .set_cursor(TextPosition::new(cursor, 0), extend_selection)
        }

        /// Cursor position as grapheme-idx.
        #[inline]
        pub fn cursor(&self) -> upos_type {
            self.masked.cursor().x
        }

        /// Selection anchor
        #[inline]
        pub fn anchor(&self) -> upos_type {
            self.masked.anchor().x
        }

        /// Any text selection.
        #[inline]
        pub fn has_selection(&self) -> bool {
            self.masked.has_selection()
        }

        /// Select text.
        #[inline]
        pub fn set_selection(&mut self, anchor: upos_type, cursor: upos_type) -> bool {
            self.masked
                .set_selection(TextPosition::new(anchor, 0), TextPosition::new(cursor, 0))
        }

        /// Select all text.
        #[inline]
        pub fn select_all(&mut self) -> bool {
            self.masked.select_all()
        }

        /// Returns the selection as TextRange.
        #[inline]
        pub fn selection(&self) -> Range<upos_type> {
            let selection = self.masked.selection();
            selection.start.x..selection.end.x
        }

        /// Selection.
        #[inline]
        pub fn selected_text(&self) -> &str {
            match self
                .masked
                .str_slice(self.masked.selection())
                .expect("valid_range")
            {
                Cow::Borrowed(v) => v,
                Cow::Owned(_) => {
                    unreachable!()
                }
            }
        }
    }

    impl MaskedCore {
        /// Empty.
        #[inline]
        pub fn is_empty(&self) -> bool {
            self.masked.text().as_str() == self.default_value()
        }

        /// Grapheme position to byte position.
        /// This is the (start,end) position of the single grapheme after pos.
        #[inline]
        pub fn byte_at(&self, pos: upos_type) -> Result<Range<usize>, TextError> {
            self.masked.byte_at(TextPosition::new(pos, 0))
        }

        /// Grapheme range to byte range.
        #[inline]
        pub fn bytes_at_range(&self, range: Range<upos_type>) -> Result<Range<usize>, TextError> {
            self.masked
                .bytes_at_range(TextRange::new((range.start, 0), (range.end, 0)))
        }

        /// Byte position to grapheme position.
        /// Returns the position that contains the given byte index.
        #[inline]
        pub fn byte_pos(&self, byte: usize) -> Result<upos_type, TextError> {
            Ok(self.masked.byte_pos(byte)?.x)
        }

        /// Byte range to grapheme range.
        #[inline]
        pub fn byte_range(&self, bytes: Range<usize>) -> Result<Range<upos_type>, TextError> {
            let r = self.masked.byte_range(bytes)?;
            Ok(r.start.x..r.end.x)
        }

        /// A range of the text as Cow<str>
        #[inline]
        pub fn str_slice(&self, range: Range<upos_type>) -> Result<Cow<'_, str>, TextError> {
            self.masked
                .str_slice(TextRange::new((range.start, 0), (range.end, 0)))
        }

        /// Iterator for the glyphs of the lines in range.
        /// Glyphs here a grapheme + display length.
        #[inline]
        pub fn glyphs(
            &self,
            rows: Range<upos_type>,
            screen_offset: u16,
            screen_width: u16,
        ) -> Result<impl Iterator<Item = Glyph<'_>>, TextError> {
            let grapheme_iter = self.masked.graphemes(
                TextRange::new((0, rows.start), (0, rows.end)),
                TextPosition::new(0, rows.start),
            )?;

            let mask_iter = self.mask.iter();

            let sym_neg = || self.neg_sym().to_string();
            let sym_dec = || self.dec_sep().to_string();
            let sym_grp = || self.grp_sep().to_string();
            let sym_pos = || self.pos_sym().to_string();

            let iter = grapheme_iter.zip(mask_iter).filter_map(move |(g, t)| {
                match (&t.right, g.grapheme()) {
                    (Mask::Numeric(_), "-") => {
                        Some(Grapheme::new(Cow::Owned(sym_neg()), g.text_bytes()))
                    }
                    (Mask::DecimalSep, ".") => {
                        Some(Grapheme::new(Cow::Owned(sym_dec()), g.text_bytes()))
                    }
                    (Mask::GroupingSep, ",") => {
                        Some(Grapheme::new(Cow::Owned(sym_grp()), g.text_bytes()))
                    }
                    (Mask::GroupingSep, "-") => {
                        Some(Grapheme::new(Cow::Owned(sym_neg()), g.text_bytes()))
                    }
                    (Mask::Sign, "-") => Some(Grapheme::new(Cow::Owned(sym_neg()), g.text_bytes())),
                    (Mask::Sign, _) => Some(Grapheme::new(Cow::Owned(sym_pos()), g.text_bytes())),
                    (_, _) => Some(g),
                }
            });

            let mut it = GlyphIter::new(TextPosition::new(0, rows.start), iter);
            it.set_screen_offset(screen_offset);
            it.set_screen_width(screen_width);
            it.set_tabs(self.masked.tab_width());
            it.set_show_ctrl(self.masked.glyph_ctrl());
            it.set_line_break(self.masked.glyph_line_break());
            Ok(it)
        }

        /// Iterator for the glyphs of the lines in range.
        /// Glyphs here a grapheme + display length.
        ///
        /// This omits unnecessary white-space.
        #[inline]
        pub fn condensed_glyphs(
            &self,
            rows: Range<upos_type>,
            screen_offset: u16,
            screen_width: u16,
        ) -> Result<impl Iterator<Item = Glyph<'_>>, TextError> {
            let grapheme_iter = self.masked.graphemes(
                TextRange::new((0, rows.start), (0, rows.end)),
                TextPosition::new(0, rows.start),
            )?;

            let mask_iter = self.mask.iter();

            let sym_neg = || self.neg_sym().to_string();
            let sym_dec = || self.dec_sep().to_string();
            let sym_grp = || self.grp_sep().to_string();
            let sym_pos = || self.pos_sym().to_string();

            let iter = grapheme_iter.zip(mask_iter).filter_map(move |(g, t)| {
                match (&t.right, g.grapheme()) {
                    (Mask::Numeric(_), "-") => {
                        Some(Grapheme::new(Cow::Owned(sym_neg()), g.text_bytes()))
                    }
                    (Mask::DecimalSep, ".") => {
                        Some(Grapheme::new(Cow::Owned(sym_dec()), g.text_bytes()))
                    }
                    (Mask::GroupingSep, ",") => {
                        Some(Grapheme::new(Cow::Owned(sym_grp()), g.text_bytes()))
                    }
                    (Mask::GroupingSep, "-") => {
                        Some(Grapheme::new(Cow::Owned(sym_neg()), g.text_bytes()))
                    }
                    (Mask::Sign, "-") => Some(Grapheme::new(Cow::Owned(sym_neg()), g.text_bytes())),

                    (Mask::Numeric(_), " ") => None,
                    (Mask::Digit(_), " ") => None,
                    (Mask::DecimalSep, " ") => None,
                    (Mask::GroupingSep, " ") => None,
                    (Mask::Sign, _) => {
                        if self.pos_sym() != ' ' {
                            Some(Grapheme::new(Cow::Owned(sym_pos()), g.text_bytes()))
                        } else {
                            None
                        }
                    }
                    (Mask::Hex, " ") => None,
                    (Mask::Oct, " ") => None,
                    (Mask::Dec, " ") => None,

                    (_, _) => Some(g),
                }
            });

            let mut it = GlyphIter::new(TextPosition::new(0, rows.start), iter);
            it.set_screen_offset(screen_offset);
            it.set_screen_width(screen_width);
            it.set_tabs(self.masked.tab_width());
            it.set_show_ctrl(self.masked.glyph_ctrl());
            it.set_line_break(self.masked.glyph_line_break());
            Ok(it)
        }

        /// Get the grapheme at the given position.
        #[inline]
        pub fn grapheme_at(&self, pos: upos_type) -> Result<Option<Grapheme<'_>>, TextError> {
            self.masked.grapheme_at(TextPosition::new(pos, 0))
        }

        /// Get a cursor over all the text with the current position set at pos.
        #[inline]
        pub fn text_graphemes(
            &self,
            pos: upos_type,
        ) -> Result<impl Iterator<Item = Grapheme<'_>> + Cursor, TextError> {
            self.masked.text_graphemes(TextPosition::new(pos, 0))
        }

        /// Get a cursor over the text-range the current position set at pos.
        #[inline]
        pub fn graphemes(
            &self,
            range: Range<upos_type>,
            pos: upos_type,
        ) -> Result<impl Iterator<Item = Grapheme<'_>> + Cursor, TextError> {
            self.masked.graphemes(
                TextRange::new((range.start, 0), (range.end, 0)),
                TextPosition::new(pos, 0),
            )
        }

        #[inline]
        pub fn line_width(&self) -> upos_type {
            self.masked.line_width(0).expect("valid_row")
        }
    }

    impl MaskedCore {
        /// Create a default value according to the mask.
        #[inline]
        fn default_value(&self) -> String {
            MaskToken::empty_section(&self.mask)
        }
    }

    impl MaskedCore {
        /// Reset value but not the mask and width.
        /// Resets offset and cursor position too.
        #[inline]
        pub fn clear(&mut self) {
            self.masked
                .set_text(TextString::new_string(self.default_value()));
            self.set_default_cursor();
        }

        /// Copy of the text-value.
        pub fn text(&self) -> &str {
            self.masked.text().as_str()
        }

        /// Sets the value.
        /// No checks if the value conforms to the mask.
        /// If the value is too short it will be filled with space.
        /// if the value is too long it will be truncated.
        #[allow(clippy::comparison_chain)]
        pub fn set_text<S: Into<String>>(&mut self, s: S) {
            let mut value = s.into();

            let len = value.graphemes(true).count();
            if len > self.mask.len() - 1 {
                for _ in len..self.mask.len() - 1 {
                    value.pop();
                }
            } else if len < self.mask.len() - 1 {
                for _ in len..self.mask.len() - 1 {
                    value.push(' ');
                }
            }
            let len = value.graphemes(true).count();

            assert_eq!(len, self.mask.len() - 1);

            self.masked.set_text(TextString::new_string(value));
        }

        /// Start at the cursor position and find a valid insert position for the input c.
        /// Put the cursor at that position.
        #[allow(clippy::if_same_then_else)]
        pub fn advance_cursor(&mut self, c: char) {
            let mut new_cursor = self.masked.cursor().x;

            // debug!("// ADVANCE CURSOR {:?}  ", c);
            // debug!("#[rustfmt::skip]");
            // debug!("let mut b = {};", test_state(self));
            // debug!("b.advance_cursor({:?});", c);

            loop {
                let mask = &self.mask[new_cursor as usize];

                if self.can_insert_integer_left(mask, new_cursor, c) {
                    // At the gap between an integer field and something else.
                    // Integer fields are served first.
                    break;
                } else if self.can_insert_integer(mask, new_cursor, c) {
                    // Insert position inside an integer field. After any spaces
                    // and the sign.
                    break;
                } else if self.can_insert_sign(mask, c) {
                    // Can insert a sign here.
                    break;
                } else if self.can_insert_decimal_sep(mask, c) {
                    // Decimal separator matches.
                    break;
                } else if mask.right == Mask::GroupingSep {
                    // Never stop here.
                    new_cursor += 1;
                } else if self.can_insert_separator(mask, c) {
                    // todo: jump to default pos of next field
                    break;
                } else if self.can_move_left_in_fraction(mask, new_cursor, c) {
                    // skip left
                    new_cursor -= 1;
                } else if self.can_insert_fraction(mask, c) {
                    break;
                } else if self.can_insert_other(mask, c) {
                    break;
                } else if mask.right == Mask::None {
                    // No better position found. Reset and break;
                    new_cursor = self.masked.cursor().x;
                    break;
                } else {
                    new_cursor += 1;
                }
            }

            // debug!("CURSOR {} => {}", self.cursor, new_cursor);
            self.masked
                .set_cursor(TextPosition::new(new_cursor, 0), false);

            // debug!("#[runtime::skip]");
            // debug!("let a = {};", test_state(self));
            // debug!("assert_eq_core(&b,&a);");
        }

        /// Use mapped-char instead of input.
        fn map_input_c(&self, mask: &Mask, c: char) -> char {
            match mask {
                Mask::Numeric(_) => {
                    if c == self.neg_sym() {
                        return '-';
                    } else if c == self.pos_sym() {
                        return ' ';
                    }
                }
                Mask::DecimalSep => {
                    if c == self.dec_sep() {
                        return '.';
                    }
                }
                Mask::Sign => {
                    if c == self.neg_sym() {
                        return '-';
                    } else if c == self.pos_sym() || c == '+' {
                        return ' ';
                    }
                }
                Mask::Plus => {
                    if c == self.neg_sym() {
                        return '-';
                    } else if c == self.pos_sym() {
                        return '+';
                    }
                }
                _ => {}
            }
            c
        }

        /// Valid input for this mask.
        fn is_valid_char(&self, mask: &Mask, c: char) -> bool {
            match mask {
                Mask::Digit0(_) => c.is_ascii_digit(),
                Mask::Digit(_) => c.is_ascii_digit() || c == ' ',
                Mask::Numeric(_) => {
                    c.is_ascii_digit()
                        || c == ' '
                        || c == self.neg_sym()
                        || c == self.pos_sym()
                        || c == '-'
                        || c == '+'
                }
                Mask::DecimalSep => c == self.dec_sep(),
                Mask::GroupingSep => false,
                Mask::Sign => c == self.neg_sym() || c == self.pos_sym() || c == '-' || c == '+',
                Mask::Plus => c == self.neg_sym() || c == self.pos_sym() || c == '-' || c == '+',
                Mask::Hex0 => c.is_ascii_hexdigit(),
                Mask::Hex => c.is_ascii_hexdigit() || c == ' ',
                Mask::Oct0 => c.is_digit(8),
                Mask::Oct => c.is_digit(8) || c == ' ',
                Mask::Dec0 => c.is_ascii_digit(),
                Mask::Dec => c.is_ascii_digit() || c == ' ',
                Mask::Letter => c.is_alphabetic(),
                Mask::LetterOrDigit => c.is_alphanumeric(),
                Mask::LetterDigitSpace => c.is_alphanumeric() || c == ' ',
                Mask::AnyChar => true,
                Mask::Separator(sep) => {
                    // todo: don't know better
                    if let Some(sepc) = sep.chars().next() {
                        sepc == c
                    } else {
                        false
                    }
                }
                Mask::None => false,
            }
        }

        // Can insert other field types
        #[inline]
        fn can_insert_other(&self, mask: &MaskToken, c: char) -> bool {
            match mask.right {
                Mask::Hex0
                | Mask::Hex
                | Mask::Oct0
                | Mask::Oct
                | Mask::Dec0
                | Mask::Dec
                | Mask::Letter
                | Mask::LetterOrDigit
                | Mask::LetterDigitSpace
                | Mask::AnyChar => self.is_valid_char(&mask.right, c),
                _ => false,
            }
        }

        // Can insert fraction.
        #[inline]
        fn can_insert_fraction(&self, mask: &MaskToken, c: char) -> bool {
            if !mask.right.is_fraction() {
                return false;
            }
            if !self.is_valid_char(&mask.right, c) {
                return false;
            }
            true
        }

        // When inserting to the fraction we want to left-align
        // the digits. This checks if a digit could possibly be
        // inserted to the left of the current position.
        #[inline]
        fn can_move_left_in_fraction(
            &self,
            mask: &MaskToken,
            new_cursor: upos_type,
            c: char,
        ) -> bool {
            if !mask.peek_left.is_fraction() {
                return false;
            }

            if !self.is_valid_char(&mask.peek_left, c) {
                return false;
            }

            let gl = self
                .masked
                .grapheme_at(TextPosition::new(new_cursor - 1, 0))
                .expect("valid_position")
                .expect("grapheme");

            // is there space to the left?
            if gl != " " {
                return false;
            }

            true
        }

        // Can input a sign here?
        #[inline]
        fn can_insert_sign(&self, mask: &MaskToken, c: char) -> bool {
            if !self.is_valid_char(&Mask::Sign, c) {
                return false;
            }
            if !mask.right.is_number() {
                return false;
            }

            // check possible positions for the sign.
            for i in mask.nr_start..mask.nr_end {
                let t = &self.mask[i as usize];
                match t.right {
                    Mask::Plus => return true,
                    Mask::Sign => return true,
                    Mask::Numeric(EditDirection::Rtol) => {
                        // Numeric fields can hold a sign.
                        // If they are not otherwise occupied.
                        let gi = self
                            .masked
                            .grapheme_at(TextPosition::new(i, 0))
                            .expect("valid_position")
                            .expect("grapheme");

                        return t.right.can_drop(gi.grapheme()) || gi == "-";
                    }
                    _ => {}
                }
            }

            false
        }

        // Is this the correct input position for a rtol field
        #[inline]
        fn can_insert_integer(&self, mask: &MaskToken, new_cursor: upos_type, c: char) -> bool {
            if !mask.right.is_rtol() {
                return false;
            }

            if !self.is_valid_char(&mask.right, c) {
                return false;
            }

            let g = self
                .masked
                .grapheme_at(TextPosition::new(new_cursor, 0))
                .expect("valid_position")
                .expect("grapheme");
            if mask.right.can_drop(g.grapheme()) {
                return false;
            }
            if g == "-" {
                return false;
            }

            true
        }

        // Separator char matches
        #[inline]
        fn can_insert_separator(&self, mask: &MaskToken, c: char) -> bool {
            if !matches!(mask.right, Mask::Separator(_)) {
                return false;
            }
            if !self.is_valid_char(&mask.right, c) {
                return false;
            }
            true
        }

        // Can insert a decimal separator.
        #[inline]
        fn can_insert_decimal_sep(&self, mask: &MaskToken, c: char) -> bool {
            if mask.right != Mask::DecimalSep {
                return false;
            }
            if !self.is_valid_char(&mask.right, c) {
                return false;
            }
            true
        }

        // Can edit the field left of the cursor.
        #[inline]
        fn can_insert_integer_left(
            &self,
            mask: &MaskToken,
            new_cursor: upos_type,
            c: char,
        ) -> bool {
            if !mask.peek_left.is_rtol() {
                return false;
            }
            if !mask.right.is_ltor() && !mask.right.is_none() {
                return false;
            }

            let left = &self.mask[new_cursor as usize - 1];
            if !self.is_valid_char(&left.right, c) {
                return false;
            }

            let mask0 = &self.mask[left.sec_start as usize];
            let g0 = self
                .masked
                .grapheme_at(TextPosition::new(left.sec_start, 0))
                .expect("valid_position")
                .expect("grapheme");
            if !mask0.right.can_drop(g0.grapheme()) {
                return false;
            }

            true
        }

        /// Insert the char if it matches the cursor mask and the current section is not full.
        ///
        /// `advance_cursor()` must be called before for correct functionality.
        ///
        /// Otherwise: your mileage might vary.
        pub fn insert_char(&mut self, c: char) {
            // let mask = &self.mask[self.cursor];
            // debug!("// INSERT CHAR {:?} {:?}", mask, c);
            // debug!("#[rustfmt::skip]");
            // debug!("let mut b = {};", test_state(self));
            // debug!("b.insert_char({:?});", c);

            let cursor = self.masked.cursor();

            // note: because of borrow checker. calls &mut methods.
            {
                let mask = &self.mask[cursor.x as usize];
                if mask.right.is_number() && self.can_insert_sign(mask, c) {
                    if self.insert_sign(c) {
                        return;
                    }
                }
            }
            {
                let mask = &self.mask[cursor.x as usize];
                if mask.peek_left.is_number() && (mask.right.is_ltor() || mask.right.is_none()) {
                    let left = &self.mask[cursor.x as usize - 1];
                    if self.can_insert_sign(left, c) {
                        if self.insert_sign(c) {
                            return;
                        }
                    }
                }
            }
            {
                let mask = &self.mask[cursor.x as usize];
                if mask.right.is_rtol() {
                    if self.insert_rtol(c) {
                        return;
                    }
                }
            }
            {
                let mask = &self.mask[cursor.x as usize];
                if mask.peek_left.is_rtol() && (mask.right.is_ltor() || mask.right.is_none()) {
                    if self.insert_rtol(c) {
                        return;
                    }
                }
            }
            {
                let mask = &self.mask[cursor.x as usize];
                if mask.right.is_ltor() {
                    if self.insert_ltor(c) {
                        #[allow(clippy::needless_return)]
                        return;
                    }
                }
            }

            // debug!("#[rustfmt::skip]");
            // debug!("let a = {};", test_state(self));
            // debug!("assert_eq_core(&b,&a);");
        }

        /// Insert c into a ltor section.
        fn insert_ltor(&mut self, c: char) -> bool {
            let cursor = self.masked.cursor();

            let mask = &self.mask[cursor.x as usize];
            let mask9 = &self.mask[mask.sec_end as usize - 1];

            let g = self
                .masked
                .grapheme_at(cursor)
                .expect("valid_cursor")
                .expect("mask");
            if mask.right.can_overwrite(g.grapheme()) && self.is_valid_char(&mask.right, c) {
                self.masked.begin_undo_seq();
                self.masked
                    .remove_char_range(TextRange::new(cursor, (cursor.x + 1, 0)))
                    .expect("valid_cursor");
                self.masked
                    .insert_char(cursor, self.map_input_c(&mask.right, c))
                    .expect("valid_cursor");
                self.masked.end_undo_seq();
                return true;
            }

            let g9 = self
                .masked
                .grapheme_at(TextPosition::new(mask.sec_end - 1, 0))
                .expect("valid_pos")
                .expect("mask");
            if mask9.right.can_drop(g9.grapheme()) && self.is_valid_char(&mask.right, c) {
                self.masked.begin_undo_seq();
                self.masked
                    .remove_char_range(TextRange::new((mask.sec_end - 1, 0), (mask.sec_end, 0)))
                    .expect("valid_range");
                self.masked
                    .insert_char(cursor, self.map_input_c(&mask.right, c))
                    .expect("valid_cursor");
                self.masked.end_undo_seq();
                return true;
            }
            false
        }

        /// Insert c into a rtol section
        fn insert_rtol(&mut self, c: char) -> bool {
            let cursor = self.masked.cursor();

            let mut mask = &self.mask[cursor.x as usize];

            // boundary right/left. prefer right, change mask.
            if mask.peek_left.is_rtol() && (mask.right.is_ltor() || mask.right.is_none()) {
                mask = &self.mask[cursor.x as usize - 1];
            }

            let mask0 = &self.mask[mask.sec_start as usize];

            let g0 = self
                .masked
                .grapheme_at(TextPosition::new(mask.sec_start, 0))
                .expect("valid_pos")
                .expect("grapheme");
            if mask0.right.can_drop(g0.grapheme()) && self.is_valid_char(&mask.right, c) {
                self.masked.begin_undo_seq();
                self.masked
                    .remove_char_range(TextRange::new((mask.sec_start, 0), (mask.sec_start + 1, 0)))
                    .expect("valid_position");
                self.masked
                    .insert_char(TextPosition::new(cursor.x - 2, 0), c)
                    .expect("valid_position");
                Self::reformat(&mut self.masked, &self.mask, mask.sec_start..mask.sec_end);
                self.masked.end_undo_seq();
                return true;
            }

            false
        }

        /// Insert a sign c into the current number section
        #[allow(clippy::single_match)]
        fn insert_sign(&mut self, c: char) -> bool {
            let cursor = self.masked.cursor();

            let mut mask = &self.mask[cursor.x as usize];
            // boundary right/left. prefer right, change mask.
            if mask.peek_left.is_number() && (mask.right.is_ltor() || mask.right.is_none()) {
                mask = &self.mask[cursor.x as usize - 1];
            }

            // existing sign somewhere?
            let idx = self
                .masked
                .graphemes(
                    TextRange::new((mask.nr_start, 0), (mask.nr_end, 0)),
                    TextPosition::new(mask.nr_start, 0),
                )
                .expect("valid_range")
                .enumerate()
                .find(|(_, g)| *g == "-" || *g == "+")
                .map(|(i, _)| i);
            let idx = if idx.is_none() {
                // explicit sign?
                self.masked
                    .graphemes(
                        TextRange::new((mask.nr_start, 0), (mask.nr_end, 0)),
                        TextPosition::new(mask.nr_start, 0),
                    )
                    .expect("valid_range")
                    .enumerate()
                    .find(|(i, _)| matches!(self.mask[*i].right, Mask::Sign | Mask::Plus))
                    .map(|(i, _)| i)
            } else {
                idx
            };
            let idx = if idx.is_none() {
                // moving sign
                self.masked
                    .graphemes(
                        TextRange::new((mask.nr_start, 0), (mask.nr_end, 0)),
                        TextPosition::new(mask.nr_start, 0),
                    )
                    .expect("valid_range")
                    .enumerate()
                    .find(|(i, g)| {
                        self.mask[*i].right == Mask::Numeric(EditDirection::Rtol)
                            && self.mask[*i].right.can_drop(g.grapheme())
                    })
                    .map(|(i, _)| i)
            } else {
                idx
            };

            if let Some(idx) = idx {
                let cc = self.map_input_c(&self.mask[idx].right, c);

                self.masked.begin_undo_seq();
                self.masked
                    .remove_char_range(TextRange::new(
                        (idx as upos_type, 0),
                        (idx as upos_type + 1, 0),
                    ))
                    .expect("valid_range");
                self.masked
                    .insert_char(TextPosition::new(idx as upos_type, 0), cc)
                    .expect("valid_range");
                self.masked.end_undo_seq();
                true
            } else {
                false
            }
        }

        /// Remove the previous char.
        pub fn remove_prev(&mut self) {
            let cursor = self.masked.cursor();

            if cursor.x == 0 {
                return;
            }

            let left = &self.mask[cursor.x as usize - 1];

            if left.right.is_rtol() {
                let l0 = &self.mask[left.sec_start as usize];

                self.masked.begin_undo_seq();
                self.masked
                    .remove_char_range(TextRange::new((cursor.x - 1, 0), cursor))
                    .expect("valid_range");
                self.masked
                    .insert_str(TextPosition::new(left.sec_start, 0), &l0.edit)
                    .expect("valid_position");
                Self::reformat(&mut self.masked, &self.mask, left.sec_start..left.sec_end);
                self.masked.end_undo_seq();
            } else if left.right.is_ltor() {
                let l9 = &self.mask[left.sec_end as usize - 1];

                self.masked.begin_undo_seq();
                self.masked
                    .remove_char_range(TextRange::new((cursor.x - 1, 0), cursor))
                    .expect("valid_range");
                self.masked
                    .insert_str(TextPosition::new(left.sec_end - 1, 0), &l9.edit)
                    .expect("valid_position");
                self.masked.end_undo_seq();
            }

            // place cursor after deletion
            if left.right.is_rtol() {
                // in a rtol field keep the cursor at the same position until the
                // whole section is empty. Only then put it at the beginning of the section
                // to continue left of the section.
                let sec_str = self
                    .masked
                    .str_slice(TextRange::new((left.sec_start, 0), (left.sec_end, 0)))
                    .expect("valid_range");
                let sec_mask = &self.mask[left.sec_start as usize..left.sec_end as usize];
                if sec_str == MaskToken::empty_section(sec_mask) {
                    self.masked
                        .set_cursor(TextPosition::new(left.sec_start, 0), false);
                } else {
                    // cursor stays
                }
            } else if left.right.is_ltor() {
                self.masked
                    .set_cursor(TextPosition::new(cursor.x - 1, 0), false);
            }
        }

        /// Remove the previous char.
        pub fn remove_next(&mut self) {
            let cursor = self.masked.cursor();

            if cursor.x as usize == self.mask.len() - 1 {
                return;
            }

            let right = &self.mask[cursor.x as usize];

            // remove and fill with empty
            if right.right.is_rtol() {
                let l0 = &self.mask[right.sec_start as usize];

                self.masked.begin_undo_seq();
                self.masked
                    .remove_char_range(TextRange::new(cursor, (cursor.x + 1, 0)))
                    .expect("valid_range");
                self.masked
                    .insert_str(TextPosition::new(right.sec_start, 0), &l0.edit)
                    .expect("valid_position");
                Self::reformat(&mut self.masked, &self.mask, right.sec_start..right.sec_end);
                self.masked.end_undo_seq();
            } else if right.right.is_ltor() {
                let l9 = &self.mask[right.sec_end as usize - 1];

                self.masked.begin_undo_seq();
                self.masked
                    .remove_char_range(TextRange::new(cursor, (cursor.x + 1, 0)))
                    .expect("valid_range");
                self.masked
                    .insert_str(TextPosition::new(right.sec_end - 1, 0), &l9.edit)
                    .expect("valid_position");
                self.masked.end_undo_seq();
            }

            // place cursor after deletion
            if right.right.is_rtol() {
                self.masked
                    .set_cursor(TextPosition::new(cursor.x + 1, 0), false);
            } else if right.right.is_ltor() {
                // in a ltor field keep the cursor at the same position until the
                // whole section is empty. Only then put it at the end of the section
                // to continue right of the section.
                let sec_str = self
                    .masked
                    .str_slice(TextRange::new((right.sec_start, 0), (right.sec_end, 0)))
                    .expect("valid_range");
                let sec_mask = &self.mask[right.sec_start as usize..right.sec_end as usize];
                if sec_str == MaskToken::empty_section(sec_mask) {
                    self.masked
                        .set_cursor(TextPosition::new(right.sec_end, 0), false);
                } else {
                    // cursor stays
                }
            }
        }

        /// Remove the selection
        pub fn remove_range(&mut self, range: Range<upos_type>) -> Result<bool, TextError> {
            // check valid range
            self.masked
                .bytes_at_range(TextRange::new((range.start, 0), (range.end, 0)))?;

            if range.is_empty() {
                return Ok(false);
            }

            let mask = &self.mask[range.start as usize];
            if range.start >= mask.sec_start && range.end <= mask.sec_end {
                if mask.right.is_rtol() {
                    self.masked.begin_undo_seq();
                    self.masked
                        .remove_str_range(TextRange::new((range.start, 0), (range.end, 0)))
                        .expect("valid_range");
                    let fill_before =
                        &self.mask[mask.sec_start as usize..mask.sec_start as usize + range.len()];
                    self.masked
                        .insert_str(
                            TextPosition::new(mask.sec_start, 0),
                            &MaskToken::empty_section(fill_before),
                        )
                        .expect("valid_range");
                    Self::reformat(&mut self.masked, &self.mask, mask.sec_start..mask.sec_end);
                    self.masked.end_undo_seq();
                } else if mask.right.is_ltor() {
                    self.masked.begin_undo_seq();
                    self.masked
                        .remove_str_range(TextRange::new((range.start, 0), (range.end, 0)))
                        .expect("valid_range");
                    let fill_after =
                        &self.mask[mask.sec_end as usize - range.len()..mask.sec_end as usize];
                    self.masked
                        .insert_str(
                            TextPosition::new(mask.sec_end - range.len() as upos_type, 0),
                            &MaskToken::empty_section(fill_after),
                        )
                        .expect("valid_range");
                    self.masked.end_undo_seq();
                }

                return Ok(true);
            }

            let mut pos = range.start;
            self.masked.begin_undo_seq();
            loop {
                let mask = &self.mask[pos as usize];

                if mask.sec_start < pos {
                    // partial start
                    if mask.right.is_rtol() {
                        self.masked
                            .remove_str_range(TextRange::new((pos, 0), (mask.sec_end, 0)))
                            .expect("valid_range");

                        let len = mask.sec_end - pos;
                        let fill_before =
                            &self.mask[mask.sec_start as usize..(mask.sec_start + len) as usize];
                        self.masked
                            .insert_str(
                                TextPosition::new(mask.sec_start, 0),
                                &MaskToken::empty_section(fill_before),
                            )
                            .expect("valid_range");

                        Self::reformat(&mut self.masked, &self.mask, mask.sec_start..mask.sec_end);
                        pos = mask.sec_end;
                    } else if mask.right.is_ltor() {
                        self.masked
                            .remove_str_range(TextRange::new((pos, 0), (mask.sec_end, 0)))
                            .expect("valid_range");

                        let fill_after = &self.mask[pos as usize..mask.sec_end as usize];
                        self.masked
                            .insert_str(
                                TextPosition::new(pos, 0),
                                &MaskToken::empty_section(fill_after),
                            )
                            .expect("valid_range");

                        pos = mask.sec_end;
                    }
                } else if mask.sec_end >= range.end {
                    // partial end
                    if mask.right.is_rtol() {
                        self.masked
                            .remove_str_range(TextRange::new((mask.sec_start, 0), (pos, 0)))
                            .expect("valid_range");

                        let fill_before = &self.mask[mask.sec_start as usize..pos as usize];
                        self.masked
                            .insert_str(
                                TextPosition::new(mask.sec_start, 0),
                                &MaskToken::empty_section(fill_before),
                            )
                            .expect("valid_range");

                        Self::reformat(&mut self.masked, &self.mask, mask.sec_start..mask.sec_end);
                        pos = mask.sec_end;
                    } else if mask.right.is_ltor() {
                        self.masked
                            .remove_str_range(TextRange::new((mask.sec_start, 0), (pos, 0)))
                            .expect("valid_range");

                        let len = pos - mask.sec_start;
                        let fill_after =
                            &self.mask[(mask.sec_end - len) as usize..mask.sec_end as usize];
                        self.masked
                            .insert_str(
                                TextPosition::new(pos, 0),
                                &MaskToken::empty_section(fill_after),
                            )
                            .expect("valid_range");

                        pos = mask.sec_end;
                    }
                } else {
                    // full section
                    if mask.right.is_rtol() {
                        self.masked
                            .remove_str_range(TextRange::new(
                                (mask.sec_start, 0),
                                (mask.sec_end, 0),
                            ))
                            .expect("valid_range");

                        let fill = &self.mask[mask.sec_start as usize..mask.sec_end as usize];
                        self.masked
                            .insert_str(
                                TextPosition::new(mask.sec_start, 0),
                                &MaskToken::empty_section(fill),
                            )
                            .expect("valid_range");

                        Self::reformat(&mut self.masked, &self.mask, mask.sec_start..mask.sec_end);
                        pos = mask.sec_end;
                    } else if mask.right.is_ltor() {
                        self.masked
                            .remove_str_range(TextRange::new(
                                (mask.sec_start, 0),
                                (mask.sec_end, 0),
                            ))
                            .expect("valid_range");

                        let fill = &self.mask[mask.sec_start as usize..mask.sec_end as usize];
                        self.masked
                            .insert_str(
                                TextPosition::new(mask.sec_start, 0),
                                &MaskToken::empty_section(fill),
                            )
                            .expect("valid_range");

                        pos = mask.sec_end;
                    }
                }

                if pos >= range.end {
                    break;
                }
            }
            self.masked.end_undo_seq();

            Ok(true)
        }

        /// Rebuild a section according to number-formatting.
        /// The main purpose is to rebuild the grouping separators.
        fn reformat(
            core: &mut TextCore<TextString>,
            mask: &Vec<MaskToken>,
            section: Range<upos_type>,
        ) {
            if !mask[section.start as usize].right.is_rtol() {
                return;
            }

            let sec_str = core
                .str_slice(TextRange::new((section.start, 0), (section.end, 0)))
                .expect("valid_range");

            // to be safe, always use our internal symbol set.
            let sym = NumberSymbols {
                decimal_sep: '.',
                decimal_grp: Some(','),
                negative_sym: '-',
                positive_sym: ' ',
                exponent_upper_sym: 'E',
                exponent_lower_sym: 'e',
                currency_sym: CurrencySym::new("$"),
            };

            // remove all non numbers and leading 0.
            let mut clean = String::new();
            _ = clean_num(sec_str.as_ref(), &sym, &mut clean);

            // create number format
            let mut tok = String::new();
            let submask = &mask[section.start as usize..section.end as usize];
            // default fmt.sym is nice
            for t in submask {
                match &t.right {
                    Mask::Digit0(_) => tok.push('0'),
                    Mask::Digit(_) => tok.push('9'),
                    Mask::Numeric(_) => tok.push('#'),
                    Mask::DecimalSep => tok.push('.'),
                    Mask::GroupingSep => tok.push(','),
                    Mask::Sign => tok.push('-'),
                    Mask::Plus => tok.push('+'),
                    Mask::Separator(s) => {
                        for c in s.chars() {
                            tok.push('\\');
                            tok.push(c);
                        }
                    }
                    Mask::None => {}
                    _ => unreachable!("invalid mask"),
                }
            }

            let fmt = match NumberFormat::news(tok, sym) {
                Ok(v) => v,
                Err(_) => unreachable!("invalid mask"),
            };
            let mut out = String::new();
            match map_num::<_, false>(clean.as_str(), &fmt, fmt.sym(), &mut out) {
                Ok(_) => {}
                Err(_) => unreachable!("invalid mask"),
            }

            core.remove_char_range(TextRange::new((section.start, 0), (section.end, 0)))
                .expect("valid_range");
            core.insert_str(TextPosition::new(section.start, 0), &out)
                .expect("valid_position");
        }

        // todo: edit
    }

    impl MaskedCore {
        /// Place cursor at decimal separator, if any.
        /// 0 otherwise.
        #[inline]
        pub fn set_default_cursor(&mut self) {
            'f: {
                for (i, t) in self.mask.iter().enumerate() {
                    if t.right == Mask::DecimalSep {
                        self.masked
                            .set_cursor(TextPosition::new(i as upos_type, 0), false);
                        self.masked
                            .set_cursor(TextPosition::new(i as upos_type, 0), true);
                        break 'f;
                    }
                }
                self.masked.set_cursor(TextPosition::new(0, 0), false);
                self.masked.set_cursor(TextPosition::new(0, 0), true);
            }
        }
    }
}
