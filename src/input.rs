//!
//! Text input.
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
use crate::event::{ReadOnly, TextOutcome};
use crate::fill::Fill;
use crate::text::graphemes::{
    is_word_boundary, next_word_end, next_word_start, prev_word_end, prev_word_start, split3,
    word_end, word_start,
};
use crate::text::textinput_core::TextInputCore;
#[allow(unused_imports)]
use log::debug;
use rat_event::util::MouseFlags;
use rat_event::{ct_event, HandleEvent, MouseOnly, Regular};
use rat_focus::{FocusFlag, HasFocusFlag};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::BlockExt;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, StatefulWidget, StatefulWidgetRef, Widget};
use std::cmp::min;
use std::ops::Range;

/// Text input widget.
#[derive(Debug, Default, Clone)]
pub struct TextInput<'a> {
    block: Option<Block<'a>>,
    style: Style,
    focus_style: Option<Style>,
    select_style: Option<Style>,
    invalid_style: Option<Style>,
}

/// State for TextInput.
#[derive(Debug, Clone)]
pub struct TextInputState {
    /// Current focus state.
    pub focus: FocusFlag,
    /// Display as invalid.
    pub invalid: bool,

    /// The whole area with block.
    pub area: Rect,
    /// Area inside a possible block.
    pub inner: Rect,

    /// Display offset
    pub offset: usize,
    /// Editing core
    pub value: TextInputCore,

    /// Mouse selection in progress.
    pub mouse: MouseFlags,
    /// Construct with `..Default::default()`
    pub non_exhaustive: NonExhaustive,
}

/// Combined style for the widget.
#[derive(Debug, Clone)]
pub struct TextInputStyle {
    pub style: Style,
    pub focus: Option<Style>,
    pub select: Option<Style>,
    pub invalid: Option<Style>,
    pub non_exhaustive: NonExhaustive,
}

