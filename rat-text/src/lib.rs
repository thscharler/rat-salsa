#![doc = include_str!("../readme.md")]
#![allow(clippy::uninlined_format_args)]
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Range;

pub mod clipboard;
#[cfg(feature = "palette")]
pub mod color_input;
pub mod date_input;
pub mod line_number;
pub mod number_input;
pub mod text_area;
pub mod text_input;
pub mod text_input_mask;
pub mod undo_buffer;

mod cache;
mod glyph2;
mod grapheme;
mod range_map;
mod text_core;
mod text_store;

pub use grapheme::Grapheme;

use crate::_private::NonExhaustive;
pub use pure_rust_locales::Locale;
pub use rat_cursor::{HasScreenCursor, impl_screen_cursor, screen_cursor};
use rat_scrolled::ScrollStyle;
use ratatui::style::Style;
use ratatui::widgets::Block;

pub mod event {
    //!
    //! Event-handler traits and Keybindings.
    //!

    pub use rat_event::*;

    /// Runs only the navigation events, not any editing.
    #[derive(Debug)]
    pub struct ReadOnly;

    /// Result of event handling.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum TextOutcome {
        /// The given event has not been used at all.
        Continue,
        /// The event has been recognized, but the result was nil.
        /// Further processing for this event may stop.
        Unchanged,
        /// The event has been recognized and there is some change
        /// due to it.
        /// Further processing for this event may stop.
        /// Rendering the ui is advised.
        Changed,
        /// Text content has changed.
        TextChanged,
    }

    impl ConsumedEvent for TextOutcome {
        fn is_consumed(&self) -> bool {
            *self != TextOutcome::Continue
        }
    }

    // Useful for converting most navigation/edit results.
    impl From<bool> for TextOutcome {
        fn from(value: bool) -> Self {
            if value {
                TextOutcome::Changed
            } else {
                TextOutcome::Unchanged
            }
        }
    }

    impl From<Outcome> for TextOutcome {
        fn from(value: Outcome) -> Self {
            match value {
                Outcome::Continue => TextOutcome::Continue,
                Outcome::Unchanged => TextOutcome::Unchanged,
                Outcome::Changed => TextOutcome::Changed,
            }
        }
    }

    impl From<TextOutcome> for Outcome {
        fn from(value: TextOutcome) -> Self {
            match value {
                TextOutcome::Continue => Outcome::Continue,
                TextOutcome::Unchanged => Outcome::Unchanged,
                TextOutcome::Changed => Outcome::Changed,
                TextOutcome::TextChanged => Outcome::Changed,
            }
        }
    }
}

/// This flag sets the behaviour of the widget when
/// it detects that it gained focus.
///
/// Available for all text-input widgets except TextArea.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TextFocusGained {
    /// None
    None,
    /// Editing overwrites the current content.
    /// Any movement resets this flag and allows editing.
    #[default]
    Overwrite,
    /// Select all text on focus gain.
    SelectAll,
}

/// This flag sets the behaviour of the widget when
/// it detects that it lost focus.
///
/// Available for all text-input widgets except TextArea.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TextFocusLost {
    /// None
    None,
    /// Sets the offset to 0. This prevents strangely clipped
    /// text for long inputs.
    #[default]
    Position0,
}

/// This flag sets the behaviour of the widget when
/// Tab/BackTab is pressed.
///
/// Available for MaskedInput.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TextTab {
    /// Tab jumps to the next section of the masked input.
    #[default]
    MoveToNextSection,
    /// Tab behaves regular and jumps to the next widget.
    MoveToNextWidget,
}

/// Combined style for the widget.
#[derive(Debug, Clone)]
pub struct TextStyle {
    pub style: Style,
    pub scroll: Option<ScrollStyle>,
    pub block: Option<Block<'static>>,
    pub border_style: Option<Style>,
    pub title_style: Option<Style>,
    pub focus: Option<Style>,
    pub select: Option<Style>,
    pub invalid: Option<Style>,

    /// Focus behaviour.
    pub on_focus_gained: Option<TextFocusGained>,
    /// Focus behaviour.
    pub on_focus_lost: Option<TextFocusLost>,
    /// Tab behaviour.
    pub on_tab: Option<TextTab>,

    pub non_exhaustive: NonExhaustive,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            scroll: Default::default(),
            block: Default::default(),
            border_style: Default::default(),
            title_style: Default::default(),
            focus: Default::default(),
            select: Default::default(),
            invalid: Default::default(),
            on_focus_gained: Default::default(),
            on_focus_lost: Default::default(),
            on_tab: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

pub mod core {
    //!
    //! Core structs for text-editing.
    //! Used to implement the widgets.
    //!

