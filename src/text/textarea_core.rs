use crate::event::TextOutcome;
use crate::text::graphemes::{
    rope_line_len, str_line_len, RopeGlyphIter, RopeGraphemes, RopeGraphemesIdx,
};
use crate::text::range_map::RangeMap;
use crate::text::undo::{StyleChange, TextPositionChange, UndoBuffer, UndoEntry, UndoVec};
use ropey::{Rope, RopeSlice};
use std::borrow::Cow;
use std::cmp::min;
use std::fmt::{Debug, Formatter};
use std::mem;
use std::ops::{Range, RangeBounds};

/// Core for text editing.
#[derive(Debug)]
pub struct TextAreaCore {
    /// Rope for text storage.
    value: Rope,
    /// Styles.
    styles: RangeMap,
    /// Cursor
    cursor: TextPosition,
    /// Anchor for the selection.
    anchor: TextPosition,

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

impl Default for TextAreaCore {
    fn default() -> Self {
        Self {
            value: Default::default(),
            styles: Default::default(),
            cursor: Default::default(),
            anchor: Default::default(),
            undo: Some(Box::new(UndoVec::new(40))),
            newline: "\n".to_string(),
            tabs: 8,
            expand_tabs: true,
            move_col: None,
            buf: Default::default(),
        }
    }
}

impl Clone for TextAreaCore {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            styles: self.styles.clone(),
            cursor: self.cursor,
            anchor: self.anchor,
            undo: self.undo.as_ref().map(|v| v.cloned()),
            newline: self.newline.clone(),
            tabs: self.tabs,
            expand_tabs: self.expand_tabs,
            move_col: self.move_col,
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

    /// Undo
    #[inline]
    pub fn undo_buffer_mut(&mut self) -> Option<&mut dyn UndoBuffer> {
        match &mut self.undo {
            None => None,
            Some(v) => Some(v.as_mut()),
        }
    }

    /// Undo last.
    pub fn undo(&mut self) -> TextOutcome {
        let Some(undo) = self.undo.as_mut() else {
            return TextOutcome::Continue;
        };

        undo.append(UndoEntry::Undo);

        self._undo()
    }

    /// Undo last.
    fn _undo(&mut self) -> TextOutcome {
        let Some(undo) = self.undo.as_mut() else {
            return TextOutcome::Continue;
        };
        let op = undo.undo();
        match op {
            Some(UndoEntry::InsertChar {
                chars,
                cursor,
                anchor,
                range,
                ..
            })
            | Some(UndoEntry::InsertStr {
                chars,
                cursor,
                anchor,
                range,
                ..
            }) => {
                self.value.remove(chars);

                self.styles.remap(|r, _| Some(range.shrink(r)));
                self.anchor = anchor.before;
                self.cursor = cursor.before;

                TextOutcome::TextChanged
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
                    self.styles.remove(s.after, s.style);
                }
                for s in &styles {
                    self.styles.add(s.before, s.style);
                }
                self.styles.remap(|r, _| {
                    if range.intersects(r) {
                        Some(r)
                    } else {
                        Some(range.expand(r))
                    }
                });
                self.anchor = anchor.before;
                self.cursor = cursor.before;

                TextOutcome::TextChanged
            }
            Some(UndoEntry::SetText {
                txt_before,
                cursor,
                anchor,
                styles_before,
                ..
            }) => {
                self.value = txt_before;
                self.cursor = cursor.before;
                self.anchor = anchor.before;
                self.styles.set(styles_before.iter().copied());
                TextOutcome::TextChanged
            }
            Some(UndoEntry::SetStyles { styles_before, .. }) => {
                self.styles.set(styles_before.iter().copied());
                TextOutcome::Changed
            }
            Some(UndoEntry::AddStyle { range, style }) => {
                self.styles.remove(range, style);
                TextOutcome::Changed
            }
            Some(UndoEntry::RemoveStyle { range, style }) => {
                self.styles.add(range, style);
                TextOutcome::Changed
            }
            Some(UndoEntry::Undo) => TextOutcome::Unchanged,
            Some(UndoEntry::Redo) => TextOutcome::Unchanged,
            None => TextOutcome::Continue,
        }
    }

    /// Redo last.
    pub fn redo(&mut self) -> TextOutcome {
        let Some(undo) = self.undo.as_mut() else {
            return TextOutcome::Continue;
        };

        undo.append(UndoEntry::Redo);

        self._redo()
    }

