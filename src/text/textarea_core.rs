use crate::text::graphemes::{
    rope_line_len, str_line_len, RopeGlyphIter, RopeGraphemes, RopeGraphemesIdx,
};
use crate::text::undo::{StyleChange, TextPositionChange, UndoBuffer, UndoEntry, UndoVec};
use iset::IntervalMap;
use log::debug;
use pure_rust_locales::Locale::en_DK;
use ropey::{Rope, RopeSlice};
use std::cmp::{max, min};
use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::ops::{Range, RangeBounds};
use std::{mem, slice};

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
    cursor: TextPosition,
    /// Anchor for the selection.
    anchor: TextPosition,

    /// temp string
    buf: String,
}

/// Exclusive range for text ranges.
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct TextRange {
    /// column, row
    pub start: TextPosition,
    /// column, row
    pub end: TextPosition,
}

/// Text position.
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct TextPosition {
    pub y: usize,
    pub x: usize,
}

/// Styles.
#[derive(Debug, Default, Clone)]
struct StyleMap {
    // todo: there is probably some order for applying the styles,
    //       on the other hand merging of styles is dubious at best.
    //       anyway, what should that order be?
    // -> order by style-nr. a bit arbitrary, but maybe ok.
    buf: Vec<(Range<TextPosition>, StyleMapEntry)>,
    styles: IntervalMap<TextPosition, StyleMapEntry>,
}

/// Styles as stored in the style map.
#[derive(Debug, Default, Clone)]
pub enum StyleMapEntry {
    #[default]
    NoStyle,
    Style(usize),
    Styles(Vec<usize>),
}

impl TextPosition {
    /// New position.
    pub fn new(x: usize, y: usize) -> TextPosition {
        Self::from((x, y))
    }
}

impl Debug for TextPosition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{}", self.x, self.y)
    }
}

impl From<(usize, usize)> for TextPosition {
    fn from(value: (usize, usize)) -> Self {
        Self {
            y: value.1,
            x: value.0,
        }
    }
}

impl From<TextPosition> for (usize, usize) {
    fn from(value: TextPosition) -> Self {
        (value.x, value.y)
    }
}

impl Debug for TextRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(
                f,
                "{}|{}-{}|{}",
                self.start.x, self.start.y, self.end.x, self.end.y
            )
        } else {
            write!(
                f,
                "TextRange  {}|{}-{}|{}",
                self.start.x, self.start.y, self.end.x, self.end.y
            )
        }
    }
}

