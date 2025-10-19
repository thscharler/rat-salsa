use crate::text_store::SkipLine;
use crate::{Cursor, TextError};
use ropey::RopeSlice;
use ropey::iter::Chunks;
use std::borrow::Cow;
use std::cmp;
use std::fmt::Debug;
use std::ops::Range;
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete};

/// One grapheme.
#[derive(Debug, PartialEq)]
pub struct Grapheme<'a> {
    /// grapheme
    grapheme: Cow<'a, str>,
    /// byte-range of the grapheme in the given slice.
    text_bytes: Range<usize>,
}

impl PartialEq<&str> for Grapheme<'_> {
    fn eq(&self, other: &&str) -> bool {
        self.grapheme.as_ref() == *other
    }
}

impl PartialEq<str> for Grapheme<'_> {
    fn eq(&self, other: &str) -> bool {
        self.grapheme.as_ref() == other
    }
}

impl PartialEq<&String> for Grapheme<'_> {
    fn eq(&self, other: &&String) -> bool {
        self.grapheme.as_ref() == *other
    }
}

impl PartialEq<String> for Grapheme<'_> {
    fn eq(&self, other: &String) -> bool {
        self.grapheme.as_ref() == other
    }
}

impl PartialEq<char> for Grapheme<'_> {
    fn eq(&self, other: &char) -> bool {
        let mut chars = self.grapheme.chars();
        chars.next() == Some(*other)
    }
}

impl<'a> Grapheme<'a> {
    pub fn new(grapheme: Cow<'a, str>, text_bytes: Range<usize>) -> Self {
        Self {
            grapheme,
            text_bytes,
        }
    }

    /// First (only) char of the grapheme is a whitespace.
    #[inline]
    pub fn is_whitespace(&self) -> bool {
        self.grapheme
            .chars()
            .next()
            .map(|v| v.is_whitespace())
            .unwrap_or(false)
    }

    /// First (only) char of the grapheme is a whitespace.
    #[inline]
    pub fn is_alphanumeric(&self) -> bool {
        self.grapheme
            .chars()
            .next()
            .map(|v| v.is_alphanumeric())
            .unwrap_or(false)
    }

    /// Is a linebreak.
    #[inline]
    #[allow(clippy::nonminimal_bool)]
    pub fn is_line_break(&self) -> bool {
        if cfg!(feature = "cr_lines") {
            self.grapheme == "\r" || self.grapheme == "\n" || self.grapheme == "\r\n"
        } else if cfg!(feature = "unicode_lines") {
            self.grapheme == "\r"
                || self.grapheme == "\n"
                || self.grapheme == "\r\n"
                || self.grapheme == "\u{000D}"
                || self.grapheme == "\u{000C}"
                || self.grapheme == "\u{000B}"
                || self.grapheme == "\u{0085}"
                || self.grapheme == "\u{2028}"
                || self.grapheme == "\u{2029}"
        } else {
            self.grapheme == "\n" || self.grapheme == "\r\n"
        }
    }

    /// Get the grapheme.
    #[inline]
    pub fn grapheme(&'a self) -> &'a str {
        self.grapheme.as_ref()
    }

    /// Destructure to the grapheme.
    #[inline]
    pub fn into_parts(self) -> (Cow<'a, str>, Range<usize>) {
        (self.grapheme, self.text_bytes)
    }

    /// Get the byte-range as absolute range into the complete text.
    #[inline]
    pub fn text_bytes(&self) -> Range<usize> {
        self.text_bytes.clone()
    }
}

/// A cursor over graphemes of a string.
#[derive(Debug, Clone)]
pub struct StrGraphemes<'a> {
    text_offset: usize,
    text: &'a str,
    cursor: GraphemeCursor,
}

impl<'a> StrGraphemes<'a> {
    /// Iterate the graphemes of a str-slice.
    ///
    /// * slice_offset - offset of the slice in the complete text.
    /// * slice - slice
    ///
    pub(crate) fn new(slice_offset: usize, slice: &'a str) -> Self {
        Self {
            text_offset: slice_offset,
            text: slice,
            cursor: GraphemeCursor::new(0, slice.len(), true),
        }
    }

    /// Iterate the graphemes of a str-slice.
    ///
    /// * slice_offset - offset of the slice in the complete text.
    /// * slice - slice
    /// * offset - relative offset into the slice
    ///
    pub(crate) fn new_offset(slice_offset: usize, slice: &'a str, offset: usize) -> Self {
        Self {
            text_offset: slice_offset,
            text: slice,
            cursor: GraphemeCursor::new(offset, slice.len(), true),
        }
    }
}

