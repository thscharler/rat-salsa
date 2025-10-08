//!
//! VI Motions
//!
//! ** UNSTABLE **
//!

use crate::vi_state::op::{Vim, apply};
use crate::vi_state::token_stream::TokenStream;
use futures_util::StreamExt;
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

    pub tok_seq: String,
    pub tok: TokenStream,
    pub motion: Pin<Box<dyn Future<Output = Vim>>>,

    pub last_find: Option<Vim>,
}

impl Default for VIMotions {
    fn default() -> Self {
        let tok = TokenStream::new();
        Self {
            mode: Default::default(),
            tok_seq: Default::default(),
            tok: tok.clone(),
            motion: Box::pin(VIMotions::next_motion(tok.clone())),
            last_find: None,
        }
    }
}

impl VIMotions {
    fn motion(&mut self, c: char) -> Poll<Vim> {
        debug!("motion {:?}", c);
        self.tok_seq.push(c);
        self.tok.push_next(c);

        let mut cx = Context::from_waker(futures::task::noop_waker_ref());
        match self.motion.as_mut().poll(&mut cx) {
            Poll::Ready(v) => {
                debug!("    -> {:?}", v);
                self.tok_seq.clear();
                self.motion = Box::pin(VIMotions::next_motion(self.tok.clone()));
                Poll::Ready(v)
            }
            Poll::Pending => {
                debug!("    -> ...");
                Poll::Pending
            }
        }
    }

    async fn next_motion(mut motion: TokenStream) -> Vim {
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
                let tok = motion.next().await.expect("token");
                match tok {
                    'e' => Vim::MovePrevWordEnd,
                    'E' => Vim::MovePrevWORDEnd,
                    '_' => Vim::EndOfText,
                    _ => Vim::Invalid,
                }
            }
            'W' => Vim::MoveNextWORDStart,
            'B' => Vim::MovePrevWORDStart,
            'E' => Vim::MoveNextWORDEnd,
            'f' => {
                let tok = motion.next().await.expect("token");
                Vim::FindNext(tok)
            }
            'F' => {
                let tok = motion.next().await.expect("token");
                Vim::FindPrev(tok)
            }
            't' => {
                let tok = motion.next().await.expect("token");
                Vim::FindUntilNext(tok)
            }
            'T' => {
                let tok = motion.next().await.expect("token");
                Vim::FindUntilPrev(tok)
            }
            ',' => Vim::FindRepeatBack,
            ';' => Vim::FindRepeatFwd,
            '0' => Vim::StartOfLine,
            '^' => Vim::StartOfText,
            '$' => Vim::EndOfLine,

            'i' => Vim::Insert,
            _ => Vim::Invalid,
        }
    }
}

mod token_stream {
    use futures_util::Stream;
    use std::cell::Cell;
    use std::pin::Pin;
    use std::rc::Rc;
    use std::task::{Context, Poll};

    // Token stream for the state-machine.
    #[derive(Debug, Default, Clone)]
    pub struct TokenStream {
        pub token: Rc<Cell<Option<char>>>,
    }

    impl TokenStream {
        pub fn new() -> Self {
            Self::default()
        }

        /// Push next token.
        pub fn push_next(&mut self, token: char) {
            self.token.set(Some(token));
        }
    }

    impl Stream for TokenStream {
        type Item = char;

        fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            if let Some(token) = self.token.take() {
                Poll::Ready(Some(token))
            } else {
                Poll::Pending
            }
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
                    if let Poll::Ready(vim) = vi.motion(*c) {
                        apply(vim, self, vi)
                    } else {
                        TextOutcome::Changed
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
                | ct_event!(key press CONTROL_ALT-c) => {
                    tc(self.insert_char(*c)) //
                }
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
        MoveNextWORDStart,
        MovePrevWORDStart,
        MoveNextWORDEnd,
        MovePrevWORDEnd,

        FindNext(char),
        FindPrev(char),
        FindRepeatBack,
        FindRepeatFwd,
        FindUntilNext(char),
        FindUntilPrev(char),

        StartOfLine,
        EndOfLine,
        StartOfText,
        EndOfText,

        Insert,
    }

