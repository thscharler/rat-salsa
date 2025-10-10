use crate::vi_state::op::{Vim, apply, display_search};
use crate::vi_state::query::Matches;
use crate::{Coroutine, Resume, SearchError, VIMode, YieldPoint};
use log::debug;
use rat_event::{HandleEvent, ct_event};
use rat_text::event::TextOutcome;
use rat_text::text_area::TextAreaState;
use std::cell::RefCell;
use std::rc::Rc;
use std::task::Poll;

pub struct VICmd {
    pub mode: VIMode,

    pub motion_buf: Rc<RefCell<String>>,
    pub motion: Coroutine<char, Vim>,

    pub find: Option<Vim>,
    pub smatches: Option<Matches>,
}

impl Default for VICmd {
    fn default() -> Self {
        let motion_buf = Rc::new(RefCell::new(String::new()));
        Self {
            mode: Default::default(),
            motion_buf: motion_buf.clone(),
            motion: Coroutine::new(|c, yp| Box::new(VICmd::next_motion(c, motion_buf, yp))),
            find: Default::default(),
            smatches: Default::default(),
        }
    }
}

impl VICmd {
    fn cancel(&mut self) {
        self.motion_buf.borrow_mut().clear();
        let mb = self.motion_buf.clone();
        self.motion = Coroutine::new(|c, yp| Box::new(VICmd::next_motion(c, mb, yp)));
        self.smatches = None;
        self.find = None;
    }

    fn motion(&mut self, c: char) -> Poll<Vim> {
        debug!("motion {:?} >> {:?}", c, self.motion_buf);
        match self.motion.resume(c) {
            Resume::Yield(v) => {
                debug!("    !> {:?}", v);
                Poll::Ready(v)
            }
            Resume::Done(v) => {
                debug!("    |> {:?}", v);
                self.motion_buf.borrow_mut().clear();
                let mb = self.motion_buf.clone();
                self.motion = Coroutine::new(|c, yp| Box::new(VICmd::next_motion(c, mb, yp)));
                Poll::Ready(v)
            }
            Resume::Pending => {
                debug!("    ...");
                Poll::Pending
            }
        }
    }

    async fn next_motion(
        tok: char,
        motion_buf: Rc<RefCell<String>>,
        yp: YieldPoint<char, Vim>,
    ) -> Vim {
        motion_buf.borrow_mut().push(tok);
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
                motion_buf.borrow_mut().push(tok);
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
                motion_buf.borrow_mut().push(tok);
                Vim::FindForward(tok)
            }
            'F' => {
                let tok = yp.yield0().await;
                motion_buf.borrow_mut().push(tok);
                Vim::FindBack(tok)
            }
            't' => {
                let tok = yp.yield0().await;
                motion_buf.borrow_mut().push(tok);
                Vim::FindUntilForward(tok)
            }
            'T' => {
                let tok = yp.yield0().await;
                motion_buf.borrow_mut().push(tok);
                Vim::FindUntilBack(tok)
            }
            ',' => Vim::FindRepeatNext,
            ';' => Vim::FindRepeatPrev,

            '/' => {
                let mut buf = String::new();
                loop {
                    let tok = yp.yield1(Vim::SearchPartialForward(buf.clone())).await;
                    if tok == '\n' {
                        break;
                    } else if tok == '\x08' {
                        let mut mb = motion_buf.borrow_mut();
                        if mb.len() > 1 {
                            mb.pop();
                        }
                        _ = buf.pop();
                    } else {
                        motion_buf.borrow_mut().push(tok);
                        buf.push(tok);
                    }
                }
                Vim::SearchForward(buf)
            }
            '?' => {
                let mut buf = String::new();
                loop {
                    let tok = yp.yield1(Vim::SearchPartialBack(buf.clone())).await;
                    if tok == '\n' {
                        break;
                    } else if tok == '\x08' {
                        let mut mb = motion_buf.borrow_mut();
                        if mb.len() > 1 {
                            mb.pop();
                        }
                        _ = buf.pop();
                    } else {
                        motion_buf.borrow_mut().push(tok);
                        buf.push(tok);
                    }
                }
                Vim::SearchBack(buf)
            }
            '*' => Vim::SearchWordForward,
            '#' => Vim::SearchWordBackward,
            'n' => Vim::SearchRepeatNext,
            'N' => Vim::SearchRepeatPrev,

