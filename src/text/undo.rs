use crate::text::graphemes::char_len;
use crate::text::textarea_core::{TextPosition, TextRange};
use ropey::Rope;
use std::fmt::Debug;
use std::mem;
use std::ops::Range;

/// Undo buffer.
pub trait UndoBuffer: Debug {
    fn cloned(&self) -> Box<dyn UndoBuffer>;

    /// Sets a new value.
    fn append(&mut self, undo: UndoEntry);

    /// Sets a new value, never adds to replay.
    fn append_no_replay(&mut self, undo: UndoEntry);

    /// Next undo.
    fn undo(&mut self) -> Option<UndoEntry>;

    /// Next redo.
    fn redo(&mut self) -> Option<UndoEntry>;

    /// Enable/disable replay recording.
    fn set_replay(&mut self, replay: bool);

    /// Is replay active?
    fn replay(&self) -> bool;

    /// Get the replay information to sync with another textarea.
    /// This empties the replay buffer.
    fn recent_replay(&mut self) -> Vec<UndoEntry>;
}

/// Stores one style change.
#[derive(Debug, Default, Clone)]
pub struct StyleChange {
    pub before: TextRange,
    pub after: TextRange,
    pub style: usize,
}

/// Stores a text position change.
#[derive(Debug, Default, Clone)]
pub struct TextPositionChange {
    pub before: TextPosition,
    pub after: TextPosition,
}

/// Storage for undo.
#[derive(Debug, Clone)]
pub enum UndoEntry {
    SetText {
        txt_before: Rope,
        txt_after: Rope,
        cursor: TextPositionChange,
        anchor: TextPositionChange,
        styles_before: Vec<(TextRange, usize)>,
    },
    InsertChar {
        /// char range for the insert.
        chars: Range<usize>,
        /// cursor position change
        cursor: TextPositionChange,
        /// anchor position change
        anchor: TextPositionChange,
        /// text range for the insert.
        range: TextRange,
        /// inserted text
        txt: String,
    },
    InsertStr {
        /// char range for the insert.
        chars: Range<usize>,
        /// cursor position change
        cursor: TextPositionChange,
        /// anchor position change
        anchor: TextPositionChange,
        /// text range for the insert.
        range: TextRange,
        /// inserted text
        txt: String,
    },
    RemoveChar {
        /// char range for the remove.
        chars: Range<usize>,
        /// cursor position change
        cursor: TextPositionChange,
        /// anchor position change
        anchor: TextPositionChange,
        /// text range for the remove.
        range: TextRange,
        /// removed text
        txt: String,
        /// removed styles
        styles: Vec<StyleChange>,
    },
    RemoveStr {
        /// char range for the remove.
        chars: Range<usize>,
        /// cursor position change
        cursor: TextPositionChange,
        /// anchor position change
        anchor: TextPositionChange,
        /// text range for the remove.
        range: TextRange,
        /// removed text
        txt: String,
        /// removed styles
        styles: Vec<StyleChange>,
    },
    SetStyles {
        styles_before: Vec<(TextRange, usize)>,
        styles_after: Vec<(TextRange, usize)>,
    },
    AddStyle {
        range: TextRange,
        style: usize,
    },
    RemoveStyle {
        range: TextRange,
        style: usize,
    },
    /// For replay only.
    Undo,
    /// For replay only.
    Redo,
}

/// Standard implementation for undo.
#[derive(Debug)]
pub struct UndoVec {
    undo_styles: bool,
    track_replay: bool,
    buf: Vec<UndoEntry>,
    replay: Vec<UndoEntry>,
    idx: usize,
}

impl Default for UndoVec {
    fn default() -> Self {
        Self {
            undo_styles: false,
            track_replay: false,
            buf: Vec::with_capacity(40),
            replay: Vec::default(),
            idx: 0,
        }
    }
}

impl UndoVec {
    pub fn new(capacity: usize) -> Self {
        Self {
            undo_styles: false,
            track_replay: false,
            buf: Vec::with_capacity(capacity),
            replay: Vec::default(),
            idx: 0,
        }
    }

