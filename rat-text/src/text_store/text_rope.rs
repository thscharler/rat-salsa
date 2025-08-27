use crate::grapheme::RopeGraphemes;
use crate::text_store::{Cursor, TextStore};
use crate::{upos_type, TextError, TextPosition, TextRange};
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
    buf: Vec<u8>,
}

/// Length as grapheme count, excluding line breaks.
#[inline]
fn rope_line_len(r: RopeSlice<'_>) -> upos_type {
    let it = RopeGraphemes::new(0, r);
    it.filter(|g| !g.is_line_break()).count() as upos_type
}

/// Length as grapheme count, excluding line breaks.
///
/// Safety
///
/// Bytes must be valid UTF-8
#[inline]
unsafe fn str_line_len(s: &[u8]) -> upos_type {
    let s = unsafe { str::from_utf8_unchecked(s) };
    let it = s.graphemes(true);
    it.filter(|c| *c != "\n" && *c != "\r\n").count() as upos_type
}

/// Length as grapheme count, including line breaks.
///
/// Safety
///
/// Bytes must be valid UTF-8
#[inline]
unsafe fn str_grapheme_len(s: &[u8]) -> upos_type {
    let s = unsafe { str::from_utf8_unchecked(s) };
    let it = s.graphemes(true);
    it.count() as upos_type
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
    fn set_min_changed(&self, byte_pos: usize) {
        self.min_changed.update(|v| match v {
            None => Some(byte_pos),
            Some(w) => Some(min(byte_pos, w)),
        });
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
    /// Can be used to invalidate caches.
    fn min_changed(&self) -> Option<usize> {
        self.min_changed.take()
    }

    /// Content as string.
    fn string(&self) -> String {
        self.text.to_string()
    }

    /// Set content.
    fn set_string(&mut self, t: &str) {
        self.set_min_changed(0);
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
        let len = self.text.len_lines() as upos_type;
        if row > len {
            Err(TextError::LineIndexOutOfBounds(row, len))
        } else if row == len {
            Ok(Cow::Borrowed(""))
        } else {
            let v = self.text.get_line(row as usize).expect("valid_row");
            match v.as_str() {
                Some(v) => Ok(Cow::Borrowed(v)),
                None => Ok(Cow::Owned(v.to_string())),
            }
        }
    }

    /// Iterate over text-lines, starting at line-offset.
    ///
    /// * row must be <= len_lines
    fn lines_at(&self, row: upos_type) -> Result<impl Iterator<Item = Cow<'_, str>>, TextError> {
        let len = self.text.len_lines() as upos_type;
        if row > len {
            Err(TextError::LineIndexOutOfBounds(row, len))
        } else {
            let it = self.text.get_lines_at(row as usize).expect("valid_row");
            Ok(it.map(|v| match v.as_str() {
                Some(v) => Cow::Borrowed(v),
                None => Cow::Owned(v.to_string()),
            }))
        }
    }

    /// Return a line as an iterator over the graphemes.
    /// This contains the '\n' at the end.
    ///
    /// * row must be <= len_lines
    #[inline]
    fn line_graphemes(&self, row: upos_type) -> Result<Self::GraphemeIter<'_>, TextError> {
        let line_byte = self.text.try_line_to_byte(row as usize)?;
        // try_line_to_byte and get_line don't have the same boundaries.
        // the former accepts one past the end, the latter doesn't.
        // here we need the first behaviour.
        if let Some(line) = self.text.get_line(row as usize) {
            Ok(RopeGraphemes::new(line_byte, line))
        } else {
            Ok(RopeGraphemes::new(line_byte, RopeSlice::from("")))
        }
    }

    /// Line width as grapheme count.
    /// Excludes the terminating '\n'.
    ///
    /// * row must be <= len_lines
    #[inline]
    fn line_width(&self, row: upos_type) -> Result<upos_type, TextError> {
        let len = self.text.len_lines() as upos_type;
        if row > len {
            Err(TextError::LineIndexOutOfBounds(row, len))
        } else if row == len {
            Ok(0)
        } else {
            let v = self.text.get_line(row as usize).expect("valid_row");
            Ok(rope_line_len(v))
        }
    }

    fn len_lines(&self) -> upos_type {
        self.text.len_lines() as upos_type
    }

    /// Insert a char at the given position.
    ///
    /// * range must be a valid range. row <= len_lines, col <= line_width of the row.
    fn insert_char(
        &mut self,
        pos: TextPosition,
        ch: char,
    ) -> Result<(TextRange, Range<usize>), TextError> {
        let pos_byte = self.byte_range_at(pos)?;

        // normalize the position (0, len_lines) to something sane.
        let pos = if pos.x == 0 && pos.y == self.len_lines() {
            self.byte_to_pos(pos_byte.start)?
        } else {
            pos
        };

        self.set_min_changed(pos_byte.start);

        let mut it_gr =
            RopeGraphemes::new_offset(0, self.text.slice(..), pos_byte.start).expect("valid_bytes");
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
        } else {
            // test for combining codepoints.
            let mut len = 0;
            self.buf.clear();
            if let Some(prev) = prev {
                len += 1;
                self.buf.extend_from_slice(prev.grapheme().as_bytes());
            }
            len += 1;
            let mut ch_buf = [0; 4];
            let ch_buf = ch.encode_utf8(&mut ch_buf);
            self.buf.extend_from_slice(ch_buf.as_bytes());

            if let Some(next) = next {
                len += 1;
                self.buf.extend_from_slice(next.grapheme().as_bytes());
            }

            let n = len - unsafe { str_grapheme_len(&self.buf) };
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

        let pos_char = self
            .text
            .try_byte_to_char(pos_byte.start)
            .expect("valid_bytes");
        self.text
            .try_insert_char(pos_char, ch)
            .expect("valid_chars");

        Ok((insert_range, pos_byte.start..pos_byte.start + ch.len_utf8()))
    }

    /// Insert a text str at the given position.
    ///
    /// * range must be a valid range. row <= len_lines, col <= line_width of the row.
    fn insert_str(
        &mut self,
        pos: TextPosition,
        txt: &str,
    ) -> Result<(TextRange, Range<usize>), TextError> {
        let pos_byte = self.byte_range_at(pos)?;

        // normalize the position (0, len_lines) to something sane.
        let pos = if pos.x == 0 && pos.y == self.len_lines() {
            self.byte_to_pos(pos_byte.start)?
        } else {
            pos
        };

        self.set_min_changed(pos_byte.start);

        let pos_char = self
            .text
            .try_byte_to_char(pos_byte.start)
            .expect("valid_bytes");

        let mut line_count = 0;
        let mut last_linebreak_idx = 0;
        let mut byte_count = 0;
        let mut was_cr = false;
        for c in txt.bytes() {
            if c == b'\r' {
                was_cr = true;
                line_count += 1;
                last_linebreak_idx = byte_count + 1;
            } else if c == b'\n' {
                if !was_cr {
                    line_count += 1;
                }
                was_cr = false;
                last_linebreak_idx = byte_count + 1;
            } else {
                was_cr = false;
            }
            byte_count += 1;
        }

        let insert_range = if line_count > 0 {
            // Find the length of line after the insert position.
            let split = self.byte_range_at(pos).expect("valid_pos");

            let line = self
                .text
                .get_line(pos.y as usize)
                .expect("valid_pos")
                .bytes();
            self.buf.clear();
            for c in line.skip(split.start) {
                self.buf.push(c);
            }
            let old_len = unsafe { str_line_len(&self.buf) };
            self.buf.clear();

            // compose the new line and find its length.
            self.buf
                .extend_from_slice(&txt.as_bytes()[last_linebreak_idx..]);

            let line = self
                .text
                .get_line(pos.y as usize)
                .expect("valid_pos")
                .bytes();
            for c in line.skip(split.start) {
                self.buf.push(c);
            }
            let new_len = unsafe { str_line_len(&self.buf) };
            self.buf.clear();

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

        Ok((insert_range, pos_byte.start..pos_byte.start + txt.len()))
    }

    /// Remove the given text range.
    ///
    /// * range must be a valid range. row <= len_lines, col <= line_width of the row.
    fn remove(
        &mut self,
        mut range: TextRange,
    ) -> Result<(String, (TextRange, Range<usize>)), TextError> {
        let start_byte_pos = self.byte_range_at(range.start)?;
        let end_byte_pos = self.byte_range_at(range.end)?;

        // normalize the position (0, len_lines) to something sane.
        range.end = if range.end.x == 0 && range.end.y == self.len_lines() {
            self.byte_to_pos(end_byte_pos.start)?
        } else {
            range.end
        };
        range.start = if range.start.x == 0 && range.start.y == self.len_lines() {
            self.byte_to_pos(start_byte_pos.start)?
        } else {
            range.start
        };

        self.set_min_changed(start_byte_pos.start);

        let old_text = self
            .text
            .get_byte_slice(start_byte_pos.start..end_byte_pos.start)
            .expect("valid_bytes");
        let old_text = old_text.to_string();

        let start_pos = self
            .text
            .try_byte_to_char(start_byte_pos.start)
            .expect("valid_bytes");
        let end_pos = self
            .text
            .try_byte_to_char(end_byte_pos.start)
            .expect("valid_bytes");

        self.text.try_remove(start_pos..end_pos).expect("valid_pos");

        Ok((old_text, (range, start_byte_pos.start..end_byte_pos.start)))
    }

    /// Insert a string at the given byte index.
    /// Call this only for undo.
    ///
    /// byte_pos must be <= len bytes.
    fn insert_b(&mut self, byte_pos: usize, t: &str) -> Result<(), TextError> {
        let pos_char = self.text.try_byte_to_char(byte_pos)?;

        self.set_min_changed(byte_pos);
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

        self.set_min_changed(byte_range.start);
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
