use crate::cache::{Cache, LineWidthCache};
use crate::clipboard::Clipboard;
#[allow(deprecated)]
use crate::glyph::{Glyph, GlyphIter};
use crate::glyph2::{GlyphIter2, TextWrap2};
use crate::grapheme::Grapheme;
use crate::range_map::{expand_range_by, ranges_intersect, shrink_range_by, RangeMap};
use crate::text_store::TextStore;
use crate::undo_buffer::{StyleChange, TextPositionChange, UndoBuffer, UndoEntry, UndoOp};
use crate::{upos_type, Cursor, TextError, TextPosition, TextRange};
use dyn_clone::clone_box;
use log::debug;
use ratatui::layout::Size;
use std::borrow::Cow;
use std::cmp::min;
use std::ops::Range;

/// Core for text editing.
#[derive(Debug)]
pub struct TextCore<Store> {
    /// Text store.
    text: Store,

    /// Cursor
    cursor: TextPosition,
    /// Anchor
    anchor: TextPosition,

    /// styles
    styles: Option<Box<RangeMap>>,
    /// undo-buffer
    undo: Option<Box<dyn UndoBuffer>>,
    /// clipboard
    clip: Option<Box<dyn Clipboard>>,
    /// cache
    cache: Cache,

    /// line-break
    newline: String,
    /// tab-width
    tabs: u16,
    /// expand tabs
    expand_tabs: bool,
    /// show ctrl chars in glyphs
    glyph_ctrl: bool,
    /// show text-wrap glyphs
    wrap_ctrl: bool,
    /// use line-breaks in glyphs
    glyph_line_break: bool,
}

impl<Store: Clone> Clone for TextCore<Store> {
    fn clone(&self) -> Self {
        Self {
            text: self.text.clone(),
            cursor: self.cursor,
            anchor: self.anchor,
            styles: self.styles.clone(),
            undo: self.undo.as_ref().map(|v| clone_box(v.as_ref())),
            clip: self.clip.as_ref().map(|v| clone_box(v.as_ref())),
            cache: Default::default(),
            newline: self.newline.clone(),
            tabs: self.tabs,
            expand_tabs: self.expand_tabs,
            glyph_ctrl: self.glyph_ctrl,
            wrap_ctrl: self.wrap_ctrl,
            glyph_line_break: self.glyph_line_break,
        }
    }
}

impl<Store: TextStore + Default> TextCore<Store> {
    pub fn new(undo: Option<Box<dyn UndoBuffer>>, clip: Option<Box<dyn Clipboard>>) -> Self {
        #[cfg(windows)]
        const LINE_ENDING: &str = "\r\n";

        #[cfg(not(windows))]
        const LINE_ENDING: &str = "\n";

        Self {
            text: Store::default(),
            cursor: Default::default(),
            anchor: Default::default(),
            styles: Default::default(),
            undo,
            clip,
            cache: Default::default(),
            newline: LINE_ENDING.to_string(),
            tabs: 8,
            expand_tabs: true,
            glyph_ctrl: false,
            wrap_ctrl: false,
            glyph_line_break: true,
        }
    }

    /// Sets the line ending to be used for insert.
    /// There is no auto-detection or conversion done for set_value().
    ///
    /// Caution: If this doesn't match the line ending used in the value, you
    /// will get a value with mixed line endings.
    ///
    /// Defaults to the system line-ending.
    #[inline]
    pub fn set_newline(&mut self, br: String) {
        self.newline = br;
    }

    /// Line ending used for insert.
    #[inline]
    pub fn newline(&self) -> &str {
        &self.newline
    }

    /// Set the tab-width.
    /// Default is 8.
    #[inline]
    pub fn set_tab_width(&mut self, tabs: u16) {
        self.tabs = tabs;
    }

    /// Tab-width
    #[inline]
    pub fn tab_width(&self) -> u16 {
        self.tabs
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

    /// Show control characters when iterating glyphs.
    #[inline]
    pub fn set_glyph_ctrl(&mut self, show_ctrl: bool) {
        self.glyph_ctrl = show_ctrl;
    }

    /// Show glyphs for text-wrap.
    pub fn glyph_ctrl(&self) -> bool {
        self.glyph_ctrl
    }

    /// Show glyphs for text-wrap.
    #[inline]
    pub fn set_wrap_ctrl(&mut self, wrap_ctrl: bool) {
        self.wrap_ctrl = wrap_ctrl;
    }

    /// Show control characters when iterating glyphs.
    pub fn wrap_ctrl(&self) -> bool {
        self.wrap_ctrl
    }

    /// Handle line-breaks when iterating glyphs.
    /// If false everything is treated as one line.
    #[inline]
    pub fn set_glyph_line_break(&mut self, line_break: bool) {
        self.glyph_line_break = line_break;
    }

    /// Handle line-breaks. If false everything is treated as one line.
    pub fn glyph_line_break(&self) -> bool {
        self.glyph_line_break
    }
}

impl<Store: TextStore + Default> TextCore<Store> {
    /// Clipboard
    pub fn set_clipboard(&mut self, clip: Option<Box<dyn Clipboard + 'static>>) {
        self.clip = clip;
    }

    /// Clipboard
    pub fn clipboard(&self) -> Option<&dyn Clipboard> {
        match &self.clip {
            None => None,
            Some(v) => Some(v.as_ref()),
        }
    }
}

impl<Store: TextStore + Default> TextCore<Store> {
    /// Undo
    #[inline]
    pub fn set_undo_buffer(&mut self, undo: Option<Box<dyn UndoBuffer>>) {
        self.undo = undo;
    }

