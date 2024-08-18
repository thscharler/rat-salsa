use crate::{Cursor, TextError, TextPosition};
use ropey::iter::Chunks;
use ropey::RopeSlice;
use std::borrow::Cow;
use std::cmp;
use std::ops::Range;
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete, UnicodeSegmentation};

/// One grapheme.
#[derive(Debug)]
pub struct Grapheme<'a> {
    /// grapheme
    grapheme: Cow<'a, str>,
    /// byte-range of the grapheme in the given slice.
    bytes: Range<usize>,
    /// offset of the slice into the complete text.
    text_offset: usize,
}

impl<'a, R: AsRef<str>> PartialEq<R> for Grapheme<'a> {
    fn eq(&self, other: &R) -> bool {
        self.grapheme.as_ref() == other.as_ref()
    }
}

impl<'a> Grapheme<'a> {
    /// First (only) char of the grapheme is a whitespace.
    pub fn is_whitespace(&self) -> bool {
        self.grapheme
            .chars()
            .next()
            .map(|v| v.is_whitespace())
            .unwrap_or(false)
    }

    /// Is a linebreak.
    pub fn is_line_break(&self) -> bool {
        self.grapheme == "\n" || self.grapheme == "\r\n"
    }

    /// Get the grapheme.
    pub fn grapheme(&'a self) -> &'a str {
        self.grapheme.as_ref()
    }

    /// Get the byte-range as relative range to the slice.
    pub fn bytes(&self) -> Range<usize> {
        self.bytes.clone()
    }

    /// Get the byte-range as absolute range into the complete text.
    pub fn text_bytes(&self) -> Range<usize> {
        self.text_offset + self.bytes.start..self.text_offset + self.bytes.end
    }
}

/// Data for rendering/mapping graphemes to screen coordinates.
#[derive(Debug)]
pub struct Glyph<'a> {
    /// First char.
    glyph: Cow<'a, str>,
    /// Display length for the glyph.
    display: u16,
    /// byte-range of the glyph in the given slice.
    bytes: Range<usize>,
    /// offset of the slice into the complete text.
    text_offset: usize,
    /// screen-position corrected by text_offset.
    /// first visible column is at 0.
    screen_pos: (u16, u16),
    /// text-position
    pos: TextPosition,
}

impl<'a> Glyph<'a> {
    /// Get the glyph.
    pub fn glyph(&'a self) -> &'a str {
        self.glyph.as_ref()
    }

    /// Display width of the glyph.
    pub fn display(&self) -> u16 {
        self.display
    }

    /// Get the byte-range as relative range to the slice.
    pub fn bytes(&self) -> Range<usize> {
        self.bytes.clone()
    }

    /// Get the byte-range as absolute range into the complete text.
    pub fn text_bytes(&self) -> Range<usize> {
        self.text_offset + self.bytes.start..self.text_offset + self.bytes.end
    }

    /// Get the position of the glyph
    pub fn pos(&self) -> TextPosition {
        self.pos
    }

    /// Get the screen position of the glyph.
    pub fn screen_pos(&self) -> (u16, u16) {
        self.screen_pos
    }
}

/// Length as grapheme count, excluding line breaks.
pub(crate) fn rope_line_len(r: RopeSlice<'_>) -> usize {
    let it = RopeGraphemes::new(0, r);
    it.filter(|g| g.grapheme != "\n" && g.grapheme != "\r\n")
        .count()
}

/// Length as grapheme count, excluding line breaks.
pub(crate) fn str_line_len(s: &str) -> usize {
    let it = s.graphemes(true);
    it.filter(|c| *c != "\n" && *c != "\r\n").count()
}

/// A cursor over graphemes of a string.
#[derive(Debug)]
pub(crate) struct StrGraphemes<'a> {
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

impl<'a> Cursor for StrGraphemes<'a> {
    fn prev(&mut self) -> Option<Self::Item> {
        let start = self.cursor.cur_cursor();
        let prev = self.cursor.prev_boundary(self.text, 0).unwrap()?;
        Some(Grapheme {
            grapheme: Cow::Borrowed(&self.text[prev..start]),
            bytes: prev..start,
            text_offset: self.text_offset,
        })
    }

    fn rev_cursor(self) -> impl Cursor<Item = Self::Item> {
        RevStrGraphemes { it: self }
    }

    fn offset(&self) -> usize {
        self.cursor.cur_cursor()
    }

