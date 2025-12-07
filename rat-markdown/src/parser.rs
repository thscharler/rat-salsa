//!
//! Special parsers for things not covered by pulldown-cmark.
//!
use rat_text::upos_type;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

/// Parsed header.
#[derive(Debug)]
pub struct MDHeader<'a> {
    pub header: u8,
    pub prefix: &'a str,
    pub tag: &'a str,
    pub text: &'a str,
    pub text_byte: Range<usize>,
}

/// Parse the text as header.
///
/// * relocate: Start offset of txt
pub fn parse_md_header(relocate: usize, txt: &str) -> Option<MDHeader<'_>> {
    let mut mark_prefix_end = 0;
    let mut mark_tag_start = 0;
    let mut mark_tag_end = 0;
    let mut mark_text_start = 0;

    #[derive(Debug, PartialEq)]
    enum It {
        Leading,
        Tag,
        LeadingText,
        Text,
        End,
        Fail,
    }

    let mut state = It::Leading;
    for (idx, c) in txt.bytes().enumerate() {
        if state == It::Leading {
            if c == b' ' || c == b'\t' {
                mark_prefix_end = idx + 1;
                mark_tag_start = idx + 1;
                mark_tag_end = idx + 1;
                mark_text_start = idx + 1;
            } else if c == b'#' {
                mark_prefix_end = idx;
                mark_tag_start = idx;
                mark_tag_end = idx + 1;
                mark_text_start = idx + 1;
                state = It::Tag;
            } else {
                state = It::Fail;
                break;
            }
        } else if state == It::Tag {
            if c == b'#' {
                mark_tag_end = idx;
                mark_text_start = idx + 1;
            } else {
                mark_tag_end = idx;
                mark_text_start = idx + 1;
                state = It::LeadingText;
            }
        } else if state == It::LeadingText {
            if c == b' ' || c == b'\t' {
                mark_text_start = idx + 1;
                // ok
            } else {
                mark_text_start = idx;
                state = It::Text;
            }
        } else if state == It::Text {
            state = It::End;
            break;
        }
    }

    if state == It::Fail {
        return None;
    }

    Some(MDHeader {
        header: (mark_tag_end - mark_tag_start) as u8,
        prefix: &txt[..mark_prefix_end],
        tag: &txt[mark_tag_start..mark_tag_end],
        text: &txt[mark_text_start..],
        text_byte: relocate + mark_text_start..relocate + txt.len(),
    })
}

/// Parsed link reference
#[derive(Debug)]
pub struct MDLinkRef<'a> {
    pub prefix: &'a str,
    pub tag: &'a str,
    pub link: &'a str,
    pub title: &'a str,
    pub suffix: &'a str,
}