    /// Set undo count
    #[inline]
    pub fn set_undo_count(&mut self, n: u32) {
        if let Some(undo) = self.undo.as_mut() {
            undo.set_undo_count(n);
        };
    }

    /// Begin a sequence of changes that should be undone in one go.
    #[inline]
    pub fn begin_undo_seq(&mut self) {
        if let Some(undo) = self.undo.as_mut() {
            undo.begin_seq();
        };
    }

    /// End a sequence of changes that should be undone in one go.
    #[inline]
    pub fn end_undo_seq(&mut self) {
        if let Some(undo) = self.undo.as_mut() {
            undo.end_seq();
        };
    }

    /// Undo
    #[inline]
    pub fn undo_buffer(&self) -> Option<&dyn UndoBuffer> {
        match &self.undo {
            None => None,
            Some(v) => Some(v.as_ref()),
        }
    }

    /// Undo
    #[inline]
    pub fn undo_buffer_mut(&mut self) -> Option<&mut dyn UndoBuffer> {
        match &mut self.undo {
            None => None,
            Some(v) => Some(v.as_mut()),
        }
    }

    /// Undo last.
    pub fn undo(&mut self) -> bool {
        let Some(undo) = self.undo.as_mut() else {
            return false;
        };

        undo.append(UndoOp::Undo);

        self._undo()
    }

    /// Undo last.
    fn _undo(&mut self) -> bool {
        let Some(undo) = self.undo.as_mut() else {
            return false;
        };
        let undo_op = undo.undo();
        let changed = !undo_op.is_empty();
        for op in undo_op {
            match op {
                UndoOp::InsertChar {
                    bytes,
                    cursor,
                    anchor,
                    ..
                }
                | UndoOp::InsertStr {
                    bytes,
                    cursor,
                    anchor,
                    ..
                } => {
                    self.text.remove_b(bytes.clone()).expect("valid_bytes");

                    if let Some(sty) = &mut self.styles {
                        sty.remap(|r, _| Some(shrink_range_by(bytes.clone(), r)));
                    }
                    self.anchor = anchor.before;
                    self.cursor = cursor.before;
                }
                UndoOp::RemoveStr {
                    bytes,
                    cursor,
                    anchor,
                    txt,
                    styles,
                }
                | UndoOp::RemoveChar {
                    bytes,
                    cursor,
                    anchor,
                    txt,
                    styles,
                } => {
                    self.text.insert_b(bytes.start, txt).expect("valid_bytes");

                    if let Some(sty) = &mut self.styles {
                        for s in styles {
                            sty.remove(s.after.clone(), s.style);
                        }
                        for s in styles {
                            sty.add(s.before.clone(), s.style);
                        }
                        sty.remap(|r, _| {
                            if ranges_intersect(bytes.clone(), r.clone()) {
                                Some(r)
                            } else {
                                Some(expand_range_by(bytes.clone(), r))
                            }
                        });
                    }
                    self.anchor = anchor.before;
                    self.cursor = cursor.before;
                }
                UndoOp::Cursor { cursor, anchor } => {
                    self.anchor = anchor.before;
                    self.cursor = cursor.before;
                }
                UndoOp::SetStyles { styles_before, .. } => {
                    if let Some(sty) = &mut self.styles {
                        sty.set(styles_before.iter().cloned());
                    }
                }
                UndoOp::AddStyle { range, style } => {
                    if let Some(sty) = &mut self.styles {
                        sty.remove(range.clone(), *style);
                    }
                }
                UndoOp::RemoveStyle { range, style } => {
                    if let Some(sty) = &mut self.styles {
                        sty.add(range.clone(), *style);
                    }
                }
                UndoOp::SetText { .. } | UndoOp::Undo | UndoOp::Redo => {
                    unreachable!()
                }
            }
        }
        changed
    }

    /// Redo last.
    pub fn redo(&mut self) -> bool {
        let Some(undo) = self.undo.as_mut() else {
            return false;
        };

        undo.append(UndoOp::Redo);

        self._redo()
    }

    fn _redo(&mut self) -> bool {
        let Some(undo) = self.undo.as_mut() else {
            return false;
        };
        let redo_op = undo.redo();
        let changed = !redo_op.is_empty();
        for op in redo_op {
            match op {
                UndoOp::InsertChar {
                    bytes,
                    cursor,
                    anchor,
                    txt,
                }
                | UndoOp::InsertStr {
                    bytes,
                    cursor,
                    anchor,
                    txt,
                } => {
                    self.text.insert_b(bytes.start, txt).expect("valid_bytes");
                    if let Some(sty) = &mut self.styles {
                        sty.remap(|r, _| Some(expand_range_by(bytes.clone(), r)));
                    }
                    self.anchor = anchor.after;
                    self.cursor = cursor.after;
                }
                UndoOp::RemoveChar {
                    bytes,
                    cursor,
                    anchor,
                    styles,
                    ..
                }
                | UndoOp::RemoveStr {
                    bytes,
                    cursor,
                    anchor,
                    styles,
                    ..
                } => {
                    self.text.remove_b(bytes.clone()).expect("valid_bytes");

                    if let Some(sty) = &mut self.styles {
                        sty.remap(|r, _| {
                            if ranges_intersect(bytes.clone(), r.clone()) {
                                Some(r)
                            } else {
                                Some(shrink_range_by(bytes.clone(), r))
                            }
                        });
                        for s in styles {
                            sty.remove(s.before.clone(), s.style);
                        }
                        for s in styles {
                            sty.add(s.after.clone(), s.style);
                        }
                    }

                    self.anchor = anchor.after;
                    self.cursor = cursor.after;
                }
                UndoOp::Cursor { cursor, anchor } => {
                    self.anchor = anchor.after;
                    self.cursor = cursor.after;
                }

                UndoOp::SetStyles { styles_after, .. } => {
                    if let Some(sty) = &mut self.styles {
                        sty.set(styles_after.iter().cloned());
                    }
                }
                UndoOp::AddStyle { range, style } => {
                    if let Some(sty) = &mut self.styles {
                        sty.add(range.clone(), *style);
                    }
                }
                UndoOp::RemoveStyle { range, style } => {
                    if let Some(sty) = &mut self.styles {
                        sty.remove(range.clone(), *style);
                    }
                }
                UndoOp::SetText { .. } | UndoOp::Undo | UndoOp::Redo => {
                    unreachable!()
                }
            }
        }
        changed
    }

