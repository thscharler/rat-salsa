//! Text input widget with an input mask.
//!
//! * Can do the usual insert/delete/move operations.
//! * Text selection with keyboard + mouse
//! * Scrolls with the cursor.
//! * Modes for focus and valid.
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
//!   * `<space>` separator character move the cursor when entered.
//!   * `\`: escapes the following character and uses it as a separator.
//!   * everything else must be escaped
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
//! date_state.set_mask("99\\/99\\/9999")?;
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
use crate::glyph2::Glyph2;
use crate::text_input::TextInputState;
use crate::text_mask_core::MaskedCore;
use crate::undo_buffer::{UndoBuffer, UndoEntry};
#[allow(deprecated)]
use crate::Glyph;
use crate::{
    ipos_type, upos_type, Cursor, Grapheme, HasScreenCursor, TextError, TextFocusGained,
    TextFocusLost, TextStyle,
};
use crossterm::event::KeyModifiers;
use format_num_pattern::NumberSymbols;
use rat_event::util::MouseFlags;
use rat_event::{ct_event, HandleEvent, MouseOnly, Regular};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_reloc::{relocate_area, relocate_dark_offset, RelocatableState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Rect, Size};
use ratatui::prelude::BlockExt;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, StatefulWidget, Widget};
use std::borrow::Cow;
use std::cmp::min;
use std::fmt;
use std::ops::Range;

/// Text input widget with input mask.
///
/// # Stateful
/// This widget implements [`StatefulWidget`], you can use it with
/// [`MaskedInputState`] to handle common actions.
#[derive(Debug, Default, Clone)]
pub struct MaskedInput<'a> {
    compact: bool,
    block: Option<Block<'a>>,
    style: Style,
    focus_style: Option<Style>,
    select_style: Option<Style>,
    invalid_style: Option<Style>,
    text_style: Vec<Style>,
    on_focus_gained: TextFocusGained,
    on_focus_lost: TextFocusLost,
}

/// State & event-handling.
#[derive(Debug)]
pub struct MaskedInputState {
    /// The whole area with block.
    /// __read only__ renewed with each render.
    pub area: Rect,
    /// Area inside a possible block.
    /// __read only__ renewed with each render.
    pub inner: Rect,
    /// Rendered dimension. This may differ from (inner.width, inner.height)
    /// if the text area has been relocated.
    pub rendered: Size,
    /// Widget has been rendered in compact mode.
    /// __read only: renewed with each render.
    pub compact: bool,

    /// Display offset
    /// __read+write__
    pub offset: upos_type,
    /// Dark offset due to clipping.
    /// __read only__ secondary offset due to clipping.
    pub dark_offset: (u16, u16),
    /// __read+write__ use scroll_cursor_to_visible().
    pub scroll_to_cursor: bool,

    /// Editing core
    pub value: MaskedCore,
    /// Display as invalid.
    /// __read+write__
    pub invalid: bool,
    /// Any edit will clear the value first.
    /// This flag will be reset by any edit and navigation.
    pub overwrite: bool,
    /// Focus behaviour.
    /// __read only__
    pub on_focus_gained: TextFocusGained,
    /// Focus behaviour.
    /// __read only__
    pub on_focus_lost: TextFocusLost,

    /// Current focus state.
    /// __read+write__
    pub focus: FocusFlag,

    /// Mouse selection in progress.
    /// __read+write__
    pub mouse: MouseFlags,

    /// Construct with `..Default::default()`
    pub non_exhaustive: NonExhaustive,
}

impl<'a> MaskedInput<'a> {
    /// New widget.
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
        if styles.invalid.is_some() {
            self.invalid_style = styles.invalid;
        }
        if let Some(of) = styles.on_focus_gained {
            self.on_focus_gained = of;
        }
        if let Some(of) = styles.on_focus_lost {
            self.on_focus_lost = of;
        }
        if let Some(border_style) = styles.border_style {
            self.block = self.block.map(|v| v.border_style(border_style));
        }
        self.block = self.block.map(|v| v.style(self.style));
        if styles.block.is_some() {
            self.block = styles.block;
        }
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Base text style.
    #[inline]
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self.block = self.block.map(|v| v.style(self.style));
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
    /// Use [MaskedInputState::add_style()] to refer a text range to
    /// one of these styles.
    pub fn text_style<T: IntoIterator<Item = Style>>(mut self, styles: T) -> Self {
        self.text_style = styles.into_iter().collect();
        self
    }

    /// Block.
    #[inline]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Focus behaviour
    #[inline]
    pub fn on_focus_gained(mut self, of: TextFocusGained) -> Self {
        self.on_focus_gained = of;
        self
    }

