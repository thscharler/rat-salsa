use crate::range_map::expand_range_by;
use crate::TextPosition;
use dyn_clone::DynClone;
use std::fmt::Debug;
use std::mem;
use std::ops::Range;

/// Undo buffer.
///
/// Keeps up to undo_count operations that can be undone/redone.
/// Additionally, it can provide a change-log which can be used
/// to sync other text-widgets.
///
pub trait UndoBuffer: DynClone + Debug {
    /// How many undoes are stored?
    fn undo_count(&self) -> u32;

    /// How many undoes are stored?
    fn set_undo_count(&mut self, n: u32);

    /// Begin a sequence of changes that should be undone at once.
    ///
    /// begin/end calls can be nested, but only the outer one
    /// will define the actual scope of the undo.
    ///
    /// A call to begin_seq must be matched with a call to end_seq.
    fn begin_seq(&mut self);

    /// End a sequence of changes that should be undone at once.
    fn end_seq(&mut self);

    /// Appends a new operation at the current undo-position.
    ///
    /// Redoes will be truncated by this call.
    ///
    /// This call tries merge InsertChar/RemoveChar operations,
    /// if they lie next to each other. InsertStr/RemoveStr
    /// will never be merged.
    fn append(&mut self, undo: UndoEntry);

    /// Appends a new operation but doesn't fill the replay-log.
    ///
    /// Used to add to the undo-buffer during replay from another
    /// text-widget.
    fn append_from_replay(&mut self, undo: UndoEntry);

    /// Clear the undo and the replay buffer.
    fn clear(&mut self);

    /// Get the list of the next undo operations.
    fn undo(&mut self) -> Vec<UndoEntry>;

    /// Get the list of the next redo operations.
    fn redo(&mut self) -> Vec<UndoEntry>;

    /// Enable/disable replay recording.
    ///
    /// __Attention__:
    /// For this to work the widget state must be in a 'cleared' state,
    /// or you must *create a clone* of the widget-state *immediately* after
    /// activating the replay-log.
    ///
    /// Only then the replay operations obtained by recent_replay()
    /// will make sense to the clone.
    ///
    /// __Info__:
    /// How you identify the widgets that should receive the replay-log
    /// and other distribution problems are in the domain of the user
    /// of this feature.
    fn enable_replay_log(&mut self, replay: bool);

    /// Is the replay-log active?
    fn has_replay_log(&self) -> bool;

    /// Get the replay-log to sync with another textarea.
    /// This empties the replay buffer.
    fn recent_replay_log(&mut self) -> Vec<UndoEntry>;

    /// Is there undo for setting/removing styles.
    fn undo_styles_enabled(&self) -> bool;
}

/// Stores one style change.
#[derive(Debug, Default, Clone)]
pub struct StyleChange {
    pub before: Range<usize>,
    pub after: Range<usize>,
    pub style: usize,
}

/// Stores a text position change.
#[derive(Debug, Default, Clone)]
pub struct TextPositionChange {
    pub before: TextPosition,
    pub after: TextPosition,
}

/// Storage for undo.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum UndoEntry {
    /// Insert a single char/grapheme.
    ///
    /// This can contain a longer text, if consecutive InsertChar have
    /// been merged.
    InsertChar {
        /// byte range for the insert.
        bytes: Range<usize>,
        /// cursor position change
        cursor: TextPositionChange,
        /// anchor position change
        anchor: TextPositionChange,
        /// inserted text
        txt: String,
    },
    /// Insert a longer text.
    InsertStr {
        /// byte range for the insert.
        bytes: Range<usize>,
        /// cursor position change
        cursor: TextPositionChange,
        /// anchor position change
        anchor: TextPositionChange,
        /// inserted text
        txt: String,
    },
    /// Remove a single char/grapheme range.
    ///
    /// This can be a longer range, if consecutive RemoveChar have been
    /// merged.
    ///
    /// styles contains only styles whose range __intersects__ the
    /// removed range. Styles that lie after the bytes-range will be
    /// shifted left.
    RemoveChar {
        /// byte range for the remove.
        bytes: Range<usize>,
        /// cursor position change
        cursor: TextPositionChange,
        /// anchor position change
        anchor: TextPositionChange,
        /// removed text
        txt: String,
        /// removed styles
        styles: Vec<StyleChange>,
    },
    /// Remove longer text range.
    ///
    /// styles contains only styles whose range __intersects__ the
    /// removed range. Styles that lie after the bytes-range will be
    /// shifted left.
    RemoveStr {
        /// byte range for the remove.
        bytes: Range<usize>,
        /// cursor position change
        cursor: TextPositionChange,
        /// anchor position change
        anchor: TextPositionChange,
        /// removed text
        txt: String,
        /// removed styles
        styles: Vec<StyleChange>,
    },

    /// Cursor/anchor changed.
    ///
    /// This will be merged with a cursor-change immediately before.
    /// And it will merge with both removes and inserts.
    Cursor {
        /// cursor position change
        cursor: TextPositionChange,
        /// anchor position change
        anchor: TextPositionChange,
    },

    /// Set of styles was replaced.
    SetStyles {
        /// old styles
        styles_before: Vec<(Range<usize>, usize)>,
        /// new styles
        styles_after: Vec<(Range<usize>, usize)>,
    },
    /// Add one style.
    AddStyle {
        /// style range
        range: Range<usize>,
        /// style
        style: usize,
    },
    /// Remove one style.
    RemoveStyle {
        /// style range
        range: Range<usize>,
        /// style
        style: usize,
    },

    /// For replay only. Complete content was replaced.
    SetText {
        /// New text
        txt: String,
    },
    /// For replay only. Undo one operation.
    Undo,
    /// For replay only. Redo one operation.
    Redo,
}