    /// Get last replay recording.
    pub fn recent_replay_log(&mut self) -> Vec<UndoEntry> {
        if let Some(undo) = &mut self.undo {
            undo.recent_replay_log()
        } else {
            Vec::default()
        }
    }

    /// Replay a recording of changes.
    pub fn replay_log(&mut self, replay: &[UndoEntry]) {
        for replay_entry in replay {
            match &replay_entry.operation {
                UndoOp::SetText { txt } => {
                    self.text.set_string(txt);
                    if let Some(sty) = &mut self.styles {
                        sty.clear();
                    }
                    if let Some(undo) = self.undo.as_mut() {
                        undo.clear();
                    };
                }
                UndoOp::InsertChar { bytes, txt, .. } | UndoOp::InsertStr { bytes, txt, .. } => {
                    self.text.insert_b(bytes.start, txt).expect("valid_range");
                    if let Some(sty) = &mut self.styles {
                        sty.remap(|r, _| Some(expand_range_by(bytes.clone(), r)));
                    }
                }
                UndoOp::RemoveChar { bytes, styles, .. }
                | UndoOp::RemoveStr { bytes, styles, .. } => {
                    self.text.remove_b(bytes.clone()).expect("valid_range");
                    if let Some(sty) = &mut self.styles {
                        sty.remap(|r, _| {
                            if ranges_intersect(bytes.clone(), r.clone()) {
                                Some(r)
                            } else {
                                Some(shrink_range_by(bytes.clone(), r))
                            }
                        });
                        for s in styles {
                            sty.remove(s.before.clone(), s.style);
                        }
                        for s in styles {
                            sty.add(s.after.clone(), s.style);
                        }
                    }
                }
                UndoOp::Cursor { .. } => {
                    // don't do cursor
                }

                UndoOp::SetStyles { styles_after, .. } => {
                    self.init_styles();
                    if let Some(sty) = &mut self.styles {
                        sty.set(styles_after.iter().cloned());
                    }
                }
                UndoOp::AddStyle { range, style } => {
                    self.init_styles();
                    if let Some(sty) = &mut self.styles {
                        sty.add(range.clone(), *style);
                    }
                }
                UndoOp::RemoveStyle { range, style } => {
                    self.init_styles();
                    if let Some(sty) = &mut self.styles {
                        sty.remove(range.clone(), *style);
                    }
                }
                UndoOp::Undo => {
                    self._undo();
                }
                UndoOp::Redo => {
                    self._redo();
                }
            }

            if let Some(undo) = self.undo.as_mut() {
                undo.append_from_replay(replay_entry.clone());
            };
        }
    }
}

impl<Store: TextStore + Default> TextCore<Store> {
    fn init_styles(&mut self) {
        if self.styles.is_none() {
            self.styles = Some(Box::new(RangeMap::default()));
        }
    }

    /// Set all styles.
    ///
    /// The ranges are byte-ranges. The usize value is the index of the
    /// actual style. Those are set with the widget.
    #[inline]
    pub fn set_styles(&mut self, new_styles: Vec<(Range<usize>, usize)>) {
        self.init_styles();

        let Some(sty) = &mut self.styles else {
            return;
        };
        if let Some(undo) = &mut self.undo {
            if undo.undo_styles_enabled() || undo.has_replay_log() {
                undo.append(UndoOp::SetStyles {
                    styles_before: sty.values().collect::<Vec<_>>(),
                    styles_after: new_styles.clone(),
                });
            }
        }
        sty.set(new_styles.iter().cloned());
    }

    /// Add a style for the given byte-range.
    ///
    /// The usize value is the index of the actual style.
    /// Those are set at the widget.
    #[inline]
    pub fn add_style(&mut self, range: Range<usize>, style: usize) {
        self.init_styles();

        if let Some(sty) = &mut self.styles {
            sty.add(range.clone(), style);
        }
        if let Some(undo) = &mut self.undo {
            if undo.undo_styles_enabled() || undo.has_replay_log() {
                undo.append(UndoOp::AddStyle { range, style });
            }
        }
    }

    /// Remove a style for the given byte-range.
    ///
    /// Range and style must match to be removed.
    #[inline]
    pub fn remove_style(&mut self, range: Range<usize>, style: usize) {
        if let Some(sty) = &mut self.styles {
            sty.remove(range.clone(), style);
        }
        if let Some(undo) = &mut self.undo {
            if undo.undo_styles_enabled() || undo.has_replay_log() {
                undo.append(UndoOp::RemoveStyle { range, style });
            }
        }
    }

