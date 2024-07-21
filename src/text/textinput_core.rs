use crate::util;
use crate::util::gr_len;
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
    pub fn selection(&self) -> Range<usize> {
        if self.cursor < self.anchor {
            self.cursor..self.anchor
        } else {
            self.anchor..self.cursor
        }
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

    /// Find next word.
    pub fn next_word_boundary(&self, pos: usize) -> Option<usize> {
        let byte_pos = self.byte_at(pos)?;

        let (_, str_after) = self.value.split_at(byte_pos.0);
        let mut it = str_after.graphemes(true);
        let mut init = true;
        let mut gp = 0;
        loop {
            let Some(c) = it.next() else {
                break;
            };

            if init {
                if let Some(c) = c.chars().next() {
                    if !c.is_whitespace() {
                        init = false;
                    }
                }
            } else {
                if let Some(c) = c.chars().next() {
                    if c.is_whitespace() {
                        break;
                    }
                }
            }

            gp += 1;
        }

        Some(pos + gp)
    }

    /// Find previous word.
    pub fn prev_word_boundary(&self, pos: usize) -> Option<usize> {
        let byte_pos = self.byte_at(pos)?;

        let (str_before, _) = self.value.split_at(byte_pos.0);
        let mut it = str_before.graphemes(true).rev();
        let mut init = true;
        let mut gp = gr_len(str_before);
        loop {
            let Some(c) = it.next() else {
                break;
            };

            if init {
                if let Some(c) = c.chars().next() {
                    if !c.is_whitespace() {
                        init = false;
                    }
                }
            } else {
                if let Some(c) = c.chars().next() {
                    if c.is_whitespace() {
                        break;
                    }
                }
            }

            gp -= 1;
        }

        Some(gp)
    }

    /// Insert a char, replacing the selection.
    pub fn insert_char(&mut self, new: char) -> bool {
        let selection = self.selection();

        let mut char_buf = mem::take(&mut self.char_buf);
        char_buf.clear();
        char_buf.push(new);

        let r = self.replace(selection, char_buf.as_str());

        self.char_buf = char_buf;

        r
    }

    /// Remove the selection.
    #[inline]
    pub fn remove(&mut self, range: Range<usize>) -> bool {
        self.replace(range, "")
    }

    /// Insert a string, replacing the selection.
    pub fn replace(&mut self, range: Range<usize>, new: &str) -> bool {
        let (before_str, remove_str, after_str) = util::split3(self.value.as_str(), range.clone());

        self.buf.clear();
        self.buf.push_str(before_str);
        self.buf.push_str(remove_str);
        let old_end = self.buf.graphemes(true).count();
        self.buf.clear();
        self.buf.push_str(before_str);
        self.buf.push_str(new);
        let new_end = self.buf.graphemes(true).count();
        self.buf.push_str(after_str);
        mem::swap(&mut self.value, &mut self.buf);

        let change_len = new_end as isize - old_end as isize;
        self.len = (self.len as isize + change_len) as usize;

        if self.cursor < range.start {
            // noop
        } else if self.cursor <= range.end {
            self.cursor = new_end;
        } else {
            self.cursor = (self.cursor as isize + change_len) as usize;
        }

        if self.anchor < range.start {
            // noop
        } else if self.anchor <= range.end {
            self.anchor = new_end;
        } else {
            self.anchor = (self.anchor as isize + change_len) as usize;
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
