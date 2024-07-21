use crate::text::graphemes::str_line_len;
use crate::util;
use crate::util::split_at;
#[allow(unused_imports)]
use log::debug;
use std::cmp::min;
use std::iter::once;
use std::mem;
use std::ops::Range;
use unicode_segmentation::{Graphemes, UnicodeSegmentation};

/// Text editing core.
#[derive(Debug, Default, Clone)]
pub struct TextInputCore {
    // Text
    value: String,
    // Len in grapheme count.
    len: usize,

    // display information
    offset: usize,
    width: usize,

    // cursor and selection
    cursor: usize,
    anchor: usize,

    // tmp string for inserting a char.
    char_buf: String,
    // tmp string for editing.
    buf: String,
}

impl TextInputCore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Offset
    #[inline]
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Change the offset
    pub fn set_offset(&mut self, offset: usize) {
        if offset > self.len {
            self.offset = self.len;
        } else if offset > self.cursor {
            self.offset = self.cursor;
        } else if offset + self.width < self.cursor {
            self.offset = self.cursor - self.width;
        } else {
            self.offset = offset;
        }
    }

    // todo: need this
    /// Display width
    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    /// Display width
    #[inline]
    pub fn set_width(&mut self, width: usize) {
        self.width = width;

        if self.offset + width < self.cursor {
            self.offset = self.cursor - self.width;
        }
    }

    /// Cursor position as grapheme-idx. Moves the cursor to the new position,
    /// but can leave the current cursor position as anchor of the selection.
    pub fn set_cursor(&mut self, cursor: usize, extend_selection: bool) -> bool {
        let old_selection = (self.cursor, self.anchor);

        let c = min(self.len, cursor);

        self.cursor = c;

        if !extend_selection {
            self.anchor = c;
        }

        if self.offset > c {
            self.offset = c;
        } else if self.offset + self.width < c {
            self.offset = c - self.width;
        }

        (self.cursor, self.anchor) != old_selection
    }

    /// Cursor position as grapheme-idx.
    #[inline]
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Selection anchor
    #[inline]
    pub fn anchor(&self) -> usize {
        self.anchor
    }

    /// Set the value. Resets cursor and anchor to 0.
    #[inline]
    pub fn set_value<S: Into<String>>(&mut self, s: S) {
        self.value = s.into();
        self.len = self.value.graphemes(true).count();
        self.cursor = 0;
        self.offset = 0;
        self.anchor = 0;
    }

    /// Value
    #[inline]
    pub fn value(&self) -> &str {
        self.value.as_str()
    }

    // todo: glyphs

    /// Value as grapheme iterator.
    #[inline]
    pub fn value_graphemes(&self) -> Graphemes<'_> {
        self.value.graphemes(true)
    }

    /// Clear
    #[inline]
    pub fn clear(&mut self) {
        self.set_value("");
    }

    /// Empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Value lenght as grapheme-count
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Anchor is active
    #[inline]
    pub fn has_selection(&self) -> bool {
        self.anchor != self.cursor
    }

    /// Selection.
    #[inline]
    pub fn set_selection(&mut self, anchor: usize, cursor: usize) -> bool {
        let old_selection = self.selection();

        self.set_cursor(anchor, false);
        self.set_cursor(cursor, true);

        old_selection != self.selection()
    }

    /// Selection.
    #[inline]
    pub fn select_all(&mut self) -> bool {
        let old_selection = self.selection();

        self.set_cursor(0, false);
        self.set_cursor(self.value.len(), true);

        old_selection != self.selection()
    }

    /// Selection.
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

    /// Insert a char at the position.
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

        // fix offset
        if self.offset > self.cursor {
            self.offset = self.cursor;
        } else if self.offset + self.width < self.cursor {
            self.offset = self.cursor - self.width;
        }

        true
    }

    /// Insert a char at the position.
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

        // fix offset
        if self.offset > self.cursor {
            self.offset = self.cursor;
        } else if self.offset + self.width < self.cursor {
            self.offset = self.cursor - self.width;
        }

        true
    }

    /// Remove the range.
    pub fn remove_range(&mut self, range: Range<usize>) -> bool {
        let (before_str, _remove_str, after_str) = util::split3(self.value.as_str(), range.clone());

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

        // fix offset
        if self.offset > self.cursor {
            self.offset = self.cursor;
        } else if self.offset + self.width < self.cursor {
            self.offset = self.cursor - self.width;
        }

        true
    }
}