impl Cursor for StrGraphemes<'_> {
    fn prev(&mut self) -> Option<Self::Item> {
        let start = self.cursor.cur_cursor();
        let prev = self.cursor.prev_boundary(self.text, 0).unwrap()?;
        Some(Grapheme {
            grapheme: Cow::Borrowed(&self.text[prev..start]),
            text_bytes: self.text_offset + prev..self.text_offset + start,
        })
    }

    fn rev_cursor(self) -> impl Cursor<Item = Self::Item> {
        RevStrGraphemes { it: self }
    }

    fn text_offset(&self) -> usize {
        self.text_offset + self.cursor.cur_cursor()
    }
}

impl SkipLine for StrGraphemes<'_> {
    fn skip_line(&mut self) -> Result<(), TextError> {
        self.cursor.set_cursor(self.text.len());
        Ok(())
    }

    fn skip_to(&mut self, byte_pos: usize) -> Result<(), TextError> {
        assert!(byte_pos >= self.text_offset);
        let offset = byte_pos - self.text_offset;
        self.cursor.set_cursor(offset);
        Ok(())
    }
}

impl<'a> Iterator for StrGraphemes<'a> {
    type Item = Grapheme<'a>;

    #[inline]
    fn next(&mut self) -> Option<Grapheme<'a>> {
        let start = self.cursor.cur_cursor();
        let next = self.cursor.next_boundary(self.text, 0).unwrap()?;
        Some(Grapheme {
            grapheme: Cow::Borrowed(&self.text[start..next]),
            text_bytes: self.text_offset + start..self.text_offset + next,
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let slen = self.text.len() - self.cursor.cur_cursor();
        (cmp::min(slen, 1), Some(slen))
    }
}

#[derive(Debug)]
pub(crate) struct RevStrGraphemes<'a> {
    it: StrGraphemes<'a>,
}

impl<'a> Iterator for RevStrGraphemes<'a> {
    type Item = Grapheme<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.it.prev()
    }
}

impl Cursor for RevStrGraphemes<'_> {
    #[inline]
    fn prev(&mut self) -> Option<Self::Item> {
        self.it.next()
    }

    #[inline]
    fn rev_cursor(self) -> impl Cursor<Item = Self::Item> {
        self.it
    }

    fn text_offset(&self) -> usize {
        self.it.text_offset()
    }
}

impl SkipLine for RevStrGraphemes<'_> {
    fn skip_line(&mut self) -> Result<(), TextError> {
        unimplemented!("no skip_line()");
    }

    fn skip_to(&mut self, _byte_pos: usize) -> Result<(), TextError> {
        unimplemented!("no skip_to()");
    }
}

/// An implementation of a graphemes iterator, for iterating over
/// the graphemes of a RopeSlice.
#[derive(Debug, Clone)]
pub struct RopeGraphemes<'a> {
    text_offset: usize,
    text: RopeSlice<'a>,
    chunks: Chunks<'a>,
    was_next: Option<bool>,
    cur_chunk: &'a str,
    cur_chunk_start: usize,
    cursor: GraphemeCursor,
}

impl<'a> RopeGraphemes<'a> {
    /// New grapheme iterator.
    ///
    /// * slice_offset - offset of the slice in the complete text.
    /// * slice - slice of the complete text
    pub(crate) fn new(slice_offset: usize, slice: RopeSlice<'a>) -> RopeGraphemes<'a> {
        let mut chunks = slice.chunks();

        // was_next is only useful, if there was a true next().
        // otherwise it confuses the algorithm.
        let (first_chunk, was_next) = match chunks.next() {
            Some(v) => (v, Some(true)),
            None => ("", None),
        };

        RopeGraphemes {
            text_offset: slice_offset,
            text: slice,
            chunks,
            was_next,
            cur_chunk: first_chunk,
            cur_chunk_start: 0,
            cursor: GraphemeCursor::new(0, slice.len_bytes(), true),
        }
    }

    /// New grapheme iterator.
    ///
    /// * slice_offset - offset of the slice in the complete text.
    /// * slice - slice of the complete text
    /// * offset - relative offset into the slice
    ///
    /// Offset must be a valid char boundary.
    pub(crate) fn new_offset(
        slice_offset: usize,
        slice: RopeSlice<'a>,
        offset: usize,
    ) -> Result<RopeGraphemes<'a>, TextError> {
        let Some((mut chunks, chunk_start, _, _)) = slice.get_chunks_at_byte(offset) else {
            return Err(TextError::ByteIndexOutOfBounds(offset, slice.len_bytes()));
        };

        // was_next is only useful, if there was a true next().
        // otherwise it confuses the algorithm.
        let (first_chunk, was_next) = match chunks.next() {
            Some(v) => (v, Some(true)),
            None => ("", None),
        };

        Ok(RopeGraphemes {
            text_offset: slice_offset,
            text: slice,
            chunks,
            was_next,
            cur_chunk: first_chunk,
            cur_chunk_start: chunk_start,
            cursor: GraphemeCursor::new(offset, slice.len_bytes(), true),
        })
    }
}