    /// Focus behaviour
    #[inline]
    pub fn on_focus_lost(mut self, of: TextFocusLost) -> Self {
        self.on_focus_lost = of;
        self
    }
}

impl<'a> StatefulWidget for &MaskedInput<'a> {
    type State = MaskedInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

impl StatefulWidget for MaskedInput<'_> {
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
    state.rendered = state.inner.as_size();
    state.compact = widget.compact;
    state.on_focus_gained = widget.on_focus_gained;
    state.on_focus_lost = widget.on_focus_lost;

    if state.scroll_to_cursor {
        let c = state.cursor();
        let o = state.offset();
        let mut no = if c < o {
            c
        } else if c >= o + state.rendered.width as upos_type {
            c.saturating_sub(state.rendered.width as upos_type)
        } else {
            o
        };
        // correct by one at right margin. block cursors appear as part of the
        // right border otherwise.
        if c == no + state.rendered.width as upos_type {
            no = no.saturating_add(1);
        }
        state.set_offset(no);
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
    let invalid_style = if let Some(invalid_style) = widget.invalid_style {
        invalid_style
    } else {
        Style::default().red()
    };

    let (style, select_style) = if state.focus.get() {
        if state.invalid {
            (
                style.patch(focus_style).patch(invalid_style),
                style
                    .patch(focus_style)
                    .patch(select_style)
                    .patch(invalid_style),
            )
        } else {
            (
                style.patch(focus_style),
                style.patch(focus_style).patch(select_style),
            )
        }
    } else {
        if state.invalid {
            (style.patch(invalid_style), style.patch(invalid_style))
        } else {
            (style, style)
        }
    };

    // set base style
    if let Some(block) = &widget.block {
        block.render(area, buf);
    } else {
        buf.set_style(area, style);
    }

    if state.inner.width == 0 || state.inner.height == 0 {
        // noop
        return;
    }

    let ox = state.offset() as u16;
    // this is just a guess at the display-width
    let show_range = {
        let start = ox as upos_type;
        let end = min(start + state.inner.width as upos_type, state.len());
        state.bytes_at_range(start..end)
    };
    let selection = state.selection();
    let mut styles = Vec::new();

    for g in state.glyphs2() {
        if g.screen_width() > 0 {
            let mut style = style;
            styles.clear();
            state
                .value
                .styles_at_page(g.text_bytes().start, show_range.clone(), &mut styles);
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
}

impl Clone for MaskedInputState {
    fn clone(&self) -> Self {
        Self {
            area: self.area,
            inner: self.inner,
            rendered: self.rendered,
            compact: self.compact,
            offset: self.offset,
            dark_offset: self.dark_offset,
            scroll_to_cursor: self.scroll_to_cursor,
            value: self.value.clone(),
            invalid: self.invalid,
            overwrite: Default::default(),
            on_focus_gained: Default::default(),
            on_focus_lost: Default::default(),
            focus: FocusFlag::named(self.focus.name()),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl Default for MaskedInputState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            inner: Default::default(),
            rendered: Default::default(),
            compact: Default::default(),
            offset: Default::default(),
            dark_offset: Default::default(),
            scroll_to_cursor: Default::default(),
            value: Default::default(),
            invalid: Default::default(),
            overwrite: Default::default(),
            on_focus_gained: Default::default(),
            on_focus_lost: Default::default(),
            focus: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl HasFocus for MaskedInputState {
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
        let sel = self.selection();

        let has_next = self
            .value
            .next_section_range(sel.end)
            .map(|v| !v.is_empty())
            .is_some();
        let has_prev = self
            .value
            .prev_section_range(sel.start.saturating_sub(1))
            .map(|v| !v.is_empty())
            .is_some();

        if has_next {
            if has_prev {
                Navigation::Reach
            } else {
                Navigation::ReachLeaveFront
            }
        } else {
            if has_prev {
                Navigation::ReachLeaveBack
            } else {
                Navigation::Regular
            }
        }
    }
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
        self.value.set_mask(mask.as_ref())?;
        Ok(self)
    }

    /// Set symbols for number display.
    ///
    /// These are only used for rendering and to map user input.
    /// The value itself uses ".", "," and "-".
    #[inline]
    pub fn set_num_symbols(&mut self, sym: NumberSymbols) {
        self.value.set_num_symbols(sym);
    }

    /// Set the input mask. This overwrites the display mask and the value
    /// with a default representation of the mask.
    ///
    /// The result value contains all punctuation and
    /// the value given as 'display' below.
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
    /// * `SPACE`: separator character move the cursor when entered.
    /// * `\`: escapes the following character and uses it as a separator.
    /// * all other ascii characters a reserved.
    ///
    /// Inspired by <https://support.microsoft.com/en-gb/office/control-data-entry-formats-with-input-masks-e125997a-7791-49e5-8672-4a47832de8da>
    #[inline]
    pub fn set_mask<S: AsRef<str>>(&mut self, s: S) -> Result<(), fmt::Error> {
        self.value.set_mask(s)
    }

    /// Display mask.
    #[inline]
    pub fn mask(&self) -> String {
        self.value.mask()
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

    /// The next edit operation will overwrite the current content
    /// instead of adding text. Any move operations will cancel
    /// this overwrite.
    #[inline]
    pub fn set_overwrite(&mut self, overwrite: bool) {
        self.overwrite = overwrite;
    }

    /// Will the next edit operation overwrite the content?
    #[inline]
    pub fn overwrite(&self) -> bool {
        self.overwrite
    }
}

impl MaskedInputState {
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

    /// Copy to internal buffer
    #[inline]
    pub fn copy_to_clip(&mut self) -> bool {
        let Some(clip) = self.value.clipboard() else {
            return false;
        };

        _ = clip.set_string(self.selected_text().as_ref());

        true
    }

    /// Cut to internal buffer
    #[inline]
    pub fn cut_to_clip(&mut self) -> bool {
        let Some(clip) = self.value.clipboard() else {
            return false;
        };

        match clip.set_string(self.selected_text().as_ref()) {
            Ok(_) => self.delete_range(self.selection()),
            Err(_) => true,
        }
    }

    /// Paste from internal buffer.
    #[inline]
    pub fn paste_from_clip(&mut self) -> bool {
        let Some(clip) = self.value.clipboard() else {
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

impl MaskedInputState {
    /// Set and replace all styles.
    #[inline]
    pub fn set_styles(&mut self, styles: Vec<(Range<usize>, usize)>) {
        self.value.set_styles(styles);
    }

    /// Add a style for a byte-range. The style-nr refers to
    /// one of the styles set with the widget.
    #[inline]
    pub fn add_style(&mut self, range: Range<usize>, style: usize) {
        self.value.add_style(range, style);
    }

    /// Add a style for a `Range<upos_type>` to denote the cells.
    /// The style-nr refers to one of the styles set with the widget.
    #[inline]
    pub fn add_range_style(
        &mut self,
        range: Range<upos_type>,
        style: usize,
    ) -> Result<(), TextError> {
        let r = self.value.bytes_at_range(range)?;
        self.value.add_style(r, style);
        Ok(())
    }

    /// Remove the exact byte-range and style.
    #[inline]
    pub fn remove_style(&mut self, range: Range<usize>, style: usize) {
        self.value.remove_style(range, style);
    }

    /// Remove the exact `Range<upos_type>` and style.
    #[inline]
    pub fn remove_range_style(
        &mut self,
        range: Range<upos_type>,
        style: usize,
    ) -> Result<(), TextError> {
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
    pub fn styles(&self) -> Option<impl Iterator<Item = (Range<usize>, usize)> + '_> {
        self.value.styles()
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
        self.scroll_to_cursor = false;
        self.offset = offset;
    }

    /// Cursor position.
    #[inline]
    pub fn cursor(&self) -> upos_type {
        self.value.cursor()
    }

    /// Set the cursor position.
    /// Scrolls the cursor to a visible position.
    #[inline]
    pub fn set_cursor(&mut self, cursor: upos_type, extend_selection: bool) -> bool {
        self.scroll_cursor_to_visible();
        self.value.set_cursor(cursor, extend_selection)
    }

    /// Place cursor at the decimal separator, if any.
    /// 0 otherwise.
    /// Scrolls the cursor to a visible position.  
    #[inline]
    pub fn set_default_cursor(&mut self) {
        self.scroll_cursor_to_visible();
        self.value.set_default_cursor();
    }

    /// Selection anchor.
    #[inline]
    pub fn anchor(&self) -> upos_type {
        self.value.anchor()
    }

    /// Selection.
    #[inline]
    pub fn has_selection(&self) -> bool {
        self.value.has_selection()
    }

    /// Selection.
    #[inline]
    pub fn selection(&self) -> Range<upos_type> {
        self.value.selection()
    }

    /// Selection.
    /// Scrolls the cursor to a visible position.
    #[inline]
    pub fn set_selection(&mut self, anchor: upos_type, cursor: upos_type) -> bool {
        self.scroll_cursor_to_visible();
        self.value.set_selection(anchor, cursor)
    }

    /// Selection.
    /// Scrolls the cursor to a visible position.
    #[inline]
    pub fn select_all(&mut self) -> bool {
        self.scroll_cursor_to_visible();
        if let Some(section) = self.value.section_range(self.cursor()) {
            if self.selection() == section {
                self.value.select_all()
            } else {
                self.value.set_selection(section.start, section.end)
            }
        } else {
            self.value.select_all()
        }
    }

    /// Selection.
    #[inline]
    pub fn selected_text(&self) -> &str {
        self.value.selected_text()
    }
}

impl MaskedInputState {
    /// Empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Value with all punctuation and default values according to the mask type.
    #[inline]
    pub fn text(&self) -> &str {
        self.value.text()
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
    pub fn str_slice(&self, range: Range<upos_type>) -> Cow<'_, str> {
        self.value.str_slice(range).expect("valid_range")
    }

    /// Text slice as `Cow<str>`
    #[inline]
    pub fn try_str_slice(&self, range: Range<upos_type>) -> Result<Cow<'_, str>, TextError> {
        self.value.str_slice(range)
    }

    /// Length as grapheme count.
    #[inline]
    pub fn len(&self) -> upos_type {
        self.value.line_width()
    }

    /// Length as grapheme count.
    #[inline]
    pub fn line_width(&self) -> upos_type {
        self.value.line_width()
    }

    /// Iterator for the glyphs of the lines in range.
    /// Glyphs here a grapheme + display length.
    #[inline]
    #[allow(deprecated)]
    #[deprecated]
    pub fn glyphs(&self, screen_offset: u16, screen_width: u16) -> impl Iterator<Item = Glyph<'_>> {
        self.value
            .glyphs(0..1, screen_offset, screen_width)
            .expect("valid_row")
    }

    /// Iterator for the glyphs of the lines in range.
    /// Glyphs here a grapheme + display length.
    #[inline]
    #[allow(deprecated)]
    #[deprecated]
    pub fn condensed_glyphs(
        &self,
        screen_offset: u16,
        screen_width: u16,
    ) -> impl Iterator<Item = Glyph<'_>> {
        self.value
            .condensed_glyphs(0..1, screen_offset, screen_width)
            .expect("valid_row")
    }

    /// Get a cursor over all the text with the current position set at pos.
    #[inline]
    pub fn text_graphemes(&self, pos: upos_type) -> impl Cursor<Item = Grapheme<'_>> {
        self.value.text_graphemes(pos).expect("valid_pos")
    }

    /// Get a cursor over all the text with the current position set at pos.
    #[inline]
    pub fn try_text_graphemes(
        &self,
        pos: upos_type,
    ) -> Result<impl Cursor<Item = Grapheme<'_>>, TextError> {
        self.value.text_graphemes(pos)
    }

    /// Get a cursor over the text-range the current position set at pos.
    #[inline]
    pub fn graphemes(
        &self,
        range: Range<upos_type>,
        pos: upos_type,
    ) -> impl Cursor<Item = Grapheme<'_>> {
        self.value.graphemes(range, pos).expect("valid_args")
    }

    /// Get a cursor over the text-range the current position set at pos.
    #[inline]
    pub fn try_graphemes(
        &self,
        range: Range<upos_type>,
        pos: upos_type,
    ) -> Result<impl Cursor<Item = Grapheme<'_>>, TextError> {
        self.value.graphemes(range, pos)
    }

    /// Grapheme position to byte position.
    /// This is the (start,end) position of the single grapheme after pos.
    #[inline]
    pub fn byte_at(&self, pos: upos_type) -> Range<usize> {
        self.value.byte_at(pos).expect("valid_pos")
    }

    /// Grapheme position to byte position.
    /// This is the (start,end) position of the single grapheme after pos.
    #[inline]
    pub fn try_byte_at(&self, pos: upos_type) -> Result<Range<usize>, TextError> {
        self.value.byte_at(pos)
    }

    /// Grapheme range to byte range.
    #[inline]
    pub fn bytes_at_range(&self, range: Range<upos_type>) -> Range<usize> {
        self.value.bytes_at_range(range).expect("valid_range")
    }

    /// Grapheme range to byte range.
    #[inline]
    pub fn try_bytes_at_range(&self, range: Range<upos_type>) -> Result<Range<usize>, TextError> {
        self.value.bytes_at_range(range)
    }

    /// Byte position to grapheme position.
    /// Returns the position that contains the given byte index.
    #[inline]
    pub fn byte_pos(&self, byte: usize) -> upos_type {
        self.value.byte_pos(byte).expect("valid_pos")
    }

    /// Byte position to grapheme position.
    /// Returns the position that contains the given byte index.
    #[inline]
    pub fn try_byte_pos(&self, byte: usize) -> Result<upos_type, TextError> {
        self.value.byte_pos(byte)
    }

    /// Byte range to grapheme range.
    #[inline]
    pub fn byte_range(&self, bytes: Range<usize>) -> Range<upos_type> {
        self.value.byte_range(bytes).expect("valid_range")
    }

    /// Byte range to grapheme range.
    #[inline]
    pub fn try_byte_range(&self, bytes: Range<usize>) -> Result<Range<upos_type>, TextError> {
        self.value.byte_range(bytes)
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
            self.value.clear();
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
        self.value.set_text(s);
        self.value.set_default_cursor();
    }

    /// Insert a char at the current position.
    #[inline]
    pub fn insert_char(&mut self, c: char) -> bool {
        self.value.begin_undo_seq();
        if self.value.has_selection() {
            let sel = self.value.selection();
            self.value
                .remove_range(sel.clone())
                .expect("valid_selection");
            self.value.set_cursor(sel.start, false);
        }
        let c0 = self.value.advance_cursor(c);
        let c1 = self.value.insert_char(c);
        self.value.end_undo_seq();

        self.scroll_cursor_to_visible();
        c0 || c1
    }

    /// Remove the selected range. The text will be replaced with the default value
    /// as defined by the mask.
    #[inline]
    pub fn delete_range(&mut self, range: Range<upos_type>) -> bool {
        self.try_delete_range(range).expect("valid_range")
    }

    /// Remove the selected range. The text will be replaced with the default value
    /// as defined by the mask.
    #[inline]
    pub fn try_delete_range(&mut self, range: Range<upos_type>) -> Result<bool, TextError> {
        self.value.begin_undo_seq();
        let r = self.value.remove_range(range.clone())?;
        if let Some(pos) = self.value.section_cursor(range.start) {
            self.value.set_cursor(pos, false);
        }
        self.value.end_undo_seq();

        self.scroll_cursor_to_visible();
        Ok(r)
    }
}

impl MaskedInputState {
    /// Delete the char after the cursor.
    #[inline]
    pub fn delete_next_char(&mut self) -> bool {
        if self.has_selection() {
            self.delete_range(self.selection())
        } else if self.cursor() == self.len() {
            false
        } else {
            self.value.remove_next();
            self.scroll_cursor_to_visible();
            true
        }
    }

