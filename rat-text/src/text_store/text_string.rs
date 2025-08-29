use crate::grapheme::StrGraphemes;
use crate::text_store::TextStore;
use crate::{upos_type, TextError, TextPosition, TextRange};
use std::borrow::Cow;
use std::cell::Cell;
use std::cmp::min;
use std::iter::once;
use std::mem;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

/// Single line text-store.
#[derive(Debug, Default, Clone)]
pub struct TextString {
    // text
    text: String,
    // len as grapheme count
    len: upos_type,
    // minimum byte position changed since last reset.
    min_changed: Cell<Option<usize>>,
    // tmp buffer
    buf: String,
}

/// Length as grapheme count, excluding line breaks.
#[inline]
fn str_len(s: &str) -> upos_type {
    s.graphemes(true).count() as upos_type
}

impl TextString {
    /// New empty.
    pub fn new() -> Self {
        Self {
            text: Default::default(),
            len: 0,
            min_changed: Default::default(),
            buf: Default::default(),
        }
    }

    /// New from string.
    pub fn new_text(t: &str) -> Self {
        Self {
            text: t.into(),
            len: str_len(t),
            min_changed: Default::default(),
            buf: Default::default(),
        }
    }

    /// New from string.
    pub fn new_string(t: String) -> Self {
        let len = str_len(&t);
        Self {
            text: t,
            len,
            min_changed: Default::default(),
            buf: Default::default(),
        }
    }

    /// str
    pub fn as_str(&self) -> &str {
        self.text.as_str()
    }
}

impl TextString {
    fn set_min_changed(&self, byte_pos: usize) {
        self.min_changed.update(|v| match v {
            None => Some(byte_pos),
            Some(w) => Some(min(byte_pos, w)),
        });
    }
}

impl TextStore for TextString {
    type GraphemeIter<'a> = StrGraphemes<'a>;

    /// Can store multi-line content?
    #[inline]
    fn is_multi_line(&self) -> bool {
        false
    }

    /// Always true.
    fn has_final_newline(&self) -> bool {
        true
    }

    /// Number of lines.
    #[inline]
    fn len_lines(&self) -> upos_type {
        1
    }

    #[inline]
    fn min_changed(&self) -> Option<usize> {
        self.min_changed.take()
    }

    /// Get content as string.
    fn string(&self) -> String {
        self.text.to_string()
    }

    /// Set content as string.
    fn set_string(&mut self, t: &str) {
        self.set_min_changed(0);
        self.text = t.to_string();
        self.len = str_len(&self.text);
    }

    /// Grapheme position to byte position.
    /// This is the (start,end) position of the single grapheme after pos.
    ///
    /// * pos must be a valid position: row < len_lines, col <= line_width of the row.
    fn byte_range_at(&self, pos: TextPosition) -> Result<Range<usize>, TextError> {
        if pos == TextPosition::new(0, 1) {
            let len = self.text.len();
            return Ok(len..len);
        }

        if pos.y != 0 {
            return Err(TextError::LineIndexOutOfBounds(pos.y, 1));
        };

        let mut byte_range = None;
        for (cidx, (idx, c)) in self
            .text
            .grapheme_indices(true)
            .chain(once((self.text.len(), "")))
            .enumerate()
        {
            if cidx == pos.x as usize {
                byte_range = Some(idx..idx + c.len());
                break;
            }
        }

        if let Some(byte_range) = byte_range {
            Ok(byte_range)
        } else {
            Err(TextError::ColumnIndexOutOfBounds(
                pos.x,
                str_len(&self.text),
            ))
        }
    }

