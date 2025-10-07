//!
//! VI Motions
//!
//! ** UNSTABLE **
//!

use crate::vi_state::motions::{Token, next_motion};
use crate::vi_state::op::Vim;
use log::debug;
use rat_event::{HandleEvent, ct_event};
use rat_text::event::TextOutcome;
use rat_text::text_area::TextAreaState;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Default, Debug, PartialEq, Eq)]
pub enum VIMode {
    #[default]
    Normal,
    Insert,
    Visual,
}

pub struct VIMotions {
    pub mode: VIMode,
    pub tok: Token,
    pub motion: Pin<Box<dyn Future<Output = Vim>>>,
}

impl Default for VIMotions {
    fn default() -> Self {
        let tok = Token::new();
        Self {
            mode: Default::default(),
            tok: tok.clone(),
            motion: Box::pin(next_motion(tok.clone())),
        }
    }
}

impl VIMotions {
    fn motion(&mut self, c: char) -> Poll<Vim> {
        self.tok.push(c);

        let mut cx = Context::from_waker(futures::task::noop_waker_ref());
        match self.motion.as_mut().poll(&mut cx) {
            Poll::Ready(v) => {
                self.tok.clear();
                self.motion = Box::pin(next_motion(self.tok.clone()));
                Poll::Ready(v)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

mod motions {
    use crate::vi_state::op::*;
    use futures_util::{Stream, StreamExt};
    use log::debug;
    use std::cell::Cell;
    use std::pin::Pin;
    use std::rc::Rc;
    use std::task::{Context, Poll};

    #[derive(Debug, Default, Clone)]
    pub struct Token {
        pub display: String,
        pub token: Rc<Cell<Option<char>>>,
    }

    impl Token {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn push(&mut self, token: char) {
            self.display.push(token);
            self.token.set(Some(token));
        }

        pub fn clear(&mut self) {
            self.display.clear();
            self.token.set(None);
        }
    }

    impl Stream for Token {
        type Item = char;

        fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            if let Some(token) = self.token.take() {
                Poll::Ready(Some(token))
            } else {
                Poll::Pending
            }
        }
    }

    pub async fn next_motion(mut motion: Token) -> Vim {
        let tok = motion.next().await.expect("token");
        match tok {
            'h' => Vim::MoveLeft,
            'l' => Vim::MoveRight,
            'k' => Vim::MoveUp,
            'j' => Vim::MoveDown,
            'w' => Vim::MoveNextWordStart,
            'b' => Vim::MovePrevWordStart,
            'e' => Vim::MoveNextWordEnd,
            'g' => {
                debug!("enter g");
                let tok = motion.next().await.expect("token");
                debug!("restart g");
                match tok {
                    'e' => Vim::MovePrevWordEnd,
                    _ => Vim::Invalid,
                }
            }
            'i' => Vim::Insert,
            _ => Vim::Invalid,
        }
    }
}

impl HandleEvent<crossterm::event::Event, &mut VIMotions, TextOutcome> for TextAreaState {
    fn handle(&mut self, event: &crossterm::event::Event, vi: &mut VIMotions) -> TextOutcome {
        let r = if vi.mode == VIMode::Normal {
            match event {
                ct_event!(key press c)
                | ct_event!(key press SHIFT-c)
                | ct_event!(key press CONTROL_ALT-c) => {
                    debug!("motion {:?}", *c);
                    if let Poll::Ready(vim) = vi.motion(*c) {
                        debug!("-> {:?}", vim);
                        vim.apply(self, vi)
                    } else {
                        debug!("-> ...");
                        TextOutcome::Unchanged
                    }
                }
                ct_event!(keycode press Esc) | ct_event!(key press CONTROL-'c') => {
                    TextOutcome::Unchanged
                }
                _ => TextOutcome::Continue,
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

mod op {
    use crate::vi_state::query::*;
    use crate::vi_state::{VIMode, VIMotions};
    use rat_text::event::TextOutcome;
    use rat_text::text_area::TextAreaState;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Vim {
        Invalid,
        MoveLeft,
        MoveRight,
        MoveUp,
        MoveDown,
        MoveNextWordStart,
        MovePrevWordStart,
        MoveNextWordEnd,
        MovePrevWordEnd,
        Insert,
    }

    impl Vim {
        pub fn apply(self, state: &mut TextAreaState, vi: &mut VIMotions) -> TextOutcome {
            match self {
                Vim::Invalid => TextOutcome::Continue,
                Vim::MoveLeft => state.move_left(1, false).into(),
                Vim::MoveRight => state.move_right(1, false).into(),
                Vim::MoveUp => state.move_up(1, false).into(),
                Vim::MoveDown => state.move_down(1, false).into(),
                Vim::MoveNextWordStart => move_next_word_start(state).into(),
                Vim::MovePrevWordStart => move_prev_word_start(state).into(),
                Vim::MoveNextWordEnd => move_next_word_end(state).into(),
                Vim::MovePrevWordEnd => move_prev_word_end(state).into(),
                Vim::Insert => {
                    vi.mode = VIMode::Insert;
                    TextOutcome::Changed
                }
            }
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
}

mod query {
    use rat_text::text_area::TextAreaState;
    use rat_text::{Cursor, Grapheme, TextPosition};

    pub(super) fn vi_next_word_end(state: &TextAreaState) -> Option<TextPosition> {
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

    pub(super) fn vi_prev_word_end(state: &TextAreaState) -> Option<TextPosition> {
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

    pub(super) fn vi_next_word_start(state: &TextAreaState) -> Option<TextPosition> {
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

    pub(super) fn vi_prev_word_start(state: &TextAreaState) -> Option<TextPosition> {
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
}
