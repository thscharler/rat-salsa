use rat_text::upos_type;
use unicode_segmentation::UnicodeSegmentation;

/// Length as grapheme count, excluding line breaks.
pub(crate) fn str_line_len(s: &str) -> upos_type {
    let it = s.graphemes(true);
    it.filter(|c| *c != "\n" && *c != "\r\n").count() as upos_type
}
