use crate::vi::query::*;
use crate::vi::{Motion, TxtObj};
use crate::{SearchError, VI};
use rat_text::TextPosition;
use rat_text::text_area::TextAreaState;
use std::ops::Range;

pub fn start_end_to_range(
    start: TextPosition,
    end: Option<TextPosition>,
) -> Option<Range<TextPosition>> {
    if let Some(end) = end {
        if start > end {
            Some(end..start)
        } else {
            Some(start..end)
        }
    } else {
        None
    }
}

pub fn motion_start_position(motion: &Motion, state: &mut TextAreaState) -> TextPosition {
    match motion {
        Motion::Visual => unreachable!(),
        Motion::FullLine => q_start_of_line(state),
        Motion::Word(to) => q_start_of_word(*to, state),
        Motion::WORD(to) => q_start_of_bigword(*to, state),
        Motion::Sentence(to) => q_prev_sentence(1, *to, state),
        Motion::Paragraph(_) => q_prev_paragraph(1, state),
        Motion::Bracket(to) => todo!(),
        Motion::Parenthesis(to) => todo!(),
        Motion::Angled(to) => todo!(),
        Motion::Tagged(to) => todo!(),
        Motion::Brace(to) => todo!(),
        Motion::Quoted(c, to) => todo!(),
        _ => state.cursor(),
    }
}

pub fn is_motion_a_jump(motion: &Motion) -> bool {
    match motion {
        Motion::Visual => unreachable!(),
        Motion::Left => false,
        Motion::Right => false,
        Motion::Up => false,
        Motion::Down => false,
        Motion::HalfPageUp => false,
        Motion::HalfPageDown => false,
        Motion::ToTopOfScreen => true,
        Motion::ToMiddleOfScreen => true,
        Motion::ToBottomOfScreen => true,
        Motion::ToCol => false,
        Motion::ToLine => true,
        Motion::ToLinePercent => true,
        Motion::ToMatchingBrace => true,
        Motion::ToMark(_, _) => true,
        Motion::StartOfFile => false,
        Motion::EndOfFile => true,
        Motion::NextWordStart => false,
        Motion::PrevWordStart => false,
        Motion::NextWordEnd => false,
        Motion::PrevWordEnd => false,
        Motion::NextWORDStart => false,
        Motion::PrevWORDStart => false,
        Motion::NextWORDEnd => false,
        Motion::PrevWORDEnd => false,
        Motion::StartOfLine => false,
        Motion::EndOfLine => false,
        Motion::StartOfLineText => false,
        Motion::EndOfLineText => false,
        Motion::PrevParagraph => true,
        Motion::NextParagraph => true,
        Motion::PrevSentence => true,
        Motion::NextSentence => true,
        Motion::FindForward(_) => false,
        Motion::FindBack(_) => false,
        Motion::FindTillForward(_) => false,
        Motion::FindTillBack(_) => false,
        Motion::FindRepeatNext => false,
        Motion::FindRepeatPrev => false,
        Motion::SearchWordForward => true,
        Motion::SearchWordBackward => true,
        Motion::SearchForward(_) => true,
        Motion::SearchBack(_) => true,
        Motion::SearchRepeatNext => true,
        Motion::SearchRepeatPrev => true,
        Motion::FullLine => false,
        Motion::Word(_) => false,
        Motion::WORD(_) => false,
        Motion::Sentence(_) => false,
        Motion::Paragraph(_) => false,
        Motion::Bracket(_) => false,
        Motion::Parenthesis(_) => false,
        Motion::Angled(_) => false,
        Motion::Tagged(_) => false,
        Motion::Brace(_) => false,
        Motion::Quoted(_, _) => false,
    }
}

