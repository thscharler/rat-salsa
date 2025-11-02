use crate::core::core_op::*;
use crate::core::{TextStore, core_op};
use crate::text_area::TextAreaState;
use crate::{Cursor, TextError, TextPosition, TextRange, upos_type};
use regex_cursor::engines::dfa::{Regex, find_iter};
use regex_cursor::regex_automata::dfa::dense::BuildError;
use regex_cursor::{Input, RopeyCursor};
use std::cmp::min;

/// Insert a character at the cursor position.
/// Removes the selection and inserts the char.
/// This function has no special handling for '\n' or '\t'.
pub fn insert_char(state: &mut TextAreaState, c: char) -> bool {
    if state.has_selection()
        && state.auto_quote
        && (c == '\'' || c == '"' || c == '`' || c == '<' || c == '[' || c == '(' || c == '{')
    {
        let sel = state.selection();
        insert_quotes(&mut state.value, sel, c).expect("valid_selection");
        state.scroll_cursor_to_visible();
        return true;
    }

    state
        .value
        .remove_str_range(state.selection())
        .expect("valid_selection");

    let pos = state.cursor();

    // insert missing newline
    if pos.x == 0
        && pos.y != 0
        && (pos.y == state.len_lines() || pos.y == state.len_lines().saturating_sub(1))
        && !state.value.text().has_final_newline()
    {
        let anchor = state.value.anchor();
        let cursor = state.value.cursor();
        state
            .value
            .insert_str(pos, &state.newline)
            .expect("valid_cursor");
        state.value.set_selection(anchor, cursor);
    }

    state.value.insert_char(pos, c).expect("valid_cursor");
    state.scroll_cursor_to_visible();

    true
}

/// Inserts tab at the current position. This respects the
/// tab-width set.
///
/// If there is a text-selection the text-rows will be indented instead.
/// This can be deactivated with auto_indent=false.
pub fn insert_tab(state: &mut TextAreaState) -> bool {
    if state.has_selection() {
        if state.auto_indent {
            indent(state, state.tab_width);
            true
        } else {
            false
        }
    } else {
        let pos = state.cursor();
        core_op::insert_tab(&mut state.value, pos, state.expand_tabs, state.tab_width)
            .expect("valid_cursor");
        state.scroll_cursor_to_visible();

        true
    }
}

/// Dedent the selected text by tab-width. If there is no
/// selection this does nothing.
///
/// This can be deactivated with auto_indent=false.
pub fn insert_backtab(state: &mut TextAreaState) -> bool {
    if state.has_selection() {
        dedent(state, state.tab_width);
        true
    } else {
        false
    }
}

/// Insert text at the cursor position.
/// Removes the selection and inserts the text.
pub fn insert_str(state: &mut TextAreaState, t: &str) -> bool {
    if state.has_selection() {
        state
            .value
            .remove_str_range(state.selection())
            .expect("valid_selection");
    }
    state
        .value
        .insert_str(state.cursor(), t)
        .expect("valid_cursor");
    state.scroll_cursor_to_visible();

    true
}

/// Insert a line break at the cursor position.
///
/// If auto_indent is set the new line starts with the same
/// indent as the current.
pub fn insert_newline(state: &mut TextAreaState) -> bool {
    if state.has_selection() {
        state
            .value
            .remove_str_range(state.selection())
            .expect("valid_selection");
    }

    state
        .value
        .insert_str(state.cursor(), &state.newline)
        .expect("valid_cursor");
    // insert leading spaces
    if state.auto_indent {
        auto_indent(state);
    }
    state.scroll_cursor_to_visible();

    true
}

