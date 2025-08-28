use crate::grapheme::Grapheme;
use crate::{upos_type, Cursor, TextError, TextPosition, TextRange};
use std::borrow::Cow;
use std::ops::Range;

pub(crate) mod text_rope;
pub(crate) mod text_string;

/// Extended Iterator that can skip over parts of
/// the underlying data.
pub trait SkipLine: Iterator {
    /// Set the iterator to the start of the next line.
    fn skip_line(&mut self) -> Result<(), TextError>;

    /// Set the iterator to this byte-position.
    ///
    /// This is a byte position for the underlying complete
    /// text, not an index into the iterated slice.
    /// Nevertheless, the byte_pos must not exceed the
    /// bounds of the slice.
    ///
    /// May panic if this is not a char boundary.
    /// May panic if the offset is not within the slice-bounds.
    fn skip_to(&mut self, byte_pos: usize) -> Result<(), TextError>;
}

/// Backing store for the TextCore.
//TODO: make pub(crate)
pub trait TextStore {
    type GraphemeIter<'a>: Cursor<Item = Grapheme<'a>> + SkipLine + Clone
    where
        Self: 'a;

    /// Can store multi-line content?
    fn is_multi_line(&self) -> bool;

    /// Minimum byte position that has been changed
    /// since the last call of min_changed().
    ///
    /// Can be used to invalidate caches.
    fn min_changed(&self) -> Option<usize>;

    /// Get content as string.
    fn string(&self) -> String;

    /// Set content from string.
    fn set_string(&mut self, t: &str);

    /// Grapheme position to byte position.
    /// This is the (start,end) position of the single grapheme after pos.
    ///
    /// * pos must be a valid position: row <= len_lines, col <= line_width of the row.
    fn byte_range_at(&self, pos: TextPosition) -> Result<Range<usize>, TextError>;

    /// Grapheme range to byte range.
    ///
    /// * range must be a valid range. row <= len_lines, col <= line_width of the row.
    fn byte_range(&self, range: TextRange) -> Result<Range<usize>, TextError>;

    /// Byte position to grapheme position.
    /// Returns the position that contains the given byte index.
    ///
    /// * byte must <= byte-len.
    fn byte_to_pos(&self, byte: usize) -> Result<TextPosition, TextError>;

    /// Byte range to grapheme range.
    ///
    /// * byte must <= byte-len.
    fn bytes_to_range(&self, bytes: Range<usize>) -> Result<TextRange, TextError>;

    /// A range of the text as `Cow<str>`.
    ///
    /// * range must be a valid range. row <= len_lines, col <= line_width of the row.
    /// * pos must be inside of range.
    fn str_slice(&self, range: TextRange) -> Result<Cow<'_, str>, TextError>;

    /// A range of the text as `Cow<str>`.
    ///
    /// * range must be valid
    fn str_slice_byte(&self, range: Range<usize>) -> Result<Cow<'_, str>, TextError>;

    /// Return a cursor over the graphemes of the range, start at the given position.
    ///
    /// * range must be a valid range. row <= len_lines, col <= line_width of the row.
    /// * pos must be inside of range.
    #[deprecated(since = "1.1.0", note = "replaced by grapheme_bytes")]
    fn graphemes(
        &self,
        range: TextRange,
        pos: TextPosition,
    ) -> Result<Self::GraphemeIter<'_>, TextError>;

    /// Return a cursor over the graphemes of the range, start at the given position.
    ///
    /// * range must be a valid byte-range.
    /// * pos must be inside of range.
    fn graphemes_byte(
        &self,
        range: Range<usize>,
        pos: usize,
    ) -> Result<Self::GraphemeIter<'_>, TextError>;

    /// Line as str.
    ///
    /// * row must be <= len_lines
    fn line_at(&self, row: upos_type) -> Result<Cow<'_, str>, TextError>;

    /// Iterate over text-lines, starting at line-offset.
    ///
    /// * row must be <= len_lines
    fn lines_at(&self, row: upos_type) -> Result<impl Iterator<Item = Cow<'_, str>>, TextError>;

    /// Return a line as an iterator over the graphemes.
    /// This contains the '\n' at the end.
    ///
    /// * row must be <= len_lines
    fn line_graphemes(&self, row: upos_type) -> Result<Self::GraphemeIter<'_>, TextError>;

    /// Line width of row as grapheme count.
    /// Excludes the terminating '\n'.
    ///
    /// * row must be <= len_lines
    fn line_width(&self, row: upos_type) -> Result<upos_type, TextError>;

    /// Does the last line end with a newline '\n'.
    fn has_final_newline(&self) -> bool;

    /// Number of lines.
    ///
    /// This counts the number of newline '\n' and adds one
    /// for the first row. And it adds one more if the last
    /// line doesn't end with a newline.
    ///
    /// `""` -> 1
    /// `"a"` -> 1
    /// `"a\n"` -> 2
    /// `"a\na"` -> 3
    /// `"a\na\n"` -> 3
    fn len_lines(&self) -> upos_type;

    /// Insert a char at the given position.
    ///
    /// * range must be a valid range. row <= len_lines, col <= line_width of the row.
    fn insert_char(
        &mut self,
        pos: TextPosition,
        c: char,
    ) -> Result<(TextRange, Range<usize>), TextError>;

    /// Insert a text str at the given position.
    ///
    /// * range must be a valid range. row <= len_lines, col <= line_width of the row.
    fn insert_str(
        &mut self,
        pos: TextPosition,
        t: &str,
    ) -> Result<(TextRange, Range<usize>), TextError>;

    /// Remove the given text range.
    ///
    /// * range must be a valid range. row <= len_lines, col <= line_width of the row.
    fn remove(
        &mut self,
        range: TextRange,
    ) -> Result<(String, (TextRange, Range<usize>)), TextError>;

    /// Insert a string at the given byte index.
    /// Call this only for undo.
    ///
    /// byte_pos must be <= len bytes.
    fn insert_b(&mut self, byte_pos: usize, t: &str) -> Result<(), TextError>;

    /// Remove the given byte-range.
    /// Call this only for undo.
    ///
    /// byte_pos must be <= len bytes.
    fn remove_b(&mut self, byte_range: Range<usize>) -> Result<(), TextError>;
}
