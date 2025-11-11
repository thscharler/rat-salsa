//!
//! Text input widget.
//!
//! * Can do the usual insert/delete/movement operations.
//! * Text selection via keyboard and mouse.
//! * Scrolls with the cursor.
//! * Invalid flag.
//!
//! The visual cursor must be set separately after rendering.
//! It is accessible as [TextInputState::screen_cursor()] after rendering.
//!
//! Event handling by calling the freestanding fn [handle_events].
//! There's [handle_mouse_events] if you want to override the default key bindings but keep
//! the mouse behaviour.
//!
use crate::_private::NonExhaustive;
use crate::clipboard::{Clipboard, global_clipboard};
use crate::core::core_op::*;
use crate::core::{TextCore, TextString};
use crate::event::{ReadOnly, TextOutcome};
use crate::glyph2::{Glyph2, TextWrap2};
use crate::text_store::TextStore;
use crate::undo_buffer::{UndoBuffer, UndoEntry, UndoVec};
use crate::{
    HasScreenCursor, TextError, TextFocusGained, TextFocusLost, TextPosition, TextRange, TextStyle,
    ipos_type, upos_type,
};
use crossterm::event::KeyModifiers;
use rat_event::util::MouseFlags;
use rat_event::{HandleEvent, MouseOnly, Regular, ct_event};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_reloc::{RelocatableState, relocate_dark_offset};
use ratatui::buffer::Buffer;
use ratatui::layout::{Rect, Size};
use ratatui::prelude::BlockExt;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, StatefulWidget, Widget};
use std::borrow::Cow;
use std::cell::Cell;
use std::cmp::min;
use std::collections::HashMap;
use std::ops::Range;
use std::rc::Rc;

/// Text input widget.
///
/// # Stateful
/// This widget implements [`StatefulWidget`], you can use it with
/// [`TextInputState`] to handle common actions.
#[derive(Debug, Default, Clone)]
pub struct TextInput<'a> {
    block: Option<Block<'a>>,
    style: Style,
    focus_style: Option<Style>,
    select_style: Option<Style>,
    invalid_style: Option<Style>,
    on_focus_gained: TextFocusGained,
    on_focus_lost: TextFocusLost,
    passwd: bool,
    text_style: HashMap<usize, Style>,
}

/// State for TextInput.
#[derive(Debug)]
pub struct TextInputState {
    /// The whole area with block.
    /// __read only__ renewed with each render.
    pub area: Rect,
    /// Area inside a possible block.
    /// __read only__ renewed with each render.
    pub inner: Rect,
    /// Rendered dimension. This may differ from (inner.width, inner.height)
    /// if the text area has been relocated.
    /// __read only__ renewed with each render.
    pub rendered: Size,

    /// Display offset.
    /// __read+write__
    pub offset: upos_type,
    /// Dark offset due to clipping. Always set during rendering.
    /// __read only__ ignore this value.
    pub dark_offset: (u16, u16),
    /// __read only__ use [scroll_cursor_to_visible](TextInputState::scroll_cursor_to_visible)
    pub scroll_to_cursor: Rc<Cell<bool>>,

    /// Editing core
    pub value: TextCore<TextString>,
    /// Display as invalid.
    /// __read only__ use [set_invalid](TextInputState::set_invalid)
    pub invalid: bool,
    /// Display as password.
    /// __read only__ use [passwd](TextInput::passwd)
    pub passwd: bool,
    /// The next user edit clears the text for doing any edit.
    /// It will reset this flag. Other interactions may reset this flag too.
    /// __read only__ use [set_overwrite](TextInputState::set_overwrite)
    pub overwrite: Rc<Cell<bool>>,
    /// Focus behaviour.
    /// __read only__ use [on_focus_gained](TextInput::on_focus_gained)
    pub on_focus_gained: Rc<Cell<TextFocusGained>>,
    /// Focus behaviour.
    /// __read only__ use [on_focus_lost](TextInput::on_focus_lost)
    pub on_focus_lost: Rc<Cell<TextFocusLost>>,

    /// Current focus state.
    /// __read + write__ But don't write it.
    pub focus: FocusFlag,

    /// Mouse selection in progress.
    /// __read+write__
    pub mouse: MouseFlags,

