use crate::{upos_type, Grapheme, TextError, TextPosition, TextRange};
use std::borrow::Cow;
use std::ops::Range;

/// Trait for a cursor.
///
/// This is not a [DoubleEndedIterator] which can iterate from both ends of
/// the iterator, but moves a cursor forward/back over the collection.
pub trait Cursor: Iterator {
    /// Return the previous item.
    fn prev(&mut self) -> Option<Self::Item>;
}

/// Backing store for the TextCore.
pub trait TextStore {
    /// Can store multi-line content?
    fn is_multi_line(&self) -> bool;

    /// Get content as string.
    fn string(&self) -> String;

    /// Set content from string.
    fn set_string(&mut self, t: &str);

    /// Grapheme position to byte position.
    /// This is the (start,end) position of the single grapheme after pos.
    fn byte_range_at(&self, pos: TextPosition) -> Result<Range<usize>, TextError>;

    /// Grapheme range to byte range.
    fn byte_range(&self, range: TextRange) -> Result<Range<usize>, TextError>;

    /// Byte position to grapheme position.
    /// Returns the position that contains the given byte index.
    fn byte_to_pos(&self, byte: usize) -> Result<TextPosition, TextError>;

    /// Byte range to grapheme range.
    fn bytes_to_range(&self, bytes: Range<usize>) -> Result<TextRange, TextError>;

    /// A range of the text as Cow<str>.
    ///
    /// If the range is long this will do extensive copying.
    fn str_slice(&self, range: TextRange) -> Result<Cow<'_, str>, TextError>;

    /// Line as str.
    fn line_at(&self, row: upos_type) -> Result<Cow<'_, str>, TextError>;

    /// Iterate over text-lines, starting at line-offset.
    fn lines_at(&self, row: upos_type) -> Result<impl Iterator<Item = Cow<'_, str>>, TextError>;

    /// Return a cursor over the graphemes at the given position.
    fn graphemes(
        &self,
        pos: TextPosition,
    ) -> Result<impl Iterator<Item = Grapheme<'_>> + Cursor, TextError>;

    /// Return a line as an iterator over the graphemes.
    /// This contains the '\n' at the end.
    fn line_graphemes(
        &self,
        row: upos_type,
    ) -> Result<impl Iterator<Item = Grapheme<'_>>, TextError>;

    /// Line width of row as grapheme count.
    fn line_width(&self, row: upos_type) -> Result<upos_type, TextError>;

    /// Number of lines.
    fn len_lines(&self) -> upos_type;

    /// Insert a char at the given position.
    fn insert_char(
        &mut self,
        pos: TextPosition,
        c: char,
    ) -> Result<(TextRange, Range<usize>), TextError>;

    /// Insert a text str at the given position.
    fn insert_str(
        &mut self,
        pos: TextPosition,
        t: &str,
    ) -> Result<(TextRange, Range<usize>), TextError>;

    /// Remove the given text range.
    /// Returns the byte-range removed.
    fn remove(
        &mut self,
        range: TextRange,
    ) -> Result<(String, (TextRange, Range<usize>)), TextError>;

    /// Insert a string at the given byte index.
    fn insert_b(&mut self, byte_pos: usize, t: &str) -> Result<(), TextError>;

    /// Remove the given byte-range.
    fn remove_b(&mut self, byte_range: Range<usize>) -> Result<(), TextError>;
}

pub mod text_rope {
    use crate::grapheme::{rope_line_len, str_line_len, RopeGraphemesIdx};
    use crate::text_store::{Cursor, TextStore};
    use crate::{upos_type, Grapheme, TextError, TextPosition, TextRange};
    use ropey::{Rope, RopeSlice};
    use std::borrow::Cow;
    use std::mem;
    use std::ops::Range;

    /// Text store with a rope.
    #[derive(Debug, Clone, Default)]
    pub struct TextRope {
        text: Rope,
        // tmp buf
        buf: String,
    }

