use rat_widget::text::upos_type;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

// parse a single list item into marker and text.
#[derive(Debug)]
pub struct MDItem<'a> {
    pub prefix: &'a str,
    pub mark_bytes: Range<usize>,
    pub mark: &'a str,
    pub mark_suffix: &'a str,
    pub mark_nr: Option<usize>,
    pub text_prefix: &'a str,
    pub text_bytes: Range<usize>,
    pub text: &'a str,
}

pub fn parse_md_item(start: usize, txt: &str) -> MDItem<'_> {
    let mut mark_byte = 0;
    let mut mark_suffix_byte = 0;
    let mut text_prefix_byte = 0;
    let mut text_byte = 0;

    let mut mark_nr = None;

    #[derive(Debug, PartialEq)]
    enum It {
        Leading,
        OrderedMark,
        TextLeading,
    }

    let mut state = It::Leading;
    for (idx, c) in txt.bytes().enumerate() {
        if state == It::Leading {
            if c == b'+' || c == b'-' || c == b'*' {
                mark_byte = idx;
                mark_suffix_byte = idx + 1;
                text_prefix_byte = idx + 1;
                text_byte = idx + 1;
                state = It::TextLeading;
            } else if c.is_ascii_digit() {
                mark_byte = idx;
                state = It::OrderedMark;
            } else if c == b' ' || c == b'\t' {
                // ok
            } else {
                // broken??
                text_prefix_byte = idx;
                text_byte = idx;
                state = It::TextLeading;
            }
        } else if state == It::OrderedMark {
            if c.is_ascii_digit() {
                // ok
            } else if c == b'.' || c == b')' {
                mark_suffix_byte = idx;
                text_prefix_byte = idx + 1;
                text_byte = idx + 1;
                mark_nr = Some(
                    txt[mark_byte..mark_suffix_byte]
                        .parse::<usize>()
                        .expect("nr"),
                );
                state = It::TextLeading;
            } else {
                // broken??
                text_prefix_byte = idx;
                text_byte = idx;
                state = It::TextLeading;
            }
        } else if state == It::TextLeading {
            if c == b' ' || c == b'\t' {
                // ok
            } else {
                text_byte = idx;
                break;
            }
        }
    }

    MDItem {
        prefix: &txt[0..mark_byte],
        mark_bytes: start + mark_byte..start + text_prefix_byte,
        mark: &txt[mark_byte..mark_suffix_byte],
        mark_suffix: &txt[mark_suffix_byte..text_prefix_byte],
        mark_nr,
        text_prefix: &txt[text_prefix_byte..text_byte],
        text_bytes: start + text_byte..start + txt.len(),
        text: &txt[text_byte..],
    }
}

#[derive(Debug)]
pub struct MDCell<'a> {
    pub txt: &'a str,
    pub txt_graphemes: Range<upos_type>,
    pub txt_bytes: Range<usize>,
}

#[derive(Debug)]
pub struct MDRow<'a> {
    pub row: Vec<MDCell<'a>>,
    // cursor cell-nr
    pub cursor_cell: usize,
    // cursor grapheme offset into the cell
    pub cursor_offset: upos_type,
    // cursor byte offset into the cell
    pub cursor_byte_offset: usize,
}

// split single row. translate x-position to cell+cell_offset.
// __info__: returns the string before the first | and the string after the last | too!!
pub fn parse_md_row(txt: &str, x: upos_type) -> MDRow<'_> {
    let mut tmp = MDRow {
        row: Default::default(),
        cursor_cell: 0,
        cursor_offset: 0,
        cursor_byte_offset: 0,
    };

    let mut grapheme_start = 0;
    let mut grapheme_last = 0;
    let mut esc = false;
    let mut cell_offset = 0;
    let mut cell_byte_start = 0;
    for (idx, (byte_idx, c)) in txt.grapheme_indices(true).enumerate() {
        if idx == x as usize {
            tmp.cursor_cell = tmp.row.len();
            tmp.cursor_offset = cell_offset;
            tmp.cursor_byte_offset = byte_idx - cell_byte_start;
        }

        if c == "\\" {
            cell_offset += 1;
            esc = true;
        } else if c == "|" && !esc {
            cell_offset = 0;
            tmp.row.push(MDCell {
                txt: &txt[cell_byte_start..byte_idx],
                txt_graphemes: grapheme_start..idx as upos_type,
                txt_bytes: cell_byte_start..byte_idx,
            });
            cell_byte_start = byte_idx + 1;
            grapheme_start = idx as upos_type + 1;
        } else {
            cell_offset += 1;
            esc = false;
        }

        grapheme_last = idx as upos_type;
    }

    tmp.row.push(MDCell {
        txt: &txt[cell_byte_start..txt.len()],
        txt_graphemes: grapheme_start..grapheme_last,
        txt_bytes: cell_byte_start..txt.len(),
    });

    tmp
}

// parse quoted text
#[derive(Debug)]
pub struct MDBlockQuote2<'a> {
    pub quote: &'a str,
    pub text_prefix: &'a str,
    pub text_bytes: Range<usize>,
    pub text: &'a str,
}

pub fn parse_md_block_quote2(start: usize, txt: &str) -> MDBlockQuote2<'_> {
    let mut quote_byte = 0;
    let mut text_prefix_byte = 0;
    let mut text_byte = 0;

    #[derive(Debug, PartialEq)]
    enum It {
        Leading,
        TextLeading,
        Text,
        NewLine,
    }

    let mut state = It::Leading;
    for (idx, c) in txt.bytes().enumerate() {
        if state == It::Leading {
            if c == b'>' {
                quote_byte = idx;
                text_prefix_byte = idx + 1;
                state = It::TextLeading;
            } else if c == b' ' || c == b'\t' {
                // ok
            } else {
                text_prefix_byte = idx;
                text_byte = idx;
                state = It::Text;
            }
        } else if state == It::TextLeading {
            if c == b' ' || c == b'\t' {
                // ok
            } else {
                text_byte = idx;
                state = It::Text;
            }
        } else if state == It::Text {
            break;
        }
    }

    MDBlockQuote2 {
        quote: &txt[quote_byte..quote_byte + 1],
        text_prefix: &txt[text_prefix_byte..text_byte],
        text_bytes: start + text_byte..start + txt.len(),
        text: &txt[text_byte..txt.len()],
    }
}
