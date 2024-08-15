use crate::clipboard::{Clipboard, LocalClipboard};
use crate::event::TextOutcome;
use crate::grapheme::{Glyph, GlyphIter, Grapheme};
use crate::range_map::{expand_range_by, ranges_intersect, shrink_range_by, RangeMap};
use crate::text_store::TextStore;
use crate::undo_buffer::{StyleChange, TextPositionChange, UndoBuffer, UndoEntry, UndoVec};
use crate::{upos_type, TextError, TextPosition, TextRange};
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
    styles: RangeMap,
    /// undo-buffer
    undo: Option<Box<dyn UndoBuffer>>,
    /// clipboard
    clip: Option<Box<dyn Clipboard>>,

    /// line-break
    newline: String,
    /// tab-width
    tabs: upos_type,
    /// expand tabs
    expand_tabs: bool,
    /// show ctrl chars
    show_ctrl: bool,
    /// movement column
    move_col: Option<upos_type>,
}

impl<Store: Default> Default for TextCore<Store> {
    fn default() -> Self {
        Self {
            text: Store::default(),
            cursor: Default::default(),
            anchor: Default::default(),
            styles: Default::default(),
            undo: Some(Box::new(UndoVec::new(40))),
            clip: Some(Box::new(LocalClipboard::default())),
            newline: "\n".to_string(),
            tabs: 8,
            expand_tabs: true,
            show_ctrl: false,
            move_col: None,
        }
    }
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
            show_ctrl: self.show_ctrl,
            move_col: self.move_col,
        }
    }
}

