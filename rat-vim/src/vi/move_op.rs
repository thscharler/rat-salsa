use crate::SearchError;
use crate::vi::motion_op::{is_motion_a_jump, motion_end_position};
use crate::vi::query::q_set_mark;
use crate::vi::{Mark, Motion, VI};
use rat_text::text_area::TextAreaState;

pub fn move_cursor(
    mul: u32,
    motion: &Motion,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Result<(), SearchError> {
    if let Some(npos) = motion_end_position(mul, motion, state, vi)? {
        if is_motion_a_jump(motion) {
            q_set_mark(Mark::Jump, state.cursor(), vi);
        }
        state.set_cursor(npos, false);
    }
    Ok(())
}

pub fn jump_history(mul: i32, state: &mut TextAreaState, vi: &mut VI) {
    vi.marks.jump_idx = vi.marks.jump_idx.saturating_add_signed(mul as isize);
    if vi.marks.jump_idx > vi.marks.jump.len() {
        vi.marks.jump_idx = vi.marks.jump.len();
    }

    if vi.marks.jump_idx < vi.marks.jump.len() {
        state.set_cursor(vi.marks.jump[vi.marks.jump_idx], false);
    }
}

pub fn jump_change(mul: i32, state: &mut TextAreaState, vi: &mut VI) {
    vi.marks.change_idx = vi.marks.change_idx.saturating_add_signed(mul as isize);
    if vi.marks.change_idx > vi.marks.change.len() {
        vi.marks.change_idx = vi.marks.change.len();
    }

    if vi.marks.change_idx < vi.marks.change.len() {
        state.set_cursor(vi.marks.change[vi.marks.change_idx].0, false);
    }
}
