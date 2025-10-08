use crate::vi_state::op::{Vim, apply};
use crate::{Coroutine, Resume, SearchError, VIMode, YieldPoint};
use log::debug;
use rat_event::{HandleEvent, ct_event};
use rat_text::event::TextOutcome;
use rat_text::text_area::TextAreaState;
use std::task::Poll;

pub struct VIMotions {
    pub mode: VIMode,

    pub tok_seq: String,
    pub motion: Coroutine<char, Vim>,

    pub last_find: Option<Vim>,
}

impl Default for VIMotions {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            tok_seq: Default::default(),
            motion: Coroutine::new(|c, yp| Box::new(VIMotions::next_motion(c, yp))),
            last_find: None,
        }
    }
}

impl VIMotions {
    fn motion(&mut self, c: char) -> Poll<Vim> {
        self.tok_seq.push(c);

        debug!("motion {:?}", c);
        let r = if self.motion.start_pending() {
            self.motion.start(c)
        } else {
            self.motion.resume(c)
        };

        match r {
            Resume::Yield(v) => {
                debug!("    !> {:?}", v);
                Poll::Ready(v)
            }
            Resume::Done(v) => {
                debug!("    |> {:?}", v);
                self.tok_seq.clear();
                self.motion = Coroutine::new(|c, yp| Box::new(VIMotions::next_motion(c, yp)));
                Poll::Ready(v)
            }
            Resume::Pending => {
                debug!("    ...");
                Poll::Pending
            }
        }
    }

    async fn next_motion(tok: char, yp: YieldPoint<char, Vim>) -> Vim {
        match tok {
            'h' => Vim::MoveLeft,
            'l' => Vim::MoveRight,
            'k' => Vim::MoveUp,
            'j' => Vim::MoveDown,
            'w' => Vim::MoveNextWordStart,
            'b' => Vim::MovePrevWordStart,
            'e' => Vim::MoveNextWordEnd,
            'g' => {
                let tok = yp.yield0().await;
                match tok {
                    'e' => Vim::MovePrevWordEnd,
                    'E' => Vim::MovePrevWORDEnd,
                    '_' => Vim::MoveEndOfText,
                    _ => Vim::Invalid,
                }
            }
            'W' => Vim::MoveNextWORDStart,
            'B' => Vim::MovePrevWORDStart,
            'E' => Vim::MoveNextWORDEnd,
            '0' => Vim::MoveStartOfLine,
            '^' => Vim::MoveStartOfText,
            '$' => Vim::MoveEndOfLine,
            '{' => Vim::MovePrevParagraph,
            '}' => Vim::MoveNextParagraph,

            'f' => {
                let tok = yp.yield0().await;
                Vim::FindNext(tok)
            }
            'F' => {
                let tok = yp.yield0().await;
                Vim::FindPrev(tok)
            }
            't' => {
                let tok = yp.yield0().await;
                Vim::FindUntilNext(tok)
            }
            'T' => {
                let tok = yp.yield0().await;
                Vim::FindUntilPrev(tok)
            }
            ',' => Vim::FindRepeatBack,
            ';' => Vim::FindRepeatFwd,

            '/' => {
                let mut buf = String::new();
                loop {
                    let tok = yp.yield1(Vim::SearchPartialForward(buf.clone())).await;
                    if tok == '\n' {
                        break;
                    } else if tok == '\u{0008}' {
                        _ = buf.pop();
                    } else {
                        buf.push(tok);
                    }
                }
                Vim::SearchForward(buf)
            }
            '?' => {
                let mut buf = String::new();
                loop {
                    let tok = yp.yield1(Vim::SearchPartialBackward(buf.clone())).await;
                    if tok == '\n' {
                        break;
                    } else if tok == '\u{0008}' {
                        _ = buf.pop();
                    } else {
                        buf.push(tok);
                    }
                }
                Vim::SearchBackward(buf)
            }
            'i' => Vim::Insert,
            _ => Vim::Invalid,
        }
    }
}

