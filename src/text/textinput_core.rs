use crate::text::graphemes::{split3, split_at, str_line_len, GlyphIter};
#[allow(unused_imports)]
use log::debug;
use std::cmp::min;
use std::iter::once;
use std::mem;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

/// Text editing core.
#[derive(Debug, Default, Clone)]
pub struct TextInputCore {
    // Text
    value: String,
    // Len in grapheme count.
    len: usize,

    // cursor and selection
    cursor: usize,
    anchor: usize,

    // Temporary space for editing.
    buf: String,
}

impl TextInputCore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Cursor position as grapheme-idx. Moves the cursor to the new position,
    /// but can leave the current cursor position as anchor of the selection.
    // xxx
    #[inline]
    pub fn set_cursor(&mut self, cursor: usize, extend_selection: bool) -> bool {
        let old_selection = (self.cursor, self.anchor);

        let c = min(self.len, cursor);
        self.cursor = c;
        if !extend_selection {
            self.anchor = c;
        }

        (self.cursor, self.anchor) != old_selection
    }

    /// Set the cursor and anchor to the defaults.
    /// Exists just to mirror MaskedInput.
    #[inline]
    pub fn set_default_cursor(&mut self) {
        self.cursor = 0;
        self.anchor = 0;
    }

    /// Cursor position as grapheme-idx.
    // xxx
    #[inline]
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Selection anchor
    // xxx
    #[inline]
    pub fn anchor(&self) -> usize {
        self.anchor
    }

    /// Set the value. Resets cursor and anchor to 0.
    // xxx
    #[inline]
    pub fn set_value<S: Into<String>>(&mut self, s: S) {
        self.value = s.into();
        self.len = self.value.graphemes(true).count();
        self.cursor = 0;
        self.anchor = 0;
    }

    /// Create a default value according to the mask.
    /// Exists just to mirror MaskedInput.
    #[inline]
    pub fn default_value(&self) -> String {
        String::new()
    }

    /// Value
    // xxx
    #[inline]
    pub fn value(&self) -> &str {
        self.value.as_str()
    }

    /// Value as glyph iterator.
    // xxx
    #[inline]
    pub fn value_glyphs(&self) -> GlyphIter<'_> {
        GlyphIter::new(self.value())
    }

    /// Reset value to an empty default.
    /// Resets offset and cursor position too.
    // xxx
    #[inline]
    pub fn clear(&mut self) {
        self.set_value(self.default_value());
        self.set_default_cursor();
    }

    /// Is equal to the default value.
    // xxx
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Value length as grapheme-count
    // xxx
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Is there a selection.
    // xxx
    #[inline]
    pub fn has_selection(&self) -> bool {
        self.anchor != self.cursor
    }

    /// Selection.
    // xxx
    #[inline]
    pub fn set_selection(&mut self, anchor: usize, cursor: usize) -> bool {
        let old_selection = self.selection();

        self.set_cursor(anchor, false);
        self.set_cursor(cursor, true);

        old_selection != self.selection()
    }

    /// Selection.
    // xxx
    #[inline]
    pub fn select_all(&mut self) -> bool {
        let old_selection = self.selection();

        self.set_cursor(0, false);
        self.set_cursor(self.len(), true);

        old_selection != self.selection()
    }

    /// Selection.
    // xxx
    #[inline]
    pub fn selection(&self) -> Range<usize> {
        if self.cursor < self.anchor {
            self.cursor..self.anchor
        } else {
            self.anchor..self.cursor
        }
    }

    /// Char position to grapheme position.
    pub fn char_pos(&self, char_pos: usize) -> Option<usize> {
        let mut cp = 0;
        for (gp, (_bp, cc)) in self
            .value
            .grapheme_indices(true)
            .chain(once((self.len(), "")))
            .enumerate()
        {
            if cp >= char_pos {
                return Some(gp);
            }
            cp += cc.chars().count();
        }

        None
    }

    /// Convert the byte-position to a grapheme position.
    // xxx
    pub fn byte_pos(&self, byte_pos: usize) -> Option<usize> {
        let mut pos = None;

        for (gp, (bp, _cc)) in self
            .value
            .grapheme_indices(true)
            .chain(once((self.len(), "")))
            .enumerate()
        {
            if bp >= byte_pos {
                pos = Some(gp);
                break;
            }
        }

        pos
    }

    /// Grapheme position to byte position.
    /// Returns the byte-range for the grapheme at pos.
    pub fn byte_at(&self, pos: usize) -> Option<(usize, usize)> {
        let mut byte_pos = None;

        for (gp, (bp, cc)) in self
            .value
            .grapheme_indices(true)
            .chain(once((self.value.len(), "")))
            .enumerate()
        {
            if gp == pos {
                byte_pos = Some((bp, bp + cc.len()));
                break;
            }
        }

        byte_pos
    }

    /// Grapheme position to char position.
    /// Returns the first char position for the grapheme at pos.
    pub fn char_at(&self, pos: usize) -> Option<usize> {
        let mut char_pos = 0;
        for (gp, (_bp, cc)) in self
            .value
            .grapheme_indices(true)
            .chain(once((self.len(), "")))
            .enumerate()
        {
            if gp == pos {
                return Some(char_pos);
            }
            char_pos += cc.chars().count();
        }

        None
    }
}

impl TextInputCore {
    /// Insert a char at the position.
    // xxx
    pub fn insert_char(&mut self, pos: usize, new: char) -> bool {
        let old_len = self.len;
        let (before, after) = split_at(&self.value, pos);
        self.buf.clear();
        self.buf.push_str(before);
        self.buf.push(new);
        self.buf.push_str(after);
        let new_len = str_line_len(&self.buf);

        mem::swap(&mut self.value, &mut self.buf);

        self.len = new_len;

        if self.cursor >= pos {
            self.cursor += new_len - old_len;
        }
        if self.anchor >= pos {
            self.anchor += new_len - old_len;
        }

        true
    }

    /// Insert a char at the position.
    // xx
    pub fn insert_str(&mut self, pos: usize, new: &str) -> bool {
        let (before, after) = split_at(&self.value, pos);

        let old_len = self.len;
        self.buf.clear();
        self.buf.push_str(before);
        self.buf.push_str(new);
        self.buf.push_str(after);
        let new_len = str_line_len(&self.buf);

        mem::swap(&mut self.value, &mut self.buf);

        self.len = new_len;

        if self.cursor >= pos {
            self.cursor += new_len - old_len;
        }
        if self.anchor >= pos {
            self.anchor += new_len - old_len;
        }

        true
    }

    /// Remove the range.
    // xxx
    pub fn remove_range(&mut self, range: Range<usize>) -> bool {
        let (before_str, _remove_str, after_str) = split3(self.value.as_str(), range.clone());

        let old_len = self.len;
        self.buf.clear();
        self.buf.push_str(before_str);
        self.buf.push_str(after_str);
        let new_len = str_line_len(&self.buf);

        mem::swap(&mut self.value, &mut self.buf);

        self.len = new_len;

        if self.cursor < range.start {
            // noop
        } else if self.cursor <= range.end {
            self.cursor = range.start;
        } else {
            self.cursor = self.cursor.saturating_sub(old_len - new_len);
        }

        if self.anchor < range.start {
            // noop
        } else if self.anchor <= range.end {
            self.anchor = range.start;
        } else {
            self.anchor = self.anchor.saturating_sub(old_len - new_len);
        }

        true
    }
}