    impl TextRope {
        /// Returns the first char position for the grapheme position.
        #[inline]
        fn char_at(&self, pos: TextPosition) -> Result<usize, TextError> {
            let byte_range = self.byte_range_at(pos)?;
            Ok(self
                .text
                .try_byte_to_char(byte_range.start)
                .expect("valid_bytes"))
        }

        /// Iterator for the chars of a given line.
        #[inline]
        fn line_chars(&self, row: upos_type) -> Result<impl Iterator<Item = char> + '_, TextError> {
            let Some(line) = self.text.get_line(row as usize) else {
                return Err(TextError::LineIndexOutOfBounds(
                    row,
                    self.text.len_lines() as upos_type,
                ));
            };
            Ok(line.chars())
        }
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
                buf: Default::default(),
            }
        }

        /// New from rope.
        pub fn new_rope(r: Rope) -> Self {
            Self {
                text: r,
                buf: Default::default(),
            }
        }

        /// Borrow the rope
        pub fn rope(&self) -> &Rope {
            &self.text
        }

        /// A range of the text as RopeSlice.
        #[inline]
        pub fn rope_slice(&self, range: TextRange) -> Result<RopeSlice<'_>, TextError> {
            let s = self.char_at(range.start)?;
            let e = self.char_at(range.end)?;
            Ok(self.text.get_slice(s..e).expect("valid_range"))
        }
    }

    impl TextStore for TextRope {
        /// Can store line-breaks.
        fn is_multi_line(&self) -> bool {
            true
        }

        /// Content as string.
        fn string(&self) -> String {
            self.text.to_string()
        }

        /// Set content.
        fn set_string(&mut self, t: &str) {
            self.text = Rope::from_str(t);
        }

        /// Grapheme position to byte position.
        /// This is the (start,end) position of the single grapheme after pos.
        fn byte_range_at(&self, pos: TextPosition) -> Result<Range<usize>, TextError> {
            let it_line = self.line_graphemes(pos.y)?;

            let mut col = 0;
            let mut byte_end = 0;
            for grapheme in it_line {
                if col == pos.x {
                    return Ok(grapheme.bytes);
                }
                col += 1;
                byte_end = grapheme.bytes.end;
            }
            // one past the end is ok.
            if col == pos.x {
                return Ok(byte_end..byte_end);
            } else {
                return Err(TextError::ColumnIndexOutOfBounds(pos.x, col));
            }
        }

        /// Grapheme range to byte range.
        fn byte_range(&self, range: TextRange) -> Result<Range<usize>, TextError> {
            if range.start.y == range.end.y {
                let it_line = self.line_graphemes(range.start.y)?;

                let mut col = 0;
                let mut range_start = None;
                let mut range_end = None;
                let mut byte_end = 0;
                for grapheme in it_line {
                    if col == range.start.x {
                        range_start = Some(grapheme.bytes.start);
                    }
                    if col == range.end.x {
                        range_end = Some(grapheme.bytes.end);
                    }
                    if range_start.is_some() && range_end.is_some() {
                        break;
                    }
                    col += 1;
                    byte_end = grapheme.bytes.end;
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
                if grapheme.bytes.start >= byte_pos {
                    break;
                }
                col += 1;
            }

            Ok(TextPosition::new(col, row))
        }

        /// Byte range to grapheme range.
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
                    if grapheme.bytes.start >= bytes.start {
                        if start == None {
                            start = Some(col);
                        }
                    }
                    if grapheme.bytes.end >= bytes.start {
                        if start == None {
                            end = Some(col);
                        }
                    }
                    if start.is_some() && end.is_some() {
                        break;
                    }
                    col += 1;
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

        /// A range of the text as Cow<str>
        fn str_slice(&self, range: TextRange) -> Result<Cow<'_, str>, TextError> {
            let start_char = self.char_at(range.start)?;
            let end_char = self.char_at(range.end)?;
            let v = self
                .text
                .get_slice(start_char..end_char)
                .expect("valid_range");
            match v.as_str() {
                Some(v) => Ok(Cow::Borrowed(v)),
                None => Ok(Cow::Owned(v.to_string())),
            }
        }

        fn line_at(&self, row: upos_type) -> Result<Cow<'_, str>, TextError> {
            let Some(v) = self.text.get_line(row as usize) else {
                return Err(TextError::LineIndexOutOfBounds(row, self.len_lines()));
            };
            match v.as_str() {
                Some(v) => Ok(Cow::Borrowed(v)),
                None => Ok(Cow::Owned(v.to_string())),
            }
        }

        /// Iterate over text-lines, starting at line-offset.
        fn lines_at(
            &self,
            row: upos_type,
        ) -> Result<impl Iterator<Item = Cow<'_, str>>, TextError> {
            let Some(it) = self.text.get_lines_at(row as usize) else {
                return Err(TextError::LineIndexOutOfBounds(row, self.len_lines()));
            };
            Ok(it.map(|v| match v.as_str() {
                Some(v) => Cow::Borrowed(v),
                None => Cow::Owned(v.to_string()),
            }))
        }

        fn graphemes(
            &self,
            pos: TextPosition,
        ) -> Result<impl Iterator<Item = Grapheme<'_>> + Cursor, TextError> {
            let pos_byte = self.byte_range_at(pos)?.start;
            let s = self.text.get_slice(..).expect("no_bounds_are_ok");
            Ok(RopeGraphemesIdx::new_offset(s, pos_byte).expect("valid_bytes"))
        }

        /// Line as grapheme iterator.
        #[inline]
        fn line_graphemes(
            &self,
            row: upos_type,
        ) -> Result<impl Iterator<Item = Grapheme<'_>>, TextError> {
            let Some(v) = self.text.get_line(row as usize) else {
                return Err(TextError::LineIndexOutOfBounds(row, self.len_lines()));
            };
            Ok(RopeGraphemesIdx::new(v))
        }

        /// Line width as grapheme count. Excludes the terminating '\n'.
        #[inline]
        fn line_width(&self, row: upos_type) -> Result<upos_type, TextError> {
            let Some(v) = self.text.get_line(row as usize) else {
                return Err(TextError::LineIndexOutOfBounds(row, self.len_lines()));
            };
            Ok(rope_line_len(v) as upos_type)
        }

        fn len_lines(&self) -> upos_type {
            self.text.len_lines() as upos_type
        }

        fn insert_char(
            &mut self,
            pos: TextPosition,
            ch: char,
        ) -> Result<(TextRange, Range<usize>), TextError> {
            let pos_byte = self.byte_range_at(pos)?;
            let pos_char = self
                .text
                .try_byte_to_char(pos_byte.start)
                .expect("valid_bytes");

            let mut line_count = 0;
            if ch == '\n' {
                line_count = 1;
            }

            let insert_range = if line_count > 0 {
                self.text
                    .try_insert_char(pos_char, ch)
                    .expect("valid_chars");

                TextRange::new(pos, (0, pos.y + line_count))
            } else {
                // no way to know if the new char combines with a surrounding char.
                // the difference of the graphem len seems safe though.
                let old_len = self.line_width(pos.y).expect("valid_line");
                self.text
                    .try_insert_char(pos_char, ch)
                    .expect("valid_chars");
                let new_len = self.line_width(pos.y).expect("valid_line");

                TextRange::new(pos, (pos.x + new_len - old_len, pos.y))
            };

            Ok((insert_range, pos_byte.start..pos_byte.start + ch.len_utf8()))
        }

        fn insert_str(
            &mut self,
            pos: TextPosition,
            txt: &str,
        ) -> Result<(TextRange, Range<usize>), TextError> {
            let pos_byte = self.byte_range_at(pos)?;
            let pos_char = self
                .text
                .try_byte_to_char(pos_byte.start)
                .expect("valid_bytes");

            let mut line_count = 0;
            let mut last_linebreak_idx = 0;
            for (p, c) in txt.char_indices() {
                if c == '\n' {
                    line_count += 1;
                    last_linebreak_idx = p + 1;
                }
            }

            let insert_range = if line_count > 0 {
                let mut buf = mem::take(&mut self.buf);

                // Find the length of line after the insert position.
                let split = self.char_at(pos).expect("valid_pos");
                let line = self.line_chars(pos.y).expect("valid_pos");
                buf.clear();
                for c in line.skip(split) {
                    buf.push(c);
                }
                let old_len = str_line_len(&buf) as upos_type;
                buf.clear();

                // compose the new line and find its length.
                buf.push_str(&txt[last_linebreak_idx..]);
                let line = self.line_chars(pos.y).expect("valid_pos");
                for c in line.skip(split) {
                    buf.push(c);
                }
                let new_len = str_line_len(&buf) as upos_type;
                buf.clear();
                self.buf = buf;

                self.text.try_insert(pos_char, txt).expect("valid_pos");

                TextRange::new(pos, (new_len - old_len, pos.y + line_count))
            } else {
                // no way to know if the insert text combines with a surrounding char.
                // the difference of the graphem len seems safe though.
                let old_len = self.line_width(pos.y).expect("valid_line");

                self.text.try_insert(pos_char, txt).expect("valid_pos");

                let new_len = self.line_width(pos.y).expect("valid_line");

                TextRange::new(pos, (pos.x + new_len - old_len, pos.y))
            };

            Ok((insert_range, pos_byte.start..pos_byte.start + txt.len()))
        }

        fn remove(
            &mut self,
            range: TextRange,
        ) -> Result<(String, (TextRange, Range<usize>)), TextError> {
            let start_byte_pos = self.byte_range_at(range.start)?;
            let end_byte_pos = self.byte_range_at(range.end)?;

            let start_pos = self
                .text
                .try_byte_to_char(start_byte_pos.start)
                .expect("valid_bytes");
            let end_pos = self
                .text
                .try_byte_to_char(end_byte_pos.start)
                .expect("valid_bytes");

            let old_text = self
                .text
                .get_slice(start_pos..end_pos)
                .expect("valid_bytes");
            let old_text = old_text.to_string();

            self.text.try_remove(start_pos..end_pos).expect("valid_pos");

            Ok((old_text, (range, start_byte_pos.start..end_byte_pos.start)))
        }

        fn insert_b(&mut self, byte_pos: usize, t: &str) -> Result<(), TextError> {
            let pos_char = self.text.try_byte_to_char(byte_pos)?;
            self.text.try_insert(pos_char, t).expect("valid_pos");
            Ok(())
        }

        fn remove_b(&mut self, byte_range: Range<usize>) -> Result<(), TextError> {
            let start_char = self.text.try_byte_to_char(byte_range.start)?;
            let end_char = self.text.try_byte_to_char(byte_range.end)?;
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
}