/// Standard implementation for undo.
#[derive(Debug, Clone)]
pub struct UndoVec {
    undo_styles: bool,
    track_replay: bool,
    undo_count: u32,

    begin: u8,
    sequence: u32,
    buf: Vec<(u32, UndoEntry)>,
    replay: Vec<UndoEntry>,

    // undo/redo split
    idx: usize,
}

impl Default for UndoVec {
    fn default() -> Self {
        Self {
            undo_styles: false,
            track_replay: false,
            undo_count: 99,
            begin: 0,
            sequence: 0,
            buf: Vec::default(),
            replay: Vec::default(),
            idx: 0,
        }
    }
}

impl UndoVec {
    /// New undo.
    pub fn new(undo_count: u32) -> Self {
        Self {
            undo_count,
            ..Default::default()
        }
    }

    /// Enable undo for style changes.
    ///
    /// Usually not what you want.
    /// Unless you allow your users to set styles manually.
    /// If your styling is done by a parser, don't activate this.
    ///
    /// Changes to the range of styles and removal of styles
    /// caused by text edits *will* be undone anyway.
    ///
    /// Recording those operations for *replay* will not be affected
    /// by this setting.
    pub fn enable_undo_styles(&mut self, undo_styles: bool) {
        self.undo_styles = undo_styles;
    }

    /// Undo for styles are enabled.
    pub fn undo_styles(&self) -> bool {
        self.undo_styles
    }

    /// Append to undo buffer.
    fn _append(&mut self, undo: UndoEntry, replay: bool) {
        // tracking?
        if replay && self.track_replay {
            self.replay.push(undo.clone());
        }

        // only useful for tracking
        if matches!(
            undo,
            UndoEntry::Undo | UndoEntry::Redo | UndoEntry::SetText { .. }
        ) {
            return;
        }

        // style changes may/may not be undone
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

        // try merge
        let undo = if let Some((last_sequence, last)) = self.buf.pop() {
            let (last, undo) = self.merge_undo(last_sequence, last, undo);
            // re-add last if it survived merge
            if let Some(last) = last {
                self.buf.push(last);
            }
            undo
        } else {
            Some(undo)
        };

        // cap undo at capacity
        if self.buf.len() > self.undo_count as usize {
            // don't drop parts of current sequence at all.
            if self.buf[0].0 != self.sequence {
                let drop_sequence = self.buf[0].0;
                loop {
                    if self.buf[0].0 == drop_sequence {
                        self.buf.remove(0);
                    } else {
                        break;
                    }
                }
            }
        }

        // add new undo if it survived merge
        if let Some(undo) = undo {
            self.buf.push((self.sequence, undo));
            // auto begin+end
            if self.begin == 0 {
                self.sequence += 1;
            }
        }

        self.idx = self.buf.len();
    }

