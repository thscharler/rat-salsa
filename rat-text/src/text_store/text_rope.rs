use crate::grapheme::{RopeGraphemes, StrGraphemes};
use crate::text_store::{Cursor, TextStore};
use crate::{TextError, TextPosition, TextRange, upos_type};
use ropey::{Rope, RopeSlice};
use std::borrow::Cow;
use std::cell::Cell;
use std::cmp::min;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

/// Text store with a rope.
#[derive(Debug, Clone, Default)]
pub struct TextRope {
    text: Rope,
    // minimum byte position changed since last reset.
    min_changed: Cell<Option<usize>>,
    // tmp buf
    buf: String,
}

impl TextRope {
    /// New empty.
    pub fn new() -> Self {
        Self::default()
    }

    /// New from string.
    pub fn new_text(t: &str) -> Self {
        Self {
            text: Rope::from_str(t),
            min_changed: Default::default(),
            buf: Default::default(),
        }
    }

    /// New from rope.
    pub fn new_rope(r: Rope) -> Self {
        Self {
            text: r,
            min_changed: Default::default(),
            buf: Default::default(),
        }
    }

    /// Borrow the rope
    pub fn rope(&self) -> &Rope {
        &self.text
    }

    /// A range of the text as RopeSlice.
    #[inline]
    #[deprecated]
    pub fn rope_slice(&self, range: TextRange) -> Result<RopeSlice<'_>, TextError> {
        let s = self.byte_range(range)?;
        Ok(self.text.get_byte_slice(s).expect("valid_range"))
    }
}

impl TextRope {
    fn invalidate(&self, byte_pos: usize) {
        self.min_changed.update(|v| match v {
            None => Some(byte_pos),
            Some(w) => Some(min(byte_pos, w)),
        });
    }

    fn normalize_row(&self, row: upos_type) -> Result<upos_type, TextError> {
        let text_len = self.len_lines() as upos_type;
        let rope_len = self.text.len_lines() as upos_type;

        if row <= rope_len {
            Ok(row)
        } else if row <= text_len {
            Ok(row - 1)
        } else {
            Err(TextError::LineIndexOutOfBounds(row, text_len))
        }
    }

    fn normalize(&self, pos: TextPosition) -> Result<(TextPosition, usize), TextError> {
        let len = self.len_lines();
        if pos.y > len {
            Err(TextError::LineIndexOutOfBounds(pos.y, len))
        } else if pos.x > 0 && pos.y == len {
            Err(TextError::ColumnIndexOutOfBounds(pos.x, 0))
        } else if pos.x > 0 && pos.y == len - 1 && !self.has_final_newline() {
            Err(TextError::ColumnIndexOutOfBounds(pos.x, 0))
        } else if pos.x == 0 && pos.y == len {
            let pos_byte = self.byte_range_at(pos)?;
            Ok((
                self.byte_to_pos(pos_byte.start).expect("valid-byte"),
                pos_byte.start,
            ))
        } else if pos.x == 0 && pos.y == len - 1 && !self.has_final_newline() {
            let pos_byte = self.byte_range_at(pos)?;
            Ok((
                self.byte_to_pos(pos_byte.start).expect("valid-byte"),
                pos_byte.start,
            ))
        } else {
            let pos_byte = self.byte_range_at(pos)?;
            Ok((pos, pos_byte.start))
        }
    }
}

impl TextStore for TextRope {
    type GraphemeIter<'a> = RopeGraphemes<'a>;

    /// Can store multi-line content?
    ///
    /// If this returns false it is an error to call any function with
    /// a row other than `0`.
    fn is_multi_line(&self) -> bool {
        true
    }

    /// Minimum byte position that has been changed
    /// since the last call of min_changed().
    ///
    /// Used to invalidate caches.
    fn cache_validity(&self) -> Option<usize> {
        self.min_changed.take()
    }

    /// Content as string.
    fn string(&self) -> String {
        self.text.to_string()
    }

    /// Set content.
    fn set_string(&mut self, t: &str) {
        self.invalidate(0);
        self.text = Rope::from_str(t);
    }

