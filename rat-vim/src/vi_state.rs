use crate::vi_state::op::{apply, display_search};
use crate::{Coroutine, Resume, SearchError, YieldPoint};
use log::debug;
use rat_event::{HandleEvent, ct_event};
use rat_text::event::TextOutcome;
use rat_text::text_area::TextAreaState;
use rat_text::upos_type;
use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;
use std::task::Poll;

pub struct VI {
    pub mode: Mode,

    pub motion_buf: Rc<RefCell<String>>,
    pub motion: Coroutine<char, Vim>,

    pub finds: Finds,
    pub matches: Matches,
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    Visual,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    #[default]
    Forward,
    Backward,
}

impl Direction {
    pub fn mul(self, d: Direction) -> Direction {
        match (self, d) {
            (Direction::Forward, Direction::Forward) => Direction::Forward,
            (Direction::Forward, Direction::Backward) => Direction::Backward,
            (Direction::Backward, Direction::Forward) => Direction::Backward,
            (Direction::Backward, Direction::Backward) => Direction::Forward,
        }
    }
}

#[derive(Debug, Default)]
pub struct Finds {
    pub term: Option<char>,
    pub row: upos_type,
    pub dir: Direction,
    pub till: bool,
    pub idx: Option<usize>,
    pub list: Vec<Range<usize>>,
}

impl Finds {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.term = Default::default();
        self.row = Default::default();
        self.dir = Default::default();
        self.till = Default::default();
        self.idx = Default::default();
        self.list.clear();
    }
}

#[derive(Debug, Default)]
pub struct Matches {
    pub term: Option<String>,
    pub dir: Direction,
    pub tmp: bool,
    pub idx: Option<usize>,
    pub list: Vec<Range<usize>>,
}

