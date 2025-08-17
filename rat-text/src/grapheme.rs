use crate::{upos_type, Cursor, TextError, TextPosition};
use ropey::iter::Chunks;
use ropey::RopeSlice;
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use std::ops::Range;
use std::{cmp, mem};
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete};

/// One grapheme.
#[derive(Debug, PartialEq)]
pub struct Grapheme<'a> {
    /// grapheme
    grapheme: Cow<'a, str>,
    /// byte-range of the grapheme in the given slice.
    text_bytes: Range<usize>,
}

impl<R: AsRef<str>> PartialEq<R> for Grapheme<'_> {
    fn eq(&self, other: &R) -> bool {
        self.grapheme.as_ref() == other.as_ref()
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

    /// Get the byte-range as absolute range into the complete text.
    pub fn text_bytes(&self) -> Range<usize> {
        self.text_bytes.clone()
    }
}

/// Data for rendering/mapping graphemes to screen coordinates.
#[derive(Debug)]
pub struct Glyph<'a> {
    /// Display glyph.
    glyph: Cow<'a, str>,
    /// byte-range of the glyph in the given slice.
    text_bytes: Range<usize>,
    /// screen-position corrected by text_offset.
    /// first visible column is at 0.
    screen_pos: (u16, u16),
    /// Display length for the glyph.
    screen_width: u16,
    /// Last item in this screen-line.
    line_break: bool,
    /// text-position
    pos: TextPosition,
}

impl<'a> Glyph<'a> {
    pub fn new(
        glyph: Cow<'a, str>,
        text_bytes: Range<usize>,
        screen_pos: (u16, u16),
        screen_width: u16,
        line_break: bool,
        pos: TextPosition,
    ) -> Self {
        Self {
            glyph,
            text_bytes,
            screen_pos,
            screen_width,
            line_break,
            pos,
        }
    }