    fn text_offset(&self) -> usize {
        self.text_offset + self.cursor.cur_cursor()
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
            bytes: start..next,
            text_offset: self.text_offset,
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
        self.prev()
    }
}

impl<'a> Cursor for RevStrGraphemes<'a> {
    #[inline]
    fn prev(&mut self) -> Option<Self::Item> {
        self.it.next()
    }

    #[inline]
    fn rev_cursor(self) -> impl Cursor<Item = Self::Item> {
        self.it
    }

    fn offset(&self) -> usize {
        self.it.offset()
    }

    fn text_offset(&self) -> usize {
        self.it.text_offset()
    }
}

/// An implementation of a graphemes iterator, for iterating over
/// the graphemes of a RopeSlice.
#[derive(Debug)]
pub(crate) struct RopeGraphemes<'a> {
    text_offset: usize,
    text: RopeSlice<'a>,
    chunks: Chunks<'a>,
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
        let first_chunk = chunks.next().unwrap_or("");
        RopeGraphemes {
            text_offset: slice_offset,
            text: slice,
            chunks,
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
        let first_chunk = chunks.next().unwrap_or("");
        Ok(RopeGraphemes {
            text_offset: slice_offset,
            text: slice,
            chunks,
            cur_chunk: first_chunk,
            cur_chunk_start: chunk_start,
            cursor: GraphemeCursor::new(offset, slice.len_bytes(), true),
        })
    }
}

impl<'a> Cursor for RopeGraphemes<'a> {
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
                    self.cur_chunk = self.chunks.prev().unwrap_or("");
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
                bytes: b..a,
                text_offset: self.text_offset,
            })
        } else {
            let a2 = a - self.cur_chunk_start;
            let b2 = b - self.cur_chunk_start;
            Some(Grapheme {
                grapheme: Cow::Borrowed(&self.cur_chunk[b2..a2]),
                bytes: b..a,
                text_offset: self.text_offset,
            })
        }
    }

    #[inline]
    fn rev_cursor(self) -> impl Cursor<Item = Self::Item> {
        RevRopeGraphemes { it: self }
    }

    fn offset(&self) -> usize {
        self.cursor.cur_cursor()
    }

    fn text_offset(&self) -> usize {
        self.text_offset + self.cursor.cur_cursor()
    }
}

