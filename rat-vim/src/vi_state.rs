//!
//! VI Motions
//!
//! ** UNSTABLE **
//!
use rat_event::{HandleEvent, ct_event};
use rat_text::event::TextOutcome;
use rat_text::text_area::TextAreaState;
use rat_text::{Cursor, Grapheme, TextPosition};

#[derive(Default, Debug, PartialEq, Eq)]
pub enum VIMode {
    #[default]
    Normal,
    Insert,
    Visual,
}

#[derive(Default, Debug)]
pub struct VIMotion {
    pub mode: VIMode,
    pub cmd: String,
}

impl HandleEvent<crossterm::event::Event, &mut VIMotion, TextOutcome> for TextAreaState {
    fn handle(&mut self, event: &crossterm::event::Event, vi: &mut VIMotion) -> TextOutcome {
        let r = if vi.mode == VIMode::Normal {
            match event {
                ct_event!(key press c)
                | ct_event!(key press SHIFT-c)
                | ct_event!(key press CONTROL_ALT-c) => {
                    vi.cmd.push(*c);
                }
                ct_event!(keycode press Esc) | ct_event!(key press CONTROL-'c') => {
                    vi.cmd.clear();
                }
                _ => {}
            }

            if !vi.cmd.is_empty() {
                let mut rec = true;
                let change = match vi.cmd.as_str() {
                    "h" => self.move_left(1, false),
                    "l" => self.move_right(1, false),
                    "k" => self.move_up(1, false),
                    "j" => self.move_down(1, false),
                    "w" => move_next_word_start(self),
                    "b" => move_prev_word_start(self),
                    "e" => move_next_word_end(self),
                    "ge" => move_prev_word_end(self),
                    "i" => {
                        vi.mode = VIMode::Insert;
                        false
                    }
                    _ => {
                        rec = false;
                        false
                    }
                };
                if rec {
                    vi.cmd.clear();
                }
                change.into()
            } else {
                TextOutcome::Continue
            }
        } else if vi.mode == VIMode::Insert {
            match event {
                ct_event!(keycode press Esc) | ct_event!(key press CONTROL-'c') => {
                    vi.mode = VIMode::Normal;
                    TextOutcome::Changed
                }

                ct_event!(key press c)
                | ct_event!(key press SHIFT-c)
                | ct_event!(key press CONTROL_ALT-c) => tc(self.insert_char(*c)),
                ct_event!(keycode press Tab) => {
                    // ignore tab from focus
                    if !self.focus.gained() {
                        tc(self.insert_tab())
                    } else {
                        TextOutcome::Unchanged
                    }
                }
                ct_event!(keycode press Enter) => tc(self.insert_newline()),
                ct_event!(keycode press Backspace) => tc(self.delete_prev_char()),
                ct_event!(keycode press Delete) => tc(self.delete_next_char()),

                _ => TextOutcome::Continue,
            }
        } else if vi.mode == VIMode::Visual {
            TextOutcome::Continue
        } else {
            TextOutcome::Continue
        };

        r
    }
}

// small helper ...
fn tc(r: bool) -> TextOutcome {
    if r {
        TextOutcome::TextChanged
    } else {
        TextOutcome::Unchanged
    }
}

fn move_next_word_start(state: &mut TextAreaState) -> bool {
    if let Some(word) = vi_next_word_start(state) {
        state.set_cursor(word, false)
    } else {
        false
    }
}

fn move_prev_word_start(state: &mut TextAreaState) -> bool {
    if let Some(word) = vi_prev_word_start(state) {
        state.set_cursor(word, false)
    } else {
        false
    }
}

fn move_next_word_end(state: &mut TextAreaState) -> bool {
    if let Some(word) = vi_next_word_end(state) {
        state.set_cursor(word, false)
    } else {
        false
    }
}

fn move_prev_word_end(state: &mut TextAreaState) -> bool {
    if let Some(word) = vi_prev_word_end(state) {
        state.set_cursor(word, false)
    } else {
        false
    }
}

fn vi_next_word_end(state: &TextAreaState) -> Option<TextPosition> {
    let mut it = state.text_graphemes(state.cursor());

    skip_white(&mut it);

    let Some(sample) = it.peek_next() else {
        return None;
    };
    if sample.is_alphanumeric() {
        skip_alpha(&mut it);
    } else {
        skip_sample(&mut it, sample);
    }

    Some(state.byte_pos(it.text_offset()))
}

fn vi_prev_word_end(state: &TextAreaState) -> Option<TextPosition> {
    let mut it = state.text_graphemes(state.cursor());

    let Some(sample) = it.peek_prev() else {
        return None;
    };
    if sample.is_alphanumeric() {
        pskip_alpha(&mut it);
    } else if sample.is_whitespace() {
        // noop
    } else {
        pskip_sample(&mut it, sample);
    }

    pskip_white(&mut it);

    Some(state.byte_pos(it.text_offset()))
}

fn vi_next_word_start(state: &TextAreaState) -> Option<TextPosition> {
    let mut it = state.text_graphemes(state.cursor());

    let Some(sample) = it.peek_next() else {
        return None;
    };
    if sample.is_alphanumeric() {
        skip_alpha(&mut it);
    } else if sample.is_whitespace() {
        // noop
    } else {
        skip_sample(&mut it, sample);
    }

    skip_white(&mut it);

    Some(state.byte_pos(it.text_offset()))
}

fn vi_prev_word_start(state: &TextAreaState) -> Option<TextPosition> {
    let mut it = state.text_graphemes(state.cursor());

    let Some(sample) = it.peek_next() else {
        return None;
    };
    if sample.is_alphanumeric() {
        pskip_alpha(&mut it);
        pskip_white(&mut it);
    } else if sample.is_whitespace() {
        pskip_white(&mut it);
    } else {
        pskip_sample(&mut it, sample);
        pskip_white(&mut it);
    }

    let Some(sample) = it.peek_next() else {
        return None;
    };
    if sample.is_alphanumeric() {
        pskip_alpha(&mut it);
    } else {
        pskip_sample(&mut it, sample);
    }

    Some(state.byte_pos(it.text_offset()))
}

fn pskip_alpha(it: &mut dyn Cursor<Item = Grapheme>) {
    loop {
        let Some(c) = it.prev() else {
            break;
        };
        if !c.is_alphanumeric() {
            it.next();
            break;
        }
    }
}

fn pskip_sample(it: &mut dyn Cursor<Item = Grapheme>, sample: Grapheme) {
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

fn pskip_white(it: &mut dyn Cursor<Item = Grapheme>) {
    loop {
        let Some(c) = it.prev() else {
            break;
        };
        if !c.is_whitespace() {
            it.next();
            break;
        }
    }
}

fn skip_alpha(it: &mut dyn Cursor<Item = Grapheme>) {
    loop {
        let Some(c) = it.next() else {
            break;
        };
        if !c.is_alphanumeric() {
            it.prev();
            break;
        }
    }
}

fn skip_sample(it: &mut dyn Cursor<Item = Grapheme>, sample: Grapheme) {
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

fn skip_white(it: &mut dyn Cursor<Item = Grapheme>) {
    loop {
        let Some(c) = it.next() else {
            break;
        };
        if !c.is_whitespace() {
            it.prev();
            break;
        }
    }
}