impl HandleEvent<crossterm::event::Event, &mut VIMotions, Result<TextOutcome, SearchError>>
    for TextAreaState
{
    fn handle(
        &mut self,
        event: &crossterm::event::Event,
        vi: &mut VIMotions,
    ) -> Result<TextOutcome, SearchError> {
        let r = if vi.mode == VIMode::Normal {
            match event {
                ct_event!(key press c)
                | ct_event!(key press SHIFT-c)
                | ct_event!(key press CONTROL_ALT-c) => {
                    if let Poll::Ready(vim) = vi.motion(*c) {
                        apply(vim, self, vi)?
                    } else {
                        TextOutcome::Changed
                    }
                }
                ct_event!(keycode press Enter) => {
                    if let Poll::Ready(vim) = vi.motion('\n') {
                        apply(vim, self, vi)?
                    } else {
                        TextOutcome::Changed
                    }
                }

                ct_event!(keycode press Esc) | ct_event!(key press CONTROL-'c') => {
                    TextOutcome::Unchanged
                }
                ct_event!(key press CONTROL-'d') => self
                    .move_down(self.vertical_page() as u16 / 2, false)
                    .into(),
                ct_event!(key press CONTROL-'u') => {
                    self.move_up(self.vertical_page() as u16 / 2, false).into()
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

        Ok(r)
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
    use crate::SearchError;
    use crate::vi_state::query::*;
    use crate::vi_state::{VIMode, VIMotions};
    use rat_text::event::TextOutcome;
    use rat_text::text_area::TextAreaState;

    #[derive(Debug, Clone, PartialEq, Eq)]
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
        MoveStartOfLine,
        MoveEndOfLine,
        MoveStartOfText,
        MoveEndOfText,
        MovePrevParagraph,
        MoveNextParagraph,

        FindNext(char),
        FindPrev(char),
        FindRepeatBack,
        FindRepeatFwd,
        FindUntilNext(char),
        FindUntilPrev(char),

        Insert,

        SearchForward(String),
        SearchBackward(String),
        SearchPartialForward(String),
        SearchPartialBackward(String),
    }

    pub fn apply(
        vim: Vim,
        state: &mut TextAreaState,
        vi: &mut VIMotions,
    ) -> Result<TextOutcome, SearchError> {
        let r = match &vim {
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
            Vim::MoveStartOfLine => move_start_of_line(state).into(),
            Vim::MoveEndOfLine => move_end_of_line(state).into(),
            Vim::MoveStartOfText => move_start_of_text(state).into(),
            Vim::MoveEndOfText => move_end_of_text(state).into(),
            Vim::MovePrevParagraph => move_prev_paragraph(state).into(),
            Vim::MoveNextParagraph => move_next_paragraph(state).into(),

            Vim::Insert => {
                vi.mode = VIMode::Insert;
                TextOutcome::Changed
            }

            Vim::FindNext(c) => {
                vi.last_find = Some(vim.clone());
                find_next(*c, state).into()
            }
            Vim::FindPrev(c) => {
                vi.last_find = Some(vim.clone());
                find_prev(*c, state).into()
            }
            Vim::FindUntilNext(c) => {
                vi.last_find = Some(vim.clone());
                until_next(*c, state).into()
            }
            Vim::FindUntilPrev(c) => {
                vi.last_find = Some(vim.clone());
                until_prev(*c, state).into()
            }
            Vim::FindRepeatBack => match &vi.last_find {
                Some(Vim::FindNext(c)) => find_prev(*c, state).into(),
                Some(Vim::FindPrev(c)) => find_next(*c, state).into(),
                Some(Vim::FindUntilNext(c)) => until_prev(*c, state).into(),
                Some(Vim::FindUntilPrev(c)) => until_next(*c, state).into(),
                _ => TextOutcome::Unchanged,
            },
            Vim::FindRepeatFwd => match &vi.last_find {
                Some(Vim::FindNext(c)) => find_next(*c, state).into(),
                Some(Vim::FindPrev(c)) => find_prev(*c, state).into(),
                Some(Vim::FindUntilNext(c)) => until_next(*c, state).into(),
                Some(Vim::FindUntilPrev(c)) => until_prev(*c, state).into(),
                _ => TextOutcome::Unchanged,
            },

            Vim::SearchPartialForward(_) => TextOutcome::Changed,
            Vim::SearchForward(s) => search_forward(s.as_str(), state)?.into(),
            Vim::SearchPartialBackward(_) => TextOutcome::Changed,
            Vim::SearchBackward(s) => search_backward(s.as_str(), state)?.into(),
        };

        Ok(r)
    }

    fn search_forward(s: &str, state: &mut TextAreaState) -> Result<bool, SearchError> {
        if let Some(cursor) = vi_search_forward(s, state)? {
            Ok(state.set_cursor(cursor, false))
        } else {
            Ok(false)
        }
    }

    fn search_backward(s: &str, state: &mut TextAreaState) -> Result<bool, SearchError> {
        if let Some(cursor) = vi_search_backward(s, state)? {
            Ok(state.set_cursor(cursor, false))
        } else {
            Ok(false)
        }
    }

    fn move_prev_paragraph(state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_prev_paragraph(state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    fn move_next_paragraph(state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_next_paragraph(state) {
            state.set_cursor(cursor, false)
        } else {
            false
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
    use crate::SearchError;
    use rat_text::text_area::TextAreaState;
    use rat_text::{Cursor, Grapheme, TextPosition};
    use regex_cursor::engines::dfa::{try_search_fwd, try_search_rev};
    use regex_cursor::regex_automata::dfa::dense::DFA;
    use regex_cursor::regex_automata::nfa::thompson;
    use regex_cursor::{Input, RopeyCursor};

    pub(super) fn vi_search_forward(
        search: &str,
        state: &mut TextAreaState,
    ) -> Result<Option<TextPosition>, SearchError> {
        let pos = state.byte_at(state.cursor());
        let cursor = RopeyCursor::new(state.rope().byte_slice(..));

        let mut input = Input::new(cursor);
        input.set_start(pos.start);

        let dfa = DFA::builder()
            .thompson(thompson::Config::new())
            .build(search)?;

        let found;
        match try_search_fwd(&dfa, &mut input)? {
            None => return Ok(None),
            Some(hm) => {
                found = hm.offset();

                if hm.offset() > input.start() {
                    input.set_start(hm.offset());
                } else {
                    // This is only necessary to handle zero-width
                    // matches, which of course occur in this example.
                    // Without this, the search would never advance
                    // backwards beyond the initial match.
                    input.set_start(input.start() + 1);
                }
            }
        }

        Ok(Some(state.byte_pos(found)))
    }

    pub(super) fn vi_search_backward(
        search: &str,
        state: &mut TextAreaState,
    ) -> Result<Option<TextPosition>, SearchError> {
        let pos = state.byte_at(state.cursor());
        let cursor = RopeyCursor::new(state.rope().byte_slice(..));

        let mut input = Input::new(cursor);
        input.set_end(pos.start);

        let dfa = DFA::builder()
            .thompson(thompson::Config::new().reverse(true))
            .build(search)?;

        let found;
        match try_search_rev(&dfa, &mut input)? {
            None => return Ok(None),
            Some(hm) => {
                found = hm.offset();

                if hm.offset() < input.end() {
                    input.set_end(hm.offset());
                } else {
                    // This is only necessary to handle zero-width
                    // matches, which of course occur in this example.
                    // Without this, the search would never advance
                    // backwards beyond the initial match.
                    input.set_end(input.end() - 1);
                }
            }
        }

        Ok(Some(state.byte_pos(found)))
    }

    pub(super) fn vi_next_paragraph(state: &mut TextAreaState) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        if let Some(c) = it.peek_next()
            && c.is_whitespace()
        {
            loop {
                let Some(c) = it.next() else {
                    return None;
                };
                if !c.is_whitespace() {
                    break;
                }
            }
        }

        let found;
        let mut brk = false;
        loop {
            let Some(c) = it.next() else {
                found = it.text_offset();
                break;
            };

            if c.is_line_break() {
                if !brk {
                    brk = true;
                } else {
                    found = c.text_bytes().start;
                    break;
                }
            } else if !c.is_whitespace() {
                brk = false;
            }
        }

        Some(state.byte_pos(found))
    }

    pub(super) fn vi_prev_paragraph(state: &mut TextAreaState) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        if let Some(c) = it.peek_prev()
            && c.is_whitespace()
        {
            loop {
                let Some(c) = it.prev() else {
                    return None;
                };
                if !c.is_whitespace() {
                    break;
                }
            }
        }

        let found;
        let mut brk = false;
        loop {
            let Some(c) = it.prev() else {
                found = it.text_offset();
                break;
            };

            if c.is_line_break() {
                if !brk {
                    brk = true;
                } else {
                    found = c.text_bytes().end;
                    break;
                }
            } else if !c.is_whitespace() {
                brk = false;
            }
        }

        Some(state.byte_pos(found))
    }

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
        let mut it = state.text_graphemes(state.cursor());
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
        let mut it = state.text_graphemes(state.cursor());
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
        let mut it = state.text_graphemes(state.cursor());
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
        let mut it = state.text_graphemes(state.cursor());
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