/// Deletes the given range.
pub fn delete_range(state: &mut TextAreaState, range: TextRange) -> Result<bool, TextError> {
    if !range.is_empty() {
        state.value.remove_str_range(range)?;
        state.scroll_cursor_to_visible();
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Indent the selection by shift-width.
pub fn indent(state: &mut TextAreaState, shift_width: u32) {
    let sel = state.selection();
    let indent = " ".repeat(shift_width as usize);

    state.begin_undo_seq();
    for r in sel.start.y..=sel.end.y {
        state
            .value
            .insert_str(TextPosition::new(0, r), &indent)
            .expect("valid_row");
    }
    state.end_undo_seq();
}

/// Dedent the selection by shift-width.
pub fn dedent(state: &mut TextAreaState, shift_width: u32) {
    let sel = state.selection();

    state.begin_undo_seq();
    for r in sel.start.y..=sel.end.y {
        let mut idx = 0;
        let g_it = state
            .graphemes(TextRange::new((0, r), (0, r + 1)), TextPosition::new(0, r))
            .take(shift_width as usize);
        for g in g_it {
            if g != " " && g != "\t" {
                break;
            }
            idx += 1;
        }

        state
            .value
            .remove_str_range(TextRange::new((0, r), (idx, r)))
            .expect("valid_range");
    }
    state.end_undo_seq();
}

/// Fill in the auto indent at the current position.
/// Cursor should be at column 0, otherwise this will make no sense.
pub fn auto_indent(state: &mut TextAreaState) {
    let cursor = state.cursor();
    if cursor.y > 0 {
        let mut blanks = String::new();
        for g in state.line_graphemes(cursor.y - 1) {
            if g == " " || g == "\t" {
                blanks.push_str(g.grapheme());
            } else {
                break;
            }
        }
        if !blanks.is_empty() {
            state
                .value
                .insert_str(cursor, &blanks)
                .expect("valid_cursor");
        }
    }
}

/// Duplicate selection or current line.
pub fn duplicate_text(state: &mut TextAreaState) -> bool {
    if state.has_selection() {
        let sel_range = state.selection();
        if !sel_range.is_empty() {
            let v = state.str_slice(sel_range).to_string();
            state
                .value
                .insert_str(sel_range.end, &v)
                .expect("valid_selection");
        }
    } else {
        let pos = state.cursor();
        let row_range = TextRange::new((0, pos.y), (0, pos.y + 1));
        let v = state.str_slice(row_range).to_string();
        state
            .value
            .insert_str(row_range.start, &v)
            .expect("valid_cursor");
    }
    true
}

/// Deletes the current line.
/// Returns true if there was any real change.
pub fn delete_line(state: &mut TextAreaState) -> bool {
    let pos = state.cursor();
    if pos.y + 1 < state.len_lines() {
        state
            .value
            .remove_str_range(TextRange::new((0, pos.y), (0, pos.y + 1)))
            .expect("valid_range")
    } else {
        let width = state.line_width(pos.y);
        state
            .value
            .remove_str_range(TextRange::new((0, pos.y), (width, pos.y)))
            .expect("valid_range")
    }
}

/// Deletes the next char or the current selection.
/// Returns true if there was any real change.
pub fn delete_next_char(state: &mut TextAreaState) -> bool {
    state.scroll_cursor_to_visible();

    if state.has_selection() {
        state
            .value
            .remove_str_range(state.selection())
            .expect("valid_selection")
    } else {
        let pos = state.cursor();
        remove_next_char(&mut state.value, pos).expect("valid_cursor")
    }
}

/// Deletes the previous char or the selection.
/// Returns true if there was any real change.
pub fn delete_prev_char(state: &mut TextAreaState) -> bool {
    state.scroll_cursor_to_visible();

    if state.has_selection() {
        state
            .value
            .remove_str_range(state.selection())
            .expect("valid_selection")
    } else {
        let pos = state.cursor();
        remove_prev_char(&mut state.value, pos).expect("valid_cursor")
    }
}

/// Delete the next word. This alternates deleting the whitespace between words and
/// the words themselves.
///
/// If there is a selection, removes only the selected text.
pub fn delete_next_word(state: &mut TextAreaState) -> bool {
    if state.has_selection() {
        state
            .value
            .remove_str_range(state.selection())
            .expect("valid_selection")
    } else {
        let cursor = state.cursor();

        let start = next_word_start(&state.value, cursor).expect("valid_cursor");
        if start > cursor {
            state
                .value
                .remove_str_range(TextRange::from(cursor..start))
                .expect("valid_range")
        } else {
            let end = next_word_end(&state.value, cursor).expect("valid_cursor");
            if end > cursor {
                state
                    .value
                    .remove_str_range(TextRange::from(cursor..end))
                    .expect("valid_range")
            } else {
                false
            }
        }
    }
}

/// Deletes the previous word. This alternates deleting the whitespace
/// between words and the words themselves.
///
/// If there is a selection, removes only the selected text.
pub fn delete_prev_word(state: &mut TextAreaState) -> bool {
    if state.has_selection() {
        state
            .value
            .remove_str_range(state.selection())
            .expect("valid_selection")
    } else {
        let cursor = state.cursor();

        // delete to beginning of line?
        let till_line_start = if cursor.x != 0 {
            let line_start = TextPosition::new(0, cursor.y);
            let mut it = state.graphemes(TextRange::from(line_start..cursor), cursor);

            let mut is_whitespace = true;
            loop {
                let Some(prev) = it.prev() else {
                    break;
                };
                if !prev.is_whitespace() {
                    is_whitespace = false;
                    break;
                }
            }

            is_whitespace
        } else {
            false
        };

        if till_line_start {
            state
                .value
                .remove_str_range(TextRange::new((0, cursor.y), cursor))
                .expect("valid_range")
        } else {
            let end = prev_word_end(&state.value, cursor).expect("valid_cursor");
            if end != cursor {
                state
                    .value
                    .remove_str_range(TextRange::from(end..cursor))
                    .expect("valid_range")
            } else {
                let start = prev_word_start(&state.value, cursor).expect("valid_cursor");
                state
                    .value
                    .remove_str_range(TextRange::from(start..cursor))
                    .expect("valid_range")
            }
        }
    }
}

/// Move the cursor left. Scrolls the cursor to visible.
/// Returns true if there was any real change.
pub fn move_left(state: &mut TextAreaState, n: u16, extend_selection: bool) -> bool {
    let mut cursor = state.cursor();
    if cursor.x == 0 {
        if cursor.y > 0 {
            cursor.y = cursor.y.saturating_sub(1);
            cursor.x = state.line_width(cursor.y);
        }
    } else {
        cursor.x = cursor.x.saturating_sub(n as upos_type);
    }

    if let Some(scr_cursor) = state.pos_to_relative_screen(cursor) {
        state.set_move_col(Some(scr_cursor.0));
    }

    state.set_cursor(cursor, extend_selection)
}

/// Move the cursor right. Scrolls the cursor to visible.
/// Returns true if there was any real change.
pub fn move_right(state: &mut TextAreaState, n: u16, extend_selection: bool) -> bool {
    let mut cursor = state.cursor();
    let c_line_width = state.line_width(cursor.y);
    if cursor.x == c_line_width {
        if cursor.y + 1 < state.len_lines() {
            cursor.y += 1;
            cursor.x = 0;
        }
    } else {
        cursor.x = min(cursor.x + n as upos_type, c_line_width)
    }

    if let Some(scr_cursor) = state.pos_to_relative_screen(cursor) {
        state.set_move_col(Some(scr_cursor.0));
    }
    state.set_cursor(cursor, extend_selection)
}

/// Move the cursor up. Scrolls the cursor to visible.
/// Returns true if there was any real change.
pub fn move_up(state: &mut TextAreaState, n: u16, extend_selection: bool) -> bool {
    let cursor = state.cursor();
    if let Some(mut scr_cursor) = state.pos_to_relative_screen(cursor) {
        if let Some(move_col) = state.move_col() {
            scr_cursor.0 = move_col;
        }
        scr_cursor.1 -= n as i16;

        if let Some(new_cursor) = state.relative_screen_to_pos(scr_cursor) {
            state.set_cursor(new_cursor, extend_selection)
        } else {
            state.scroll_cursor_to_visible();
            true
        }
    } else {
        state.scroll_cursor_to_visible();
        true
    }
}

/// Move the cursor down. Scrolls the cursor to visible.
/// Returns true if there was any real change.
pub fn move_down(state: &mut TextAreaState, n: u16, extend_selection: bool) -> bool {
    let cursor = state.cursor();
    if let Some(mut scr_cursor) = state.pos_to_relative_screen(cursor) {
        if let Some(move_col) = state.move_col() {
            scr_cursor.0 = move_col;
        }
        scr_cursor.1 += n as i16;

        if let Some(new_cursor) = state.relative_screen_to_pos(scr_cursor) {
            state.set_cursor(new_cursor, extend_selection)
        } else {
            state.scroll_cursor_to_visible();
            true
        }
    } else {
        state.scroll_cursor_to_visible();
        true
    }
}

/// Move the cursor to the start of the line.
/// Scrolls the cursor to visible.
/// Returns true if there was any real change.
pub fn move_to_line_start(state: &mut TextAreaState, extend_selection: bool) -> bool {
    let cursor = state.cursor();

    let mut line_start = state.pos_to_line_start(cursor);
    for g in state
        .glyphs2(
            0,
            line_start.x,
            line_start.y..min(line_start.y + 1, state.len_lines()),
        )
        .expect("valid-pos")
    {
        if g.glyph() != " " && g.glyph() != "\t" {
            if g.pos().x != cursor.x {
                line_start.x = g.pos().x;
            }
            break;
        }
    }

    if let Some(scr_pos) = state.pos_to_relative_screen(line_start) {
        state.set_move_col(Some(scr_pos.0));
    }
    state.set_cursor(line_start, extend_selection)
}

/// Move the cursor to the end of the line. Scrolls to visible, if
/// necessary.
/// Returns true if there was any real change.
pub fn move_to_line_end(state: &mut TextAreaState, extend_selection: bool) -> bool {
    let cursor = state.cursor();
    let line_end = state.pos_to_line_end(cursor);
    if let Some(scr_pos) = state.pos_to_relative_screen(line_end) {
        state.set_move_col(Some(scr_pos.0));
    }
    state.set_cursor(line_end, extend_selection)
}

/// Move the cursor to the document start.
pub fn move_to_start(state: &mut TextAreaState, extend_selection: bool) -> bool {
    let cursor = TextPosition::new(0, 0);

    state.set_move_col(Some(0));
    state.set_cursor(cursor, extend_selection)
}

/// Move the cursor to the document end.
pub fn move_to_end(state: &mut TextAreaState, extend_selection: bool) -> bool {
    let cursor = TextPosition::new(
        state.line_width(state.len_lines().saturating_sub(1)),
        state.len_lines().saturating_sub(1),
    );

    let line_start = state.pos_to_line_start(cursor);
    state.set_move_col(Some(0));
    state.set_cursor(line_start, extend_selection)
}

/// Move the cursor to the start of the visible area.
pub fn move_to_screen_start(state: &mut TextAreaState, extend_selection: bool) -> bool {
    let (ox, oy) = state.offset();

    let cursor = TextPosition::new(ox as upos_type, oy as upos_type);

    state.set_move_col(Some(0));
    state.set_cursor(cursor, extend_selection)
}

/// Move the cursor to the end of the visible area.
pub fn move_to_screen_end(state: &mut TextAreaState, extend_selection: bool) -> bool {
    let scr_end = (0, (state.inner.height as i16).saturating_sub(1));
    if let Some(pos) = state.relative_screen_to_pos(scr_end) {
        state.set_move_col(Some(0));
        state.set_cursor(pos, extend_selection)
    } else {
        state.scroll_cursor_to_visible();
        true
    }
}

/// Move the cursor to the next word.
pub fn move_to_next_word(state: &mut TextAreaState, extend_selection: bool) -> bool {
    let cursor = state.cursor();

    let word = state.next_word_end(cursor);

    if let Some(scr_pos) = state.pos_to_relative_screen(word) {
        state.set_move_col(Some(scr_pos.0));
    }
    state.set_cursor(word, extend_selection)
}

/// Move the cursor to the previous word.
pub fn move_to_prev_word(state: &mut TextAreaState, extend_selection: bool) -> bool {
    let cursor = state.cursor();

    let word = state.prev_word_start(cursor);

    if let Some(scr_pos) = state.pos_to_relative_screen(word) {
        state.set_move_col(Some(scr_pos.0));
    }
    state.set_cursor(word, extend_selection)
}

/// Clear the search.
pub fn clear_search(state: &mut TextAreaState, match_style: usize) {
    state.remove_style_fully(match_style);
}

/// Search the term and highlight the finds as style `style`.
#[allow(clippy::result_large_err)]
pub fn search(
    state: &mut TextAreaState,
    search: &str,
    match_style: usize,
) -> Result<bool, BuildError> {
    clear_search(state, match_style);

    if search.is_empty() {
        return Ok(false);
    }

    let re = Regex::new(search)?;

    let cursor = RopeyCursor::new(state.rope().byte_slice(..));
    let input = Input::new(cursor);
    let mut matches = Vec::new();
    for m in find_iter(&re, input) {
        matches.push(m.start()..m.end());
    }
    let found = !matches.is_empty();

    for r in matches {
        state.add_style(r, match_style);
    }

    Ok(found)
}

pub fn move_to_next_match(state: &mut TextAreaState, match_style: usize) -> bool {
    let pos = state.cursor();
    let pos = state.byte_at(pos);

    let find = if let Some(mut styles) = state.styles() {
        let find = styles.find(|(range, style)| {
            if *style == match_style {
                range.start > pos.start
            } else {
                false
            }
        });

        if let Some((find_range, _)) = find {
            Some(state.byte_range(find_range))
        } else {
            None
        }
    } else {
        None
    };

    if let Some(find) = find {
        state.set_cursor(find.end, false);
        state.set_cursor(find.start, true);
        true
    } else {
        false
    }
}

pub fn move_to_prev_match(state: &mut TextAreaState, match_style: usize) -> bool {
    let pos = state.cursor();
    let pos = state.byte_at(pos);

    let find = if let Some(styles) = state.styles() {
        let find = styles
            .filter(|(range, style)| {
                if *style == match_style {
                    range.start < pos.start
                } else {
                    false
                }
            })
            .last();

        if let Some((find_range, _)) = find {
            Some(state.byte_range(find_range))
        } else {
            None
        }
    } else {
        None
    };

    if let Some(find) = find {
        state.set_cursor(find.end, false);
        state.set_cursor(find.start, true);
        true
    } else {
        false
    }
}
