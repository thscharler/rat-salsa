use crate::cache::{Cache, LineWidthCache};
use crate::clipboard::Clipboard;
use crate::glyph2::{GlyphIter2, TextWrap2};
use crate::grapheme::Grapheme;
use crate::range_map::{RangeMap, expand_range_by, ranges_intersect, shrink_range_by};
use crate::text_store::TextStore;
use crate::undo_buffer::{StyleChange, TextPositionChange, UndoBuffer, UndoEntry, UndoOp};
use crate::{TextError, TextPosition, TextRange, upos_type};
use dyn_clone::clone_box;
use ratatui::layout::Size;
use std::borrow::Cow;
use std::cell::Cell;
use std::cmp::min;
use std::ops::Range;
use std::rc::Rc;

pub mod core_op;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Hash)]
pub(crate) struct TextCursor {
    pub(crate) anchor: TextPosition,
    pub(crate) cursor: TextPosition,
}

impl TextCursor {
    pub fn new(anchor: TextPosition, cursor: TextPosition) -> Self {
        Self { anchor, cursor }
    }
}

/// Core for text editing.
#[derive(Debug)]
pub struct TextCore<Store> {
    /// Text store.
    text: Store,

    cursor: Rc<Cell<TextCursor>>,

    /// styles
    styles: Option<Box<RangeMap>>,
    /// undo-buffer
    undo: Option<Box<dyn UndoBuffer>>,
    /// clipboard
    clip: Option<Box<dyn Clipboard>>,
    /// cache
    cache: Cache,

    /// show ctrl chars in glyphs
    glyph_ctrl: bool,
    /// show text-wrap glyphs
    wrap_ctrl: bool,
}

impl<Store: Clone> Clone for TextCore<Store> {
    fn clone(&self) -> Self {
        Self {
            text: self.text.clone(),
            cursor: self.cursor.clone(),
            styles: self.styles.clone(),
            undo: self.undo.as_ref().map(|v| clone_box(v.as_ref())),
            clip: self.clip.as_ref().map(|v| clone_box(v.as_ref())),
            cache: Default::default(),
            glyph_ctrl: self.glyph_ctrl,
            wrap_ctrl: self.wrap_ctrl,
        }
    }
}