impl<'a> Cursor for RopeGraphemes<'a> {
    #[inline]
    fn prev(&mut self) -> Option<Grapheme<'a>> {
        let a = self.cursor.cur_cursor();
        let b;
        loop {
            match self
                .cursor
                .prev_boundary(self.cur_chunk, self.cur_chunk_start)
            {
                Ok(None) => {
                    return None;
                }
                Ok(Some(n)) => {
                    b = n;
                    break;
                }
                Err(GraphemeIncomplete::PrevChunk) => {
                    if self.was_next == Some(true) {
                        // skip current
                        self.chunks.prev();
                    }
                    (self.cur_chunk, self.was_next) = match self.chunks.prev() {
                        Some(v) => (v, Some(false)),
                        None => ("", None),
                    };
                    self.cur_chunk_start -= self.cur_chunk.len();
                }
                Err(GraphemeIncomplete::PreContext(idx)) => {
                    let (chunk, byte_idx, _, _) = self.text.chunk_at_byte(idx.saturating_sub(1));
                    self.cursor.provide_context(chunk, byte_idx);
                }
                _ => unreachable!(),
            }
        }

        if a >= self.cur_chunk_start + self.cur_chunk.len() {
            let a_char = self.text.byte_to_char(a);
            let b_char = self.text.byte_to_char(b);

            Some(Grapheme {
                grapheme: Cow::Owned(self.text.slice(b_char..a_char).to_string()),
                text_bytes: self.text_offset + b..self.text_offset + a,
            })
        } else {
            let a2 = a - self.cur_chunk_start;
            let b2 = b - self.cur_chunk_start;
            Some(Grapheme {
                grapheme: Cow::Borrowed(&self.cur_chunk[b2..a2]),
                text_bytes: self.text_offset + b..self.text_offset + a,
            })
        }
    }

    fn rev_cursor(self) -> impl Cursor<Item = Self::Item> {
        RevRopeGraphemes { it: self }
    }

    fn text_offset(&self) -> usize {
        self.text_offset + self.cursor.cur_cursor()
    }
}

impl<'a> SkipLine for RopeGraphemes<'a> {
    fn skip_line(&mut self) -> Result<(), TextError> {
        let cursor = self.cursor.cur_cursor();
        let line = self.text.try_byte_to_line(cursor)?;
        let next_offset = self.text.try_line_to_byte(line + 1)?;

        let Some((mut chunks, chunk_start, _, _)) = self.text.get_chunks_at_byte(next_offset)
        else {
            return Err(TextError::ByteIndexOutOfBounds(
                next_offset,
                self.text.len_bytes(),
            ));
        };

        // was_next is only useful, if there was a true next().
        // otherwise it confuses the algorithm.
        let (first_chunk, _was_next) = match chunks.next() {
            Some(v) => (v, Some(true)),
            None => ("", None),
        };

        self.chunks = chunks;
        self.cur_chunk = first_chunk;
        self.cur_chunk_start = chunk_start;
        self.cursor = GraphemeCursor::new(next_offset, self.text.len_bytes(), true);

        Ok(())
    }

    fn skip_to(&mut self, byte_pos: usize) -> Result<(), TextError> {
        assert!(byte_pos >= self.text_offset);
        // byte_pos is absolute to all text, but everything here is
        // relative to the slice.
        let byte_pos = byte_pos - self.text_offset;

        let Some((mut chunks, chunk_start, _, _)) = self.text.get_chunks_at_byte(byte_pos) else {
            return Err(TextError::ByteIndexOutOfBounds(
                byte_pos,
                self.text.len_bytes(),
            ));
        };

        // was_next is only useful, if there was a true next().
        // otherwise it confuses the algorithm.
        let (first_chunk, _was_next) = match chunks.next() {
            Some(v) => (v, Some(true)),
            None => ("", None),
        };

        self.chunks = chunks;
        self.cur_chunk = first_chunk;
        self.cur_chunk_start = chunk_start;
        self.cursor = GraphemeCursor::new(byte_pos, self.text.len_bytes(), true);

        Ok(())
    }
}

impl<'a> Iterator for RopeGraphemes<'a> {
    type Item = Grapheme<'a>;