    /// Find all values for the given position and writes them
    /// to the output buffer. Clears the output buffer first.
    ///
    /// This creates a cache for the styles in the given range.
    #[inline]
    pub(crate) fn styles_at_page(&self, pos: usize, range: Range<usize>, buf: &mut Vec<usize>) {
        if let Some(sty) = &self.styles {
            sty.values_at_page(pos, range, buf);
        }
    }

    /// Find all styles that touch the given range.
    #[inline]
    pub fn styles_in(&self, range: Range<usize>, buf: &mut Vec<(Range<usize>, usize)>) {
        if let Some(sty) = &self.styles {
            sty.values_in(range, buf);
        }
    }

    /// Finds all styles for the given position.
    #[inline]
    pub fn styles_at(&self, byte_pos: usize, buf: &mut Vec<(Range<usize>, usize)>) {
        if let Some(sty) = &self.styles {
            sty.values_at(byte_pos, buf);
        }
    }

    /// Check if the given style applies at the position and
    /// return the complete range for the style.
    #[inline]
    pub fn style_match(&self, byte_pos: usize, style: usize) -> Option<Range<usize>> {
        if let Some(sty) = &self.styles {
            sty.value_match(byte_pos, style)
        } else {
            None
        }
    }

    /// List of all styles.
    #[inline]
    pub fn styles(&self) -> Option<impl Iterator<Item = (Range<usize>, usize)> + '_> {
        self.styles.as_ref().map(|v| v.values())
    }
}

impl<Store: TextStore + Default> TextCore<Store> {
    /// Set the cursor position.
    /// The value is capped to the number of text lines and
    /// the line-width for the given line.
    ///
    /// Returns true, if the cursor actually changed.
    pub fn set_cursor(&mut self, mut cursor: TextPosition, extend_selection: bool) -> bool {
        let old_cursor = self.cursor;
        let old_anchor = self.anchor;

        cursor.y = min(cursor.y, self.len_lines());
        cursor.x = min(cursor.x, self.line_width(cursor.y).expect("valid-line"));

        self.cursor = cursor;
        if !extend_selection {
            self.anchor = cursor;
        }

        if let Some(undo) = self.undo.as_mut() {
            undo.append(UndoOp::Cursor {
                cursor: TextPositionChange {
                    before: old_cursor,
                    after: self.cursor,
                },
                anchor: TextPositionChange {
                    before: old_anchor,
                    after: self.anchor,
                },
            });
        }

        old_cursor != self.cursor || old_anchor != self.anchor
    }

    /// Cursor position as grapheme-idx.
    #[inline]
    pub fn cursor(&self) -> TextPosition {
        self.cursor
    }

    /// Selection anchor
    #[inline]
    pub fn anchor(&self) -> TextPosition {
        self.anchor
    }

    /// Any text selection.
    #[inline]
    pub fn has_selection(&self) -> bool {
        self.anchor != self.cursor
    }

    /// Select text.
    /// Anchor and cursor are capped to a valid value.
    #[inline]
    pub fn set_selection(&mut self, anchor: TextPosition, cursor: TextPosition) -> bool {
        let old_selection = self.selection();

        self.set_cursor(anchor, false);
        self.set_cursor(cursor, true);

        old_selection != self.selection()
    }

    /// Select all text.
    #[inline]
    pub fn select_all(&mut self) -> bool {
        let old_selection = self.selection();

        let last = self.len_lines().saturating_sub(1);
        let last_width = self.line_width(last).expect("valid_line");
        self.set_cursor(TextPosition::new(last_width, last), false);
        self.set_cursor(TextPosition::new(0, 0), true);

        old_selection != self.selection()
    }

    /// Returns the selection as TextRange.
    #[inline]
    pub fn selection(&self) -> TextRange {
        #[allow(clippy::comparison_chain)]
        if self.cursor.y < self.anchor.y {
            TextRange {
                start: self.cursor,
                end: self.anchor,
            }
        } else if self.cursor.y > self.anchor.y {
            TextRange {
                start: self.anchor,
                end: self.cursor,
            }
        } else {
            if self.cursor.x < self.anchor.x {
                TextRange {
                    start: self.cursor,
                    end: self.anchor,
                }
            } else {
                TextRange {
                    start: self.anchor,
                    end: self.cursor,
                }
            }
        }
    }
}

impl<Store: TextStore + Default> TextCore<Store> {
    /// Minimum byte position that has been changed
    /// since the last call of min_changed().
    ///
    /// Can be used to invalidate caches.
    pub(crate) fn min_changed(&self) -> Option<usize> {
        self.text.min_changed()
    }
}

