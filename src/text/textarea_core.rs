use crate::text::graphemes::{
    rope_line_len, str_line_len, RopeGlyphIter, RopeGraphemes, RopeGraphemesIdx,
};
use crate::text::undo::{UndoBuffer, UndoEntry, UndoVec};
#[allow(unused_imports)]
use log::debug;
use ratatui::layout::Position;
use ropey::{Rope, RopeSlice};
use std::any::Any;
use std::cmp::{min, Ordering};
use std::fmt::{Debug, Formatter};
use std::mem;
use std::ops::RangeBounds;

/// Core for text editing.
#[derive(Debug)]
pub struct TextAreaCore {
    /// Rope for text storage.
    value: Rope,
    /// Styles.
    styles: StyleMap,

    /// Undo-Buffer.
    undo: Option<Box<dyn UndoBuffer>>,

    /// Line-break chars.
    newline: String,
    /// Tab width.
    tabs: u16,
    /// Expand tabs
    expand_tabs: bool,

    /// Secondary column, remembered for moving up/down.
    move_col: Option<usize>,
    /// Cursor
    cursor: (usize, usize),
    /// Anchor for the selection.
    anchor: (usize, usize),

    /// temp string
    buf: String,
}

/// Range for text ranges.
#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub struct TextRange {
    /// column, row
    pub start: (usize, usize),
    /// column, row
    pub end: (usize, usize),
}

// Helper for lexical comparison of positions.
// Switches x and y compared to regular ratatui positions.
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct LexicalPosition {
    y: usize,
    x: usize,
}

impl From<(usize, usize)> for LexicalPosition {
    fn from(value: (usize, usize)) -> Self {
        Self {
            y: value.1,
            x: value.0,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StyledRange {
    pub range: TextRange,
    pub style: usize,
}

#[derive(Debug, Default, Clone)]
struct StyleMap {
    /// Vec of (range, style-idx)
    styles: Vec<StyledRange>,
}

#[derive(Debug, Clone)]
struct StyleMapIter<'a> {
    styles: &'a [StyledRange],
    filter_pos: (usize, usize),
    idx: usize,
}

impl Debug for TextRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(
                f,
                "{}|{}-{}|{}",
                self.start.0, self.start.1, self.end.0, self.end.1
            )
        } else {
            write!(
                f,
                "TextRange  {}|{}-{}|{}",
                self.start.0, self.start.1, self.end.0, self.end.1
            )
        }
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
    pub fn contains_pos(&self, pos: (usize, usize)) -> bool {
        *self == pos
    }

    /// Range fully before the given position.
    #[inline]
    pub fn before_pos(&self, pos: (usize, usize)) -> bool {
        *self < pos
    }

    /// Range fully after the given position.
    #[inline]
    pub fn after_pos(&self, pos: (usize, usize)) -> bool {
        *self > pos
    }

    /// Range contains the other range.
    #[inline(always)]
    pub fn contains(&self, range: TextRange) -> bool {
        let lex_start = LexicalPosition::from(self.start);
        let lex_end = LexicalPosition::from(self.end);

        let o_lex_start = LexicalPosition::from(range.start);
        let o_lex_end = LexicalPosition::from(range.end);

        o_lex_start >= lex_start && o_lex_end <= lex_end
    }

    /// Range before the other range.
    #[inline(always)]
    pub fn before(&self, range: TextRange) -> bool {
        let lex_end = LexicalPosition::from(self.end);
        let o_lex_start = LexicalPosition::from(range.start);

        o_lex_start > lex_end
    }

    /// Range after the other range.
    #[inline(always)]
    pub fn after(&self, range: TextRange) -> bool {
        let lex_start = LexicalPosition::from(self.start);
        let o_lex_end = LexicalPosition::from(range.end);

        o_lex_end < lex_start
    }