    /// Delete the char before the cursor.
    #[inline]
    pub fn delete_prev_char(&mut self) -> bool {
        if self.has_selection() {
            self.delete_range(self.selection())
        } else if self.cursor() == 0 {
            false
        } else {
            self.value.remove_prev();
            self.scroll_cursor_to_visible();
            true
        }
    }

    /// Delete the previous section.
    #[inline]
    pub fn delete_prev_section(&mut self) -> bool {
        if self.has_selection() {
            self.delete_range(self.selection())
        } else {
            if let Some(range) = self.value.prev_section_range(self.cursor()) {
                self.delete_range(range)
            } else {
                false
            }
        }
    }

    /// Delete the next section.
    #[inline]
    pub fn delete_next_section(&mut self) -> bool {
        if self.has_selection() {
            self.delete_range(self.selection())
        } else {
            if let Some(range) = self.value.next_section_range(self.cursor()) {
                self.delete_range(range)
            } else {
                false
            }
        }
    }

    /// Move to the next char.
    #[inline]
    pub fn move_right(&mut self, extend_selection: bool) -> bool {
        let c = min(self.cursor() + 1, self.len());
        self.set_cursor(c, extend_selection)
    }

    /// Move to the previous char.
    #[inline]
    pub fn move_left(&mut self, extend_selection: bool) -> bool {
        let c = self.cursor().saturating_sub(1);
        self.set_cursor(c, extend_selection)
    }