    #[inline]
    fn next(&mut self) -> Option<Grapheme<'a>> {
        let a = self.cursor.cur_cursor();
        let b;
        loop {
            match self
                .cursor
                .next_boundary(self.cur_chunk, self.cur_chunk_start)
            {
                Ok(None) => {
                    return None;
                }
                Ok(Some(n)) => {
                    b = n;
                    break;
                }
                Err(GraphemeIncomplete::NextChunk) => {
                    self.cur_chunk_start += self.cur_chunk.len();
                    if self.was_next == Some(false) {
                        // skip current
                        self.chunks.next();
                    }
                    (self.cur_chunk, self.was_next) = match self.chunks.next() {
                        Some(v) => (v, Some(true)),
                        None => ("", None),
                    };
                }
                Err(GraphemeIncomplete::PreContext(idx)) => {
                    let (chunk, byte_idx, _, _) = self.text.chunk_at_byte(idx.saturating_sub(1));
                    self.cursor.provide_context(chunk, byte_idx);
                }
                _ => unreachable!(),
            }
        }

        if a < self.cur_chunk_start {
            let a_char = self.text.byte_to_char(a);
            let b_char = self.text.byte_to_char(b);

            Some(Grapheme {
                grapheme: Cow::Owned(self.text.slice(a_char..b_char).to_string()),
                text_bytes: self.text_offset + a..self.text_offset + b,
            })
        } else {
            let a2 = a - self.cur_chunk_start;
            let b2 = b - self.cur_chunk_start;
            Some(Grapheme {
                grapheme: Cow::Borrowed(&self.cur_chunk[a2..b2]),
                text_bytes: self.text_offset + a..self.text_offset + b,
            })
        }
    }
}

#[derive(Debug)]
pub(crate) struct RevRopeGraphemes<'a> {
    it: RopeGraphemes<'a>,
}

impl<'a> Iterator for RevRopeGraphemes<'a> {
    type Item = Grapheme<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.it.prev()
    }
}

impl Cursor for RevRopeGraphemes<'_> {
    #[inline]
    fn prev(&mut self) -> Option<Self::Item> {
        self.it.next()
    }

    #[inline]
    fn rev_cursor(self) -> impl Cursor<Item = Self::Item> {
        self.it
    }

    fn text_offset(&self) -> usize {
        self.it.text_offset()
    }
}

impl SkipLine for RevRopeGraphemes<'_> {
    fn skip_line(&mut self) -> Result<(), TextError> {
        unimplemented!("no skip_line()")
    }

    fn skip_to(&mut self, _byte_pos: usize) -> Result<(), TextError> {
        unimplemented!("no skip_to()")
    }
}

#[cfg(test)]
mod test_str {
    use crate::Cursor;
    use crate::grapheme::StrGraphemes;

    #[test]
    fn test_str_graphemes0() {
        let s = String::from("\r\n");
        let mut s0 = StrGraphemes::new(0, &s);
        assert_eq!(s0.next().unwrap(), "\r\n");
    }

    #[test]
    fn test_str_graphemes1() {
        // basic graphemes
        let s = String::from("qwertz");

        let mut s0 = StrGraphemes::new(0, &s);
        assert_eq!(s0.next().unwrap(), "q");
        assert_eq!(s0.next().unwrap(), "w");
        assert_eq!(s0.next().unwrap(), "e");
        assert_eq!(s0.next().unwrap(), "r");
        assert_eq!(s0.next().unwrap(), "t");
        assert_eq!(s0.next().unwrap(), "z");
        assert!(s0.next().is_none());
        assert_eq!(s0.prev().unwrap(), "z");
        assert_eq!(s0.prev().unwrap(), "t");
        assert_eq!(s0.prev().unwrap(), "r");
        assert_eq!(s0.prev().unwrap(), "e");
        assert_eq!(s0.prev().unwrap(), "w");
        assert_eq!(s0.prev().unwrap(), "q");

        let mut s0 = StrGraphemes::new(1, &s[1..s.len() - 1]);
        assert_eq!(s0.next().unwrap(), "w");
        assert_eq!(s0.next().unwrap(), "e");
        assert_eq!(s0.next().unwrap(), "r");
        assert_eq!(s0.next().unwrap(), "t");
        assert!(s0.next().is_none());
        assert_eq!(s0.prev().unwrap(), "t");
        assert_eq!(s0.prev().unwrap(), "r");
        assert_eq!(s0.prev().unwrap(), "e");
        assert_eq!(s0.prev().unwrap(), "w");

        let mut s0 = StrGraphemes::new(3, &s[3..3]);
        assert!(s0.next().is_none());
        assert!(s0.prev().is_none());
    }