    /// Range overlaps with other range.
    #[inline(always)]
    pub fn intersects(&self, range: TextRange) -> bool {
        let lex_start = LexicalPosition::from(self.start);
        let lex_end = LexicalPosition::from(self.end);

        let o_lex_start = LexicalPosition::from(range.start);
        let o_lex_end = LexicalPosition::from(range.end);

        o_lex_start <= lex_end && o_lex_end >= lex_start
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
    pub fn expand_all<'a>(&self, it: impl Iterator<Item = &'a mut StyledRange>) {
        for r in it {
            r.range.start = self.expand(r.range.start);
            r.range.end = self.expand(r.range.end);
        }
    }

    /// Modify all positions in place.
    #[inline]
    pub fn shrink_all<'a>(&self, it: impl Iterator<Item = &'a mut StyledRange>) {
        for r in it {
            r.range.start = self.shrink(r.range.start);
            r.range.end = self.shrink(r.range.end);
        }
    }

    /// Return the modified position, as if this range expanded from its
    /// start to its full expansion.
    #[inline]
    pub fn expand(&self, pos: (usize, usize)) -> (usize, usize) {
        let delta_lines = self.end.1 - self.start.1;

        // swap x and y to enable tuple comparison
        let lex_start = LexicalPosition::from(self.start);
        let lex_pos = LexicalPosition::from(pos);

        if lex_pos < lex_start {
            pos
        } else if lex_pos == lex_start {
            self.end
        } else {
            if lex_pos.y > lex_start.y {
                (pos.0, pos.1 + delta_lines)
            } else if lex_pos.y == lex_start.y {
                if lex_pos.x >= lex_start.x {
                    (pos.0 - self.start.0 + self.end.0, pos.1 + delta_lines)
                } else {
                    pos
                }
            } else {
                pos
            }
        }
    }

    /// Return the modified position, as if this range would shrink to nothing.
    #[inline]
    pub fn shrink(&self, pos: (usize, usize)) -> (usize, usize) {
        let delta_lines = self.end.1 - self.start.1;

        // swap x and y to enable tuple comparison
        let lex_start = LexicalPosition::from(self.start);
        let lex_end = LexicalPosition::from(self.end);
        let lex_pos = LexicalPosition::from(pos);

        if lex_pos < lex_start {
            pos
        } else if lex_pos >= lex_start && lex_pos <= lex_end {
            self.start
        } else {
            // after row
            if lex_pos.y > lex_end.y {
                (pos.0, pos.1 - delta_lines)
            } else if lex_pos.y == lex_end.y {
                if lex_pos.x >= lex_end.x {
                    (pos.0 - self.end.0 + self.start.0, pos.1 - delta_lines)
                } else {
                    pos
                }
            } else {
                pos
            }
        }
    }
}

impl PartialEq<(usize, usize)> for TextRange {
    #[inline]
    fn eq(&self, pos: &(usize, usize)) -> bool {
        let lex_start = LexicalPosition::from(self.start);
        let lex_end = LexicalPosition::from(self.end);
        let lex_pos = LexicalPosition::from(*pos);

        lex_pos >= lex_start && lex_pos < lex_end
    }
}

impl PartialOrd<(usize, usize)> for TextRange {
    #[inline]
    fn partial_cmp(&self, pos: &(usize, usize)) -> Option<Ordering> {
        let lex_start = LexicalPosition::from(self.start);
        let lex_end = LexicalPosition::from(self.end);
        let lex_pos = LexicalPosition::from(*pos);

        if lex_pos >= lex_end {
            Some(Ordering::Less)
        } else if lex_pos < lex_start {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Equal)
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
        let start = LexicalPosition::from(self.start);
        let end = LexicalPosition::from(self.end);
        let ostart = LexicalPosition::from(other.start);
        let oend = LexicalPosition::from(other.end);

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
    fn new(styles: &'a [StyledRange], first: usize, pos: (usize, usize)) -> Self {
        Self {
            styles,
            filter_pos: pos,
            idx: first,
        }
    }
}

impl<'a> Iterator for StyleMapIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.idx;
        if idx < self.styles.len() {
            if self.styles[idx].range.contains_pos(self.filter_pos) {
                self.idx += 1;
                Some(self.styles[idx].style)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl StyledRange {}

impl Debug for StyledRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "StyledRange {{{:#?} -> {}}}", self.range, self.style)
    }
}