    pub use crate::text_core::TextCore;
    pub use crate::text_core::core_op;
    pub use crate::text_store::SkipLine;
    pub use crate::text_store::TextStore;
    pub use crate::text_store::text_rope::TextRope;
    pub use crate::text_store::text_string::TextString;
}

#[derive(Debug, PartialEq)]
pub enum TextError {
    /// Invalid text.
    InvalidText(String),
    /// Clipboard error occurred.
    Clipboard,
    /// Indicates that the passed text-range was out of bounds.
    TextRangeOutOfBounds(TextRange),
    /// Indicates that the passed text-position was out of bounds.
    TextPositionOutOfBounds(TextPosition),
    /// Indicates that the passed line index was out of bounds.
    ///
    /// Contains the index attempted and the actual length of the
    /// `Rope`/`RopeSlice` in lines, in that order.
    LineIndexOutOfBounds(upos_type, upos_type),
    /// Column index is out of bounds.
    ColumnIndexOutOfBounds(upos_type, upos_type),
    /// Indicates that the passed byte index was out of bounds.
    ///
    /// Contains the index attempted and the actual length of the
    /// `Rope`/`RopeSlice` in bytes, in that order.
    ByteIndexOutOfBounds(usize, usize),
    /// Indicates that the passed char index was out of bounds.
    ///
    /// Contains the index attempted and the actual length of the
    /// `Rope`/`RopeSlice` in chars, in that order.
    CharIndexOutOfBounds(usize, usize),
    /// out of bounds.
    ///
    /// Contains the [start, end) byte indices of the range and the actual
    /// length of the `Rope`/`RopeSlice` in bytes, in that order.  When
    /// either the start or end are `None`, that indicates a half-open range.
    ByteRangeOutOfBounds(Option<usize>, Option<usize>, usize),
    /// Indicates that the passed char-index range was partially or fully
    /// out of bounds.
    ///
    /// Contains the [start, end) char indices of the range and the actual
    /// length of the `Rope`/`RopeSlice` in chars, in that order.  When
    /// either the start or end are `None`, that indicates a half-open range.
    CharRangeOutOfBounds(Option<usize>, Option<usize>, usize),
    /// Indicates that the passed byte index was not a char boundary.
    ///
    /// Contains the passed byte index.
    ByteIndexNotCharBoundary(usize),
    /// Indicates that the passed byte range didn't line up with char
    /// boundaries.
    ///
    /// Contains the [start, end) byte indices of the range, in that order.
    /// When either the start or end are `None`, that indicates a half-open
    /// range.
    ByteRangeNotCharBoundary(
        Option<usize>, // Start.
        Option<usize>, // End.
    ),
    /// Indicates that a reversed byte-index range (end < start) was
    /// encountered.
    ///
    /// Contains the [start, end) byte indices of the range, in that order.
    ByteRangeInvalid(
        usize, // Start.
        usize, // End.
    ),
    /// Indicates that a reversed char-index range (end < start) was
    /// encountered.
    ///
    /// Contains the [start, end) char indices of the range, in that order.
    CharRangeInvalid(
        usize, // Start.
        usize, // End.
    ),
    /// Invalid regex for search.
    InvalidSearch,
    /// Invalid format string.
    InvalidFmt,
    /// Text value is somehow invalid.
    InvalidValue,
}

impl Display for TextError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<std::fmt::Error> for TextError {
    fn from(_: std::fmt::Error) -> Self {
        TextError::InvalidFmt
    }
}

impl Error for TextError {}

/// Row/Column type.
#[allow(non_camel_case_types)]
pub type upos_type = u32;
/// Row/Column type.
#[allow(non_camel_case_types)]
pub type ipos_type = i32;

/// Text position.
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct TextPosition {
    pub y: upos_type,
    pub x: upos_type,
}

impl TextPosition {
    /// New position.
    pub const fn new(x: upos_type, y: upos_type) -> TextPosition {
        Self { y, x }
    }
}

impl Debug for TextPosition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{}", self.x, self.y)
    }
}

impl From<(upos_type, upos_type)> for TextPosition {
    fn from(value: (upos_type, upos_type)) -> Self {
        Self {
            y: value.1,
            x: value.0,
        }
    }
}

impl From<TextPosition> for (upos_type, upos_type) {
    fn from(value: TextPosition) -> Self {
        (value.x, value.y)
    }
}

// TODO: replace with standard Range.
/// Exclusive range for text ranges.
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct TextRange {
    /// column, row
    pub start: TextPosition,
    /// column, row
    pub end: TextPosition,
}

impl Debug for TextRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}|{}-{}|{}",
            self.start.x, self.start.y, self.end.x, self.end.y
        )
    }
}

impl From<Range<TextPosition>> for TextRange {
    fn from(value: Range<TextPosition>) -> Self {
        assert!(value.start <= value.end);
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl From<Range<(upos_type, upos_type)>> for TextRange {
    fn from(value: Range<(upos_type, upos_type)>) -> Self {
        Self {
            start: TextPosition::from(value.start),
            end: TextPosition::from(value.end),
        }
    }
}

impl From<TextRange> for Range<TextPosition> {
    fn from(value: TextRange) -> Self {
        value.start..value.end
    }
}

impl TextRange {
    /// Maximum text range.
    pub const MAX: TextRange = TextRange {
        start: TextPosition {
            y: upos_type::MAX,
            x: upos_type::MAX,
        },
        end: TextPosition {
            y: upos_type::MAX,
            x: upos_type::MAX,
        },
    };

    /// New text range.
    ///
    /// Panic
    /// Panics if start > end.
    pub fn new(start: impl Into<TextPosition>, end: impl Into<TextPosition>) -> Self {
        let start = start.into();
        let end = end.into();

        assert!(start <= end);

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

/// Trait for a cursor (akin to an Iterator, not the blinking thing).
///
/// This is not a [DoubleEndedIterator] which can iterate from both ends of
/// the iterator, but moves a cursor forward/back over the collection.
pub trait Cursor: Iterator {
    /// Return the previous item.
    fn prev(&mut self) -> Option<Self::Item>;

    /// Peek next.
    fn peek_next(&mut self) -> Option<Self::Item> {
        let v = self.next();
        self.prev();
        v
    }

    /// Peek prev.
    fn peek_prev(&mut self) -> Option<Self::Item> {
        let v = self.prev();
        self.next();
        v
    }

    /// Offset of the current cursor position into the underlying text.
    fn text_offset(&self) -> usize;
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
