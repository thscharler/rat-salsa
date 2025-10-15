use crate::vi::{Direction, Finds, Matches, SyncRanges, TxtObj};
use crate::{SearchError, VI, ctrl};
use log::debug;
use rat_text::text_area::TextAreaState;
use rat_text::{Cursor, Grapheme, TextPosition, TextRange, upos_type};
use regex_cursor::engines::dfa::{Regex, find_iter};
use regex_cursor::{Input, RopeyCursor};
use std::cmp::min;

pub fn q_move_left(mul: u32, state: &mut TextAreaState) -> Option<TextPosition> {
    let mut pos = state.cursor();
    if pos.x == 0 {
        if pos.y > 0 {
            pos.y = pos.y.saturating_sub(1);
            pos.x = state.line_width(pos.y);
        }
    } else {
        pos.x = pos.x.saturating_sub(mul as upos_type);
    }
    Some(pos)
}

pub fn q_move_right(mul: u32, state: &mut TextAreaState) -> Option<TextPosition> {
    let mut pos = state.cursor();
    let c_line_width = state.line_width(pos.y);
    if pos.x == c_line_width {
        if pos.y + 1 < state.len_lines() {
            pos.y += 1;
            pos.x = 0;
        }
    } else {
        pos.x = min(pos.x + mul as upos_type, c_line_width)
    }
    Some(pos)
}

