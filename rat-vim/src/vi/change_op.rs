use crate::vi::motion_op::{motion_end_position, motion_start_position, start_end_to_range};
use crate::vi::query::*;
use crate::vi::{Mark, Motion, SyncRanges};
use crate::{SearchError, VI};
use rat_text::text_area::TextAreaState;
use rat_text::{TextPosition, upos_type};
use std::mem;
use std::ops::Range;

pub fn prepend_line_str(mut mul: u32, state: &mut TextAreaState, vi: &mut VI) {
    if mul > 0 {
        vi.finds.sync = SyncRanges::FromTextArea;
        vi.matches.sync = SyncRanges::FromTextArea;
    }
    while mul > 0 {
        q_prepend_line_str(&vi.text, state);
        mul -= 1;
    }
}

pub fn append_line_str(mut mul: u32, state: &mut TextAreaState, vi: &mut VI) {
    if mul > 0 {
        vi.finds.sync = SyncRanges::FromTextArea;
        vi.matches.sync = SyncRanges::FromTextArea;
    }
    while mul > 0 {
        q_append_line_str(&vi.text, state);
        mul -= 1;
    }
}

pub fn append_str(mut mul: u32, state: &mut TextAreaState, vi: &mut VI) {
    if mul > 0 {
        vi.finds.sync = SyncRanges::FromTextArea;
        vi.matches.sync = SyncRanges::FromTextArea;
    }
    while mul > 0 {
        q_append_str(&vi.text, state);
        mul -= 1;
    }
}

pub fn insert_str(mut mul: u32, state: &mut TextAreaState, vi: &mut VI) {
    if mul > 0 {
        vi.finds.sync = SyncRanges::FromTextArea;
        vi.matches.sync = SyncRanges::FromTextArea;
    }
    while mul > 0 {
        q_insert_str(&vi.text, state);
        mul -= 1;
    }
}

pub fn insert_char(cc: char, state: &mut TextAreaState, vi: &mut VI) {
    vi.finds.sync = SyncRanges::FromTextArea;
    vi.matches.sync = SyncRanges::FromTextArea;
    vi.text.push(cc);
    q_insert(cc, state);
}