impl Matches {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.term = Default::default();
        self.dir = Default::default();
        self.tmp = Default::default();
        self.idx = Default::default();
        self.list.clear();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Motion {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveToCol,

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
    MoveStartOfLineText,
    MoveEndOfLineText,
    MovePrevParagraph,
    MoveNextParagraph,
    MoveStartOfFile,
    MoveEndOfFile,
    MoveToMatching,

    FindForward(char),
    FindBack(char),
    FindTillForward(char),
    FindTillBack(char),
    FindRepeatNext,
    FindRepeatPrev,

    SearchWordForward,
    SearchWordBackward,
    SearchForward(String),
    SearchBack(String),
    SearchPartialForward(String),
    SearchPartialBack(String),
    SearchRepeatNext,
    SearchRepeatPrev,
}

impl Motion {
    pub fn search_str(&self) -> Option<&str> {
        match self {
            Motion::SearchForward(s) => Some(s.as_str()),
            Motion::SearchBack(s) => Some(s.as_str()),
            Motion::SearchPartialForward(s) => Some(s.as_str()),
            Motion::SearchPartialBack(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Vim {
    Invalid,

    Move(u16, Motion),

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

impl Default for VI {
    fn default() -> Self {
        let motion_buf = Rc::new(RefCell::new(String::new()));
        Self {
            mode: Default::default(),
            motion_buf: motion_buf.clone(),
            motion: Coroutine::new(|c, yp| Box::new(VI::next_motion(c, motion_buf, yp))),
            finds: Default::default(),
            matches: Default::default(),
        }
    }
}
impl VI {
    fn cancel(&mut self) {
        self.motion_buf.borrow_mut().clear();
        let mb = self.motion_buf.clone();
        self.motion = Coroutine::new(|c, yp| Box::new(VI::next_motion(c, mb, yp)));
        self.matches.clear();
        self.finds.clear();
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
                self.motion = Coroutine::new(|c, yp| Box::new(VI::next_motion(c, mb, yp)));
                Poll::Ready(v)
            }
            Resume::Pending => {
                debug!("    ...");
                Poll::Pending
            }
        }
    }

    async fn next_motion(
        mut tok: char,
        motion_buf: Rc<RefCell<String>>,
        yp: YieldPoint<char, Vim>,
    ) -> Vim {
        if tok == '0' {
            return Vim::Move(0, Motion::MoveToCol);
        }

        let mut mul = String::new();
        while tok.is_ascii_digit() || tok == '\x08' {
            if tok == '\x08' {
                mul.pop();
                motion_buf.borrow_mut().pop();
            } else {
                mul.push(tok);
                motion_buf.borrow_mut().push(tok);
            }
            tok = yp.yield0().await;
        }
        let mul = mul.parse::<u16>().ok();

        motion_buf.borrow_mut().push(tok);
        match tok {
            'h' => Vim::Move(mul.unwrap_or(1), Motion::MoveLeft),
            'l' => Vim::Move(mul.unwrap_or(1), Motion::MoveRight),
            'k' => Vim::Move(mul.unwrap_or(1), Motion::MoveUp),
            'j' => Vim::Move(mul.unwrap_or(1), Motion::MoveDown),
            '|' => Vim::Move(mul.unwrap_or(0), Motion::MoveToCol),
            'w' => Vim::Move(mul.unwrap_or(1), Motion::MoveNextWordStart),
            'b' => Vim::Move(mul.unwrap_or(1), Motion::MovePrevWordStart),
            'e' => Vim::Move(mul.unwrap_or(1), Motion::MoveNextWordEnd),
            'g' => {
                let tok = yp.yield0().await;
                motion_buf.borrow_mut().push(tok);
                match tok {
                    'e' => Vim::Move(mul.unwrap_or(1), Motion::MovePrevWordEnd),
                    'E' => Vim::Move(mul.unwrap_or(1), Motion::MovePrevWORDEnd),
                    '_' => Vim::Move(0, Motion::MoveEndOfLineText),
                    'g' => Vim::Move(0, Motion::MoveStartOfFile),
                    _ => Vim::Invalid,
                }
            }
            'W' => Vim::Move(mul.unwrap_or(1), Motion::MoveNextWORDStart),
            'B' => Vim::Move(mul.unwrap_or(1), Motion::MovePrevWORDStart),
            'E' => Vim::Move(mul.unwrap_or(1), Motion::MoveNextWORDEnd),
            '{' => Vim::Move(mul.unwrap_or(1), Motion::MovePrevParagraph),
            '}' => Vim::Move(mul.unwrap_or(1), Motion::MoveNextParagraph),
            'G' => Vim::Move(0, Motion::MoveEndOfFile),
            '^' => Vim::Move(0, Motion::MoveStartOfLineText),
            '$' => Vim::Move(0, Motion::MoveEndOfLine),
            '%' => Vim::Move(0, Motion::MoveToMatching),

            'f' => {
                let tok = yp.yield0().await;
                motion_buf.borrow_mut().push(tok);
                Vim::Move(mul.unwrap_or(1), Motion::FindForward(tok))
            }
            'F' => {
                let tok = yp.yield0().await;
                motion_buf.borrow_mut().push(tok);
                Vim::Move(mul.unwrap_or(1), Motion::FindBack(tok))
            }
            't' => {
                let tok = yp.yield0().await;
                motion_buf.borrow_mut().push(tok);
                Vim::Move(mul.unwrap_or(1), Motion::FindTillForward(tok))
            }
            'T' => {
                let tok = yp.yield0().await;
                motion_buf.borrow_mut().push(tok);
                Vim::Move(mul.unwrap_or(1), Motion::FindTillBack(tok))
            }
            ';' => Vim::Move(mul.unwrap_or(1), Motion::FindRepeatNext),
            ',' => Vim::Move(mul.unwrap_or(1), Motion::FindRepeatPrev),

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

mod op {
    use crate::SearchError;
    use crate::vi_state::query::*;
    use crate::vi_state::{Direction, Mode, Motion, VI, Vim};
    use log::debug;
    use rat_text::event::TextOutcome;
    use rat_text::text_area::TextAreaState;

    pub fn apply(
        vim: Vim,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Result<TextOutcome, SearchError> {
        let r = match vim {
            Vim::Invalid => TextOutcome::Continue,
            Vim::Move(mul, Motion::MoveLeft) => state.move_left(mul, false).into(),
            Vim::Move(mul, Motion::MoveRight) => state.move_right(mul, false).into(),
            Vim::Move(mul, Motion::MoveUp) => state.move_up(mul, false).into(),
            Vim::Move(mul, Motion::MoveDown) => state.move_down(mul, false).into(),
            Vim::Move(mul, Motion::MoveToCol) => move_to_col(mul, state).into(),
            Vim::Move(mul, Motion::MoveNextWordStart) => move_next_word_start(mul, state).into(),
            Vim::Move(mul, Motion::MovePrevWordStart) => move_prev_word_start(mul, state).into(),
            Vim::Move(mul, Motion::MoveNextWordEnd) => move_next_word_end(mul, state).into(),
            Vim::Move(mul, Motion::MovePrevWordEnd) => move_prev_word_end(mul, state).into(),
            Vim::Move(mul, Motion::MoveNextWORDStart) => move_next_bigword_start(mul, state).into(),
            Vim::Move(mul, Motion::MovePrevWORDStart) => move_prev_bigword_start(mul, state).into(),
            Vim::Move(mul, Motion::MoveNextWORDEnd) => move_next_bigword_end(mul, state).into(),
            Vim::Move(mul, Motion::MovePrevWORDEnd) => move_prev_bigword_end(mul, state).into(),
            Vim::Move(_, Motion::MoveStartOfLine) => move_start_of_line(state).into(),
            Vim::Move(_, Motion::MoveEndOfLine) => move_end_of_line(state).into(),
            Vim::Move(_, Motion::MoveStartOfLineText) => move_start_of_text(state).into(),
            Vim::Move(_, Motion::MoveEndOfLineText) => move_end_of_text(state).into(),
            Vim::Move(mul, Motion::MovePrevParagraph) => move_prev_paragraph(mul, state).into(),
            Vim::Move(mul, Motion::MoveNextParagraph) => move_next_paragraph(mul, state).into(),
            Vim::Move(_, Motion::MoveStartOfFile) => move_start_of_file(state).into(),
            Vim::Move(_, Motion::MoveEndOfFile) => move_end_of_file(state).into(),
            Vim::Move(_, Motion::MoveToMatching) => move_matching_brace(state).into(),

            Vim::Move(mul, Motion::FindForward(c)) => find_fwd(mul, c, state, vi).into(),
            Vim::Move(mul, Motion::FindBack(c)) => find_back(mul, c, state, vi).into(),
            Vim::Move(mul, Motion::FindTillForward(c)) => till_fwd(mul, c, state, vi).into(),
            Vim::Move(mul, Motion::FindTillBack(c)) => till_back(mul, c, state, vi).into(),
            Vim::Move(mul, Motion::FindRepeatNext) => find_repeat_fwd(mul, state, vi).into(),
            Vim::Move(mul, Motion::FindRepeatPrev) => find_repeat_back(mul, state, vi).into(),

            Vim::SearchPartialForward(s) => search_fwd(s, true, state, vi)?.into(),
            Vim::SearchForward(s) => search_fwd(s, false, state, vi)?.into(),
            Vim::SearchPartialBack(s) => search_back(s, true, state, vi)?.into(),
            Vim::SearchBack(s) => search_back(s, false, state, vi)?.into(),
            Vim::SearchWordForward => search_word_fwd(state, vi)?.into(),
            Vim::SearchWordBackward => search_word_back(state, vi)?.into(),
            Vim::SearchRepeatNext => search_repeat_fwd(state, vi).into(),
            Vim::SearchRepeatPrev => search_repeat_back(state, vi).into(),

            Vim::Insert => {
                vi.mode = Mode::Insert;
                TextOutcome::Changed
            }
            Vim::Move(_, _) => TextOutcome::Unchanged,
        };
        debug!("apply -> {:?}", r);
        Ok(TextOutcome::Changed)
    }

    pub(crate) fn search_repeat_back(state: &mut TextAreaState, vi: &mut VI) -> bool {
        if vi.matches.term.is_none() {
            return false;
        }
        vi_search_idx(&mut vi.matches, Direction::Backward, state);
        display_search_idx(state, vi);
        true
    }

    pub(crate) fn search_repeat_fwd(state: &mut TextAreaState, vi: &mut VI) -> bool {
        if vi.matches.term.is_none() {
            return false;
        }
        vi_search_idx(&mut vi.matches, Direction::Forward, state);
        display_search_idx(state, vi);
        true
    }

    pub(crate) fn search_word_back(
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Result<bool, SearchError> {
        let start = vi_word_start(state);
        let end = vi_word_end(state);
        let term = state.str_slice(start..end).to_string();

        vi_search(&mut vi.matches, term, Direction::Backward, false, state)?;
        vi_search_idx(&mut vi.matches, Direction::Forward, state);
        display_search(state, vi);
        Ok(true)
    }

    pub(crate) fn search_word_fwd(
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Result<bool, SearchError> {
        let start = vi_word_start(state);
        let end = vi_word_end(state);
        let term = state.str_slice(start..end).to_string();

        vi_search(&mut vi.matches, term, Direction::Forward, false, state)?;
        vi_search_idx(&mut vi.matches, Direction::Forward, state);
        display_search(state, vi);
        Ok(true)
    }

    pub(crate) fn search_back(
        term: String,
        tmp: bool,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Result<bool, SearchError> {
        vi_search(&mut vi.matches, term, Direction::Backward, tmp, state)?;
        vi_search_idx(&mut vi.matches, Direction::Forward, state);
        display_search(state, vi);
        Ok(true)
    }

    pub(crate) fn search_fwd(
        term: String,
        tmp: bool,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Result<bool, SearchError> {
        vi_search(&mut vi.matches, term, Direction::Forward, tmp, state)?;
        vi_search_idx(&mut vi.matches, Direction::Forward, state);
        display_search(state, vi);
        Ok(true)
    }

    pub(crate) fn display_search(state: &mut TextAreaState, vi: &mut VI) {
        state.remove_style_fully(999);
        for r in &vi.matches.list {
            state.add_style(r.clone(), 999);
        }
        display_search_idx(state, vi);
    }

    pub(crate) fn display_search_idx(state: &mut TextAreaState, vi: &mut VI) {
        if let Some(idx) = vi.matches.idx {
            let pos = state.byte_pos(vi.matches.list[idx].start);
            if vi.matches.tmp {
                state.scroll_to_pos(pos);
            } else {
                state.set_cursor(pos, false);
            }
        }
    }

    pub(crate) fn move_prev_paragraph(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_prev_paragraph(mul, state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    pub(crate) fn move_next_paragraph(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_next_paragraph(mul, state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    pub(crate) fn move_start_of_file(state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_start_of_file(state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    pub(crate) fn move_end_of_file(state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_end_of_file(state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    pub(crate) fn move_matching_brace(state: &mut TextAreaState) -> bool {
        if let Some(cursor) = vi_matching_brace(state) {
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

    pub(crate) fn find_repeat_back(mul: u16, state: &mut TextAreaState, vi: &mut VI) -> bool {
        let Some(last_term) = vi.finds.term else {
            return false;
        };
        let last_dir = vi.finds.dir;
        let last_till = vi.finds.till;

        vi_find(&mut vi.finds, last_term, last_dir, last_till, state);
        vi_find_idx(&mut vi.finds, mul, Direction::Backward, state);

        let dir = vi.finds.dir.mul(Direction::Backward);

        if let Some(idx) = vi.finds.idx {
            let pos = if vi.finds.till && dir == Direction::Backward {
                state.byte_pos(vi.finds.list[idx].end)
            } else {
                state.byte_pos(vi.finds.list[idx].start)
            };
            state.set_cursor(pos, false)
        } else {
            false
        }
    }

    pub(crate) fn find_repeat_fwd(mul: u16, state: &mut TextAreaState, vi: &mut VI) -> bool {
        let Some(last_term) = vi.finds.term else {
            return false;
        };
        let last_dir = vi.finds.dir;
        let last_till = vi.finds.till;

        vi_find(&mut vi.finds, last_term, last_dir, last_till, state);
        vi_find_idx(&mut vi.finds, mul, Direction::Forward, state);

        let dir = vi.finds.dir.mul(Direction::Forward);

        if let Some(idx) = vi.finds.idx {
            let pos = if vi.finds.till && dir == Direction::Backward {
                state.byte_pos(vi.finds.list[idx].end)
            } else {
                state.byte_pos(vi.finds.list[idx].start)
            };
            state.set_cursor(pos, false)
        } else {
            false
        }
    }

    pub(crate) fn till_back(mul: u16, term: char, state: &mut TextAreaState, vi: &mut VI) -> bool {
        vi_find(&mut vi.finds, term, Direction::Backward, true, state);
        vi_find_idx(&mut vi.finds, mul, Direction::Forward, state);

        if let Some(i) = vi.finds.idx {
            let pos = state.byte_pos(vi.finds.list[i].end);
            state.set_cursor(pos, false)
        } else {
            false
        }
    }

    pub(crate) fn till_fwd(mul: u16, term: char, state: &mut TextAreaState, vi: &mut VI) -> bool {
        vi_find(&mut vi.finds, term, Direction::Forward, true, state);
        vi_find_idx(&mut vi.finds, mul, Direction::Forward, state);

        if let Some(i) = vi.finds.idx {
            let pos = state.byte_pos(vi.finds.list[i].start);
            state.set_cursor(pos, false)
        } else {
            false
        }
    }

    pub(crate) fn find_back(mul: u16, term: char, state: &mut TextAreaState, vi: &mut VI) -> bool {
        vi_find(&mut vi.finds, term, Direction::Backward, false, state);
        vi_find_idx(&mut vi.finds, mul, Direction::Forward, state);

        if let Some(i) = vi.finds.idx {
            let pos = state.byte_pos(vi.finds.list[i].start);
            state.set_cursor(pos, false)
        } else {
            false
        }
    }

    pub(crate) fn find_fwd(mul: u16, term: char, state: &mut TextAreaState, vi: &mut VI) -> bool {
        vi_find(&mut vi.finds, term, Direction::Forward, false, state);
        vi_find_idx(&mut vi.finds, mul, Direction::Forward, state);

        if let Some(i) = vi.finds.idx {
            let pos = state.byte_pos(vi.finds.list[i].start);
            state.set_cursor(pos, false)
        } else {
            false
        }
    }

    pub(crate) fn move_to_col(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(pos) = vi_col(mul, state) {
            state.set_cursor(pos, false)
        } else {
            false
        }
    }

    pub(crate) fn move_next_word_start(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(word) = vi_next_word_start(mul, state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }

    pub(crate) fn move_prev_word_start(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(word) = vi_prev_word_start(mul, state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }

    pub(crate) fn move_next_word_end(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(word) = vi_next_word_end(mul, state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }

    pub(crate) fn move_prev_word_end(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(word) = vi_prev_word_end(mul, state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }

    pub(crate) fn move_next_bigword_start(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(word) = vi_next_bigword_start(mul, state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }

    pub(crate) fn move_prev_bigword_start(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(word) = vi_prev_bigword_start(mul, state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }

    pub(crate) fn move_next_bigword_end(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(word) = vi_next_bigword_end(mul, state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }

    pub(crate) fn move_prev_bigword_end(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(word) = vi_prev_bigword_end(mul, state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }
}

mod query {
    use crate::SearchError;
    use crate::vi_state::{Direction, Finds, Matches};
    use rat_text::text_area::TextAreaState;
    use rat_text::{Cursor, Grapheme, TextPosition, TextRange, upos_type};
    use regex_cursor::engines::dfa::{Regex, find_iter};
    use regex_cursor::{Input, RopeyCursor};

    pub(super) fn vi_search_idx(matches: &mut Matches, dir: Direction, state: &mut TextAreaState) {
        let c = state.byte_at(state.cursor()).start;
        let dir = matches.dir.mul(dir);

        if dir == Direction::Forward {
            matches.idx = matches.list.iter().position(|v| v.start > c);
        } else {
            matches.idx = matches
                .list
                .iter()
                .enumerate()
                .filter_map(|(i, v)| if v.end < c { Some(i) } else { None })
                .last();
        }
    }

    pub(super) fn vi_search(
        matches: &mut Matches,
        term: String,
        dir: Direction,
        tmp: bool,
        state: &mut TextAreaState,
    ) -> Result<(), SearchError> {
        if matches.term.as_ref() != Some(&term) {
            matches.term = Some(term);
            matches.dir = dir;
            matches.tmp = tmp;
            matches.idx = None;
            matches.list.clear();

            if let Ok(re) = Regex::new(matches.term.as_ref().expect("term")) {
                let cursor = RopeyCursor::new(state.rope().byte_slice(..));
                let input = Input::new(cursor);

                for m in find_iter(&re, input) {
                    matches.list.push(m.start()..m.end());
                }
            }
        } else {
            matches.dir = dir;
            matches.tmp = tmp;
            matches.idx = None;
        }
        Ok(())
    }

    pub(super) fn vi_start_of_file(_state: &mut TextAreaState) -> Option<TextPosition> {
        Some(TextPosition::new(0, 0))
    }

    pub(super) fn vi_end_of_file(state: &mut TextAreaState) -> Option<TextPosition> {
        let y = state.len_lines().saturating_sub(1);
        Some(TextPosition::new(state.line_width(y), y))
    }

    pub(super) fn vi_matching_brace(state: &mut TextAreaState) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        let peek_prev = it.peek_prev().map(|v| v.into_parts().0);
        let peek_prev = peek_prev.as_ref().map(|v| v.as_ref());
        let peek_next = it.peek_next().map(|v| v.into_parts().0);
        let peek_next = peek_next.as_ref().map(|v| v.as_ref());

        let (cc, co) = match (peek_prev, peek_next) {
            (Some(")"), _) => ('(', ')'),
            (Some("}"), _) => ('{', '}'),
            (Some("]"), _) => ('[', ']'),
            (Some(">"), _) => ('<', '>'),
            (_, Some("(")) => (')', '('),
            (_, Some("{")) => ('}', '{'),
            (_, Some("[")) => (']', '['),
            (_, Some("<")) => ('>', '<'),
            (Some("("), _) => {
                it.prev();
                (')', '(')
            }
            (Some("{"), _) => {
                it.prev();
                ('}', '{')
            }
            (Some("["), _) => {
                it.prev();
                (']', '[')
            }
            (Some("<"), _) => {
                it.prev();
                ('>', '<')
            }
            (_, Some(")")) => {
                it.next();
                ('(', ')')
            }
            (_, Some("}")) => {
                it.next();
                ('{', '}')
            }
            (_, Some("]")) => {
                it.next();
                ('[', ']')
            }
            (_, Some(">")) => {
                it.next();
                ('<', '>')
            }
            (_, _) => return None,
        };

        if cc == '(' || cc == '{' || cc == '[' || cc == '<' {
            let mut n = 0;
            loop {
                let Some(c) = it.prev() else {
                    break;
                };
                if c == cc {
                    n -= 1;
                } else if c == co {
                    n += 1;
                }
                if n == 0 {
                    break;
                }
            }
        } else {
            let mut n = 0;
            loop {
                let Some(c) = it.next() else {
                    break;
                };
                if c == cc {
                    n -= 1;
                } else if c == co {
                    n += 1;
                }
                if n == 0 {
                    break;
                }
            }
        }
        Some(state.byte_pos(it.text_offset()))
    }

    pub(super) fn vi_next_paragraph(
        mut mul: u16,
        state: &mut TextAreaState,
    ) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        let found;
        'f: loop {
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

            let mut brk = false;
            loop {
                let Some(c) = it.next() else {
                    if mul == 1 {
                        found = it.text_offset();
                        break 'f;
                    } else {
                        return None;
                    }
                };

                if c.is_line_break() {
                    if !brk {
                        brk = true;
                    } else {
                        break;
                    }
                } else if !c.is_whitespace() {
                    brk = false;
                }
            }

            mul -= 1;
            if mul == 0 {
                it.prev();
                found = it.text_offset();
                break 'f;
            }
        }

        Some(state.byte_pos(found))
    }

    pub(super) fn vi_prev_paragraph(
        mut mul: u16,
        state: &mut TextAreaState,
    ) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        let found;
        'f: loop {
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

            let mut brk = false;
            loop {
                let Some(c) = it.prev() else {
                    if mul == 1 {
                        found = it.text_offset();
                        break 'f;
                    } else {
                        return None;
                    }
                };

                if c.is_line_break() {
                    if !brk {
                        brk = true;
                    } else {
                        break;
                    }
                } else if !c.is_whitespace() {
                    brk = false;
                }
            }

            mul -= 1;
            if mul == 0 {
                it.next();
                found = it.text_offset();
                break 'f;
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

    pub(super) fn vi_find_idx(
        finds: &mut Finds,
        mul: u16,
        dir: Direction,
        state: &mut TextAreaState,
    ) {
        let mut c = state.byte_at(state.cursor()).start;

        let dir = finds.dir.mul(dir);
        let mul = (mul as usize).saturating_sub(1);

        if dir == Direction::Forward {
            finds.idx = finds.list.iter().position(|v| v.start > c);

            finds.idx = if let Some(idx) = finds.idx {
                if idx + mul < finds.list.len() {
                    Some(idx + mul)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            // Till backwards might need to correct the cursor.
            if finds.till {
                if let Some(i) = finds.idx {
                    let r = finds.list[i].clone();
                    if c == r.end {
                        c = r.start;
                    }
                }
            }

            finds.idx = finds
                .list
                .iter()
                .enumerate()
                .filter_map(|(i, v)| if v.end < c { Some(i) } else { None })
                .last();

            finds.idx = if let Some(idx) = finds.idx {
                if idx >= mul { Some(idx - mul) } else { None }
            } else {
                None
            }
        }
    }

    pub(super) fn vi_find(
        finds: &mut Finds,
        term: char,
        dir: Direction,
        till: bool,
        state: &TextAreaState,
    ) {
        if finds.term != Some(term) || finds.row != state.cursor().y {
            finds.term = Some(term);
            finds.row = state.cursor().y;
            finds.dir = dir;
            finds.till = till;
            finds.idx = None;
            finds.list.clear();

            let cursor = state.cursor();
            let start = TextPosition::new(0, cursor.y);
            let end = TextPosition::new(state.line_width(cursor.y), cursor.y);
            let mut it = state.graphemes(TextRange::new(start, end), start);
            loop {
                let Some(c) = it.next() else {
                    break;
                };
                if c == term {
                    finds.list.push(c.text_bytes());
                }
            }
        } else {
            finds.row = state.cursor().y;
            finds.dir = dir;
            finds.till = till;
            finds.idx = None;
        }
    }

    pub(super) fn vi_prev_bigword_start(
        mut mul: u16,
        state: &TextAreaState,
    ) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        loop {
            let Some(sample) = it.peek_prev() else {
                return None;
            };
            if !sample.is_whitespace() {
                pskip_nonwhite(&mut it);
            } else {
                pskip_white(&mut it);
                pskip_nonwhite(&mut it);
            }

            mul -= 1;
            if mul == 0 {
                break;
            }
        }

        Some(state.byte_pos(it.text_offset()))
    }

    pub(super) fn vi_next_bigword_start(
        mut mul: u16,
        state: &TextAreaState,
    ) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        loop {
            let Some(sample) = it.peek_next() else {
                return None;
            };
            if !sample.is_whitespace() {
                skip_nonwhite(&mut it);
            }
            skip_white(&mut it);

            mul -= 1;
            if mul == 0 {
                break;
            }
        }

        Some(state.byte_pos(it.text_offset()))
    }

    pub(super) fn vi_prev_bigword_end(mut mul: u16, state: &TextAreaState) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        loop {
            let Some(sample) = it.peek_prev() else {
                return None;
            };
            if !sample.is_whitespace() {
                pskip_nonwhite(&mut it);
            }
            pskip_white(&mut it);

            mul -= 1;
            if mul == 0 {
                break;
            }
        }

        Some(state.byte_pos(it.text_offset()))
    }

    pub(super) fn vi_next_bigword_end(mut mul: u16, state: &TextAreaState) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        loop {
            skip_white(&mut it);

            let Some(sample) = it.peek_next() else {
                return None;
            };
            if !sample.is_whitespace() {
                skip_nonwhite(&mut it);
            }

            mul -= 1;
            if mul == 0 {
                break;
            }
        }

        Some(state.byte_pos(it.text_offset()))
    }

    pub(super) fn vi_next_word_end(mut mul: u16, state: &TextAreaState) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        loop {
            skip_white(&mut it);

            let Some(sample) = it.peek_next() else {
                return None;
            };
            if sample.is_alphanumeric() {
                skip_alpha(&mut it);
            } else {
                skip_sample(&mut it, sample);
            }

            mul -= 1;
            if mul == 0 {
                break;
            }
        }

        Some(state.byte_pos(it.text_offset()))
    }

    pub(super) fn vi_prev_word_end(mut mul: u16, state: &TextAreaState) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        loop {
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

            mul -= 1;
            if mul == 0 {
                break;
            }
        }

        Some(state.byte_pos(it.text_offset()))
    }

    pub(super) fn vi_col(mul: u16, state: &TextAreaState) -> Option<TextPosition> {
        let c = state.cursor();
        if mul as upos_type <= state.line_width(c.y) {
            Some(TextPosition::new(mul as upos_type, c.y))
        } else {
            None
        }
    }

    pub(super) fn vi_next_word_start(mut mul: u16, state: &TextAreaState) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        loop {
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

            mul -= 1;
            if mul == 0 {
                break;
            }
        }

        Some(state.byte_pos(it.text_offset()))
    }

    pub(super) fn vi_prev_word_start(mut mul: u16, state: &TextAreaState) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        loop {
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

            mul -= 1;
            if mul == 0 {
                break;
            }
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

impl HandleEvent<crossterm::event::Event, &mut VI, Result<TextOutcome, SearchError>>
    for TextAreaState
{
    fn handle(
        &mut self,
        event: &crossterm::event::Event,
        vi: &mut VI,
    ) -> Result<TextOutcome, SearchError> {
        let r = if vi.mode == Mode::Normal {
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
        } else if vi.mode == Mode::Insert {
            match event {
                ct_event!(keycode press Esc) | ct_event!(key press CONTROL-'c') => {
                    vi.mode = Mode::Normal;
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
        } else if vi.mode == Mode::Visual {
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