    /// Start of line
    #[inline]
    pub fn move_to_line_start(&mut self, extend_selection: bool) -> bool {
        if let Some(c) = self.value.section_cursor(self.cursor()) {
            if c != self.cursor() {
                self.set_cursor(c, extend_selection)
            } else {
                self.set_cursor(0, extend_selection)
            }
        } else {
            self.set_cursor(0, extend_selection)
        }
    }

    /// End of line
    #[inline]
    pub fn move_to_line_end(&mut self, extend_selection: bool) -> bool {
        self.set_cursor(self.len(), extend_selection)
    }

    /// Move to start of previous section.
    #[inline]
    pub fn move_to_prev_section(&mut self, extend_selection: bool) -> bool {
        if let Some(curr) = self.value.section_range(self.cursor()) {
            if self.value.cursor() != curr.start {
                return self.value.set_cursor(curr.start, extend_selection);
            }
        }
        if let Some(range) = self.value.prev_section_range(self.cursor()) {
            self.value.set_cursor(range.start, extend_selection)
        } else {
            false
        }
    }

    /// Move to end of previous section.
    #[inline]
    pub fn move_to_next_section(&mut self, extend_selection: bool) -> bool {
        if let Some(curr) = self.value.section_range(self.cursor()) {
            if self.value.cursor() != curr.end {
                return self.value.set_cursor(curr.end, extend_selection);
            }
        }
        if let Some(range) = self.value.next_section_range(self.cursor()) {
            self.value.set_cursor(range.end, extend_selection)
        } else {
            false
        }
    }

