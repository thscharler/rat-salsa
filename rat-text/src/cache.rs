use crate::glyph2::TextWrap2;
use crate::{upos_type, TextPosition, TextRange};
#[cfg(not(debug_assertions))]
use fxhash::FxBuildHasher;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
#[cfg(debug_assertions)]
use std::collections::BTreeSet;
#[cfg(not(debug_assertions))]
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::ops::Range;
use std::rc::Rc;

#[cfg(not(debug_assertions))]
type Map<K, V> = HashMap<K, V, FxBuildHasher>;
#[cfg(debug_assertions)]
type Map<K, V> = BTreeMap<K, V>;

#[cfg(not(debug_assertions))]
type Set<K> = HashSet<K, FxBuildHasher>;
#[cfg(debug_assertions)]
type Set<K> = BTreeSet<K>;

/// Glyph cache.
#[derive(Debug, Clone, Default)]
pub struct Cache {
    /// Cache validity: wrapping mode
    pub(crate) text_wrap: Cell<TextWrap2>,
    /// Cache validity: left shift
    pub(crate) shift_left: Cell<upos_type>,
    /// Cache validity: rendered text-width.
    pub(crate) screen_width: Cell<upos_type>,
    /// Cache validity: rendered text-height.
    pub(crate) screen_height: Cell<upos_type>,
    /// Cache validity: show ctrl-chars (changes width!)
    pub(crate) screen_ctrl: Cell<bool>,

    /// line-width the same
    pub(crate) line_width: Rc<RefCell<Map<upos_type, LineWidthCache>>>,
    /// position to bytes, for glyphs2()
    pub(crate) pos_to_bytes: Rc<RefCell<Map<TextPosition, Range<usize>>>>,
    /// range to bytes, for glyphs2()
    pub(crate) range_to_bytes: Rc<RefCell<Map<TextRange, Range<usize>>>>,

    /// Mark the byte-positions of each line-start.
    /// Used when text-wrap is ShiftText.
    pub(crate) line_start: Rc<RefCell<Map<upos_type, LineOffsetCache>>>,

    /// Has the specific line been fully wrapped from column 0 to width.
    pub(crate) full_line_break: Rc<RefCell<Set<upos_type>>>,

    /// All known line-breaks for wrapped text.
    /// Has the text-position of the glyph which is marked as 'line-break'.
    /// That means the line-break occurs *after* this position.
    pub(crate) line_break: Rc<RefCell<BTreeMap<TextPosition, LineBreakCache>>>,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct LineOffsetCache {
    // start column of the row
    pub pos_x: upos_type,
    // start screen column of the row
    pub screen_pos_x: upos_type,
    // byte pos of the line start offset
    pub byte_pos: usize,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct LineBreakCache {
    // start of new line.
    pub start_pos: TextPosition,
    // byte pos of the break
    pub byte_pos: usize,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct LineWidthCache {
    // line-width
    pub width: upos_type,
    // start byte pos of the line
    pub byte_pos: usize,
}

impl Cache {
    pub(crate) fn clear(&self) {
        self.line_width.borrow_mut().clear();
        self.pos_to_bytes.borrow_mut().clear();
        self.range_to_bytes.borrow_mut().clear();
        self.shift_left.set(0);
        self.screen_width.set(0);
        self.screen_height.set(0);
        self.screen_ctrl.set(false);
        self.line_start.borrow_mut().clear();
        self.full_line_break.borrow_mut().clear();
        self.line_break.borrow_mut().clear();
    }

    /// Clear out parts of the cache that correspond to
    /// text-changes after the given byte-position.
    pub(crate) fn validate_byte_pos(&self, byte_pos: Option<usize>) {
        let Some(byte_pos) = byte_pos else {
            return;
        };

        self.line_width
            .borrow_mut()
            .retain(|_, cache| cache.byte_pos < byte_pos);

        self.line_start
            .borrow_mut()
            .retain(|_, cache| cache.byte_pos < byte_pos);

        self.pos_to_bytes
            .borrow_mut()
            .retain(|_, cache| cache.end < byte_pos);
        self.range_to_bytes
            .borrow_mut()
            .retain(|_, cache| cache.end < byte_pos);

        self.line_break.borrow_mut().retain(|pos, cache| {
            if cache.byte_pos < byte_pos {
                true
            } else {
                self.full_line_break.borrow_mut().remove(&pos.y);
                false
            }
        });
    }

    /// Remove stuff from the cache, that doesn't match the parameters.
    pub(crate) fn validate(
        &self,
        text_wrap: TextWrap2,
        shift_left: upos_type,
        screen_width: upos_type,
        screen_height: upos_type,
        screen_ctrl: bool,
        byte_pos: Option<usize>,
    ) {
        if text_wrap != self.text_wrap.get() {
            self.full_line_break.borrow_mut().clear();
            self.line_break.borrow_mut().clear();
            self.line_start.borrow_mut().clear();
        }

        match text_wrap {
            TextWrap2::Shift => {
                if self.shift_left.get() != shift_left {
                    self.line_start.borrow_mut().clear();
                }
            }
            TextWrap2::Hard | TextWrap2::Word => {
                self.line_start.borrow_mut().clear();
                if self.screen_width.get() != screen_width
                    || self.screen_height.get() != screen_height
                    || self.screen_ctrl.get() != screen_ctrl
                {
                    self.line_break.borrow_mut().clear();
                    self.full_line_break.borrow_mut().clear();
                }
            }
        }

        self.validate_byte_pos(byte_pos);

        self.text_wrap.set(text_wrap);
        self.shift_left.set(shift_left);
        self.screen_width.set(screen_width);
        self.screen_height.set(screen_height);
        self.screen_ctrl.set(screen_ctrl);
    }
}