    /// Get the glyph.
    pub fn glyph(&'a self) -> &'a str {
        self.glyph.as_ref()
    }

    /// Get the byte-range as absolute range into the complete text.
    pub fn text_bytes(&self) -> Range<usize> {
        self.text_bytes.clone()
    }

    /// Get the position of the glyph
    pub fn pos(&self) -> TextPosition {
        self.pos
    }

    /// Get the screen position of the glyph. Starts at (0,0) in
    /// the top/left of the widget.
    pub fn screen_pos(&self) -> (u16, u16) {
        self.screen_pos
    }

    /// Display width of the glyph.
    pub fn screen_width(&self) -> u16 {
        self.screen_width
    }

    /// Last item in this screen line
    pub fn line_break(&self) -> bool {
        self.line_break
    }

    /// Does the glyph cover the given screen-position?
    pub fn contains_screen_pos(&self, screen_pos: (u16, u16)) -> bool {
        if self.screen_pos.1 == screen_pos.1 {
            if screen_pos.0 >= self.screen_pos.0 {
                if screen_pos.0 < self.screen_pos.0 + self.screen_width {
                    return true;
                }
                if self.line_break {
                    return true;
                }
            }
        }

        false
    }

    /// Does teh glyph cover the given x-position.
    /// Doesn't check for the y-position.
    pub fn contains_screen_x(&self, screen_x: u16) -> bool {
        if screen_x >= self.screen_pos.0 {
            if screen_x < self.screen_pos.0 + self.screen_width {
                return true;
            }
            if self.line_break {
                return true;
            }
        }

        false
    }
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

/// An implementation of a graphemes iterator, for iterating over
/// the graphemes of a RopeSlice.
#[derive(Debug)]
pub(crate) struct RopeGraphemes<'a> {
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

    #[inline]
    fn rev_cursor(self) -> impl Cursor<Item = Self::Item> {
        RevRopeGraphemes { it: self }
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

#[derive(Debug)]
#[non_exhaustive]
pub enum TextBreak2 {
    /// shift glyphs to the left and clip at right margin.
    ShiftText,
    /// break text at right margin.
    /// if a word-margin is set, use it.
    BreakText,
}

impl Default for TextBreak2 {
    fn default() -> Self {
        Self::ShiftText
    }
}

pub(crate) struct GlyphIter2<Iter> {
    iter: Iter,
    done: bool,

    /// Sometimes one grapheme creates two glyphs.
    next_glyph: Option<Glyph<'static>>,

    /// Next glyph position.
    next_pos: TextPosition,
    next_screen_pos: (upos_type, upos_type),
    /// Text position of the previous glyph.
    last_pos: TextPosition,
    last_byte: usize,

    /// Tab expansion
    tabs: upos_type,
    /// Show CTRL chars
    show_ctrl: bool,
    /// Line-break enabled?
    line_break: bool,
    /// Text-break enabled?
    text_break: TextBreak2,
    /// Left margin
    left_margin: upos_type,
    /// Right margin
    right_margin: upos_type,
    /// Word breaking after this margin.
    word_margin: upos_type,
}

impl<Iter> Debug for GlyphIter2<Iter> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlyphIter2")
            .field("done", &self.done)
            .field("next_glyph", &self.next_glyph)
            .field("next_pos", &self.next_pos)
            .field("next_screen_pos", &self.next_screen_pos)
            .field("last_pos", &self.last_pos)
            .field("last_byte", &self.last_byte)
            .field("tabs", &self.tabs)
            .field("show_ctrl", &self.show_ctrl)
            .field("line_break", &self.line_break)
            .field("text_break", &self.text_break)
            .field("left_margin", &self.left_margin)
            .field("right_margin", &self.right_margin)
            .field("word_margin", &self.word_margin)
            .finish()
    }
}

impl<Iter> GlyphIter2<Iter> {
    /// New iterator.
    pub(crate) fn new(pos: TextPosition, iter: Iter) -> Self {
        Self {
            iter,
            done: Default::default(),
            next_pos: pos,
            next_screen_pos: Default::default(),
            last_pos: Default::default(),
            last_byte: Default::default(),
            next_glyph: Default::default(),
            tabs: 8,
            show_ctrl: false,
            line_break: true,
            text_break: Default::default(),
            left_margin: Default::default(),
            right_margin: Default::default(),
            word_margin: Default::default(),
        }
    }

    /// Tab width
    pub(crate) fn set_tabs(&mut self, tabs: upos_type) {
        self.tabs = tabs;
    }

    /// Handle line-breaks. If false everything is treated as one line.
    pub(crate) fn set_line_break(&mut self, line_break: bool) {
        self.line_break = line_break;
    }

    /// Show ASCII control codes.
    pub(crate) fn set_show_ctrl(&mut self, show_ctrl: bool) {
        self.show_ctrl = show_ctrl;
    }

    /// Handle text-breaks. Breaks the line and continues on the
    /// next screen line.
    pub(crate) fn set_text_break(&mut self, text_break: TextBreak2) {
        self.text_break = text_break;
    }

    pub(crate) fn set_left_margin(&mut self, left_margin: upos_type) {
        self.left_margin = left_margin;
    }

    pub(crate) fn set_right_margin(&mut self, right_margin: upos_type) {
        self.right_margin = right_margin;
    }