impl<Store: TextStore + Default> TextCore<Store> {
    /// Empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len_lines() == 1 && self.line_width(0).expect("line") == 0
    }

    /// Grapheme position to byte position.
    /// This is the (start,end) position of the single grapheme after pos.
    #[inline]
    pub fn byte_at(&self, pos: TextPosition) -> Result<Range<usize>, TextError> {
        self.text.byte_range_at(pos)
    }

    /// Grapheme range to byte range.
    #[inline]
    pub fn bytes_at_range(&self, range: TextRange) -> Result<Range<usize>, TextError> {
        self.text.byte_range(range)
    }

    /// Byte position to grapheme position.
    /// Returns the position that contains the given byte index.
    #[inline]
    pub fn byte_pos(&self, byte: usize) -> Result<TextPosition, TextError> {
        self.text.byte_to_pos(byte)
    }

    /// Byte range to grapheme range.
    #[inline]
    pub fn byte_range(&self, bytes: Range<usize>) -> Result<TextRange, TextError> {
        self.text.bytes_to_range(bytes)
    }

    /// A range of the text as `Cow<str>`
    #[inline]
    pub fn str_slice(&self, range: TextRange) -> Result<Cow<'_, str>, TextError> {
        self.text.str_slice(range)
    }

    /// A range of the text as `Cow<str>`
    #[inline]
    pub fn str_slice_byte(&self, range: Range<usize>) -> Result<Cow<'_, str>, TextError> {
        self.text.str_slice_byte(range)
    }

    /// Iterator for the glyphs of the lines in range.
    /// Glyphs here a grapheme + display length.
    #[inline]
    #[deprecated(since = "1.1.0", note = "discontinued api")]
    #[allow(deprecated)]
    pub fn glyphs(
        &self,
        rows: Range<upos_type>,
        screen_offset: u16,
        screen_width: u16,
    ) -> Result<impl Iterator<Item = Glyph<'_>>, TextError> {
        let iter = self.graphemes(
            TextRange::new((0, rows.start), (0, rows.end)),
            TextPosition::new(0, rows.start),
        )?;

        let mut it = GlyphIter::new(TextPosition::new(0, rows.start), iter);
        it.set_screen_offset(screen_offset);
        it.set_screen_width(screen_width);
        it.set_tabs(self.tabs);
        it.set_show_ctrl(self.glyph_ctrl);
        it.set_line_break(self.glyph_line_break);
        Ok(it)
    }

    /// Limited access to the cache.
    /// Gives only access to Debug.
    #[inline]
    pub fn cache(&self) -> &Cache {
        &self.cache
    }

    /// Fill the cache for all the given rows completely.
    pub(crate) fn fill_cache(
        &self,
        rendered: Size,
        sub_row_offset: upos_type,
        rows: Range<upos_type>,
        text_wrap: TextWrap2,
        ctrl_char: bool,
        left_margin: upos_type,
        right_margin: upos_type,
        word_margin: upos_type,
    ) -> Result<(), TextError> {
        _ = self.glyphs2(
            rendered,
            sub_row_offset,
            rows,
            text_wrap,
            ctrl_char,
            left_margin,
            right_margin,
            word_margin,
        )?;
        Ok(())
    }

    /// Iterator for the glyphs of the lines in range.
    /// Glyphs here a grapheme + display length.
    #[inline]
    pub(crate) fn glyphs2(
        &self,
        rendered: Size,
        sub_row_offset: upos_type,
        rows: Range<upos_type>,
        text_wrap: TextWrap2,
        ctrl_char: bool,
        left_margin: upos_type,
        right_margin: upos_type,
        word_margin: upos_type,
    ) -> Result<GlyphIter2<Store::GraphemeIter<'_>>, TextError> {
        self.cache.validate(
            text_wrap,
            left_margin,
            rendered.width as upos_type,
            rendered.height as upos_type,
            ctrl_char,
            self.min_changed(),
        );

        let range = TextRange::new((sub_row_offset, rows.start), (0, rows.end));
        let pos = TextPosition::new(sub_row_offset, rows.start);

        let range_bytes;
        let pos_byte;

        let mut range_to_bytes = self.cache.range_to_bytes.borrow_mut();
        if let Some(cache) = range_to_bytes.get(&range) {
            range_bytes = cache.clone();
        } else {
            let cache = self.text.byte_range(range)?;
            range_to_bytes.insert(range, cache.clone());
            range_bytes = cache;
        }

        let mut pos_to_bytes = self.cache.pos_to_bytes.borrow_mut();
        if let Some(cache) = pos_to_bytes.get(&pos) {
            pos_byte = cache.start;
        } else {
            let cache = self.text.byte_range_at(pos)?;
            pos_to_bytes.insert(pos, cache.clone());
            pos_byte = cache.start;
        }

        let iter = self.graphemes_byte(range_bytes, pos_byte)?;

        let mut it = GlyphIter2::new(
            TextPosition::new(sub_row_offset, rows.start),
            iter,
            self.cache.clone(),
        );
        it.set_tabs(self.tabs as upos_type);
        it.set_show_ctrl(self.glyph_ctrl);
        it.set_wrap_ctrl(self.wrap_ctrl);
        it.set_lf_breaks(self.glyph_line_break);
        it.set_text_wrap(text_wrap);
        it.set_left_margin(left_margin);
        it.set_right_margin(right_margin);
        it.set_word_margin(word_margin);
        it.prepare()?;
        Ok(it)
    }

    /// Get the grapheme at the given position.
    #[inline]
    pub fn grapheme_at(&self, pos: TextPosition) -> Result<Option<Grapheme<'_>>, TextError> {
        let range_bytes = self.bytes_at_range(TextRange::new(pos, (pos.x + 1, pos.y)))?;
        let pos_byte = self.byte_at(pos)?.start;

        let mut it = self.text.graphemes_byte(range_bytes, pos_byte)?;

        Ok(it.next())
    }

    /// Get a cursor over all the text with the current position set at pos.
    #[inline]
    pub fn text_graphemes(
        &self,
        pos: TextPosition,
    ) -> Result<impl Cursor<Item = Grapheme<'_>>, TextError> {
        let rows = self.len_lines();
        let cols = self.line_width(rows).expect("valid_row");

        let range_bytes = self.bytes_at_range(TextRange::new((0, 0), (cols, rows)))?;
        let pos_byte = self.byte_at(pos)?.start;

        self.text.graphemes_byte(range_bytes, pos_byte)
    }

    /// Get a cursor over the text-range the current position set at pos.
    #[inline]
    pub fn graphemes(
        &self,
        range: TextRange,
        pos: TextPosition,
    ) -> Result<Store::GraphemeIter<'_>, TextError> {
        let range_bytes = self.bytes_at_range(range)?;
        let pos_byte = self.byte_at(pos)?.start;

        self.text.graphemes_byte(range_bytes, pos_byte)
    }

    /// Get a cursor over the text-range the current position set at pos.
    #[inline]
    pub fn graphemes_byte(
        &self,
        range: Range<usize>,
        pos: usize,
    ) -> Result<Store::GraphemeIter<'_>, TextError> {
        self.text.graphemes_byte(range, pos)
    }

    /// Line as str.
    ///
    /// * row must be < len_lines
    #[inline]
    pub fn line_at(&self, row: upos_type) -> Result<Cow<'_, str>, TextError> {
        self.text.line_at(row)
    }

    /// Iterate over text-lines, starting at row.
    ///
    /// * row must be < len_lines
    #[inline]
    pub fn lines_at(
        &self,
        row: upos_type,
    ) -> Result<impl Iterator<Item = Cow<'_, str>>, TextError> {
        self.text.lines_at(row)
    }

    /// Get the text for a line as iterator over the graphemes.
    #[inline]
    pub fn line_graphemes(&self, row: upos_type) -> Result<Store::GraphemeIter<'_>, TextError> {
        self.text.line_graphemes(row)
    }

    /// Line width as grapheme count. Excludes the terminating '\n'.
    #[inline]
    pub fn line_width(&self, row: upos_type) -> Result<upos_type, TextError> {
        self.cache.validate_byte_pos(self.min_changed());

        let mut line_width = self.cache.line_width.borrow_mut();
        if let Some(cache) = line_width.get(&row) {
            Ok(cache.width)
        } else {
            let width = self.text.line_width(row)?;
            let byte_pos = self.text.byte_range_at(TextPosition::new(width, row))?;
            line_width.insert(
                row,
                LineWidthCache {
                    width,
                    byte_pos: byte_pos.start,
                },
            );
            Ok(width)
        }
    }

    /// Number of lines.
    #[inline]
    pub fn len_lines(&self) -> upos_type {
        self.cache.validate_byte_pos(self.min_changed());

        if let Some(len_lines) = self.cache.len_lines.get() {
            len_lines
        } else {
            let len_lines = self.text.len_lines();
            self.cache.len_lines.set(Some(len_lines));
            len_lines
        }
    }
}