pub fn q_move_up(mul: u32, state: &mut TextAreaState) -> Option<TextPosition> {
    let pos = state.cursor();
    if let Some(mut scr_cursor) = state.pos_to_relative_screen(pos) {
        scr_cursor.1 -= mul as i16;
        if let Some(npos) = state.relative_screen_to_pos(scr_cursor) {
            Some(npos)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn q_move_down(mul: u32, state: &mut TextAreaState) -> Option<TextPosition> {
    let pos = state.cursor();
    if let Some(mut scr_cursor) = state.pos_to_relative_screen(pos) {
        scr_cursor.1 += mul as i16;

        if let Some(npos) = state.relative_screen_to_pos(scr_cursor) {
            Some(npos)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn q_col(mul: u32, state: &TextAreaState) -> Option<TextPosition> {
    let c = state.cursor();
    if mul as upos_type <= state.line_width(c.y) {
        Some(TextPosition::new(mul as upos_type, c.y))
    } else {
        None
    }
}

pub fn q_line(mul: u32, state: &TextAreaState) -> Option<TextPosition> {
    let line = min(
        mul.saturating_sub(1) as upos_type,
        state.len_lines().saturating_sub(1),
    );
    Some(TextPosition::new(0, line))
}

pub fn q_line_percent(mul: u32, state: &TextAreaState) -> Option<TextPosition> {
    let len = state.len_lines() as u64;
    let pc = min(mul.saturating_sub(1), 100) as u64;
    let line = ((len * pc) / 100) as u32;
    Some(TextPosition::new(0, line))
}

pub fn q_matching_brace(state: &mut TextAreaState) -> Option<TextPosition> {
    let mut it = state.text_graphemes(state.cursor());

    let peek_prev = it.peek_prev().map(|v| v.into_parts().0);
    let peek_prev = peek_prev.as_ref().map(|v| v.as_ref());
    let peek_next = it.peek_next().map(|v| v.into_parts().0);
    let peek_next = peek_next.as_ref().map(|v| v.as_ref());

    let (cc, co) = match (peek_prev, peek_next) {
        (Some(")"), _) => ('(', ')'),
        (Some("}"), _) => ('{', '}'),
        (Some("]"), _) => ('[', ']'),
        (Some(">"), _) => ('<', '>'),
        (_, Some("(")) => (')', '('),
        (_, Some("{")) => ('}', '{'),
        (_, Some("[")) => (']', '['),
        (_, Some("<")) => ('>', '<'),
        (Some("("), _) => {
            it.prev();
            (')', '(')
        }
        (Some("{"), _) => {
            it.prev();
            ('}', '{')
        }
        (Some("["), _) => {
            it.prev();
            (']', '[')
        }
        (Some("<"), _) => {
            it.prev();
            ('>', '<')
        }
        (_, Some(")")) => {
            it.next();
            ('(', ')')
        }
        (_, Some("}")) => {
            it.next();
            ('{', '}')
        }
        (_, Some("]")) => {
            it.next();
            ('[', ']')
        }
        (_, Some(">")) => {
            it.next();
            ('<', '>')
        }
        (_, _) => return None,
    };

    if cc == '(' || cc == '{' || cc == '[' || cc == '<' {
        let mut n = 0;
        loop {
            let Some(c) = it.prev() else {
                break;
            };
            if c == cc {
                n -= 1;
            } else if c == co {
                n += 1;
            }
            if n == 0 {
                break;
            }
        }
    } else {
        let mut n = 0;
        loop {
            let Some(c) = it.next() else {
                break;
            };
            if c == cc {
                n -= 1;
            } else if c == co {
                n += 1;
            }
            if n == 0 {
                break;
            }
        }
    }
    Some(state.byte_pos(it.text_offset()))
}

pub fn q_mark_pos(mark: char, marks: &[Option<TextPosition>; 26]) -> Option<TextPosition> {
    if let Some(mark) = q_mark_idx(mark) {
        marks[mark]
    } else {
        None
    }
}

pub fn q_mark_idx(mark: char) -> Option<usize> {
    let mark = mark.to_ascii_lowercase();
    if mark >= 'a' && mark <= 'z' {
        Some(mark as usize - 'a' as usize)
    } else {
        None
    }
}

pub fn q_start_of_file() -> Option<TextPosition> {
    Some(TextPosition::new(0, 0))
}

pub fn q_end_of_file(state: &mut TextAreaState) -> Option<TextPosition> {
    let y = state.len_lines().saturating_sub(1);
    Some(TextPosition::new(state.line_width(y), y))
}

pub fn q_start_of_paragraph(to: TxtObj, state: &TextAreaState) -> TextPosition {
    let mut it = state.text_graphemes(state.cursor());

    if is_paragraph_whitespace(&mut it) {
        pskip_paragraph_whitespace(&mut it);
    } else {
        let mut paragraph = false;
        loop {
            let Some(c) = it.prev() else {
                break;
            };
            if track_paragraph_back(&c, &mut paragraph, &mut it) {
                break;
            }
        }
    }
    state.byte_pos(it.text_offset())
}

pub fn q_end_of_paragraph(mut mul: u32, to: TxtObj, state: &TextAreaState) -> TextPosition {
    let mut it = state.text_graphemes(state.cursor());

    loop {
        if mul == 0 {
            break;
        }

        if is_paragraph_whitespace(&mut it) {
            skip_paragraph_whitespace(&mut it);
        } else {
            let mut paragraph = false;
            loop {
                let Some(c) = it.next() else {
                    break;
                };
                if track_paragraph_fwd(&c, &mut paragraph, &mut it) {
                    if to == TxtObj::A {
                        skip_paragraph_whitespace(&mut it);
                    }
                    break;
                }
            }
        }
        mul -= 1;
    }
    state.byte_pos(it.text_offset())
}

pub fn q_start_of_sentence(to: TxtObj, state: &TextAreaState) -> TextPosition {
    let mut it = state.text_graphemes(state.cursor());

    if is_paragraph_whitespace(&mut it) {
        pskip_paragraph_whitespace(&mut it);
    } else {
        let rewind;
        let mut paragraph = false;
        loop {
            let Some(c) = it.prev() else {
                rewind = false;
                break;
            };

            if matches!(c.grapheme(), "." | "!" | "?") {
                rewind = true;
                break;
            }
            if track_paragraph_back(&c, &mut paragraph, &mut it) {
                rewind = false;
                break;
            }
        }
        if rewind {
            skip_nonwhite(&mut it);
            if to == TxtObj::A {
                skip_white(&mut it);
            }
        }
    }
    state.byte_pos(it.text_offset())
}

pub fn q_end_of_sentence(mut mul: u32, to: TxtObj, state: &TextAreaState) -> TextPosition {
    let mut it = state.text_graphemes(state.cursor());

    loop {
        if mul == 0 {
            break;
        }

        if is_paragraph_whitespace(&mut it) {
            skip_paragraph_whitespace(&mut it);
        } else {
            let forward;
            let mut paragraph = false;
            loop {
                let Some(c) = it.next() else {
                    forward = false;
                    break;
                };
                if matches!(c.grapheme(), "." | "!" | "?") {
                    forward = true;
                    break;
                }
                if track_paragraph_fwd(&c, &mut paragraph, &mut it) {
                    forward = false;
                    break;
                }
            }
            if forward {
                skip_nonwhite(&mut it);
                if to == TxtObj::A {
                    skip_white(&mut it);
                }
            }
        }
        mul -= 1;
    }
    state.byte_pos(it.text_offset())
}

pub fn q_start_of_word(to: TxtObj, state: &TextAreaState) -> TextPosition {
    let mut it = state.text_graphemes(state.cursor());

    let leading_whitespace;
    if to == TxtObj::A {
        loop {
            let Some(c) = it.next() else {
                leading_whitespace = true;
                break;
            };
            if is_whitespace(&c) {
                leading_whitespace = false;
                break;
            }
            if is_linebreak(&c) {
                leading_whitespace = true;
                break;
            }
        }
    } else {
        leading_whitespace = false;
    }

    let mut it = state.text_graphemes(state.cursor());

    let Some(sample) = it.peek_next() else {
        return state.byte_pos(it.text_offset());
    };
    if is_alphanumeric(&sample) {
        pskip_alpha(&mut it);
        if leading_whitespace {
            pskip_white(&mut it);
        }
    } else if is_whitespace(&sample) {
        pskip_white(&mut it);
    } else {
        pskip_sample(&mut it, sample);
        if leading_whitespace {
            pskip_white(&mut it);
        }
    }

    state.byte_pos(it.text_offset())
}

pub fn q_end_of_word(mut mul: u32, to: TxtObj, state: &TextAreaState) -> TextPosition {
    let mut it = state.text_graphemes(state.cursor());

    loop {
        if mul == 0 {
            break;
        }
        let Some(sample) = it.peek_next() else {
            return state.byte_pos(it.text_offset());
        };
        if is_alphanumeric(&sample) {
            skip_alpha(&mut it);
            if to == TxtObj::A {
                skip_white(&mut it);
            }
        } else if is_whitespace(&sample) {
            skip_white(&mut it);
        } else {
            skip_sample(&mut it, sample);
            if to == TxtObj::A {
                skip_white(&mut it);
            }
        }
        mul -= 1;
    }

    state.byte_pos(it.text_offset())
}

pub fn q_next_word_start(mut mul: u32, state: &TextAreaState) -> Option<TextPosition> {
    let mut it = state.text_graphemes(state.cursor());

    while mul > 0 {
        let Some(sample) = it.peek_next() else {
            return None;
        };
        if is_alphanumeric(&sample) {
            skip_alpha(&mut it);
        } else if is_whitespace(&sample) {
            // noop
        } else {
            skip_sample(&mut it, sample);
        }

        skip_white(&mut it);

        mul -= 1;
    }

    Some(state.byte_pos(it.text_offset()))
}

pub fn q_prev_word_start(mut mul: u32, state: &TextAreaState) -> Option<TextPosition> {
    let mut it = state.text_graphemes(state.cursor());

    while mul > 0 {
        let Some(sample) = it.peek_prev() else {
            return None;
        };
        if is_alphanumeric(&sample) {
            pskip_alpha(&mut it);
        } else if is_whitespace(&sample) {
            pskip_white(&mut it);
            let Some(sample) = it.peek_prev() else {
                return None;
            };
            if is_alphanumeric(&sample) {
                pskip_alpha(&mut it);
            } else {
                pskip_sample(&mut it, sample);
            }
        } else {
            pskip_sample(&mut it, sample);
        }

        mul -= 1;
    }

    Some(state.byte_pos(it.text_offset()))
}

pub fn q_next_word_end(mut mul: u32, state: &TextAreaState) -> Option<TextPosition> {
    let mut it = state.text_graphemes(state.cursor());

    while mul > 0 {
        skip_white(&mut it);

        let Some(sample) = it.peek_next() else {
            return None;
        };
        if is_alphanumeric(&sample) {
            skip_alpha(&mut it);
        } else {
            skip_sample(&mut it, sample);
        }

        mul -= 1;
    }

    Some(state.byte_pos(it.text_offset()))
}

pub fn q_prev_word_end(mut mul: u32, state: &TextAreaState) -> Option<TextPosition> {
    let mut it = state.text_graphemes(state.cursor());

    while mul > 0 {
        let Some(sample) = it.peek_prev() else {
            return None;
        };
        if is_alphanumeric(&sample) {
            pskip_alpha(&mut it);
        } else if is_whitespace(&sample) {
            // noop
        } else {
            pskip_sample(&mut it, sample);
        }

        pskip_white(&mut it);

        mul -= 1;
    }

    Some(state.byte_pos(it.text_offset()))
}

pub fn q_start_of_bigword(to: TxtObj, state: &TextAreaState) -> TextPosition {
    let mut it = state.text_graphemes(state.cursor());

    let leading_whitespace;
    if to == TxtObj::A {
        loop {
            let Some(c) = it.next() else {
                leading_whitespace = true;
                break;
            };
            if is_whitespace(&c) {
                leading_whitespace = false;
                break;
            }
            if is_linebreak(&c) {
                leading_whitespace = true;
                break;
            }
        }
    } else {
        leading_whitespace = false;
    }

    let mut it = state.text_graphemes(state.cursor());

    let Some(sample) = it.peek_next() else {
        return state.byte_pos(it.text_offset());
    };
    if !is_whitespace(&sample) {
        pskip_nonwhite(&mut it);
        if leading_whitespace {
            pskip_white(&mut it);
        }
    } else {
        pskip_white(&mut it);
    }

    state.byte_pos(it.text_offset())
}

pub fn q_end_of_bigword(mut mul: u32, to: TxtObj, state: &TextAreaState) -> TextPosition {
    let mut it = state.text_graphemes(state.cursor());

    loop {
        if mul == 0 {
            break;
        }
        let Some(sample) = it.peek_next() else {
            return state.byte_pos(it.text_offset());
        };
        if !is_whitespace(&sample) {
            skip_nonwhite(&mut it);
            if to == TxtObj::A {
                skip_white(&mut it);
            }
        } else {
            skip_white(&mut it);
        }
        mul -= 1;
    }

    state.byte_pos(it.text_offset())
}

pub fn q_next_bigword_start(mut mul: u32, state: &TextAreaState) -> Option<TextPosition> {
    let mut it = state.text_graphemes(state.cursor());

    while mul > 0 {
        let Some(sample) = it.peek_next() else {
            return None;
        };
        if !is_whitespace(&sample) {
            skip_nonwhite(&mut it);
        }
        skip_white(&mut it);

        mul -= 1;
    }

    Some(state.byte_pos(it.text_offset()))
}

pub fn q_prev_bigword_start(mut mul: u32, state: &TextAreaState) -> Option<TextPosition> {
    let mut it = state.text_graphemes(state.cursor());

    while mul > 0 {
        let Some(sample) = it.peek_prev() else {
            return None;
        };
        if !is_whitespace(&sample) {
            pskip_nonwhite(&mut it);
        } else {
            pskip_white(&mut it);
            pskip_nonwhite(&mut it);
        }

        mul -= 1;
    }

    Some(state.byte_pos(it.text_offset()))
}

pub fn q_next_bigword_end(mut mul: u32, state: &TextAreaState) -> Option<TextPosition> {
    let mut it = state.text_graphemes(state.cursor());

    while mul > 0 {
        skip_white(&mut it);

        let Some(sample) = it.peek_next() else {
            return None;
        };
        if !is_whitespace(&sample) {
            skip_nonwhite(&mut it);
        }

        mul -= 1;
    }

    Some(state.byte_pos(it.text_offset()))
}

pub fn q_prev_bigword_end(mut mul: u32, state: &TextAreaState) -> Option<TextPosition> {
    let mut it = state.text_graphemes(state.cursor());

    while mul > 0 {
        let Some(sample) = it.peek_prev() else {
            return None;
        };
        if !is_whitespace(&sample) {
            pskip_nonwhite(&mut it);
        }
        pskip_white(&mut it);

        mul -= 1;
    }

    Some(state.byte_pos(it.text_offset()))
}

pub fn q_start_of_line(state: &mut TextAreaState) -> TextPosition {
    TextPosition::new(0, state.cursor().y)
}

pub fn q_start_of_next_line(mul: u32, state: &mut TextAreaState) -> TextPosition {
    let y = min(state.cursor().y + mul as upos_type, state.len_lines());
    TextPosition::new(0, y)
}

pub fn q_end_of_line(mul: u32, state: &mut TextAreaState) -> TextPosition {
    let y = min(
        state.cursor().y + mul.saturating_sub(1) as upos_type,
        state.len_lines().saturating_sub(1),
    );
    TextPosition::new(state.line_width(y), y)
}

pub fn q_start_of_text(state: &mut TextAreaState) -> Option<TextPosition> {
    let mut it = state.line_graphemes(state.cursor().y);
    let found;
    loop {
        let Some(c) = it.next() else {
            return None;
        };
        if !is_whitespace(&c) {
            found = c.text_bytes().start;
            break;
        }
    }

    Some(state.byte_pos(found))
}

pub fn q_end_of_text(mul: u32, state: &mut TextAreaState) -> Option<TextPosition> {
    let y = min(
        state.cursor().y + mul.saturating_sub(1) as upos_type,
        state.len_lines().saturating_sub(1),
    );

    let width = state.line_width(y);
    let mut it = state.graphemes(
        TextPosition::new(0, y)..TextPosition::new(width, y),
        TextPosition::new(width, y),
    );
    let found;
    loop {
        let Some(c) = it.prev() else {
            return None;
        };
        if !is_whitespace(&c) {
            found = c.text_bytes().end;
            break;
        }
    }

    Some(state.byte_pos(found))
}

pub fn q_prev_paragraph(mut mul: u32, state: &mut TextAreaState) -> Option<TextPosition> {
    let mut it = state.text_graphemes(state.cursor());

    let found;
    'f: loop {
        // TODO: pskip_paragraph_whitespace
        if let Some(c) = it.peek_prev()
            && is_whitespace(&c)
        {
            loop {
                let Some(c) = it.prev() else {
                    return None;
                };
                if !is_whitespace(&c) && !is_linebreak(&c) {
                    break;
                }
            }
        }

        let mut paragraph = false;
        loop {
            let Some(c) = it.prev() else {
                if mul == 1 {
                    found = it.text_offset();
                    break 'f;
                } else {
                    return None;
                }
            };

            if is_linebreak(&c) {
                if !paragraph {
                    paragraph = true;
                } else {
                    break;
                }
            } else if !is_whitespace(&c) {
                paragraph = false;
            }
        }

        mul -= 1;
        if mul == 0 {
            it.next();
            found = it.text_offset();
            break 'f;
        }
    }

    Some(state.byte_pos(found))
}

pub fn q_next_paragraph(mut mul: u32, state: &mut TextAreaState) -> Option<TextPosition> {
    let mut it = state.text_graphemes(state.cursor());

    let found;
    'f: loop {
        if let Some(c) = it.peek_next()
            && is_whitespace(&c)
        {
            loop {
                let Some(c) = it.next() else {
                    return None;
                };
                if !is_whitespace(&c) {
                    break;
                }
            }
        }

        let mut paragraph = false;
        loop {
            let Some(c) = it.next() else {
                if mul == 1 {
                    found = it.text_offset();
                    break 'f;
                } else {
                    return None;
                }
            };

            if is_linebreak(&c) {
                if !paragraph {
                    paragraph = true;
                } else {
                    break;
                }
            } else if !is_whitespace(&c) {
                paragraph = false;
            }
        }

        mul -= 1;
        if mul == 0 {
            it.prev();
            found = it.text_offset();
            break 'f;
        }
    }

    Some(state.byte_pos(found))
}

pub fn q_find_fwd(
    mul: u32,
    term: char,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Option<TextPosition> {
    q_find(&mut vi.finds, term, Direction::Forward, false, state);
    q_find_idx(&mut vi.finds, mul, Direction::Forward, state);

    if let Some(i) = vi.finds.idx {
        let pos = state.byte_pos(vi.finds.list[i].0.end);
        Some(pos)
    } else {
        None
    }
}

pub fn q_find_back(
    mul: u32,
    term: char,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Option<TextPosition> {
    q_find(&mut vi.finds, term, Direction::Backward, false, state);
    q_find_idx(&mut vi.finds, mul, Direction::Forward, state);

    if let Some(i) = vi.finds.idx {
        let pos = state.byte_pos(vi.finds.list[i].0.start);
        Some(pos)
    } else {
        None
    }
}

pub fn q_till_fwd(
    mul: u32,
    term: char,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Option<TextPosition> {
    q_find(&mut vi.finds, term, Direction::Forward, true, state);
    q_find_idx(&mut vi.finds, mul, Direction::Forward, state);

    if let Some(i) = vi.finds.idx {
        let pos = state.byte_pos(vi.finds.list[i].0.start);
        Some(pos)
    } else {
        None
    }
}

pub fn q_till_back(
    mul: u32,
    term: char,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Option<TextPosition> {
    q_find(&mut vi.finds, term, Direction::Backward, true, state);
    q_find_idx(&mut vi.finds, mul, Direction::Forward, state);

    if let Some(i) = vi.finds.idx {
        let pos = state.byte_pos(vi.finds.list[i].0.end);
        Some(pos)
    } else {
        None
    }
}

pub fn q_find_repeat_fwd(mul: u32, state: &mut TextAreaState, vi: &mut VI) -> Option<TextPosition> {
    let Some(last_term) = vi.finds.term else {
        return None;
    };

    let last_dir = vi.finds.dir;
    let last_till = vi.finds.till;

    q_find(&mut vi.finds, last_term, last_dir, last_till, state);
    q_find_idx(&mut vi.finds, mul, Direction::Forward, state);

    let dir = vi.finds.dir.mul(Direction::Forward);

    if let Some(idx) = vi.finds.idx {
        let pos = if vi.finds.till {
            if dir == Direction::Backward {
                state.byte_pos(vi.finds.list[idx].0.end)
            } else {
                state.byte_pos(vi.finds.list[idx].0.start)
            }
        } else {
            if dir == Direction::Backward {
                state.byte_pos(vi.finds.list[idx].0.start)
            } else {
                state.byte_pos(vi.finds.list[idx].0.end)
            }
        };
        Some(pos)
    } else {
        None
    }
}

pub fn q_find_repeat_back(
    mul: u32,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Option<TextPosition> {
    let Some(last_term) = vi.finds.term else {
        return None;
    };

    let last_dir = vi.finds.dir;
    let last_till = vi.finds.till;

    q_find(&mut vi.finds, last_term, last_dir, last_till, state);
    q_find_idx(&mut vi.finds, mul, Direction::Backward, state);

    let dir = vi.finds.dir.mul(Direction::Backward);

    if let Some(idx) = vi.finds.idx {
        let pos = if vi.finds.till {
            if dir == Direction::Backward {
                state.byte_pos(vi.finds.list[idx].0.end)
            } else {
                state.byte_pos(vi.finds.list[idx].0.start)
            }
        } else {
            if dir == Direction::Backward {
                state.byte_pos(vi.finds.list[idx].0.start)
            } else {
                state.byte_pos(vi.finds.list[idx].0.end)
            }
        };
        Some(pos)
    } else {
        None
    }
}

pub fn q_find_idx(finds: &mut Finds, mul: u32, dir: Direction, state: &mut TextAreaState) {
    let mut c = state.byte_at(state.cursor()).start;

    let dir = finds.dir.mul(dir);
    let mul = (mul as usize).saturating_sub(1);

    if dir == Direction::Forward {
        finds.idx = finds.list.iter().position(|(v, _)| v.start > c);

        finds.idx = if let Some(idx) = finds.idx {
            if idx + mul < finds.list.len() {
                Some(idx + mul)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        // Till backwards might need to correct the cursor.
        if finds.till {
            if let Some(i) = finds.idx {
                let r = finds.list[i].0.clone();
                if c == r.end {
                    c = r.start;
                }
            }
        }

        finds.idx = finds
            .list
            .iter()
            .enumerate()
            .filter_map(|(i, (v, _))| if v.end < c { Some(i) } else { None })
            .last();

        finds.idx = if let Some(idx) = finds.idx {
            if idx >= mul { Some(idx - mul) } else { None }
        } else {
            None
        }
    }
}

pub fn q_find(finds: &mut Finds, term: char, dir: Direction, till: bool, state: &TextAreaState) {
    if finds.term != Some(term) || finds.row != state.cursor().y {
        finds.term = Some(term);
        finds.row = state.cursor().y;
        finds.dir = dir;
        finds.till = till;
        finds.idx = None;
        finds.list.clear();
        finds.sync = SyncRanges::ToTextArea;

        let cursor = state.cursor();
        let start = TextPosition::new(0, cursor.y);
        let end = TextPosition::new(state.line_width(cursor.y), cursor.y);
        let mut it = state.graphemes(TextRange::new(start, end), start);
        loop {
            let Some(c) = it.next() else {
                break;
            };
            if c == term {
                finds.list.push((c.text_bytes(), 998));
            }
        }
    } else {
        finds.row = state.cursor().y;
        finds.dir = dir;
        finds.till = till;
        finds.idx = None;
    }
}

pub fn q_search_word_fwd(
    mul: u32,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Result<Option<TextPosition>, SearchError> {
    let start = qq_word_start(state);
    let end = qq_word_end(state);
    let term = state.str_slice(TextRange::from(start..end)).to_string();

    q_search(&mut vi.matches, &term, Direction::Forward, false, state)?;
    q_search_idx(&mut vi.matches, mul, Direction::Forward, state);
    Ok(q_current_search_idx(state, vi))
}

pub fn q_search_word_back(
    mul: u32,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Result<Option<TextPosition>, SearchError> {
    let start = qq_word_start(state);
    let end = qq_word_end(state);
    let term = state.str_slice(TextRange::from(start..end)).to_string();

    q_search(&mut vi.matches, &term, Direction::Backward, false, state)?;
    q_search_idx(&mut vi.matches, mul, Direction::Forward, state);
    Ok(q_current_search_idx(state, vi))
}

fn qq_word_start(state: &TextAreaState) -> TextPosition {
    let mut it = state.text_graphemes(state.cursor());

    if let Some(sample) = it.peek_prev() {
        if is_alphanumeric(&sample) {
            pskip_alpha(&mut it);
        } else if is_whitespace(&sample) {
            // noop
        } else {
            pskip_sample(&mut it, sample);
        }
    }

    state.byte_pos(it.text_offset())
}

fn qq_word_end(state: &TextAreaState) -> TextPosition {
    let mut it = state.text_graphemes(state.cursor());

    if let Some(sample) = it.peek_next() {
        if is_alphanumeric(&sample) {
            skip_alpha(&mut it);
        } else if is_whitespace(&sample) {
            // noop
        } else {
            skip_sample(&mut it, sample);
        }
    }

    state.byte_pos(it.text_offset())
}

pub fn q_search_back(
    mul: u32,
    term: &str,
    tmp: bool,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Result<Option<TextPosition>, SearchError> {
    q_search(&mut vi.matches, term, Direction::Backward, tmp, state)?;
    q_search_idx(&mut vi.matches, mul, Direction::Forward, state);
    Ok(q_current_search_idx(state, vi))
}

pub fn q_search_fwd(
    mul: u32,
    term: &str,
    tmp: bool,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Result<Option<TextPosition>, SearchError> {
    q_search(&mut vi.matches, term, Direction::Forward, tmp, state)?;
    q_search_idx(&mut vi.matches, mul, Direction::Forward, state);
    Ok(q_current_search_idx(state, vi))
}

pub fn q_search_repeat_back(
    mul: u32,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Option<TextPosition> {
    if vi.matches.term.is_none() {
        return None;
    }
    q_search_idx(&mut vi.matches, mul, Direction::Backward, state);
    q_current_search_idx(state, vi)
}

pub fn q_search_repeat_fwd(
    mul: u32,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Option<TextPosition> {
    if vi.matches.term.is_none() {
        return None;
    }
    q_search_idx(&mut vi.matches, mul, Direction::Forward, state);
    q_current_search_idx(state, vi)
}

pub fn q_search(
    matches: &mut Matches,
    term: &str,
    dir: Direction,
    tmp: bool,
    state: &mut TextAreaState,
) -> Result<(), SearchError> {
    if matches
        .term
        .as_ref()
        .map(|v| v.as_str() != term)
        .unwrap_or(true)
    {
        matches.term = Some(term.to_string());
        matches.dir = dir;
        matches.tmp = tmp;
        matches.idx = None;
        matches.list.clear();
        matches.sync = SyncRanges::ToTextArea;

        if let Ok(re) = Regex::new(matches.term.as_ref().expect("term")) {
            let cursor = RopeyCursor::new(state.rope().byte_slice(..));
            let input = Input::new(cursor);

            for m in find_iter(&re, input) {
                matches.list.push((m.start()..m.end(), 999));
            }
        }
    } else {
        matches.dir = dir;
        matches.tmp = tmp;
        matches.idx = None;
    }
    Ok(())
}

pub fn q_search_idx(matches: &mut Matches, mul: u32, dir: Direction, state: &mut TextAreaState) {
    let c = state.byte_at(state.cursor()).start;
    let dir = matches.dir.mul(dir);
    let mul = (mul as usize).saturating_sub(1);

    if dir == Direction::Forward {
        matches.idx = matches.list.iter().position(|(v, _)| v.start > c);

        matches.idx = if let Some(idx) = matches.idx {
            if idx + mul < matches.list.len() {
                Some(idx + mul)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        matches.idx = matches
            .list
            .iter()
            .enumerate()
            .filter_map(|(i, (v, _))| if v.end < c { Some(i) } else { None })
            .last();

        matches.idx = if let Some(idx) = matches.idx {
            if idx >= mul { Some(idx - mul) } else { None }
        } else {
            None
        }
    }
}

pub fn q_current_search_idx(state: &mut TextAreaState, vi: &mut VI) -> Option<TextPosition> {
    if let Some(idx) = vi.matches.idx {
        let pos = state.byte_pos(vi.matches.list[idx].0.start);
        Some(pos)
    } else {
        None
    }
}

pub fn q_line_break_and_leading_space(state: &mut TextAreaState) -> TextRange {
    let c = state.cursor();
    let width = state.line_width(c.y);

    let start = TextPosition::new(width, c.y);

    let mut it = state.text_graphemes(start);
    skip_white(&mut it);
    let end = state.byte_pos(it.text_offset());

    TextRange::new(start, end)
}

fn pskip_paragraph_whitespace<'a, C: Cursor<Item = Grapheme<'a>>>(it: &mut C) {
    loop {
        let Some(c) = it.prev() else {
            break;
        };
        if !is_whitespace(&c) && !is_linebreak(&c) {
            it.next();
            break;
        }
    }
}

fn skip_paragraph_whitespace<'a, C: Cursor<Item = Grapheme<'a>>>(it: &mut C) {
    loop {
        let Some(c) = it.next() else {
            break;
        };
        if !is_whitespace(&c) && !is_linebreak(&c) {
            it.prev();
            break;
        }
    }
}

fn is_paragraph_whitespace<'a, C: Cursor<Item = Grapheme<'a>> + Clone>(it: &mut C) -> bool {
    let mut jt = it.clone();
    let before;
    loop {
        let Some(c) = jt.prev() else {
            before = true;
            break;
        };
        if is_linebreak(&c) {
            before = true;
            break;
        }
        if !is_whitespace(&c) {
            before = false;
            break;
        }
    }

    let mut jt = it.clone();
    let after;
    loop {
        let Some(c) = jt.next() else {
            after = true;
            break;
        };
        if is_linebreak(&c) {
            after = true;
            break;
        }
        if !is_whitespace(&c) {
            after = false;
            break;
        }
    }

    before && after
}

fn track_paragraph_fwd<'a, C: Cursor<Item = Grapheme<'a>>>(
    c: &Grapheme,
    track: &mut bool,
    it: &mut C,
) -> bool {
    if is_linebreak(&c) {
        if !*track {
            *track = true;
            false
        } else {
            it.prev();
            loop {
                let Some(d) = it.prev() else {
                    break;
                };
                if is_linebreak(&d) {
                    break;
                }
            }
            true
        }
    } else if !is_whitespace(&c) {
        *track = false;
        false
    } else {
        // keep state
        false
    }
}

fn track_paragraph_back<'a, C: Cursor<Item = Grapheme<'a>>>(
    c: &Grapheme,
    track: &mut bool,
    it: &mut C,
) -> bool {
    if is_linebreak(&c) {
        if !*track {
            *track = true;
            false
        } else {
            it.next();
            loop {
                let Some(d) = it.next() else {
                    break;
                };
                if is_linebreak(&d) {
                    // it.prev(); don't include linebreak.
                    break;
                }
            }
            true
        }
    } else if !is_whitespace(&c) {
        *track = false;
        false
    } else {
        // keep state
        false
    }
}

#[inline]
fn pskip_alpha<'a, C: Cursor<Item = Grapheme<'a>>>(it: &mut C) {
    loop {
        let Some(c) = it.prev() else {
            break;
        };
        if !is_alphanumeric(&c) {
            it.next();
            break;
        }
    }
}

#[inline]
fn pskip_sample<'a, C: Cursor<Item = Grapheme<'a>>>(it: &mut C, sample: Grapheme) {
    loop {
        let Some(c) = it.prev() else {
            break;
        };
        if c != sample {
            it.next();
            break;
        }
    }
}

#[inline]
fn pskip_white<'a, C: Cursor<Item = Grapheme<'a>>>(it: &mut C) {
    loop {
        let Some(c) = it.prev() else {
            break;
        };
        if !is_whitespace(&c) {
            it.next();
            break;
        }
    }
}

#[inline]
fn pskip_nonwhite<'a, C: Cursor<Item = Grapheme<'a>>>(it: &mut C) {
    loop {
        let Some(c) = it.prev() else {
            break;
        };
        if is_whitespace(&c) || is_linebreak(&c) {
            it.next();
            break;
        }
    }
}

#[inline]
fn skip_alpha<'a, C: Cursor<Item = Grapheme<'a>>>(it: &mut C) {
    loop {
        let Some(c) = it.next() else {
            break;
        };
        if !is_alphanumeric(&c) {
            it.prev();
            break;
        }
    }
}

#[inline]
fn skip_sample<'a, C: Cursor<Item = Grapheme<'a>>>(it: &mut C, sample: Grapheme) {
    loop {
        let Some(c) = it.next() else {
            break;
        };
        if c != sample {
            it.prev();
            break;
        }
    }
}

#[inline]
fn skip_white<'a, C: Cursor<Item = Grapheme<'a>>>(it: &mut C) {
    loop {
        let Some(c) = it.next() else {
            break;
        };
        if !is_whitespace(&c) {
            it.prev();
            break;
        }
    }
}

#[inline]
fn skip_nonwhite<'a, C: Cursor<Item = Grapheme<'a>>>(it: &mut C) {
    loop {
        let Some(c) = it.next() else {
            break;
        };
        if is_whitespace(&c) || is_linebreak(&c) {
            it.prev();
            break;
        }
    }
}

pub fn is_alphanumeric(g: &Grapheme<'_>) -> bool {
    g.grapheme()
        .chars()
        .next()
        .map(|v| v.is_alphanumeric() || v == '_')
        .unwrap_or(false)
}

pub fn is_whitespace(g: &Grapheme<'_>) -> bool {
    g.grapheme()
        .chars()
        .next()
        .map(|v| match v {
            '\x0a' | '\x0b' | '\x0c' | '\x0d' => false,
            c => c.is_whitespace(),
        })
        .unwrap_or(false)
}

pub fn is_linebreak(g: &Grapheme<'_>) -> bool {
    g.grapheme()
        .chars()
        .next()
        .map(|v| match v {
            '\x0a' | '\x0d' => true,
            _ => false,
        })
        .unwrap_or(false)
}

pub fn q_prepend_line_str(v: &str, state: &mut TextAreaState) {
    let c = state.cursor();
    state.begin_undo_seq();
    state.set_cursor(TextPosition::new(0, c.y), false);
    state.insert_newline();
    state.set_cursor(TextPosition::new(0, c.y), false);
    for c in v.chars() {
        q_insert(c, state);
    }
    state.end_undo_seq();
}

pub fn q_append_line_str(v: &str, state: &mut TextAreaState) {
    let c = state.cursor();
    let width = state.line_width(c.y);
    state.begin_undo_seq();
    state.set_cursor(TextPosition::new(width, c.y), false);
    state.insert_newline();
    for c in v.chars() {
        q_insert(c, state);
    }
    state.end_undo_seq();
}

pub fn q_append_str(v: &str, state: &mut TextAreaState) {
    state.begin_undo_seq();
    state.move_right(1, false);
    for c in v.chars() {
        q_insert(c, state);
    }
    state.end_undo_seq();
}

pub fn q_insert_str(v: &str, state: &mut TextAreaState) {
    state.begin_undo_seq();
    for c in v.chars() {
        q_insert(c, state);
    }
    state.end_undo_seq();
}

pub fn q_insert(cc: char, state: &mut TextAreaState) {
    _ = match cc {
        '\n' => state.insert_newline(),
        '\t' => state.insert_tab(),
        ctrl::BS => state.delete_prev_char(),
        ctrl::DEL => state.delete_next_char(),
        _ => state.insert_char(cc),
    };
}

pub fn q_visual_select(state: &mut TextAreaState, vi: &mut VI) {
    vi.visual.sync = SyncRanges::ToTextArea;
    vi.visual.list.clear();
    if vi.visual.block {
        let (x0, x1) = if vi.visual.anchor.x > vi.visual.lead.x {
            (vi.visual.lead.x, vi.visual.anchor.x)
        } else {
            (vi.visual.anchor.x, vi.visual.lead.x)
        };
        let (y0, y1) = if vi.visual.anchor.y > vi.visual.lead.y {
            (vi.visual.lead.y, vi.visual.anchor.y)
        } else {
            (vi.visual.anchor.y, vi.visual.lead.y)
        };

        for y in y0..=y1 {
            let width = state.line_width(y);
            let xx0 = min(x0, width);
            let xx1 = min(x1, width);
            let r = state.bytes_at_range((xx0, y)..(xx1, y));
            vi.visual.list.push((r, 997));
        }
    } else {
        let (begin, end) = if vi.visual.anchor > vi.visual.lead {
            (vi.visual.lead, vi.visual.anchor)
        } else {
            (vi.visual.anchor, vi.visual.lead)
        };

        let begin = state.byte_at(begin);
        let end = state.byte_at(end);
        vi.visual.list.push((begin.start..end.start, 997));
    }
}
