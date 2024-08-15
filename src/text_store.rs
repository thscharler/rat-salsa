use crate::grapheme::Grapheme;
use crate::{upos_type, TextError, TextPosition, TextRange};
use std::borrow::Cow;
use std::ops::Range;

/// Returns three kinds of ranges.
/// Used as Result for all text editing functions.
#[derive(Debug)]
pub struct StoreRange {
    pub range: TextRange,
    pub bytes: Range<usize>,
}

/// Backing store for the TextCore.
pub trait TextStore {
    /// Can store multi-line content?
    fn is_multi_line(&self) -> bool;

    /// Get content as string.
    fn string(&self) -> String;

    /// Set content from string.
    fn set_string(&mut self, t: &str);

    /// Is empty?
    fn is_empty(&self) -> bool;

    /// Grapheme position to byte position.
    /// This is the (start,end) position of the single grapheme after pos.
    fn byte_at(&self, pos: TextPosition) -> Result<Range<usize>, TextError>;

    /// Byte position to grapheme position.
    /// Returns the position that contains the given byte index.
    fn byte_pos(&self, byte: usize) -> Result<TextPosition, TextError>;

    /// Line as str
    fn line_at(&self, row: upos_type) -> Option<Cow<'_, str>>;

    /// Iterate over text-lines, starting at line-offset.
    fn lines_at(&self, row: upos_type) -> impl Iterator<Item = Cow<'_, str>>;

    /// Return a line as an iterator over the graphemes.
    /// This contains the '\n' at the end.
    fn line_graphemes(&self, row: upos_type) -> Option<impl Iterator<Item = Grapheme<'_>>>;

    /// Line width of row as grapheme count.
    fn line_width(&self, row: upos_type) -> Option<upos_type>;

    /// Number of lines.
    fn len_lines(&self) -> upos_type;

    /// Insert a char at the given position.
    fn insert_char(&mut self, pos: TextPosition, c: char) -> Result<StoreRange, TextError>;

    /// Insert a text str at the given position.
    fn insert_str(&mut self, pos: TextPosition, t: &str) -> Result<StoreRange, TextError>;

    /// Remove the given text range.
    /// Returns the byte-range removed.
    fn remove(&mut self, range: TextRange) -> Result<(String, StoreRange), TextError>;

    /// Insert a char at the given byte index.
    fn insert_char_b(&mut self, byte_pos: usize, c: char) -> Result<(), TextError>;

    /// Insert a string at the given byte index.
    fn insert_str_b(&mut self, byte_pos: usize, t: &str) -> Result<(), TextError>;

    /// Remove the given byte-range.
    fn remove_b(&mut self, byte_range: Range<usize>) -> Result<(), TextError>;
}

