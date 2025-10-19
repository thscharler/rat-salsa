use crate::vi::VI;
use rat_text::text_area::TextAreaState;

pub fn scroll_to_search_idx(state: &mut TextAreaState, vi: &mut VI) {
    if let Some(idx) = vi.matches.idx {
        let pos = state.byte_pos(vi.matches.list[idx].0.start);
        state.scroll_to_pos(pos);
    }
}

pub fn scroll_cursor_to_middle(state: &mut TextAreaState) {
    let c = state.cursor();
    if let Some((_rx, ry)) = state.pos_to_relative_screen(c) {
        let noy = ry - (state.rendered.height as i16) / 2;
        if let Some(no) = state.relative_screen_to_pos((0, noy)) {
            state.set_sub_row_offset(no.x);
            state.set_offset((0, no.y));
        } else {
            // ???
        }
    } else {
        state.scroll_cursor_to_visible();
    }
}

pub fn scroll_cursor_to_bottom(state: &mut TextAreaState) {
    let c = state.cursor();
    if let Some((_rx, ry)) = state.pos_to_relative_screen(c) {
        let noy = ry - state.rendered.height.saturating_sub(1) as i16;
        if let Some(no) = state.relative_screen_to_pos((0, noy)) {
            state.set_sub_row_offset(no.x);
            state.set_offset((0, no.y));
        } else {
            // ???
        }
    } else {
        state.scroll_cursor_to_visible();
    }
}

pub fn scroll_cursor_to_top(state: &mut TextAreaState) {
    let c = state.cursor();
    if let Some((_rx, ry)) = state.pos_to_relative_screen(c) {
        if let Some(no) = state.relative_screen_to_pos((0, ry)) {
            state.set_sub_row_offset(no.x);
            state.set_offset((0, no.y));
        } else {
            // ???
        }
    } else {
        state.scroll_cursor_to_visible();
    }
}

pub fn scroll_up(mul: u32, state: &mut TextAreaState) {
    state.scroll_up(mul);
}

pub fn scroll_down(mul: u32, state: &mut TextAreaState) {
    state.scroll_down(mul);
}

pub fn scroll_page_up(mul: u32, state: &mut TextAreaState, vi: &mut VI) {
    if vi.page.0 != state.vertical_page() as u32 {
        vi.page = (state.vertical_page() as u32, (state.vertical_page() / 2));
    }

    state.scroll_up((vi.page.0 * mul).saturating_sub(2));
}

pub fn scroll_page_down(mul: u32, state: &mut TextAreaState, vi: &mut VI) {
    if vi.page.0 != state.vertical_page() as u32 {
        vi.page = (state.vertical_page() as u32, (state.vertical_page() / 2));
    }

    state.scroll_down((vi.page.0 * mul).saturating_sub(2));
}
