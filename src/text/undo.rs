use crate::text::graphemes::{char_len, str_line_len};
use crate::text::textarea_core::TextRange;
use log::debug;
use std::fmt::Debug;
use std::mem;

/// Undo buffer.
pub trait UndoBuffer: Debug {
    /// Add an insert operation to the undo buffer.
    fn insert_char(
        &mut self,
        char_pos: usize,
        before_cursor: (usize, usize),
        before_anchor: (usize, usize),
        after_cursor: (usize, usize),
        after_anchor: (usize, usize),
        range: TextRange,
        c: char,
    );

    /// Add an insert operation to the undo buffer.
    fn insert_str(
        &mut self,
        chars: (usize, usize),
        cursor: (usize, usize),
        anchor: (usize, usize),
        after_cursor: (usize, usize),
        after_anchor: (usize, usize),
        range: TextRange,
        txt: String,
    );

    /// Add a remove operation to the undo buffer.
    fn remove_char(
        &mut self,
        chars: (usize, usize),
        cursor: (usize, usize),
        anchor: (usize, usize),
        after_cursor: (usize, usize),
        after_anchor: (usize, usize),
        range: TextRange,
        txt: String,
    );

    /// Add a remove operation to the undo buffer.
    fn remove_str(
        &mut self,
        chars: (usize, usize),
        cursor: (usize, usize),
        anchor: (usize, usize),
        after_cursor: (usize, usize),
        after_anchor: (usize, usize),
        range: TextRange,
        txt: String,
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
        cursor: (usize, usize),
        anchor: (usize, usize),
        redo_cursor: (usize, usize),
        redo_anchor: (usize, usize),
        range: TextRange,
        txt: String,
    },
    InsertStr {
        chars: (usize, usize),
        cursor: (usize, usize),
        anchor: (usize, usize),
        redo_cursor: (usize, usize),
        redo_anchor: (usize, usize),
        range: TextRange,
        txt: String,
    },
    RemoveChar {
        chars: (usize, usize),
        cursor: (usize, usize),
        anchor: (usize, usize),
        redo_cursor: (usize, usize),
        redo_anchor: (usize, usize),
        range: TextRange,
        txt: String,
    },
    RemoveStr {
        chars: (usize, usize),
        cursor: (usize, usize),
        anchor: (usize, usize),
        redo_cursor: (usize, usize),
        redo_anchor: (usize, usize),
        range: TextRange,
        txt: String,
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
                redo_cursor: _last_redo_cursor,
                redo_anchor: _last_redo_anchor,
                range: last_range,
                txt: last_txt,
            } => match &mut curr {
                UndoEntry::RemoveChar {
                    chars: curr_chars,
                    cursor: _curr_cursor,
                    anchor: _curr_anchor,
                    redo_cursor: curr_redo_cursor,
                    redo_anchor: curr_redo_anchor,
                    range: curr_range,
                    txt: curr_txt,
                } => {
                    if curr_chars.1 == last_chars.0 {
                        // backspace
                        let mut curr_txt = mem::take(curr_txt);
                        curr_txt.push_str(last_txt);
                        (
                            Some(UndoEntry::RemoveChar {
                                chars: (curr_chars.0, last_chars.1),
                                cursor: *last_cursor,
                                anchor: *last_anchor,
                                redo_cursor: *curr_redo_cursor,
                                redo_anchor: *curr_redo_anchor,
                                range: TextRange::new(curr_range.start, last_range.end),
                                txt: curr_txt,
                            }),
                            None,
                        )
                    } else if curr_chars.0 == last_chars.0 {
                        // delete
                        let mut last_txt = mem::take(last_txt);
                        last_txt.push_str(curr_txt);
                        (
                            Some(UndoEntry::RemoveChar {
                                chars: (last_chars.0, last_chars.1 + char_len(curr_txt)),
                                cursor: *last_cursor,
                                anchor: *last_anchor,
                                redo_cursor: *curr_redo_cursor,
                                redo_anchor: *curr_redo_anchor,
                                range: TextRange::new(curr_range.start, last_range.end),
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

            UndoEntry::InsertStr { .. } => (Some(last), Some(curr)),
            UndoEntry::RemoveStr { .. } => (Some(last), Some(curr)),
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
        before_cursor: (usize, usize),
        before_anchor: (usize, usize),
        after_cursor: (usize, usize),
        after_anchor: (usize, usize),
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
        before_cursor: (usize, usize),
        before_anchor: (usize, usize),
        after_cursor: (usize, usize),
        after_anchor: (usize, usize),
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
        chars: (usize, usize),
        before_cursor: (usize, usize),
        before_anchor: (usize, usize),
        after_cursor: (usize, usize),
        after_anchor: (usize, usize),
        range: TextRange,
        txt: String,
    ) {
        self.append(UndoEntry::RemoveChar {
            chars,
            cursor: before_cursor,
            anchor: before_anchor,
            redo_cursor: after_cursor,
            redo_anchor: after_anchor,
            range,
            txt,
        });
    }

    fn remove_str(
        &mut self,
        chars: (usize, usize),
        before_cursor: (usize, usize),
        before_anchor: (usize, usize),
        after_cursor: (usize, usize),
        after_anchor: (usize, usize),
        range: TextRange,
        txt: String,
    ) {
        self.append(UndoEntry::RemoveStr {
            chars,
            cursor: before_cursor,
            anchor: before_anchor,
            redo_cursor: after_cursor,
            redo_anchor: after_anchor,
            range,
            txt,
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
