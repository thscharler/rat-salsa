use crate::SearchError;
use crate::coroutine::Coroutine;
use crate::vi::change_op::*;
use crate::vi::query::q_set_mark;
use crate::vi::state_machine::*;
use crate::vi::visual_op::end_visual_change;
use crate::vi::{Mark, Mode, Motion, SyncRanges, VI, Vim};
use rat_text::TextPosition;
use rat_text::text_area::TextAreaState;
use std::mem;

pub fn reset_co_normal(vi: &mut VI) {
    vi.command_display.borrow_mut().clear();
    let mb = vi.command_display.clone();
    vi.co_normal = Coroutine::new(|c, yp| Box::new(next_normal(c, mb, yp)));
}

pub fn reset_co_visual(vi: &mut VI) {
    vi.command_display.borrow_mut().clear();
    let mb = vi.command_display.clone();
    vi.co_visual = Coroutine::new(|c, yp| Box::new(next_visual(c, mb, yp)));
}

pub fn reset_normal_mode(vi: &mut VI) {
    vi.mode = Mode::Normal;
    vi.matches.clear();
    vi.finds.clear();
}

pub fn reset_visual_mode(vi: &mut VI) {
    vi.mode = Mode::Normal;
    vi.visual.clear();
}

pub fn reset_insert_mode(vi: &mut VI) {
    vi.mode = Mode::Normal;
    vi.matches.sync = SyncRanges::FromTextArea;
    vi.finds.sync = SyncRanges::FromTextArea;
}

pub fn begin_visual(block: bool, state: &mut TextAreaState, vi: &mut VI) {
    vi.mode = Mode::Visual;
    vi.visual.block = block;
    vi.visual.anchor = state.cursor();
    vi.visual.lead = state.cursor();
    vi.visual.sync = SyncRanges::ToTextArea;
}

pub fn begin_prepend_line(state: &mut TextAreaState, vi: &mut VI) {
    vi.mode = Mode::Insert;
    vi.text.clear();

    q_set_mark(Mark::ChangeStart, state.cursor(), vi);

    let c = state.cursor();
    state.set_cursor(TextPosition::new(0, c.y), false);
    state.insert_newline();
    state.set_cursor(TextPosition::new(0, c.y), false);
}

pub fn begin_append_line(state: &mut TextAreaState, vi: &mut VI) {
    vi.mode = Mode::Insert;
    vi.text.clear();

    q_set_mark(Mark::ChangeStart, state.cursor(), vi);

    let c = state.cursor();
    let width = state.line_width(c.y);
    state.set_cursor(TextPosition::new(width, c.y), false);
    state.insert_newline();
}

pub fn begin_append(state: &mut TextAreaState, vi: &mut VI) {
    vi.mode = Mode::Insert;
    vi.text.clear();

    q_set_mark(Mark::ChangeStart, state.cursor(), vi);

    state.move_right(1, false);
}

pub fn begin_change_text(
    mul: u32,
    motion: &Motion,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Result<(), SearchError> {
    vi.mode = Mode::Insert;
    vi.text.clear();

    q_set_mark(Mark::ChangeStart, state.cursor(), vi);

    change_text_str(mul, motion, state, vi)?;

    Ok(())
}

pub fn begin_insert(state: &mut TextAreaState, vi: &mut VI) {
    vi.mode = Mode::Insert;
    vi.text.clear();

    q_set_mark(Mark::ChangeStart, state.cursor(), vi);
}

pub fn end_insert(state: &mut TextAreaState, vi: &mut VI) {
    let mut command = mem::take(&mut vi.command);
    match &command {
        Vim::Insert(mul) => {
            insert_str(mul.saturating_sub(1), state, vi);
        }
        Vim::Change(_, Motion::Visual) => {
            end_visual_change(state, vi);
            // don't allow repeat
            command = Vim::Invalid;
        }
        Vim::Change(_, _) => {
            // noop
        }
        Vim::Append(mul) => {
            append_str(mul.saturating_sub(1), state, vi);
        }
        Vim::AppendLine(mul) => {
            append_line_str(mul.saturating_sub(1), state, vi);
        }
        Vim::PrependLine(mul) => {
            prepend_line_str(mul.saturating_sub(1), state, vi);
        }
        _ => {
            unreachable!()
        }
    };
    q_set_mark(Mark::Insert, state.cursor(), vi);
    q_set_mark(Mark::ChangeEnd, state.cursor(), vi);

    vi.command = command;
}
