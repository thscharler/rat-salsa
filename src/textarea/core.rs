use crate::textarea::graphemes::{rope_len, RopeGraphemesIdx};
#[allow(unused_imports)]
use log::debug;
use ropey::iter::Lines;
use ropey::{Rope, RopeSlice};
use std::cmp::{min, Ordering};
use std::fmt::{Debug, Formatter};
use std::iter::Skip;
use std::mem;
use std::slice::IterMut;

pub use crate::textarea::graphemes::RopeGraphemes;

/// Core for text editing.
#[derive(Debug, Clone)]
pub struct InputCore {
    /// Rope for text storage.
    value: Rope,
    /// Styles.
    styles: StyleMap,

    /// Line-break chars.
    line_break: String,

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
    pub start: (usize, usize),
    pub end: (usize, usize),
}

#[derive(Debug, Default, Clone)]
struct StyleMap {
    /// Vec of (range, style-idx)
    styles: Vec<(TextRange, usize)>,
}

#[derive(Debug)]
pub struct ScrolledIter<'a> {
    lines: Lines<'a>,
    offset: usize,
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
    pub fn expand_all(&self, it: Skip<IterMut<'_, (TextRange, usize)>>) {
        for (r, _s) in it {
            self._expand(&mut r.start);
            self._expand(&mut r.end);
        }
    }

    /// Modify all positions in place.
    #[inline]
    pub fn shrink_all(&self, it: Skip<IterMut<'_, (TextRange, usize)>>) {
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

    /// Return the modified position, if this range would shrink to nothing.
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

    /// Find all styles
    pub(crate) fn styles_after_mut(
        &mut self,
        pos: (usize, usize),
    ) -> Skip<IterMut<'_, (TextRange, usize)>> {
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

    /// Find all styles for the given position.
    ///
    pub(crate) fn styles_at(&self, pos: (usize, usize), result: &mut Vec<usize>) {
        match self.styles.binary_search_by(|v| v.0.ordering(pos)) {
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

                // collect all matching styles.
                result.clear();
                for i in i..self.styles.len() {
                    if self.styles[i].0.contains(pos) {
                        result.push(self.styles[i].1);
                    } else {
                        break;
                    }
                }
            }
            Err(_) => result.clear(),
        }
    }
}

impl<'a> Iterator for ScrolledIter<'a> {
    type Item = Skip<RopeGraphemes<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let s = self.lines.next()?;
        Some(RopeGraphemes::new(s).skip(self.offset))
    }
}

impl Default for InputCore {
    fn default() -> Self {
        Self {
            value: Default::default(),
            styles: Default::default(),
            line_break: "\n".to_string(),
            move_col: None,
            cursor: (0, 0),
            anchor: (0, 0),
        }
    }
}

