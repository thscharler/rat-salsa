use crate::text::graphemes::{
    rope_line_len, str_line_len, GlyphIter, RopeGraphemes, RopeGraphemesIdx,
};
#[allow(unused_imports)]
use log::debug;
use ropey::{Rope, RopeSlice};
use std::cmp::{min, Ordering};
use std::fmt::{Debug, Formatter};
use std::iter::repeat_with;
use std::mem;
use std::ops::RangeBounds;
use unicode_segmentation::UnicodeSegmentation;

/// Core for text editing.
#[derive(Debug, Clone)]
pub struct TextAreaCore {
    /// Rope for text storage.
    value: Rope,
    /// Styles.
    styles: StyleMap,

    /// Line-break chars.
    newline: String,
    /// Tab width.
    tabs: u16,

    /// Secondary column, remembered for moving up/down.
    move_col: Option<usize>,
    /// Cursor
    cursor: (usize, usize),
    /// Anchor for the selection.
    anchor: (usize, usize),
}

/// Range for text ranges.
#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub struct TextRange {
    /// column, row
    pub start: (usize, usize),
    /// column, row
    pub end: (usize, usize),
}

#[derive(Debug, Default, Clone)]
struct StyleMap {
    /// Vec of (range, style-idx)
    styles: Vec<(TextRange, usize)>,
}

#[derive(Debug, Clone)]
struct StyleMapIter<'a> {
    styles: &'a [(TextRange, usize)],
    filter_pos: (usize, usize),
    idx: usize,
}

impl Debug for TextRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TextRange  {}|{}-{}|{}",
            self.start.0, self.start.1, self.end.0, self.end.1
        )
    }
}

impl TextRange {
    /// New text range.
    ///
    /// Panic
    /// Panics if start > end.
    pub fn new(start: (usize, usize), end: (usize, usize)) -> Self {
        // reverse the args, then it works.
        if (start.1, start.0) > (end.1, end.0) {
            panic!("start {:?} > end {:?}", start, end);
        }
        TextRange { start, end }
    }

    /// Start position
    #[inline]
    pub fn start(&self) -> (usize, usize) {
        self.start
    }

    /// End position
    #[inline]
    pub fn end(&self) -> (usize, usize) {
        self.end
    }

    /// Empty range
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Range contains the given position.
    #[inline]
    pub fn contains(&self, pos: (usize, usize)) -> bool {
        self.ordering(pos) == Ordering::Equal
    }

    /// Range contains the other range.
    #[inline(always)]
    pub fn contains_range(&self, range: TextRange) -> bool {
        self.ordering(range.start) == Ordering::Equal
            && self.ordering_inclusive(range.end) == Ordering::Equal
    }

    /// What place is the range respective to the given position.
    #[inline(always)]
    #[allow(clippy::comparison_chain)]
    pub fn ordering(&self, pos: (usize, usize)) -> Ordering {
        if pos.1 < self.start.1 {
            return Ordering::Greater;
        } else if pos.1 == self.start.1 {
            if pos.0 < self.start.0 {
                return Ordering::Greater;
            }
        }

        if pos.1 < self.end.1 {
            return Ordering::Equal;
        } else if pos.1 == self.end.1 {
            if pos.0 < self.end.0 {
                return Ordering::Equal;
            }
        }

        Ordering::Less

        // SURPRISE: contrary to ordering_inclusive the code below
        //           takes the same time as the above in debug mode.

        // // reverse the args, then tuple cmp it works.
        // if (pos.1, pos.0) < (self.start.1, self.start.0) {
        //     Ordering::Greater
        // } else if (pos.1, pos.0) < (self.end.1, self.end.0) {
        //     Ordering::Equal
        // } else {
        //     Ordering::Less
        // }
    }