    /// Construct with `..Default::default()`
    pub non_exhaustive: NonExhaustive,
}

impl<'a> TextInput<'a> {
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
        if styles.invalid.is_some() {
            self.invalid_style = styles.invalid;
        }
        if let Some(of) = styles.on_focus_gained {
            self.on_focus_gained = of;
        }
        if let Some(of) = styles.on_focus_lost {
            self.on_focus_lost = of;
        }
        self.block = self.block.map(|v| v.style(self.style));
        if let Some(border_style) = styles.border_style {
            self.block = self.block.map(|v| v.border_style(border_style));
        }
        if styles.block.is_some() {
            self.block = styles.block;
        }
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

    /// Display as password field.
    #[inline]
    pub fn passwd(mut self) -> Self {
        self.passwd = true;
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

    /// Preferred width: 0
    pub fn width(&self) -> u16 {
        0
    }

    /// Preferred height: 1
    pub fn height(&self) -> u16 {
        1
    }
}

impl<'a> StatefulWidget for &TextInput<'a> {
    type State = TextInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

impl StatefulWidget for TextInput<'_> {
    type State = TextInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

fn render_ref(widget: &TextInput<'_>, area: Rect, buf: &mut Buffer, state: &mut TextInputState) {
    state.area = area;
    state.inner = widget.block.inner_if_some(area);
    state.rendered = state.inner.as_size();
    state.passwd = widget.passwd;
    state.on_focus_gained.set(widget.on_focus_gained);
    state.on_focus_lost.set(widget.on_focus_lost);

    if state.scroll_to_cursor.get() {
        let c = state.cursor();
        let o = state.offset();

        if state.rendered.width > 0 {
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
        } else {
            // no render area. don't do nothing.
        }
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
            (
                style.patch(invalid_style),
                style.patch(select_style).patch(invalid_style),
            )
        } else {
            (style, style.patch(select_style))
        }
    };

    // set base style
    if let Some(block) = &widget.block {
        block.render(area, buf);
    }
    buf.set_style(state.inner, style);

    if state.inner.width == 0 || state.inner.height == 0 {
        // noop
        return;
    }

    let ox = state.offset() as u16;
    // this is just a guess at the display-width
    let show_range = {
        let start = min(ox as upos_type, state.len());
        let end = min(start + state.inner.width as upos_type, state.len());
        state.bytes_at_range(start..end)
    };
    let selection = state.selection();
    let mut styles = Vec::new();