    #[test]
    fn test_str_graphemes2() {
        // complicated graphemes
        let s = String::from("w🤷‍♂️xw🤷‍♀️xw🤦‍♂️xw❤️xw🤦‍♀️xw💕🙍🏿‍♀️x");

        let mut s0 = StrGraphemes::new(0, &s);
        assert_eq!(s0.next().unwrap(), "w");
        assert_eq!(s0.next().unwrap(), "🤷‍♂️");
        assert_eq!(s0.next().unwrap(), "x");
        assert_eq!(s0.next().unwrap(), "w");
        assert_eq!(s0.next().unwrap(), "🤷‍♀️");
        assert_eq!(s0.next().unwrap(), "x");
        assert_eq!(s0.next().unwrap(), "w");
        assert_eq!(s0.next().unwrap(), "🤦‍♂️");
        assert_eq!(s0.next().unwrap(), "x");
        assert_eq!(s0.next().unwrap(), "w");
        assert_eq!(s0.next().unwrap(), "❤️");
        assert_eq!(s0.next().unwrap(), "x");
        assert_eq!(s0.next().unwrap(), "w");
        assert_eq!(s0.next().unwrap(), "🤦‍♀️");
        assert_eq!(s0.next().unwrap(), "x");
        assert_eq!(s0.next().unwrap(), "w");
        assert_eq!(s0.next().unwrap(), "💕");
        assert_eq!(s0.next().unwrap(), "🙍🏿‍♀️");
        assert_eq!(s0.next().unwrap(), "x");
        assert!(s0.next().is_none());
        assert_eq!(s0.prev().unwrap(), "x");
        assert_eq!(s0.prev().unwrap(), "🙍🏿‍♀️");
        assert_eq!(s0.prev().unwrap(), "💕");
        assert_eq!(s0.prev().unwrap(), "w");
        assert_eq!(s0.prev().unwrap(), "x");
        assert_eq!(s0.prev().unwrap(), "🤦‍♀️");
        assert_eq!(s0.prev().unwrap(), "w");
        assert_eq!(s0.prev().unwrap(), "x");
        assert_eq!(s0.prev().unwrap(), "❤️");
        assert_eq!(s0.prev().unwrap(), "w");
        assert_eq!(s0.prev().unwrap(), "x");
        assert_eq!(s0.prev().unwrap(), "🤦‍♂️");
        assert_eq!(s0.prev().unwrap(), "w");
        assert_eq!(s0.prev().unwrap(), "x");
        assert_eq!(s0.prev().unwrap(), "🤷‍♀️");
        assert_eq!(s0.prev().unwrap(), "w");
        assert_eq!(s0.prev().unwrap(), "x");
        assert_eq!(s0.prev().unwrap(), "🤷‍♂️");
        assert_eq!(s0.prev().unwrap(), "w");
    }

    #[test]
    fn test_str_graphemes3() {
        // complicated slices
        let s = String::from("qwertz");
        let mut s0 = StrGraphemes::new_offset(0, &s, 3);
        assert_eq!(s0.next().unwrap(), "r");
        assert_eq!(s0.prev().unwrap(), "r");
        assert_eq!(s0.prev().unwrap(), "e");

        let mut s0 = StrGraphemes::new_offset(0, &s, 3);
        assert_eq!(s0.next().unwrap().text_bytes(), 3..4);
        assert_eq!(s0.prev().unwrap().text_bytes(), 3..4);
        assert_eq!(s0.prev().unwrap().text_bytes(), 2..3);

        let s = String::from("w🤷‍♂️🤷‍♀️🤦‍♂️❤️🤦‍♀️💕🙍🏿‍♀️x");
        let mut s0 = StrGraphemes::new_offset(0, &s, 21);
        assert_eq!(s0.next().unwrap(), "♀\u{fe0f}");
        assert_eq!(s0.next().unwrap(), "🤦\u{200d}♂\u{fe0f}");
        assert_eq!(s0.prev().unwrap(), "🤦\u{200d}♂\u{fe0f}");
        assert_eq!(s0.prev().unwrap(), "🤷\u{200d}♀\u{fe0f}");

        let s = String::from("w🤷‍♂️🤷‍♀️🤦‍♂️❤️🤦‍♀️💕🙍🏿‍♀️x");
        let mut s0 = StrGraphemes::new_offset(0, &s, 21);
        assert_eq!(s0.next().unwrap().text_bytes(), 21..27);
        assert_eq!(s0.next().unwrap().text_bytes(), 27..40);
        assert_eq!(s0.prev().unwrap().text_bytes(), 27..40);
        assert_eq!(s0.prev().unwrap().text_bytes(), 14..27);
    }