    /// Undo style-changes.
    ///
    /// This is not recommended if you do your styling
    /// with some parser and rerun styling after every text-change.
    pub fn set_undo_styles(&mut self, undo_styles: bool) {
        self.undo_styles = undo_styles;
    }

    /// Append to undo buffer.
    fn _append(&mut self, undo: UndoEntry, replay: bool) {
        // tracking?
        if replay && self.track_replay {
            self.replay.push(undo.clone());
        }
        // only useful for tracking
        match &undo {
            UndoEntry::Undo | UndoEntry::Redo => return,
            _ => {}
        }

        // filter style changes
        if !self.undo_styles {
            match &undo {
                UndoEntry::SetStyles { .. }
                | UndoEntry::AddStyle { .. }
                | UndoEntry::RemoveStyle { .. } => return,
                _ => {}
            }
        }

        // remove redo
        while self.idx < self.buf.len() {
            self.buf.pop();
        }
        let (last, undo) = if let Some(last) = self.buf.pop() {
            merge_undo(last, undo)
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
    fn cloned(&self) -> Box<dyn UndoBuffer> {
        Box::new(Self {
            undo_styles: self.undo_styles,
            track_replay: self.track_replay,
            buf: self.buf.clone(),
            replay: self.replay.clone(),
            idx: self.idx,
        })
    }

    fn append(&mut self, undo: UndoEntry) {
        self._append(undo, true);
    }

    fn append_no_replay(&mut self, undo: UndoEntry) {
        self._append(undo, false);
    }

    /// Get next undo
    fn undo(&mut self) -> Option<UndoEntry> {
        if self.idx > 0 {
            self.idx -= 1;
            Some(self.buf[self.idx].clone())
        } else {
            None
        }
    }

    /// Get next redo
    fn redo(&mut self) -> Option<UndoEntry> {
        if self.idx < self.buf.len() {
            self.idx += 1;
            Some(self.buf[self.idx - 1].clone())
        } else {
            None
        }
    }

    /// Enable replay functionality.
    ///
    /// This keeps track of all changes to a textarea.
    /// These changes can be copied to another textarea with
    /// the replay() function.
    fn set_replay(&mut self, replay: bool) {
        if self.track_replay != replay {
            self.replay.clear();
        }
        self.track_replay = replay;
    }

    fn replay(&self) -> bool {
        self.track_replay
    }

    /// Get all new replay entries.
    fn recent_replay(&mut self) -> Vec<UndoEntry> {
        mem::take(&mut self.replay)
    }
}

fn merge_undo(mut last: UndoEntry, mut curr: UndoEntry) -> (Option<UndoEntry>, Option<UndoEntry>) {
    match &mut last {
        UndoEntry::InsertChar {
            chars: last_chars,
            cursor: last_cursor,
            anchor: last_anchor,
            range: last_range,
            txt: last_txt,
        } => match &mut curr {
            UndoEntry::InsertChar {
                chars: curr_chars,
                cursor: curr_cursor,
                anchor: curr_anchor,
                range: curr_range,
                txt: curr_txt,
            } => {
                if last_chars.end == curr_chars.start {
                    let mut last_txt = mem::take(last_txt);
                    last_txt.push_str(curr_txt);
                    (
                        Some(UndoEntry::InsertChar {
                            chars: last_chars.start..curr_chars.end,
                            cursor: TextPositionChange {
                                before: last_cursor.before,
                                after: curr_cursor.after,
                            },
                            anchor: TextPositionChange {
                                before: last_anchor.before,
                                after: curr_anchor.after,
                            },
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
                    // backspace
                    let mut txt = mem::take(curr_txt);
                    txt.push_str(last_txt);

                    // merge into last_styles
                    let mut styles = mem::take(last_styles);
                    merge_remove_style(*last_range, &mut styles, curr_styles);

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
                    merge_remove_style(*last_range, &mut styles, curr_styles);

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

        _ => (Some(last), Some(curr)),
    }
}

/// Merge styles from two deletes.
fn merge_remove_style(
    last_range: TextRange,
    last: &mut Vec<StyleChange>,
    curr: &mut Vec<StyleChange>,
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