impl InputCore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Extra column information for cursor movement.
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

    /// Set the cursor position.
    /// The value is capped to the number of text lines and the line-width.
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

    /// Sets the line-break to be used for insert.
    /// There is no auto-detection or conversion when setting
    /// the value.
    #[inline]
    pub fn set_line_break(&mut self, br: String) {
        self.line_break = br;
    }

    /// Set the text.
    /// Resets the selection and any styles.
    pub fn set_value<S: AsRef<str>>(&mut self, s: S) {
        self.set_value_rope(Rope::from_str(s.as_ref()));
    }

    /// Set the text value as a Rope.
    /// Resets all internal state.
    #[inline]
    pub fn set_value_rope(&mut self, s: Rope) {
        self.value = s;
        self.cursor = (0, 0);
        self.anchor = (0, 0);
        self.move_col = None;
        self.styles.clear_styles();
    }

    /// Text value.
    #[inline]
    pub fn value(&self) -> String {
        String::from(&self.value)
    }

    /// Borrow the rope
    #[inline]
    pub fn value_rope(&self) -> &Rope {
        &self.value
    }

    /// A range of the text as RopeSlice.
    pub fn value_range(&self, range: TextRange) -> Option<RopeSlice<'_>> {
        let s = self.char_at(range.start)?;
        let e = self.char_at(range.end)?;
        Some(self.value.slice(s..e))
    }

    /// Value as Bytes iterator.
    pub fn value_as_bytes(&self) -> ropey::iter::Bytes<'_> {
        self.value.bytes()
    }

    /// Value as Chars iterator.
    pub fn value_as_chars(&self) -> ropey::iter::Chars<'_> {
        self.value.chars()
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

    /// Style map.
    #[inline]
    pub fn styles(&self) -> &[(TextRange, usize)] {
        &self.styles.styles
    }

    /// Finds all styles for the given position.
    ///
    /// Returns the indexes into the style vec.
    #[inline]
    pub fn styles_at(&self, pos: (usize, usize), result: &mut Vec<usize>) {
        self.styles.styles_at(pos, result)
    }

    /// Returns a line as an iterator over the graphemes for the line.
    /// This contains the \n at the end.
    pub fn line(&self, n: usize) -> Option<RopeGraphemes<'_>> {
        let mut lines = self.value.get_lines_at(n)?;
        let line = lines.next();
        if let Some(line) = line {
            Some(RopeGraphemes::new(line))
        } else {
            Some(RopeGraphemes::new(RopeSlice::from("")))
        }
    }

    /// Returns a line as an iterator over the graphemes for the line.
    /// This contains the \n at the end.
    pub fn line_idx(&self, n: usize) -> Option<RopeGraphemesIdx<'_>> {
        let mut lines = self.value.get_lines_at(n)?;
        let line = lines.next();
        if let Some(line) = line {
            Some(RopeGraphemesIdx::new(line))
        } else {
            Some(RopeGraphemesIdx::new(RopeSlice::from("")))
        }
    }

    /// Line width as grapheme count.
    pub fn line_width(&self, n: usize) -> Option<usize> {
        let mut lines = self.value.get_lines_at(n)?;
        let line = lines.next();
        if let Some(line) = line {
            Some(rope_len(line))
        } else {
            Some(0)
        }
    }

    /// Number of lines.
    #[inline]
    pub fn len_lines(&self) -> usize {
        self.value.len_lines()
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

    /// Iterate over text-lines, starting at offset.
    #[inline]
    pub fn iter_lines(&self, line_offset: usize) -> Lines<'_> {
        self.value.lines_at(line_offset)
    }

    /// Iterate over the text, shifted by the offset.
    #[inline]
    pub fn iter_scrolled(&self, offset: (usize, usize)) -> ScrolledIter<'_> {
        let Some(l) = self.value.get_lines_at(offset.1) else {
            panic!("invalid offset {:?} value {:?}", offset, self.value);
        };
        ScrolledIter {
            lines: l,
            offset: offset.0,
        }
    }

    /// Find next word.
    pub fn next_word_boundary(&self, pos: (usize, usize)) -> Option<(usize, usize)> {
        let mut char_pos = self.char_at(pos)?;

        let chars_after = self.value.slice(char_pos..);
        let mut it = chars_after.chars_at(0);
        let mut init = true;
        loop {
            let Some(c) = it.next() else {
                break;
            };

            if init {
                if !c.is_whitespace() {
                    init = false;
                }
            } else {
                if c.is_whitespace() {
                    break;
                }
            }

            char_pos += 1;
        }

        self.char_pos(char_pos)
    }

    /// Find prev word.
    pub fn prev_word_boundary(&self, pos: (usize, usize)) -> Option<(usize, usize)> {
        let mut char_pos = self.char_at(pos)?;

        let chars_before = self.value.slice(..char_pos);
        let mut it = chars_before.chars_at(chars_before.len_chars());
        let mut init = true;
        loop {
            let Some(c) = it.prev() else {
                break;
            };

            if init {
                if !c.is_whitespace() {
                    init = false;
                }
            } else {
                if c.is_whitespace() {
                    break;
                }
            }

            char_pos -= 1;
        }

        self.char_pos(char_pos)
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

    /// Byte position to grapheme position.
    pub fn byte_pos(&self, byte: usize) -> Option<(usize, usize)> {
        let Ok(y) = self.value.try_byte_to_line(byte) else {
            return None;
        };
        let mut x = 0;
        let byte_y = self.value.try_line_to_byte(y).expect("valid_y");

        let mut it_line = self.line_idx(y).expect("valid_y");
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
        let mut it_line = self.line_idx(pos.1).expect("valid_line");
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

    /// Insert a character.
    pub fn insert_char(&mut self, pos: (usize, usize), c: char) {
        if c == '\n' {
            self.insert_newline(pos);
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

        let insert = TextRange::new((pos.0, pos.1), (pos.0 + new_len - old_len, pos.1));
        insert.expand_all(self.styles.styles_after_mut(pos));
        self.anchor = insert.expand(self.anchor);
        self.cursor = insert.expand(self.cursor);
    }

    // todo: insert_str

    /// Insert a line break.
    pub fn insert_newline(&mut self, pos: (usize, usize)) {
        let Some(char_pos) = self.char_at(pos) else {
            panic!("invalid pos {:?} value {:?}", pos, self.value);
        };

        self.value.insert(char_pos, &self.line_break);

        let insert = TextRange::new((pos.0, pos.1), (0, pos.1 + 1));

        insert.expand_all(self.styles.styles_after_mut(pos));
        self.anchor = insert.expand(self.anchor);
        self.cursor = insert.expand(self.cursor);
    }

    pub fn remove(&mut self, range: TextRange) {
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