    #[test]
    fn test_str_graphemes4() {
        // offsets and partial slices
        let s = String::from("qwertz");
        let mut s0 = StrGraphemes::new_offset(1, &s[1..5], 2);
        s0.next();
        assert_eq!(s0.text_offset(), 4);
        s0.next();
        assert_eq!(s0.text_offset(), 5);
        s0.next();
        assert_eq!(s0.text_offset(), 5);
        s0.next();
        assert_eq!(s0.text_offset(), 5);
        s0.prev();
        assert_eq!(s0.text_offset(), 4);
        s0.prev();
        assert_eq!(s0.text_offset(), 3);
        s0.prev();
        assert_eq!(s0.text_offset(), 2);
        s0.prev();
        assert_eq!(s0.text_offset(), 1);
        s0.prev();
        assert_eq!(s0.text_offset(), 1);
    }

    #[allow(deprecated)]
    #[test]
    #[allow(deprecated)]
    fn test_str_graphemes5() {
        // offsets and partial slices
        let s = String::from("qwertz");
        let mut s0 = StrGraphemes::new_offset(1, &s[1..5], 2).rev_cursor();
        assert_eq!(s0.next().unwrap(), "e");
        assert_eq!(s0.text_offset(), 2);

        assert_eq!(s0.next().unwrap(), "w");
        assert_eq!(s0.text_offset(), 1);

        assert_eq!(s0.prev().unwrap(), "w");
        assert_eq!(s0.text_offset(), 2);

        assert_eq!(s0.prev().unwrap(), "e");
        assert_eq!(s0.text_offset(), 3);

        assert_eq!(s0.prev().unwrap(), "r");
        assert_eq!(s0.text_offset(), 4);

        assert_eq!(s0.prev().unwrap(), "t");
        assert_eq!(s0.text_offset(), 5);
    }
}

#[cfg(test)]
mod test_rope {
    use crate::Cursor;
    use crate::grapheme::{RopeGraphemes, StrGraphemes};
    use ropey::Rope;

    #[test]
    fn test_rope_graphemes1() {
        // basic graphemes
        let s = Rope::from("qwertz");

        let mut s0 = RopeGraphemes::new(0, s.byte_slice(..));
        assert_eq!(s0.next().unwrap(), "q");
        assert_eq!(s0.next().unwrap(), "w");
        assert_eq!(s0.next().unwrap(), "e");
        assert_eq!(s0.next().unwrap(), "r");
        assert_eq!(s0.next().unwrap(), "t");
        assert_eq!(s0.next().unwrap(), "z");
        assert!(s0.next().is_none());
        assert_eq!(s0.prev().unwrap(), "z");
        assert_eq!(s0.prev().unwrap(), "t");
        assert_eq!(s0.prev().unwrap(), "r");
        assert_eq!(s0.prev().unwrap(), "e");
        assert_eq!(s0.prev().unwrap(), "w");
        assert_eq!(s0.prev().unwrap(), "q");

        let mut s0 = RopeGraphemes::new(1, s.byte_slice(1..s.len_bytes() - 1));
        assert_eq!(s0.next().unwrap(), "w");
        assert_eq!(s0.next().unwrap(), "e");
        assert_eq!(s0.next().unwrap(), "r");
        assert_eq!(s0.next().unwrap(), "t");
        assert!(s0.next().is_none());
        assert_eq!(s0.prev().unwrap(), "t");
        assert_eq!(s0.prev().unwrap(), "r");
        assert_eq!(s0.prev().unwrap(), "e");
        assert_eq!(s0.prev().unwrap(), "w");

        let mut s0 = RopeGraphemes::new(3, s.byte_slice(3..3));
        assert!(s0.next().is_none());
        assert!(s0.prev().is_none());
    }