    /// Grapheme position to byte position.
    /// This is the (start,end) position of the single grapheme after pos.
    ///
    /// * pos must be a valid position: row <= len_lines, col <= line_width of the row.
    fn byte_range_at(&self, pos: TextPosition) -> Result<Range<usize>, TextError> {
        let it_line = self.line_graphemes(pos.y)?;

        let mut col = 0;
        let mut byte_end = it_line.text_offset();
        for grapheme in it_line {
            if col == pos.x {
                return Ok(grapheme.text_bytes());
            }
            col += 1;
            byte_end = grapheme.text_bytes().end;
        }
        // one past the end is ok.
        if col == pos.x {
            Ok(byte_end..byte_end)
        } else {
            Err(TextError::ColumnIndexOutOfBounds(pos.x, col))
        }
    }

    /// Grapheme range to byte range.
    ///
    /// * range must be a valid range. row <= len_lines, col <= line_width of the row.
    fn byte_range(&self, range: TextRange) -> Result<Range<usize>, TextError> {
        if range.start.y == range.end.y {
            let it_line = self.line_graphemes(range.start.y)?;

            let mut range_start = None;
            let mut range_end = None;
            let mut col = 0;
            let mut byte_end = it_line.text_offset();
            for grapheme in it_line {
                if col == range.start.x {
                    range_start = Some(grapheme.text_bytes().start);
                }
                if col == range.end.x {
                    range_end = Some(grapheme.text_bytes().end);
                }
                if range_start.is_some() && range_end.is_some() {
                    break;
                }
                col += 1;
                byte_end = grapheme.text_bytes().end;
            }
            // one past the end is ok.
            if col == range.start.x {
                range_start = Some(byte_end);
            }
            if col == range.end.x {
                range_end = Some(byte_end);
            }

            let Some(range_start) = range_start else {
                return Err(TextError::ColumnIndexOutOfBounds(range.start.x, col));
            };
            let Some(range_end) = range_end else {
                return Err(TextError::ColumnIndexOutOfBounds(range.end.x, col));
            };

            Ok(range_start..range_end)
        } else {
            let range_start = self.byte_range_at(range.start)?;
            let range_end = self.byte_range_at(range.end)?;

            Ok(range_start.start..range_end.start)
        }
    }

    /// Byte position to grapheme position.
    /// Returns the position that contains the given byte index.
    ///
    /// * byte must <= byte-len.
    fn byte_to_pos(&self, byte_pos: usize) -> Result<TextPosition, TextError> {
        let Ok(row) = self.text.try_byte_to_line(byte_pos) else {
            return Err(TextError::ByteIndexOutOfBounds(
                byte_pos,
                self.text.len_bytes(),
            ));
        };
        let row = row as upos_type;

        let mut col = 0;
        let it_line = self.line_graphemes(row)?;
        for grapheme in it_line {
            if byte_pos < grapheme.text_bytes().end {
                break;
            }
            col += 1;
        }

        Ok(TextPosition::new(col, row))
    }

    /// Byte range to grapheme range.
    ///
    /// * byte must <= byte-len.
    fn bytes_to_range(&self, bytes: Range<usize>) -> Result<TextRange, TextError> {
        let Ok(start_row) = self.text.try_byte_to_line(bytes.start) else {
            return Err(TextError::ByteIndexOutOfBounds(
                bytes.start,
                self.text.len_bytes(),
            ));
        };
        let start_row = start_row as upos_type;
        let Ok(end_row) = self.text.try_byte_to_line(bytes.end) else {
            return Err(TextError::ByteIndexOutOfBounds(
                bytes.end,
                self.text.len_bytes(),
            ));
        };
        let end_row = end_row as upos_type;

        if start_row == end_row {
            let mut col = 0;
            let mut start = None;
            let mut end = None;
            let it_line = self.line_graphemes(start_row)?;
            for grapheme in it_line {
                if bytes.start < grapheme.text_bytes().end {
                    if start.is_none() {
                        start = Some(col);
                    }
                }
                if bytes.end < grapheme.text_bytes().end {
                    if end.is_none() {
                        end = Some(col);
                    }
                }
                if start.is_some() && end.is_some() {
                    break;
                }
                col += 1;
            }
            if bytes.start == self.text.len_bytes() {
                start = Some(col);
            }
            if bytes.end == self.text.len_bytes() {
                end = Some(col);
            }

            let Some(start) = start else {
                return Err(TextError::ByteIndexOutOfBounds(
                    bytes.start,
                    self.text.len_bytes(),
                ));
            };
            let Some(end) = end else {
                return Err(TextError::ByteIndexOutOfBounds(
                    bytes.end,
                    self.text.len_bytes(),
                ));
            };

            Ok(TextRange::new((start, start_row), (end, end_row)))
        } else {
            let start = self.byte_to_pos(bytes.start)?;
            let end = self.byte_to_pos(bytes.end)?;

            Ok(TextRange::new(start, end))
        }
    }