    /// Grapheme range to byte range.
    ///
    /// Allows the special text-position (0,1) as a substitute for EOL.
    ///
    /// * range must be a valid range. row < len_lines, col <= line_width of the row.
    fn byte_range(&self, range: TextRange) -> Result<Range<usize>, TextError> {
        if range.start.y != 0 && range.start != TextPosition::new(0, 1) {
            return Err(TextError::LineIndexOutOfBounds(range.start.y, 1));
        };
        if range.end.y != 0 && range.end != TextPosition::new(0, 1) {
            return Err(TextError::LineIndexOutOfBounds(range.end.y, 1));
        };

        let mut byte_start = None;
        let mut byte_end = None;

        if range.start == TextPosition::new(0, 1) {
            byte_start = Some(self.text.len());
        }
        if range.end == TextPosition::new(0, 1) {
            byte_end = Some(self.text.len());
        }

        if byte_start.is_none() || byte_end.is_none() {
            for (cidx, (idx, _)) in self
                .text
                .grapheme_indices(true)
                .chain(once((self.text.len(), "")))
                .enumerate()
            {
                if TextPosition::new(cidx as upos_type, 0) == range.start {
                    byte_start = Some(idx);
                }
                if TextPosition::new(cidx as upos_type, 0) == range.end {
                    byte_end = Some(idx);
                }
                if byte_start.is_some() && byte_end.is_some() {
                    break;
                }
            }
        }

        let Some(byte_start) = byte_start else {
            return Err(TextError::ColumnIndexOutOfBounds(
                range.start.x,
                str_len(&self.text),
            ));
        };
        let Some(byte_end) = byte_end else {
            return Err(TextError::ColumnIndexOutOfBounds(
                range.end.x,
                str_len(&self.text),
            ));
        };

        Ok(byte_start..byte_end)
    }

    /// Byte position to grapheme position.
    /// Returns the position that contains the given byte index.
    ///
    /// * byte must <= byte-len.
    fn byte_to_pos(&self, byte_pos: usize) -> Result<TextPosition, TextError> {
        let mut pos = None;

        for (cidx, (c_start, c)) in self
            .text
            .grapheme_indices(true)
            .chain(once((self.text.len(), " ")))
            .enumerate()
        {
            if byte_pos < c_start + c.len() {
                pos = Some(cidx);
                break;
            }
        }

        if let Some(pos) = pos {
            Ok(TextPosition::new(pos as upos_type, 0))
        } else {
            Err(TextError::ByteIndexOutOfBounds(byte_pos, self.text.len()))
        }
    }

    /// Byte range to grapheme range.
    ///
    /// * byte must <= byte-len.
    fn bytes_to_range(&self, bytes: Range<usize>) -> Result<TextRange, TextError> {
        let mut start = None;
        let mut end = None;
        for (cidx, (c_start, c)) in self
            .text
            .grapheme_indices(true)
            .chain(once((self.text.len(), " ")))
            .enumerate()
        {
            if bytes.start < c_start + c.len() {
                if start.is_none() {
                    start = Some(cidx as upos_type);
                }
            }
            if bytes.end < c_start + c.len() {
                if end.is_none() {
                    end = Some(cidx as upos_type);
                }
            }
            if start.is_some() && end.is_some() {
                break;
            }
        }

        let Some(start) = start else {
            return Err(TextError::ByteIndexOutOfBounds(
                bytes.start,
                self.text.len(),
            ));
        };
        let Some(end) = end else {
            return Err(TextError::ByteIndexOutOfBounds(bytes.end, self.text.len()));
        };

        Ok(TextRange::new((start, 0), (end, 0)))
    }