    if widget.passwd {
        // Render as passwd
        for g in state.glyphs2() {
            if g.screen_width() > 0 {
                let mut style = style;
                state
                    .value
                    .styles_at_page(g.text_bytes().start, show_range.clone(), &mut styles);
                for style_nr in &styles {
                    if let Some(s) = widget.text_style.get(style_nr) {
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
                    cell.set_symbol("*");
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
    } else {
        for g in state.glyphs2() {
            if g.screen_width() > 0 {
                let mut style = style;
                state
                    .value
                    .styles_at_page(g.text_bytes().start, show_range.clone(), &mut styles);
                for style_nr in &styles {
                    if let Some(s) = widget.text_style.get(style_nr) {
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
}

impl Clone for TextInputState {
    fn clone(&self) -> Self {
        Self {
            area: self.area,
            inner: self.inner,
            rendered: self.rendered,
            offset: self.offset,
            dark_offset: self.dark_offset,
            scroll_to_cursor: Rc::new(Cell::new(self.scroll_to_cursor.get())),
            value: self.value.clone(),
            invalid: self.invalid,
            passwd: self.passwd,
            overwrite: Rc::new(Cell::new(self.overwrite.get())),
            on_focus_gained: Rc::new(Cell::new(self.on_focus_gained.get())),
            on_focus_lost: Rc::new(Cell::new(self.on_focus_lost.get())),
            focus: self.focus_cb(self.focus.fake_clone()),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl Default for TextInputState {
    fn default() -> Self {
        let value = TextCore::new(Some(Box::new(UndoVec::new(99))), Some(global_clipboard()));

        let mut z = Self {
            area: Default::default(),
            inner: Default::default(),
            rendered: Default::default(),
            offset: Default::default(),
            dark_offset: Default::default(),
            scroll_to_cursor: Default::default(),
            value,
            invalid: Default::default(),
            passwd: Default::default(),
            overwrite: Default::default(),
            on_focus_gained: Default::default(),
            on_focus_lost: Default::default(),
            focus: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        };
        z.focus = z.focus_cb(FocusFlag::default());
        z
    }
}

impl TextInputState {
    fn focus_cb(&self, flag: FocusFlag) -> FocusFlag {
        let on_focus_lost = self.on_focus_lost.clone();
        let cursor = self.value.shared_cursor();
        let scroll_cursor_to_visible = self.scroll_to_cursor.clone();
        flag.on_lost(move || match on_focus_lost.get() {
            TextFocusLost::None => {}
            TextFocusLost::Position0 => {
                scroll_cursor_to_visible.set(true);
                let mut new_cursor = cursor.get();
                new_cursor.cursor.x = 0;
                new_cursor.anchor.x = 0;
                cursor.set(new_cursor);
            }
        });
        let on_focus_gained = self.on_focus_gained.clone();
        let overwrite = self.overwrite.clone();
        let cursor = self.value.shared_cursor();
        let scroll_cursor_to_visible = self.scroll_to_cursor.clone();
        flag.on_gained(move || match on_focus_gained.get() {
            TextFocusGained::None => {}
            TextFocusGained::Overwrite => {
                overwrite.set(true);
            }
            TextFocusGained::SelectAll => {
                scroll_cursor_to_visible.set(true);
                let mut new_cursor = cursor.get();
                new_cursor.anchor = TextPosition::new(0, 0);
                new_cursor.cursor = TextPosition::new(0, 1);
                cursor.set(new_cursor);
            }
        });

        flag
    }
}

impl HasFocus for TextInputState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl TextInputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn named(name: &str) -> Self {
        let mut z = Self::default();
        z.focus = z.focus.with_name(name);
        z
    }

    /// Renders the widget in invalid style.
    #[inline]
    pub fn set_invalid(&mut self, invalid: bool) {
        self.invalid = invalid;
    }

    /// Renders the widget in invalid style.
    #[inline]
    pub fn invalid(&self) -> bool {
        self.invalid
    }

    /// The next edit operation will overwrite the current content
    /// instead of adding text. Any move operations will cancel
    /// this overwrite.
    #[inline]
    pub fn set_overwrite(&mut self, overwrite: bool) {
        self.overwrite.set(overwrite);
    }

    /// Will the next edit operation overwrite the content?
    #[inline]
    pub fn overwrite(&self) -> bool {
        self.overwrite.get()
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
}

impl TextInputState {
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
        if self.passwd {
            return false;
        }

        _ = clip.set_string(self.selected_text().as_ref());
        false
    }

    /// Cut to internal buffer
    #[inline]
    pub fn cut_to_clip(&mut self) -> bool {
        let Some(clip) = self.value.clipboard() else {
            return false;
        };
        if self.passwd {
            return false;
        }

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

impl TextInputState {
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

impl TextInputState {
    /// Set and replace all styles.
    ///
    /// The ranges are byte-ranges into the text.
    /// Each byte-range maps to an index into the styles set
    /// with the widget.
    ///
    /// Any style-idx that don't have a match there are just
    /// ignored. You can use this to store other range based information.
    /// The ranges are corrected during edits, no need to recalculate
    /// everything after each keystroke.
    #[inline]
    pub fn set_styles(&mut self, styles: Vec<(Range<usize>, usize)>) {
        self.value.set_styles(styles);
    }

    /// Add a style for a [TextRange].
    ///
    /// The style-idx refers to one of the styles set with the widget.
    /// Missing styles are just ignored.
    #[inline]
    pub fn add_style(&mut self, range: Range<usize>, style: usize) {
        self.value.add_style(range, style);
    }

    /// Add a style for char range.
    /// The style-nr refers to one of the styles set with the widget.
    #[inline]
    pub fn add_range_style(
        &mut self,
        range: Range<upos_type>,
        style: usize,
    ) -> Result<(), TextError> {
        let r = self
            .value
            .bytes_at_range(TextRange::new((range.start, 0), (range.end, 0)))?;
        self.value.add_style(r, style);
        Ok(())
    }

    /// Remove the exact char-range and style.
    #[inline]
    pub fn remove_style(&mut self, range: Range<usize>, style: usize) {
        self.value.remove_style(range, style);
    }

    /// Remove the exact Range<upos_type> and style.
    #[inline]
    pub fn remove_range_style(
        &mut self,
        range: Range<upos_type>,
        style: usize,
    ) -> Result<(), TextError> {
        let r = self
            .value
            .bytes_at_range(TextRange::new((range.start, 0), (range.end, 0)))?;
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
    pub fn styles_at_match(&self, byte_pos: usize, style: usize) -> Option<Range<usize>> {
        self.value.styles_at_match(byte_pos, style)
    }

    /// List of all styles.
    #[inline]
    pub fn styles(&self) -> Option<impl Iterator<Item = (Range<usize>, usize)> + '_> {
        self.value.styles()
    }
}

impl TextInputState {
    /// Text-offset.
    #[inline]
    pub fn offset(&self) -> upos_type {
        self.offset
    }

    /// Set the text-offset.
    #[inline]
    pub fn set_offset(&mut self, offset: upos_type) {
        self.scroll_to_cursor.set(false);
        self.offset = offset;
    }

    /// Cursor position.
    #[inline]
    pub fn cursor(&self) -> upos_type {
        self.value.cursor().x
    }

    /// Selection anchor.
    #[inline]
    pub fn anchor(&self) -> upos_type {
        self.value.anchor().x
    }

    /// Set the cursor position.
    /// Scrolls the cursor to a visible position.
    #[inline]
    pub fn set_cursor(&mut self, cursor: upos_type, extend_selection: bool) -> bool {
        self.scroll_cursor_to_visible();
        self.value
            .set_cursor(TextPosition::new(cursor, 0), extend_selection)
    }

    /// Selection.
    #[inline]
    pub fn has_selection(&self) -> bool {
        self.value.has_selection()
    }

    /// Selection.
    #[inline]
    pub fn selection(&self) -> Range<upos_type> {
        let mut v = self.value.selection();
        if v.start == TextPosition::new(0, 1) {
            v.start = TextPosition::new(self.line_width(), 0);
        }
        if v.end == TextPosition::new(0, 1) {
            v.end = TextPosition::new(self.line_width(), 0);
        }
        v.start.x..v.end.x
    }

    /// Selection.
    /// Scrolls the cursor to a visible position.
    #[inline]
    pub fn set_selection(&mut self, anchor: upos_type, cursor: upos_type) -> bool {
        self.scroll_cursor_to_visible();
        self.value
            .set_selection(TextPosition::new(anchor, 0), TextPosition::new(cursor, 0))
    }

    /// Selection.
    /// Scrolls the cursor to a visible position.
    #[inline]
    pub fn select_all(&mut self) -> bool {
        self.scroll_cursor_to_visible();
        self.value.select_all()
    }

    /// Selection.
    #[inline]
    pub fn selected_text(&self) -> &str {
        match self.str_slice(self.selection()) {
            Cow::Borrowed(v) => v,
            Cow::Owned(_) => {
                unreachable!()
            }
        }
    }
}

impl TextInputState {
    /// Empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Text value.
    #[inline]
    pub fn value<T: for<'a> From<&'a str>>(&self) -> T {
        self.value.text().as_str().into()
    }

    /// Text value.
    #[inline]
    pub fn text(&self) -> &str {
        self.value.text().as_str()
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
        self.value
            .str_slice(TextRange::new((range.start, 0), (range.end, 0)))
            .expect("valid_range")
    }

    /// Text slice as `Cow<str>`
    #[inline]
    pub fn try_str_slice(&self, range: Range<upos_type>) -> Result<Cow<'_, str>, TextError> {
        self.value
            .str_slice(TextRange::new((range.start, 0), (range.end, 0)))
    }

    /// Length as grapheme count.
    #[inline]
    pub fn len(&self) -> upos_type {
        self.value.line_width(0).expect("valid_row")
    }

    /// Length in bytes.
    #[inline]
    pub fn len_bytes(&self) -> usize {
        self.value.len_bytes()
    }

    /// Length as grapheme count.
    #[inline]
    pub fn line_width(&self) -> upos_type {
        self.value.line_width(0).expect("valid_row")
    }

    /// Get a cursor over all the text with the current position set at pos.
    #[inline]
    pub fn text_graphemes(&self, pos: upos_type) -> <TextString as TextStore>::GraphemeIter<'_> {
        self.value
            .text_graphemes(TextPosition::new(pos, 0))
            .expect("valid_pos")
    }

    /// Get a cursor over all the text with the current position set at pos.
    #[inline]
    pub fn try_text_graphemes(
        &self,
        pos: upos_type,
    ) -> Result<<TextString as TextStore>::GraphemeIter<'_>, TextError> {
        self.value.text_graphemes(TextPosition::new(pos, 0))
    }

    /// Get a cursor over the text-range the current position set at pos.
    #[inline]
    pub fn graphemes(
        &self,
        range: Range<upos_type>,
        pos: upos_type,
    ) -> <TextString as TextStore>::GraphemeIter<'_> {
        self.value
            .graphemes(
                TextRange::new((range.start, 0), (range.end, 0)),
                TextPosition::new(pos, 0),
            )
            .expect("valid_args")
    }

    /// Get a cursor over the text-range the current position set at pos.
    #[inline]
    pub fn try_graphemes(
        &self,
        range: Range<upos_type>,
        pos: upos_type,
    ) -> Result<<TextString as TextStore>::GraphemeIter<'_>, TextError> {
        self.value.graphemes(
            TextRange::new((range.start, 0), (range.end, 0)),
            TextPosition::new(pos, 0),
        )
    }

    /// Grapheme position to byte position.
    /// This is the (start,end) position of the single grapheme after pos.
    #[inline]
    pub fn byte_at(&self, pos: upos_type) -> Range<usize> {
        self.value
            .byte_at(TextPosition::new(pos, 0))
            .expect("valid_pos")
    }

    /// Grapheme position to byte position.
    /// This is the (start,end) position of the single grapheme after pos.
    #[inline]
    pub fn try_byte_at(&self, pos: upos_type) -> Result<Range<usize>, TextError> {
        self.value.byte_at(TextPosition::new(pos, 0))
    }

    /// Grapheme range to byte range.
    #[inline]
    pub fn bytes_at_range(&self, range: Range<upos_type>) -> Range<usize> {
        self.value
            .bytes_at_range(TextRange::new((range.start, 0), (range.end, 0)))
            .expect("valid_range")
    }

    /// Grapheme range to byte range.
    #[inline]
    pub fn try_bytes_at_range(&self, range: Range<upos_type>) -> Result<Range<usize>, TextError> {
        self.value
            .bytes_at_range(TextRange::new((range.start, 0), (range.end, 0)))
    }

    /// Byte position to grapheme position.
    /// Returns the position that contains the given byte index.
    #[inline]
    pub fn byte_pos(&self, byte: usize) -> upos_type {
        self.value.byte_pos(byte).map(|v| v.x).expect("valid_pos")
    }

    /// Byte position to grapheme position.
    /// Returns the position that contains the given byte index.
    #[inline]
    pub fn try_byte_pos(&self, byte: usize) -> Result<upos_type, TextError> {
        self.value.byte_pos(byte).map(|v| v.x)
    }

    /// Byte range to grapheme range.
    #[inline]
    pub fn byte_range(&self, bytes: Range<usize>) -> Range<upos_type> {
        self.value
            .byte_range(bytes)
            .map(|v| v.start.x..v.end.x)
            .expect("valid_range")
    }

    /// Byte range to grapheme range.
    #[inline]
    pub fn try_byte_range(&self, bytes: Range<usize>) -> Result<Range<upos_type>, TextError> {
        self.value.byte_range(bytes).map(|v| v.start.x..v.end.x)
    }
}

impl TextInputState {
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

    /// Set text.
    ///
    /// Returns an error if the text contains line-breaks.
    #[inline]
    pub fn set_value<S: Into<String>>(&mut self, s: S) {
        self.offset = 0;
        self.value.set_text(TextString::new_string(s.into()));
    }

    /// Set text.
    ///
    /// Returns an error if the text contains line-breaks.
    #[inline]
    pub fn set_text<S: Into<String>>(&mut self, s: S) {
        self.offset = 0;
        self.value.set_text(TextString::new_string(s.into()));
    }

    /// Insert a char at the current position.
    #[inline]
    pub fn insert_char(&mut self, c: char) -> bool {
        if self.has_selection() {
            self.value
                .remove_str_range(self.value.selection())
                .expect("valid_selection");
        }
        if c == '\n' {
            return false;
        } else {
            self.value
                .insert_char(self.value.cursor(), c)
                .expect("valid_cursor");
        }
        self.scroll_cursor_to_visible();
        true
    }

    /// Insert a str at the current position.
    #[inline]
    pub fn insert_str(&mut self, t: impl AsRef<str>) -> bool {
        let t = t.as_ref();
        if self.has_selection() {
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

    /// Deletes the given range.
    #[inline]
    pub fn delete_range(&mut self, range: Range<upos_type>) -> bool {
        self.try_delete_range(range).expect("valid_range")
    }

    /// Deletes the given range.
    #[inline]
    pub fn try_delete_range(&mut self, range: Range<upos_type>) -> Result<bool, TextError> {
        if !range.is_empty() {
            self.value
                .remove_str_range(TextRange::new((range.start, 0), (range.end, 0)))?;
            self.scroll_cursor_to_visible();
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl TextInputState {
    /// Delete the char after the cursor.
    #[inline]
    pub fn delete_next_char(&mut self) -> bool {
        if self.has_selection() {
            self.delete_range(self.selection())
        } else {
            let pos = self.value.cursor();
            let r = remove_next_char(&mut self.value, pos).expect("valid_cursor");
            self.scroll_cursor_to_visible();
            r
        }
    }

    /// Delete the char before the cursor.
    #[inline]
    pub fn delete_prev_char(&mut self) -> bool {
        if self.value.has_selection() {
            self.delete_range(self.selection())
        } else {
            let pos = self.value.cursor();
            let r = remove_prev_char(&mut self.value, pos).expect("valid_cursor");
            self.scroll_cursor_to_visible();
            r
        }
    }

    /// Find the start of the next word. Word is everything that is not whitespace.
    pub fn next_word_start(&self, pos: upos_type) -> upos_type {
        self.try_next_word_start(pos).expect("valid_pos")
    }

    /// Find the start of the next word. Word is everything that is not whitespace.
    pub fn try_next_word_start(&self, pos: upos_type) -> Result<upos_type, TextError> {
        next_word_start(&self.value, TextPosition::new(pos, 0)).map(|v| v.x)
    }

    /// Find the end of the next word.  Skips whitespace first, then goes on
    /// until it finds the next whitespace.
    pub fn next_word_end(&self, pos: upos_type) -> upos_type {
        self.try_next_word_end(pos).expect("valid_pos")
    }

    /// Find the end of the next word.  Skips whitespace first, then goes on
    /// until it finds the next whitespace.
    pub fn try_next_word_end(&self, pos: upos_type) -> Result<upos_type, TextError> {
        next_word_end(&self.value, TextPosition::new(pos, 0)).map(|v| v.x)
    }

    /// Find prev word. Skips whitespace first.
    /// Attention: start/end are mirrored here compared to next_word_start/next_word_end,
    /// both return start<=end!
    pub fn prev_word_start(&self, pos: upos_type) -> upos_type {
        self.try_prev_word_start(pos).expect("valid_pos")
    }

    /// Find prev word. Skips whitespace first.
    /// Attention: start/end are mirrored here compared to next_word_start/next_word_end,
    /// both return start<=end!
    pub fn try_prev_word_start(&self, pos: upos_type) -> Result<upos_type, TextError> {
        prev_word_start(&self.value, TextPosition::new(pos, 0)).map(|v| v.x)
    }

    /// Find the end of the previous word. Word is everything that is not whitespace.
    /// Attention: start/end are mirrored here compared to next_word_start/next_word_end,
    /// both return start<=end!
    pub fn prev_word_end(&self, pos: upos_type) -> upos_type {
        self.try_prev_word_end(pos).expect("valid_pos")
    }

    /// Find the end of the previous word. Word is everything that is not whitespace.
    /// Attention: start/end are mirrored here compared to next_word_start/next_word_end,
    /// both return start<=end!
    pub fn try_prev_word_end(&self, pos: upos_type) -> Result<upos_type, TextError> {
        prev_word_end(&self.value, TextPosition::new(pos, 0)).map(|v| v.x)
    }

    /// Is the position at a word boundary?
    pub fn is_word_boundary(&self, pos: upos_type) -> bool {
        self.try_is_word_boundary(pos).expect("valid_pos")
    }

    /// Is the position at a word boundary?
    pub fn try_is_word_boundary(&self, pos: upos_type) -> Result<bool, TextError> {
        is_word_boundary(&self.value, TextPosition::new(pos, 0))
    }

    /// Find the start of the word at pos.
    pub fn word_start(&self, pos: upos_type) -> upos_type {
        self.try_word_start(pos).expect("valid_pos")
    }

    /// Find the start of the word at pos.
    pub fn try_word_start(&self, pos: upos_type) -> Result<upos_type, TextError> {
        word_start(&self.value, TextPosition::new(pos, 0)).map(|v| v.x)
    }

    /// Find the end of the word at pos.
    pub fn word_end(&self, pos: upos_type) -> upos_type {
        self.try_word_end(pos).expect("valid_pos")
    }

    /// Find the end of the word at pos.
    pub fn try_word_end(&self, pos: upos_type) -> Result<upos_type, TextError> {
        word_end(&self.value, TextPosition::new(pos, 0)).map(|v| v.x)
    }

    /// Deletes the next word.
    #[inline]
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

    /// Deletes the given range.
    #[inline]
    pub fn delete_prev_word(&mut self) -> bool {
        if self.has_selection() {
            self.delete_range(self.selection())
        } else {
            let cursor = self.cursor();

            let end = self.prev_word_end(cursor);
            if end != cursor {
                self.delete_range(end..cursor)
            } else {
                let start = self.prev_word_start(cursor);
                self.delete_range(start..cursor)
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
        self.set_cursor(0, extend_selection)
    }

    /// End of line
    #[inline]
    pub fn move_to_line_end(&mut self, extend_selection: bool) -> bool {
        self.set_cursor(self.len(), extend_selection)
    }

    #[inline]
    pub fn move_to_next_word(&mut self, extend_selection: bool) -> bool {
        let cursor = self.cursor();
        let end = self.next_word_end(cursor);
        self.set_cursor(end, extend_selection)
    }

    #[inline]
    pub fn move_to_prev_word(&mut self, extend_selection: bool) -> bool {
        let cursor = self.cursor();
        let start = self.prev_word_start(cursor);
        self.set_cursor(start, extend_selection)
    }
}

impl HasScreenCursor for TextInputState {
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

impl RelocatableState for TextInputState {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.area.relocate(shift, clip);
        self.inner.relocate(shift, clip);
        // clip offset for some corrections.
        self.dark_offset = relocate_dark_offset(self.inner, shift, clip);
    }
}

impl TextInputState {
    fn glyphs2(&self) -> impl Iterator<Item = Glyph2<'_>> {
        let (text_wrap, left_margin, right_margin, word_margin) = (
            TextWrap2::Shift,
            self.offset() as upos_type,
            self.offset() as upos_type + self.rendered.width as upos_type,
            self.offset() as upos_type + self.rendered.width as upos_type,
        );
        self.value
            .glyphs2(
                self.rendered,
                0,
                0..1,
                0, /* no tabs */
                text_wrap,
                false,
                left_margin,
                right_margin,
                word_margin,
            )
            .expect("valid-row")
    }

    /// Converts from a widget relative screen coordinate to a grapheme index.
    /// x is the relative screen position.
    pub fn screen_to_col(&self, scx: i16) -> upos_type {
        let ox = self.offset();

        let scx = scx + self.dark_offset.0 as i16;

        if scx < 0 {
            ox.saturating_sub((scx as ipos_type).unsigned_abs())
        } else if scx as u16 >= self.rendered.width {
            min(ox + scx as upos_type, self.len())
        } else {
            let scx = scx as u16;

            let line = self.glyphs2();

            let mut col = ox;
            for g in line {
                if g.contains_screen_x(scx) {
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
    /// rounds the position to the next word start/end.
    ///
    /// The cursor positions are relative to the inner rect.
    /// They may be negative too, this allows setting the cursor
    /// to a position that is currently scrolled away.
    pub fn set_screen_cursor_words(&mut self, screen_cursor: i16, extend_selection: bool) -> bool {
        let anchor = self.anchor();

        let cx = self.screen_to_col(screen_cursor);
        let cursor = cx;

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
        self.scroll_to_cursor.set(true);
    }
}

impl HandleEvent<crossterm::event::Event, Regular, TextOutcome> for TextInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: Regular) -> TextOutcome {
        // small helper ...
        fn tc(r: bool) -> TextOutcome {
            if r {
                TextOutcome::TextChanged
            } else {
                TextOutcome::Unchanged
            }
        }
        fn overwrite(state: &mut TextInputState) {
            if state.overwrite.get() {
                state.overwrite.set(false);
                state.clear();
            }
        }
        fn clear_overwrite(state: &mut TextInputState) {
            state.overwrite.set(false);
        }

        // // focus behaviour
        // if self.lost_focus() {
        //     match self.on_focus_lost {
        //         TextFocusLost::None => {}
        //         TextFocusLost::Position0 => {
        //             self.move_to_line_start(false);
        //             // repaint is triggered by focus-change
        //         }
        //     }
        // }
        // if self.gained_focus() {
        //     match self.on_focus_gained {
        //         TextFocusGained::None => {}
        //         TextFocusGained::Overwrite => {
        //             self.overwrite = true;
        //         }
        //         TextFocusGained::SelectAll => {
        //             self.select_all();
        //             // repaint is triggered by focus-change
        //         }
        //     }
        // }

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
                    tc(self.delete_prev_word())
                }
                ct_event!(keycode press CONTROL-Delete) => {
                    clear_overwrite(self);
                    tc(self.delete_next_word())
                }
                ct_event!(key press CONTROL-'x') => {
                    clear_overwrite(self);
                    tc(self.cut_to_clip())
                }
                ct_event!(key press CONTROL-'v') => {
                    overwrite(self);
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
                | ct_event!(keycode release Tab)
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

impl HandleEvent<crossterm::event::Event, ReadOnly, TextOutcome> for TextInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: ReadOnly) -> TextOutcome {
        fn clear_overwrite(state: &mut TextInputState) {
            state.overwrite.set(false);
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
                    self.move_to_prev_word(false).into()
                }
                ct_event!(keycode press CONTROL-Right) => {
                    clear_overwrite(self);
                    self.move_to_next_word(false).into()
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
                    self.move_to_prev_word(true).into()
                }
                ct_event!(keycode press CONTROL_SHIFT-Right) => {
                    clear_overwrite(self);
                    self.move_to_next_word(true).into()
                }
                ct_event!(keycode press SHIFT-Home) => {
                    clear_overwrite(self);
                    self.move_to_line_start(true).into()
                }
                ct_event!(keycode press SHIFT-End) => {
                    clear_overwrite(self);
                    self.move_to_line_end(true).into()
                }
                ct_event!(keycode press ALT-Left) => {
                    clear_overwrite(self);
                    self.scroll_left(1).into()
                }
                ct_event!(keycode press ALT-Right) => {
                    clear_overwrite(self);
                    self.scroll_right(1).into()
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

impl HandleEvent<crossterm::event::Event, MouseOnly, TextOutcome> for TextInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> TextOutcome {
        fn clear_overwrite(state: &mut TextInputState) {
            state.overwrite.set(false);
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
                self.set_screen_cursor_words(cx, true).into()
            }
            ct_event!(mouse any for m) if self.mouse.doubleclick(self.inner, m) => {
                let tx = self.screen_to_col(m.column as i16 - self.inner.x as i16);
                let start = self.word_start(tx);
                let end = self.word_end(tx);
                clear_overwrite(self);
                self.set_selection(start, end).into()
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
                    self.set_screen_cursor_words(cx, true).into()
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
    state: &mut TextInputState,
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
    state: &mut TextInputState,
    event: &crossterm::event::Event,
) -> TextOutcome {
    state.handle(event, MouseOnly)
}