impl From<Range<TextPosition>> for TextRange {
    fn from(value: Range<TextPosition>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl From<TextRange> for Range<TextPosition> {
    fn from(value: TextRange) -> Self {
        value.start..value.end
    }
}

impl TextRange {
    /// New text range.
    ///
    /// Panic
    /// Panics if start > end.
    pub fn new(start: impl Into<TextPosition>, end: impl Into<TextPosition>) -> Self {
        let start = start.into();
        let end = end.into();

        // reverse the args, then it works.
        if start > end {
            panic!("start {:?} > end {:?}", start, end);
        }
        TextRange { start, end }
    }

    /// Empty range
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Range contains the given position.
    #[inline]
    pub fn contains_pos(&self, pos: impl Into<TextPosition>) -> bool {
        let pos = pos.into();
        pos >= self.start && pos < self.end
    }

    /// Range fully before the given position.
    #[inline]
    pub fn before_pos(&self, pos: impl Into<TextPosition>) -> bool {
        let pos = pos.into();
        pos >= self.end
    }

    /// Range fully after the given position.
    #[inline]
    pub fn after_pos(&self, pos: impl Into<TextPosition>) -> bool {
        let pos = pos.into();
        pos < self.start
    }

    /// Range contains the other range.
    #[inline(always)]
    pub fn contains(&self, other: TextRange) -> bool {
        other.start >= self.start && other.end <= self.end
    }

    /// Range before the other range.
    #[inline(always)]
    pub fn before(&self, other: TextRange) -> bool {
        other.start > self.end
    }

    /// Range after the other range.
    #[inline(always)]
    pub fn after(&self, other: TextRange) -> bool {
        other.end < self.start
    }

    /// Range overlaps with other range.
    #[inline(always)]
    pub fn intersects(&self, other: TextRange) -> bool {
        other.start <= self.end && other.end >= self.start
    }

    /// Return the modified value range, that accounts for a
    /// text insertion of range.
    #[inline]
    pub fn expand(&self, range: TextRange) -> TextRange {
        TextRange::new(self.expand_pos(range.start), self.expand_pos(range.end))
    }

    /// Return the modified position, that accounts for a
    /// text insertion of range.
    #[inline]
    pub fn expand_pos(&self, pos: TextPosition) -> TextPosition {
        let delta_lines = self.end.y - self.start.y;

        // swap x and y to enable tuple comparison
        if pos < self.start {
            pos
        } else if pos == self.start {
            self.end
        } else {
            if pos.y > self.start.y {
                TextPosition::new(pos.x, pos.y + delta_lines)
            } else if pos.y == self.start.y {
                if pos.x >= self.start.x {
                    TextPosition::new(pos.x - self.start.x + self.end.x, pos.y + delta_lines)
                } else {
                    pos
                }
            } else {
                pos
            }
        }
    }

    /// Return the modified value range, that accounts for a
    /// text deletion of range.
    #[inline]
    pub fn shrink(&self, range: TextRange) -> TextRange {
        TextRange::new(self.shrink_pos(range.start), self.shrink_pos(range.end))
    }

    /// Return the modified position, that accounts for a
    /// text deletion of the range.
    #[inline]
    pub fn shrink_pos(&self, pos: TextPosition) -> TextPosition {
        let delta_lines = self.end.y - self.start.y;

        // swap x and y to enable tuple comparison
        if pos < self.start {
            pos
        } else if pos >= self.start && pos <= self.end {
            self.start
        } else {
            // after row
            if pos.y > self.end.y {
                TextPosition::new(pos.x, pos.y - delta_lines)
            } else if pos.y == self.end.y {
                if pos.x >= self.end.x {
                    TextPosition::new(pos.x - self.end.x + self.start.x, pos.y - delta_lines)
                } else {
                    pos
                }
            } else {
                pos
            }
        }
    }
}

impl StyleMapEntry {
    pub fn iter(&self) -> StyleMapEntryIter<'_> {
        match self {
            StyleMapEntry::NoStyle => StyleMapEntryIter {
                single: None,
                iter: None,
            },
            StyleMapEntry::Style(s) => StyleMapEntryIter {
                single: Some(*s),
                iter: None,
            },
            StyleMapEntry::Styles(s) => StyleMapEntryIter {
                single: None,
                iter: Some(s.iter()),
            },
        }
    }
}

pub struct StyleMapEntryIter<'a> {
    single: Option<usize>,
    iter: Option<slice::Iter<'a, usize>>,
}