impl Default for TextInputStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            focus: Default::default(),
            select: Default::default(),
            invalid: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a> TextInput<'a> {
    /// New widget.
    pub fn new() -> Self {
        Self::default()
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

    /// Block.
    #[inline]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> StatefulWidgetRef for TextInput<'a> {
    type State = TextInputState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

impl<'a> StatefulWidget for TextInput<'a> {
    type State = TextInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

fn render_ref(widget: &TextInput<'_>, area: Rect, buf: &mut Buffer, state: &mut TextInputState) {
    state.area = area;
    state.inner = widget.block.inner_if_some(area);

    widget.block.render(area, buf);

    let area = state.inner;

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

    Fill::new().style(style).render(area, buf);

    let selection = state.selection();

    let ox = state.offset();
    let mut col = 0u16;
    let mut tx = ox;

    let mut glyph_iter = state.value.value_glyphs();
    glyph_iter.set_offset(ox);

    'line: for d in glyph_iter {
        if d.len > 0 {
            let style = if selection.contains(&tx) {
                select_style
            } else {
                style
            };

            let cell = buf.get_mut(area.x + col, area.y);
            cell.set_symbol(d.glyph.as_ref());
            cell.set_style(style);
            col += 1;
            if col >= area.width {
                break 'line;
            }

            for _ in 1..d.len {
                let cell = buf.get_mut(area.x + col, area.y);
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

impl Default for TextInputState {
    fn default() -> Self {
        Self {
            focus: Default::default(),
            invalid: false,
            area: Default::default(),
            inner: Default::default(),
            mouse: Default::default(),
            value: Default::default(),
            non_exhaustive: NonExhaustive,
            offset: 0,
        }
    }
}

impl TextInputState {
    pub fn new() -> Self {
        Self::default()
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

    /// Offset shown.
    #[inline]
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Offset shown. This is corrected if the cursor wouldn't be visible.
    #[inline]
    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    /// Cursor position.
    #[inline]
    pub fn cursor(&self) -> usize {
        self.value.cursor()
    }

    /// Set the cursor position, reset selection.
    #[inline]
    pub fn set_cursor(&mut self, cursor: usize, extend_selection: bool) -> bool {
        self.value.set_cursor(cursor, extend_selection)
    }

    /// Set the cursor and anchor to the defaults.
    /// Exists just to mirror MaskedInput.
    #[inline]
    pub fn set_default_cursor(&mut self) {
        self.value.set_default_cursor();
    }

    /// Selection anchor.
    #[inline]
    pub fn anchor(&self) -> usize {
        self.value.anchor()
    }

    /// Text value.
    #[inline]
    pub fn value(&self) -> &str {
        self.value.value()
    }

    /// Create a default value according to the mask.
    /// Exists just to mirror MaskedInput.
    #[inline]
    pub fn default_value(&self) -> String {
        self.value.default_value()
    }

    /// Set text.
    #[inline]
    pub fn set_value<S: Into<String>>(&mut self, s: S) {
        self.offset = 0;
        self.value.set_value(s);
    }

    /// Empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Length as grapheme count.
    #[inline]
    pub fn len(&self) -> usize {
        self.value.len()
    }

    /// Selection.
    #[inline]
    pub fn has_selection(&self) -> bool {
        self.value.has_selection()
    }

    /// Selection.
    #[inline]
    pub fn selection(&self) -> Range<usize> {
        self.value.selection()
    }

    /// Selection.
    #[inline]
    pub fn set_selection(&mut self, anchor: usize, cursor: usize) -> bool {
        self.value.set_selection(anchor, cursor)
    }

    /// Selection.
    #[inline]
    pub fn select_all(&mut self) -> bool {
        self.value.select_all()
    }

    /// Selection.
    #[inline]
    pub fn selected_value(&self) -> &str {
        split3(self.value.value(), self.value.selection()).1
    }

    /// Change the offset in a way that the cursor is visible.
    pub fn scroll_cursor_to_visible(&mut self) -> bool {
        let old_offset = self.offset();

        let c = self.cursor();
        let o = self.offset();

        let no = if c < o {
            c
        } else if c >= o + self.inner.width as usize {
            c.saturating_sub(self.inner.width as usize)
        } else {
            o
        };

        self.set_offset(no);

        self.offset() != old_offset
    }

    /// Insert a char at the current position.
    #[inline]
    pub fn insert_char(&mut self, c: char) -> bool {
        if self.value.has_selection() {
            self.value.remove_range(self.value.selection());
        }
        self.value.insert_char(self.value.cursor(), c);
        self.scroll_cursor_to_visible();
        true
    }

    /// Insert a str at the current position.
    #[inline]
    pub fn insert_str(&mut self, t: &str) -> bool {
        if self.value.has_selection() {
            self.value.remove_range(self.value.selection());
        }
        self.value.insert_str(self.value.cursor(), t);
        self.scroll_cursor_to_visible();
        true
    }

    /// Deletes the given range.
    #[inline]
    pub fn delete_range(&mut self, range: Range<usize>) -> bool {
        if !range.is_empty() {
            self.value.remove_range(range);
            self.scroll_cursor_to_visible();
            true
        } else {
            false
        }
    }

    /// Find the start of the next word. Word is everything that is not whitespace.
    pub fn next_word_start(&self, pos: usize) -> usize {
        next_word_start(self.value(), pos)
    }

    /// Find the end of the next word.  Skips whitespace first, then goes on
    /// until it finds the next whitespace.
    pub fn next_word_end(&self, pos: usize) -> usize {
        next_word_end(self.value(), pos)
    }

    /// Find prev word. Skips whitespace first.
    /// Attention: start/end are mirrored here compared to next_word_start/next_word_end,
    /// both return start<=end!
    pub fn prev_word_start(&self, pos: usize) -> usize {
        prev_word_start(self.value(), pos)
    }

    /// Find the end of the previous word. Word is everything that is not whitespace.
    /// Attention: start/end are mirrored here compared to next_word_start/next_word_end,
    /// both return start<=end!
    pub fn prev_word_end(&self, pos: usize) -> usize {
        prev_word_end(self.value(), pos)
    }

    /// Is the position at a word boundary?
    pub fn is_word_boundary(&self, pos: usize) -> bool {
        is_word_boundary(self.value(), pos)
    }

    /// Find the start of the word at pos.
    pub fn word_start(&self, pos: usize) -> usize {
        word_start(self.value(), pos)
    }

    /// Find the end of the word at pos.
    pub fn word_end(&self, pos: usize) -> usize {
        word_end(self.value(), pos)
    }

    /// Deletes the next word.
    #[inline]
    pub fn delete_next_word(&mut self) -> bool {
        if self.has_selection() {
            self.delete_range(self.selection())
        } else {
            let cp = self.cursor();
            let sp = self.next_word_start(cp);

            let ep = if cp == sp {
                next_word_end(self.value(), cp)
            } else {
                sp
            };

            self.delete_range(cp..ep)
        }
    }

    /// Deletes the given range.
    #[inline]
    pub fn delete_prev_word(&mut self) -> bool {
        if self.has_selection() {
            self.delete_range(self.selection())
        } else {
            let cp = self.cursor();
            let ep = prev_word_end(self.value(), cp);

            let sp = if cp == ep {
                prev_word_start(self.value(), cp)
            } else {
                ep
            };

            self.delete_range(sp..cp)
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
            self.delete_range(self.cursor() - 1..self.cursor())
        }
    }

    /// Delete the char after the cursor.
    #[inline]
    pub fn delete_next_char(&mut self) -> bool {
        if self.has_selection() {
            self.delete_range(self.selection())
        } else if self.cursor() == self.len() {
            false
        } else {
            self.delete_range(self.cursor()..self.cursor() + 1)
        }
    }

    #[inline]
    pub fn move_to_next_word(&mut self, extend_selection: bool) -> bool {
        let cp = self.cursor();
        let ep = next_word_end(self.value(), cp);
        let c = self.set_cursor(ep, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    #[inline]
    pub fn move_to_prev_word(&mut self, extend_selection: bool) -> bool {
        let cp = self.cursor();
        let sp = prev_word_start(self.value(), cp);
        let c = self.set_cursor(sp, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move to the next char.
    #[inline]
    pub fn move_to_next(&mut self, extend_selection: bool) -> bool {
        let c = min(self.cursor() + 1, self.len());
        let c = self.set_cursor(c, extend_selection);
        let s = self.scroll_cursor_to_visible();
        c || s
    }

    /// Move to the previous char.
    #[inline]
    pub fn move_to_prev(&mut self, extend_selection: bool) -> bool {
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

    /// Converts a grapheme based position to a screen position
    /// relative to the widget area.
    pub fn to_screen_col(&self, pos: usize) -> Option<u16> {
        let px = pos;
        let ox = self.offset();

        let mut line = self.value.value_glyphs();
        line.set_offset(ox);

        let sx = line.take(px - ox).map(|v| v.len).sum::<usize>();

        Some(sx as u16)
    }

    /// Converts from a widget relative screen coordinate to a grapheme index.
    /// x is the relative screen position.
    pub fn from_screen_col(&self, scx: i16) -> usize {
        let ox = self.offset();

        if scx < 0 {
            ox.saturating_sub((scx as isize).abs() as usize)
        } else if scx as u16 >= self.area.width {
            min(ox + scx as usize, self.len())
        } else {
            let scx = scx as u16;

            let mut line = self.value.value_glyphs();
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

            min(ox + cx, self.len())
        }
    }

    /// Set the cursor position from a screen position relative to the origin
    /// of the widget. This value can be negative, which selects a currently
    /// not visible position and scrolls to it.
    #[inline]
    pub fn set_screen_cursor(&mut self, cursor: i16, extend_selection: bool) -> bool {
        let scx = cursor;

        let cx = self.from_screen_col(scx);

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
            } else if cx > ox + self.inner.width as usize {
                None
            } else {
                let sc = self.to_screen_col(cx).expect("valid_cursor");
                Some((self.inner.x + sc, self.inner.y))
            }
        } else {
            None
        }
    }
}

impl HasFocusFlag for TextInputState {
    #[inline]
    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    #[inline]
    fn area(&self) -> Rect {
        self.area
    }
}

impl HandleEvent<crossterm::event::Event, Regular, TextOutcome> for TextInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: Regular) -> TextOutcome {
        let mut r = if self.is_focused() {
            match event {
                ct_event!(key press c)
                | ct_event!(key press SHIFT-c)
                | ct_event!(key press CONTROL_ALT-c) => self.insert_char(*c).into(),
                ct_event!(keycode press Backspace) => self.delete_prev_char().into(),
                ct_event!(keycode press Delete) => self.delete_next_char().into(),
                ct_event!(keycode press CONTROL-Backspace) => self.delete_prev_word().into(),
                ct_event!(keycode press CONTROL-Delete) => self.delete_next_word().into(),
                ct_event!(key press CONTROL-'d') => self.clear().into(),

                ct_event!(key release _)
                | ct_event!(key release SHIFT-_)
                | ct_event!(key release CONTROL_ALT-_)
                | ct_event!(keycode release Backspace)
                | ct_event!(keycode release Delete)
                | ct_event!(keycode release CONTROL-Backspace)
                | ct_event!(keycode release CONTROL-Delete)
                | ct_event!(key release CONTROL-'d') => TextOutcome::Unchanged,

                _ => TextOutcome::Continue,
            }
        } else {
            TextOutcome::Continue
        };
        // remap to TextChanged, comes from the bool->OutCome conversion!
        if r == TextOutcome::Changed {
            r = TextOutcome::TextChanged;
        }

        if r == TextOutcome::Continue {
            r = self.handle(event, ReadOnly);
        }
        r
    }
}

impl HandleEvent<crossterm::event::Event, ReadOnly, TextOutcome> for TextInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: ReadOnly) -> TextOutcome {
        let mut r = if self.is_focused() {
            match event {
                ct_event!(keycode press Left) => self.move_to_prev(false).into(),
                ct_event!(keycode press Right) => self.move_to_next(false).into(),
                ct_event!(keycode press CONTROL-Left) => self.move_to_prev_word(false).into(),
                ct_event!(keycode press CONTROL-Right) => self.move_to_next_word(false).into(),
                ct_event!(keycode press Home) => self.move_to_line_start(false).into(),
                ct_event!(keycode press End) => self.move_to_line_end(false).into(),
                ct_event!(keycode press SHIFT-Left) => self.move_to_prev(true).into(),
                ct_event!(keycode press SHIFT-Right) => self.move_to_next(true).into(),
                ct_event!(keycode press CONTROL_SHIFT-Left) => self.move_to_prev_word(true).into(),
                ct_event!(keycode press CONTROL_SHIFT-Right) => self.move_to_next_word(true).into(),
                ct_event!(keycode press SHIFT-Home) => self.move_to_line_start(true).into(),
                ct_event!(keycode press SHIFT-End) => self.move_to_line_end(true).into(),
                ct_event!(key press CONTROL-'a') => self.select_all().into(),

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

impl HandleEvent<crossterm::event::Event, MouseOnly, TextOutcome> for TextInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> TextOutcome {
        match event {
            ct_event!(mouse any for m) if self.mouse.drag(self.area, m) => {
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
