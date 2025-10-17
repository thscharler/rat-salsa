use crate::SearchError;
use crate::vi::query::*;
use crate::vi::{Direction, VI};
use rat_text::text_area::TextAreaState;

pub fn search_back(
    mul: u32,
    term: &str,
    tmp: bool,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Result<(), SearchError> {
    q_search(&mut vi.matches, term, Direction::Backward, tmp, state)?;
    q_search_idx(&mut vi.matches, mul, Direction::Forward, state);
    Ok(())
}

pub fn search_fwd(
    mul: u32,
    term: &str,
    tmp: bool,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Result<(), SearchError> {
    q_search(&mut vi.matches, term, Direction::Forward, tmp, state)?;
    q_search_idx(&mut vi.matches, mul, Direction::Forward, state);
    Ok(())
}