    /// A range of the text as `Cow<str>`.
    ///
    /// * range must be a valid range. row <= len_lines, col <= line_width of the row.
    /// * pos must be inside of range.
    fn str_slice(&self, range: TextRange) -> Result<Cow<'_, str>, TextError> {
        let range = self.byte_range(range)?;
        let v = self.text.byte_slice(range);
        match v.as_str() {
            Some(v) => Ok(Cow::Borrowed(v)),
            None => Ok(Cow::Owned(v.to_string())),
        }
    }

    /// A range of the text as `Cow<str>`.
    ///
    /// The byte-range must be a valid range.
    fn str_slice_byte(&self, range: Range<usize>) -> Result<Cow<'_, str>, TextError> {
        let Some(v) = self.text.get_byte_slice(range.clone()) else {
            return Err(TextError::ByteRangeOutOfBounds(
                Some(range.start),
                Some(range.end),
                self.text.len_bytes(),
            ));
        };
        match v.as_str() {
            Some(v) => Ok(Cow::Borrowed(v)),
            None => Ok(Cow::Owned(v.to_string())),
        }
    }

    /// Return a cursor over the graphemes of the range, start at the given position.
    ///
    /// * range must be a valid range. row <= len_lines, col <= line_width of the row.
    /// * pos must be inside of range.
    fn graphemes(
        &self,
        range: TextRange,
        pos: TextPosition,
    ) -> Result<Self::GraphemeIter<'_>, TextError> {
        if !range.contains_pos(pos) && range.end != pos {
            return Err(TextError::TextPositionOutOfBounds(pos));
        }

        let range_bytes = self.byte_range(range)?;
        let pos_byte = self.byte_range_at(pos)?.start;

        let s = self
            .text
            .get_byte_slice(range_bytes.clone())
            .expect("valid_range");

        let r = RopeGraphemes::new_offset(range_bytes.start, s, pos_byte - range_bytes.start)
            .expect("valid_bytes");

        Ok(r)
    }

    /// Return a cursor over the graphemes of the range, start at the given position.
    ///
    /// * range must be a valid byte-range.
    /// * pos must be inside of range.
    fn graphemes_byte(
        &self,
        range: Range<usize>,
        pos: usize,
    ) -> Result<Self::GraphemeIter<'_>, TextError> {
        if !range.contains(&pos) && range.end != pos {
            return Err(TextError::ByteIndexOutOfBounds(pos, range.end));
        }

        let Some(s) = self.text.get_byte_slice(range.clone()) else {
            return Err(TextError::ByteRangeInvalid(range.start, range.end));
        };

        let r = RopeGraphemes::new_offset(range.start, s, pos - range.start)?;

        Ok(r)
    }

    /// Line as str.
    ///
    /// * row must be <= len_lines
    fn line_at(&self, row: upos_type) -> Result<Cow<'_, str>, TextError> {
        let len = self.len_lines() as upos_type;
        if row < len {
            if row < self.text.len_lines() as upos_type {
                let v = self.text.get_line(row as usize).expect("valid_row");
                match v.as_str() {
                    Some(v) => Ok(Cow::Borrowed(v)),
                    None => Ok(Cow::Owned(v.to_string())),
                }
            } else {
                Ok(Cow::Borrowed(""))
            }
        } else {
            Err(TextError::LineIndexOutOfBounds(row, len))
        }
    }

    /// Iterate over text-lines, starting at line-offset.
    ///
    /// * row must be <= len_lines
    fn lines_at(&self, row: upos_type) -> Result<impl Iterator<Item = Cow<'_, str>>, TextError> {
        let len = self.len_lines() as upos_type;
        if row < len {
            let it = self.text.get_lines_at(row as usize).expect("valid_row");
            Ok(it.map(|v| match v.as_str() {
                Some(v) => Cow::Borrowed(v),
                None => Cow::Owned(v.to_string()),
            }))
        } else {
            Err(TextError::LineIndexOutOfBounds(row, len))
        }
    }

    /// Return a line as an iterator over the graphemes.
    /// This contains the '\n' at the end.
    ///
    /// * row must be <= len_lines
    #[inline]
    fn line_graphemes(&self, row: upos_type) -> Result<Self::GraphemeIter<'_>, TextError> {
        let row = self.normalize_row(row)?;
        let line_byte = self.text.try_line_to_byte(row as usize)?;
        let line = if row < self.text.len_lines() as upos_type {
            self.text.get_line(row as usize).expect("valid_row")
        } else {
            RopeSlice::from("")
        };
        Ok(RopeGraphemes::new(line_byte, line))
    }

    /// Line width as grapheme count.
    /// Excludes the terminating '\n'.
    ///
    /// * row must be <= len_lines
    #[inline]
    fn line_width(&self, row: upos_type) -> Result<upos_type, TextError> {
        let row = self.normalize_row(row)?;

        if row < self.text.len_lines() as upos_type {
            let r = self.text.get_line(row as usize).expect("valid_row");
            let len = RopeGraphemes::new(0, r)
                .filter(|g| !g.is_line_break())
                .count() as upos_type;
            Ok(len)
        } else {
            Ok(0)
        }
    }

    #[inline]
    #[allow(clippy::match_like_matches_macro)]
    fn has_final_newline(&self) -> bool {
        let len = self.text.len_bytes();
        if len > 3 {
            match (
                self.text.get_byte(len - 3).expect("valid_pos"),
                self.text.get_byte(len - 2).expect("valid_pos"),
                self.text.get_byte(len - 1).expect("valid_pos"),
            ) {
                (_, _, b'\n')
                | (_, _, b'\r')
                | (_, _, 0x0c)
                | (_, _, 0x0b)
                | (_, _, 0x85)
                | (0xE2, 0x80, 0xA8)
                | (0xE2, 0x80, 0xA9) => true,
                _ => false,
            }
        } else if len > 0 {
            match self.text.get_byte(len - 1).expect("valid_pos") {
                b'\n' | b'\r' | 0x0c | 0x0b | 0x85 => true,
                _ => false,
            }
        } else {
            false
        }
    }

    #[inline]
    fn len_bytes(&self) -> usize {
        self.text.len_bytes()
    }

    #[inline]
    fn len_lines(&self) -> upos_type {
        match self.text.len_bytes() {
            0 => 1,
            _ => {
                let l = self.text.len_lines();
                let t = if self.has_final_newline() { 0 } else { 1 };
                (l + t) as upos_type
            }
        }
    }

    /// Insert a char at the given position.
    ///
    /// * range must be a valid range. row <= len_lines, col <= line_width of the row.
    fn insert_char(
        &mut self,
        mut pos: TextPosition,
        ch: char,
    ) -> Result<(TextRange, Range<usize>), TextError> {
        // normalize the position (0, len_lines) to something sane.
        let pos_byte;
        (pos, pos_byte) = self.normalize(pos)?;

        // invalidate cache
        self.invalidate(pos_byte);

        let mut it_gr =
            RopeGraphemes::new_offset(0, self.text.slice(..), pos_byte).expect("valid_bytes");
        let prev = it_gr.prev();
        it_gr.next();
        let next = it_gr.next();

        let insert_range = if ch == '\n' {
            if let Some(prev) = prev {
                if prev == "\r" {
                    TextRange::new(pos, pos)
                } else {
                    TextRange::new(pos, (0, pos.y + 1))
                }
            } else {
                TextRange::new(pos, (0, pos.y + 1))
            }
        } else if ch == '\r' {
            if let Some(next) = next {
                if next == "\n" {
                    TextRange::new(pos, pos)
                } else {
                    TextRange::new(pos, (0, pos.y + 1))
                }
            } else {
                TextRange::new(pos, (0, pos.y + 1))
            }
        } else if cfg!(feature = "unicode_lines")
            && (ch == '\u{000C}'
                || ch == '\u{000B}'
                || ch == '\u{0085}'
                || ch == '\u{2028}'
                || ch == '\u{2029}')
        {
            TextRange::new(pos, (0, pos.y + 1))
        } else {
            // test for combining codepoints.
            let mut len = 0;
            self.buf.clear();
            if let Some(prev) = prev {
                len += 1;
                self.buf.push_str(prev.grapheme());
            }
            len += 1;
            self.buf.push(ch);
            if let Some(next) = next {
                len += 1;
                self.buf.push_str(next.grapheme());
            }
            let buf_len = self.buf.graphemes(true).count();

            let n = len - buf_len;

            if n == 0 {
                TextRange::new(pos, (pos.x + 1, pos.y))
            } else if n == 1 {
                // combined some
                TextRange::new(pos, pos)
            } else if n == 2 {
                // combined some
                TextRange::new(pos, pos)
            } else {
                unreachable!("insert_char {:?}", self.buf);
            }
        };

        let pos_char = self.text.try_byte_to_char(pos_byte).expect("valid_bytes");

        self.text
            .try_insert_char(pos_char, ch)
            .expect("valid_chars");

        Ok((insert_range, pos_byte..pos_byte + ch.len_utf8()))
    }

    /// Insert a text str at the given position.
    ///
    /// * range must be a valid range. row <= len_lines, col <= line_width of the row.
    fn insert_str(
        &mut self,
        mut pos: TextPosition,
        txt: &str,
    ) -> Result<(TextRange, Range<usize>), TextError> {
        // normalize the position (0, len_lines-1) to something sane.
        let pos_byte;
        (pos, pos_byte) = self.normalize(pos)?;

        self.invalidate(pos_byte);

        let pos_char = self.text.try_byte_to_char(pos_byte).expect("valid_bytes");

        let mut line_count = 0;
        let mut last_linebreak_idx = 0;
        for c in StrGraphemes::new(0, txt) {
            let test = if cfg!(feature = "cr_lines") {
                c == "\r" || c == "\n" || c == "\r\n"
            } else if cfg!(feature = "unicode_lines") {
                c == "\r"
                    || c == "\n"
                    || c == "\r\n"
                    || c == "\u{000C}"
                    || c == "\u{000B}"
                    || c == "\u{0085}"
                    || c == "\u{2028}"
                    || c == "\u{2029}"
            } else {
                c == "\n" || c == "\r\n"
            };

            if test {
                line_count += 1;
                last_linebreak_idx = c.text_bytes().end;
            }
        }

        let insert_range = if line_count > 0 {
            // the remainder of the line after pos extends the last line of
            // the inserted text. they might combine in some way.

            // Fill in the last line of the inserted text.
            self.buf.clear();
            self.buf.push_str(&txt[last_linebreak_idx..]);
            let old_offset = self.buf.len();

            // Fill in the remainder of the current text after the insert position.
            let line_offset = self
                .text
                .try_line_to_byte(pos.y as usize)
                .expect("valid-pos");
            let split = self //
                .byte_range_at(pos)
                .expect("valid_pos")
                .start
                - line_offset;
            let remainder = self
                .text
                .get_line(pos.y as usize)
                .expect("valid-pos")
                .get_byte_slice(split..)
                .expect("valid-pos");
            for cc in remainder.chars() {
                self.buf.push(cc);
            }
            let new_len = self.buf.graphemes(true).count() as upos_type;
            let old_len = self.buf[old_offset..].graphemes(true).count() as upos_type;

            self.text.try_insert(pos_char, txt).expect("valid_pos");

            TextRange::new(pos, (new_len - old_len, pos.y + line_count))
        } else {
            // no way to know if the insert text combines with a surrounding char.
            // the difference of the grapheme len seems safe though.
            let old_len = self.line_width(pos.y).expect("valid_line");
            self.text.try_insert(pos_char, txt).expect("valid_pos");
            let new_len = self.line_width(pos.y).expect("valid_line");

            TextRange::new(pos, (pos.x + new_len - old_len, pos.y))
        };

        Ok((insert_range, pos_byte..pos_byte + txt.len()))
    }

    /// Remove the given text range.
    ///
    /// * range must be a valid range. row <= len_lines, col <= line_width of the row.
    fn remove(
        &mut self,
        mut range: TextRange,
    ) -> Result<(String, (TextRange, Range<usize>)), TextError> {
        let start_byte_pos;
        let end_byte_pos;

        (range.start, start_byte_pos) = self.normalize(range.start)?;
        (range.end, end_byte_pos) = self.normalize(range.end)?;

        self.invalidate(start_byte_pos);

        let old_text = self
            .text
            .get_byte_slice(start_byte_pos..end_byte_pos)
            .expect("valid_bytes");
        let old_text = old_text.to_string();

        let start_pos = self
            .text
            .try_byte_to_char(start_byte_pos)
            .expect("valid_bytes");
        let end_pos = self
            .text
            .try_byte_to_char(end_byte_pos)
            .expect("valid_bytes");

        self.text.try_remove(start_pos..end_pos).expect("valid_pos");

        Ok((old_text, (range, start_byte_pos..end_byte_pos)))
    }

    /// Insert a string at the given byte index.
    /// Call this only for undo.
    ///
    /// byte_pos must be <= len bytes.
    fn insert_b(&mut self, byte_pos: usize, t: &str) -> Result<(), TextError> {
        let pos_char = self.text.try_byte_to_char(byte_pos)?;

        self.invalidate(byte_pos);
        self.text.try_insert(pos_char, t).expect("valid_pos");
        Ok(())
    }

    /// Remove the given byte-range.
    /// Call this only for undo.
    ///
    /// byte_pos must be <= len bytes.
    fn remove_b(&mut self, byte_range: Range<usize>) -> Result<(), TextError> {
        let start_char = self.text.try_byte_to_char(byte_range.start)?;
        let end_char = self.text.try_byte_to_char(byte_range.end)?;

        self.invalidate(byte_range.start);
        self.text
            .try_remove(start_char..end_char)
            .expect("valid_range");
        Ok(())
    }
}

