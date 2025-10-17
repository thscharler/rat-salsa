use crate::VI;
use crate::vi::Mark;
use crate::vi::query::q_set_mark;
use rat_text::text_area::TextAreaState;

pub fn set_mark(mark: Mark, state: &mut TextAreaState, vi: &mut VI) {
    q_set_mark(mark, state.cursor(), vi);
}