    /// What place is the range respective to the given position.
    /// This one includes the `range.end`.
    #[inline(always)]
    #[allow(clippy::comparison_chain)]
    pub fn ordering_inclusive(&self, pos: (usize, usize)) -> Ordering {
        if pos.1 < self.start.1 {
            return Ordering::Greater;
        } else if pos.1 == self.start.1 {
            if pos.0 < self.start.0 {
                return Ordering::Greater;
            }
        }

        if pos.1 < self.end.1 {
            return Ordering::Equal;
        } else if pos.1 == self.end.1 {
            if pos.0 <= self.end.0 {
                return Ordering::Equal;
            }
        }

        Ordering::Less

        // SURPRISE: above is pretty much faster than that: ???
        //           at least in debug mode...

        // // reverse the args, then tuple cmp it works.
        // if (pos.1, pos.0) < (self.start.1, self.start.0) {
        //     Ordering::Greater
        // } else if (pos.1, pos.0) <= (self.end.1, self.end.0) {
        //     Ordering::Equal
        // } else {
        //     Ordering::Less
        // }
    }

    /// Modify all positions in place.
    #[inline]
    pub fn expand_all<'a>(&self, it: impl Iterator<Item = &'a mut (TextRange, usize)>) {
        for (r, _s) in it {
            self._expand(&mut r.start);
            self._expand(&mut r.end);
        }
    }

    /// Modify all positions in place.
    #[inline]
    pub fn shrink_all<'a>(&self, it: impl Iterator<Item = &'a mut (TextRange, usize)>) {
        for (r, _s) in it {
            self._shrink(&mut r.start);
            self._shrink(&mut r.end);
        }
    }

    /// Return the modified position, as if this range expanded from its
    /// start to its full expansion.
    #[inline]
    pub fn expand(&self, pos: (usize, usize)) -> (usize, usize) {
        let mut tmp = pos;
        self._expand(&mut tmp);
        tmp
    }

    /// Return the modified position, as if this range would shrink to nothing.
    #[inline]
    pub fn shrink(&self, pos: (usize, usize)) -> (usize, usize) {
        let mut tmp = pos;
        self._shrink(&mut tmp);
        tmp
    }

    #[inline(always)]
    #[allow(clippy::comparison_chain)]
    fn _expand(&self, pos: &mut (usize, usize)) {
        let delta_lines = self.end.1 - self.start.1;

        // comparing only the starting position.
        // the range doesn't exist yet.
        // have to flip the positions for tuple comparison
        match (self.start.1, self.start.0).cmp(&(pos.1, pos.0)) {
            Ordering::Greater => {
                // noop
            }
            Ordering::Equal => {
                *pos = self.end;
            }
            Ordering::Less => {
                if pos.1 > self.start.1 {
                    pos.1 += delta_lines;
                } else if pos.1 == self.start.1 {
                    if pos.0 >= self.start.0 {
                        pos.0 = pos.0 - self.start.0 + self.end.0;
                        pos.1 += delta_lines;
                    }
                }
            }
        }
    }

    /// Return the modified position, if this range would shrink to nothing.
    #[inline(always)]
    #[allow(clippy::comparison_chain)]
    fn _shrink(&self, pos: &mut (usize, usize)) {
        let delta_lines = self.end.1 - self.start.1;
        match self.ordering_inclusive(*pos) {
            Ordering::Greater => {
                // noop
            }
            Ordering::Equal => {
                *pos = self.start;
            }
            Ordering::Less => {
                if pos.1 > self.end.1 {
                    pos.1 -= delta_lines;
                } else if pos.1 == self.end.1 {
                    if pos.0 >= self.end.0 {
                        pos.0 = pos.0 - self.end.0 + self.start.0;
                        pos.1 -= delta_lines;
                    }
                }
            }
        }
    }
}

// This needs its own impl, because the order is exactly wrong.
// For any sane range I'd need (row,col) but what I got is (col,row).
// Need this to conform with the rest of ratatui ...
impl PartialOrd for TextRange {
    #[allow(clippy::comparison_chain)]
    #[allow(clippy::non_canonical_partial_ord_impl)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // reverse the args, then it works.
        let start = (self.start.1, self.start.0);
        let end = (self.end.1, self.end.0);
        let ostart = (other.start.1, other.start.0);
        let oend = (other.end.1, other.end.0);