pub fn motion_end_position(
    mul: u32,
    motion: &Motion,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Result<Option<TextPosition>, SearchError> {
    let r = match motion {
        Motion::Visual => unreachable!(),
        Motion::Left => Some(q_move_left(mul, state)),
        Motion::Right => Some(q_move_right(mul, state)),
        Motion::Up => Some(q_move_up(mul, state)),
        Motion::Down => Some(q_move_down(mul, state)),
        Motion::HalfPageUp => Some(q_half_page_up(mul, state, vi)),
        Motion::HalfPageDown => Some(q_half_page_down(mul, state, vi)),
        Motion::ToTopOfScreen => todo!(),
        Motion::ToMiddleOfScreen => todo!(),
        Motion::ToBottomOfScreen => todo!(),
        Motion::ToCol => Some(q_col(mul, state)),
        Motion::ToLine => Some(q_line(mul, state)),
        Motion::ToLinePercent => Some(q_line_percent(mul, state)),
        Motion::ToMatchingBrace => q_matching_brace(state),
        Motion::ToMark(mark, line) => q_mark(*mark, *line, state, vi),
        Motion::StartOfFile => Some(q_start_of_file()),
        Motion::EndOfFile => Some(q_end_of_file(state)),
        Motion::NextWordStart => Some(q_next_word_start(mul, state)),
        Motion::PrevWordStart => Some(q_prev_word_start(mul, state)),
        Motion::NextWordEnd => Some(q_next_word_end(mul, state)),
        Motion::PrevWordEnd => Some(q_prev_word_end(mul, state)),
        Motion::NextWORDStart => Some(q_next_bigword_start(mul, state)),
        Motion::PrevWORDStart => Some(q_prev_bigword_start(mul, state)),
        Motion::NextWORDEnd => Some(q_next_bigword_end(mul, state)),
        Motion::PrevWORDEnd => Some(q_prev_bigword_end(mul, state)),
        Motion::StartOfLine => Some(q_start_of_line(state)),
        Motion::EndOfLine => Some(q_end_of_line(mul, state)),
        Motion::StartOfLineText => Some(q_start_of_text(state)),
        Motion::EndOfLineText => Some(q_end_of_text(mul, state)),
        Motion::PrevParagraph => Some(q_prev_paragraph(mul, state)),
        Motion::NextParagraph => Some(q_next_paragraph(mul, TxtObj::A, state)),
        Motion::PrevSentence => Some(q_prev_sentence(mul, TxtObj::A, state)),
        Motion::NextSentence => Some(q_next_sentence(mul, TxtObj::I, state)),
        Motion::FindForward(f) => q_find_fwd(mul, *f, state, vi),
        Motion::FindBack(f) => q_find_back(mul, *f, state, vi),
        Motion::FindTillForward(f) => q_till_fwd(mul, *f, state, vi),
        Motion::FindTillBack(f) => q_till_back(mul, *f, state, vi),
        Motion::FindRepeatNext => q_find_repeat_fwd(mul, state, vi),
        Motion::FindRepeatPrev => q_find_repeat_back(mul, state, vi),
        Motion::SearchWordForward => q_search_word_fwd(mul, state, vi)?,
        Motion::SearchWordBackward => q_search_word_back(mul, state, vi)?,
        Motion::SearchForward(term) => q_search_fwd(mul, &term, false, state, vi)?,
        Motion::SearchBack(term) => q_search_back(mul, &term, false, state, vi)?,
        Motion::SearchRepeatNext => q_search_repeat_fwd(mul, state, vi),
        Motion::SearchRepeatPrev => q_search_repeat_back(mul, state, vi),
        Motion::FullLine => Some(q_start_of_next_line(mul, state)),
        Motion::Word(to) => Some(q_end_of_word(mul, *to, state)),
        Motion::WORD(to) => Some(q_end_of_bigword(mul, *to, state)),
        Motion::Sentence(to) => Some(q_next_sentence(mul, *to, state)),
        Motion::Paragraph(to) => Some(q_next_paragraph(mul, *to, state)),
        Motion::Bracket(to) => todo!(),
        Motion::Parenthesis(to) => todo!(),
        Motion::Angled(to) => todo!(),
        Motion::Tagged(to) => todo!(),
        Motion::Brace(to) => todo!(),
        Motion::Quoted(c, to) => todo!(),
    };

    Ok(r)
}