impl<Store: TextStore + Default> TextCore<Store> {
    pub fn new() -> Self {
        Self::default()
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
    pub fn set_tab_width(&mut self, tabs: upos_type) {
        self.tabs = tabs;
    }

    /// Tab-width
    #[inline]
    pub fn tab_width(&self) -> upos_type {
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

    /// Show control characters.
    #[inline]
    pub fn set_show_ctrl(&mut self, show_ctrl: bool) {
        self.show_ctrl = show_ctrl;
    }

    /// Show control characters.
    pub fn show_ctrl(&self) -> bool {
        self.show_ctrl
    }
}

impl<Store: TextStore + Default> TextCore<Store> {
    /// Undo
    #[inline]
    pub fn set_undo_buffer(&mut self, undo: Box<dyn UndoBuffer>) {
        self.undo = Some(undo);
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
    pub fn undo(&mut self) -> Result<TextOutcome, TextError> {
        let Some(undo) = self.undo.as_mut() else {
            return Ok(TextOutcome::Continue);
        };

        undo.append(UndoEntry::Undo);

        self._undo()
    }

    /// Undo last.
    fn _undo(&mut self) -> Result<TextOutcome, TextError> {
        let Some(undo) = self.undo.as_mut() else {
            return Ok(TextOutcome::Continue);
        };
        let op = undo.undo();
        match op {
            Some(UndoEntry::InsertChar {
                bytes,
                cursor,
                anchor,
                ..
            })
            | Some(UndoEntry::InsertStr {
                bytes,
                cursor,
                anchor,
                ..
            }) => {
                self.text.remove_b(bytes.clone())?;

                self.styles
                    .remap(|r, _| Some(shrink_range_by(bytes.clone(), r)));
                self.anchor = anchor.before;
                self.cursor = cursor.before;

                Ok(TextOutcome::TextChanged)
            }
            Some(UndoEntry::RemoveStr {
                bytes,
                cursor,
                anchor,
                txt,
                styles,
            })
            | Some(UndoEntry::RemoveChar {
                bytes,
                cursor,
                anchor,
                txt,
                styles,
            }) => {
                self.text.insert_str_b(bytes.start, &txt)?;

                for s in &styles {
                    self.styles.remove(s.after.clone(), s.style);
                }
                for s in &styles {
                    self.styles.add(s.before.clone(), s.style);
                }
                self.styles.remap(|r, _| {
                    if ranges_intersect(bytes.clone(), r.clone()) {
                        Some(r)
                    } else {
                        Some(expand_range_by(bytes.clone(), r))
                    }
                });
                self.anchor = anchor.before;
                self.cursor = cursor.before;

                Ok(TextOutcome::TextChanged)
            }
            Some(UndoEntry::SetStyles { styles_before, .. }) => {
                self.styles.set(styles_before.iter().cloned());
                Ok(TextOutcome::Changed)
            }
            Some(UndoEntry::AddStyle { range, style }) => {
                self.styles.remove(range, style);
                Ok(TextOutcome::Changed)
            }
            Some(UndoEntry::RemoveStyle { range, style }) => {
                self.styles.add(range, style);
                Ok(TextOutcome::Changed)
            }
            Some(UndoEntry::SetText { .. }) => Ok(TextOutcome::Unchanged),
            Some(UndoEntry::Undo) => Ok(TextOutcome::Unchanged),
            Some(UndoEntry::Redo) => Ok(TextOutcome::Unchanged),
            None => Ok(TextOutcome::Continue),
        }
    }

    /// Redo last.
    pub fn redo(&mut self) -> Result<TextOutcome, TextError> {
        let Some(undo) = self.undo.as_mut() else {
            return Ok(TextOutcome::Continue);
        };

        undo.append(UndoEntry::Redo);

        self._redo()
    }

    fn _redo(&mut self) -> Result<TextOutcome, TextError> {
        let Some(undo) = self.undo.as_mut() else {
            return Ok(TextOutcome::Continue);
        };
        let op = undo.redo();
        match op {
            Some(UndoEntry::InsertChar {
                bytes,
                cursor,
                anchor,
                txt,
            })
            | Some(UndoEntry::InsertStr {
                bytes,
                cursor,
                anchor,
                txt,
            }) => {
                self.text.insert_str_b(bytes.start, &txt)?;
                self.styles
                    .remap(|r, _| Some(expand_range_by(bytes.clone(), r)));
                self.anchor = anchor.after;
                self.cursor = cursor.after;

                Ok(TextOutcome::TextChanged)
            }
            Some(UndoEntry::RemoveChar {
                bytes,
                cursor,
                anchor,
                styles,
                ..
            })
            | Some(UndoEntry::RemoveStr {
                bytes,
                cursor,
                anchor,
                styles,
                ..
            }) => {
                self.text.remove_b(bytes.clone())?;

                self.styles.remap(|r, _| {
                    if ranges_intersect(bytes.clone(), r.clone()) {
                        Some(r)
                    } else {
                        Some(shrink_range_by(bytes.clone(), r))
                    }
                });
                for s in &styles {
                    self.styles.remove(s.before.clone(), s.style);
                }
                for s in &styles {
                    self.styles.add(s.after.clone(), s.style);
                }

                self.anchor = anchor.after;
                self.cursor = cursor.after;

                Ok(TextOutcome::TextChanged)
            }

            Some(UndoEntry::SetStyles { styles_after, .. }) => {
                self.styles.set(styles_after.iter().cloned());
                Ok(TextOutcome::Changed)
            }
            Some(UndoEntry::AddStyle { range, style }) => {
                self.styles.add(range, style);
                Ok(TextOutcome::Changed)
            }
            Some(UndoEntry::RemoveStyle { range, style }) => {
                self.styles.remove(range, style);
                Ok(TextOutcome::Changed)
            }
            Some(UndoEntry::SetText { .. }) => Ok(TextOutcome::Unchanged),
            Some(UndoEntry::Undo) => Ok(TextOutcome::Unchanged),
            Some(UndoEntry::Redo) => Ok(TextOutcome::Unchanged),
            None => Ok(TextOutcome::Continue),
        }
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
    pub fn replay_log(&mut self, replay: &[UndoEntry]) -> Result<(), TextError> {
        for replay_entry in replay {
            match replay_entry {
                UndoEntry::SetText { txt } => {
                    self.text.set_string(txt);
                    self.styles.clear();
                    if let Some(undo) = self.undo.as_mut() {
                        undo.clear();
                    };
                }
                UndoEntry::InsertChar { bytes, txt, .. }
                | UndoEntry::InsertStr { bytes, txt, .. } => {
                    self.text.insert_str_b(bytes.start, txt)?;
                    self.styles
                        .remap(|r, _| Some(expand_range_by(bytes.clone(), r)));
                }
                UndoEntry::RemoveChar { bytes, styles, .. }
                | UndoEntry::RemoveStr { bytes, styles, .. } => {
                    self.text.remove_b(bytes.clone())?;
                    self.styles.remap(|r, _| {
                        if ranges_intersect(bytes.clone(), r.clone()) {
                            Some(r)
                        } else {
                            Some(shrink_range_by(bytes.clone(), r))
                        }
                    });
                    for s in styles {
                        self.styles.remove(s.before.clone(), s.style);
                    }
                    for s in styles {
                        self.styles.add(s.after.clone(), s.style);
                    }
                }
                UndoEntry::SetStyles { styles_after, .. } => {
                    self.styles.set(styles_after.iter().cloned());
                }
                UndoEntry::AddStyle { range, style } => {
                    self.styles.add(range.clone(), *style);
                }
                UndoEntry::RemoveStyle { range, style } => {
                    self.styles.remove(range.clone(), *style);
                }
                UndoEntry::Undo => {
                    self._undo()?;
                }
                UndoEntry::Redo => {
                    self._redo()?;
                }
            }

            if let Some(undo) = self.undo.as_mut() {
                undo.append_no_replay(replay_entry.clone());
            };
        }
        Ok(())
    }
}

impl<Store: TextStore + Default> TextCore<Store> {
    /// Set all styles.
    ///
    /// The ranges are byte-ranges. The usize value is the index of the
    /// actual style. Those are set with the widget.
    #[inline]
    pub fn set_styles(&mut self, styles: Vec<(Range<usize>, usize)>) {
        if let Some(undo) = &mut self.undo {
            if undo.undo_styles_enabled() || undo.replay_log() {
                undo.append(UndoEntry::SetStyles {
                    styles_before: self.styles.values().collect::<Vec<_>>(),
                    styles_after: styles.clone(),
                });
            }
        }
        self.styles.set(styles.iter().cloned());
    }

    /// Add a style for the given byte-range.
    ///
    /// The usize value is the index of the actual style.
    /// Those are set at the widget.
    #[inline]
    pub fn add_style(&mut self, range: Range<usize>, style: usize) {
        self.styles.add(range.clone(), style);

        if let Some(undo) = &mut self.undo {
            if undo.undo_styles_enabled() || undo.replay_log() {
                undo.append(UndoEntry::AddStyle { range, style });
            }
        }
    }

    /// Remove a style for the given byte-range.
    ///
    /// Range and style must match to be removed.
    #[inline]
    pub fn remove_style(&mut self, range: Range<usize>, style: usize) {
        self.styles.remove(range.clone(), style);

        if let Some(undo) = &mut self.undo {
            if undo.undo_styles_enabled() || undo.replay_log() {
                undo.append(UndoEntry::RemoveStyle { range, style });
            }
        }
    }

    /// Finds all styles for the given position.
    #[inline]
    pub fn styles_at(&self, byte_pos: usize, buf: &mut Vec<usize>) {
        self.styles.values_at(byte_pos, buf)
    }

    /// Check if the given style applies at the position and
    /// return the complete range for the style.
    #[inline]
    pub fn style_match(&self, byte_pos: usize, style: usize) -> Option<Range<usize>> {
        self.styles.value_match(byte_pos, style)
    }

    /// List of all styles.
    #[inline]
    pub fn styles(&self) -> impl Iterator<Item = (Range<usize>, usize)> + '_ {
        self.styles.values()
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
        self.text.is_empty()
    }

    /// Line as RopeSlice
    #[inline]
    pub fn line_at(&self, row: upos_type) -> Option<Cow<'_, str>> {
        self.text.line_at(row)
    }

    /// Iterate over text-lines, starting at offset.
    #[inline]
    pub fn lines_at(&self, row: upos_type) -> impl Iterator<Item = Cow<'_, str>> {
        self.text.lines_at(row)
    }

    /// Iterator for the glyphs of a given line.
    /// Glyphs here a grapheme + display length.
    #[inline]
    pub fn line_glyphs(
        &self,
        row: upos_type,
        col_offset: upos_type,
    ) -> Option<impl Iterator<Item = Glyph<'_>>> {
        let iter = self.line_graphemes(row)?;

        let mut it = GlyphIter::new(iter);
        it.set_offset(col_offset as usize);
        it.set_tabs(self.tabs as u16);
        it.set_show_ctrl(self.show_ctrl);
        Some(it)
    }