    fn _redo(&mut self) -> TextOutcome {
        let Some(undo) = self.undo.as_mut() else {
            return TextOutcome::Continue;
        };
        let op = undo.redo();
        match op {
            Some(UndoEntry::InsertChar {
                chars,
                cursor,
                anchor,
                range,
                txt,
            })
            | Some(UndoEntry::InsertStr {
                chars,
                cursor,
                anchor,
                range,
                txt,
            }) => {
                self.value.insert(chars.start, &txt);
                self.styles.remap(|r, _| Some(range.expand(r)));
                self.anchor = anchor.after;
                self.cursor = cursor.after;

                TextOutcome::TextChanged
            }
            Some(UndoEntry::RemoveChar {
                chars,
                cursor,
                anchor,
                range,
                styles,
                ..
            })
            | Some(UndoEntry::RemoveStr {
                chars,
                cursor,
                anchor,
                range,
                styles,
                ..
            }) => {
                self.value.remove(chars);

                self.styles.remap(|r, _| {
                    if range.intersects(r) {
                        Some(r)
                    } else {
                        Some(range.shrink(r))
                    }
                });
                for s in &styles {
                    self.styles.remove(s.before, s.style);
                }
                for s in &styles {
                    self.styles.add(s.after, s.style);
                }

                self.anchor = anchor.after;
                self.cursor = cursor.after;

                TextOutcome::TextChanged
            }
            Some(UndoEntry::SetText {
                txt_after,
                cursor,
                anchor,
                ..
            }) => {
                self.value = txt_after;
                self.cursor = cursor.after;
                self.anchor = anchor.after;
                self.styles.clear();
                TextOutcome::TextChanged
            }
            Some(UndoEntry::SetStyles { styles_after, .. }) => {
                self.styles.set(styles_after.iter().copied());
                TextOutcome::Changed
            }
            Some(UndoEntry::AddStyle { range, style }) => {
                self.styles.add(range, style);
                TextOutcome::Changed
            }
            Some(UndoEntry::RemoveStyle { range, style }) => {
                self.styles.remove(range, style);
                TextOutcome::Changed
            }
            Some(UndoEntry::Undo) => TextOutcome::Unchanged,
            Some(UndoEntry::Redo) => TextOutcome::Unchanged,
            None => TextOutcome::Continue,
        }
    }

    /// Get last replay recording.
    pub fn recent_replay(&mut self) -> Vec<UndoEntry> {
        if let Some(undo) = &mut self.undo {
            undo.recent_replay()
        } else {
            Vec::default()
        }
    }

    /// Replay a recording of changes.
    pub fn replay(&mut self, replay: &[UndoEntry]) {
        for replay_entry in replay {
            match replay_entry {
                UndoEntry::SetText { txt_after, .. } => {
                    self.value = txt_after.clone();
                    self.styles.clear();
                    if let Some(undo) = self.undo.as_mut() {
                        undo.clear();
                    };
                }
                UndoEntry::InsertChar {
                    chars, range, txt, ..
                }
                | UndoEntry::InsertStr {
                    chars, range, txt, ..
                } => {
                    self.value.insert(chars.start, txt);
                    self.styles.remap(|r, _| Some(range.expand(r)));
                }
                UndoEntry::RemoveChar {
                    chars,
                    range,
                    styles,
                    ..
                }
                | UndoEntry::RemoveStr {
                    chars,
                    range,
                    styles,
                    ..
                } => {
                    self.value.remove(chars.clone());
                    self.styles.remap(|r, _| {
                        if range.intersects(r) {
                            Some(r)
                        } else {
                            Some(range.shrink(r))
                        }
                    });
                    for s in styles {
                        self.styles.remove(s.before, s.style);
                    }
                    for s in styles {
                        self.styles.add(s.after, s.style);
                    }
                }
                UndoEntry::SetStyles { styles_after, .. } => {
                    self.styles.set(styles_after.iter().copied());
                }
                UndoEntry::AddStyle { range, style } => {
                    self.styles.add(*range, *style);
                }
                UndoEntry::RemoveStyle { range, style } => {
                    self.styles.remove(*range, *style);
                }
                UndoEntry::Undo => {
                    self._undo();
                }
                UndoEntry::Redo => {
                    self._redo();
                }
            }

            if let Some(undo) = self.undo.as_mut() {
                undo.append_no_replay(replay_entry.clone());
            };
        }
    }

    /// Set all styles.
    #[inline]
    pub fn set_styles(&mut self, styles: Vec<(TextRange, usize)>) {
        if let Some(undo) = &mut self.undo {
            if undo.undo_styles_enabled() || undo.replay() {
                undo.append(UndoEntry::SetStyles {
                    styles_before: self.styles.values().collect::<Vec<_>>(),
                    styles_after: styles.clone(),
                });
            }
        }
        self.styles.set(styles.iter().copied());
    }

    /// Add a style for the given range.
    ///
    /// What is given here is the index into the Vec with the actual Styles.
    /// Those are set at the widget.
    #[inline]
    pub fn add_style(&mut self, range: TextRange, style: usize) {
        self.styles.add(range, style);

        if let Some(undo) = &mut self.undo {
            if undo.undo_styles_enabled() || undo.replay() {
                undo.append(UndoEntry::AddStyle { range, style });
            }
        }
    }

    /// Remove a style for the given range.
    ///
    /// Range and style must match to be removed.
    #[inline]
    pub fn remove_style(&mut self, range: TextRange, style: usize) {
        self.styles.remove(range, style);

        if let Some(undo) = &mut self.undo {
            if undo.undo_styles_enabled() || undo.replay() {
                undo.append(UndoEntry::RemoveStyle { range, style });
            }
        }
    }

    /// Finds all styles for the given position.
    #[inline]
    pub fn styles_at(&self, pos: TextPosition, buf: &mut Vec<usize>) {
        self.styles.values_at(pos, buf)
    }

