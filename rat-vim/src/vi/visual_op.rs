use crate::SearchError;
use crate::vi::motion_op::motion_end_position;
use crate::vi::query::*;
use crate::vi::{Mark, Mode, Motion, VI};
use rat_text::TextPosition;
use rat_text::text_area::TextAreaState;

pub fn visual_move(
    mul: u32,
    motion: &Motion,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Result<(), SearchError> {
    if let Some(npos) = motion_end_position(mul, motion, state, vi)? {
        vi.visual.lead = npos;
        q_visual_select(state, vi);
        state.set_cursor(npos, false);
    }
    Ok(())
}

pub fn visual_swap_diagonal(state: &mut TextAreaState, vi: &mut VI) {
    let anchor = vi.visual.anchor;
    let lead = vi.visual.lead;

    vi.visual.anchor = TextPosition::new(lead.x, anchor.y);
    vi.visual.lead = TextPosition::new(anchor.x, lead.y);
    q_visual_select(state, vi);
    state.set_cursor(vi.visual.lead, false);
}

pub fn visual_swap_lead(state: &mut TextAreaState, vi: &mut VI) {
    let anchor = vi.visual.anchor;
    let lead = vi.visual.lead;

    vi.visual.anchor = lead;
    vi.visual.lead = anchor;
    q_visual_select(state, vi);
    state.set_cursor(vi.visual.lead, false);
}

pub fn visual_delete(state: &mut TextAreaState, vi: &mut VI) {
    // undo would restore these.
    state.remove_style_fully(997);

    if let Some((vpos, _)) = vi.visual.list.first() {
        let vpos = state.byte_pos(vpos.start);
        q_set_mark(Mark::VisualAnchor, vpos, vi);
        q_set_mark(Mark::ChangeStart, state.cursor(), vi);
        q_set_mark(Mark::ChangeEnd, state.cursor(), vi);
    }
    if let Some((vpos, _)) = vi.visual.list.last() {
        let vpos = state.byte_pos(vpos.start);
        q_set_mark(Mark::VisualAnchor, vpos, vi);
    }

    vi.yank.list.clear();

    state.begin_undo_seq();
    loop {
        let Some((r, _)) = vi.visual.list.pop() else {
            break;
        };
        let r = state.byte_range(r);
        vi.yank.list.push(state.str_slice(r).into_owned());
        state.delete_range(r);
    }
    state.end_undo_seq();

    vi.yank.list.reverse();

    vi.mode = Mode::Normal;
    vi.visual.clear();
}

/// Begin visual change.
///
/// [end_insert](crate::vi::modes_op::end_insert) does the rest.
/// Most importantly calling [end_visual_change].
pub fn visual_change(state: &mut TextAreaState, vi: &mut VI) {
    let Some((r, _)) = vi.visual.list.first() else {
        vi.mode = Mode::Normal;
        vi.visual.clear();
        return;
    };
    state.remove_style(r.clone(), 997);
    let r = state.byte_range(r.clone());
    state.begin_undo_seq(); // ends with visual_multi_change()
    state.delete_range(r);
    state.set_cursor(r.start, false);

    q_set_mark(Mark::ChangeStart, state.cursor(), vi);

    vi.mode = Mode::Insert;
    vi.text.clear();
}

/// Ends a visual change and applies it to the multi selection.
pub fn end_visual_change(state: &mut TextAreaState, vi: &mut VI) {
    let Some((r, _)) = vi.visual.list.first() else {
        unreachable!("invalid change");
    };
    let new_cursor = state.byte_pos(r.start);

    // sync back visual selection
    vi.visual.list.clear();
    state.styles_in_match(0..state.len_bytes(), 997, &mut vi.visual.list);

    // undo would restore these.
    state.remove_style_fully(997);

    loop {
        let Some((r, _)) = vi.visual.list.pop() else {
            break;
        };
        let r = state.byte_range(r);
        state.delete_range(r);
        state.set_cursor(r.start, false);
        q_insert_str(&vi.text, state);
    }
    state.end_undo_seq(); // starts with visual_change()

    state.set_cursor(new_cursor, false);

    vi.mode = Mode::Normal;
    vi.visual.clear();
}

pub fn visual_yank(state: &mut TextAreaState, vi: &mut VI) {
    // undo would restore these.
    state.remove_style_fully(997);

    vi.yank.list.clear();
    for (r, _) in &vi.visual.list {
        let r = state.byte_range(r.clone());
        vi.yank.list.push(state.str_slice(r).into_owned())
    }

    vi.mode = Mode::Normal;
    vi.visual.clear();
}

pub fn visual_copy_clipboard(state: &mut TextAreaState, vi: &mut VI) {
    if state.clipboard().is_none() {
        return;
    };

    // undo would restore these.
    state.remove_style_fully(997);

    let mut buf = String::new();
    for (r, _) in &vi.visual.list {
        let r = state.byte_range(r.clone());
        buf.push_str(state.str_slice(r).as_ref());
        buf.push('\n');
    }

    let clip = state.clipboard().expect("clip");
    _ = clip.set_string(buf.as_str());

    vi.mode = Mode::Normal;
    vi.visual.clear();
}
