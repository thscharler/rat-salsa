use crate::mdedit_parts::parser::{parse_md_item, parse_md_row};
use crate::mdedit_parts::str_line_len;
use crate::mdedit_parts::styles::MDStyle;
use rat_widget::event::TextOutcome;
use rat_widget::text::{upos_type, TextRange};
use rat_widget::textarea::TextAreaState;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

pub fn md_tab(state: &mut TextAreaState) -> TextOutcome {
    if is_md_table(state) {
        let cursor = state.cursor();
        let row = state.line_at(cursor.y);
        let x = next_tab_md_row(row.as_ref(), cursor.x);
        state.set_cursor((x, cursor.y), false);
        state.set_move_col(Some(x));
        TextOutcome::TextChanged
    } else if is_md_item(state) {
        let cursor = state.cursor();

        let (item_byte, item_range) = md_item(state).expect("item");
        let indent_x = if item_range.start.y < cursor.y {
            let item_str = state.str_slice_byte(item_byte.clone());
            let item = parse_md_item(item_byte.start, item_str.as_ref());
            state.byte_pos(item.text_bytes.start).x
        } else if let Some((prev_byte, prev_range)) = md_prev_item(state) {
            let prev_str = state.str_slice_byte(prev_byte.clone());
            let prev_item = parse_md_item(prev_byte.start, prev_str.as_ref());
            state.byte_pos(prev_item.text_bytes.start).x
        } else {
            0
        };

        if cursor.x < indent_x {
            state
                .value
                .insert_str(cursor, &(" ".repeat((indent_x - cursor.x) as usize)))
                .expect("fine");
            TextOutcome::TextChanged
        } else {
            TextOutcome::Continue
        }
    } else {
        TextOutcome::Continue
    }
}

pub fn md_backtab(state: &mut TextAreaState) -> TextOutcome {
    if is_md_table(state) {
        let cursor = state.cursor();

        let row_str = state.line_at(cursor.y);
        let x = prev_tab_md_row(row_str.as_ref(), cursor.x);

        state.set_cursor((x, cursor.y), false);
        state.set_move_col(Some(x));
        TextOutcome::TextChanged
    } else {
        TextOutcome::Continue
    }
}

pub fn md_line_break(state: &mut TextAreaState) -> TextOutcome {
    let cursor = state.cursor();
    if is_md_table(state) {
        let line = state.line_at(cursor.y);
        if cursor.x == state.line_width(cursor.y) {
            let (x, row) = empty_md_row(line.as_ref(), state.newline());
            state.insert_str(row);
            state.set_cursor((x, cursor.y + 1), false);
            TextOutcome::TextChanged
        } else {
            let (x, row) = split_md_row(line.as_ref(), cursor.x, state.newline());
            state.begin_undo_seq();
            state.delete_range(TextRange::new((0, cursor.y), (0, cursor.y + 1)));
            state.insert_str(row);
            state.set_cursor((x, cursor.y + 1), false);
            state.end_undo_seq();
            TextOutcome::TextChanged
        }
    } else {
        let cursor = state.cursor();
        if cursor.x == state.line_width(cursor.y) {
            let (maybe_table, maybe_header) = is_md_maybe_table(state);
            if maybe_header {
                let line = state.line_at(cursor.y);
                let (x, row) = empty_md_row(line.as_ref(), state.newline());
                state.insert_str(row);
                state.set_cursor((x, cursor.y + 1), false);
                TextOutcome::TextChanged
            } else if maybe_table {
                let line = state.line_at(cursor.y);
                let (x, row) = create_md_title(line.as_ref(), state.newline());
                state.insert_str(row);
                state.set_cursor((x, cursor.y + 1), false);
                TextOutcome::TextChanged
            } else {
                TextOutcome::Continue
            }
        } else {
            TextOutcome::Continue
        }
    }
}

// duplicate as empty row
fn empty_md_row(txt: &str, newline: &str) -> (upos_type, String) {
    let row = parse_md_row(txt, 0);
    let mut new_row = String::new();
    new_row.push_str(newline);
    new_row.push('|');
    for idx in 1..row.row.len() - 1 {
        for g in row.row[idx].txt.graphemes(true) {
            new_row.push(' ');
        }
        new_row.push('|');
    }

    let x = if row.row.len() > 1 && row.row[1].txt.len() > 0 {
        str_line_len(row.row[0].txt) + 1 + 1
    } else {
        str_line_len(row.row[0].txt) + 1
    };

    (x, new_row)
}

