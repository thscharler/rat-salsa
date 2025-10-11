use crate::vi_state::op::*;
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

    pub page: (u16, u16),
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
    MoveToLine,
    MoveToLinePercent,
    MoveMiddleOfScreen,
    MoveTopOfScreen,
    MoveBottomOfScreen,
    MoveToMatching,

    MoveStartOfFile,
    MoveEndOfFile,

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
    MovePageUp,
    MovePageDown,

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
    Scroll(u16, Motion),
    Insert,
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
            page: Default::default(),
        }
    }
}

#[allow(dead_code)]
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

    fn ctrl(c: char) -> char {
        match c {
            'a' | 'A' => VI::CTRL_A,
            'b' | 'B' => VI::CTRL_B,
            'c' | 'C' => VI::CTRL_C,
            'd' | 'D' => VI::CTRL_D,
            'e' | 'E' => VI::CTRL_E,
            'f' | 'F' => VI::CTRL_F,
            'g' | 'G' => VI::CTRL_G,
            'h' | 'H' => VI::CTRL_H,
            'i' | 'I' => VI::CTRL_I,
            'j' | 'J' => VI::CTRL_J,
            'k' | 'K' => VI::CTRL_K,
            'l' | 'L' => VI::CTRL_L,
            'm' | 'M' => VI::CTRL_M,
            'n' | 'N' => VI::CTRL_N,
            'o' | 'O' => VI::CTRL_O,
            'p' | 'P' => VI::CTRL_P,
            'q' | 'Q' => VI::CTRL_Q,
            'r' | 'R' => VI::CTRL_R,
            's' | 'S' => VI::CTRL_S,
            't' | 'T' => VI::CTRL_T,
            'u' | 'U' => VI::CTRL_U,
            'v' | 'V' => VI::CTRL_V,
            'w' | 'W' => VI::CTRL_W,
            'x' | 'X' => VI::CTRL_X,
            'y' | 'Y' => VI::CTRL_Y,
            'z' | 'Z' => VI::CTRL_Z,
            _ => unimplemented!(),
        }
    }

    const CTRL_A: char = '\x01';
    const CTRL_B: char = '\x02';
    const CTRL_C: char = '\x03';
    const CTRL_D: char = '\x04';
    const CTRL_E: char = '\x05';
    const CTRL_F: char = '\x06';
    const CTRL_G: char = '\x07';
    const CTRL_H: char = '\x08';
    const BS: char = '\x08';
    const CTRL_I: char = '\x09';
    const CTRL_J: char = '\x0A';
    const CTRL_K: char = '\x0B';
    const CTRL_L: char = '\x0C';
    const CTRL_M: char = '\x0D';
    const CTRL_N: char = '\x0E';
    const CTRL_O: char = '\x0F';
    const CTRL_P: char = '\x10';
    const CTRL_Q: char = '\x11';
    const CTRL_R: char = '\x12';
    const CTRL_S: char = '\x13';
    const CTRL_T: char = '\x14';
    const CTRL_U: char = '\x15';
    const CTRL_V: char = '\x16';
    const CTRL_W: char = '\x17';
    const CTRL_X: char = '\x18';
    const CTRL_Y: char = '\x19';
    const CTRL_Z: char = '\x1A';

    async fn next_motion(
        mut tok: char,
        motion_buf: Rc<RefCell<String>>,
        yp: YieldPoint<char, Vim>,
    ) -> Vim {
        if tok == '0' {
            return Vim::Move(0, Motion::MoveStartOfLine);
        }

        let mut mul = String::new();
        while tok.is_ascii_digit() || tok == VI::BS {
            if tok == VI::BS {
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
                    '_' => Vim::Move(mul.unwrap_or(1), Motion::MoveEndOfLineText),
                    'g' => {
                        if let Some(mul) = mul {
                            Vim::Move(mul, Motion::MoveToLine)
                        } else {
                            Vim::Move(0, Motion::MoveStartOfFile)
                        }
                    }
                    _ => Vim::Invalid,
                }
            }
            'W' => Vim::Move(mul.unwrap_or(1), Motion::MoveNextWORDStart),
            'B' => Vim::Move(mul.unwrap_or(1), Motion::MovePrevWORDStart),
            'E' => Vim::Move(mul.unwrap_or(1), Motion::MoveNextWORDEnd),
            '{' => Vim::Move(mul.unwrap_or(1), Motion::MovePrevParagraph),
            '}' => Vim::Move(mul.unwrap_or(1), Motion::MoveNextParagraph),
            'G' => {
                if let Some(mul) = mul {
                    Vim::Move(mul, Motion::MoveToLine)
                } else {
                    Vim::Move(0, Motion::MoveEndOfFile)
                }
            }
            '^' => Vim::Move(0, Motion::MoveStartOfLineText),
            '$' => Vim::Move(mul.unwrap_or(1), Motion::MoveEndOfLine),
            '%' => {
                if let Some(mul) = mul {
                    Vim::Move(mul, Motion::MoveToLinePercent)
                } else {
                    Vim::Move(0, Motion::MoveToMatching)
                }
            }
            VI::CTRL_U => Vim::Move(mul.unwrap_or(0), Motion::MovePageUp),
            VI::CTRL_D => Vim::Move(mul.unwrap_or(0), Motion::MovePageDown),
            VI::CTRL_Y => Vim::Scroll(mul.unwrap_or(1), Motion::MoveUp),
            VI::CTRL_E => Vim::Scroll(mul.unwrap_or(1), Motion::MoveDown),
            VI::CTRL_B => Vim::Scroll(mul.unwrap_or(1), Motion::MovePageUp),
            VI::CTRL_F => Vim::Scroll(mul.unwrap_or(1), Motion::MovePageDown),
            'z' => {
                let tok = yp.yield0().await;
                motion_buf.borrow_mut().push(tok);
                match tok {
                    'z' => Vim::Scroll(0, Motion::MoveMiddleOfScreen),
                    't' => Vim::Scroll(0, Motion::MoveTopOfScreen),
                    'b' => Vim::Scroll(0, Motion::MoveBottomOfScreen),
                    _ => Vim::Invalid,
                }
            }

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
                    let tok = yp
                        .yield1(Vim::Move(
                            mul.unwrap_or(1),
                            Motion::SearchPartialForward(buf.clone()),
                        ))
                        .await;
                    if tok == '\n' {
                        break;
                    } else if tok == VI::BS {
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
                Vim::Move(mul.unwrap_or(1), Motion::SearchForward(buf))
            }
            '?' => {
                let mut buf = String::new();
                loop {
                    let tok = yp
                        .yield1(Vim::Move(
                            mul.unwrap_or(1),
                            Motion::SearchPartialBack(buf.clone()),
                        ))
                        .await;
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
                Vim::Move(mul.unwrap_or(1), Motion::SearchBack(buf))
            }
            '*' => Vim::Move(mul.unwrap_or(1), Motion::SearchWordForward),
            '#' => Vim::Move(mul.unwrap_or(1), Motion::SearchWordBackward),
            'n' => Vim::Move(mul.unwrap_or(1), Motion::SearchRepeatNext),
            'N' => Vim::Move(mul.unwrap_or(1), Motion::SearchRepeatPrev),

            'i' => Vim::Insert,

            _ => Vim::Invalid,
        }
    }
}