            'i' => Vim::Insert,
            _ => Vim::Invalid,
        }
    }
}

impl HandleEvent<crossterm::event::Event, &mut VICmd, Result<TextOutcome, SearchError>>
    for TextAreaState
{
    fn handle(
        &mut self,
        event: &crossterm::event::Event,
        vi: &mut VICmd,
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
                ct_event!(keycode press Backspace) => {
                    if let Poll::Ready(vim) = vi.motion('\x08') {
                        apply(vim, self, vi)?
                    } else {
                        TextOutcome::Changed
                    }
                }

                ct_event!(keycode press Esc) | ct_event!(key press CONTROL-'c') => {
                    vi.cancel();
                    self.scroll_cursor_to_visible();
                    display_search(self, vi);
                    TextOutcome::Changed
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
    use crate::vi_state::query::*;
    use crate::vi_state::{VICmd, VIMode};
    use crate::{MoveDirection, SearchError};
    use log::debug;
    use rat_text::event::TextOutcome;
    use rat_text::text_area::TextAreaState;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Motion {
        Invalid,
    }

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

        FindForward(char),
        FindBack(char),
        FindRepeatNext,
        FindRepeatPrev,
        FindUntilForward(char),
        FindUntilBack(char),

        SearchWordForward,
        SearchWordBackward,
        SearchForward(String),
        SearchBack(String),
        SearchPartialForward(String),
        SearchPartialBack(String),
        SearchRepeatNext,
        SearchRepeatPrev,

        Insert,
    }

    impl Vim {
        pub fn search_str(&self) -> Option<&str> {
            match self {
                Vim::SearchForward(s) => Some(s.as_str()),
                Vim::SearchBack(s) => Some(s.as_str()),
                Vim::SearchPartialForward(s) => Some(s.as_str()),
                Vim::SearchPartialBack(s) => Some(s.as_str()),
                _ => None,
            }
        }
    }

    pub fn apply(
        vim: Vim,
        state: &mut TextAreaState,
        vi: &mut VICmd,
    ) -> Result<TextOutcome, SearchError> {
        let r = match vim {
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

            Vim::FindForward(c) => {
                vi.find = Some(vim.clone());
                find_next(c, state).into()
            }
            Vim::FindBack(c) => {
                vi.find = Some(vim.clone());
                find_prev(c, state).into()
            }
            Vim::FindUntilForward(c) => {
                vi.find = Some(vim.clone());
                until_next(c, state).into()
            }
            Vim::FindUntilBack(c) => {
                vi.find = Some(vim.clone());
                until_prev(c, state).into()
            }
            Vim::FindRepeatNext => match &vi.find {
                Some(Vim::FindForward(c)) => find_prev(*c, state).into(),
                Some(Vim::FindBack(c)) => find_next(*c, state).into(),
                Some(Vim::FindUntilForward(c)) => until_prev(*c, state).into(),
                Some(Vim::FindUntilBack(c)) => until_next(*c, state).into(),
                _ => TextOutcome::Unchanged,
            },
            Vim::FindRepeatPrev => match &vi.find {
                Some(Vim::FindForward(c)) => find_next(*c, state).into(),
                Some(Vim::FindBack(c)) => find_prev(*c, state).into(),
                Some(Vim::FindUntilForward(c)) => until_next(*c, state).into(),
                Some(Vim::FindUntilBack(c)) => until_prev(*c, state).into(),
                _ => TextOutcome::Unchanged,
            },

            Vim::SearchPartialForward(s) => search_fwd(s, true, state, vi)?.into(),
            Vim::SearchForward(s) => search_fwd(s, false, state, vi)?.into(),
            Vim::SearchPartialBack(s) => search_back(s, true, state, vi)?.into(),
            Vim::SearchBack(s) => search_back(s, false, state, vi)?.into(),
            Vim::SearchWordForward => search_word_fwd(state, vi)?.into(),
            Vim::SearchWordBackward => search_word_back(state, vi)?.into(),
            Vim::SearchRepeatNext => search_repeat_fwd(state, vi).into(),
            Vim::SearchRepeatPrev => search_repeat_back(state, vi).into(),

            Vim::Insert => {
                vi.mode = VIMode::Insert;
                TextOutcome::Changed
            }
        };
        debug!("apply -> {:?}", r);
        Ok(r)
    }

    pub(crate) fn search_repeat_back(state: &mut TextAreaState, vi: &mut VICmd) -> bool {
        if let Some(matches) = &mut vi.smatches {
            matches.idx = vi_search_idx(&matches, MoveDirection::Backward, state);
            display_search_idx(state, vi);
        }
        true
    }

    pub(crate) fn search_repeat_fwd(state: &mut TextAreaState, vi: &mut VICmd) -> bool {
        if let Some(matches) = &mut vi.smatches {
            matches.idx = vi_search_idx(&matches, MoveDirection::Forward, state);
            display_search_idx(state, vi);
        }
        true
    }

    pub(crate) fn search_word_back(
        state: &mut TextAreaState,
        vi: &mut VICmd,
    ) -> Result<bool, SearchError> {
        let start = vi_word_start(state);
        let end = vi_word_end(state);
        let term = state.str_slice(start..end).to_string();

        vi.smatches = Some(vi_search_update(
            vi.smatches.take(),
            term,
            MoveDirection::Backward,
            false,
            state,
        )?);
        display_search(state, vi);
        Ok(true)
    }

    pub(crate) fn search_word_fwd(
        state: &mut TextAreaState,
        vi: &mut VICmd,
    ) -> Result<bool, SearchError> {
        let start = vi_word_start(state);
        let end = vi_word_end(state);
        let term = state.str_slice(start..end).to_string();

        vi.smatches = Some(vi_search_update(
            vi.smatches.take(),
            term,
            MoveDirection::Forward,
            false,
            state,
        )?);
        display_search(state, vi);
        Ok(true)
    }

    pub(crate) fn search_back(
        term: String,
        tmp: bool,
        state: &mut TextAreaState,
        vi: &mut VICmd,
    ) -> Result<bool, SearchError> {
        vi.smatches = Some(vi_search_update(
            vi.smatches.take(),
            term,
            MoveDirection::Backward,
            tmp,
            state,
        )?);
        display_search(state, vi);
        Ok(true)
    }

    pub(crate) fn search_fwd(
        term: String,
        tmp: bool,
        state: &mut TextAreaState,
        vi: &mut VICmd,
    ) -> Result<bool, SearchError> {
        vi.smatches = Some(vi_search_update(
            vi.smatches.take(),
            term,
            MoveDirection::Forward,
            tmp,
            state,
        )?);
        display_search(state, vi);
        Ok(true)
    }

    pub(crate) fn display_search(state: &mut TextAreaState, vi: &mut VICmd) {
        state.remove_style_fully(999);
        if let Some(matches) = vi.smatches.as_ref() {
            for r in &matches.list {
                state.add_style(r.clone(), 999);
            }
        }
        display_search_idx(state, vi);
    }

    pub(crate) fn display_search_idx(state: &mut TextAreaState, vi: &mut VICmd) {
        if let Some(matches) = vi.smatches.as_ref() {
            if let Some(idx) = matches.idx {
                let pos = state.byte_pos(matches.list[idx].start);
                if matches.tmp {
                    state.scroll_to_pos(pos);
                } else {
                    state.set_cursor(pos, false);
                }
            }
        }
    }

    pub(crate) fn move_prev_paragraph(state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_prev_paragraph(state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    pub(crate) fn move_next_paragraph(state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_next_paragraph(state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    pub(crate) fn move_end_of_text(state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_end_of_text(state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    pub(crate) fn move_start_of_text(state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_start_of_text(state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    pub(crate) fn move_start_of_line(state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_start_of_line(state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    pub(crate) fn move_end_of_line(state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_end_of_line(state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    pub(crate) fn until_prev(cc: char, state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_until_prev(cc, state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    pub(crate) fn until_next(cc: char, state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_until_next(cc, state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    pub(crate) fn find_prev(cc: char, state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_find_prev(cc, state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    pub(crate) fn find_next(cc: char, state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_find_next(cc, state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    pub(crate) fn move_next_word_start(state: &mut TextAreaState) -> bool {
        if let Some(word) = vi_next_word_start(state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }

    pub(crate) fn move_prev_word_start(state: &mut TextAreaState) -> bool {
        if let Some(word) = vi_prev_word_start(state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }

    pub(crate) fn move_next_word_end(state: &mut TextAreaState) -> bool {
        if let Some(word) = vi_next_word_end(state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }

    pub(crate) fn move_prev_word_end(state: &mut TextAreaState) -> bool {
        if let Some(word) = vi_prev_word_end(state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }

    pub(crate) fn move_next_bigword_start(state: &mut TextAreaState) -> bool {
        if let Some(word) = vi_next_bigword_start(state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }

    pub(crate) fn move_prev_bigword_start(state: &mut TextAreaState) -> bool {
        if let Some(word) = vi_prev_bigword_start(state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }

    pub(crate) fn move_next_bigword_end(state: &mut TextAreaState) -> bool {
        if let Some(word) = vi_next_bigword_end(state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }

    pub(crate) fn move_prev_bigword_end(state: &mut TextAreaState) -> bool {
        if let Some(word) = vi_prev_bigword_end(state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }
}

mod query {
    use crate::{MoveDirection, SearchError};
    use log::debug;
    use rat_text::text_area::TextAreaState;
    use rat_text::{Cursor, Grapheme, TextPosition};
    use regex_cursor::engines::dfa::{Regex, find_iter};
    use regex_cursor::{Input, RopeyCursor};
    use std::ops::Range;

    #[derive(Debug)]
    pub(super) struct Matches {
        pub term: String,
        pub dir: MoveDirection,
        pub tmp: bool,
        pub idx: Option<usize>,
        pub list: Vec<Range<usize>>,
    }

    pub(super) fn vi_search_idx(
        matches: &Matches,
        dir: MoveDirection,
        state: &mut TextAreaState,
    ) -> Option<usize> {
        let c = state.byte_at(state.cursor()).start;

        let dir = matches.dir.mul(dir);

        if dir == MoveDirection::Forward {
            matches.list.iter().position(|v| v.start > c)
        } else {
            matches
                .list
                .iter()
                .enumerate()
                .filter_map(|(i, v)| if v.end < c { Some(i) } else { None })
                .last()
        }
    }

    pub(super) fn vi_search_update(
        matches: Option<Matches>,
        term: String,
        dir: MoveDirection,
        tmp: bool,
        state: &mut TextAreaState,
    ) -> Result<Matches, SearchError> {
        if let Some(mut matches) = matches {
            if matches.term != term {
                Ok(vi_search(term, dir, tmp, state)?)
            } else {
                matches.tmp = tmp;
                matches.idx = vi_search_idx(&matches, MoveDirection::Forward, state);
                Ok(matches)
            }
        } else {
            Ok(vi_search(term, dir, tmp, state)?)
        }
    }

    pub(super) fn vi_search(
        term: String,
        dir: MoveDirection,
        tmp: bool,
        state: &mut TextAreaState,
    ) -> Result<Matches, SearchError> {
        let mut matches = Matches {
            term,
            dir,
            tmp,
            idx: None,
            list: Vec::new(),
        };
        if let Ok(re) = Regex::new(&matches.term) {
            let cursor = RopeyCursor::new(state.rope().byte_slice(..));
            let input = Input::new(cursor);

            for m in find_iter(&re, input) {
                matches.list.push(m.start()..m.end());
            }

            matches.idx = vi_search_idx(&matches, MoveDirection::Forward, state);
        }

        Ok(matches)
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

    pub(super) fn vi_word_start(state: &TextAreaState) -> TextPosition {
        let mut it = state.text_graphemes(state.cursor());

        if let Some(sample) = it.peek_prev() {
            if sample.is_alphanumeric() {
                pskip_alpha(&mut it);
            } else if sample.is_whitespace() {
                // noop
            } else {
                pskip_sample(&mut it, sample);
            }
        }

        state.byte_pos(it.text_offset())
    }

    pub(super) fn vi_word_end(state: &TextAreaState) -> TextPosition {
        let mut it = state.text_graphemes(state.cursor());

        if let Some(sample) = it.peek_next() {
            if sample.is_alphanumeric() {
                skip_alpha(&mut it);
            } else if sample.is_whitespace() {
                // noop
            } else {
                skip_sample(&mut it, sample);
            }
        }

        state.byte_pos(it.text_offset())
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