impl<Store: TextStore + Default> TextCore<Store> {
    pub fn new(undo: Option<Box<dyn UndoBuffer>>, clip: Option<Box<dyn Clipboard>>) -> Self {
        Self {
            text: Store::default(),
            cursor: Default::default(),
            styles: Default::default(),
            undo,
            clip,
            cache: Default::default(),
            glyph_ctrl: false,
            wrap_ctrl: false,
        }
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
                    self.cursor
                        .set(TextCursor::new(anchor.before, cursor.before));
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
                    self.cursor
                        .set(TextCursor::new(anchor.before, cursor.before));
                }
                UndoOp::Cursor { cursor, anchor } => {
                    self.cursor
                        .set(TextCursor::new(anchor.before, cursor.before));
                }
                UndoOp::SetStyles { styles_before, .. } => {
                    if let Some(sty) = &mut self.styles {
                        sty.set(styles_before.iter().cloned());
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
                    self.cursor.set(TextCursor::new(anchor.after, cursor.after));
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
                    self.cursor.set(TextCursor::new(anchor.after, cursor.after));
                }
                UndoOp::Cursor { cursor, anchor } => {
                    self.cursor.set(TextCursor::new(anchor.after, cursor.after));
                }

                UndoOp::SetStyles { styles_after, .. } => {
                    if let Some(sty) = &mut self.styles {
                        sty.set(styles_after.iter().cloned());
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
        sty.set(new_styles.into_iter());
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
    }

    /// Remove a style for the given byte-range.
    ///
    /// Range and style must match to be removed.
    #[inline]
    pub fn remove_style(&mut self, range: Range<usize>, style: usize) {
        if let Some(sty) = &mut self.styles {
            sty.remove(range.clone(), style);
        }
    }

    /// Remove all ranges for the given style.
    #[inline]
    pub fn remove_style_fully(&mut self, style: usize) {
        let Some(sty) = self.styles.as_mut() else {
            return;
        };
        let styles = sty
            .values()
            .filter(|(_, s)| *s == style)
            .collect::<Vec<_>>();
        for (range, style) in &styles {
            sty.remove(range.clone(), *style);
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

    /// Find all styles that touch the given range.
    #[inline]
    pub fn styles_in_match(
        &self,
        range: Range<usize>,
        style: usize,
        buf: &mut Vec<(Range<usize>, usize)>,
    ) {
        if let Some(sty) = &self.styles {
            sty.values_in_match(range, style, buf);
        }
    }

    /// Finds all styles for the given position.
    #[inline]
    pub fn styles_at(&self, byte_pos: usize, buf: &mut Vec<(Range<usize>, usize)>) {
        if let Some(sty) = &self.styles {
            sty.values_at(byte_pos, buf);
        }
    }

    /// Finds all styles for the given position.
    #[inline]
    pub fn styles_at_match(&self, byte_pos: usize, style: usize) -> Option<Range<usize>> {
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
    /// Shared cursor.
    pub(crate) fn shared_cursor(&self) -> Rc<Cell<TextCursor>> {
        self.cursor.clone()
    }

    /// Set the cursor position.
    /// The value is capped to the number of text lines and
    /// the line-width for the given line.
    ///
    /// Returns true, if the cursor actually changed.
    pub fn set_cursor(&mut self, cursor: TextPosition, extend_selection: bool) -> bool {
        let old_cursor = self.cursor.get();

        let cursor_pos = TextPosition::new(
            min(cursor.x, self.line_width(cursor.y).expect("valid-line")),
            min(cursor.y, self.len_lines()),
        );
        let new_cursor = TextCursor {
            cursor: cursor_pos,
            anchor: if !extend_selection {
                cursor_pos
            } else {
                old_cursor.anchor
            },
        };
        self.cursor.set(new_cursor);

        if let Some(undo) = self.undo.as_mut() {
            undo.append(UndoOp::Cursor {
                cursor: TextPositionChange {
                    before: old_cursor.cursor,
                    after: new_cursor.cursor,
                },
                anchor: TextPositionChange {
                    before: old_cursor.anchor,
                    after: new_cursor.anchor,
                },
            });
        }

        old_cursor != self.cursor.get()
    }

    /// Cursor position as grapheme-idx.
    #[inline]
    pub fn cursor(&self) -> TextPosition {
        self.cursor.get().cursor
    }

    /// Selection anchor
    #[inline]
    pub fn anchor(&self) -> TextPosition {
        self.cursor.get().anchor
    }

    /// Any text selection.
    #[inline]
    pub fn has_selection(&self) -> bool {
        let cursor = self.cursor.get();
        cursor.anchor != cursor.cursor
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

        self.set_cursor(TextPosition::new(0, self.len_lines()), false);
        self.set_cursor(TextPosition::new(0, 0), true);

        old_selection != self.selection()
    }

    /// Returns the selection as TextRange.
    #[inline]
    pub fn selection(&self) -> TextRange {
        let cursor = self.cursor.get();
        #[allow(clippy::comparison_chain)]
        if cursor.cursor.y < cursor.anchor.y {
            TextRange {
                start: cursor.cursor,
                end: cursor.anchor,
            }
        } else if cursor.cursor.y > cursor.anchor.y {
            TextRange {
                start: cursor.anchor,
                end: cursor.cursor,
            }
        } else {
            if cursor.cursor.x < cursor.anchor.x {
                TextRange {
                    start: cursor.cursor,
                    end: cursor.anchor,
                }
            } else {
                TextRange {
                    start: cursor.anchor,
                    end: cursor.cursor,
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
    pub(crate) fn cache_validity(&self) -> Option<usize> {
        self.text.cache_validity()
    }
}

impl<Store: TextStore + Default> TextCore<Store> {
    /// Empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len_bytes() == 0
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

    /// Limited access to the cache.
    /// Gives only access to Debug.
    #[inline]
    pub fn cache(&self) -> &Cache {
        &self.cache
    }

    /// Fill the cache for all the given rows completely.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn fill_cache(
        &self,
        rendered: Size,
        sub_row_offset: upos_type,
        rows: Range<upos_type>,
        tab_width: u32,
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
            tab_width,
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
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn glyphs2(
        &self,
        rendered: Size,
        sub_row_offset: upos_type,
        rows: Range<upos_type>,
        tab_width: u32,
        text_wrap: TextWrap2,
        ctrl_char: bool,
        left_margin: upos_type,
        right_margin: upos_type,
        word_margin: upos_type,
    ) -> Result<GlyphIter2<'_, Store::GraphemeIter<'_>>, TextError> {
        self.cache.validate(
            text_wrap,
            left_margin,
            rendered.width as upos_type,
            rendered.height as upos_type,
            ctrl_char,
            self.cache_validity(),
        );

        let range = TextRange::new((sub_row_offset, rows.start), (0, rows.end));

        let range_bytes;
        let mut range_to_bytes = self.cache.range_to_bytes.borrow_mut();
        if let Some(cache) = range_to_bytes.get(&range) {
            range_bytes = cache.clone();
        } else {
            let cache = self.text.byte_range(range)?;
            range_to_bytes.insert(range, cache.clone());
            range_bytes = cache;
        }

        let iter = self.graphemes_byte(range_bytes.clone(), range_bytes.start)?;

        let mut it = GlyphIter2::new(
            range.start, //
            range_bytes.start,
            iter,
            self.cache.clone(),
        );
        it.set_tabs(tab_width);
        it.set_show_ctrl(self.glyph_ctrl);
        it.set_wrap_ctrl(self.wrap_ctrl);
        it.set_lf_breaks(self.text().is_multi_line());
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
    pub fn text_graphemes(&self, pos: TextPosition) -> Result<Store::GraphemeIter<'_>, TextError> {
        let rows = self.len_lines() - 1;
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

    /// Get a cursor over the text-range with the current position set at pos.
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
    /// * row must be <= len_lines
    #[inline]
    pub fn line_at(&self, row: upos_type) -> Result<Cow<'_, str>, TextError> {
        self.text.line_at(row)
    }

    /// Iterate over text-lines, starting at row.
    ///
    /// * row must be <= len_lines
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
        self.cache.validate_byte_pos(self.cache_validity());

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
        self.text.len_lines()
    }

    /// Length in bytes.
    #[inline]
    pub fn len_bytes(&self) -> usize {
        self.text.len_bytes()
    }
}

impl<Store: TextStore + Default> TextCore<Store> {
    /// Clear the internal state.
    pub fn clear(&mut self) {
        self.text.set_string(Default::default());
        self.cursor.set(Default::default());
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

    /// Returns the TextStore.
    pub fn text(&self) -> &Store {
        &self.text
    }

    /// Set the text as a TextStore.
    /// Clears the styles, cursor and anchor.
    pub fn set_text(&mut self, t: Store) -> bool {
        self.text = t;
        if let Some(sty) = &mut self.styles {
            sty.clear();
        }
        self.cache.clear();
        self.cursor.set(Default::default());

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

    /// Insert a character.
    ///
    /// Has no special handling for '\n' and '\t' and just adds them
    /// as they are. '\n' *is* treated as line-break, but it might not be
    /// the correct byte-sequence for your platform.
    pub fn insert_char(&mut self, pos: TextPosition, c: char) -> Result<bool, TextError> {
        let (inserted_range, inserted_bytes) = self.text.insert_char(pos, c)?;

        let old_cursor = self.cursor.get();

        if let Some(sty) = &mut self.styles {
            sty.remap(|r, _| Some(expand_range_by(inserted_bytes.clone(), r)));
        }

        let new_cursor = TextCursor {
            anchor: inserted_range.expand_pos(old_cursor.cursor),
            cursor: inserted_range.expand_pos(old_cursor.anchor),
        };
        self.cursor.set(new_cursor);

        if let Some(undo) = self.undo.as_mut() {
            undo.append(UndoOp::InsertChar {
                bytes: inserted_bytes.clone(),
                cursor: TextPositionChange {
                    before: old_cursor.cursor,
                    after: new_cursor.cursor,
                },
                anchor: TextPositionChange {
                    before: old_cursor.anchor,
                    after: new_cursor.anchor,
                },
                txt: c.to_string(),
            });
        }

        Ok(true)
    }

    /// Insert a string at position.
    pub fn insert_str(&mut self, pos: TextPosition, t: &str) -> Result<bool, TextError> {
        let old_cursor = self.cursor.get();

        let (inserted_range, inserted_bytes) = self.text.insert_str(pos, t)?;

        if let Some(sty) = &mut self.styles {
            sty.remap(|r, _| Some(expand_range_by(inserted_bytes.clone(), r)));
        }
        let new_cursor = TextCursor {
            anchor: inserted_range.expand_pos(old_cursor.anchor),
            cursor: inserted_range.expand_pos(old_cursor.cursor),
        };
        self.cursor.set(new_cursor);

        if let Some(undo) = self.undo.as_mut() {
            undo.append(UndoOp::InsertStr {
                bytes: inserted_bytes.clone(),
                cursor: TextPositionChange {
                    before: old_cursor.cursor,
                    after: new_cursor.cursor,
                },
                anchor: TextPositionChange {
                    before: old_cursor.anchor,
                    after: new_cursor.anchor,
                },
                txt: t.to_string(),
            });
        }

        Ok(true)
    }

    /// Remove a range.
    ///
    /// Put it into undo as 'char-removed'. This can merge with other 'char-removed'
    /// undoes if they are next to each other.
    pub fn remove_char_range(&mut self, range: TextRange) -> Result<bool, TextError> {
        self._remove_range(range, true)
    }

    /// Remove a range
    ///
    /// Put it into undo as 'str-removed'. This will not be merged with other undoes.
    pub fn remove_str_range(&mut self, range: TextRange) -> Result<bool, TextError> {
        self._remove_range(range, false)
    }

    fn _remove_range(&mut self, range: TextRange, char_range: bool) -> Result<bool, TextError> {
        let old_cursor = self.cursor.get();

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
        let new_cursor = TextCursor {
            anchor: range.shrink_pos(old_cursor.anchor),
            cursor: range.shrink_pos(old_cursor.cursor),
        };
        self.cursor.set(new_cursor);

        if let Some(undo) = &mut self.undo {
            if char_range {
                undo.append(UndoOp::RemoveChar {
                    bytes: removed_bytes.clone(),
                    cursor: TextPositionChange {
                        before: old_cursor.cursor,
                        after: new_cursor.cursor,
                    },
                    anchor: TextPositionChange {
                        before: old_cursor.anchor,
                        after: new_cursor.anchor,
                    },
                    txt: old_text,
                    styles: changed_style,
                });
            } else {
                undo.append(UndoOp::RemoveStr {
                    bytes: removed_bytes.clone(),
                    cursor: TextPositionChange {
                        before: old_cursor.cursor,
                        after: new_cursor.cursor,
                    },
                    anchor: TextPositionChange {
                        before: old_cursor.anchor,
                        after: new_cursor.anchor,
                    },
                    txt: old_text,
                    styles: changed_style,
                });
            }
        }

        Ok(true)
    }
}
