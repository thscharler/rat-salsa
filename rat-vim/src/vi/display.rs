use crate::vi::query::{q_find_idx, q_search_idx};
use crate::vi::{Direction, SyncRanges, VI};
use rat_text::text_area::TextAreaState;

pub fn display_visual(state: &mut TextAreaState, vi: &mut VI) {
    match vi.visual.sync {
        SyncRanges::None => {}
        SyncRanges::ToTextArea => {
            vi.visual.sync = SyncRanges::None;
            state.remove_style_fully(997);
            for r in &vi.visual.list {
                state.add_style(r.0.clone(), 997)
            }
        }
        SyncRanges::FromTextArea => {
            unreachable!("no sync");
        }
    }
}

pub fn display_matches(state: &mut TextAreaState, vi: &mut VI) {
    match vi.matches.sync {
        SyncRanges::None => {}
        SyncRanges::ToTextArea => {
            vi.matches.sync = SyncRanges::None;
            state.remove_style_fully(999);
            for r in &vi.matches.list {
                state.add_style(r.0.clone(), 999)
            }
        }
        SyncRanges::FromTextArea => {
            vi.matches.sync = SyncRanges::None;
            vi.matches.list.clear();
            state.styles_in_match(0..state.len_bytes(), 999, &mut vi.matches.list);
            q_search_idx(&mut vi.matches, 1, Direction::Forward, state);
        }
    }
}

pub fn display_finds(state: &mut TextAreaState, vi: &mut VI) {
    match vi.finds.sync {
        SyncRanges::None => {}
        SyncRanges::ToTextArea => {
            vi.finds.sync = SyncRanges::None;
            state.remove_style_fully(998);
            for r in &vi.finds.list {
                state.add_style(r.0.clone(), 998)
            }
        }
        SyncRanges::FromTextArea => {
            vi.finds.sync = SyncRanges::None;
            vi.finds.list.clear();
            state.styles_in_match(0..state.len_bytes(), 998, &mut vi.finds.list);
            q_find_idx(&mut vi.finds, 1, Direction::Forward, state);
        }
    }
}