    pub fn apply(vim: Vim, state: &mut TextAreaState, vi: &mut VIMotions) -> TextOutcome {
        match vim {
            Vim::Invalid => TextOutcome::Continue,
            Vim::MoveLeft => state.move_left(1, false).into(),
            Vim::MoveRight => state.move_right(1, false).into(),
            Vim::MoveUp => state.move_up(1, false).into(),
            Vim::MoveDown => state.move_down(1, false).into(),
            Vim::MoveNextWordStart => move_next_word_start(state).into(),
            Vim::MovePrevWordStart => move_prev_word_start(state).into(),
            Vim::MoveNextWordEnd => move_next_word_end(state).into(),
            Vim::MovePrevWordEnd => move_prev_word_end(state).into(),
            Vim::MoveNextWORDStart => move_next_bigword_start(state).into(),
            Vim::MovePrevWORDStart => move_prev_bigword_start(state).into(),
            Vim::MoveNextWORDEnd => move_next_bigword_end(state).into(),
            Vim::MovePrevWORDEnd => move_prev_bigword_end(state).into(),
            Vim::Insert => {
                vi.mode = VIMode::Insert;
                TextOutcome::Changed
            }
            Vim::FindNext(c) => {
                vi.last_find = Some(vim);
                find_next(c, state).into()
            }
            Vim::FindPrev(c) => {
                vi.last_find = Some(vim);
                find_prev(c, state).into()
            }
            Vim::FindUntilNext(c) => {
                vi.last_find = Some(vim);
                until_next(c, state).into()
            }
            Vim::FindUntilPrev(c) => {
                vi.last_find = Some(vim);
                until_prev(c, state).into()
            }
            Vim::FindRepeatBack => match vi.last_find {
                Some(Vim::FindNext(c)) => find_prev(c, state).into(),
                Some(Vim::FindPrev(c)) => find_next(c, state).into(),
                Some(Vim::FindUntilNext(c)) => until_prev(c, state).into(),
                Some(Vim::FindUntilPrev(c)) => until_next(c, state).into(),
                _ => TextOutcome::Unchanged,
            },
            Vim::FindRepeatFwd => match vi.last_find {
                Some(Vim::FindNext(c)) => find_next(c, state).into(),
                Some(Vim::FindPrev(c)) => find_prev(c, state).into(),
                Some(Vim::FindUntilNext(c)) => until_next(c, state).into(),
                Some(Vim::FindUntilPrev(c)) => until_prev(c, state).into(),
                _ => TextOutcome::Unchanged,
            },
            Vim::StartOfLine => move_start_of_line(state).into(),
            Vim::EndOfLine => move_end_of_line(state).into(),
            Vim::StartOfText => move_start_of_text(state).into(),
            Vim::EndOfText => move_end_of_text(state).into(),
        }
    }