    pub(crate) fn set_word_margin(&mut self, word_margin: upos_type) {
        self.word_margin = word_margin;
    }
}

impl<'a, Iter> Iterator for GlyphIter2<Iter>
where
    Iter: Iterator<Item = Grapheme<'a>>,
{
    type Item = Glyph<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        if let Some(glyph) = self.next_glyph.take() {
            return Some(glyph);
        }

        loop {
            let Some(Grapheme {
                mut grapheme,
                text_bytes,
            }) = self.iter.next()
            else {
                self.done = true;

                // emit a synthetic EOT at the very end.
                // helps if the last line doesn't end in a line-break.
                let glyph = Glyph {
                    glyph: if self.show_ctrl {
                        Cow::Borrowed("\u{2403}")
                    } else {
                        Cow::Borrowed("")
                    },
                    text_bytes: self.last_byte..self.last_byte,
                    screen_pos: (
                        self.next_screen_pos.0.saturating_sub(self.left_margin) as u16,
                        self.next_screen_pos.1 as u16,
                    ),
                    screen_width: if self.show_ctrl { 1 } else { 0 },
                    line_break: true,
                    pos: self.next_pos,
                };

                return Some(glyph);
            };

            let mut glyph;
            let mut text_bytes = text_bytes;
            let mut screen_pos = (self.next_screen_pos.0, self.next_screen_pos.1);
            let mut screen_width;
            let mut line_break;
            let mut pos = self.next_pos;

            // remap grapheme
            if grapheme == "\n" || grapheme == "\r\n" {
                if self.line_break {
                    line_break = true;
                    screen_width = if self.show_ctrl { 1 } else { 0 };
                    glyph = Cow::Borrowed(if self.show_ctrl { "\u{240A}" } else { "" });
                } else {
                    line_break = false;
                    screen_width = 1;
                    glyph = Cow::Borrowed("\u{240A}");
                }
            } else if grapheme == "\t" {
                line_break = false;
                screen_width = (self.tabs - (self.next_screen_pos.0 % self.tabs)) as u16;
                glyph = Cow::Borrowed(if self.show_ctrl { "\u{2409}" } else { " " });
            } else if ("\x00".."\x20").contains(&grapheme.as_ref()) {
                line_break = false;
                screen_width = 1;

                // Control char unicode display replacement.
                static CONTROL_CHARS: [&str; 32] = [
                    "\u{2400}", "\u{2401}", "\u{2402}", "\u{2403}", "\u{2404}", "\u{2405}",
                    "\u{2406}", "\u{2407}", "\u{2408}", "\u{2409}", "\u{240A}", "\u{240B}",
                    "\u{240C}", "\u{240D}", "\u{240E}", "\u{240F}", "\u{2410}", "\u{2411}",
                    "\u{2412}", "\u{2413}", "\u{2414}", "\u{2415}", "\u{2416}", "\u{2417}",
                    "\u{2418}", "\u{2419}", "\u{241A}", "\u{241B}", "\u{241C}", "\u{241D}",
                    "\u{241E}", "\u{241F}",
                ];

                glyph = Cow::Borrowed(if self.show_ctrl {
                    CONTROL_CHARS[grapheme.as_bytes()[0] as usize]
                } else {
                    "\u{FFFD}"
                });
            } else {
                line_break = false;
                screen_width = unicode_display_width::width(&grapheme) as u16;
                glyph = mem::take(&mut grapheme);
            }

            // next glyph positioning
            if let TextBreak2::ShiftText = self.text_break {
                let right_margin = if self.show_ctrl {
                    self.right_margin.saturating_sub(1)
                } else {
                    self.right_margin
                };

                // self.next_screen_pos later
                self.last_pos = self.next_pos;
                self.last_byte = text_bytes.end;
                self.next_pos.x += 1;
                // next_pos.y doesn't change

                // Clip glyphs and correct left offset
                if line_break {
                    self.next_screen_pos.0 = 0;
                    self.next_screen_pos.1 += 1;
                    self.next_pos.x = 0;
                    self.next_pos.y += 1;

                    if screen_pos.0 as upos_type <= right_margin {
                        screen_pos.0 = screen_pos.0.saturating_sub(self.left_margin);

                        return Some(Glyph {
                            glyph,
                            text_bytes,
                            screen_pos: (screen_pos.0 as u16, screen_pos.1 as u16),
                            screen_width,
                            line_break,
                            pos,
                        });
                    } else {
                        continue;
                    }
                } else if self.next_screen_pos.0 < self.left_margin
                    && self.next_screen_pos.0 + screen_width as upos_type > self.left_margin
                {
                    // show replacement for split glyph
                    glyph = Cow::Borrowed("\u{2426}");
                    screen_width = (self.next_screen_pos.0 + screen_width as upos_type
                        - self.left_margin) as u16;
                    screen_pos.0 = 0;

                    self.next_screen_pos.0 += screen_width as upos_type;
                    // self.next_screen_pos.1 doesn't change

                    return Some(Glyph {
                        glyph,
                        text_bytes,
                        screen_pos: (screen_pos.0 as u16, screen_pos.1 as u16),
                        screen_width,
                        line_break,
                        pos,
                    });
                } else if self.next_screen_pos.0 < self.left_margin {
                    screen_pos.0 = screen_pos.0.saturating_sub(self.left_margin);

                    self.next_screen_pos.0 += screen_width as upos_type;
                    // self.next_screen_pos.1 doesn't change

                    // skip glyph
                    continue;
                } else if self.next_screen_pos.0 == right_margin {
                    line_break = true;
                    screen_pos.0 = screen_pos.0.saturating_sub(self.left_margin);
                    screen_width = if self.show_ctrl { 1 } else { 0 };
                    glyph = Cow::Borrowed(if self.show_ctrl { "\u{2424}" } else { "" });
                    text_bytes = self.last_byte..self.last_byte;

                    self.next_screen_pos.0 += 1;
                    // self.next_screen_pos.1 doesn't change

                    return Some(Glyph {
                        glyph,
                        text_bytes,
                        screen_pos: (screen_pos.0 as u16, screen_pos.1 as u16),
                        screen_width,
                        line_break,
                        pos,
                    });
                } else if self.next_screen_pos.0 > right_margin {
                    screen_pos.0 = screen_pos.0.saturating_sub(self.left_margin);

                    self.next_screen_pos.0 += screen_width as upos_type;
                    // self.next_screen_pos.1 doesn't change

                    // skip glyph
                    continue;
                } else if self.next_screen_pos.0 < right_margin
                    && self.next_screen_pos.0 + screen_width as upos_type > right_margin
                {
                    // show replacement for split glyph
                    glyph = Cow::Borrowed("\u{2426}");
                    screen_pos.0 = screen_pos.0.saturating_sub(self.left_margin);
                    screen_width = (self.next_screen_pos.0 + screen_width as upos_type
                        - self.right_margin as upos_type) as u16;

                    self.next_screen_pos.0 += screen_width as upos_type;
                    // self.next_screen_pos.1 doesn't change

                    return Some(Glyph {
                        glyph,
                        text_bytes,
                        screen_pos: (screen_pos.0 as u16, screen_pos.1 as u16),
                        screen_width,
                        line_break,
                        pos,
                    });
                } else {
                    screen_pos.0 = screen_pos.0.saturating_sub(self.left_margin);

                    self.next_screen_pos.0 += screen_width as upos_type;
                    // self.next_screen_pos.1 doesn't change

                    return Some(Glyph {
                        glyph,
                        text_bytes,
                        screen_pos: (screen_pos.0 as u16, screen_pos.1 as u16),
                        screen_width,
                        line_break,
                        pos,
                    });
                }
            } else if let TextBreak2::BreakText = self.text_break {
                let right_margin = if self.show_ctrl {
                    self.right_margin.saturating_sub(1)
                } else {
                    self.right_margin
                };

                // self.next_screen_pos later
                self.last_pos = self.next_pos;
                self.last_byte = text_bytes.end;

                if line_break {
                    self.next_screen_pos.0 = 0;
                    self.next_screen_pos.1 += 1;
                    self.next_pos.x = 0;
                    self.next_pos.y += 1;

                    return Some(Glyph {
                        glyph,
                        text_bytes,
                        screen_pos: (screen_pos.0 as u16, screen_pos.1 as u16),
                        screen_width,
                        line_break,
                        pos,
                    });
                } else if screen_pos.0 + screen_width as upos_type > right_margin {
                    // break before glyph

                    // after current grapheme
                    self.next_screen_pos.0 = screen_width as upos_type;
                    self.next_screen_pos.1 += 1;
                    self.next_pos.x += 1;
                    // next_pos.y doesn't change

                    self.next_glyph = Some(Glyph {
                        glyph: Cow::Owned(glyph.to_string()),
                        text_bytes: text_bytes.clone(),
                        screen_pos: (0, screen_pos.1 as u16 + 1),
                        screen_width,
                        line_break: false,
                        pos: pos,
                    });

                    glyph = if self.show_ctrl {
                        Cow::Borrowed("\u{2424}")
                    } else {
                        Cow::Borrowed("")
                    };
                    text_bytes = text_bytes.start..text_bytes.start;
                    // screen_pos is ok
                    screen_width = 1;
                    line_break = true;
                    pos = self.last_pos;

                    return Some(Glyph {
                        glyph,
                        text_bytes,
                        screen_pos: (screen_pos.0 as u16, screen_pos.1 as u16),
                        screen_width,
                        line_break,
                        pos,
                    });
                } else if screen_pos.0 > self.word_margin && glyph == " " {
                    // break after space

                    self.next_screen_pos.0 = 0;
                    self.next_screen_pos.1 += 1;
                    self.next_pos.x += 1;
                    // next_pos.y doesn't change

                    self.next_glyph = Some(Glyph {
                        glyph: if self.show_ctrl {
                            Cow::Borrowed("\u{2424}")
                        } else {
                            Cow::Borrowed("")
                        },
                        text_bytes: text_bytes.end..text_bytes.end,
                        screen_pos: (screen_pos.0 as u16 + 1, screen_pos.1 as u16),
                        screen_width: if self.show_ctrl { 1 } else { 0 },
                        line_break: true,
                        pos: pos,
                    });

                    return Some(Glyph {
                        glyph,
                        text_bytes,
                        screen_pos: (screen_pos.0 as u16, screen_pos.1 as u16),
                        screen_width,
                        line_break,
                        pos,
                    });
                } else {
                    self.next_screen_pos.0 += screen_width as upos_type;
                    self.next_pos.x += 1;

                    return Some(Glyph {
                        glyph,
                        text_bytes,
                        screen_pos: (screen_pos.0 as u16, screen_pos.1 as u16),
                        screen_width,
                        line_break,
                        pos,
                    });
                }
            } else {
                unreachable!()
            }
        }
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
    line_break: bool,
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
            line_break: true,
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

    /// Handle line-breaks. If false everything is treated as one line.
    pub(crate) fn set_line_break(&mut self, line_break: bool) {
        self.line_break = line_break;
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
        loop {
            let Some(grapheme) = self.iter.next() else {
                return None;
            };

            let mut glyph = Glyph {
                glyph: Default::default(),
                text_bytes: grapheme.text_bytes(),
                screen_pos: self.screen_pos,
                screen_width: 0,
                line_break: false,
                pos: self.pos,
            };

            self.remap(grapheme, &mut glyph);

            let pos = self.pos;
            let screen_pos = self.screen_pos;

            if glyph.line_break {
                self.screen_pos.0 = 0;
                self.screen_pos.1 += 1;
                self.pos.x = 0;
                self.pos.y += 1;
            } else {
                self.screen_pos.0 += glyph.screen_width;
                self.pos.x += 1;
            }

            // clip left
            if screen_pos.0 < self.screen_offset {
                if screen_pos.0 + glyph.screen_width > self.screen_offset {
                    // don't show partial glyphs, but show the space they need.
                    // avoids flickering when scrolling left/right.
                    return Some(Glyph {
                        glyph: Cow::Borrowed("\u{2203}"),
                        text_bytes: glyph.text_bytes,
                        screen_width: screen_pos.0 + glyph.screen_width - self.screen_offset,
                        line_break: glyph.line_break,
                        pos,
                        screen_pos: (0, screen_pos.1),
                    });
                } else {
                    // out left
                }
            } else if screen_pos.0 + glyph.screen_width > self.screen_offset + self.screen_width {
                if screen_pos.0 < self.screen_offset + self.screen_width {
                    // don't show partial glyphs, but show the space they need.
                    // avoids flickering when scrolling left/right.
                    return Some(Glyph {
                        glyph: Cow::Borrowed("\u{2203}"),
                        text_bytes: glyph.text_bytes,
                        screen_width: screen_pos.0 + glyph.screen_width
                            - (self.screen_offset + self.screen_width),
                        line_break: glyph.line_break,
                        pos,
                        screen_pos: (screen_pos.0 - self.screen_offset, screen_pos.1),
                    });
                } else {
                    // out right
                    if !self.line_break {
                        break;
                    }
                }
            } else {
                return Some(Glyph {
                    glyph: glyph.glyph,
                    text_bytes: glyph.text_bytes,
                    screen_width: glyph.screen_width,
                    line_break: glyph.line_break,
                    pos,
                    screen_pos: (screen_pos.0 - self.screen_offset, screen_pos.1),
                });
            }
        }

        None
    }
}

impl<'a, Iter> GlyphIter<Iter>
where
    Iter: Iterator<Item = Grapheme<'a>>,
{
    fn remap(&mut self, mut grapheme: Grapheme<'a>, glyph: &mut Glyph<'a>) {
        let cc = grapheme.grapheme();

        // remap grapheme
        if cc == "\n" || cc == "\r\n" {
            if self.line_break {
                glyph.line_break = true;
                glyph.screen_width = if self.show_ctrl { 1 } else { 0 };
                glyph.glyph = Cow::Borrowed(if self.show_ctrl { "\u{240A}" } else { "" });
            } else {
                glyph.line_break = false;
                glyph.screen_width = 1;
                glyph.glyph = Cow::Borrowed("\u{240A}");
            }
        } else if cc == "\t" {
            glyph.line_break = false;
            glyph.screen_width = self.tabs - (self.screen_pos.0 % self.tabs);
            glyph.glyph = Cow::Borrowed(if self.show_ctrl { "\u{2409}" } else { " " });
        } else if ("\x00".."\x20").contains(&cc) {
            glyph.line_break = false;
            glyph.screen_width = 1;

            // Control char unicode display replacement.
            static CONTROL_CHARS: [&str; 32] = [
                "\u{2400}", "\u{2401}", "\u{2402}", "\u{2403}", "\u{2404}", "\u{2405}", "\u{2406}",
                "\u{2407}", "\u{2408}", "\u{2409}", "\u{240A}", "\u{240B}", "\u{240C}", "\u{240D}",
                "\u{240E}", "\u{240F}", "\u{2410}", "\u{2411}", "\u{2412}", "\u{2413}", "\u{2414}",
                "\u{2415}", "\u{2416}", "\u{2417}", "\u{2418}", "\u{2419}", "\u{241A}", "\u{241B}",
                "\u{241C}", "\u{241D}", "\u{241E}", "\u{241F}",
            ];

            glyph.glyph = Cow::Borrowed(if self.show_ctrl {
                CONTROL_CHARS[cc.as_bytes()[0] as usize]
            } else {
                "\u{FFFD}"
            });
        } else {
            glyph.line_break = false;
            glyph.screen_width = unicode_display_width::width(cc) as u16;
            glyph.glyph = mem::take(&mut grapheme.grapheme);
        }
    }
}

#[cfg(test)]
mod test_str {
    use crate::grapheme::StrGraphemes;
    use crate::Cursor;

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