    #[test]
    fn test_rope_graphemes2() {
        // complicated graphemes
        let s = Rope::from("w🤷‍♂️xw🤷‍♀️xw🤦‍♂️xw❤️xw🤦‍♀️xw💕🙍🏿‍♀️x");

        let mut s0 = RopeGraphemes::new(0, s.byte_slice(..));
        assert_eq!(s0.next().unwrap(), "w");
        assert_eq!(s0.next().unwrap(), "🤷‍♂️");
        assert_eq!(s0.next().unwrap(), "x");
        assert_eq!(s0.next().unwrap(), "w");
        assert_eq!(s0.next().unwrap(), "🤷‍♀️");
        assert_eq!(s0.next().unwrap(), "x");
        assert_eq!(s0.next().unwrap(), "w");
        assert_eq!(s0.next().unwrap(), "🤦‍♂️");
        assert_eq!(s0.next().unwrap(), "x");
        assert_eq!(s0.next().unwrap(), "w");
        assert_eq!(s0.next().unwrap(), "❤️");
        assert_eq!(s0.next().unwrap(), "x");
        assert_eq!(s0.next().unwrap(), "w");
        assert_eq!(s0.next().unwrap(), "🤦‍♀️");
        assert_eq!(s0.next().unwrap(), "x");
        assert_eq!(s0.next().unwrap(), "w");
        assert_eq!(s0.next().unwrap(), "💕");
        assert_eq!(s0.next().unwrap(), "🙍🏿‍♀️");
        assert_eq!(s0.next().unwrap(), "x");
        assert!(s0.next().is_none());
        assert_eq!(s0.prev().unwrap(), "x");
        assert_eq!(s0.prev().unwrap(), "🙍🏿‍♀️");
        assert_eq!(s0.prev().unwrap(), "💕");
        assert_eq!(s0.prev().unwrap(), "w");
        assert_eq!(s0.prev().unwrap(), "x");
        assert_eq!(s0.prev().unwrap(), "🤦‍♀️");
        assert_eq!(s0.prev().unwrap(), "w");
        assert_eq!(s0.prev().unwrap(), "x");
        assert_eq!(s0.prev().unwrap(), "❤️");
        assert_eq!(s0.prev().unwrap(), "w");
        assert_eq!(s0.prev().unwrap(), "x");
        assert_eq!(s0.prev().unwrap(), "🤦‍♂️");
        assert_eq!(s0.prev().unwrap(), "w");
        assert_eq!(s0.prev().unwrap(), "x");
        assert_eq!(s0.prev().unwrap(), "🤷‍♀️");
        assert_eq!(s0.prev().unwrap(), "w");
        assert_eq!(s0.prev().unwrap(), "x");
        assert_eq!(s0.prev().unwrap(), "🤷‍♂️");
        assert_eq!(s0.prev().unwrap(), "w");
    }

    #[test]
    fn test_rope_graphemes3() {
        // complicated graphemes
        let s = Rope::from("qwertz");
        let mut s0 = RopeGraphemes::new_offset(0, s.byte_slice(..), 3).expect("fine");
        assert_eq!(s0.next().unwrap(), "r");
        assert_eq!(s0.prev().unwrap(), "r");
        assert_eq!(s0.prev().unwrap(), "e");

        let mut s0 = RopeGraphemes::new_offset(0, s.byte_slice(..), 3).expect("fine");
        assert_eq!(s0.next().unwrap().text_bytes(), 3..4);
        assert_eq!(s0.prev().unwrap().text_bytes(), 3..4);
        assert_eq!(s0.prev().unwrap().text_bytes(), 2..3);

        let s = Rope::from("w🤷‍♂️🤷‍♀️🤦‍♂️❤️🤦‍♀️💕🙍🏿‍♀️x");
        let mut s0 = RopeGraphemes::new_offset(0, s.byte_slice(..), 21).expect("fine");
        assert_eq!(s0.next().unwrap(), "♀\u{fe0f}");
        assert_eq!(s0.next().unwrap(), "🤦\u{200d}♂\u{fe0f}");
        assert_eq!(s0.prev().unwrap(), "🤦\u{200d}♂\u{fe0f}");
        assert_eq!(s0.prev().unwrap(), "🤷\u{200d}♀\u{fe0f}");

        let s = Rope::from("w🤷‍♂️🤷‍♀️🤦‍♂️❤️🤦‍♀️💕🙍🏿‍♀️x");
        let mut s0 = RopeGraphemes::new_offset(0, s.byte_slice(..), 21).expect("fine");
        assert_eq!(s0.next().unwrap().text_bytes(), 21..27);
        assert_eq!(s0.next().unwrap().text_bytes(), 27..40);
        assert_eq!(s0.prev().unwrap().text_bytes(), 27..40);
        assert_eq!(s0.prev().unwrap().text_bytes(), 14..27);
    }

    #[test]
    fn test_rope_graphemes4() {
        // offsets and partial slices
        let s = Rope::from("qwertz");
        let mut s0 = RopeGraphemes::new_offset(1, s.byte_slice(1..5), 2).expect("fine");
        s0.next();
        assert_eq!(s0.text_offset(), 4);
        s0.next();
        assert_eq!(s0.text_offset(), 5);
        s0.next();
        assert_eq!(s0.text_offset(), 5);
        s0.next();
        assert_eq!(s0.text_offset(), 5);
        s0.prev();
        assert_eq!(s0.text_offset(), 4);
        s0.prev();
        assert_eq!(s0.text_offset(), 3);
        s0.prev();
        assert_eq!(s0.text_offset(), 2);
        s0.prev();
        assert_eq!(s0.text_offset(), 1);
        s0.prev();
        assert_eq!(s0.text_offset(), 1);
    }