/// Parse the text as link reference
///
/// * relocate - start offset of txt.
pub fn parse_md_link_ref(_relocate: usize, txt: &str) -> Option<MDLinkRef<'_>> {
    let mut mark_prefix_end = 0;
    let mut mark_tag_start = 0;
    let mut mark_tag_end = 0;
    let mut mark_link_start = 0;
    let mut mark_link_end = 0;
    let mut mark_title_start = 0;
    let mut mark_title_end = 0;

    #[derive(Debug, PartialEq)]
    enum It {
        Leading,
        Tag,
        AfterTag,
        LeadingLink,
        BracketLink,
        Link,
        LinkEsc,
        LeadingTitle,
        TitleSingle,
        TitleSingleEsc,
        TitleDouble,
        TitleDoubleEsc,
        End,
        Fail,
    }

    let mut state = It::Leading;
    for (idx, c) in txt.bytes().enumerate() {
        if state == It::Leading {
            if c == b'[' {
                mark_prefix_end = idx;
                mark_tag_start = idx + 1;
                mark_tag_end = idx + 1;
                mark_link_start = idx + 1;
                mark_link_end = idx + 1;
                mark_title_start = idx + 1;
                mark_title_end = idx + 1;
                state = It::Tag;
            } else if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' {
                mark_prefix_end = idx + 1;
                mark_tag_start = idx + 1;
                mark_tag_end = idx + 1;
                mark_link_start = idx + 1;
                mark_link_end = idx + 1;
                mark_title_start = idx + 1;
                mark_title_end = idx + 1;
            } else {
                state = It::Fail;
                break;
            }
        } else if state == It::Tag {
            if c == b']' {
                mark_tag_end = idx;
                mark_link_start = idx + 1;
                mark_link_end = idx + 1;
                mark_title_start = idx + 1;
                mark_title_end = idx + 1;
                state = It::AfterTag;
            } else {
                mark_tag_end = idx;
                mark_link_start = idx + 1;
                mark_link_end = idx + 1;
                mark_title_start = idx + 1;
                mark_title_end = idx + 1;
            }
        } else if state == It::AfterTag {
            if c == b':' {
                mark_link_start = idx + 1;
                mark_link_end = idx + 1;
                mark_title_start = idx + 1;
                mark_title_end = idx + 1;
                state = It::LeadingLink;
            } else {
                state = It::Fail;
                break;
            }
        } else if state == It::LeadingLink {
            if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' {
                mark_link_start = idx + 1;
                mark_link_end = idx + 1;
                mark_title_start = idx + 1;
                mark_title_end = idx + 1;
                // ok
            } else if c == b'<' {
                mark_link_start = idx + 1;
                mark_link_end = idx + 1;
                mark_title_start = idx + 1;
                mark_title_end = idx + 1;
                state = It::BracketLink;
            } else {
                mark_link_start = idx;
                mark_link_end = idx;
                mark_title_start = idx;
                mark_title_end = idx;
                state = It::Link;
            }
        } else if state == It::BracketLink {
            if c == b'>' {
                mark_link_end = idx;
                mark_title_start = idx + 1;
                mark_title_end = idx + 1;
                state = It::LeadingTitle;
            } else {
                mark_link_end = idx;
                mark_title_start = idx;
                mark_title_end = idx;
            }
        } else if state == It::Link {
            if c == b'\\' {
                mark_link_end = idx;
                mark_title_start = idx;
                mark_title_end = idx;
                state = It::LinkEsc;
            } else if c == b'\n' || c == b'\r' {
                mark_link_end = idx;
                mark_title_start = idx + 1;
                mark_title_end = idx + 1;
                state = It::LeadingTitle;
            } else if c == b'\'' {
                mark_link_end = idx;
                mark_title_start = idx + 1;
                mark_title_end = idx + 1;
                state = It::TitleSingle;
            } else if c == b'"' {
                mark_link_end = idx;
                mark_title_start = idx + 1;
                mark_title_end = idx + 1;
                state = It::TitleDouble;
            } else {
                mark_link_end = idx;
                mark_title_start = idx;
                mark_title_end = idx;
            }
        } else if state == It::LinkEsc {
            mark_link_end = idx;
            mark_title_start = idx;
            mark_title_end = idx;
            state = It::Link;
        } else if state == It::LeadingTitle {
            if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' {
                mark_title_start = idx + 1;
                mark_title_end = idx + 1;
            } else if c == b'\'' {
                mark_title_start = idx + 1;
                mark_title_end = idx + 1;
                state = It::TitleSingle;
            } else if c == b'"' {
                mark_title_start = idx + 1;
                mark_title_end = idx + 1;
                state = It::TitleDouble;
            } else {
                // no title, just suffix
                mark_title_start = idx;
                mark_title_end = idx;
                state = It::End;
                break;
            }
        } else if state == It::TitleSingle {
            if c == b'\'' {
                mark_title_end = idx;
                state = It::End;
                break;
            } else if c == b'\\' {
                mark_title_end = idx;
                state = It::TitleSingleEsc;
            } else {
                mark_title_end = idx;
            }
        } else if state == It::TitleSingleEsc {
            mark_title_end = idx;
            state = It::TitleSingle;
        } else if state == It::TitleDouble {
            if c == b'"' {
                mark_title_end = idx;
                state = It::End;
                break;
            } else if c == b'\\' {
                mark_title_end = idx;
                state = It::TitleDoubleEsc;
            } else {
                mark_title_end = idx;
            }
        } else if state == It::TitleDoubleEsc {
            mark_title_end = idx;
            state = It::TitleDouble;
        }
    }

    if state == It::Fail {
        return None;
    }

    Some(MDLinkRef {
        prefix: &txt[..mark_prefix_end],
        tag: &txt[mark_tag_start..mark_tag_end],
        link: &txt[mark_link_start..mark_link_end],
        title: &txt[mark_title_start..mark_title_end],
        suffix: &txt[mark_title_end..],
    })
}