    /// Select next section.
    #[inline]
    pub fn select_current_section(&mut self) -> bool {
        let selection = self.selection();

        if let Some(next) = self.value.section_range(selection.start.saturating_sub(1)) {
            if !next.is_empty() {
                self.set_selection(next.start, next.end)
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Select next section.
    #[inline]
    pub fn select_next_section(&mut self) -> bool {
        let selection = self.selection();

        if let Some(next) = self.value.next_section_range(selection.start) {
            if !next.is_empty() {
                self.set_selection(next.start, next.end)
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Select previous section.
    #[inline]
    pub fn select_prev_section(&mut self) -> bool {
        let selection = self.selection();

        if let Some(next) = self
            .value
            .prev_section_range(selection.start.saturating_sub(1))
        {
            if !next.is_empty() {
                self.set_selection(next.start, next.end)
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl HasScreenCursor for MaskedInputState {
    /// The current text cursor as an absolute screen position.
    #[inline]
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        if self.is_focused() {
            if self.has_selection() {
                None
            } else {
                let cx = self.cursor();
                let ox = self.offset();

                if cx < ox {
                    None
                } else if cx > ox + (self.inner.width + self.dark_offset.0) as upos_type {
                    None
                } else {
                    self.col_to_screen(cx)
                        .map(|sc| (self.inner.x + sc, self.inner.y))
                }
            }
        } else {
            None
        }
    }
}

impl RelocatableState for MaskedInputState {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        // clip offset for some corrections.
        self.dark_offset = relocate_dark_offset(self.inner, shift, clip);
        self.area = relocate_area(self.area, shift, clip);
        self.inner = relocate_area(self.inner, shift, clip);
    }
}

impl MaskedInputState {
    fn glyphs2(&self) -> impl Iterator<Item = Glyph2<'_>> {
        self.value
            .glyphs2(
                self.offset(),
                self.offset() + self.rendered.width as upos_type,
                self.compact && !self.is_focused(),
            )
            .expect("valid-rows")
    }

    /// Converts from a widget relative screen coordinate to a grapheme index.
    /// x is the relative screen position.
    pub fn screen_to_col(&self, scx: i16) -> upos_type {
        let ox = self.offset();

        let scx = scx + self.dark_offset.0 as i16;

        if scx < 0 {
            ox.saturating_sub((scx as ipos_type).unsigned_abs())
        } else if scx as u16 >= (self.inner.width + self.dark_offset.0) {
            min(ox + scx as upos_type, self.len())
        } else {
            let scx = scx as u16;

            let line = self.glyphs2();

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
    pub fn col_to_screen(&self, pos: upos_type) -> Option<u16> {
        let ox = self.offset();

        if pos < ox {
            return None;
        }

        let line = self.glyphs2();
        let mut screen_x = 0;
        for g in line {
            if g.pos().x == pos {
                break;
            }
            screen_x = g.screen_pos().0 + g.screen_width();
        }

        if screen_x >= self.dark_offset.0 {
            Some(screen_x - self.dark_offset.0)
        } else {
            None
        }
    }

    /// Set the cursor position from a screen position relative to the origin
    /// of the widget. This value can be negative, which selects a currently
    /// not visible position and scrolls to it.
    #[inline]
    pub fn set_screen_cursor(&mut self, cursor: i16, extend_selection: bool) -> bool {
        let scx = cursor;

        let cx = self.screen_to_col(scx);

        self.set_cursor(cx, extend_selection)
    }

    /// Set the cursor position from screen coordinates,
    /// rounds the position to the next section bounds.
    ///
    /// The cursor positions are relative to the inner rect.
    /// They may be negative too, this allows setting the cursor
    /// to a position that is currently scrolled away.
    pub fn set_screen_cursor_sections(
        &mut self,
        screen_cursor: i16,
        extend_selection: bool,
    ) -> bool {
        let anchor = self.anchor();
        let cursor = self.screen_to_col(screen_cursor);

        let Some(range) = self.value.section_range(cursor) else {
            return false;
        };

        let cursor = if cursor < anchor {
            range.start
        } else {
            range.end
        };

        // extend anchor
        if !self.value.is_section_boundary(anchor) {
            if let Some(range) = self.value.section_range(anchor) {
                if cursor < anchor {
                    self.set_cursor(range.end, false);
                } else {
                    self.set_cursor(range.start, false);
                }
            };
        }

        self.set_cursor(cursor, extend_selection)
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
    pub fn scroll_cursor_to_visible(&mut self) {
        self.scroll_to_cursor = true;
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
        fn overwrite(state: &mut MaskedInputState) {
            if state.overwrite {
                state.overwrite = false;
                state.clear();
            }
        }
        fn clear_overwrite(state: &mut MaskedInputState) {
            state.overwrite = false;
        }

        // focus behaviour
        if self.lost_focus() {
            match self.on_focus_lost {
                TextFocusLost::None => {}
                TextFocusLost::Position0 => {
                    self.set_default_cursor();
                    self.scroll_cursor_to_visible();
                    // repaint is triggered by focus-change
                }
            }
        }
        if self.gained_focus() {
            match self.on_focus_gained {
                TextFocusGained::None => {}
                TextFocusGained::Overwrite => {
                    self.overwrite = true;
                }
                TextFocusGained::SelectAll => {
                    self.select_all();
                    // repaint is triggered by focus-change
                }
            }
        }

        let mut r = if self.is_focused() {
            match event {
                ct_event!(key press c)
                | ct_event!(key press SHIFT-c)
                | ct_event!(key press CONTROL_ALT-c) => {
                    overwrite(self);
                    tc(self.insert_char(*c))
                }
                ct_event!(keycode press Backspace) => {
                    clear_overwrite(self);
                    tc(self.delete_prev_char())
                }
                ct_event!(keycode press Delete) => {
                    clear_overwrite(self);
                    tc(self.delete_next_char())
                }
                ct_event!(keycode press CONTROL-Backspace)
                | ct_event!(keycode press ALT-Backspace) => {
                    clear_overwrite(self);
                    tc(self.delete_prev_section())
                }
                ct_event!(keycode press CONTROL-Delete) => {
                    clear_overwrite(self);
                    tc(self.delete_next_section())
                }
                ct_event!(key press CONTROL-'x') => {
                    clear_overwrite(self);
                    tc(self.cut_to_clip())
                }
                ct_event!(key press CONTROL-'v') => {
                    clear_overwrite(self);
                    tc(self.paste_from_clip())
                }
                ct_event!(key press CONTROL-'d') => {
                    clear_overwrite(self);
                    tc(self.clear())
                }
                ct_event!(key press CONTROL-'z') => {
                    clear_overwrite(self);
                    tc(self.undo())
                }
                ct_event!(key press CONTROL_SHIFT-'Z') => {
                    clear_overwrite(self);
                    tc(self.redo())
                }

                ct_event!(key release _)
                | ct_event!(key release SHIFT-_)
                | ct_event!(key release CONTROL_ALT-_)
                | ct_event!(keycode release Backspace)
                | ct_event!(keycode release Delete)
                | ct_event!(keycode release CONTROL-Backspace)
                | ct_event!(keycode release ALT-Backspace)
                | ct_event!(keycode release CONTROL-Delete)
                | ct_event!(key release CONTROL-'x')
                | ct_event!(key release CONTROL-'v')
                | ct_event!(key release CONTROL-'d')
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

impl HandleEvent<crossterm::event::Event, ReadOnly, TextOutcome> for MaskedInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: ReadOnly) -> TextOutcome {
        fn clear_overwrite(state: &mut MaskedInputState) {
            state.overwrite = false;
        }

        let mut r = if self.is_focused() {
            match event {
                ct_event!(keycode press Left) => {
                    clear_overwrite(self);
                    self.move_left(false).into()
                }
                ct_event!(keycode press Right) => {
                    clear_overwrite(self);
                    self.move_right(false).into()
                }
                ct_event!(keycode press CONTROL-Left) => {
                    clear_overwrite(self);
                    self.move_to_prev_section(false).into()
                }
                ct_event!(keycode press CONTROL-Right) => {
                    clear_overwrite(self);
                    self.move_to_next_section(false).into()
                }
                ct_event!(keycode press Home) => {
                    clear_overwrite(self);
                    self.move_to_line_start(false).into()
                }
                ct_event!(keycode press End) => {
                    clear_overwrite(self);
                    self.move_to_line_end(false).into()
                }
                ct_event!(keycode press SHIFT-Left) => {
                    clear_overwrite(self);
                    self.move_left(true).into()
                }
                ct_event!(keycode press SHIFT-Right) => {
                    clear_overwrite(self);
                    self.move_right(true).into()
                }
                ct_event!(keycode press CONTROL_SHIFT-Left) => {
                    clear_overwrite(self);
                    self.move_to_prev_section(true).into()
                }
                ct_event!(keycode press CONTROL_SHIFT-Right) => {
                    clear_overwrite(self);
                    self.move_to_next_section(true).into()
                }
                ct_event!(keycode press SHIFT-Home) => {
                    clear_overwrite(self);
                    self.move_to_line_start(true).into()
                }
                ct_event!(keycode press SHIFT-End) => {
                    clear_overwrite(self);
                    self.move_to_line_end(true).into()
                }
                ct_event!(keycode press Tab) => {
                    // ignore tab from focus
                    if !self.focus.gained() {
                        clear_overwrite(self);
                        self.select_next_section().into()
                    } else {
                        TextOutcome::Unchanged
                    }
                }
                ct_event!(keycode press SHIFT-BackTab) => {
                    // ignore tab from focus
                    if !self.focus.gained() {
                        clear_overwrite(self);
                        self.select_prev_section().into()
                    } else {
                        TextOutcome::Unchanged
                    }
                }
                ct_event!(key press CONTROL-'a') => {
                    clear_overwrite(self);
                    self.select_all().into()
                }
                ct_event!(key press CONTROL-'c') => {
                    clear_overwrite(self);
                    self.copy_to_clip().into()
                }

                ct_event!(keycode release Left)
                | ct_event!(keycode release Right)
                | ct_event!(keycode release CONTROL-Left)
                | ct_event!(keycode release CONTROL-Right)
                | ct_event!(keycode release Home)
                | ct_event!(keycode release End)
                | ct_event!(keycode release SHIFT-Left)
                | ct_event!(keycode release SHIFT-Right)
                | ct_event!(keycode release CONTROL_SHIFT-Left)
                | ct_event!(keycode release CONTROL_SHIFT-Right)
                | ct_event!(keycode release SHIFT-Home)
                | ct_event!(keycode release SHIFT-End)
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

impl HandleEvent<crossterm::event::Event, MouseOnly, TextOutcome> for MaskedInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> TextOutcome {
        fn clear_overwrite(state: &mut MaskedInputState) {
            state.overwrite = false;
        }

        match event {
            ct_event!(mouse any for m) if self.mouse.drag(self.inner, m) => {
                let c = (m.column as i16) - (self.inner.x as i16);
                clear_overwrite(self);
                self.set_screen_cursor(c, true).into()
            }
            ct_event!(mouse any for m) if self.mouse.drag2(self.inner, m, KeyModifiers::ALT) => {
                let cx = m.column as i16 - self.inner.x as i16;
                clear_overwrite(self);
                self.set_screen_cursor_sections(cx, true).into()
            }
            ct_event!(mouse any for m) if self.mouse.doubleclick(self.inner, m) => {
                let tx = self.screen_to_col(m.column as i16 - self.inner.x as i16);
                clear_overwrite(self);
                if let Some(range) = self.value.section_range(tx) {
                    self.set_selection(range.start, range.end).into()
                } else {
                    TextOutcome::Unchanged
                }
            }
            ct_event!(mouse down Left for column,row) => {
                if self.gained_focus() {
                    // don't react to the first click that's for
                    // focus. this one shouldn't demolish the selection.
                    TextOutcome::Unchanged
                } else if self.inner.contains((*column, *row).into()) {
                    let c = (column - self.inner.x) as i16;
                    clear_overwrite(self);
                    self.set_screen_cursor(c, false).into()
                } else {
                    TextOutcome::Continue
                }
            }
            ct_event!(mouse down CONTROL-Left for column,row) => {
                if self.inner.contains((*column, *row).into()) {
                    let cx = (column - self.inner.x) as i16;
                    clear_overwrite(self);
                    self.set_screen_cursor(cx, true).into()
                } else {
                    TextOutcome::Continue
                }
            }
            ct_event!(mouse down ALT-Left for column,row) => {
                if self.inner.contains((*column, *row).into()) {
                    let cx = (column - self.inner.x) as i16;
                    clear_overwrite(self);
                    self.set_screen_cursor_sections(cx, true).into()
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