        if start < ostart {
            Some(Ordering::Less)
        } else if start > ostart {
            Some(Ordering::Greater)
        } else {
            if end < oend {
                Some(Ordering::Less)
            } else if end > oend {
                Some(Ordering::Greater)
            } else {
                Some(Ordering::Equal)
            }
        }
    }
}

impl Ord for TextRange {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).expect("order")
    }
}

impl<'a> StyleMapIter<'a> {
    fn new(styles: &'a [(TextRange, usize)], pos: (usize, usize)) -> Self {
        match styles.binary_search_by(|v| v.0.ordering(pos)) {
            Ok(mut i) => {
                // binary-search found *some* matching style, we need all of them.
                // this finds the first one.
                loop {
                    if i == 0 {
                        break;
                    }
                    if !styles[i - 1].0.contains(pos) {
                        break;
                    }
                    i -= 1;
                }

                Self {
                    styles,
                    filter_pos: pos,
                    idx: i,
                }
            }
            Err(_) => Self {
                styles,
                filter_pos: pos,
                idx: styles.len(),
            },
        }
    }
}

impl<'a> Iterator for StyleMapIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.idx;
        if idx < self.styles.len() {
            if self.styles[idx].0.contains(self.filter_pos) {
                self.idx += 1;
                Some(self.styles[idx].1)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl StyleMap {
    /// Remove all styles.
    pub(crate) fn clear_styles(&mut self) {
        self.styles.clear();
    }

    /// Add a text-style for a range.
    ///
    /// The same range can be added again with a different style.
    /// Overlapping regions get the merged style.
    pub(crate) fn add_style(&mut self, range: TextRange, style: usize) {
        let stylemap = (range, style);
        match self.styles.binary_search(&stylemap) {
            Ok(_) => {
                // noop
            }
            Err(idx) => {
                self.styles.insert(idx, stylemap);
            }
        }
    }

    /// Remove a text-style for a range.
    ///
    /// This must match exactly in range and style to be removed.
    pub(crate) fn remove_style(&mut self, range: TextRange, style: usize) {
        let stylemap = (range, style);
        match self.styles.binary_search(&stylemap) {
            Ok(idx) => {
                self.styles.remove(idx);
            }
            Err(_) => {
                // noop
            }
        }
    }

    /// Find styles that touch the given pos and all styles after that point.
    pub(crate) fn styles_after_mut(
        &mut self,
        pos: (usize, usize),
    ) -> impl Iterator<Item = &mut (TextRange, usize)> {
        let first = match self.styles.binary_search_by(|v| v.0.ordering(pos)) {
            Ok(mut i) => {
                // binary-search found *some* matching style, we need all of them.
                // this finds the first one.
                loop {
                    if i == 0 {
                        break;
                    }
                    if !self.styles[i - 1].0.contains(pos) {
                        break;
                    }
                    i -= 1;
                }
                i
            }
            Err(i) => i,
        };

        self.styles.iter_mut().skip(first)
    }

    /// Find all styles that touch the given position.
    pub(crate) fn styles_at(&self, pos: (usize, usize)) -> impl Iterator<Item = usize> + '_ {
        StyleMapIter::new(&self.styles, pos)
    }
}

impl Default for TextAreaCore {
    fn default() -> Self {
        Self {
            value: Default::default(),
            styles: Default::default(),
            newline: "\n".to_string(),
            tabs: 8,
            move_col: None,
            cursor: (0, 0),
            anchor: (0, 0),
        }
    }
}

impl TextAreaCore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Extra column information for cursor movement.
    ///
    /// The cursor position is capped to the current line length, so if you
    /// move up one row, you might end at a position left of the current column.
    /// If you move up once more you want to return to the original position.
    /// That's what is stored here.
    #[inline]
    pub fn set_move_col(&mut self, col: Option<usize>) {
        self.move_col = col;
    }

    /// Extra column information for cursor movement.
    #[inline]
    pub fn move_col(&mut self) -> Option<usize> {
        self.move_col
    }

    /// Sets the line ending to be used for insert.
    /// There is no auto-detection or conversion done for set_value().
    ///
    /// Caution: If this doesn't match the line ending used in the value, you
    /// will get a value with mixed line endings.
    #[inline]
    pub fn set_newline(&mut self, br: String) {
        self.newline = br;
    }

    /// Line ending used for insert.
    #[inline]
    pub fn newline(&self) -> &str {
        &self.newline
    }

    /// Set the tab-width.
    /// Default is 8.
    #[inline]
    pub fn set_tab_width(&mut self, tabs: u16) {
        self.tabs = tabs;
    }

    /// Tab-width
    #[inline]
    pub fn tab_width(&self) -> u16 {
        self.tabs
    }

    /// Clear styles.
    #[inline]
    pub fn clear_styles(&mut self) {
        self.styles.clear_styles();
    }

    /// Add a style for the given range.
    ///
    /// What is given here is the index into the Vec with the actual Styles.
    /// Those are set at the widget.
    #[inline]
    pub fn add_style(&mut self, range: TextRange, style: usize) {
        self.styles.add_style(range, style);
    }

    /// Remove a style for the given range.
    ///
    /// Range and style must match to be removed.
    #[inline]
    pub fn remove_style(&mut self, range: TextRange, style: usize) {
        self.styles.remove_style(range, style);
    }

    /// Style map.
    #[inline]
    pub fn styles(&self) -> &[(TextRange, usize)] {
        &self.styles.styles
    }

    /// Finds all styles for the given position.
    #[inline]
    pub fn styles_at(&self, pos: (usize, usize)) -> impl Iterator<Item = usize> + '_ {
        self.styles.styles_at(pos)
    }

    /// Set the cursor position.
    /// The value is capped to the number of text lines and the line-width for the given line.
    /// Returns true, if the cursor actually changed.
    pub fn set_cursor(&mut self, mut cursor: (usize, usize), extend_selection: bool) -> bool {
        let old_cursor = self.cursor;
        let old_anchor = self.anchor;

        let (mut cx, mut cy) = cursor;
        cy = min(cy, self.len_lines() - 1);
        cx = min(cx, self.line_width(cy).expect("valid_line"));

        cursor = (cx, cy);

        self.cursor = cursor;

        if !extend_selection {
            self.anchor = cursor;
        }

        old_cursor != self.cursor || old_anchor != self.anchor
    }

    /// Cursor position.
    #[inline]
    pub fn cursor(&self) -> (usize, usize) {
        self.cursor
    }

    /// Selection anchor.
    #[inline]
    pub fn anchor(&self) -> (usize, usize) {
        self.anchor
    }

    /// Set the text.
    /// Resets the selection and any styles.
    pub fn set_value<S: AsRef<str>>(&mut self, s: S) {
        self.set_rope(Rope::from_str(s.as_ref()));
    }

    /// Copy of the text value.
    #[inline]
    pub fn value(&self) -> String {
        String::from(&self.value)
    }

    /// Set the text value as a Rope.
    /// Resets the selection and any styles.
    #[inline]
    pub fn set_rope(&mut self, s: Rope) {
        self.value = s;
        self.cursor = (0, 0);
        self.anchor = (0, 0);
        self.move_col = None;
        self.styles.clear_styles();
    }

    /// Access the underlying Rope with the text value.
    #[inline]
    pub fn rope(&self) -> &Rope {
        &self.value
    }

    /// A range of the text as RopeSlice.
    pub fn text_slice(&self, range: TextRange) -> Option<RopeSlice<'_>> {
        let s = self.char_at(range.start)?;
        let e = self.char_at(range.end)?;
        Some(self.value.slice(s..e))
    }

    /// Value as Bytes iterator.
    pub fn byte_slice<R>(&self, byte_range: R) -> RopeSlice<'_>
    where
        R: RangeBounds<usize>,
    {
        self.value.byte_slice(byte_range)
    }

    /// Value as Bytes iterator.
    pub fn bytes(&self) -> impl Iterator<Item = u8> + '_ {
        self.value.bytes()
    }

    /// Value as Chars iterator.
    pub fn char_slice<R>(&self, char_range: R) -> RopeSlice<'_>
    where
        R: RangeBounds<usize>,
    {
        self.value.slice(char_range)
    }

    /// Value as Chars iterator.
    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.value.chars()
    }

    /// Line as RopeSlice
    #[inline]
    pub fn line_at(&self, n: usize) -> Option<RopeSlice<'_>> {
        self.value.get_line(n)
    }

    /// Iterate over text-lines, starting at offset.
    #[inline]
    pub fn lines_at(&self, line_offset: usize) -> impl Iterator<Item = RopeSlice<'_>> {
        self.value.lines_at(line_offset)
    }

    /// Iterator for the glyphs of a given line.
    /// Glyphs here a grapheme + display length.
    #[inline]
    pub fn line_glyphs(&self, n: usize) -> Option<GlyphIter<'_>> {
        let mut lines = self.value.get_lines_at(n)?;
        if let Some(line) = lines.next() {
            let mut it = GlyphIter::new(line);
            it.set_tabs(self.tabs);
            Some(it)
        } else {
            None
        }
    }

    /// Returns a line as an iterator over the graphemes for the line.
    /// This contains the \n at the end.
    #[inline]
    pub fn line_graphemes(&self, n: usize) -> Option<impl Iterator<Item = RopeSlice<'_>>> {
        let mut lines = self.value.get_lines_at(n)?;
        if let Some(line) = lines.next() {
            Some(RopeGraphemes::new(line))
        } else {
            None
        }
    }

    /// Iterator for the chars of a given line.
    #[inline]
    pub fn line_chars(&self, n: usize) -> Option<impl Iterator<Item = char> + '_> {
        let mut lines = self.value.get_lines_at(n)?;
        if let Some(line) = lines.next() {
            Some(line.chars())
        } else {
            None
        }
    }

    /// Iterator for the bytes of a given line.
    #[inline]
    pub fn line_bytes(&self, n: usize) -> Option<impl Iterator<Item = u8> + '_> {
        let mut lines = self.value.get_lines_at(n)?;
        if let Some(line) = lines.next() {
            Some(line.bytes())
        } else {
            None
        }
    }

    /// Line width as grapheme count. Excludes the terminating '\n'.
    #[inline]
    pub fn line_width(&self, n: usize) -> Option<usize> {
        let mut lines = self.value.get_lines_at(n)?;
        let line = lines.next();
        if let Some(line) = line {
            Some(rope_line_len(line))
        } else {
            Some(0)
        }
    }

    /// Reset.
    #[inline]
    pub fn clear(&mut self) -> bool {
        if self.is_empty() {
            false
        } else {
            self.set_value("");
            true
        }
    }

    /// Empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.value.len_bytes() == 0
    }

    /// Number of lines.
    #[inline]
    pub fn len_lines(&self) -> usize {
        self.value.len_lines()
    }

    /// Any text selection.
    #[inline]
    pub fn has_selection(&self) -> bool {
        self.anchor != self.cursor
    }

    #[inline]
    pub fn set_selection(&mut self, range: TextRange) -> bool {
        let old_selection = self.selection();

        self.set_cursor(range.start, false);
        self.set_cursor(range.end, true);

        old_selection != self.selection()
    }

    #[inline]
    pub fn select_all(&mut self) -> bool {
        let old_selection = self.selection();

        self.set_cursor((0, 0), false);
        let last = self.len_lines() - 1;
        let last_width = self.line_width(last).expect("valid_last_line");
        self.set_cursor((last_width, last), true);

        old_selection != self.selection()
    }

    /// Returns the selection as TextRange.
    pub fn selection(&self) -> TextRange {
        #[allow(clippy::comparison_chain)]
        if self.cursor.1 < self.anchor.1 {
            TextRange {
                start: self.cursor,
                end: self.anchor,
            }
        } else if self.cursor.1 > self.anchor.1 {
            TextRange {
                start: self.anchor,
                end: self.cursor,
            }
        } else {
            if self.cursor.0 < self.anchor.0 {
                TextRange {
                    start: self.cursor,
                    end: self.anchor,
                }
            } else {
                TextRange {
                    start: self.anchor,
                    end: self.cursor,
                }
            }
        }
    }

    /// Len in chars
    pub fn len_chars(&self) -> usize {
        self.value.len_chars()
    }

    /// Len in bytes
    pub fn len_bytes(&self) -> usize {
        self.value.len_bytes()
    }

    /// Char position to grapheme position.
    pub fn char_pos(&self, char_pos: usize) -> Option<(usize, usize)> {
        let Ok(byte_pos) = self.value.try_char_to_byte(char_pos) else {
            return None;
        };
        self.byte_pos(byte_pos)
    }

    /// Returns a line as an iterator over the graphemes for the line.
    /// This contains the \n at the end.
    /// Returns byte-start and byte-end position and the grapheme.
    #[inline]
    fn line_grapheme_idx(
        &self,
        n: usize,
    ) -> Option<impl Iterator<Item = ((usize, usize), RopeSlice<'_>)>> {
        let mut lines = self.value.get_lines_at(n)?;
        let line = lines.next();
        if let Some(line) = line {
            Some(RopeGraphemesIdx::new(line))
        } else {
            None
        }
    }

    /// Byte position to grapheme position.
    /// Returns the position that contains the given byte index.
    pub fn byte_pos(&self, byte: usize) -> Option<(usize, usize)> {
        let Ok(y) = self.value.try_byte_to_line(byte) else {
            return None;
        };
        let mut x = 0;
        let byte_y = self.value.try_line_to_byte(y).expect("valid_y");

        let mut it_line = self.line_grapheme_idx(y).expect("valid_y");
        loop {
            let Some(((sb, _eb), _cc)) = it_line.next() else {
                break;
            };
            if byte_y + sb >= byte {
                break;
            }
            x += 1;
        }

        Some((x, y))
    }

    /// Grapheme position to byte position.
    /// This is the (start,end) position of the single grapheme after pos.
    pub fn byte_at(&self, pos: (usize, usize)) -> Option<(usize, usize)> {
        let Ok(line_byte) = self.value.try_line_to_byte(pos.1) else {
            return None;
        };

        let len_bytes = self.value.len_bytes();
        let mut it_line = self.line_grapheme_idx(pos.1).expect("valid_line");
        let mut x = -1isize;
        let mut last_eb = 0;
        loop {
            let (sb, eb, last) = if let Some((v, _)) = it_line.next() {
                x += 1;
                last_eb = v.1;
                (v.0, v.1, false)
            } else {
                (last_eb, last_eb, true)
            };

            if pos.0 == x as usize {
                return Some((line_byte + sb, line_byte + eb));
            }
            // one past the end is ok.
            if pos.0 == (x + 1) as usize && line_byte + eb == len_bytes {
                return Some((line_byte + eb, line_byte + eb));
            }
            if last {
                return None;
            }
        }
    }

    /// Returns the first char position for the grapheme position.
    pub fn char_at(&self, pos: (usize, usize)) -> Option<usize> {
        let (byte_pos, _) = self.byte_at(pos)?;
        Some(
            self.value
                .try_byte_to_char(byte_pos)
                .expect("valid_byte_pos"),
        )
    }

    /// Insert some text.
    pub fn insert_str(&mut self, pos: (usize, usize), t: &str) {
        let Some(char_pos) = self.char_at(pos) else {
            panic!("invalid pos {:?} value {:?}", pos, self.value);
        };

        // dissect t
        let mut line_breaks = 0;
        let mut current = String::new();
        let mut state = 0;
        for g in t.graphemes(true) {
            current.push_str(g);

            if g == "\n" || g == "\r\n" {
                line_breaks += 1;
                if state == 0 {
                    current = String::new();
                    state = 1;
                } else {
                    current.clear();
                }
            }
        }
        let last = current;

        let insert = if line_breaks > 0 {
            // split insert line
            let (split_byte, _) = self.byte_at(pos).expect("valid_pos");
            let line = self.line_bytes(pos.1).expect("valid_pos");
            let mut line_second = Vec::new();
            for b in line.skip(split_byte) {
                line_second.push(b);
            }
            let line_second = String::from_utf8(line_second).expect("valid_str");

            // this should cope with all unicode joins.
            let old_len = str_line_len(&line_second);
            let mut new_second = last;
            new_second.push_str(&line_second);
            let new_len = str_line_len(&new_second);

            self.value.insert(char_pos, t);

            TextRange::new(pos, (new_len - old_len, pos.1 + line_breaks))
        } else {
            // no way to know if the insert text combines with a surrounding char.
            // the difference of the graphem len seems safe though.
            let old_len = self.line_width(pos.1).expect("valid_pos");
            self.value.insert(char_pos, t);
            let new_len = self.line_width(pos.1).expect("valid_pos");

            TextRange::new(pos, (pos.0 + new_len - old_len, pos.1))
        };

        insert.expand_all(self.styles.styles_after_mut(pos));
        self.anchor = insert.expand(self.anchor);
        self.cursor = insert.expand(self.cursor);
    }

    /// Insert a character.
    pub fn insert_tab(&mut self, pos: (usize, usize)) {
        let n = self.tabs as usize - pos.0 % self.tabs as usize;
        let tabs = repeat_with(|| ' ').take(n).collect::<String>();
        self.insert_str(pos, &tabs);
    }

    /// Insert a character.
    pub fn insert_char(&mut self, pos: (usize, usize), c: char) {
        if c == '\n' {
            self.insert_newline(pos);
            return;
        } else if c == '\t' {
            self.insert_tab(pos);
            return;
        }

        let Some(char_pos) = self.char_at(pos) else {
            panic!("invalid pos {:?} value {:?}", pos, self.value);
        };

        // no way to know if the new char combines with a surrounding char.
        // the difference of the graphem len seems safe though.
        let old_len = self.line_width(pos.1).expect("valid_pos");
        self.value.insert_char(char_pos, c);
        let new_len = self.line_width(pos.1).expect("valid_pos");

        let insert = TextRange::new(pos, (pos.0 + new_len - old_len, pos.1));
        insert.expand_all(self.styles.styles_after_mut(pos));
        self.anchor = insert.expand(self.anchor);
        self.cursor = insert.expand(self.cursor);
    }

    /// Insert a line break.
    pub fn insert_newline(&mut self, pos: (usize, usize)) {
        let Some(char_pos) = self.char_at(pos) else {
            panic!("invalid pos {:?} value {:?}", pos, self.value);
        };

        self.value.insert(char_pos, &self.newline);

        let insert = TextRange::new((pos.0, pos.1), (0, pos.1 + 1));

        insert.expand_all(self.styles.styles_after_mut(pos));
        self.anchor = insert.expand(self.anchor);
        self.cursor = insert.expand(self.cursor);
    }

    /// Remove the given range.
    pub fn remove_range(&mut self, range: TextRange) {
        let Some(start_pos) = self.char_at(range.start) else {
            panic!("invalid range {:?} value {:?}", range, self.value);
        };
        let Some(end_pos) = self.char_at(range.end) else {
            panic!("invalid range {:?} value {:?}", range, self.value);
        };

        self.value.remove(start_pos..end_pos);

        // remove deleted styles.
        // this is not a simple range, so filter+collect seems ok.
        let styles = mem::take(&mut self.styles.styles);
        self.styles.styles = styles
            .into_iter()
            .filter(|(r, _)| !range.contains_range(*r))
            .collect();

        range.shrink_all(self.styles.styles_after_mut(range.start));
        self.anchor = range.shrink(self.anchor);
        self.cursor = range.shrink(self.anchor);
    }
}