impl<'a> Iterator for StyleMapEntryIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(single) = self.single {
            self.single = None;
            Some(single)
        } else if let Some(iter) = &mut self.iter {
            iter.next().copied()
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
        self.styles
            .entry(range.into())
            .and_modify(|v| {
                let new_v = match v {
                    StyleMapEntry::NoStyle => {
                        //
                        StyleMapEntry::Style(style)
                    }
                    StyleMapEntry::Style(w) => {
                        if *w != style {
                            let mut s_vec = vec![*w, style];
                            s_vec.sort();
                            StyleMapEntry::Styles(s_vec)
                        } else {
                            StyleMapEntry::Style(*w)
                        }
                    }
                    StyleMapEntry::Styles(w) => {
                        let mut w = mem::take(w);
                        match w.binary_search(&style) {
                            Ok(_) => {}
                            Err(i) => w.insert(i, style),
                        }
                        StyleMapEntry::Styles(w)
                    }
                };
                *v = new_v;
            })
            .or_insert_with(|| StyleMapEntry::Style(style));
    }

    /// Remove a text-style for a range.
    ///
    /// This must match exactly in range and style to be removed.
    pub(crate) fn remove_style(&mut self, range: TextRange, style: usize) {
        let Some(v) = self.styles.get_mut(range.into()) else {
            return;
        };

        let new_v = match v {
            StyleMapEntry::NoStyle => {
                //
                StyleMapEntry::NoStyle
            }
            StyleMapEntry::Style(w) => {
                if *w == style {
                    StyleMapEntry::NoStyle
                } else {
                    StyleMapEntry::Style(*w)
                }
            }
            StyleMapEntry::Styles(w) => {
                let mut w = mem::take(w);
                w.retain(|s| *s != style);
                if w.len() > 1 {
                    StyleMapEntry::Styles(w)
                } else {
                    StyleMapEntry::Style(w[0])
                }
            }
        };
        let is_empty = matches!(new_v, StyleMapEntry::NoStyle);
        *v = new_v;

        if is_empty {
            self.styles.remove(range.into());
        }
    }

    /// Find all styles that touch the given position.
    pub(crate) fn styles_at(&self, pos: TextPosition, buf: &mut Vec<usize>) {
        for v in self
            .styles
            .overlap(pos) //
            .map(|v| v.1)
        {
            match v {
                StyleMapEntry::NoStyle => {}
                StyleMapEntry::Style(w) => {
                    buf.push(*w);
                }
                StyleMapEntry::Styles(w) => {
                    buf.extend_from_slice(w);
                }
            }
        }
    }

    /// Map and rebuild the IntervalMap.
    #[inline]
    pub(crate) fn remap(
        &mut self,
        mut remap_fn: impl FnMut(TextRange, &StyleMapEntry) -> Option<TextRange>,
    ) {
        self.buf.clear();

        let styles = mem::take(&mut self.styles);
        for (r, s) in styles.into_iter(..) {
            if let Some(r) = remap_fn(r.into(), &s) {
                self.buf.push((r.into(), s));
            }
        }
        self.styles = IntervalMap::from_sorted(self.buf.drain(..));
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
            cursor: Default::default(),
            anchor: Default::default(),
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
                range,
                txt,
                styles,
            })
            | Some(UndoEntry::RemoveChar {
                chars,
                cursor,
                anchor,
                range,
                txt,
                styles,
            }) => {
                self.value.insert(chars.start, &txt);

                for s in &styles {
                    self.styles.remove_style(s.after, s.style);
                }
                for s in &styles {
                    self.styles.add_style(s.after, s.style);
                }
                // todo:
                // for s in self.styles.styles_after_mut(range.end) {
                //     s.range = range.expand(s.range);
                // }

                self.anchor = anchor.before;
                self.cursor = cursor.before;

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
                cursor,
                anchor,
                range: _,
                txt: _,
                styles,
            })
            | Some(UndoEntry::RemoveStr {
                chars,
                cursor,
                anchor,
                range: _,
                txt: _,
                styles,
            }) => {
                self.value.remove(chars);

                // todo: ranges
                self.anchor = anchor.after;
                self.cursor = cursor.after;

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

    /// Finds all styles for the given position.
    #[inline]
    pub fn styles_at(&self, pos: TextPosition, buf: &mut Vec<usize>) {
        self.styles.styles_at(pos, buf)
    }

    /// Set the cursor position.
    /// The value is capped to the number of text lines and the line-width for the given line.
    /// Returns true, if the cursor actually changed.
    pub fn set_cursor(&mut self, mut cursor: TextPosition, extend_selection: bool) -> bool {
        let old_cursor = self.cursor;
        let old_anchor = self.anchor;

        let mut c = cursor;
        c.y = min(c.y, self.len_lines() - 1);
        c.x = min(c.x, self.line_width(c.y).expect("valid_line"));

        cursor = c;

        self.cursor = cursor;

        if !extend_selection {
            self.anchor = cursor;
        }

        old_cursor != self.cursor || old_anchor != self.anchor
    }

    /// Cursor position.
    #[inline]
    pub fn cursor(&self) -> TextPosition {
        self.cursor
    }

    /// Selection anchor.
    #[inline]
    pub fn anchor(&self) -> TextPosition {
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
        self.cursor = Default::default();
        self.anchor = Default::default();
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
    pub fn lines_at(&self, n: usize) -> impl Iterator<Item = RopeSlice<'_>> {
        self.value.lines_at(n)
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

        self.set_cursor(TextPosition::new(0, 0), false);
        let last = self.len_lines() - 1;
        let last_width = self.line_width(last).expect("valid_last_line");
        self.set_cursor(TextPosition::new(last_width, last), true);

        old_selection != self.selection()
    }

    /// Returns the selection as TextRange.
    pub fn selection(&self) -> TextRange {
        #[allow(clippy::comparison_chain)]
        if self.cursor.y < self.anchor.y {
            TextRange {
                start: self.cursor,
                end: self.anchor,
            }
        } else if self.cursor.y > self.anchor.y {
            TextRange {
                start: self.anchor,
                end: self.cursor,
            }
        } else {
            if self.cursor.x < self.anchor.x {
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
    pub fn char_pos(&self, char_pos: usize) -> Option<TextPosition> {
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
    ) -> Option<impl Iterator<Item = (Range<usize>, RopeSlice<'_>)>> {
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
    pub fn byte_pos(&self, byte: usize) -> Option<TextPosition> {
        let Ok(y) = self.value.try_byte_to_line(byte) else {
            return None;
        };
        let mut x = 0;
        let byte_y = self.value.try_line_to_byte(y).expect("valid_y");

        let mut it_line = self.line_grapheme_idx(y).expect("valid_y");
        loop {
            let Some((Range { start: sb, .. }, _cc)) = it_line.next() else {
                break;
            };
            if byte_y + sb >= byte {
                break;
            }
            x += 1;
        }

        Some(TextPosition::new(x, y))
    }

    /// Grapheme position to byte position.
    /// This is the (start,end) position of the single grapheme after pos.
    pub fn byte_at(&self, pos: TextPosition) -> Option<Range<usize>> {
        let Ok(line_byte) = self.value.try_line_to_byte(pos.y) else {
            return None;
        };

        let len_bytes = self.value.len_bytes();
        let mut it_line = self.line_grapheme_idx(pos.y).expect("valid_line");
        let mut x = -1isize;
        let mut last_eb = 0;
        loop {
            let (range, last) = if let Some((range, _)) = it_line.next() {
                x += 1;
                last_eb = range.end;
                (range, false)
            } else {
                (last_eb..last_eb, true)
            };

            if pos.x == x as usize {
                return Some(line_byte + range.start..line_byte + range.end);
            }
            // one past the end is ok.
            if pos.x == (x + 1) as usize && line_byte + range.end == len_bytes {
                return Some(line_byte + range.end..line_byte + range.end);
            }
            if last {
                return None;
            }
        }
    }

    /// Returns the first char position for the grapheme position.
    pub fn char_at(&self, pos: TextPosition) -> Option<usize> {
        let byte_range = self.byte_at(pos)?;
        Some(
            self.value
                .try_byte_to_char(byte_range.start)
                .expect("valid_byte_pos"),
        )
    }

    /// Insert a character.
    pub fn insert_tab(&mut self, mut pos: TextPosition) {
        if self.expand_tabs {
            let n = self.tabs as usize - pos.x % self.tabs as usize;
            for _ in 0..n {
                self.insert_char(pos, ' ');
                pos.x += 1;
            }
        } else {
            self.insert_char(pos, '\t');
        }
    }

    /// Insert a line break.
    pub fn insert_newline(&mut self, mut pos: TextPosition) {
        for c in self.newline.clone().chars() {
            self.insert_char(pos, c);
            pos.x += 1;
        }
    }

    /// Insert a character.
    pub fn insert_char(&mut self, pos: TextPosition, c: char) {
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

            TextRange::new(pos, (0, pos.y + line_count))
        } else {
            // no way to know if the new char combines with a surrounding char.
            // the difference of the graphem len seems safe though.
            let old_len = self.line_width(pos.y).expect("valid_pos");
            self.value.insert_char(char_pos, c);
            let new_len = self.line_width(pos.y).expect("valid_pos");

            TextRange::new(pos, (pos.x + new_len - old_len, pos.y))
        };

        self.styles
            .remap(|r: TextRange, s: &StyleMapEntry| Some(insert.expand(r)));
        self.anchor = insert.expand_pos(self.anchor);
        self.cursor = insert.expand_pos(self.cursor);

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
    pub fn insert_str(&mut self, pos: TextPosition, t: &str) {
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
            let line = self.line_chars(pos.y).expect("valid_pos");
            buf.clear();
            for c in line.skip(split) {
                buf.push(c);
            }
            let old_len = str_line_len(&buf);

            // compose the new line and find its length.
            buf.clear();
            buf.push_str(&t[linebreak_idx..]);
            let line = self.line_chars(pos.y).expect("valid_pos");
            for c in line.skip(split) {
                buf.push(c);
            }
            let new_len = str_line_len(&buf);

            buf.clear();
            self.buf = buf;

            self.value.insert(char_pos, t);

            TextRange::new(pos, (new_len - old_len, pos.y + line_count))
        } else {
            // no way to know if the insert text combines with a surrounding char.
            // the difference of the graphem len seems safe though.
            let old_len = self.line_width(pos.y).expect("valid_pos");
            self.value.insert(char_pos, t);
            let new_len = self.line_width(pos.y).expect("valid_pos");

            TextRange::new(pos, (pos.x + new_len - old_len, pos.y))
        };

        self.styles.remap(|r, s| Some(insert.expand(r)));
        self.anchor = insert.expand_pos(self.anchor);
        self.cursor = insert.expand_pos(self.cursor);

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
    pub fn remove_prev_char(&mut self, pos: TextPosition) -> bool {
        let (sx, sy) = if pos.y == 0 && pos.x == 0 {
            (0, 0)
        } else if pos.y != 0 && pos.x == 0 {
            let prev_line_width = self.line_width(pos.y - 1).expect("line_width");
            (prev_line_width, pos.y - 1)
        } else {
            (pos.x - 1, pos.y)
        };

        let range = TextRange::new((sx, sy), (pos.x, pos.y));

        self._remove_range(range, true)
    }

    /// Remove the next character
    pub fn remove_next_char(&mut self, pos: TextPosition) -> bool {
        let c_line_width = self.line_width(pos.y).expect("width");
        let c_last_line = self.len_lines() - 1;

        let (ex, ey) = if pos.y == c_last_line && pos.x == c_line_width {
            (pos.x, pos.y)
        } else if pos.y != c_last_line && pos.x == c_line_width {
            (0, pos.y + 1)
        } else {
            (pos.x + 1, pos.y)
        };
        let range = TextRange::new((pos.x, pos.y), (ex, ey));

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
        let mut changed = Vec::new();
        self.styles.remap(|r: TextRange, s: &StyleMapEntry| {
            if range.after(r) {
                debug!("before {:?} -> {:?}", r, r);
                Some(r)
            } else if range.contains(r) {
                debug!("contains {:?} -> ", r);
                for s in s.iter() {
                    changed.push(StyleChange {
                        before: r,
                        after: TextRange::new(r.start, r.start),
                        style: s,
                    });
                }
                None
            } else if range.intersects(r) {
                let new_range = range.shrink(r);
                debug!("intersect {:?} -> {:?}", r, new_range);
                for s in s.iter() {
                    changed.push(StyleChange {
                        before: r,
                        after: new_range,
                        style: s,
                    });
                }
                Some(new_range)
            } else if range.before(r) {
                let new_range = range.shrink(r);
                debug!("after {:?} -> {:?}", r, new_range);
                // can be calculated
                Some(new_range)
            } else {
                unreachable!("fail range check")
            }
        });

        self.anchor = range.shrink_pos(self.anchor);
        self.cursor = range.shrink_pos(self.anchor);

        if let Some(undo) = &mut self.undo {
            if char_range {
                undo.remove_char(
                    start_pos..end_pos,
                    TextPositionChange {
                        before: old_cursor,
                        after: self.cursor,
                    },
                    TextPositionChange {
                        before: old_anchor,
                        after: self.anchor,
                    },
                    range,
                    old_text,
                    changed,
                );
            } else {
                undo.remove_str(
                    start_pos..end_pos,
                    TextPositionChange {
                        before: old_cursor,
                        after: self.cursor,
                    },
                    TextPositionChange {
                        before: old_anchor,
                        after: self.anchor,
                    },
                    range,
                    old_text,
                    changed,
                );
            }
        }

        true
    }
}
