use crate::text_store::Cursor;
use crate::{Glyph, Grapheme, TextError};
use ropey::iter::Chunks;
use ropey::RopeSlice;
use std::borrow::Cow;
use std::cmp;
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete, UnicodeSegmentation};

/// Length as grapheme count, excluding line breaks.
pub(crate) fn rope_line_len(r: RopeSlice<'_>) -> usize {
    let it = RopeGraphemesIdx::new(r);
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
pub(crate) struct StrGraphemesIdx<'a> {
    string: &'a str,
    cursor: GraphemeCursor,
}

impl<'a> StrGraphemesIdx<'a> {
    pub(crate) fn new(s: &'a str) -> Self {
        Self {
            string: s,
            cursor: GraphemeCursor::new(0, s.len(), true),
        }
    }

    pub(crate) fn new_offset(s: &'a str, offset: usize) -> Self {
        Self {
            string: s,
            cursor: GraphemeCursor::new(offset, s.len(), true),
        }
    }
}

impl<'a> Cursor for StrGraphemesIdx<'a> {
    fn prev(&mut self) -> Option<Self::Item> {
        let start = self.cursor.cur_cursor();
        let prev = self.cursor.prev_boundary(self.string, 0).unwrap()?;
        Some(Grapheme {
            grapheme: Cow::Borrowed(&self.string[prev..start]),
            bytes: prev..start,
        })
    }
}

impl<'a> Iterator for StrGraphemesIdx<'a> {
    type Item = Grapheme<'a>;

    #[inline]
    fn next(&mut self) -> Option<Grapheme<'a>> {
        let start = self.cursor.cur_cursor();
        let next = self.cursor.next_boundary(self.string, 0).unwrap()?;
        Some(Grapheme {
            grapheme: Cow::Borrowed(&self.string[start..next]),
            bytes: start..next,
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let slen = self.string.len() - self.cursor.cur_cursor();
        (cmp::min(slen, 1), Some(slen))
    }
}

/// An implementation of a graphemes iterator, for iterating over
/// the graphemes of a RopeSlice.
#[derive(Debug)]
pub struct RopeGraphemesIdx<'a> {
    text: RopeSlice<'a>,
    chunks: Chunks<'a>,
    cur_chunk: &'a str,
    cur_chunk_start: usize,
    cursor: GraphemeCursor,
}

impl<'a> RopeGraphemesIdx<'a> {
    /// New grapheme iterator.
    pub(crate) fn new(slice: RopeSlice<'a>) -> RopeGraphemesIdx<'a> {
        let mut chunks = slice.chunks();
        let first_chunk = chunks.next().unwrap_or("");
        RopeGraphemesIdx {
            text: slice,
            chunks,
            cur_chunk: first_chunk,
            cur_chunk_start: 0,
            cursor: GraphemeCursor::new(0, slice.len_bytes(), true),
        }
    }

    /// New grapheme iterator.
    ///
    /// Offset must be a valid char boundary.
    pub(crate) fn new_offset(
        slice: RopeSlice<'a>,
        offset: usize,
    ) -> Result<RopeGraphemesIdx<'a>, TextError> {
        let Some((mut chunks, chunk_start, _, _)) = slice.get_chunks_at_byte(offset) else {
            return Err(TextError::ByteIndexOutOfBounds(offset, slice.len_bytes()));
        };
        let first_chunk = chunks.next().unwrap_or("");
        Ok(RopeGraphemesIdx {
            text: slice,
            chunks,
            cur_chunk: first_chunk,
            cur_chunk_start: chunk_start,
            cursor: GraphemeCursor::new(offset, slice.len_bytes(), true),
        })
    }
}

impl<'a> Cursor for RopeGraphemesIdx<'a> {
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
            })
        } else {
            let a2 = a - self.cur_chunk_start;
            let b2 = b - self.cur_chunk_start;
            Some(Grapheme {
                grapheme: Cow::Borrowed(&self.cur_chunk[b2..a2]),
                bytes: b..a,
            })
        }
    }
}

impl<'a> Iterator for RopeGraphemesIdx<'a> {
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
            })
        } else {
            let a2 = a - self.cur_chunk_start;
            let b2 = b - self.cur_chunk_start;
            Some(Grapheme {
                grapheme: Cow::Borrowed(&self.cur_chunk[a2..b2]),
                bytes: a..b,
            })
        }
    }
}

/// Converts a grapheme-iterator into a glyph iterator.
///
/// This is used for rendering text, and for mapping text-positions
/// to screen-positions and vice versa.
///
/// It
/// * has a length for the glyph. This is used for wide characters
///   and tab support.
/// * has a column-offset.
/// * can translate control-codes to visible graphemes.
#[derive(Debug)]
pub(crate) struct GlyphIter<Iter> {
    iter: Iter,
    offset: usize,
    tabs: u16,
    show_ctrl: bool,
    col: usize,
}

impl<'a, Iter> GlyphIter<Iter>
where
    Iter: Iterator<Item = Grapheme<'a>>,
{
    /// New iterator.
    pub(crate) fn new(iter: Iter) -> Self {
        Self {
            iter,
            offset: 0,
            tabs: 8,
            show_ctrl: false,
            col: 0,
        }
    }

    /// Text offset.
    /// Iterates only graphemes beyond this offset.
    /// Might return partial glyphs.
    pub(crate) fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
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
            let mut glyph;
            let mut len: usize;

            match grapheme.grapheme.as_ref() {
                "\n" | "\r\n" => {
                    len = if self.show_ctrl { 1 } else { 0 };
                    glyph = Cow::Borrowed(if self.show_ctrl { "\u{2424}" } else { "" });
                }
                "\t" => {
                    len = self.tabs as usize - self.col % self.tabs as usize;
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
                    len = unicode_display_width::width(c) as usize;
                    glyph = grapheme.grapheme;
                }
            }

            let next_col = self.col + len;

            // clip left
            if self.col < self.offset {
                if self.col + len > self.offset {
                    // don't show partial glyphs, but show the space they need.
                    // avoids flickering when scrolling left/right.
                    glyph = Cow::Borrowed(" ");
                    len = self.offset - self.col;
                    self.col = next_col;
                    return Some(Glyph {
                        glyph,
                        bytes: grapheme.bytes,
                        display: len,
                    });
                } else {
                    // out left
                    self.col = next_col;
                }
            } else {
                self.col = next_col;
                return Some(Glyph {
                    glyph,
                    bytes: grapheme.bytes,
                    display: len,
                });
            }
        }

        None
    }
}