impl<Store: TextStore + Default> TextCore<Store> {
    /// Clear the internal state.
    pub fn clear(&mut self) {
        self.text.set_string("");
        self.cursor = TextPosition::default();
        self.anchor = TextPosition::default();
        if let Some(sty) = &mut self.styles {
            sty.clear();
        }
        if let Some(undo) = &mut self.undo {
            undo.clear();

            if undo.has_replay_log() {
                undo.append(UndoOp::SetText {
                    txt: self.text.string(),
                });
            }
        }
    }

    /// Copy of the text-value.
    pub fn text(&self) -> &Store {
        &self.text
    }

    /// Set the text as a TextStore
    /// Clears the styles.
    /// Caps cursor and anchor.
    pub fn set_text(&mut self, t: Store) -> bool {
        self.text = t;
        if let Some(sty) = &mut self.styles {
            sty.clear();
        }
        self.cache.clear();

        self.cursor.y = 0;
        self.cursor.x = 0;
        self.anchor.y = 0;
        self.anchor.x = 0;

        if let Some(undo) = &mut self.undo {
            undo.clear();

            if undo.has_replay_log() {
                undo.append(UndoOp::SetText {
                    txt: self.text.string(),
                });
            }
        }

        true
    }

    /// Auto-quote the selected text.
    #[allow(clippy::needless_bool)]
    pub fn insert_quotes(&mut self, mut sel: TextRange, c: char) -> Result<bool, TextError> {
        self.begin_undo_seq();

        // remove matching quotes/brackets
        if sel.end.x > 0 {
            let first = TextRange::new(sel.start, (sel.start.x + 1, sel.start.y));
            let last = TextRange::new((sel.end.x - 1, sel.end.y), sel.end);
            let c0 = self.str_slice(first).expect("valid_slice");
            let c1 = self.str_slice(last).expect("valid_slice");
            let remove_quote = if c == '\'' || c == '`' || c == '"' {
                if c0 == "'" && c1 == "'" {
                    true
                } else if c0 == "\"" && c1 == "\"" {
                    true
                } else if c0 == "`" && c1 == "`" {
                    true
                } else {
                    false
                }
            } else {
                if c0 == "<" && c1 == ">" {
                    true
                } else if c0 == "(" && c1 == ")" {
                    true
                } else if c0 == "[" && c1 == "]" {
                    true
                } else if c0 == "{" && c1 == "}" {
                    true
                } else {
                    false
                }
            };
            if remove_quote {
                self.remove_char_range(last)?;
                self.remove_char_range(first)?;
                if sel.start.y == sel.end.y {
                    sel = TextRange::new(sel.start, TextPosition::new(sel.end.x - 2, sel.end.y));
                } else {
                    sel = TextRange::new(sel.start, TextPosition::new(sel.end.x - 1, sel.end.y));
                }
            }
        }

        let cc = match c {
            '\'' => '\'',
            '`' => '`',
            '"' => '"',
            '<' => '>',
            '(' => ')',
            '[' => ']',
            '{' => '}',
            _ => unreachable!("invalid quotes"),
        };
        self.insert_char(sel.end, cc)?;
        self.insert_char(sel.start, c)?;
        if sel.start.y == sel.end.y {
            sel = TextRange::new(sel.start, TextPosition::new(sel.end.x + 2, sel.end.y));
        } else {
            sel = TextRange::new(sel.start, TextPosition::new(sel.end.x + 1, sel.end.y));
        }
        self.set_selection(sel.start, sel.end);
        self.end_undo_seq();
        Ok(true)
    }