fn run_vim(vim: Vim, state: &mut TextAreaState, vi: &mut VI) -> Result<TextOutcome, SearchError> {
    let r = match vim {
        Vim::Invalid => TextOutcome::Continue,
        Vim::Move(mul, Motion::MoveLeft) => state.move_left(mul, false).into(),
        Vim::Move(mul, Motion::MoveRight) => state.move_right(mul, false).into(),
        Vim::Move(mul, Motion::MoveUp) => state.move_up(mul, false).into(),
        Vim::Move(mul, Motion::MoveDown) => state.move_down(mul, false).into(),
        Vim::Move(mul, Motion::MoveToCol) => move_to_col(mul, state).into(),
        Vim::Move(mul, Motion::MoveToLine) => move_to_line(mul, state).into(),
        Vim::Move(mul, Motion::MoveToLinePercent) => move_to_line_percent(mul, state).into(),
        Vim::Move(mul, Motion::MoveNextWordStart) => move_next_word_start(mul, state).into(),
        Vim::Move(mul, Motion::MovePrevWordStart) => move_prev_word_start(mul, state).into(),
        Vim::Move(mul, Motion::MoveNextWordEnd) => move_next_word_end(mul, state).into(),
        Vim::Move(mul, Motion::MovePrevWordEnd) => move_prev_word_end(mul, state).into(),
        Vim::Move(mul, Motion::MoveNextWORDStart) => move_next_bigword_start(mul, state).into(),
        Vim::Move(mul, Motion::MovePrevWORDStart) => move_prev_bigword_start(mul, state).into(),
        Vim::Move(mul, Motion::MoveNextWORDEnd) => move_next_bigword_end(mul, state).into(),
        Vim::Move(mul, Motion::MovePrevWORDEnd) => move_prev_bigword_end(mul, state).into(),
        Vim::Move(_, Motion::MoveStartOfLine) => move_start_of_line(state).into(),
        Vim::Move(mul, Motion::MoveEndOfLine) => move_end_of_line(mul, state).into(),
        Vim::Move(_, Motion::MoveStartOfLineText) => move_start_of_text(state).into(),
        Vim::Move(mul, Motion::MoveEndOfLineText) => move_end_of_text(mul, state).into(),
        Vim::Move(mul, Motion::MovePrevParagraph) => move_prev_paragraph(mul, state).into(),
        Vim::Move(mul, Motion::MoveNextParagraph) => move_next_paragraph(mul, state).into(),
        Vim::Move(_, Motion::MoveStartOfFile) => move_start_of_file(state).into(),
        Vim::Move(_, Motion::MoveEndOfFile) => move_end_of_file(state).into(),
        Vim::Move(_, Motion::MoveToMatching) => move_matching_brace(state).into(),
        Vim::Move(mul, Motion::MovePageUp) => move_page_up(mul, state, vi).into(),
        Vim::Move(mul, Motion::MovePageDown) => move_page_down(mul, state, vi).into(),

        Vim::Move(mul, Motion::FindForward(c)) => find_fwd(mul, c, state, vi).into(),
        Vim::Move(mul, Motion::FindBack(c)) => find_back(mul, c, state, vi).into(),
        Vim::Move(mul, Motion::FindTillForward(c)) => till_fwd(mul, c, state, vi).into(),
        Vim::Move(mul, Motion::FindTillBack(c)) => till_back(mul, c, state, vi).into(),
        Vim::Move(mul, Motion::FindRepeatNext) => find_repeat_fwd(mul, state, vi).into(),
        Vim::Move(mul, Motion::FindRepeatPrev) => find_repeat_back(mul, state, vi).into(),

        Vim::Move(mul, Motion::SearchPartialForward(s)) => {
            search_fwd(mul, s, true, state, vi)?.into()
        }
        Vim::Move(mul, Motion::SearchPartialBack(s)) => {
            search_back(mul, s, true, state, vi)?.into()
        }
        Vim::Move(mul, Motion::SearchForward(s)) => search_fwd(mul, s, false, state, vi)?.into(),
        Vim::Move(mul, Motion::SearchBack(s)) => search_back(mul, s, false, state, vi)?.into(),
        Vim::Move(mul, Motion::SearchWordForward) => search_word_fwd(mul, state, vi)?.into(),
        Vim::Move(mul, Motion::SearchWordBackward) => search_word_back(mul, state, vi)?.into(),
        Vim::Move(mul, Motion::SearchRepeatNext) => search_repeat_fwd(mul, state, vi).into(),
        Vim::Move(mul, Motion::SearchRepeatPrev) => search_repeat_back(mul, state, vi).into(),
        Vim::Move(_, _) => TextOutcome::Unchanged,

        Vim::Scroll(mul, Motion::MovePageUp) => scroll_page_up(mul, state, vi).into(),
        Vim::Scroll(mul, Motion::MovePageDown) => scroll_page_down(mul, state, vi).into(),
        Vim::Scroll(mul, Motion::MoveUp) => scroll_up(mul, state).into(),
        Vim::Scroll(mul, Motion::MoveDown) => scroll_down(mul, state).into(),
        Vim::Scroll(_, Motion::MoveMiddleOfScreen) => scroll_cursor_to_middle(state).into(),
        Vim::Scroll(_, Motion::MoveTopOfScreen) => scroll_cursor_to_top(state).into(),
        Vim::Scroll(_, Motion::MoveBottomOfScreen) => scroll_cursor_to_bottom(state).into(),
        Vim::Scroll(_, _) => TextOutcome::Unchanged,

        Vim::Insert => {
            vi.mode = Mode::Insert;
            TextOutcome::Changed
        }
    };
    debug!("run_vim -> {:?}", r);
    Ok(TextOutcome::Changed)
}