pub fn replace_text(
    mut mul: u32,
    cc: char,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Result<(), SearchError> {
    q_set_mark(Mark::ChangeStart, state.cursor(), vi);
    q_set_mark(Mark::ChangeEnd, state.cursor(), vi);

    if let Some(range) = change_range(mul, &Motion::Right, state, vi)? {
        vi.finds.sync = SyncRanges::FromTextArea;
        vi.matches.sync = SyncRanges::FromTextArea;
        state.begin_undo_seq();
        state.delete_range(range);
        while mul > 0 {
            state.insert_char(cc);
            mul -= 1;
        }
        state.end_undo_seq();
    }
    Ok(())
}

fn change_range(
    mul: u32,
    motion: &Motion,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Result<Option<Range<TextPosition>>, SearchError> {
    let start = motion_start_position(motion, state);
    let end = match motion {
        Motion::NextWordStart => Some(q_next_word_end(mul, state)),
        Motion::FullLine => Some(q_end_of_line(mul, state)),
        _ => motion_end_position(mul, motion, state, vi)?,
    };
    Ok(start_end_to_range(start, end))
}

pub fn change_text_str(
    mul: u32,
    motion: &Motion,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Result<(), SearchError> {
    if let Some(range) = change_range(mul, motion, state, vi)? {
        vi.finds.sync = SyncRanges::FromTextArea;
        vi.matches.sync = SyncRanges::FromTextArea;
        state.delete_range(range);
    }
    Ok(())
}

fn delete_range(
    mul: u32,
    motion: &Motion,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Result<Option<Range<TextPosition>>, SearchError> {
    let start = motion_start_position(motion, state);
    let end = match motion {
        Motion::NextWordStart => Some(q_next_word_end(mul, state)),
        _ => motion_end_position(mul, motion, state, vi)?,
    };
    Ok(start_end_to_range(start, end))
}

pub fn delete_text(
    mul: u32,
    motion: &Motion,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Result<(), SearchError> {
    q_set_mark(Mark::ChangeStart, state.cursor(), vi);
    q_set_mark(Mark::ChangeEnd, state.cursor(), vi);

    if let Some(range) = delete_range(mul, motion, state, vi)? {
        vi.finds.sync = SyncRanges::FromTextArea;
        vi.matches.sync = SyncRanges::FromTextArea;

        vi.yank.list.clear();
        vi.yank
            .list
            .push(state.str_slice(range.clone()).into_owned());
        state.delete_range(range);
    }
    Ok(())
}

fn yank_range(
    mul: u32,
    motion: &Motion,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Result<Option<Range<TextPosition>>, SearchError> {
    let start = motion_start_position(motion, state);
    let end = motion_end_position(mul, motion, state, vi)?;
    Ok(start_end_to_range(start, end))
}

pub fn yank_text(
    mul: u32,
    motion: &Motion,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Result<(), SearchError> {
    if let Some(range) = yank_range(mul, motion, state, vi)? {
        q_set_mark(Mark::ChangeStart, state.cursor(), vi);
        q_set_mark(Mark::ChangeEnd, state.cursor(), vi);

        vi.yank.list.clear();
        vi.yank.list.push(state.str_slice(range).into_owned());
    }
    Ok(())
}

pub fn copy_clipboard_text(
    mul: u32,
    motion: &Motion,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Result<(), SearchError> {
    if let Some(range) = yank_range(mul, motion, state, vi)? {
        if let Some(clip) = state.clipboard() {
            _ = clip.set_string(state.str_slice(range).as_ref());
        }
    }
    Ok(())
}

fn paste(text: &[String], mul: u32, before: bool, state: &mut TextAreaState, vi: &mut VI) {
    if text.len() > 1 {
        let cursor = state.cursor();
        let len_lines = state.len_lines();

        let start = if before {
            cursor
        } else {
            // TODO: mode for caret/block cursor??
            // let x = min(cursor.x + 1, state.line_width(cursor.y));
            // TextPosition::new(x, cursor.y)
            cursor
        };

        q_set_mark(Mark::ChangeStart, state.cursor(), vi);

        state.begin_undo_seq();
        state.set_cursor(start, false);
        for i in 0..text.len() {
            let y = start.y + i as upos_type;
            if y < len_lines {
                state.set_cursor((start.x, y), false);
                for _ in 0..mul {
                    state.insert_str(&text[i]);
                }
            }
        }
        state.set_cursor(cursor, false);
        state.end_undo_seq();

        q_set_mark(Mark::ChangeEnd, state.cursor(), vi);
    } else if text[0].contains('\n') {
        let nl = text[0].ends_with('\n');

        let start = if before {
            q_start_of_line(state)
        } else {
            q_start_of_next_line(1, state)
        };

        q_set_mark(Mark::ChangeStart, state.cursor(), vi);

        state.begin_undo_seq();
        state.set_cursor(start, false);
        for _ in 0..mul {
            state.insert_str(&text[0]);
            if nl {
                state.insert_newline();
            }
        }
        state.set_cursor(start, false);
        state.end_undo_seq();

        q_set_mark(Mark::ChangeEnd, state.cursor(), vi);
    } else {
        q_set_mark(Mark::ChangeStart, state.cursor(), vi);

        for _ in 0..mul {
            state.insert_str(&text[0]);
        }

        q_set_mark(Mark::ChangeEnd, state.cursor(), vi);
    }
}

pub fn paste_text(mul: u32, before: bool, state: &mut TextAreaState, vi: &mut VI) {
    let yanked = mem::take(&mut vi.yank.list);
    paste(&yanked, mul, before, state, vi);
    vi.yank.list = yanked;
}

pub fn paste_clipboard_text(mul: u32, before: bool, state: &mut TextAreaState, vi: &mut VI) {
    let Some(clip) = state.clipboard() else {
        return;
    };
    let mut text = [String::default(); 1];
    text[0] = clip.get_string().unwrap_or(String::default());
    paste(&text, mul, before, state, vi);
}

pub fn join_line(mut mul: u32, state: &mut TextAreaState, vi: &mut VI) {
    vi.finds.sync = SyncRanges::FromTextArea;
    vi.matches.sync = SyncRanges::FromTextArea;

    q_set_mark(Mark::ChangeStart, state.cursor(), vi);
    q_set_mark(Mark::ChangeEnd, state.cursor(), vi);

    while mul > 0 {
        let range = q_line_break_and_leading_space(state);
        if !range.is_empty() {
            state.set_selection(range.start, range.end);
            state.insert_char(' ');
        }

        mul -= 1;
    }
}

pub fn undo(mut mul: u32, state: &mut TextAreaState, vi: &mut VI) {
    vi.finds.sync = SyncRanges::FromTextArea;
    vi.matches.sync = SyncRanges::FromTextArea;

    while mul > 0 {
        if !state.undo() {
            break;
        }

        mul -= 1;
    }
}

pub fn redo(mut mul: u32, state: &mut TextAreaState, vi: &mut VI) {
    vi.finds.sync = SyncRanges::FromTextArea;
    vi.matches.sync = SyncRanges::FromTextArea;

    while mul > 0 {
        if !state.redo() {
            break;
        }

        mul -= 1;
    }
}