    fn move_end_of_text(state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_end_of_text(state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    fn move_start_of_text(state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_start_of_text(state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    fn move_start_of_line(state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_start_of_line(state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    fn move_end_of_line(state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_end_of_line(state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    fn until_prev(cc: char, state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_until_prev(cc, state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    fn until_next(cc: char, state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_until_next(cc, state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    fn find_prev(cc: char, state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_find_prev(cc, state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    fn find_next(cc: char, state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_find_next(cc, state) {
            state.set_cursor(cursor, false)
        } else {
            false
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

    fn move_next_bigword_start(state: &mut TextAreaState) -> bool {
        if let Some(word) = vi_next_bigword_start(state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }

    fn move_prev_bigword_start(state: &mut TextAreaState) -> bool {
        if let Some(word) = vi_prev_bigword_start(state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }

    fn move_next_bigword_end(state: &mut TextAreaState) -> bool {
        if let Some(word) = vi_next_bigword_end(state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }

    fn move_prev_bigword_end(state: &mut TextAreaState) -> bool {
        if let Some(word) = vi_prev_bigword_end(state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }
}

mod query {
    use rat_text::text_area::TextAreaState;
    use rat_text::{Cursor, Grapheme, TextPosition, TextRange};

    pub(super) fn vi_end_of_text(state: &mut TextAreaState) -> Option<TextPosition> {
        let y = state.cursor().y;
        let width = state.line_width(y);
        let mut it = state.graphemes(
            (TextPosition::new(0, y)..TextPosition::new(width, y)).into(),
            TextPosition::new(width, y),
        );
        let found;
        loop {
            let Some(c) = it.prev() else {
                return None;
            };
            if !c.is_whitespace() {
                found = c.text_bytes().end;
                break;
            }
        }

        Some(state.byte_pos(found))
    }

    pub(super) fn vi_start_of_text(state: &mut TextAreaState) -> Option<TextPosition> {
        let mut it = state.line_graphemes(state.cursor().y);
        let found;
        loop {
            let Some(c) = it.next() else {
                return None;
            };
            if !c.is_whitespace() {
                found = c.text_bytes().start;
                break;
            }
        }

        Some(state.byte_pos(found))
    }

    pub(super) fn vi_start_of_line(state: &mut TextAreaState) -> Option<TextPosition> {
        Some(TextPosition::new(0, state.cursor().y))
    }

    pub(super) fn vi_end_of_line(state: &mut TextAreaState) -> Option<TextPosition> {
        let cursor = state.cursor();
        Some(TextPosition::new(state.line_width(cursor.y), cursor.y))
    }

    pub(super) fn vi_until_prev(cc: char, state: &mut TextAreaState) -> Option<TextPosition> {
        let mut it = state.line_graphemes(state.cursor().y);
        if let Some(c) = it.peek_prev() {
            if c == cc {
                it.prev();
            }
        }
        let found;
        loop {
            let Some(c) = it.prev() else { return None };

            if c.is_line_break() {
                return None;
            } else if c == cc {
                found = c.text_bytes().end;
                break;
            }
        }

        Some(state.byte_pos(found))
    }

    pub(super) fn vi_until_next(cc: char, state: &mut TextAreaState) -> Option<TextPosition> {
        let mut it = state.line_graphemes(state.cursor().y);
        if let Some(c) = it.peek_next() {
            if c == cc {
                it.next();
            }
        }
        let found;
        loop {
            let Some(c) = it.next() else { return None };

            if c.is_line_break() {
                return None;
            } else if c == cc {
                found = c.text_bytes().start;
                break;
            }
        }

        Some(state.byte_pos(found))
    }

    pub(super) fn vi_find_prev(cc: char, state: &mut TextAreaState) -> Option<TextPosition> {
        let mut it = state.line_graphemes(state.cursor().y);
        let found;
        loop {
            let Some(c) = it.prev() else { return None };

            if c.is_line_break() {
                return None;
            } else if c == cc {
                found = c.text_bytes().start;
                break;
            }
        }

        Some(state.byte_pos(found))
    }

    pub(super) fn vi_find_next(cc: char, state: &mut TextAreaState) -> Option<TextPosition> {
        let mut it = state.line_graphemes(state.cursor().y);
        if let Some(c) = it.peek_next() {
            if c == cc {
                it.next();
            }
        }
        let found;
        loop {
            let Some(c) = it.next() else { return None };

            if c.is_line_break() {
                return None;
            } else if c == cc {
                found = c.text_bytes().start;
                break;
            }
        }

        Some(state.byte_pos(found))
    }

    pub(super) fn vi_prev_bigword_start(state: &TextAreaState) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        let Some(sample) = it.peek_prev() else {
            return None;
        };
        if !sample.is_whitespace() {
            pskip_nonwhite(&mut it);
        } else {
            pskip_white(&mut it);
            pskip_nonwhite(&mut it);
        }

        Some(state.byte_pos(it.text_offset()))
    }

    pub(super) fn vi_next_bigword_start(state: &TextAreaState) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        let Some(sample) = it.peek_next() else {
            return None;
        };
        if !sample.is_whitespace() {
            skip_nonwhite(&mut it);
        }
        skip_white(&mut it);

        Some(state.byte_pos(it.text_offset()))
    }

    pub(super) fn vi_prev_bigword_end(state: &TextAreaState) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        let Some(sample) = it.peek_prev() else {
            return None;
        };
        if !sample.is_whitespace() {
            pskip_nonwhite(&mut it);
        }
        pskip_white(&mut it);

        Some(state.byte_pos(it.text_offset()))
    }

    pub(super) fn vi_next_bigword_end(state: &TextAreaState) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        skip_white(&mut it);

        let Some(sample) = it.peek_next() else {
            return None;
        };
        if !sample.is_whitespace() {
            skip_nonwhite(&mut it);
        }

        Some(state.byte_pos(it.text_offset()))
    }

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

        let Some(sample) = it.peek_prev() else {
            return None;
        };
        if sample.is_alphanumeric() {
            pskip_alpha(&mut it);
        } else if sample.is_whitespace() {
            pskip_white(&mut it);
            let Some(sample) = it.peek_prev() else {
                return None;
            };
            if sample.is_alphanumeric() {
                pskip_alpha(&mut it);
            } else {
                pskip_sample(&mut it, sample);
            }
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

    fn pskip_nonwhite(it: &mut dyn Cursor<Item = Grapheme>) {
        loop {
            let Some(c) = it.prev() else {
                break;
            };
            if c.is_whitespace() {
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

    fn skip_nonwhite(it: &mut dyn Cursor<Item = Grapheme>) {
        loop {
            let Some(c) = it.next() else {
                break;
            };
            if c.is_whitespace() {
                it.prev();
                break;
            }
        }
    }
}