    /// Get the text for a line as iterator over the graphemes.
    #[inline]
    pub fn line_graphemes(&self, row: upos_type) -> Option<impl Iterator<Item = Grapheme<'_>>> {
        self.text.line_graphemes(row)
    }

    /// Line width as grapheme count. Excludes the terminating '\n'.
    pub fn line_width(&self, row: upos_type) -> Option<upos_type> {
        self.text.line_width(row)
    }

    /// Number of lines.
    pub fn len_lines(&self) -> upos_type {
        self.text.len_lines()
    }

    /// Copy of the text-value.
    pub fn text(&self) -> String {
        self.text.string()
    }

    /// Set the text as a TextStore
    /// Clears the styles.
    /// Caps cursor and anchor.
    pub fn set_text(&mut self, t: Store) -> Result<bool, TextError> {
        self.text = t;
        self.styles.clear();

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

            if undo.replay_log() {
                undo.append(UndoEntry::SetText {
                    txt: self.text.string(),
                });
            }
        }

        Ok(true)
    }

    /// Insert a tab, either expanded or literally.
    pub fn insert_tab(&mut self, mut pos: TextPosition) -> Result<bool, TextError> {
        if self.expand_tabs {
            let n = self.tabs - pos.x % self.tabs;
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
        let inserted = self.text.insert_char(pos, c)?;

        let old_cursor = self.cursor;
        let old_anchor = self.anchor;

        self.styles
            .remap(|r, _| Some(expand_range_by((&inserted.bytes).clone(), r)));
        self.cursor = inserted.range.expand_pos(self.cursor);
        self.anchor = inserted.range.expand_pos(self.anchor);

        if let Some(undo) = self.undo.as_mut() {
            undo.append(UndoEntry::InsertChar {
                bytes: inserted.bytes.clone(),
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

        let inserted = self.text.insert_str(pos, t)?;

        self.styles
            .remap(|r, _| Some(expand_range_by((&inserted.bytes).clone(), r)));
        self.anchor = inserted.range.expand_pos(self.anchor);
        self.cursor = inserted.range.expand_pos(self.cursor);

        if let Some(undo) = self.undo.as_mut() {
            undo.append(UndoEntry::InsertStr {
                bytes: inserted.bytes.clone(),
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
        let Some(c_line_width) = self.line_width(pos.y) else {
            return Err(TextError::LineIndexOutOfBounds(pos.y, self.len_lines()));
        };
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

        let (old_text, removed) = self.text.remove(range)?;

        // remove deleted styles.
        let mut changed_style = Vec::new();
        self.styles.remap(|r, s| {
            let new_range = shrink_range_by(removed.bytes.clone(), r.clone());
            if ranges_intersect(r.clone(), removed.bytes.clone()) {
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
        self.anchor = range.shrink_pos(self.anchor);
        self.cursor = range.shrink_pos(self.anchor);

        if let Some(undo) = &mut self.undo {
            if char_range {
                undo.append(UndoEntry::RemoveChar {
                    bytes: removed.bytes.clone(),
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
                    bytes: removed.bytes.clone(),
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