// add a line break
fn split_md_row(txt: &str, cursor: upos_type, newline: &str) -> (upos_type, String) {
    let row = parse_md_row(txt, 0);

    let mut tmp0 = String::new();
    let mut tmp1 = String::new();
    let mut tmp_pos = 0;
    tmp0.push('|');
    tmp1.push('|');
    for row in &row.row[1..row.row.len() - 1] {
        if row.txt_graphemes.contains(&cursor) {
            tmp_pos = row.txt_graphemes.start + 1;

            let mut pos = row.txt_graphemes.start;
            if cursor > row.txt_graphemes.start {
                tmp1.push(' ');
            }
            for g in row.txt.graphemes(true) {
                if pos < cursor {
                    tmp0.push_str(g);
                } else {
                    tmp1.push_str(g);
                }
                pos += 1;
            }
            pos = row.txt_graphemes.start;
            for g in row.txt.graphemes(true) {
                if pos < cursor {
                    // omit one blank
                    if pos != row.txt_graphemes.start || cursor == row.txt_graphemes.start {
                        tmp1.push(' ');
                    }
                } else {
                    tmp0.push(' ');
                }
                pos += 1;
            }
        } else if row.txt_graphemes.start < cursor {
            tmp0.push_str(row.txt);
            tmp1.push_str(" ".repeat(row.txt_graphemes.len()).as_str());
        } else if row.txt_graphemes.start >= cursor {
            tmp0.push_str(" ".repeat(row.txt_graphemes.len()).as_str());
            tmp1.push_str(row.txt);
        }

        tmp0.push('|');
        tmp1.push('|');
    }
    tmp0.push_str(newline);
    tmp0.push_str(tmp1.as_str());
    tmp0.push_str(newline);

    (tmp_pos, tmp0)
}

// create underlines under the header
fn create_md_title(txt: &str, newline: &str) -> (upos_type, String) {
    let row = parse_md_row(txt, 0);

    let mut new_row = String::new();
    new_row.push_str(newline);
    new_row.push_str(row.row[0].txt);
    new_row.push('|');
    for idx in 1..row.row.len() - 1 {
        for g in row.row[idx].txt.graphemes(true) {
            new_row.push('-');
        }
        new_row.push('|');
    }

    let len = str_line_len(&new_row);

    (len, new_row)
}

fn is_md_table(state: &TextAreaState) -> bool {
    let cursor = state.cursor();
    let cursor_byte = state.byte_at(cursor).start;
    state
        .style_match(cursor_byte, MDStyle::Table as usize)
        .is_some()
}

fn is_md_maybe_table(state: &TextAreaState) -> (bool, bool) {
    let mut gr = state.line_graphemes(state.cursor().y);
    let (maybe_table, maybe_header) = if let Some(first) = gr.next() {
        if first == "|" {
            if let Some(second) = gr.next() {
                if second == "-" {
                    (true, true)
                } else {
                    (true, false)
                }
            } else {
                (true, false)
            }
        } else {
            (false, false)
        }
    } else {
        (false, false)
    };
    (maybe_table, maybe_header)
}

fn is_md_item(state: &TextAreaState) -> bool {
    let cursor = state.cursor();
    let cursor_byte = state.byte_at(cursor).start;
    state
        .style_match(cursor_byte, MDStyle::Item as usize)
        .is_some()
}

fn next_tab_md_row(txt: &str, pos: upos_type) -> upos_type {
    let row = parse_md_row(txt, pos);
    if row.cursor_cell + 1 < row.row.len() {
        row.row[row.cursor_cell + 1].txt_graphemes.start
    } else {
        pos
    }
}

fn prev_tab_md_row(txt: &str, pos: upos_type) -> upos_type {
    let row = parse_md_row(txt, pos);
    if row.cursor_cell > 0 {
        row.row[row.cursor_cell - 1].txt_graphemes.start
    } else {
        pos
    }
}

fn md_item(state: &TextAreaState) -> Option<(Range<usize>, TextRange)> {
    let cursor = state.cursor();
    let cursor_byte = state.byte_at(cursor).start;

    let item_byte = state.style_match(cursor_byte, MDStyle::Item as usize);

    if let Some(item_byte) = item_byte {
        Some((item_byte.clone(), state.byte_range(item_byte)))
    } else {
        None
    }
}

fn md_prev_item(state: &TextAreaState) -> Option<(Range<usize>, TextRange)> {
    let cursor = state.cursor();
    let cursor_byte = state.byte_at(cursor).start;

    let item_byte = state.style_match(cursor_byte, MDStyle::Item as usize);
    let list_byte = state.style_match(cursor_byte, MDStyle::List as usize);

    if let Some(list_byte) = list_byte {
        if let Some(item_byte) = item_byte {
            let mut sty = Vec::new();
            state.styles_in(list_byte.start..item_byte.start, &mut sty);

            let prev = sty.iter().filter(|v| v.1 == MDStyle::Item as usize).last();

            if let Some((prev_bytes, _)) = prev {
                let prev = state.byte_range(prev_bytes.clone());
                Some((prev_bytes.clone(), prev))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn md_next_item(state: &TextAreaState) -> Option<(Range<usize>, TextRange)> {
    let cursor = state.cursor();
    let cursor_byte = state.byte_at(cursor).start;

    let item_byte = state.style_match(cursor_byte, MDStyle::Item as usize);
    let list_byte = state.style_match(cursor_byte, MDStyle::List as usize);

    if let Some(list_byte) = list_byte {
        if let Some(item_byte) = item_byte {
            let mut sty = Vec::new();
            state.styles_in(item_byte.end..list_byte.end, &mut sty);

            let next = sty.iter().filter(|v| v.1 == MDStyle::Item as usize).next();

            if let Some((next_bytes, _)) = next {
                let next = state.byte_range(next_bytes.clone());
                Some((next_bytes.clone(), next))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}