impl From<(TextRange, usize)> for StyledRange {
    fn from(value: (TextRange, usize)) -> Self {
        Self {
            range: value.0,
            style: value.1,
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
        let stylemap = StyledRange::from((range, style));
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
        let stylemap = StyledRange::from((range, style));
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
    ) -> impl Iterator<Item = &mut StyledRange> {
        let first = match self
            .styles
            .binary_search_by(|v| v.range.partial_cmp(&pos).expect("ordering"))
        {
            Ok(mut i) => {
                // binary-search found *some* matching style, we need all of them.
                // this finds the first one.
                loop {
                    if i == 0 {
                        break;
                    }
                    if !self.styles[i - 1].range.contains_pos(pos) {
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
        let first = match self
            .styles
            .binary_search_by(|v| v.range.partial_cmp(&pos).expect("order"))
        {
            Ok(mut i) => {
                // binary-search found *some* matching style, we need all of them.
                // this finds the first one.
                loop {
                    if i == 0 {
                        break;
                    }
                    if !self.styles[i - 1].range.contains_pos(pos) {
                        break;
                    }
                    i -= 1;
                }
                i
            }
            Err(_) => self.styles.len(),
        };

        StyleMapIter::new(&self.styles, first, pos)
    }
}

impl Default for TextAreaCore {
    fn default() -> Self {
        Self {
            value: Default::default(),
            styles: Default::default(),
            undo: Some(Box::new(UndoVec::new(40))),
            newline: "\n".to_string(),
            tabs: 8,
            expand_tabs: true,
            move_col: None,
            cursor: (0, 0),
            anchor: (0, 0),
            buf: Default::default(),
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

    /// Expand tabs to spaces. Only for new inputs.
    #[inline]
    pub fn set_expand_tabs(&mut self, expand: bool) {
        self.expand_tabs = expand;
    }

    /// Expand tabs to spaces. Only for new inputs.
    #[inline]
    pub fn expand_tabs(&self) -> bool {
        self.expand_tabs
    }

    /// Undo
    #[inline]
    pub fn set_undo_buffer(&mut self, undo: Box<dyn UndoBuffer>) {
        self.undo = Some(undo);
    }

    /// Undo
    #[inline]
    pub fn undo_buffer(&self) -> Option<&dyn UndoBuffer> {
        match &self.undo {
            None => None,
            Some(v) => Some(v.as_ref()),
        }
    }

    /// Undo last.
    pub fn undo(&mut self) -> bool {
        let Some(undo) = self.undo.as_mut() else {
            return false;
        };
        let op = undo.undo();
        debug!("undo call {:?}", op);
        match op {
            Some(UndoEntry::InsertChar {
                chars,
                cursor,
                anchor,
                redo_cursor: _,
                redo_anchor: _,
                range: _,
                txt: _,
            })
            | Some(UndoEntry::InsertStr {
                chars,
                cursor,
                anchor,
                redo_cursor: _,
                redo_anchor: _,
                range: _,
                txt: _,
            }) => {
                self.value.remove(chars.0..chars.1);

                // todo: ranges
                self.anchor = anchor;
                self.cursor = cursor;

                true
            }
            Some(UndoEntry::RemoveStr {
                chars,
                cursor,
                anchor,
                redo_cursor: _,
                redo_anchor: _,
                range: _,
                txt,
            })
            | Some(UndoEntry::RemoveChar {
                chars,
                cursor,
                anchor,
                redo_cursor: _,
                redo_anchor: _,
                range: _,
                txt,
            }) => {
                self.value.insert(chars.0, &txt);

                // todo: ranges
                self.anchor = anchor;
                self.cursor = cursor;

                true
            }
            None => false,
        }
    }

    /// Redo last.
    pub fn redo(&mut self) -> bool {
        let Some(undo) = self.undo.as_mut() else {
            return false;
        };
        let op = undo.redo();
        debug!("redo call {:?}", op);
        match op {
            Some(UndoEntry::InsertChar {
                chars,
                cursor: _,
                anchor: _,
                redo_cursor,
                redo_anchor,
                range: _,
                txt,
            })
            | Some(UndoEntry::InsertStr {
                chars,
                cursor: _,
                anchor: _,
                redo_cursor,
                redo_anchor,
                range: _,
                txt,
            }) => {
                self.value.insert(chars.0, &txt);

                // todo: ranges
                self.anchor = redo_anchor;
                self.cursor = redo_cursor;

                true
            }

            Some(UndoEntry::RemoveChar {
                chars,
                cursor: _,
                anchor: _,
                redo_cursor,
                redo_anchor,
                range: _,
                txt: _,
            })
            | Some(UndoEntry::RemoveStr {
                chars,
                cursor: _,
                anchor: _,
                redo_cursor,
                redo_anchor,
                range: _,
                txt: _,
            }) => {
                self.value.remove(chars.0..chars.1);

                // todo: ranges
                self.anchor = redo_anchor;
                self.cursor = redo_cursor;

                true
            }
            None => false,
        }
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
    pub fn styles(&self) -> &[StyledRange] {
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
    pub fn line_glyphs(&self, n: usize) -> Option<RopeGlyphIter<'_>> {
        let mut lines = self.value.get_lines_at(n)?;
        if let Some(line) = lines.next() {
            let mut it = RopeGlyphIter::new(line);
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

    /// Insert a character.
    pub fn insert_tab(&mut self, mut pos: (usize, usize)) {
        if self.expand_tabs {
            let n = self.tabs as usize - pos.0 % self.tabs as usize;
            for _ in 0..n {
                self.insert_char(pos, ' ');
                pos.0 += 1;
            }
        } else {
            self.insert_char(pos, '\t');
        }
    }

    /// Insert a line break.
    pub fn insert_newline(&mut self, mut pos: (usize, usize)) {
        for c in self.newline.clone().chars() {
            self.insert_char(pos, c);
            pos.0 += 1;
        }
    }

    /// Insert a character.
    pub fn insert_char(&mut self, pos: (usize, usize), c: char) {
        let Some(char_pos) = self.char_at(pos) else {
            panic!("invalid pos {:?} value {:?}", pos, self.value);
        };

        let old_cursor = self.cursor;
        let old_anchor = self.anchor;

        let mut line_count = 0;
        if c == '\n' {
            line_count = 1;
        }

        let insert = if line_count > 0 {
            self.value.insert_char(char_pos, c);

            TextRange::new(pos, (0, pos.1 + line_count))
        } else {
            // no way to know if the new char combines with a surrounding char.
            // the difference of the graphem len seems safe though.
            let old_len = self.line_width(pos.1).expect("valid_pos");
            self.value.insert_char(char_pos, c);
            let new_len = self.line_width(pos.1).expect("valid_pos");

            TextRange::new(pos, (pos.0 + new_len - old_len, pos.1))
        };

        insert.expand_all(self.styles.styles_after_mut(pos));
        self.anchor = insert.expand(self.anchor);
        self.cursor = insert.expand(self.cursor);

        if let Some(undo) = self.undo.as_mut() {
            undo.insert_char(
                char_pos,
                old_cursor,
                old_anchor,
                self.cursor,
                self.anchor,
                insert,
                c,
            );
        }
    }

    /// Insert some text.
    pub fn insert_str(&mut self, pos: (usize, usize), t: &str) {
        let Some(char_pos) = self.char_at(pos) else {
            panic!("invalid pos {:?} value {:?}", pos, self.value);
        };

        let old_cursor = self.cursor;
        let old_anchor = self.anchor;

        let mut char_count = 0;
        let mut line_count = 0;
        let mut linebreak_idx = 0;
        for (p, c) in t.char_indices() {
            if c == '\n' {
                line_count += 1;
                linebreak_idx = p + 1;
            }
            char_count += 1;
        }

        let insert = if line_count > 0 {
            let mut buf = mem::take(&mut self.buf);

            // Find the length of line after the insert position.
            let split = self.char_at(pos).expect("valid_pos");
            let line = self.line_chars(pos.1).expect("valid_pos");
            buf.clear();
            for c in line.skip(split) {
                buf.push(c);
            }
            let old_len = str_line_len(&buf);

            // compose the new line and find its length.
            buf.clear();
            buf.push_str(&t[linebreak_idx..]);
            let line = self.line_chars(pos.1).expect("valid_pos");
            for c in line.skip(split) {
                buf.push(c);
            }
            let new_len = str_line_len(&buf);

            buf.clear();
            self.buf = buf;

            self.value.insert(char_pos, t);

            TextRange::new(pos, (new_len - old_len, pos.1 + line_count))
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

        if let Some(undo) = self.undo.as_mut() {
            undo.insert_str(
                (char_pos, char_pos + char_count),
                old_cursor,
                old_anchor,
                self.cursor,
                self.anchor,
                insert,
                t.to_string(),
            );
        }
    }

    /// Remove the previous character
    pub fn remove_prev_char(&mut self, pos: (usize, usize)) -> bool {
        let (sx, sy) = if pos.1 == 0 && pos.0 == 0 {
            (0, 0)
        } else if pos.1 != 0 && pos.0 == 0 {
            let prev_line_width = self.line_width(pos.1 - 1).expect("line_width");
            (prev_line_width, pos.1 - 1)
        } else {
            (pos.0 - 1, pos.1)
        };

        let range = TextRange::new((sx, sy), (pos.0, pos.1));

        self._remove_range(range, true)
    }

    /// Remove the next character
    pub fn remove_next_char(&mut self, pos: (usize, usize)) -> bool {
        let c_line_width = self.line_width(pos.1).expect("width");
        let c_last_line = self.len_lines() - 1;

        let (ex, ey) = if pos.1 == c_last_line && pos.0 == c_line_width {
            (pos.0, pos.1)
        } else if pos.1 != c_last_line && pos.0 == c_line_width {
            (0, pos.1 + 1)
        } else {
            (pos.0 + 1, pos.1)
        };
        let range = TextRange::new((pos.0, pos.1), (ex, ey));

        self._remove_range(range, true)
    }

    /// Remove the given range.
    pub fn remove_range(&mut self, range: TextRange) -> bool {
        self._remove_range(range, false)
    }

    /// Remove the given range.
    fn _remove_range(&mut self, range: TextRange, char_range: bool) -> bool {
        let Some(start_pos) = self.char_at(range.start) else {
            panic!("invalid range {:?} value {:?}", range, self.value);
        };
        let Some(end_pos) = self.char_at(range.end) else {
            panic!("invalid range {:?} value {:?}", range, self.value);
        };

        if range.is_empty() {
            return false;
        }

        let old_cursor = self.cursor;
        let old_anchor = self.anchor;
        let old_text = self.text_slice(range).expect("some text").to_string();

        self.value.remove(start_pos..end_pos);

        // remove deleted styles.
        // this is not a simple range, so filter+collect seems ok.
        let styles = mem::take(&mut self.styles.styles);
        self.styles.styles = styles
            .into_iter()
            .filter(|v| !range.contains(v.range))
            .collect();

        range.shrink_all(self.styles.styles_after_mut(range.start));
        self.anchor = range.shrink(self.anchor);
        self.cursor = range.shrink(self.anchor);

        if let Some(undo) = &mut self.undo {
            if char_range {
                undo.remove_char(
                    (start_pos, end_pos),
                    old_cursor,
                    old_anchor,
                    self.cursor,
                    self.anchor,
                    range,
                    old_text,
                );
            } else {
                undo.remove_str(
                    (start_pos, end_pos),
                    old_cursor,
                    old_anchor,
                    self.cursor,
                    self.anchor,
                    range,
                    old_text,
                );
            }
        }

        true
    }
}