mod op {
    use crate::SearchError;
    use crate::vi_state::query::*;
    use crate::vi_state::{Direction, VI};
    use log::debug;
    use rat_text::text_area::TextAreaState;
    use std::cmp::max;

    pub fn search_repeat_back(mul: u16, state: &mut TextAreaState, vi: &mut VI) -> bool {
        if vi.matches.term.is_none() {
            return false;
        }
        q_search_idx(&mut vi.matches, mul, Direction::Backward, state);
        display_search_idx(state, vi);
        true
    }

    pub fn search_repeat_fwd(mul: u16, state: &mut TextAreaState, vi: &mut VI) -> bool {
        if vi.matches.term.is_none() {
            return false;
        }
        q_search_idx(&mut vi.matches, mul, Direction::Forward, state);
        display_search_idx(state, vi);
        true
    }

    pub fn search_word_back(
        mul: u16,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Result<bool, SearchError> {
        let start = q_word_start(state);
        let end = q_word_end(state);
        let term = state.str_slice(start..end).to_string();

        q_search(&mut vi.matches, term, Direction::Backward, false, state)?;
        q_search_idx(&mut vi.matches, mul, Direction::Forward, state);
        display_search(state, vi);
        Ok(true)
    }

    pub fn search_word_fwd(
        mul: u16,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Result<bool, SearchError> {
        let start = q_word_start(state);
        let end = q_word_end(state);
        let term = state.str_slice(start..end).to_string();

        q_search(&mut vi.matches, term, Direction::Forward, false, state)?;
        q_search_idx(&mut vi.matches, mul, Direction::Forward, state);
        display_search(state, vi);
        Ok(true)
    }

    pub fn search_back(
        mul: u16,
        term: String,
        tmp: bool,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Result<bool, SearchError> {
        q_search(&mut vi.matches, term, Direction::Backward, tmp, state)?;
        q_search_idx(&mut vi.matches, mul, Direction::Forward, state);
        display_search(state, vi);
        Ok(true)
    }

    pub fn search_fwd(
        mul: u16,
        term: String,
        tmp: bool,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Result<bool, SearchError> {
        q_search(&mut vi.matches, term, Direction::Forward, tmp, state)?;
        q_search_idx(&mut vi.matches, mul, Direction::Forward, state);
        display_search(state, vi);
        Ok(true)
    }

    pub fn display_search(state: &mut TextAreaState, vi: &mut VI) {
        state.remove_style_fully(999);
        for r in &vi.matches.list {
            state.add_style(r.clone(), 999);
        }
        display_search_idx(state, vi);
    }

    pub fn display_search_idx(state: &mut TextAreaState, vi: &mut VI) {
        if let Some(idx) = vi.matches.idx {
            let pos = state.byte_pos(vi.matches.list[idx].start);
            if vi.matches.tmp {
                state.scroll_to_pos(pos);
            } else {
                state.set_cursor(pos, false);
            }
        }
    }

    pub fn move_prev_paragraph(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(cursor) = q_prev_paragraph(mul, state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    pub fn move_next_paragraph(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(cursor) = q_next_paragraph(mul, state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    pub fn move_start_of_file(state: &mut TextAreaState) -> bool {
        if let Some(cursor) = q_start_of_file(state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    pub fn move_end_of_file(state: &mut TextAreaState) -> bool {
        if let Some(cursor) = q_end_of_file(state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    pub fn scroll_cursor_to_middle(state: &mut TextAreaState) -> bool {
        let c = state.cursor();
        if let Some((_rx, ry)) = state.pos_to_relative_screen(c) {
            let noy = ry - (state.rendered.height as i16) / 2;
            if let Some(no) = state.relative_screen_to_pos((0, noy)) {
                state.set_sub_row_offset(no.x);
                state.set_offset((0, no.y as usize));
                true
            } else {
                // ???
                false
            }
        } else {
            state.scroll_cursor_to_visible();
            true
        }
    }

    pub fn scroll_cursor_to_bottom(state: &mut TextAreaState) -> bool {
        let c = state.cursor();
        if let Some((_rx, ry)) = state.pos_to_relative_screen(c) {
            let noy = ry - state.rendered.height.saturating_sub(1) as i16;
            if let Some(no) = state.relative_screen_to_pos((0, noy)) {
                state.set_sub_row_offset(no.x);
                state.set_offset((0, no.y as usize));
                true
            } else {
                // ???
                false
            }
        } else {
            state.scroll_cursor_to_visible();
            true
        }
    }

    pub fn scroll_cursor_to_top(state: &mut TextAreaState) -> bool {
        let c = state.cursor();
        if let Some((_rx, ry)) = state.pos_to_relative_screen(c) {
            if let Some(no) = state.relative_screen_to_pos((0, ry)) {
                state.set_sub_row_offset(no.x);
                state.set_offset((0, no.y as usize));
                true
            } else {
                // ???
                false
            }
        } else {
            state.scroll_cursor_to_visible();
            true
        }
    }

    pub fn scroll_up(mul: u16, state: &mut TextAreaState) -> bool {
        state.scroll_up(mul as usize)
    }

    pub fn scroll_down(mul: u16, state: &mut TextAreaState) -> bool {
        state.scroll_down(mul as usize)
    }

    pub fn scroll_page_up(mul: u16, state: &mut TextAreaState, vi: &mut VI) -> bool {
        if vi.page.0 != state.vertical_page() as u16 {
            vi.page = (
                state.vertical_page() as u16,
                (state.vertical_page() / 2) as u16,
            );
        }

        state.scroll_up((vi.page.0 * mul).saturating_sub(2) as usize)
    }

    pub fn scroll_page_down(mul: u16, state: &mut TextAreaState, vi: &mut VI) -> bool {
        if vi.page.0 != state.vertical_page() as u16 {
            vi.page = (
                state.vertical_page() as u16,
                (state.vertical_page() / 2) as u16,
            );
        }

        state.scroll_down((vi.page.0 * mul).saturating_sub(2) as usize)
    }

    pub fn move_page_up(mul: u16, state: &mut TextAreaState, vi: &mut VI) -> bool {
        if vi.page.0 != state.vertical_page() as u16 {
            vi.page = (
                state.vertical_page() as u16,
                (state.vertical_page() / 2) as u16,
            );
        }
        if mul != 0 {
            vi.page.1 = mul;
        }

        state.move_up(vi.page.1, false)
    }

    pub fn move_page_down(mul: u16, state: &mut TextAreaState, vi: &mut VI) -> bool {
        if vi.page.0 != state.vertical_page() as u16 {
            vi.page = (
                state.vertical_page() as u16,
                (state.vertical_page() / 2) as u16,
            );
        }
        if mul != 0 {
            vi.page.1 = mul;
        }

        state.move_down(vi.page.1, false)
    }

    pub fn move_matching_brace(state: &mut TextAreaState) -> bool {
        if let Some(cursor) = q_matching_brace(state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    pub fn move_end_of_text(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(cursor) = q_end_of_text(mul, state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    pub fn move_start_of_text(state: &mut TextAreaState) -> bool {
        if let Some(cursor) = q_start_of_text(state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    pub fn move_start_of_line(state: &mut TextAreaState) -> bool {
        if let Some(cursor) = q_start_of_line(state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    pub fn move_end_of_line(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(cursor) = q_end_of_line(mul, state) {
            state.set_cursor(cursor, false)
        } else {
            false
        }
    }

    pub fn find_repeat_back(mul: u16, state: &mut TextAreaState, vi: &mut VI) -> bool {
        let Some(last_term) = vi.finds.term else {
            return false;
        };
        let last_dir = vi.finds.dir;
        let last_till = vi.finds.till;

        q_find(&mut vi.finds, last_term, last_dir, last_till, state);
        q_find_idx(&mut vi.finds, mul, Direction::Backward, state);

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

    pub fn find_repeat_fwd(mul: u16, state: &mut TextAreaState, vi: &mut VI) -> bool {
        let Some(last_term) = vi.finds.term else {
            return false;
        };
        let last_dir = vi.finds.dir;
        let last_till = vi.finds.till;

        q_find(&mut vi.finds, last_term, last_dir, last_till, state);
        q_find_idx(&mut vi.finds, mul, Direction::Forward, state);

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

    pub fn till_back(mul: u16, term: char, state: &mut TextAreaState, vi: &mut VI) -> bool {
        q_find(&mut vi.finds, term, Direction::Backward, true, state);
        q_find_idx(&mut vi.finds, mul, Direction::Forward, state);

        if let Some(i) = vi.finds.idx {
            let pos = state.byte_pos(vi.finds.list[i].end);
            state.set_cursor(pos, false)
        } else {
            false
        }
    }

    pub fn till_fwd(mul: u16, term: char, state: &mut TextAreaState, vi: &mut VI) -> bool {
        q_find(&mut vi.finds, term, Direction::Forward, true, state);
        q_find_idx(&mut vi.finds, mul, Direction::Forward, state);

        if let Some(i) = vi.finds.idx {
            let pos = state.byte_pos(vi.finds.list[i].start);
            state.set_cursor(pos, false)
        } else {
            false
        }
    }

    pub fn find_back(mul: u16, term: char, state: &mut TextAreaState, vi: &mut VI) -> bool {
        q_find(&mut vi.finds, term, Direction::Backward, false, state);
        q_find_idx(&mut vi.finds, mul, Direction::Forward, state);

        if let Some(i) = vi.finds.idx {
            let pos = state.byte_pos(vi.finds.list[i].start);
            state.set_cursor(pos, false)
        } else {
            false
        }
    }

    pub fn find_fwd(mul: u16, term: char, state: &mut TextAreaState, vi: &mut VI) -> bool {
        q_find(&mut vi.finds, term, Direction::Forward, false, state);
        q_find_idx(&mut vi.finds, mul, Direction::Forward, state);

        if let Some(i) = vi.finds.idx {
            let pos = state.byte_pos(vi.finds.list[i].start);
            state.set_cursor(pos, false)
        } else {
            false
        }
    }

    pub fn move_to_line_percent(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(pos) = q_line_percent(mul, state) {
            state.set_cursor(pos, false)
        } else {
            false
        }
    }

    pub fn move_to_line(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(pos) = q_line(mul, state) {
            state.set_cursor(pos, false)
        } else {
            false
        }
    }

    pub fn move_to_col(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(pos) = q_col(mul, state) {
            state.set_cursor(pos, false)
        } else {
            false
        }
    }

    pub fn move_next_word_start(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(word) = q_next_word_start(mul, state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }

    pub fn move_prev_word_start(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(word) = q_prev_word_start(mul, state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }

    pub fn move_next_word_end(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(word) = q_next_word_end(mul, state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }

    pub fn move_prev_word_end(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(word) = q_prev_word_end(mul, state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }

    pub fn move_next_bigword_start(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(word) = q_next_bigword_start(mul, state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }

    pub fn move_prev_bigword_start(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(word) = q_prev_bigword_start(mul, state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }

    pub fn move_next_bigword_end(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(word) = q_next_bigword_end(mul, state) {
            state.set_cursor(word, false)
        } else {
            false
        }
    }

    pub fn move_prev_bigword_end(mul: u16, state: &mut TextAreaState) -> bool {
        if let Some(word) = q_prev_bigword_end(mul, state) {
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
    use std::cmp::min;

    pub fn q_search_idx(
        matches: &mut Matches,
        mul: u16,
        dir: Direction,
        state: &mut TextAreaState,
    ) {
        let c = state.byte_at(state.cursor()).start;
        let dir = matches.dir.mul(dir);
        let mul = (mul as usize).saturating_sub(1);

        if dir == Direction::Forward {
            matches.idx = matches.list.iter().position(|v| v.start > c);

            matches.idx = if let Some(idx) = matches.idx {
                if idx + mul < matches.list.len() {
                    Some(idx + mul)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            matches.idx = matches
                .list
                .iter()
                .enumerate()
                .filter_map(|(i, v)| if v.end < c { Some(i) } else { None })
                .last();

            matches.idx = if let Some(idx) = matches.idx {
                if idx >= mul { Some(idx - mul) } else { None }
            } else {
                None
            }
        }
    }

    pub fn q_search(
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

    pub fn q_start_of_file(_state: &mut TextAreaState) -> Option<TextPosition> {
        Some(TextPosition::new(0, 0))
    }

    pub fn q_end_of_file(state: &mut TextAreaState) -> Option<TextPosition> {
        let y = state.len_lines().saturating_sub(1);
        Some(TextPosition::new(state.line_width(y), y))
    }

    pub fn q_matching_brace(state: &mut TextAreaState) -> Option<TextPosition> {
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

    pub fn q_next_paragraph(mut mul: u16, state: &mut TextAreaState) -> Option<TextPosition> {
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

    pub fn q_prev_paragraph(mut mul: u16, state: &mut TextAreaState) -> Option<TextPosition> {
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

    pub fn q_end_of_text(mul: u16, state: &mut TextAreaState) -> Option<TextPosition> {
        let y = min(
            state.cursor().y + mul.saturating_sub(1) as upos_type,
            state.len_lines().saturating_sub(1),
        );

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

    pub fn q_start_of_text(state: &mut TextAreaState) -> Option<TextPosition> {
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

    pub fn q_start_of_line(state: &mut TextAreaState) -> Option<TextPosition> {
        Some(TextPosition::new(0, state.cursor().y))
    }

    pub fn q_end_of_line(mul: u16, state: &mut TextAreaState) -> Option<TextPosition> {
        let y = min(
            state.cursor().y + mul.saturating_sub(1) as upos_type,
            state.len_lines().saturating_sub(1),
        );
        Some(TextPosition::new(state.line_width(y), y))
    }

    pub fn q_find_idx(finds: &mut Finds, mul: u16, dir: Direction, state: &mut TextAreaState) {
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

    pub fn q_find(
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

    pub fn q_prev_bigword_start(mut mul: u16, state: &TextAreaState) -> Option<TextPosition> {
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

    pub fn q_next_bigword_start(mut mul: u16, state: &TextAreaState) -> Option<TextPosition> {
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

    pub fn q_prev_bigword_end(mut mul: u16, state: &TextAreaState) -> Option<TextPosition> {
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

    pub fn q_next_bigword_end(mut mul: u16, state: &TextAreaState) -> Option<TextPosition> {
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

    pub fn q_next_word_end(mut mul: u16, state: &TextAreaState) -> Option<TextPosition> {
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

    pub fn q_prev_word_end(mut mul: u16, state: &TextAreaState) -> Option<TextPosition> {
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

    pub fn q_line_percent(mul: u16, state: &TextAreaState) -> Option<TextPosition> {
        let len = state.len_lines() as u64;
        let pc = min(mul.saturating_sub(1), 100) as u64;
        let line = ((len * pc) / 100) as u32;
        Some(TextPosition::new(0, line))
    }

    pub fn q_line(mul: u16, state: &TextAreaState) -> Option<TextPosition> {
        let line = min(
            mul.saturating_sub(1) as upos_type,
            state.len_lines().saturating_sub(1),
        );
        Some(TextPosition::new(0, line))
    }

    pub fn q_col(mul: u16, state: &TextAreaState) -> Option<TextPosition> {
        let c = state.cursor();
        if mul as upos_type <= state.line_width(c.y) {
            Some(TextPosition::new(mul as upos_type, c.y))
        } else {
            None
        }
    }

    pub fn q_next_word_start(mut mul: u16, state: &TextAreaState) -> Option<TextPosition> {
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

    pub fn q_prev_word_start(mut mul: u16, state: &TextAreaState) -> Option<TextPosition> {
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

    pub fn q_word_start(state: &TextAreaState) -> TextPosition {
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

    pub fn q_word_end(state: &TextAreaState) -> TextPosition {
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
                ct_event!(keycode press Esc) | ct_event!(key press CONTROL-'c') => {
                    vi.cancel();
                    self.scroll_cursor_to_visible();
                    display_search(self, vi);
                    TextOutcome::Changed
                }

                ct_event!(key press c)
                | ct_event!(key press SHIFT-c)
                | ct_event!(key press CONTROL_ALT-c) => {
                    if let Poll::Ready(vim) = vi.motion(*c) {
                        run_vim(vim, self, vi)?
                    } else {
                        TextOutcome::Changed
                    }
                }
                ct_event!(keycode press Enter) => {
                    if let Poll::Ready(vim) = vi.motion('\n') {
                        run_vim(vim, self, vi)?
                    } else {
                        TextOutcome::Changed
                    }
                }
                ct_event!(keycode press Backspace) => {
                    if let Poll::Ready(vim) = vi.motion(VI::BS) {
                        run_vim(vim, self, vi)?
                    } else {
                        TextOutcome::Changed
                    }
                }
                ct_event!(key press CONTROL-cc) => {
                    if let Poll::Ready(vim) = vi.motion(VI::ctrl(*cc)) {
                        run_vim(vim, self, vi)?
                    } else {
                        TextOutcome::Changed
                    }
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