    /// A range of the text as `Cow<str>`.
    ///
    /// * range must be a valid range. row < len_lines, col <= line_width of the row.
    /// * pos must be inside of range.
    #[inline]
    fn str_slice(&self, range: TextRange) -> Result<Cow<'_, str>, TextError> {
        let range = self.byte_range(range)?;
        Ok(Cow::Borrowed(&self.text[range.start..range.end]))
    }

    /// A range of the text as `Cow<str>`.
    ///
    /// * range must be valid
    #[inline]
    fn str_slice_byte(&self, range: Range<usize>) -> Result<Cow<'_, str>, TextError> {
        Ok(Cow::Borrowed(&self.text[range.start..range.end]))
    }

    /// Return a cursor over the graphemes of the range, start at the given position.
    ///
    /// * range must be a valid range. row < len_lines, col <= line_width of the row.
    /// * pos must be inside of range.
    fn graphemes(
        &self,
        range: TextRange,
        pos: TextPosition,
    ) -> Result<Self::GraphemeIter<'_>, TextError> {
        let range_byte = self.byte_range(range)?;
        let pos_byte = self.byte_range_at(pos)?;
        Ok(StrGraphemes::new_offset(
            range_byte.start,
            &self.text[range_byte.clone()],
            pos_byte.start - range_byte.start,
        ))
    }

    fn graphemes_byte(
        &self,
        range: Range<usize>,
        pos: usize,
    ) -> Result<Self::GraphemeIter<'_>, TextError> {
        if !range.contains(&pos) && range.end != pos {
            return Err(TextError::ByteIndexOutOfBounds(pos, range.end));
        }
        if !self.text.is_char_boundary(range.start) || !self.text.is_char_boundary(range.end) {
            return Err(TextError::ByteRangeNotCharBoundary(
                Some(range.start),
                Some(range.end),
            ));
        }
        if !self.text.is_char_boundary(pos) {
            return Err(TextError::ByteIndexNotCharBoundary(pos));
        }

        Ok(StrGraphemes::new_offset(
            range.start,
            &self.text[range.clone()],
            pos - range.start,
        ))
    }

    /// Line as str.
    ///
    /// * row must be < len_lines
    #[inline]
    fn line_at(&self, row: upos_type) -> Result<Cow<'_, str>, TextError> {
        if row == 0 {
            Ok(Cow::Borrowed(&self.text))
        } else if row == 1 {
            Ok(Cow::Borrowed(""))
        } else {
            Err(TextError::LineIndexOutOfBounds(row, 1))
        }
    }

    /// Iterate over text-lines, starting at line-offset.
    ///
    /// * row must be < len_lines
    #[inline]
    fn lines_at(&self, row: upos_type) -> Result<impl Iterator<Item = Cow<'_, str>>, TextError> {
        if row == 0 {
            Ok(once(Cow::Borrowed(self.text.as_str())))
        } else if row == 1 {
            Ok(once(Cow::Borrowed("")))
        } else {
            Err(TextError::LineIndexOutOfBounds(row, 1))
        }
    }

    /// Return a line as an iterator over the graphemes.
    /// This contains the '\n' at the end.
    ///
    /// * row must be < len_lines
    #[inline]
    fn line_graphemes(&self, row: upos_type) -> Result<Self::GraphemeIter<'_>, TextError> {
        if row == 0 {
            Ok(StrGraphemes::new(0, &self.text))
        } else if row == 1 {
            Ok(StrGraphemes::new(self.text.len(), ""))
        } else {
            Err(TextError::LineIndexOutOfBounds(row, 1))
        }
    }

    /// Line width of row as grapheme count.
    /// Excludes the terminating '\n'.
    ///
    /// * row must be < len_lines
    #[inline]
    fn line_width(&self, row: upos_type) -> Result<upos_type, TextError> {
        if row == 0 {
            Ok(self.len)
        } else if row == 1 {
            Ok(0)
        } else {
            Err(TextError::LineIndexOutOfBounds(row, 1))
        }
    }

    /// Insert a char at the given position.
    ///
    /// * range must be a valid range. row < len_lines, col <= line_width of the row.
    fn insert_char(
        &mut self,
        mut pos: TextPosition,
        c: char,
    ) -> Result<(TextRange, Range<usize>), TextError> {
        if pos == TextPosition::new(0, 1) {
            pos = TextPosition::new(self.len, 0);
        }

        if pos.y != 0 {
            return Err(TextError::TextPositionOutOfBounds(pos));
        }

        let byte_pos = self.byte_range_at(pos)?;
        let (before, after) = self.text.split_at(byte_pos.start);

        self.set_min_changed(byte_pos.start);

        let old_len = self.len;
        self.buf.clear();
        self.buf.push_str(before);
        self.buf.push(c);
        self.buf.push_str(after);

        let before_bytes = before.len();
        let new_len = str_len(&self.buf);

        mem::swap(&mut self.text, &mut self.buf);
        self.len = new_len;

        Ok((
            TextRange::new((pos.x, 0), (pos.x + (new_len - old_len), 0)),
            before_bytes..before_bytes + c.len_utf8(),
        ))
    }

    /// Insert a str at position.
    fn insert_str(
        &mut self,
        mut pos: TextPosition,
        t: &str,
    ) -> Result<(TextRange, Range<usize>), TextError> {
        if pos == TextPosition::new(0, 1) {
            pos = TextPosition::new(self.len, 0);
        }

        if pos.y != 0 {
            return Err(TextError::TextPositionOutOfBounds(pos));
        }

        let byte_pos = self.byte_range_at(pos)?;
        let (before, after) = self.text.split_at(byte_pos.start);

        self.set_min_changed(byte_pos.start);

        let old_len = self.len;
        self.buf.clear();
        self.buf.push_str(before);
        self.buf.push_str(t);
        self.buf.push_str(after);

        let before_bytes = before.len();
        let new_len = str_len(&self.buf);

        mem::swap(&mut self.text, &mut self.buf);
        self.len = new_len;

        Ok((
            TextRange::new((pos.x, 0), (pos.x + (new_len - old_len), 0)),
            before_bytes..before_bytes + t.len(),
        ))
    }

    /// Remove a range.
    fn remove(
        &mut self,
        mut range: TextRange,
    ) -> Result<(String, (TextRange, Range<usize>)), TextError> {
        if range.start == TextPosition::new(0, 1) {
            range.start = TextPosition::new(self.len, 0);
        }
        if range.end == TextPosition::new(0, 1) {
            range.end = TextPosition::new(self.len, 0);
        }

        if range.start.y != 0 {
            return Err(TextError::TextRangeOutOfBounds(range));
        }
        if range.end.y != 0 {
            return Err(TextError::TextRangeOutOfBounds(range));
        }

        let bytes = self.byte_range(range)?;

        self.set_min_changed(bytes.start);

        let (before, remove, after) = (
            &self.text[..bytes.start],
            &self.text[bytes.start..bytes.end],
            &self.text[bytes.end..],
        );

        self.buf.clear();
        self.buf.push_str(before);
        self.buf.push_str(after);

        let remove_str = remove.to_string();
        let before_bytes = before.len();
        let remove_bytes = remove.len();
        let new_len = str_len(&self.buf);

        mem::swap(&mut self.text, &mut self.buf);
        self.len = new_len;

        Ok((
            remove_str,
            (range, before_bytes..before_bytes + remove_bytes),
        ))
    }

    /// Insert a string at the given byte index.
    fn insert_b(&mut self, byte_pos: usize, t: &str) -> Result<(), TextError> {
        let Some((before, after)) = self.text.split_at_checked(byte_pos) else {
            return Err(TextError::ByteIndexNotCharBoundary(byte_pos));
        };

        self.set_min_changed(byte_pos);

        self.buf.clear();
        self.buf.push_str(before);
        self.buf.push_str(t);
        self.buf.push_str(after);
        let new_len = str_len(&self.buf);

        mem::swap(&mut self.text, &mut self.buf);
        self.len = new_len;

        Ok(())
    }

    /// Remove the given byte-range.
    fn remove_b(&mut self, byte_range: Range<usize>) -> Result<(), TextError> {
        let Some((before, after)) = self.text.split_at_checked(byte_range.start) else {
            return Err(TextError::ByteIndexNotCharBoundary(byte_range.start));
        };
        let Some((_remove, after)) = after.split_at_checked(byte_range.end - byte_range.start)
        else {
            return Err(TextError::ByteIndexNotCharBoundary(byte_range.end));
        };

        self.set_min_changed(byte_range.start);

        self.buf.clear();
        self.buf.push_str(before);
        self.buf.push_str(after);
        let new_len = str_len(&self.buf);

        mem::swap(&mut self.text, &mut self.buf);
        self.len = new_len;

        Ok(())
    }
}