    #[test]
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
    use crate::grapheme::RopeGraphemes;
    use crate::Cursor;
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

    #[test]
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
             "
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
             "
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
}

#[cfg(test)]
mod test_glyph {
    use crate::grapheme::{GlyphIter, RopeGraphemes};
    use crate::TextPosition;
    use ropey::Rope;

    #[test]
    fn test_glyph1() {
        let s = Rope::from(
            r#"0123456789
abcdefghij
jklöjklöjk
uiopü+uiop"#,
        );
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let mut glyphs = GlyphIter::new(TextPosition::new(0, 0), r);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "0");
        assert_eq!(n.text_bytes(), 0..1);
        assert_eq!(n.screen_pos(), (0, 0));
        assert_eq!(n.pos(), TextPosition::new(0, 0));
        assert_eq!(n.screen_width(), 1);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "1");
        assert_eq!(n.text_bytes(), 1..2);
        assert_eq!(n.screen_pos(), (1, 0));
        assert_eq!(n.pos(), TextPosition::new(1, 0));
        assert_eq!(n.screen_width(), 1);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "2");
        assert_eq!(n.text_bytes(), 2..3);
        assert_eq!(n.screen_pos(), (2, 0));
        assert_eq!(n.pos(), TextPosition::new(2, 0));
        assert_eq!(n.screen_width(), 1);

        let n = glyphs.nth(7).unwrap();
        assert_eq!(n.glyph(), "");
        assert_eq!(n.text_bytes(), 10..11);
        assert_eq!(n.screen_pos(), (10, 0));
        assert_eq!(n.pos(), TextPosition::new(10, 0));
        assert_eq!(n.screen_width(), 0);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "a");
        assert_eq!(n.text_bytes(), 11..12);
        assert_eq!(n.screen_pos(), (0, 1));
        assert_eq!(n.pos(), TextPosition::new(0, 1));
        assert_eq!(n.screen_width(), 1);
    }

    #[test]
    fn test_glyph2() {
        // screen offset
        let s = Rope::from(
            r#"0123456789
abcdefghij
jklöjklöjk
uiopü+uiop"#,
        );
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let mut glyphs = GlyphIter::new(TextPosition::new(0, 0), r);
        glyphs.set_screen_offset(2);
        glyphs.set_screen_width(100);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "2");
        assert_eq!(n.text_bytes(), 2..3);
        assert_eq!(n.screen_pos(), (0, 0));
        assert_eq!(n.pos(), TextPosition::new(2, 0));
        assert_eq!(n.screen_width(), 1);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "3");
        assert_eq!(n.text_bytes(), 3..4);
        assert_eq!(n.screen_pos(), (1, 0));
        assert_eq!(n.pos(), TextPosition::new(3, 0));
        assert_eq!(n.screen_width(), 1);

        let n = glyphs.nth(6).unwrap();
        assert_eq!(n.glyph(), "");
        assert_eq!(n.text_bytes(), 10..11);
        assert_eq!(n.screen_pos(), (8, 0));
        assert_eq!(n.pos(), TextPosition::new(10, 0));
        assert_eq!(n.screen_width(), 0);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "c");
        assert_eq!(n.text_bytes(), 13..14);
        assert_eq!(n.screen_pos(), (0, 1));
        assert_eq!(n.pos(), TextPosition::new(2, 1));
        assert_eq!(n.screen_width(), 1);
    }

    #[test]
    fn test_glyph3() {
        // screen offset + width
        let s = Rope::from(
            r#"0123456789
abcdefghij
jklöjklöjk
uiopü+uiop"#,
        );
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let mut glyphs = GlyphIter::new(TextPosition::new(0, 0), r);
        glyphs.set_screen_offset(2);
        glyphs.set_screen_width(6);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "2");
        assert_eq!(n.text_bytes(), 2..3);
        assert_eq!(n.screen_pos(), (0, 0));
        assert_eq!(n.pos(), TextPosition::new(2, 0));
        assert_eq!(n.screen_width(), 1);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "3");
        assert_eq!(n.text_bytes(), 3..4);
        assert_eq!(n.screen_pos(), (1, 0));
        assert_eq!(n.pos(), TextPosition::new(3, 0));
        assert_eq!(n.screen_width(), 1);

        let n = glyphs.nth(2).unwrap();
        assert_eq!(n.glyph(), "6");

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "7");
        assert_eq!(n.text_bytes(), 7..8);
        assert_eq!(n.screen_pos(), (5, 0));
        assert_eq!(n.pos(), TextPosition::new(7, 0));
        assert_eq!(n.screen_width(), 1);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "c");
        assert_eq!(n.text_bytes(), 13..14);
        assert_eq!(n.screen_pos(), (0, 1));
        assert_eq!(n.pos(), TextPosition::new(2, 1));
        assert_eq!(n.screen_width(), 1);
    }

    #[test]
    fn test_glyph4() {
        // tabs
        let s = Rope::from(
            "012\t3456789
abcdefghij
jklöjklöjk
uiopü+uiop",
        );
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let mut glyphs = GlyphIter::new(TextPosition::new(0, 0), r);
        glyphs.set_screen_offset(2);
        glyphs.set_screen_width(100);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "2");
        assert_eq!(n.text_bytes(), 2..3);
        assert_eq!(n.screen_pos(), (0, 0));
        assert_eq!(n.pos(), TextPosition::new(2, 0));
        assert_eq!(n.screen_width(), 1);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), " ");
        assert_eq!(n.text_bytes(), 3..4);
        assert_eq!(n.screen_pos(), (1, 0));
        assert_eq!(n.pos(), TextPosition::new(3, 0));
        assert_eq!(n.screen_width(), 5);

        let n = glyphs.nth(7).unwrap();
        assert_eq!(n.glyph(), "");
        assert_eq!(n.text_bytes(), 11..12);
        assert_eq!(n.screen_pos(), (13, 0));
        assert_eq!(n.pos(), TextPosition::new(11, 0));
        assert_eq!(n.screen_width(), 0);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "c");
        assert_eq!(n.text_bytes(), 14..15);
        assert_eq!(n.screen_pos(), (0, 1));
        assert_eq!(n.pos(), TextPosition::new(2, 1));
        assert_eq!(n.screen_width(), 1);
    }

    #[test]
    fn test_glyph5() {
        // clipping wide
        let s = Rope::from(
            "0\t12345678\t9
abcdefghij
jklöjklöjk
uiopü+uiop",
        );
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let mut glyphs = GlyphIter::new(TextPosition::new(0, 0), r);
        glyphs.set_screen_offset(2);
        glyphs.set_screen_width(20);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "∃");
        assert_eq!(n.text_bytes(), 1..2);
        assert_eq!(n.screen_pos(), (0, 0));
        assert_eq!(n.pos(), TextPosition::new(1, 0));
        assert_eq!(n.screen_width(), 6);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "1");
        assert_eq!(n.text_bytes(), 2..3);
        assert_eq!(n.screen_pos(), (6, 0));
        assert_eq!(n.pos(), TextPosition::new(2, 0));
        assert_eq!(n.screen_width(), 1);

        let n = glyphs.nth(6).unwrap();
        assert_eq!(n.glyph(), "8");

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "∃");
        assert_eq!(n.text_bytes(), 10..11);
        assert_eq!(n.screen_pos(), (14, 0));
        assert_eq!(n.pos(), TextPosition::new(10, 0));
        assert_eq!(n.screen_width(), 2);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "c");
        assert_eq!(n.text_bytes(), 15..16);
        assert_eq!(n.screen_pos(), (0, 1));
        assert_eq!(n.pos(), TextPosition::new(2, 1));
        assert_eq!(n.screen_width(), 1);
    }
}