    #[allow(deprecated)]
    #[test]
    #[allow(deprecated)]
    fn test_rope_graphemes5() {
        // offsets and partial slices
        let s = Rope::from("qwertz");
        let mut s0 = RopeGraphemes::new_offset(1, s.byte_slice(1..5), 2)
            .expect("fine")
            .rev_cursor();
        assert_eq!(s0.next().unwrap(), "e");
        assert_eq!(s0.text_offset(), 2);

        assert_eq!(s0.next().unwrap(), "w");
        assert_eq!(s0.text_offset(), 1);

        assert_eq!(s0.prev().unwrap(), "w");
        assert_eq!(s0.text_offset(), 2);

        assert_eq!(s0.prev().unwrap(), "e");
        assert_eq!(s0.text_offset(), 3);

        assert_eq!(s0.prev().unwrap(), "r");
        assert_eq!(s0.text_offset(), 4);

        assert_eq!(s0.prev().unwrap(), "t");
        assert_eq!(s0.text_offset(), 5);
    }

    #[test]
    fn test_rope_graphemes6() {
        // text rope boundary
        let s = Rope::from(
            "012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678)\
             abcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghiJ\
             012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678)\
             abcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghiJ\
             012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678)\
             abcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghiJ\
             012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678)\
             abcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghiJ\
             012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678)\
             abcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghiJ\
             012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678)\
             abcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghiJ\
             ",
        );
        assert_eq!(s.len_bytes(), 1200);
        let mut s0 = RopeGraphemes::new_offset(1, s.byte_slice(1..1199), 0).expect("fine");
        assert_eq!(s0.nth(598).unwrap(), "J");

        assert_eq!(s0.next().unwrap(), "0");
        assert_eq!(s0.text_offset(), 601);
        assert_eq!(s0.next().unwrap(), "1");
        assert_eq!(s0.text_offset(), 602);
        assert_eq!(s0.prev().unwrap(), "1");
        assert_eq!(s0.text_offset(), 601);
        assert_eq!(s0.prev().unwrap(), "0");
        assert_eq!(s0.text_offset(), 600);
        assert_eq!(s0.prev().unwrap(), "J");
        assert_eq!(s0.text_offset(), 599);
    }

    #[test]
    fn test_rope_graphemes7() {
        // test complicated grapheme at rope boundary
        let s = Rope::from(
            "012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678)\
             abcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghiJ\
             012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678)\
             abcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghiJ\
             012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678)\
             abcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghi🤷‍♂️\
             012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678)\
             abcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghiJ\
             012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678)\
             abcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghiJ\
             012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678)\
             abcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghiJ\
             ",
        );
        assert_eq!(s.len_bytes(), 1212);
        assert_eq!(s.chunks().next().unwrap().len(), 606);
        let mut s0 = RopeGraphemes::new_offset(1, s.byte_slice(1..1199), 0).expect("fine");
        assert_eq!(s0.nth(598).unwrap(), "🤷‍♂️");

        assert_eq!(s0.next().unwrap(), "0");
        assert_eq!(s0.text_offset(), 613);
        assert_eq!(s0.next().unwrap(), "1");
        assert_eq!(s0.text_offset(), 614);
        assert_eq!(s0.prev().unwrap(), "1");
        assert_eq!(s0.text_offset(), 613);
        assert_eq!(s0.prev().unwrap(), "0");
        assert_eq!(s0.text_offset(), 612);
        assert_eq!(s0.prev().unwrap(), "🤷‍♂️");
        assert_eq!(s0.text_offset(), 599);
        assert_eq!(s0.prev().unwrap(), "i");
        assert_eq!(s0.text_offset(), 598);

        assert_eq!(s0.next().unwrap(), "i");
        assert_eq!(s0.text_offset(), 599);
        assert_eq!(s0.next().unwrap(), "🤷‍♂️");
        assert_eq!(s0.text_offset(), 612);
        assert_eq!(s0.next().unwrap(), "0");
        assert_eq!(s0.text_offset(), 613);
        assert_eq!(s0.next().unwrap(), "1");
        assert_eq!(s0.text_offset(), 614);
    }

    #[test]
    fn test_rev_graphemes() {
        let mut it = StrGraphemes::new_offset(0, "\r\n", 2);
        assert_eq!(it.prev().unwrap(), "\r\n");

        let mut it = StrGraphemes::new_offset(0, "\r\r\n", 3);
        assert_eq!(it.prev().unwrap(), "\r\n");
        assert_eq!(it.prev().unwrap(), "\r");

        let mut it = StrGraphemes::new_offset(0, "\r\r\n\n", 4);
        assert_eq!(it.prev().unwrap(), "\n");
        assert_eq!(it.prev().unwrap(), "\r\n");
        assert_eq!(it.prev().unwrap(), "\r");
    }
}
