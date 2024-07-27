use crate::text::graphemes::{char_len, str_line_len};
use crate::text::textarea_core::{TextPosition, TextRange};
use log::debug;
use std::fmt::Debug;
use std::mem;
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct StyledRangeChange {
    pub before: TextRange,
    pub after: TextRange,
    pub style: usize,
}

#[derive(Debug, Clone)]
pub struct TextPositionChange {
    pub before: TextPosition,
    pub after: TextPosition,
}

/// Undo buffer.
pub trait UndoBuffer: Debug {
    /// Add an insert operation to the undo buffer.
    fn insert_char(
        &mut self,
        char_pos: usize,
        before_cursor: TextPosition,
        before_anchor: TextPosition,
        after_cursor: TextPosition,
        after_anchor: TextPosition,
        range: TextRange,
        c: char,
    );

    /// Add an insert operation to the undo buffer.
    fn insert_str(
        &mut self,
        chars: (usize, usize),
        cursor: TextPosition,
        anchor: TextPosition,
        after_cursor: TextPosition,
        after_anchor: TextPosition,
        range: TextRange,
        txt: String,
    );

    /// Add a remove operation to the undo buffer.
    fn remove_char(
        &mut self,
        chars: Range<usize>,
        cursor: TextPositionChange,
        anchor: TextPositionChange,
        range: TextRange,
        txt: String,
        styles: Vec<StyledRangeChange>,
    );

    /// Add a remove operation to the undo buffer.
    fn remove_str(
        &mut self,
        chars: Range<usize>,
        cursor: TextPositionChange,
        anchor: TextPositionChange,
        range: TextRange,
        txt: String,
        styles: Vec<StyledRangeChange>,
    );

    /// Next undo.
    fn undo(&mut self) -> Option<UndoEntry>;

    /// Next redo.
    fn redo(&mut self) -> Option<UndoEntry>;

    /// Reset undo.
    fn clear(&mut self);
}

/// Storage for undo.
#[derive(Debug, Clone)]
pub enum UndoEntry {
    InsertChar {
        chars: (usize, usize),
        cursor: TextPosition,
        anchor: TextPosition,
        redo_cursor: TextPosition,
        redo_anchor: TextPosition,
        range: TextRange,
        txt: String,
    },
    InsertStr {
        chars: (usize, usize),
        cursor: TextPosition,
        anchor: TextPosition,
        redo_cursor: TextPosition,
        redo_anchor: TextPosition,
        range: TextRange,
        txt: String,
    },
    RemoveChar {
        chars: Range<usize>,
        cursor: TextPositionChange,
        anchor: TextPositionChange,
        range: TextRange,
        txt: String,
        styles: Vec<StyledRangeChange>,
    },
    RemoveStr {
        chars: Range<usize>,
        cursor: TextPositionChange,
        anchor: TextPositionChange,
        range: TextRange,
        txt: String,
        styles: Vec<StyledRangeChange>,
    },
}

/// Standard implementation for undo.
#[derive(Debug, Default)]
pub struct UndoVec {
    buf: Vec<UndoEntry>,
    idx: usize,
}