    fn merge_undo(
        &self,
        last_sequence: u32,
        mut last: UndoEntry,
        mut curr: UndoEntry,
    ) -> (Option<(u32, UndoEntry)>, Option<UndoEntry>) {
        if self.begin > 0 && last_sequence != self.sequence {
            return (Some((last_sequence, last)), Some(curr));
        }

        match &mut curr {
            UndoEntry::InsertChar {
                bytes: curr_bytes,
                cursor: curr_cursor,
                anchor: curr_anchor,
                txt: curr_txt,
            } => match &mut last {
                UndoEntry::InsertChar {
                    bytes: last_bytes,
                    cursor: last_cursor,
                    anchor: last_anchor,
                    txt: last_txt,
                } => {
                    if last_bytes.end == curr_bytes.start {
                        let mut last_txt = mem::take(last_txt);
                        last_txt.push_str(curr_txt);
                        (
                            Some((
                                last_sequence,
                                UndoEntry::InsertChar {
                                    bytes: last_bytes.start..curr_bytes.end,
                                    cursor: TextPositionChange {
                                        before: last_cursor.before,
                                        after: curr_cursor.after,
                                    },
                                    anchor: TextPositionChange {
                                        before: last_anchor.before,
                                        after: curr_anchor.after,
                                    },
                                    txt: last_txt,
                                },
                            )),
                            None,
                        )
                    } else {
                        (Some((last_sequence, last)), Some(curr))
                    }
                }
                _ => (Some((last_sequence, last)), Some(curr)),
            },
            UndoEntry::RemoveChar {
                bytes: curr_bytes,
                cursor: curr_cursor,
                anchor: curr_anchor,
                txt: curr_txt,
                styles: curr_styles,
            } => match &mut last {
                UndoEntry::RemoveChar {
                    bytes: last_bytes,
                    cursor: last_cursor,
                    anchor: last_anchor,
                    txt: last_txt,
                    styles: last_styles,
                } => {
                    if curr_bytes.end == last_bytes.start {
                        // backspace
                        let mut txt = mem::take(curr_txt);
                        txt.push_str(last_txt);

                        // merge into last_styles
                        let mut styles = mem::take(last_styles);
                        self.merge_remove_style(last_bytes.clone(), &mut styles, curr_styles);

                        (
                            Some((
                                last_sequence,
                                UndoEntry::RemoveChar {
                                    bytes: curr_bytes.start..last_bytes.end,
                                    cursor: TextPositionChange {
                                        before: last_cursor.before,
                                        after: curr_cursor.after,
                                    },
                                    anchor: TextPositionChange {
                                        before: last_anchor.before,
                                        after: curr_anchor.after,
                                    },
                                    txt,
                                    styles,
                                },
                            )),
                            None,
                        )
                    } else if curr_bytes.start == last_bytes.start {
                        // delete
                        let mut txt = mem::take(last_txt);
                        txt.push_str(curr_txt);

                        let curr_byte_len = curr_bytes.end - curr_bytes.start;

                        // merge into last_styles
                        let mut styles = mem::take(last_styles);
                        self.merge_remove_style(last_bytes.clone(), &mut styles, curr_styles);

                        (
                            Some((
                                last_sequence,
                                UndoEntry::RemoveChar {
                                    bytes: last_bytes.start..last_bytes.end + curr_byte_len,
                                    cursor: TextPositionChange {
                                        before: last_cursor.before,
                                        after: curr_cursor.after,
                                    },
                                    anchor: TextPositionChange {
                                        before: last_anchor.before,
                                        after: curr_anchor.after,
                                    },
                                    txt,
                                    styles,
                                },
                            )),
                            None,
                        )
                    } else {
                        (Some((last_sequence, last)), Some(curr))
                    }
                }
                _ => (Some((last_sequence, last)), Some(curr)),
            },

            UndoEntry::Cursor {
                cursor: curr_cursor,
                anchor: curr_anchor,
            } => match &mut last {
                UndoEntry::InsertChar {
                    bytes: last_bytes,
                    cursor: last_cursor,
                    anchor: last_anchor,
                    txt: last_txt,
                } => (
                    Some((
                        last_sequence,
                        UndoEntry::InsertChar {
                            bytes: mem::take(last_bytes),
                            cursor: TextPositionChange {
                                before: last_cursor.before,
                                after: curr_cursor.after,
                            },
                            anchor: TextPositionChange {
                                before: last_anchor.before,
                                after: curr_anchor.after,
                            },
                            txt: mem::take(last_txt),
                        },
                    )),
                    None,
                ),
                UndoEntry::InsertStr {
                    bytes: last_bytes,
                    cursor: last_cursor,
                    anchor: last_anchor,
                    txt: last_txt,
                } => (
                    Some((
                        last_sequence,
                        UndoEntry::InsertStr {
                            bytes: mem::take(last_bytes),
                            cursor: TextPositionChange {
                                before: last_cursor.before,
                                after: curr_cursor.after,
                            },
                            anchor: TextPositionChange {
                                before: last_anchor.before,
                                after: curr_anchor.after,
                            },
                            txt: mem::take(last_txt),
                        },
                    )),
                    None,
                ),
                UndoEntry::RemoveChar {
                    bytes: last_bytes,
                    cursor: last_cursor,
                    anchor: last_anchor,
                    txt: last_txt,
                    styles: last_styles,
                } => (
                    Some((
                        last_sequence,
                        UndoEntry::RemoveChar {
                            bytes: mem::take(last_bytes),
                            cursor: TextPositionChange {
                                before: last_cursor.before,
                                after: curr_cursor.after,
                            },
                            anchor: TextPositionChange {
                                before: last_anchor.before,
                                after: curr_anchor.after,
                            },
                            txt: mem::take(last_txt),
                            styles: mem::take(last_styles),
                        },
                    )),
                    None,
                ),
                UndoEntry::RemoveStr {
                    bytes: last_bytes,
                    cursor: last_cursor,
                    anchor: last_anchor,
                    txt: last_txt,
                    styles: last_styles,
                } => (
                    Some((
                        last_sequence,
                        UndoEntry::RemoveChar {
                            bytes: mem::take(last_bytes),
                            cursor: TextPositionChange {
                                before: last_cursor.before,
                                after: curr_cursor.after,
                            },
                            anchor: TextPositionChange {
                                before: last_anchor.before,
                                after: curr_anchor.after,
                            },
                            txt: mem::take(last_txt),
                            styles: mem::take(last_styles),
                        },
                    )),
                    None,
                ),
                UndoEntry::Cursor {
                    cursor: last_cursor,
                    anchor: last_anchor,
                } => (
                    Some((
                        last_sequence,
                        UndoEntry::Cursor {
                            cursor: TextPositionChange {
                                before: last_cursor.before,
                                after: curr_cursor.after,
                            },
                            anchor: TextPositionChange {
                                before: last_anchor.before,
                                after: curr_anchor.after,
                            },
                        },
                    )),
                    None,
                ),
                _ => (Some((last_sequence, last)), Some(curr)),
            },
            _ => (Some((last_sequence, last)), Some(curr)),
        }
    }