    /// Check if the given style applies at the position and
    /// return the complete range for the style.
    #[inline]
    pub fn style_match(&self, pos: TextPosition, style: usize) -> Option<TextRange> {
        self.styles.value_match(pos, style)
    }

    /// List of all styles.
    #[inline]
    pub fn styles(&self) -> impl Iterator<Item = (TextRange, usize)> + '_ {
        self.styles.values()
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
    pub fn set_rope(&mut self, value: Rope) {
        if let Some(undo) = &mut self.undo {
            undo.clear();

            if undo.replay() {
                undo.append(UndoEntry::SetText {
                    txt_before: self.value.clone(),
                    txt_after: value.clone(),
                    cursor: TextPositionChange {
                        before: self.cursor,
                        after: Default::default(),
                    },
                    anchor: TextPositionChange {
                        before: self.anchor,
                        after: Default::default(),
                    },
                    styles_before: self
                        .styles
                        .values()
                        .map(|(r, s)| (r.into(), s))
                        .collect::<Vec<_>>(),
                });
            }
        }

        self.value = value;
        self.cursor = Default::default();
        self.anchor = Default::default();
        self.move_col = None;
        self.styles.clear();
    }

    /// Access the underlying Rope with the text value.
    #[inline]
    pub fn rope(&self) -> &Rope {
        &self.value
    }

    /// A range of the text as RopeSlice.
    pub fn rope_slice(&self, range: TextRange) -> Option<RopeSlice<'_>> {
        let s = self.char_at(range.start)?;
        let e = self.char_at(range.end)?;
        Some(self.value.slice(s..e))
    }

    /// A range of the text as Cow<str>
    pub fn str_slice(&self, range: TextRange) -> Option<Cow<'_, str>> {
        let s = self.rope_slice(range)?;
        if let Some(str) = s.as_str() {
            Some(Cow::Borrowed(str))
        } else {
            Some(Cow::Owned(s.to_string()))
        }
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
        let byte_y = self.value.try_line_to_byte(y).expect("valid_roundtrip");
        let Some(mut it_line) = self.line_grapheme_idx(y) else {
            return None;
        };
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
        let Some(mut it_line) = self.line_grapheme_idx(pos.y) else {
            return None;
        };

        let len_bytes = self.value.len_bytes();
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
        self.value.try_byte_to_char(byte_range.start).ok()
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

        self.styles.remap(|r, _| Some(insert.expand(r)));
        self.anchor = insert.expand_pos(self.anchor);
        self.cursor = insert.expand_pos(self.cursor);

        if let Some(undo) = self.undo.as_mut() {
            undo.append(UndoEntry::InsertChar {
                chars: char_pos..char_pos + 1,
                cursor: TextPositionChange {
                    before: old_cursor,
                    after: self.cursor,
                },
                anchor: TextPositionChange {
                    before: old_anchor,
                    after: self.anchor,
                },
                range: insert,
                txt: c.to_string(),
            });
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

        self.styles.remap(|r, _| Some(insert.expand(r)));
        self.anchor = insert.expand_pos(self.anchor);
        self.cursor = insert.expand_pos(self.cursor);

        if let Some(undo) = self.undo.as_mut() {
            undo.append(UndoEntry::InsertStr {
                chars: char_pos..char_pos + char_count,
                cursor: TextPositionChange {
                    before: old_cursor,
                    after: self.cursor,
                },
                anchor: TextPositionChange {
                    before: old_anchor,
                    after: self.anchor,
                },
                range: insert,
                txt: t.to_string(),
            });
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
        let old_text = self.rope_slice(range).expect("some text").to_string();

        self.value.remove(start_pos..end_pos);

        // remove deleted styles.
        let mut changed_style = Vec::new();
        self.styles.remap(|r, s| {
            let new_range = range.shrink(r);
            if range.intersects(r) {
                changed_style.push(StyleChange {
                    before: r,
                    after: new_range,
                    style: s,
                });
                if new_range.is_empty() {
                    None
                } else {
                    Some(new_range)
                }
            } else {
                Some(new_range)
            }
        });
        self.anchor = range.shrink_pos(self.anchor);
        self.cursor = range.shrink_pos(self.anchor);

        if let Some(undo) = &mut self.undo {
            if char_range {
                undo.append(UndoEntry::RemoveChar {
                    chars: start_pos..end_pos,
                    cursor: TextPositionChange {
                        before: old_cursor,
                        after: self.cursor,
                    },
                    anchor: TextPositionChange {
                        before: old_anchor,
                        after: self.anchor,
                    },
                    range,
                    txt: old_text,
                    styles: changed_style,
                });
            } else {
                undo.append(UndoEntry::RemoveStr {
                    chars: start_pos..end_pos,
                    cursor: TextPositionChange {
                        before: old_cursor,
                        after: self.cursor,
                    },
                    anchor: TextPositionChange {
                        before: old_anchor,
                        after: self.anchor,
                    },
                    range,
                    txt: old_text,
                    styles: changed_style,
                });
            }
        }

        true
    }
}