impl UndoVec {
    pub fn new(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
            idx: 0,
        }
    }

    fn merge(
        &mut self,
        mut last: UndoEntry,
        mut curr: UndoEntry,
    ) -> (Option<UndoEntry>, Option<UndoEntry>) {
        match &mut last {
            UndoEntry::InsertChar {
                chars: last_chars,
                cursor: last_cursor,
                anchor: last_anchor,
                redo_cursor: _last_redo_cursor,
                redo_anchor: _last_redo_anchor,
                range: last_range,
                txt: last_txt,
            } => match &mut curr {
                UndoEntry::InsertChar {
                    chars: curr_chars,
                    cursor: _curr_cursor,
                    anchor: _curr_anchor,
                    redo_cursor: curr_redo_cursor,
                    redo_anchor: curr_redo_anchor,
                    range: curr_range,
                    txt: curr_txt,
                } => {
                    if last_chars.1 == curr_chars.0 {
                        let mut last_txt = mem::take(last_txt);
                        last_txt.push_str(curr_txt);
                        (
                            Some(UndoEntry::InsertChar {
                                chars: (last_chars.0, curr_chars.1),
                                cursor: *last_cursor,
                                anchor: *last_anchor,
                                redo_cursor: *curr_redo_cursor,
                                redo_anchor: *curr_redo_anchor,
                                range: TextRange::new(last_range.start, curr_range.end),
                                txt: last_txt,
                            }),
                            None,
                        )
                    } else {
                        (Some(last), Some(curr))
                    }
                }
                _ => (Some(last), Some(curr)),
            },
            UndoEntry::RemoveChar {
                chars: last_chars,
                cursor: last_cursor,
                anchor: last_anchor,
                range: last_range,
                txt: last_txt,
                styles: last_styles,
            } => match &mut curr {
                UndoEntry::RemoveChar {
                    chars: curr_chars,
                    cursor: curr_cursor,
                    anchor: curr_anchor,
                    range: curr_range,
                    txt: curr_txt,
                    styles: curr_styles,
                } => {
                    if curr_chars.end == last_chars.start {
                        debug!("backspace {:#?}\n=> {:#?}", last_range, curr_range);
                        debug!("backspace {:#?}\n=> {:#?}", last_styles, curr_styles);
                        // backspace
                        let mut txt = mem::take(curr_txt);
                        txt.push_str(last_txt);

                        // merge into last_styles
                        let mut styles = mem::take(last_styles);
                        Self::remove_merge_style(*last_range, &mut styles, curr_styles);

                        debug!("backspace m {:#?}", styles);

                        (
                            Some(UndoEntry::RemoveChar {
                                chars: curr_chars.start..last_chars.end,
                                cursor: TextPositionChange {
                                    before: last_cursor.before,
                                    after: curr_cursor.after,
                                },
                                anchor: TextPositionChange {
                                    before: last_anchor.before,
                                    after: curr_anchor.after,
                                },
                                range: TextRange::new(curr_range.start, last_range.end),
                                txt,
                                styles,
                            }),
                            None,
                        )
                    } else if curr_chars.start == last_chars.start {
                        debug!("delete {:#?}\n=> {:#?}", last_range, curr_range);
                        debug!("delete {:#?}\n=> {:#?}", last_styles, curr_styles);
                        // delete
                        let mut txt = mem::take(last_txt);
                        txt.push_str(curr_txt);

                        let delta_x = char_len(curr_txt);
                        let delta_y = if matches!(curr_txt.as_str(), "\n" | "\r\n") {
                            1
                        } else {
                            0
                        };

                        // merge into last_styles
                        let mut styles = mem::take(last_styles);
                        Self::remove_merge_style(*last_range, &mut styles, curr_styles);

                        debug!("delete m {:#?}", styles);

                        (
                            Some(UndoEntry::RemoveChar {
                                chars: last_chars.start..last_chars.end + delta_x,
                                cursor: TextPositionChange {
                                    before: last_cursor.before,
                                    after: curr_cursor.after,
                                },
                                anchor: TextPositionChange {
                                    before: last_anchor.before,
                                    after: curr_anchor.after,
                                },
                                range: TextRange::new(
                                    last_range.start,
                                    TextPosition::new(
                                        last_range.end.x + delta_x,
                                        last_range.end.y + delta_y,
                                    ),
                                ),
                                txt,
                                styles,
                            }),
                            None,
                        )
                    } else {
                        (Some(last), Some(curr))
                    }
                }
                _ => (Some(last), Some(curr)),
            },

            UndoEntry::InsertStr { .. } => (Some(last), Some(curr)),
            UndoEntry::RemoveStr { .. } => (Some(last), Some(curr)),
        }
    }

    /// Merge styles from two deletes.
    fn remove_merge_style(
        last_range: TextRange,
        last: &mut Vec<StyledRangeChange>,
        curr: &mut Vec<StyledRangeChange>,
    ) {
        for i in (0..last.len()).rev() {
            for j in (0..curr.len()).rev() {
                if last[i].style == curr[j].style {
                    if last[i].after == curr[j].before {
                        last[i].after = curr[j].after;
                        curr.remove(j);
                    }
                }
            }
        }

        // expand before and add
        for mut curr in curr.drain(..) {
            curr.before = last_range.expand(curr.before);
            last.push(curr);
        }
    }

    fn append(&mut self, undo: UndoEntry) {
        // remove redo
        while self.idx < self.buf.len() {
            self.buf.pop();
        }

        let (last, undo) = if let Some(last) = self.buf.pop() {
            self.merge(last, undo)
        } else {
            (None, Some(undo))
        };

        if let Some(last) = last {
            self.buf.push(last);
        }
        if self.buf.capacity() == self.buf.len() {
            self.buf.remove(0);
        }
        if let Some(undo) = undo {
            self.buf.push(undo);
        }
        self.idx = self.buf.len();
    }
}

impl UndoBuffer for UndoVec {
    fn insert_char(
        &mut self,
        char_pos: usize,
        before_cursor: TextPosition,
        before_anchor: TextPosition,
        after_cursor: TextPosition,
        after_anchor: TextPosition,
        range: TextRange,
        c: char,
    ) {
        self.append(UndoEntry::InsertChar {
            chars: (char_pos, char_pos + 1),
            cursor: before_cursor,
            anchor: before_anchor,
            redo_cursor: after_cursor,
            redo_anchor: after_anchor,
            range,
            txt: c.to_string(),
        });
    }

    fn insert_str(
        &mut self,
        chars: (usize, usize),
        before_cursor: TextPosition,
        before_anchor: TextPosition,
        after_cursor: TextPosition,
        after_anchor: TextPosition,
        range: TextRange,
        txt: String,
    ) {
        self.append(UndoEntry::InsertStr {
            chars,
            cursor: before_cursor,
            anchor: before_anchor,
            redo_cursor: after_cursor,
            redo_anchor: after_anchor,
            range,
            txt,
        });
    }

    fn remove_char(
        &mut self,
        chars: Range<usize>,
        cursor: TextPositionChange,
        anchor: TextPositionChange,
        range: TextRange,
        txt: String,
        styles: Vec<StyledRangeChange>,
    ) {
        self.append(UndoEntry::RemoveChar {
            chars,
            cursor,
            anchor,
            range,
            txt,
            styles,
        });
    }

    fn remove_str(
        &mut self,
        chars: Range<usize>,
        cursor: TextPositionChange,
        anchor: TextPositionChange,
        range: TextRange,
        txt: String,
        styles: Vec<StyledRangeChange>,
    ) {
        self.append(UndoEntry::RemoveStr {
            chars,
            cursor,
            anchor,
            range,
            txt,
            styles,
        });
    }

    fn undo(&mut self) -> Option<UndoEntry> {
        if self.idx > 0 {
            self.idx -= 1;
            Some(self.buf[self.idx].clone())
        } else {
            None
        }
    }

    fn redo(&mut self) -> Option<UndoEntry> {
        if self.idx < self.buf.len() {
            self.idx += 1;
            Some(self.buf[self.idx - 1].clone())
        } else {
            None
        }
    }

    fn clear(&mut self) {
        self.buf.clear();
        self.idx = 0;
    }
}