impl<'a> Iterator for RopeGraphemes<'a> {
    type Item = Grapheme<'a>;

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
                    self.cur_chunk = self.chunks.next().unwrap_or("");
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
                bytes: a..b,
                text_offset: self.text_offset,
            })
        } else {
            let a2 = a - self.cur_chunk_start;
            let b2 = b - self.cur_chunk_start;
            Some(Grapheme {
                grapheme: Cow::Borrowed(&self.cur_chunk[a2..b2]),
                bytes: a..b,
                text_offset: self.text_offset,
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
        self.prev()
    }
}

impl<'a> Cursor for RevRopeGraphemes<'a> {
    #[inline]
    fn prev(&mut self) -> Option<Self::Item> {
        self.it.next()
    }

    #[inline]
    fn rev_cursor(self) -> impl Cursor<Item = Self::Item> {
        self.it
    }

    fn offset(&self) -> usize {
        self.it.offset()
    }

    fn text_offset(&self) -> usize {
        self.it.text_offset()
    }
}

/// Iterates over the glyphs of a row-range.
///
/// Keeps track of the text-position and the display-position on screen.
/// Does a conversion from graphemes to glyph-text and glyph-width.
///
/// This is used for rendering text, and for mapping text-positions
/// to screen-positions and vice versa.
#[derive(Debug)]
pub(crate) struct GlyphIter<Iter> {
    iter: Iter,

    pos: TextPosition,

    screen_offset: u16,
    screen_width: u16,
    screen_pos: (u16, u16),

    tabs: u16,
    show_ctrl: bool,
}

impl<'a, Iter> GlyphIter<Iter>
where
    Iter: Iterator<Item = Grapheme<'a>>,
{
    /// New iterator.
    pub(crate) fn new(pos: TextPosition, iter: Iter) -> Self {
        Self {
            iter,
            pos,
            screen_offset: 0,
            screen_width: u16::MAX,
            screen_pos: Default::default(),
            tabs: 8,
            show_ctrl: false,
        }
    }

    /// Screen offset.
    pub(crate) fn set_screen_offset(&mut self, offset: u16) {
        self.screen_offset = offset;
    }

    /// Screen width.
    pub(crate) fn set_screen_width(&mut self, width: u16) {
        self.screen_width = width;
    }

    /// Tab width
    pub(crate) fn set_tabs(&mut self, tabs: u16) {
        self.tabs = tabs;
    }

    /// Show ASCII control codes.
    pub(crate) fn set_show_ctrl(&mut self, show_ctrl: bool) {
        self.show_ctrl = show_ctrl;
    }
}

impl<'a, Iter> Iterator for GlyphIter<Iter>
where
    Iter: Iterator<Item = Grapheme<'a>>,
{
    type Item = Glyph<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(grapheme) = self.iter.next() {
            let glyph;
            let len: u16;
            let mut lbrk = false;

            // todo: maybe add some ligature support.

            match grapheme.grapheme.as_ref() {
                "\n" | "\r\n" => {
                    lbrk = true;
                    len = if self.show_ctrl { 1 } else { 0 };
                    glyph = Cow::Borrowed(if self.show_ctrl { "\u{2424}" } else { "" });
                }
                "\t" => {
                    len = self.tabs - (self.screen_pos.0 % self.tabs);
                    glyph = Cow::Borrowed(if self.show_ctrl { "\u{2409}" } else { " " });
                }
                c if ("\x00".."\x20").contains(&c) => {
                    static CCHAR: [&str; 32] = [
                        "\u{2400}", "\u{2401}", "\u{2402}", "\u{2403}", "\u{2404}", "\u{2405}",
                        "\u{2406}", "\u{2407}", "\u{2408}", "\u{2409}", "\u{240A}", "\u{240B}",
                        "\u{240C}", "\u{240D}", "\u{240E}", "\u{240F}", "\u{2410}", "\u{2411}",
                        "\u{2412}", "\u{2413}", "\u{2414}", "\u{2415}", "\u{2416}", "\u{2417}",
                        "\u{2418}", "\u{2419}", "\u{241A}", "\u{241B}", "\u{241C}", "\u{241D}",
                        "\u{241E}", "\u{241F}",
                    ];
                    let c0 = c.bytes().next().expect("byte");
                    len = 1;
                    glyph = Cow::Borrowed(if self.show_ctrl {
                        &CCHAR[c0 as usize]
                    } else {
                        "\u{FFFD}"
                    });
                }
                c => {
                    len = unicode_display_width::width(c) as u16;
                    glyph = grapheme.grapheme;
                }
            }

            let pos = self.pos;
            let screen_pos = self.screen_pos;

            if lbrk {
                self.screen_pos.0 = 0;
                self.screen_pos.1 += 1;
                self.pos.x = 0;
                self.pos.y += 1;
            } else {
                self.screen_pos.0 += len;
                self.pos.x += 1;
            }

            // clip left
            if screen_pos.0 < self.screen_offset {
                if screen_pos.0 + len > self.screen_offset {
                    // don't show partial glyphs, but show the space they need.
                    // avoids flickering when scrolling left/right.
                    return Some(Glyph {
                        glyph: Cow::Borrowed("\u{2203}"),
                        bytes: grapheme.bytes,
                        text_offset: grapheme.text_offset,
                        display: screen_pos.0 + len - self.screen_offset,
                        pos,
                        screen_pos: (0, screen_pos.1),
                    });
                } else {
                    // out left
                }
            } else if screen_pos.0 + len >= self.screen_offset + self.screen_width {
                if screen_pos.0 < self.screen_offset + self.screen_width {
                    // don't show partial glyphs, but show the space they need.
                    // avoids flickering when scrolling left/right.
                    return Some(Glyph {
                        glyph: Cow::Borrowed("\u{2203}"),
                        bytes: grapheme.bytes,
                        text_offset: grapheme.text_offset,
                        display: screen_pos.0 + len - (self.screen_offset + self.screen_width),
                        pos,
                        screen_pos: (0, screen_pos.1),
                    });
                } else {
                    // out right
                }
            } else {
                return Some(Glyph {
                    glyph,
                    bytes: grapheme.bytes,
                    text_offset: grapheme.text_offset,
                    display: len,
                    pos,
                    screen_pos: (screen_pos.0 - self.screen_offset, screen_pos.1),
                });
            }
        }

        None
    }
}