impl From<ropey::Error> for TextError {
    fn from(err: ropey::Error) -> Self {
        use ropey::Error;
        match err {
            Error::ByteIndexOutOfBounds(i, l) => TextError::ByteIndexOutOfBounds(i, l),
            Error::CharIndexOutOfBounds(i, l) => TextError::CharIndexOutOfBounds(i, l),
            Error::LineIndexOutOfBounds(i, l) => {
                TextError::LineIndexOutOfBounds(i as upos_type, l as upos_type)
            }
            Error::Utf16IndexOutOfBounds(_, _) => {
                unreachable!("{:?}", err)
            }
            Error::ByteIndexNotCharBoundary(i) => TextError::ByteIndexNotCharBoundary(i),
            Error::ByteRangeNotCharBoundary(s, e) => TextError::ByteRangeNotCharBoundary(s, e),
            Error::ByteRangeInvalid(s, e) => TextError::ByteRangeInvalid(s, e),
            Error::CharRangeInvalid(s, e) => TextError::CharRangeInvalid(s, e),
            Error::ByteRangeOutOfBounds(s, e, l) => TextError::ByteRangeOutOfBounds(s, e, l),
            Error::CharRangeOutOfBounds(s, e, l) => TextError::CharRangeOutOfBounds(s, e, l),
            _ => {
                unreachable!("{:?}", err)
            }
        }
    }
}