pub mod text_string {
    use crate::grapheme::{str_line_len, StrGraphemesIdx};
    use crate::text_store::{Cursor, TextStore};
    use crate::{upos_type, Grapheme, TextError, TextPosition, TextRange};
    use std::borrow::Cow;
    use std::iter::once;
    use std::mem;
    use std::ops::Range;
    use unicode_segmentation::UnicodeSegmentation;

    #[derive(Debug, Default, Clone)]
    pub struct TextString {
        // text
        text: String,
        // len as grapheme count
        len: upos_type,
        // tmp buffer
        buf: String,
    }

    impl TextString {
        /// New empty.
        pub fn new() -> Self {
            Self {
                text: Default::default(),
                len: 0,
                buf: Default::default(),
            }
        }

        /// New from string.
        pub fn new_text(t: &str) -> Result<Self, TextError> {
            if t.contains(|c| c == '\n' || c == '\r') {
                return Err(TextError::InvalidText(t.to_string()));
            }

            Ok(Self {
                text: t.into(),
                len: str_line_len(t) as upos_type,
                buf: Default::default(),
            })
        }

        /// New from string.
        pub fn new_string(t: String) -> Result<Self, TextError> {
            if t.contains(|c| c == '\n' || c == '\r') {
                return Err(TextError::InvalidText(t));
            }

            let len = str_line_len(&t) as upos_type;
            Ok(Self {
                text: t,
                len,
                buf: Default::default(),
            })
        }
    }