pub mod text_rope {
    use crate::grapheme::{rope_line_len, str_line_len, Grapheme, RopeGraphemesIdx};
    use crate::text_store::{StoreRange, TextStore};
    use crate::{upos_type, TextError, TextPosition, TextRange};
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
            let byte_range = self.byte_at(pos)?;
            Ok(self.text.try_byte_to_char(byte_range.start)?)
        }

        /// Iterator for the chars of a given line.
        #[inline]
        fn line_chars(&self, row: upos_type) -> Result<impl Iterator<Item = char> + '_, TextError> {
            let Some(mut lines) = self.text.get_lines_at(row as usize) else {
                return Err(TextError::LineIndexOutOfBounds(
                    row,
                    self.text.len_lines() as upos_type,
                ));
            };

            if let Some(line) = lines.next() {
                Ok(line.chars())
            } else {
                Ok(RopeSlice::from("").chars())
            }
        }

        /// A range of the text as RopeSlice.
        #[inline]
        fn rope_slice(&self, range: TextRange) -> Result<RopeSlice<'_>, TextError> {
            let s = self.char_at(range.start)?;
            let e = self.char_at(range.end)?;
            let Some(slice) = self.text.get_slice(s..e) else {
                return Err(TextError::TextRangeOutOfBounds(range));
            };
            Ok(slice)
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
    }

    impl TextStore for TextRope {
        /// Can store line-breaks.
        fn is_multi_line(&self) -> bool {
            true
        }

        fn string(&self) -> String {
            self.text.to_string()
        }

        fn set_string(&mut self, t: &str) {
            self.text = Rope::from_str(t);
        }

        /// Is empty?
        fn is_empty(&self) -> bool {
            self.text.len_bytes() == 0
        }

        /// Grapheme position to byte position.
        /// This is the (start,end) position of the single grapheme after pos.
        fn byte_at(&self, pos: TextPosition) -> Result<Range<usize>, TextError> {
            let Some(it_line) = self.line_graphemes(pos.y) else {
                return Err(TextError::LineIndexOutOfBounds(pos.y, self.len_lines()));
            };

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

        /// Byte position to grapheme position.
        /// Returns the position that contains the given byte index.
        fn byte_pos(&self, byte_pos: usize) -> Result<TextPosition, TextError> {
            let Ok(y) = self.text.try_byte_to_line(byte_pos) else {
                return Err(TextError::ByteIndexOutOfBounds(
                    byte_pos,
                    self.text.len_bytes(),
                ));
            };

            let Some(it_line) = self.line_graphemes(y as upos_type) else {
                return Err(TextError::LineIndexOutOfBounds(
                    y as upos_type,
                    self.len_lines(),
                ));
            };

            let mut col = 0;
            for grapheme in it_line {
                if grapheme.bytes.start >= byte_pos {
                    break;
                }
                col += 1;
            }

            Ok(TextPosition::new(col, y as upos_type))
        }

        fn line_at(&self, row: upos_type) -> Option<Cow<'_, str>> {
            let v = self.text.get_line(row as usize)?;
            match v.as_str() {
                Some(v) => Some(Cow::Borrowed(v)),
                None => Some(Cow::Owned(v.to_string())),
            }
        }

        fn lines_at(&self, row: upos_type) -> impl Iterator<Item = Cow<'_, str>> {
            self.text.lines_at(row as usize).map(|v| match v.as_str() {
                Some(v) => Cow::Borrowed(v),
                None => Cow::Owned(v.to_string()),
            })
        }

        /// Line as grapheme iterator.
        #[inline]
        fn line_graphemes(&self, row: upos_type) -> Option<impl Iterator<Item = Grapheme<'_>>> {
            let line = self.text.get_line(row as usize)?;
            Some(RopeGraphemesIdx::new(line))
        }

        /// Line width as grapheme count. Excludes the terminating '\n'.
        #[inline]
        fn line_width(&self, row: upos_type) -> Option<upos_type> {
            let line = self.text.get_line(row as usize)?;
            Some(rope_line_len(line) as upos_type)
        }

        fn len_lines(&self) -> upos_type {
            self.text.len_lines() as upos_type
        }

        fn insert_char(&mut self, pos: TextPosition, c: char) -> Result<StoreRange, TextError> {
            let byte_pos = self.byte_at(pos)?;
            let char_pos = self.text.try_byte_to_char(byte_pos.start)?;

            let mut line_count = 0;
            if c == '\n' {
                line_count = 1;
            }

            let insert_range = if line_count > 0 {
                self.text.try_insert_char(char_pos, c)?;

                TextRange::new(pos, (0, pos.y + line_count))
            } else {
                // no way to know if the new char combines with a surrounding char.
                // the difference of the graphem len seems safe though.
                let old_len = self.line_width(pos.y).expect("valid_line");
                self.text.try_insert_char(char_pos, c)?;
                let new_len = self.line_width(pos.y).expect("valid_line");

                TextRange::new(pos, (pos.x + new_len - old_len, pos.y))
            };

            Ok(StoreRange {
                range: insert_range,
                bytes: byte_pos.start..byte_pos.start + c.len_utf8(),
            })
        }

        fn insert_str(&mut self, pos: TextPosition, t: &str) -> Result<StoreRange, TextError> {
            let byte_pos = self.byte_at(pos)?;
            let char_pos = self.text.try_byte_to_char(byte_pos.start)?;

            let mut line_count = 0;
            let mut last_linebreak_idx = 0;
            for (p, c) in t.char_indices() {
                if c == '\n' {
                    line_count += 1;
                    last_linebreak_idx = p + 1;
                }
            }

            let insert_range = if line_count > 0 {
                let mut buf = mem::take(&mut self.buf);

                // Find the length of line after the insert position.
                let split = self.char_at(pos)?;
                let line = self.line_chars(pos.y)?;
                buf.clear();
                for c in line.skip(split) {
                    buf.push(c);
                }
                let old_len = str_line_len(&buf) as upos_type;
                buf.clear();

                // compose the new line and find its length.
                buf.push_str(&t[last_linebreak_idx..]);
                let line = self.line_chars(pos.y)?;
                for c in line.skip(split) {
                    buf.push(c);
                }
                let new_len = str_line_len(&buf) as upos_type;
                buf.clear();
                self.buf = buf;

                self.text.try_insert(char_pos, t)?;

                TextRange::new(pos, (new_len - old_len, pos.y + line_count))
            } else {
                // no way to know if the insert text combines with a surrounding char.
                // the difference of the graphem len seems safe though.
                let old_len = self.line_width(pos.y).expect("valid_line");

                self.text.try_insert(char_pos, t)?;

                let new_len = self.line_width(pos.y).expect("valid_line");

                TextRange::new(pos, (pos.x + new_len - old_len, pos.y))
            };

            Ok(StoreRange {
                range: insert_range,
                bytes: byte_pos.start..byte_pos.start + t.len(),
            })
        }

        fn remove(&mut self, range: TextRange) -> Result<(String, StoreRange), TextError> {
            let start_byte_pos = self.byte_at(range.start)?;
            let end_byte_pos = self.byte_at(range.end)?;

            let start_pos = self.text.try_byte_to_char(start_byte_pos.start)?;
            let end_pos = self.text.try_byte_to_char(end_byte_pos.start)?;

            let old_text = self.rope_slice(range)?.to_string();

            self.text.try_remove(start_pos..end_pos)?;

            Ok((
                old_text,
                StoreRange {
                    range,
                    bytes: start_byte_pos.start..end_byte_pos.start,
                },
            ))
        }

        fn insert_char_b(&mut self, byte_pos: usize, c: char) -> Result<(), TextError> {
            let char_pos = self.text.try_byte_to_char(byte_pos)?;
            self.text.try_insert_char(char_pos, c)?;
            Ok(())
        }

        fn insert_str_b(&mut self, byte_pos: usize, t: &str) -> Result<(), TextError> {
            let char_pos = self.text.try_byte_to_char(byte_pos)?;
            self.text.try_insert(char_pos, t)?;
            Ok(())
        }

        fn remove_b(&mut self, byte_range: Range<usize>) -> Result<(), TextError> {
            let start_pos = self.text.try_byte_to_char(byte_range.start)?;
            let end_pos = self.text.try_byte_to_char(byte_range.end)?;
            self.text.try_remove(start_pos..end_pos)?;
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
    use crate::grapheme::{split3, str_line_len, Grapheme};
    use crate::text_store::{StoreRange, TextStore};
    use crate::{upos_type, TextError, TextPosition, TextRange};
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
        ///
        /// __Panic__
        /// Panics if the text contains line-breaks.
        pub fn new_text(t: &str) -> Self {
            assert!(!t.contains(|c| c == '\n' || c == '\r'));

            Self {
                text: t.into(),
                len: str_line_len(t) as upos_type,
                buf: Default::default(),
            }
        }

        /// New from string.
        ///
        /// __Panic__
        /// Panics if the text contains line-breaks.
        pub fn new_string(t: String) -> Self {
            assert!(!t.contains(|c| c == '\n' || c == '\r'));

            let len = str_line_len(&t) as upos_type;
            Self {
                text: t,
                len,
                buf: Default::default(),
            }
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
        }

        /// Is empty?
        fn is_empty(&self) -> bool {
            self.text.is_empty()
        }

        fn byte_at(&self, pos: TextPosition) -> Result<Range<usize>, TextError> {
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

        fn byte_pos(&self, byte_pos: usize) -> Result<TextPosition, TextError> {
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

        fn line_at(&self, row: upos_type) -> Option<Cow<'_, str>> {
            if row == 0 {
                Some(Cow::Borrowed(&self.text))
            } else {
                None
            }
        }

        fn lines_at(&self, row: upos_type) -> impl Iterator<Item = Cow<'_, str>> {
            if row == 0 {
                once(Cow::Borrowed(self.text.as_str()))
            } else {
                let mut it = once(Cow::Borrowed(""));
                it.next();
                it
            }
        }

        fn line_graphemes(&self, row: upos_type) -> Option<impl Iterator<Item = Grapheme<'_>>> {
            if row == 0 {
                Some(self.text.grapheme_indices(true).map(|(idx, gr)| Grapheme {
                    grapheme: Cow::Borrowed(gr),
                    bytes: idx..idx + gr.len(),
                }))
            } else {
                None
            }
        }

        /// Line width as grapheme count.
        fn line_width(&self, row: upos_type) -> Option<upos_type> {
            if row == 0 {
                Some(self.len)
            } else {
                None
            }
        }

        /// Number of lines.
        fn len_lines(&self) -> upos_type {
            1
        }

        /// Insert a char at position.
        fn insert_char(&mut self, pos: TextPosition, c: char) -> Result<StoreRange, TextError> {
            if pos.y != 0 {
                return Err(TextError::TextPositionOutOfBounds(pos));
            }
            if c == '\n' || c == '\r' {
                return Err(TextError::InvalidText(c.to_string()));
            }

            let byte_pos = self.byte_at(pos)?;
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

            Ok(StoreRange {
                range: TextRange::new((0, pos.x), (0, pos.x + (new_len - old_len))),
                bytes: before_bytes..before_bytes + c.len_utf8(),
            })
        }

        /// Insert a str at position.
        fn insert_str(&mut self, pos: TextPosition, t: &str) -> Result<StoreRange, TextError> {
            if pos.y != 0 {
                return Err(TextError::TextPositionOutOfBounds(pos));
            }
            if t.contains(|c| c == '\n' || c == '\r') {
                return Err(TextError::InvalidText(t.to_string()));
            }

            let byte_pos = self.byte_at(pos)?;
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

            Ok(StoreRange {
                range: TextRange::new((0, pos.x), (0, pos.x + (new_len - old_len))),
                bytes: before_bytes..before_bytes + t.len(),
            })
        }

        /// Remove a range.
        fn remove(&mut self, range: TextRange) -> Result<(String, StoreRange), TextError> {
            if range.start.y != 0 || range.end.y != 0 {
                return Err(TextError::TextRangeOutOfBounds(range));
            }

            let (before, remove, after) = split3(
                self.text.as_str(),
                (range.start.x as usize)..(range.end.x as usize),
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
                StoreRange {
                    range,
                    bytes: before_bytes..before_bytes + remove_bytes,
                },
            ))
        }

        /// Insert a char at the given byte index.
        fn insert_char_b(&mut self, byte_pos: usize, c: char) -> Result<(), TextError> {
            let Some((before, after)) = self.text.split_at_checked(byte_pos) else {
                return Err(TextError::ByteIndexNotCharBoundary(byte_pos));
            };

            self.buf.clear();
            self.buf.push_str(before);
            self.buf.push(c);
            self.buf.push_str(after);
            let new_len = str_line_len(&self.buf) as upos_type;

            mem::swap(&mut self.text, &mut self.buf);
            self.len = new_len;

            Ok(())
        }

        /// Insert a string at the given byte index.
        fn insert_str_b(&mut self, byte_pos: usize, t: &str) -> Result<(), TextError> {
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
