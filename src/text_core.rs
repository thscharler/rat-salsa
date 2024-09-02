use crate::clipboard::Clipboard;
use crate::grapheme::{Glyph, GlyphIter, Grapheme};
use crate::range_map::{expand_range_by, ranges_intersect, shrink_range_by, RangeMap};
use crate::text_store::TextStore;
use crate::undo_buffer::{StyleChange, TextPositionChange, UndoBuffer, UndoEntry};
use crate::{upos_type, Cursor, TextError, TextPosition, TextRange};
use dyn_clone::clone_box;
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

    /// line-break
    newline: String,
    /// tab-width
    tabs: u16,
    /// expand tabs
    expand_tabs: bool,
    /// show ctrl chars in glyphs
    glyph_ctrl: bool,
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
            newline: self.newline.clone(),
            tabs: self.tabs,
            expand_tabs: self.expand_tabs,
            glyph_ctrl: self.glyph_ctrl,
            glyph_line_break: self.glyph_line_break,
        }
    }
}

impl<Store: TextStore + Default> TextCore<Store> {
    pub fn new(undo: Option<Box<dyn UndoBuffer>>, clip: Option<Box<dyn Clipboard>>) -> Self {
        Self {
            text: Store::default(),
            cursor: Default::default(),
            anchor: Default::default(),
            styles: Default::default(),
            undo,
            clip,
            newline: "\n".to_string(),
            tabs: 8,
            expand_tabs: true,
            glyph_ctrl: false,
            glyph_line_break: true,
        }
    }

    /// Sets the line ending to be used for insert.
    /// There is no auto-detection or conversion done for set_value().
    ///
    /// Caution: If this doesn't match the line ending used in the value, you
    /// will get a value with mixed line endings.
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