    impl TextStore for TextString {
        /// Can store line-breaks.
        fn is_multi_line(&self) -> bool {
            false
        }

        fn string(&self) -> String {
            self.text.to_string()
        }

        fn set_string(&mut self, t: &str) {
            self.text = t.to_string();
            self.len = str_line_len(&self.text) as upos_type;
        }

        fn byte_range_at(&self, pos: TextPosition) -> Result<Range<usize>, TextError> {
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
                    str_line_len(&self.text) as upos_type,
                ))
            }
        }

        fn byte_range(&self, range: TextRange) -> Result<Range<usize>, TextError> {
            if range.start.y != 0 {
                return Err(TextError::LineIndexOutOfBounds(range.start.y, 1));
            };
            if range.end.y != 0 {
                return Err(TextError::LineIndexOutOfBounds(range.end.y, 1));
            };

            let mut byte_start = None;
            let mut byte_end = None;
            for (cidx, (idx, _)) in self
                .text
                .grapheme_indices(true)
                .chain(once((self.text.len(), "")))
                .enumerate()
            {
                if cidx == range.start.x as usize {
                    byte_start = Some(idx);
                }
                if cidx == range.end.x as usize {
                    byte_end = Some(idx);
                }
                if byte_start.is_some() && byte_end.is_some() {
                    break;
                }
            }

            let Some(byte_start) = byte_start else {
                return Err(TextError::ColumnIndexOutOfBounds(
                    range.start.x,
                    str_line_len(&self.text) as upos_type,
                ));
            };
            let Some(byte_end) = byte_end else {
                return Err(TextError::ColumnIndexOutOfBounds(
                    range.end.x,
                    str_line_len(&self.text) as upos_type,
                ));
            };

            Ok(byte_start..byte_end)
        }

        fn byte_to_pos(&self, byte_pos: usize) -> Result<TextPosition, TextError> {
            let mut pos = None;

            for (cidx, (idx, _c)) in self
                .text
                .grapheme_indices(true)
                .chain(once((self.text.len(), "")))
                .enumerate()
            {
                if idx >= byte_pos {
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
        fn bytes_to_range(&self, bytes: Range<usize>) -> Result<TextRange, TextError> {
            let mut start = None;
            let mut end = None;
            for (cidx, (idx, _c)) in self
                .text
                .grapheme_indices(true)
                .chain(once((self.text.len(), "")))
                .enumerate()
            {
                if idx >= bytes.start {
                    if start.is_none() {
                        start = Some(cidx as upos_type);
                    }
                }
                if idx >= bytes.end {
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

        /// A range of the text as Cow<str>
        fn str_slice(&self, range: TextRange) -> Result<Cow<'_, str>, TextError> {
            let range = self.byte_range(range)?;
            Ok(Cow::Borrowed(&self.text[range.start..range.end]))
        }

        /// Line as str.
        fn line_at(&self, row: upos_type) -> Result<Cow<'_, str>, TextError> {
            if row == 0 {
                Ok(Cow::Borrowed(&self.text))
            } else {
                Err(TextError::LineIndexOutOfBounds(row, 1))
            }
        }

        fn lines_at(
            &self,
            row: upos_type,
        ) -> Result<impl Iterator<Item = Cow<'_, str>>, TextError> {
            if row == 0 {
                Ok(once(Cow::Borrowed(self.text.as_str())))
            } else {
                Err(TextError::LineIndexOutOfBounds(row, 1))
            }
        }

        fn graphemes(
            &self,
            pos: TextPosition,
        ) -> Result<impl Iterator<Item = Grapheme<'_>> + Cursor, TextError> {
            let pos_byte = self.byte_range_at(pos)?;
            Ok(StrGraphemesIdx::new_offset(&self.text, pos_byte.start))
        }

        fn line_graphemes(
            &self,
            row: upos_type,
        ) -> Result<impl Iterator<Item = Grapheme<'_>>, TextError> {
            if row == 0 {
                Ok(StrGraphemesIdx::new(&self.text))
            } else {
                Err(TextError::LineIndexOutOfBounds(row, 1))
            }
        }

        /// Line width as grapheme count.
        fn line_width(&self, row: upos_type) -> Result<upos_type, TextError> {
            if row == 0 {
                Ok(self.len)
            } else {
                Err(TextError::LineIndexOutOfBounds(row, 1))
            }
        }

        /// Number of lines.
        fn len_lines(&self) -> upos_type {
            1
        }

        /// Insert a char at position.
        fn insert_char(
            &mut self,
            pos: TextPosition,
            c: char,
        ) -> Result<(TextRange, Range<usize>), TextError> {
            if pos.y != 0 {
                return Err(TextError::TextPositionOutOfBounds(pos));
            }
            if c == '\n' || c == '\r' {
                return Err(TextError::InvalidText(c.to_string()));
            }

            let byte_pos = self.byte_range_at(pos)?;
            let (before, after) = self.text.split_at(byte_pos.start);

            let old_len = self.len;
            self.buf.clear();
            self.buf.push_str(before);
            self.buf.push(c);
            self.buf.push_str(after);

            let before_bytes = before.len();
            let new_len = str_line_len(&self.buf) as upos_type;

            mem::swap(&mut self.text, &mut self.buf);
            self.len = new_len;

            Ok((
                TextRange::new((0, pos.x), (0, pos.x + (new_len - old_len))),
                before_bytes..before_bytes + c.len_utf8(),
            ))
        }

        /// Insert a str at position.
        fn insert_str(
            &mut self,
            pos: TextPosition,
            t: &str,
        ) -> Result<(TextRange, Range<usize>), TextError> {
            if pos.y != 0 {
                return Err(TextError::TextPositionOutOfBounds(pos));
            }
            if t.contains(|c| c == '\n' || c == '\r') {
                return Err(TextError::InvalidText(t.to_string()));
            }

            let byte_pos = self.byte_range_at(pos)?;
            let (before, after) = self.text.split_at(byte_pos.start);

            let old_len = self.len;
            self.buf.clear();
            self.buf.push_str(before);
            self.buf.push_str(t);
            self.buf.push_str(after);

            let before_bytes = before.len();
            let new_len = str_line_len(&self.buf) as upos_type;

            mem::swap(&mut self.text, &mut self.buf);
            self.len = new_len;

            Ok((
                TextRange::new((0, pos.x), (0, pos.x + (new_len - old_len))),
                before_bytes..before_bytes + t.len(),
            ))
        }

        /// Remove a range.
        fn remove(
            &mut self,
            range: TextRange,
        ) -> Result<(String, (TextRange, Range<usize>)), TextError> {
            if range.start.y != 0 || range.end.y != 0 {
                return Err(TextError::TextRangeOutOfBounds(range));
            }

            let bytes = self.byte_range(range)?;

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
            let new_len = str_line_len(&self.buf) as upos_type;

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

            self.buf.clear();
            self.buf.push_str(before);
            self.buf.push_str(t);
            self.buf.push_str(after);
            let new_len = str_line_len(&self.buf) as upos_type;

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

            self.buf.clear();
            self.buf.push_str(before);
            self.buf.push_str(after);
            let new_len = str_line_len(&self.buf) as upos_type;

            mem::swap(&mut self.text, &mut self.buf);
            self.len = new_len;

            Ok(())
        }
    }
}