    /// Insert a tab, either expanded or literally.
    pub fn insert_tab(&mut self, mut pos: TextPosition) -> Result<bool, TextError> {
        if self.expand_tabs {
            let n = self.tabs as upos_type - (pos.x % self.tabs as upos_type);
            for _ in 0..n {
                self.insert_char(pos, ' ')?;
                pos.x += 1;
            }
        } else {
            self.insert_char(pos, '\t')?;
        }
        Ok(true)
    }

    /// Insert a line break.
    pub fn insert_newline(&mut self, mut pos: TextPosition) -> Result<bool, TextError> {
        if self.text.is_multi_line() {
            for c in self.newline.clone().chars() {
                self.insert_char(pos, c)?;
                pos.x += 1;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Insert a character.
    pub fn insert_char(&mut self, pos: TextPosition, c: char) -> Result<bool, TextError> {
        let (inserted_range, inserted_bytes) = self.text.insert_char(pos, c)?;

        let old_cursor = self.cursor;
        let old_anchor = self.anchor;

        if let Some(sty) = &mut self.styles {
            sty.remap(|r, _| Some(expand_range_by(inserted_bytes.clone(), r)));
        }
        self.cursor = inserted_range.expand_pos(self.cursor);
        self.anchor = inserted_range.expand_pos(self.anchor);

        if let Some(undo) = self.undo.as_mut() {
            undo.append(UndoOp::InsertChar {
                bytes: inserted_bytes.clone(),
                cursor: TextPositionChange {
                    before: old_cursor,
                    after: self.cursor,
                },
                anchor: TextPositionChange {
                    before: old_anchor,
                    after: self.anchor,
                },
                txt: c.to_string(),
            });
        }

        Ok(true)
    }

    /// Insert a string at position.
    pub fn insert_str(&mut self, pos: TextPosition, t: &str) -> Result<bool, TextError> {
        let old_cursor = self.cursor;
        let old_anchor = self.anchor;

        let (inserted_range, inserted_bytes) = self.text.insert_str(pos, t)?;

        if let Some(sty) = &mut self.styles {
            sty.remap(|r, _| Some(expand_range_by(inserted_bytes.clone(), r)));
        }
        self.anchor = inserted_range.expand_pos(self.anchor);
        self.cursor = inserted_range.expand_pos(self.cursor);

        if let Some(undo) = self.undo.as_mut() {
            undo.append(UndoOp::InsertStr {
                bytes: inserted_bytes.clone(),
                cursor: TextPositionChange {
                    before: old_cursor,
                    after: self.cursor,
                },
                anchor: TextPositionChange {
                    before: old_anchor,
                    after: self.anchor,
                },
                txt: t.to_string(),
            });
        }

        Ok(true)
    }

    /// Remove the previous character
    pub fn remove_prev_char(&mut self, pos: TextPosition) -> Result<bool, TextError> {
        let (sx, sy) = if pos.y == 0 && pos.x == 0 {
            (0, 0)
        } else if pos.y > 0 && pos.x == 0 {
            let prev_line_width = self.line_width(pos.y - 1).expect("line_width"); // TODO
            (prev_line_width, pos.y - 1)
        } else {
            (pos.x - 1, pos.y)
        };
        let range = TextRange::new((sx, sy), (pos.x, pos.y));

        self.remove_char_range(range)
    }

    /// Remove the next characters.
    pub fn remove_next_char(&mut self, pos: TextPosition) -> Result<bool, TextError> {
        let c_line_width = self.line_width(pos.y)?;
        let c_last_line = self.len_lines().saturating_sub(1);

        let (ex, ey) = if pos.y == c_last_line && pos.x == c_line_width {
            (pos.x, pos.y)
        } else if pos.y != c_last_line && pos.x == c_line_width {
            (0, pos.y + 1)
        } else {
            (pos.x + 1, pos.y)
        };
        let range = TextRange::new((pos.x, pos.y), (ex, ey));

        self.remove_char_range(range)
    }

    /// Remove a range.
    /// Put it into undo as 'char-removed'.
    pub fn remove_char_range(&mut self, range: TextRange) -> Result<bool, TextError> {
        self._remove_range(range, true)
    }

    /// Remove a range
    /// Put it into undo as 'str-removed'.
    pub fn remove_str_range(&mut self, range: TextRange) -> Result<bool, TextError> {
        self._remove_range(range, false)
    }

    fn _remove_range(&mut self, range: TextRange, char_range: bool) -> Result<bool, TextError> {
        let old_cursor = self.cursor;
        let old_anchor = self.anchor;

        if range.is_empty() {
            return Ok(false);
        }

        let (old_text, (_removed_range, removed_bytes)) = self.text.remove(range)?;

        // remove deleted styles.
        let mut changed_style = Vec::new();
        if let Some(sty) = &mut self.styles {
            sty.remap(|r, s| {
                let new_range = shrink_range_by(removed_bytes.clone(), r.clone());
                if ranges_intersect(r.clone(), removed_bytes.clone()) {
                    changed_style.push(StyleChange {
                        before: r.clone(),
                        after: new_range.clone(),
                        style: s,
                    });
                    if new_range.is_empty() {
                        None
                    } else {
                        Some(new_range)
                    }
                } else {
                    Some(new_range)
                }
            });
        }
        self.anchor = range.shrink_pos(self.anchor);
        self.cursor = range.shrink_pos(self.cursor);

        if let Some(undo) = &mut self.undo {
            if char_range {
                undo.append(UndoOp::RemoveChar {
                    bytes: removed_bytes.clone(),
                    cursor: TextPositionChange {
                        before: old_cursor,
                        after: self.cursor,
                    },
                    anchor: TextPositionChange {
                        before: old_anchor,
                        after: self.anchor,
                    },
                    txt: old_text,
                    styles: changed_style,
                });
            } else {
                undo.append(UndoOp::RemoveStr {
                    bytes: removed_bytes.clone(),
                    cursor: TextPositionChange {
                        before: old_cursor,
                        after: self.cursor,
                    },
                    anchor: TextPositionChange {
                        before: old_anchor,
                        after: self.anchor,
                    },
                    txt: old_text,
                    styles: changed_style,
                });
            }
        }

        Ok(true)
    }
}

impl<Store: TextStore + Default> TextCore<Store> {
    /// Find the start of the next word. If the position is at the start
    /// or inside a word, the same position is returned.
    pub fn next_word_start(&self, pos: TextPosition) -> Result<TextPosition, TextError> {
        let mut it = self.text_graphemes(pos)?;
        let mut last_pos = it.text_offset();
        loop {
            let Some(c) = it.next() else {
                break;
            };
            last_pos = c.text_bytes().start;
            if !c.is_whitespace() {
                break;
            }
        }

        Ok(self.byte_pos(last_pos).expect("valid_pos"))
    }

    /// Find the end of the next word. Skips whitespace first, then goes on
    /// until it finds the next whitespace.
    pub fn next_word_end(&self, pos: TextPosition) -> Result<TextPosition, TextError> {
        let mut it = self.text_graphemes(pos)?;
        let mut last_pos = it.text_offset();
        let mut init = true;
        loop {
            let Some(c) = it.next() else {
                break;
            };
            last_pos = c.text_bytes().start;
            if init {
                if !c.is_whitespace() {
                    init = false;
                }
            } else {
                if c.is_whitespace() {
                    break;
                }
            }
            last_pos = c.text_bytes().end;
        }

        Ok(self.byte_pos(last_pos).expect("valid_pos"))
    }

    /// Find the start of the prev word. Skips whitespace first, then goes on
    /// until it finds the next whitespace.
    ///
    /// Attention: start/end are mirrored here compared to next_word_start/next_word_end,
    /// both return start<=end!
    pub fn prev_word_start(&self, pos: TextPosition) -> Result<TextPosition, TextError> {
        let mut it = self.text_graphemes(pos)?;
        let mut last_pos = it.text_offset();
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
            last_pos = c.text_bytes().start;
        }

        Ok(self.byte_pos(last_pos).expect("valid_pos"))
    }

    /// Find the end of the previous word. Word is everything that is not whitespace.
    /// Attention: start/end are mirrored here compared to next_word_start/next_word_end,
    /// both return start<=end!
    pub fn prev_word_end(&self, pos: TextPosition) -> Result<TextPosition, TextError> {
        let mut it = self.text_graphemes(pos)?;
        let mut last_pos = it.text_offset();
        loop {
            let Some(c) = it.prev() else {
                break;
            };
            if !c.is_whitespace() {
                break;
            }
            last_pos = c.text_bytes().start;
        }

        Ok(self.byte_pos(last_pos).expect("valid_pos"))
    }

    /// Is the position at a word boundary?
    pub fn is_word_boundary(&self, pos: TextPosition) -> Result<bool, TextError> {
        let mut it = self.text_graphemes(pos)?;
        if let Some(c0) = it.prev() {
            it.next();
            if let Some(c1) = it.next() {
                Ok(c0.is_whitespace() && !c1.is_whitespace()
                    || !c0.is_whitespace() && c1.is_whitespace())
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    /// Find the start of the word at pos.
    /// Returns pos if the position is not inside a word.
    pub fn word_start(&self, pos: TextPosition) -> Result<TextPosition, TextError> {
        let mut it = self.text_graphemes(pos)?;
        let mut last_pos = it.text_offset();
        loop {
            let Some(c) = it.prev() else {
                break;
            };
            if c.is_whitespace() {
                break;
            }
            last_pos = c.text_bytes().start;
        }

        Ok(self.byte_pos(last_pos).expect("valid_pos"))
    }

    /// Find the end of the word at pos.
    /// Returns pos if the position is not inside a word.
    pub fn word_end(&self, pos: TextPosition) -> Result<TextPosition, TextError> {
        let mut it = self.text_graphemes(pos)?;
        let mut last_pos = it.text_offset();
        loop {
            let Some(c) = it.next() else {
                break;
            };
            last_pos = c.text_bytes().start;
            if c.is_whitespace() {
                break;
            }
            last_pos = c.text_bytes().end;
        }

        Ok(self.byte_pos(last_pos).expect("valid_pos"))
    }
}