/// One list item.
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

/// Parse a single list item into marker and text.
pub fn parse_md_item(relocate: usize, txt: &str) -> Option<MDItem<'_>> {
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
        Fail,
        End,
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
                state = It::Fail;
                break;
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
                state = It::Fail;
                break;
            }
        } else if state == It::TextLeading {
            if c == b' ' || c == b'\t' {
                // ok
            } else {
                text_byte = idx;
                state = It::End;
                break;
            }
        }
    }

    if state == It::Fail {
        return None;
    }

    Some(MDItem {
        prefix: &txt[0..mark_byte],
        mark_bytes: relocate + mark_byte..relocate + text_prefix_byte,
        mark: &txt[mark_byte..mark_suffix_byte],
        mark_suffix: &txt[mark_suffix_byte..text_prefix_byte],
        mark_nr,
        text_prefix: &txt[text_prefix_byte..text_byte],
        text_bytes: relocate + text_byte..relocate + txt.len(),
        text: &txt[text_byte..],
    })
}

/// One table cell.
#[derive(Debug)]
pub struct MDCell<'a> {
    pub txt: &'a str,
    pub txt_graphemes: Range<upos_type>,
    pub txt_bytes: Range<usize>,
}

/// One table row.
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

/// Split single row. Translate x-position to cell+cell_offset.
/// __info__: returns the string before the first | and the string after the last | too!!
pub fn parse_md_row(relocate: usize, txt: &str, x: upos_type) -> MDRow<'_> {
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
                txt_bytes: relocate + cell_byte_start..relocate + byte_idx,
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
        txt_bytes: relocate + cell_byte_start..relocate + txt.len(),
    });

    tmp
}

/// Quoted text
#[derive(Debug)]
pub struct MDBlockQuote<'a> {
    pub quote: &'a str,
    pub text_prefix: &'a str,
    pub text_bytes: Range<usize>,
    pub text: &'a str,
}

/// Parse a block-quote.
///
/// * relocate - offset of txt.
pub fn parse_md_block_quote(relocate: usize, txt: &str) -> Option<MDBlockQuote<'_>> {
    let mut quote_byte = 0;
    let mut text_prefix_byte = 0;
    let mut text_byte = 0;

    #[derive(Debug, PartialEq)]
    enum It {
        Leading,
        TextLeading,
        Text,
        End,
        Fail,
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
                state = It::Fail;
                break;
            }
        } else if state == It::TextLeading {
            if c == b' ' || c == b'\t' {
                // ok
            } else {
                text_byte = idx;
                state = It::Text;
            }
        } else if state == It::Text {
            state = It::End;
            break;
        }
    }

    if state == It::Fail {
        return None;
    }

    Some(MDBlockQuote {
        quote: &txt[quote_byte..quote_byte + 1],
        text_prefix: &txt[text_prefix_byte..text_byte],
        text_bytes: relocate + text_byte..relocate + txt.len(),
        text: &txt[text_byte..txt.len()],
    })
}