    /// Show control characters when iterating glyphs.
    pub fn glyph_ctrl(&self) -> bool {
        self.glyph_ctrl
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

        undo.append(UndoEntry::Undo);

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
                UndoEntry::InsertChar {
                    bytes,
                    cursor,
                    anchor,
                    ..
                }
                | UndoEntry::InsertStr {
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
                UndoEntry::RemoveStr {
                    bytes,
                    cursor,
                    anchor,
                    txt,
                    styles,
                }
                | UndoEntry::RemoveChar {
                    bytes,
                    cursor,
                    anchor,
                    txt,
                    styles,
                } => {
                    self.text.insert_b(bytes.start, &txt).expect("valid_bytes");

                    if let Some(sty) = &mut self.styles {
                        for s in &styles {
                            sty.remove(s.after.clone(), s.style);
                        }
                        for s in &styles {
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
                UndoEntry::Cursor { cursor, anchor } => {
                    self.anchor = anchor.before;
                    self.cursor = cursor.before;
                }
                UndoEntry::SetStyles { styles_before, .. } => {
                    if let Some(sty) = &mut self.styles {
                        sty.set(styles_before.iter().cloned());
                    }
                }
                UndoEntry::AddStyle { range, style } => {
                    if let Some(sty) = &mut self.styles {
                        sty.remove(range, style);
                    }
                }
                UndoEntry::RemoveStyle { range, style } => {
                    if let Some(sty) = &mut self.styles {
                        sty.add(range, style);
                    }
                }
                UndoEntry::SetText { .. } | UndoEntry::Undo | UndoEntry::Redo => {
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

        undo.append(UndoEntry::Redo);

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
                UndoEntry::InsertChar {
                    bytes,
                    cursor,
                    anchor,
                    txt,
                }
                | UndoEntry::InsertStr {
                    bytes,
                    cursor,
                    anchor,
                    txt,
                } => {
                    self.text.insert_b(bytes.start, &txt).expect("valid_bytes");
                    if let Some(sty) = &mut self.styles {
                        sty.remap(|r, _| Some(expand_range_by(bytes.clone(), r)));
                    }
                    self.anchor = anchor.after;
                    self.cursor = cursor.after;
                }
                UndoEntry::RemoveChar {
                    bytes,
                    cursor,
                    anchor,
                    styles,
                    ..
                }
                | UndoEntry::RemoveStr {
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
                        for s in &styles {
                            sty.remove(s.before.clone(), s.style);
                        }
                        for s in &styles {
                            sty.add(s.after.clone(), s.style);
                        }
                    }

                    self.anchor = anchor.after;
                    self.cursor = cursor.after;
                }
                UndoEntry::Cursor { cursor, anchor } => {
                    self.anchor = anchor.after;
                    self.cursor = cursor.after;
                }

                UndoEntry::SetStyles { styles_after, .. } => {
                    if let Some(sty) = &mut self.styles {
                        sty.set(styles_after.iter().cloned());
                    }
                }
                UndoEntry::AddStyle { range, style } => {
                    if let Some(sty) = &mut self.styles {
                        sty.add(range, style);
                    }
                }
                UndoEntry::RemoveStyle { range, style } => {
                    if let Some(sty) = &mut self.styles {
                        sty.remove(range, style);
                    }
                }
                UndoEntry::SetText { .. } | UndoEntry::Undo | UndoEntry::Redo => {
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
            match replay_entry {
                UndoEntry::SetText { txt } => {
                    self.text.set_string(txt);
                    if let Some(sty) = &mut self.styles {
                        sty.clear();
                    }
                    if let Some(undo) = self.undo.as_mut() {
                        undo.clear();
                    };
                }
                UndoEntry::InsertChar { bytes, txt, .. }
                | UndoEntry::InsertStr { bytes, txt, .. } => {
                    self.text.insert_b(bytes.start, txt).expect("valid_range");
                    if let Some(sty) = &mut self.styles {
                        sty.remap(|r, _| Some(expand_range_by(bytes.clone(), r)));
                    }
                }
                UndoEntry::RemoveChar { bytes, styles, .. }
                | UndoEntry::RemoveStr { bytes, styles, .. } => {
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
                UndoEntry::Cursor { .. } => {
                    // don't do cursor
                }

                UndoEntry::SetStyles { styles_after, .. } => {
                    self.init_styles();
                    if let Some(sty) = &mut self.styles {
                        sty.set(styles_after.iter().cloned());
                    }
                }
                UndoEntry::AddStyle { range, style } => {
                    self.init_styles();
                    if let Some(sty) = &mut self.styles {
                        sty.add(range.clone(), *style);
                    }
                }
                UndoEntry::RemoveStyle { range, style } => {
                    self.init_styles();
                    if let Some(sty) = &mut self.styles {
                        sty.remove(range.clone(), *style);
                    }
                }
                UndoEntry::Undo => {
                    self._undo();
                }
                UndoEntry::Redo => {
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
                undo.append(UndoEntry::SetStyles {
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
                undo.append(UndoEntry::AddStyle { range, style });
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
                undo.append(UndoEntry::RemoveStyle { range, style });
            }
        }
    }

    /// Find all values for the given position.
    ///
    /// Creates a cache for the styles in range.
    #[inline]
    pub(crate) fn styles_at_page(&self, range: Range<usize>, pos: usize, buf: &mut Vec<usize>) {
        if let Some(sty) = &self.styles {
            sty.values_at_page(range, pos, buf);
        }
    }

    /// Find all styles that touch the given range.
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
        if let Some(sty) = &self.styles {
            Some(sty.values())
        } else {
            None
        }
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

        cursor.y = min(cursor.y, self.len_lines().saturating_sub(1));
        cursor.x = min(cursor.x, self.line_width(cursor.y).expect("valid-line"));

        self.cursor = cursor;
        if !extend_selection {
            self.anchor = cursor;
        }

        if let Some(undo) = self.undo.as_mut() {
            undo.append(UndoEntry::Cursor {
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

        self.set_cursor(TextPosition::new(0, 0), false);
        let last = self.len_lines().saturating_sub(1);
        let last_width = self.line_width(last).expect("valid_line");
        self.set_cursor(TextPosition::new(last_width, last), true);

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
    /// Empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.text.len_lines() == 1 && self.text.line_width(0).expect("line") == 0
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

    /// A range of the text as Cow<str>
    #[inline]
    pub fn str_slice(&self, range: TextRange) -> Result<Cow<'_, str>, TextError> {
        self.text.str_slice(range)
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

    /// Get the grapheme at the given position.
    #[inline]
    pub fn grapheme_at(&self, pos: TextPosition) -> Result<Option<Grapheme<'_>>, TextError> {
        let mut it = self
            .text
            .graphemes(TextRange::new(pos, (pos.x + 1, pos.y)), pos)?;
        Ok(it.next())
    }

    /// Get a cursor over all the text with the current position set at pos.
    #[inline]
    pub fn text_graphemes(
        &self,
        pos: TextPosition,
    ) -> Result<impl Iterator<Item = Grapheme<'_>> + Cursor, TextError> {
        let rows = self.text.len_lines();
        let cols = self.text.line_width(rows).expect("valid_row");
        self.text
            .graphemes(TextRange::new((0, 0), (cols, rows)), pos)
    }

    /// Get a cursor over the text-range the current position set at pos.
    #[inline]
    pub fn graphemes(
        &self,
        range: TextRange,
        pos: TextPosition,
    ) -> Result<impl Iterator<Item = Grapheme<'_>> + Cursor, TextError> {
        self.text.graphemes(range, pos)
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
    pub fn line_graphemes(
        &self,
        row: upos_type,
    ) -> Result<impl Iterator<Item = Grapheme<'_>> + Cursor, TextError> {
        self.text.line_graphemes(row)
    }

    /// Line width as grapheme count. Excludes the terminating '\n'.
    #[inline]
    pub fn line_width(&self, row: upos_type) -> Result<upos_type, TextError> {
        self.text.line_width(row)
    }

    /// Number of lines.
    #[inline]
    pub fn len_lines(&self) -> upos_type {
        self.text.len_lines()
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
                undo.append(UndoEntry::SetText {
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

        self.cursor.y = min(self.cursor.y, self.len_lines().saturating_sub(1));
        self.cursor.x = min(
            self.cursor.x,
            self.line_width(self.cursor.y).expect("valid_line"),
        );
        self.anchor.y = min(self.anchor.y, self.len_lines().saturating_sub(1));
        self.anchor.x = min(
            self.anchor.x,
            self.line_width(self.anchor.y).expect("valid_line"),
        );

        if let Some(undo) = &mut self.undo {
            undo.clear();

            if undo.has_replay_log() {
                undo.append(UndoEntry::SetText {
                    txt: self.text.string(),
                });
            }
        }

        true
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
            sty.remap(|r, _| Some(expand_range_by((&inserted_bytes).clone(), r)));
        }
        self.cursor = inserted_range.expand_pos(self.cursor);
        self.anchor = inserted_range.expand_pos(self.anchor);

        if let Some(undo) = self.undo.as_mut() {
            undo.append(UndoEntry::InsertChar {
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
            sty.remap(|r, _| Some(expand_range_by((&inserted_bytes).clone(), r)));
        }
        self.anchor = inserted_range.expand_pos(self.anchor);
        self.cursor = inserted_range.expand_pos(self.cursor);

        if let Some(undo) = self.undo.as_mut() {
            undo.append(UndoEntry::InsertStr {
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
        } else if pos.y != 0 && pos.x == 0 {
            let prev_line_width = self.line_width(pos.y - 1).expect("line_width");
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
                undo.append(UndoEntry::RemoveChar {
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
                undo.append(UndoEntry::RemoveStr {
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
        let pos = pos.into();

        let mut cursor = self.text_graphemes(pos)?;
        let mut last_pos = cursor.text_offset();
        loop {
            let Some(c) = cursor.next() else {
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
        let pos = pos.into();

        let mut cursor = self.text_graphemes(pos)?;
        let mut last_pos = cursor.text_offset();
        let mut init = true;
        loop {
            let Some(c) = cursor.next() else {
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
        }

        Ok(self.byte_pos(last_pos).expect("valid_pos"))
    }

    /// Find the start of the prev word. Skips whitespace first, then goes on
    /// until it finds the next whitespace.
    ///
    /// Attention: start/end are mirrored here compared to next_word_start/next_word_end,
    /// both return start<=end!
    pub fn prev_word_start(&self, pos: TextPosition) -> Result<TextPosition, TextError> {
        let pos = pos.into();

        let mut cursor = self.text_graphemes(pos)?;
        let mut last_pos = cursor.text_offset();
        let mut init = true;
        loop {
            let Some(c) = cursor.prev() else {
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
        let pos = pos.into();

        let mut cursor = self.text_graphemes(pos)?;
        let mut last_pos = cursor.text_offset();
        loop {
            let Some(c) = cursor.prev() else {
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
        let pos = pos.into();

        let mut cursor = self.text_graphemes(pos)?;
        if let Some(c0) = cursor.prev() {
            cursor.next();
            if let Some(c1) = cursor.next() {
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
        let pos = pos.into();

        let mut cursor = self.text_graphemes(pos)?;
        let mut last_pos = cursor.text_offset();
        loop {
            let Some(c) = cursor.prev() else {
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
        let pos = pos.into();

        let mut cursor = self.text_graphemes(pos)?;
        let mut last_pos = cursor.text_offset();
        loop {
            let Some(c) = cursor.next() else {
                break;
            };
            last_pos = c.text_bytes().start;
            if c.is_whitespace() {
                break;
            }
        }

        Ok(self.byte_pos(last_pos).expect("valid_pos"))
    }
}