    /// Merge styles from two deletes.
    fn merge_remove_style(
        &self,
        last_range: Range<usize>,
        last: &mut Vec<StyleChange>,
        curr: &mut Vec<StyleChange>,
    ) {
        for i in (0..last.len()).rev() {
            for j in (0..curr.len()).rev() {
                if last[i].style == curr[j].style {
                    if last[i].after == curr[j].before {
                        last[i].after = curr[j].after.clone();
                        curr.remove(j);
                    }
                }
            }
        }

        // expand before and add
        for mut curr in curr.drain(..) {
            curr.before = expand_range_by(last_range.clone(), curr.before);
            last.push(curr);
        }
    }
}

impl UndoBuffer for UndoVec {
    fn undo_count(&self) -> u32 {
        self.undo_count
    }

    fn set_undo_count(&mut self, n: u32) {
        self.undo_count = n;
    }

    fn undo_styles_enabled(&self) -> bool {
        self.undo_styles
    }

    /// Begin a sequence of changes that should be undone at once.
    fn begin_seq(&mut self) {
        self.begin += 1;
        if self.begin == 1 {
            self.sequence += 1;
        }
    }

    fn end_seq(&mut self) {
        self.begin -= 1;
    }

    fn append(&mut self, undo: UndoEntry) {
        self._append(undo, true);
    }

    fn append_from_replay(&mut self, undo: UndoEntry) {
        self._append(undo, false);
    }

    fn clear(&mut self) {
        self.buf.clear();
        self.idx = 0;
        self.begin = 0;
        self.sequence = 0;
        self.replay.clear();
    }

    /// Get next undo
    fn undo(&mut self) -> Vec<UndoEntry> {
        if self.idx > 0 {
            let sequence = self.buf[self.idx - 1].0;
            let mut undo = Vec::new();
            loop {
                if self.buf[self.idx - 1].0 == sequence {
                    undo.push(self.buf[self.idx - 1].1.clone());
                    self.idx -= 1;
                } else {
                    break;
                }
                if self.idx == 0 {
                    break;
                }
            }
            undo
        } else {
            Vec::default()
        }
    }

    /// Get next redo
    fn redo(&mut self) -> Vec<UndoEntry> {
        if self.idx < self.buf.len() {
            let sequence = self.buf[self.idx].0;
            let mut redo = Vec::new();
            loop {
                if self.buf[self.idx].0 == sequence {
                    redo.push(self.buf[self.idx].1.clone());
                    self.idx += 1;
                } else {
                    break;
                }
                if self.idx == self.buf.len() {
                    break;
                }
            }
            redo
        } else {
            Vec::default()
        }
    }

    /// Enable replay functionality.
    ///
    /// This keeps track of all changes to a textarea.
    /// These changes can be copied to another textarea with
    /// the replay() function.
    fn enable_replay_log(&mut self, replay: bool) {
        if self.track_replay != replay {
            self.replay.clear();
        }
        self.track_replay = replay;
    }

    fn has_replay_log(&self) -> bool {
        self.track_replay
    }

    /// Get all new replay entries.
    fn recent_replay_log(&mut self) -> Vec<UndoEntry> {
        mem::take(&mut self.replay)
    }
}
