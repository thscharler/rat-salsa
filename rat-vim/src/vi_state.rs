use crate::vi_state::change_op::*;
use crate::vi_state::display::*;
use crate::vi_state::mark_op::set_mark;
use crate::vi_state::move_op::*;
use crate::vi_state::partial_op::*;
use crate::vi_state::scroll_op::*;
use crate::{Coroutine, Resume, SearchError, YieldPoint, ctrl, yield_};
use log::debug;
use rat_event::{HandleEvent, ct_event};
use rat_text::event::TextOutcome;
use rat_text::text_area::TextAreaState;
use rat_text::{TextPosition, upos_type};
use std::cell::RefCell;
use std::mem;
use std::ops::Range;
use std::rc::Rc;
use std::task::Poll;
use std::time::SystemTime;

pub struct VI {
    pub mode: Mode,

    pub motion_display: Rc<RefCell<String>>,
    pub motion: Coroutine<char, Vim>,

    pub command: Vim,
    pub text: String,

    pub finds: Finds,
    pub matches: Matches,

    pub marks: [Option<TextPosition>; 26],
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

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum SyncRanges {
    #[default]
    None,
    ToTextArea,
    FromTextArea,
}

#[derive(Debug, Default)]
pub struct Finds {
    pub term: Option<char>,
    pub row: upos_type,
    pub dir: Direction,
    pub till: bool,
    pub idx: Option<usize>,
    pub list: Vec<(Range<usize>, usize)>,
    pub sync: SyncRanges,
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
    pub list: Vec<(Range<usize>, usize)>,
    pub sync: SyncRanges,
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
    MoveToMatching,
    MoveToMark(char),

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

    FullLine,

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
    SearchRepeatNext,
    SearchRepeatPrev,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scrolling {
    Up,
    Down,
    HalfPageUp,
    HalfPageDown,
    PageUp,
    PageDown,
    MiddleOfScreen,
    TopOfScreen,
    BottomOfScreen,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Vim {
    #[default]
    Invalid,

    Repeat(u32),

    Partial(u32, Motion),
    Move(u32, Motion),
    Scroll(u32, Scrolling),
    Mark(char),

    Undo(u32),
    Redo(u32),

    JoinLines(u32),
    Insert(u32),
    Append(u32),
    AppendLine(u32),
    PrependLine(u32),
    Delete(u32, Motion),
}

fn is_memo(vim: &Vim) -> bool {
    match vim {
        Vim::Invalid => false,
        Vim::Repeat(_) => false,
        Vim::Partial(_, _) => false,
        Vim::Move(_, _) => false,
        Vim::Scroll(_, _) => false,
        Vim::Mark(_) => false,
        Vim::Undo(_) => false,
        Vim::Redo(_) => false,
        Vim::JoinLines(_) => true,
        Vim::Insert(_) => true,
        Vim::Append(_) => true,
        Vim::AppendLine(_) => true,
        Vim::PrependLine(_) => true,
        Vim::Delete(_, _) => true,
    }
}

impl Default for VI {
    fn default() -> Self {
        let motion_buf = Rc::new(RefCell::new(String::new()));
        Self {
            mode: Default::default(),
            motion_display: motion_buf.clone(),
            motion: Coroutine::new(|c, yp| Box::new(next_motion(c, motion_buf, yp))),
            command: Default::default(),
            text: Default::default(),
            finds: Default::default(),
            matches: Default::default(),
            marks: Default::default(),
            page: Default::default(),
        }
    }
}

fn reset_motion(vi: &mut VI) {
    vi.motion_display.borrow_mut().clear();
    let mb = vi.motion_display.clone();
    vi.motion = Coroutine::new(|c, yp| Box::new(next_motion(c, mb, yp)));
}

fn motion(vi: &mut VI, c: char) -> Poll<Vim> {
    match vi.motion.resume(c) {
        Resume::Yield(v) => {
            debug!("VIM !> {:?}", v);
            Poll::Ready(v)
        }
        Resume::Done(v) => {
            if matches!(v, Vim::Repeat(_)) {
                debug!("VIM |> {:?} {:?} {:?}", v, vi.command, vi.text);
            } else {
                debug!("VIM |> {:?}", v);
            }
            vi.motion_display.borrow_mut().clear();
            let mb = vi.motion_display.clone();
            vi.motion = Coroutine::new(|c, yp| Box::new(next_motion(c, mb, yp)));
            Poll::Ready(v)
        }
        Resume::Pending => {
            debug!("VIM ... {:?}", vi.motion_display.borrow());
            Poll::Pending
        }
    }
}

async fn bare_multiplier(
    mut tok: char,
    motion_buf: &RefCell<String>,
    yp: &YieldPoint<char, Vim>,
) -> (Option<u32>, char) {
    let mut mul = String::new();
    while tok.is_ascii_digit() || tok == ctrl::BS {
        if tok == ctrl::BS {
            mul.pop();
            motion_buf.borrow_mut().pop();
        } else {
            mul.push(tok);
            motion_buf.borrow_mut().push(tok);
        }
        tok = yield_!(yp);
    }
    let mul = mul.parse::<u32>().ok();

    (mul, tok)
}

async fn bare_motion(
    tok: char,
    mul: Option<u32>,
    motion_buf: &RefCell<String>,
    yp: &YieldPoint<char, Vim>,
) -> Result<Vim, char> {
    match tok {
        'h' => Ok(Vim::Move(mul.unwrap_or(1), Motion::MoveLeft)),
        'l' => Ok(Vim::Move(mul.unwrap_or(1), Motion::MoveRight)),
        'k' => Ok(Vim::Move(mul.unwrap_or(1), Motion::MoveUp)),
        'j' => Ok(Vim::Move(mul.unwrap_or(1), Motion::MoveDown)),
        '|' => Ok(Vim::Move(mul.unwrap_or(0), Motion::MoveToCol)),
        'w' => Ok(Vim::Move(mul.unwrap_or(1), Motion::MoveNextWordStart)),
        'b' => Ok(Vim::Move(mul.unwrap_or(1), Motion::MovePrevWordStart)),
        'e' => Ok(Vim::Move(mul.unwrap_or(1), Motion::MoveNextWordEnd)),
        'g' => {
            let tok = yield_!(yp);
            motion_buf.borrow_mut().push(tok);
            match tok {
                'e' => Ok(Vim::Move(mul.unwrap_or(1), Motion::MovePrevWordEnd)),
                'E' => Ok(Vim::Move(mul.unwrap_or(1), Motion::MovePrevWORDEnd)),
                '_' => Ok(Vim::Move(mul.unwrap_or(1), Motion::MoveEndOfLineText)),
                'g' => {
                    if let Some(mul) = mul {
                        Ok(Vim::Move(mul, Motion::MoveToLine))
                    } else {
                        Ok(Vim::Move(0, Motion::MoveStartOfFile))
                    }
                }
                _ => Ok(Vim::Invalid),
            }
        }
        'W' => Ok(Vim::Move(mul.unwrap_or(1), Motion::MoveNextWORDStart)),
        'B' => Ok(Vim::Move(mul.unwrap_or(1), Motion::MovePrevWORDStart)),
        'E' => Ok(Vim::Move(mul.unwrap_or(1), Motion::MoveNextWORDEnd)),
        '{' => Ok(Vim::Move(mul.unwrap_or(1), Motion::MovePrevParagraph)),
        '}' => Ok(Vim::Move(mul.unwrap_or(1), Motion::MoveNextParagraph)),
        'G' => {
            if let Some(mul) = mul {
                Ok(Vim::Move(mul, Motion::MoveToLine))
            } else {
                Ok(Vim::Move(0, Motion::MoveEndOfFile))
            }
        }
        '^' => Ok(Vim::Move(0, Motion::MoveStartOfLineText)),
        '$' => Ok(Vim::Move(mul.unwrap_or(1), Motion::MoveEndOfLine)),
        '%' => {
            if let Some(mul) = mul {
                Ok(Vim::Move(mul, Motion::MoveToLinePercent))
            } else {
                Ok(Vim::Move(0, Motion::MoveToMatching))
            }
        }

        'f' => {
            let tok = yield_!(yp);
            motion_buf.borrow_mut().push(tok);
            Ok(Vim::Move(mul.unwrap_or(1), Motion::FindForward(tok)))
        }
        'F' => {
            let tok = yield_!(yp);
            motion_buf.borrow_mut().push(tok);
            Ok(Vim::Move(mul.unwrap_or(1), Motion::FindBack(tok)))
        }
        't' => {
            let tok = yield_!(yp);
            motion_buf.borrow_mut().push(tok);
            Ok(Vim::Move(mul.unwrap_or(1), Motion::FindTillForward(tok)))
        }
        'T' => {
            let tok = yield_!(yp);
            motion_buf.borrow_mut().push(tok);
            Ok(Vim::Move(mul.unwrap_or(1), Motion::FindTillBack(tok)))
        }
        ';' => Ok(Vim::Move(mul.unwrap_or(1), Motion::FindRepeatNext)),
        ',' => Ok(Vim::Move(mul.unwrap_or(1), Motion::FindRepeatPrev)),

        '/' => {
            let mut buf = String::new();
            loop {
                let tok = yield_!(
                    Vim::Partial(mul.unwrap_or(1), Motion::SearchForward(buf.clone()),),
                    yp
                );
                if tok == '\n' {
                    break;
                } else if tok == ctrl::BS {
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
            Ok(Vim::Move(mul.unwrap_or(1), Motion::SearchForward(buf)))
        }
        '?' => {
            let mut buf = String::new();
            loop {
                let tok = yield_!(
                    Vim::Partial(mul.unwrap_or(1), Motion::SearchBack(buf.clone()),),
                    yp
                );
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
            Ok(Vim::Move(mul.unwrap_or(1), Motion::SearchBack(buf)))
        }
        '*' => Ok(Vim::Move(mul.unwrap_or(1), Motion::SearchWordForward)),
        '#' => Ok(Vim::Move(mul.unwrap_or(1), Motion::SearchWordBackward)),
        'n' => Ok(Vim::Move(mul.unwrap_or(1), Motion::SearchRepeatNext)),
        'N' => Ok(Vim::Move(mul.unwrap_or(1), Motion::SearchRepeatPrev)),

        '\'' => {
            let tok = yield_!(yp);
            motion_buf.borrow_mut().push(tok);
            Ok(Vim::Move(0, Motion::MoveToMark(tok)))
        }

        _ => Err(tok),
    }
}

async fn bare_scroll(
    tok: char,
    mul: Option<u32>,
    motion_buf: &RefCell<String>,
    yp: &YieldPoint<char, Vim>,
) -> Result<Vim, char> {
    match tok {
        ctrl::CTRL_U => Ok(Vim::Scroll(mul.unwrap_or(0), Scrolling::HalfPageUp)),
        ctrl::CTRL_D => Ok(Vim::Scroll(mul.unwrap_or(0), Scrolling::HalfPageDown)),
        'z' => {
            let tok = yield_!(yp);
            motion_buf.borrow_mut().push(tok);
            match tok {
                'z' => Ok(Vim::Scroll(0, Scrolling::MiddleOfScreen)),
                't' => Ok(Vim::Scroll(0, Scrolling::TopOfScreen)),
                'b' => Ok(Vim::Scroll(0, Scrolling::BottomOfScreen)),
                _ => Ok(Vim::Invalid),
            }
        }
        ctrl::CTRL_Y => Ok(Vim::Scroll(mul.unwrap_or(1), Scrolling::Up)),
        ctrl::CTRL_E => Ok(Vim::Scroll(mul.unwrap_or(1), Scrolling::Down)),
        ctrl::CTRL_B => Ok(Vim::Scroll(mul.unwrap_or(1), Scrolling::PageUp)),
        ctrl::CTRL_F => Ok(Vim::Scroll(mul.unwrap_or(1), Scrolling::PageDown)),
        _ => Err(tok),
    }
}

async fn next_motion(
    mut tok: char,
    motion_buf: Rc<RefCell<String>>,
    yp: YieldPoint<char, Vim>,
) -> Vim {
    if tok == '0' {
        return Vim::Move(0, Motion::MoveStartOfLine);
    }

    let mul;
    (mul, tok) = bare_multiplier(tok, &motion_buf, &yp).await;

    motion_buf.borrow_mut().push(tok);
    tok = match bare_motion(tok, mul, &motion_buf, &yp).await {
        Ok(v @ Vim::Move(_, _)) => {
            return v;
        }
        Ok(Vim::Invalid) => return Vim::Invalid,
        Err(tok) => tok,
        _ => unreachable!("no"),
    };
    tok = match bare_scroll(tok, mul, &motion_buf, &yp).await {
        Ok(v @ Vim::Scroll(_, _)) => return v,
        Ok(Vim::Invalid) => return Vim::Invalid,
        Err(tok) => tok,
        _ => unreachable!("no"),
    };

    match tok {
        'm' => {
            tok = yield_!(yp);
            motion_buf.borrow_mut().push(tok);
            Vim::Mark(tok)
        }

        'd' => {
            tok = yield_!(yp);

            let mul2;
            (mul2, tok) = bare_multiplier(tok, &motion_buf, &yp).await;

            motion_buf.borrow_mut().push(tok);
            tok = match bare_motion(tok, mul2, &motion_buf, &yp).await {
                Ok(Vim::Move(mul2, motion)) => {
                    let mul = mul.unwrap_or(1);
                    return Vim::Delete(mul * mul2, motion);
                }
                Ok(Vim::Invalid) => return Vim::Invalid,
                Err(tok) => tok,
                _ => unreachable!("no"),
            };
            match tok {
                'd' => Vim::Delete(mul.unwrap_or(1), Motion::FullLine),
                _ => Vim::Invalid,
            }
        }
        'i' => Vim::Insert(mul.unwrap_or(1)),
        'a' => Vim::Append(mul.unwrap_or(1)),
        'o' => Vim::AppendLine(mul.unwrap_or(1)),
        'O' => Vim::PrependLine(mul.unwrap_or(1)),
        'x' => Vim::Delete(mul.unwrap_or(1), Motion::MoveRight),
        'X' => Vim::Delete(mul.unwrap_or(1), Motion::MoveLeft),
        'J' => Vim::JoinLines(mul.unwrap_or(1)),
        'u' => Vim::Undo(mul.unwrap_or(1)),
        ctrl::CTRL_R => Vim::Redo(mul.unwrap_or(1)),

        '.' => Vim::Repeat(mul.unwrap_or(1)),

        _ => Vim::Invalid,
    }
}

fn eval_insert(cc: char, state: &mut TextAreaState, vi: &mut VI) -> TextOutcome {
    let r = insert_char(cc, state, vi);
    display_matches(state, vi);
    display_finds(state, vi);
    r
}

fn eval_normal(
    cc: char,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Result<TextOutcome, SearchError> {
    match motion(vi, cc) {
        Poll::Ready(Vim::Repeat(mut mul)) => {
            assert!(mul > 0);
            let tt = SystemTime::now();
            let vim = mem::take(&mut vi.command);
            let r = loop {
                let rr = match execute_vim(&vim, true, state, vi) {
                    Ok(v) => Ok(v),
                    Err(e) => {
                        vi.command = vim;
                        break Err(e);
                    }
                };

                mul -= 1;
                if mul == 0 {
                    vi.command = vim;
                    break rr;
                }
            };
            debug!("TT {:?}", tt.elapsed());
            r
        }
        Poll::Ready(vim) => {
            let tt = SystemTime::now();
            let r = match execute_vim(&vim, false, state, vi) {
                Ok(v) => {
                    if is_memo(&vim) {
                        vi.command = vim;
                    }
                    Ok(v)
                }
                Err(e) => Err(e),
            };
            debug!("TT {:?}", tt.elapsed());
            r
        }
        Poll::Pending => Ok(TextOutcome::Changed),
    }
}

fn execute_vim(
    vim: &Vim,
    repeat: bool,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Result<TextOutcome, SearchError> {
    match vim {
        Vim::Invalid => return Ok(TextOutcome::Unchanged),
        Vim::Repeat(_) => unreachable!("wrong spot for repeat"),

        Vim::Move(mul, m) => {
            move_cursor(*mul, m, state, vi)?;
        }

        Vim::Partial(mul, Motion::SearchForward(s)) => {
            search_fwd(*mul, s, true, state, vi)?;
            scroll_to_search_idx(state, vi);
        }
        Vim::Partial(mul, Motion::SearchBack(s)) => {
            search_back(*mul, s, true, state, vi)?;
            scroll_to_search_idx(state, vi);
        }
        Vim::Partial(_, _) => unreachable!("unknown partial"),

        Vim::Scroll(mul, Scrolling::HalfPageUp) => scroll_half_page_up(*mul, state, vi),
        Vim::Scroll(mul, Scrolling::HalfPageDown) => scroll_half_page_down(*mul, state, vi),
        Vim::Scroll(mul, Scrolling::PageUp) => scroll_page_up(*mul, state, vi),
        Vim::Scroll(mul, Scrolling::PageDown) => scroll_page_down(*mul, state, vi),
        Vim::Scroll(mul, Scrolling::Up) => scroll_up(*mul, state),
        Vim::Scroll(mul, Scrolling::Down) => scroll_down(*mul, state),
        Vim::Scroll(_, Scrolling::MiddleOfScreen) => scroll_cursor_to_middle(state),
        Vim::Scroll(_, Scrolling::TopOfScreen) => scroll_cursor_to_top(state),
        Vim::Scroll(_, Scrolling::BottomOfScreen) => scroll_cursor_to_bottom(state),

        Vim::Mark(mark) => set_mark(*mark, state, vi),

        Vim::Undo(mul) => {
            return Ok(undo(*mul, state, vi));
        }
        Vim::Redo(mul) => {
            return Ok(redo(*mul, state, vi));
        }
        Vim::JoinLines(mul) => {
            return Ok(join_line(*mul, state, vi));
        }
        Vim::Insert(mul) => {
            if repeat {
                return Ok(insert_str(*mul, state, vi));
            } else {
                begin_insert(vi);
            }
        }
        Vim::Append(mul) => {
            if repeat {
                return Ok(append_str(*mul, state, vi));
            } else {
                begin_append(state, vi);
            }
        }
        Vim::AppendLine(mul) => {
            if repeat {
                return Ok(append_line_str(*mul, state, vi));
            } else {
                begin_append_line(state, vi);
            }
        }
        Vim::PrependLine(mul) => {
            if repeat {
                return Ok(prepend_line_str(*mul, state, vi));
            } else {
                begin_prepend_line(state, vi);
            }
        }
        Vim::Delete(mul, m) => {
            delete_text(*mul, m, state, vi)?;
        }
    }

    display_matches(state, vi);
    display_finds(state, vi);

    Ok(TextOutcome::Changed)
}

mod display {
    use crate::vi_state::query::{q_find_idx, q_search_idx};
    use crate::vi_state::{Direction, SyncRanges, VI};
    use log::debug;
    use rat_text::text_area::TextAreaState;

    pub fn display_matches(state: &mut TextAreaState, vi: &mut VI) {
        match vi.matches.sync {
            SyncRanges::None => {}
            SyncRanges::ToTextArea => {
                debug!("sync matches ->");
                vi.matches.sync = SyncRanges::None;
                state.remove_style_fully(999);
                for r in &vi.matches.list {
                    state.add_style(r.0.clone(), 999)
                }
            }
            SyncRanges::FromTextArea => {
                debug!("sync matches <-");
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
                debug!("sync finds ->");
                vi.finds.sync = SyncRanges::None;
                state.remove_style_fully(998);
                for r in &vi.finds.list {
                    state.add_style(r.0.clone(), 998)
                }
            }
            SyncRanges::FromTextArea => {
                debug!("sync finds <-");
                vi.finds.sync = SyncRanges::None;
                vi.finds.list.clear();
                state.styles_in_match(0..state.len_bytes(), 998, &mut vi.finds.list);
                q_find_idx(&mut vi.finds, 1, Direction::Forward, state);
            }
        }
    }
}

mod scroll_op {
    use crate::vi_state::VI;
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
                state.set_offset((0, no.y as usize));
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
                state.set_offset((0, no.y as usize));
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
                state.set_offset((0, no.y as usize));
            } else {
                // ???
            }
        } else {
            state.scroll_cursor_to_visible();
        }
    }

    pub fn scroll_up(mul: u32, state: &mut TextAreaState) {
        state.scroll_up(mul as usize);
    }

    pub fn scroll_down(mul: u32, state: &mut TextAreaState) {
        state.scroll_down(mul as usize);
    }

    pub fn scroll_page_up(mul: u32, state: &mut TextAreaState, vi: &mut VI) {
        if vi.page.0 != state.vertical_page() as u16 {
            vi.page = (
                state.vertical_page() as u16,
                (state.vertical_page() / 2) as u16,
            );
        }

        state.scroll_up((vi.page.0 as u32 * mul).saturating_sub(2) as usize);
    }

    pub fn scroll_page_down(mul: u32, state: &mut TextAreaState, vi: &mut VI) {
        if vi.page.0 != state.vertical_page() as u16 {
            vi.page = (
                state.vertical_page() as u16,
                (state.vertical_page() / 2) as u16,
            );
        }

        state.scroll_down((vi.page.0 as u32 * mul).saturating_sub(2) as usize);
    }

    pub fn scroll_half_page_up(mul: u32, state: &mut TextAreaState, vi: &mut VI) {
        if vi.page.0 != state.vertical_page() as u16 {
            vi.page = (
                state.vertical_page() as u16,
                (state.vertical_page() / 2) as u16,
            );
        }
        if mul != 0 {
            vi.page.1 = mul as u16;
        }

        state.move_up(vi.page.1, false);
    }

    pub fn scroll_half_page_down(mul: u32, state: &mut TextAreaState, vi: &mut VI) {
        if vi.page.0 != state.vertical_page() as u16 {
            vi.page = (
                state.vertical_page() as u16,
                (state.vertical_page() / 2) as u16,
            );
        }
        if mul != 0 {
            vi.page.1 = mul as u16;
        }

        state.move_down(vi.page.1, false);
    }
}

mod move_op {
    use crate::SearchError;
    use crate::vi_state::query::*;
    use crate::vi_state::{Motion, VI};
    use rat_text::TextPosition;
    use rat_text::text_area::TextAreaState;

    pub fn move_position(
        mul: u32,
        motion: &Motion,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Result<Option<TextPosition>, SearchError> {
        Ok(match motion {
            Motion::MoveLeft => q_move_left(mul, state),
            Motion::MoveRight => q_move_right(mul, state),
            Motion::MoveUp => q_move_up(mul, state),
            Motion::MoveDown => q_move_down(mul, state),
            Motion::MoveToCol => q_col(mul, state),
            Motion::MoveToLine => q_line(mul, state),
            Motion::MoveToLinePercent => q_line_percent(mul, state),
            Motion::MoveToMatching => q_matching_brace(state),
            Motion::MoveToMark(mark) => q_mark_pos(*mark, &vi.marks),
            Motion::MoveStartOfFile => q_start_of_file(),
            Motion::MoveEndOfFile => q_end_of_file(state),
            Motion::MoveNextWordStart => q_next_word_start(mul, state),
            Motion::MovePrevWordStart => q_prev_word_start(mul, state),
            Motion::MoveNextWordEnd => q_next_word_end(mul, state),
            Motion::MovePrevWordEnd => q_prev_word_end(mul, state),
            Motion::MoveNextWORDStart => q_next_bigword_start(mul, state),
            Motion::MovePrevWORDStart => q_prev_bigword_start(mul, state),
            Motion::MoveNextWORDEnd => q_next_bigword_end(mul, state),
            Motion::MovePrevWORDEnd => q_prev_bigword_end(mul, state),
            Motion::MoveStartOfLine => q_start_of_line(state),
            Motion::MoveEndOfLine => q_end_of_line(mul, state),
            Motion::MoveStartOfLineText => q_start_of_text(state),
            Motion::MoveEndOfLineText => q_end_of_text(mul, state),
            Motion::MovePrevParagraph => q_prev_paragraph(mul, state),
            Motion::MoveNextParagraph => q_next_paragraph(mul, state),
            Motion::FindForward(f) => q_find_fwd(mul, *f, state, vi),
            Motion::FindBack(f) => q_find_back(mul, *f, state, vi),
            Motion::FindTillForward(f) => q_till_fwd(mul, *f, state, vi),
            Motion::FindTillBack(f) => q_till_back(mul, *f, state, vi),
            Motion::FindRepeatNext => q_find_repeat_fwd(mul, state, vi),
            Motion::FindRepeatPrev => q_find_repeat_back(mul, state, vi),
            Motion::SearchWordForward => q_search_word_fwd(mul, state, vi)?,
            Motion::SearchWordBackward => q_search_word_back(mul, state, vi)?,
            Motion::SearchForward(term) => q_search_fwd(mul, &term, false, state, vi)?,
            Motion::SearchBack(term) => q_search_back(mul, &term, false, state, vi)?,
            Motion::SearchRepeatNext => q_search_repeat_fwd(mul, state, vi),
            Motion::SearchRepeatPrev => q_search_repeat_back(mul, state, vi),
            Motion::FullLine => q_start_of_next_line(mul, state),
        })
    }

    pub fn move_cursor(
        mul: u32,
        motion: &Motion,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Result<(), SearchError> {
        if let Some(npos) = move_position(mul, motion, state, vi)? {
            state.set_cursor(npos, false);
        }
        Ok(())
    }
}

mod partial_op {
    use crate::SearchError;
    use crate::vi_state::query::*;
    use crate::vi_state::{Direction, VI};
    use rat_text::text_area::TextAreaState;

    pub fn search_back(
        mul: u32,
        term: &str,
        tmp: bool,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Result<(), SearchError> {
        q_search(&mut vi.matches, term, Direction::Backward, tmp, state)?;
        q_search_idx(&mut vi.matches, mul, Direction::Forward, state);
        Ok(())
    }

    pub fn search_fwd(
        mul: u32,
        term: &str,
        tmp: bool,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Result<(), SearchError> {
        q_search(&mut vi.matches, term, Direction::Forward, tmp, state)?;
        q_search_idx(&mut vi.matches, mul, Direction::Forward, state);
        Ok(())
    }
}

mod mark_op {
    use crate::VI;
    use crate::vi_state::query::*;
    use rat_text::text_area::TextAreaState;

    pub fn set_mark(mark: char, state: &mut TextAreaState, vi: &mut VI) {
        if let Some(mark) = q_mark_idx(mark) {
            vi.marks[mark] = Some(state.cursor());
        }
    }
}

mod change_op {
    use crate::vi_state::query::*;
    use crate::vi_state::{Motion, SyncRanges, Vim};
    use crate::{Mode, SearchError, VI};
    use rat_text::TextPosition;
    use rat_text::event::TextOutcome;
    use rat_text::text_area::TextAreaState;
    use std::mem;
    use std::ops::Range;

    pub fn end_insert_mode(state: &mut TextAreaState, vi: &mut VI) -> TextOutcome {
        vi.mode = Mode::Normal;

        let command = mem::take(&mut vi.command);
        let r = match &command {
            Vim::Insert(mul) => {
                insert_str(mul.saturating_sub(1), state, vi);
                TextOutcome::TextChanged
            }
            Vim::Append(mul) => {
                append_str(mul.saturating_sub(1), state, vi);
                TextOutcome::TextChanged
            }
            _ => TextOutcome::Changed,
        };
        vi.command = command;

        r
    }

    pub fn begin_prepend_line(state: &mut TextAreaState, vi: &mut VI) {
        vi.mode = Mode::Insert;
        vi.finds.sync = SyncRanges::FromTextArea;
        vi.matches.sync = SyncRanges::FromTextArea;
        vi.text.clear();

        let c = state.cursor();
        state.set_cursor(TextPosition::new(0, c.y), false);
        state.insert_newline();
        state.set_cursor(TextPosition::new(0, c.y), false);
    }

    pub fn prepend_line_str(mut mul: u32, state: &mut TextAreaState, vi: &mut VI) -> TextOutcome {
        if mul > 0 {
            vi.finds.sync = SyncRanges::FromTextArea;
            vi.matches.sync = SyncRanges::FromTextArea;
        }
        while mul > 0 {
            q_prepend_line_str(&vi.text, state);
            mul -= 1;
        }
        TextOutcome::TextChanged
    }

    pub fn begin_append_line(state: &mut TextAreaState, vi: &mut VI) {
        vi.mode = Mode::Insert;
        vi.finds.sync = SyncRanges::FromTextArea;
        vi.matches.sync = SyncRanges::FromTextArea;
        vi.text.clear();

        let c = state.cursor();
        let width = state.line_width(c.y);
        state.set_cursor(TextPosition::new(width, c.y), false);
        state.insert_newline();
    }

    pub fn append_line_str(mut mul: u32, state: &mut TextAreaState, vi: &mut VI) -> TextOutcome {
        if mul > 0 {
            vi.finds.sync = SyncRanges::FromTextArea;
            vi.matches.sync = SyncRanges::FromTextArea;
        }
        while mul > 0 {
            q_append_line_str(&vi.text, state);
            mul -= 1;
        }
        TextOutcome::TextChanged
    }

    pub fn begin_append(state: &mut TextAreaState, vi: &mut VI) {
        vi.mode = Mode::Insert;
        vi.text.clear();

        state.move_right(1, false);
    }

    pub fn append_str(mut mul: u32, state: &mut TextAreaState, vi: &mut VI) -> TextOutcome {
        if mul > 0 {
            vi.finds.sync = SyncRanges::FromTextArea;
            vi.matches.sync = SyncRanges::FromTextArea;
        }
        while mul > 0 {
            q_append_str(&vi.text, state);
            mul -= 1;
        }
        TextOutcome::TextChanged
    }

    pub fn begin_insert(vi: &mut VI) {
        vi.mode = Mode::Insert;
        vi.text.clear();
    }

    pub fn insert_str(mut mul: u32, state: &mut TextAreaState, vi: &mut VI) -> TextOutcome {
        if mul > 0 {
            vi.finds.sync = SyncRanges::FromTextArea;
            vi.matches.sync = SyncRanges::FromTextArea;
        }
        while mul > 0 {
            q_insert_str(&vi.text, state);
            mul -= 1;
        }
        TextOutcome::TextChanged
    }

    pub fn insert_char(cc: char, state: &mut TextAreaState, vi: &mut VI) -> TextOutcome {
        vi.finds.sync = SyncRanges::FromTextArea;
        vi.matches.sync = SyncRanges::FromTextArea;
        vi.text.push(cc);
        q_insert(cc, state);
        TextOutcome::TextChanged
    }

    pub fn change_range(
        mul: u32,
        motion: &Motion,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Result<Option<Range<TextPosition>>, SearchError> {
        let start = match motion {
            Motion::FullLine => q_start_of_line(state).expect("start_of_line"),
            _ => state.cursor(),
        };
        let end = match motion {
            Motion::MoveLeft => q_move_left(mul, state),
            Motion::MoveRight => q_move_right(mul, state),
            Motion::MoveUp => q_move_up(mul, state),
            Motion::MoveDown => q_move_down(mul, state),
            Motion::MoveToCol => q_col(mul, state),
            Motion::MoveToLine => q_line(mul, state),
            Motion::MoveToLinePercent => q_line_percent(mul, state),
            Motion::MoveToMatching => q_matching_brace(state),
            Motion::MoveToMark(mark) => q_mark_pos(*mark, &vi.marks),
            Motion::MoveStartOfFile => q_start_of_file(),
            Motion::MoveEndOfFile => q_end_of_file(state),
            Motion::MoveNextWordStart => q_next_word_start(mul, state),
            Motion::MovePrevWordStart => q_prev_word_start(mul, state),
            Motion::MoveNextWordEnd => q_next_word_end(mul, state),
            Motion::MovePrevWordEnd => q_prev_word_end(mul, state),
            Motion::MoveNextWORDStart => q_next_bigword_start(mul, state),
            Motion::MovePrevWORDStart => q_prev_bigword_start(mul, state),
            Motion::MoveNextWORDEnd => q_next_bigword_end(mul, state),
            Motion::MovePrevWORDEnd => q_prev_bigword_end(mul, state),
            Motion::MoveStartOfLine => q_start_of_line(state),
            Motion::MoveEndOfLine => q_end_of_line(mul, state),
            Motion::MoveStartOfLineText => q_start_of_text(state),
            Motion::MoveEndOfLineText => q_end_of_text(mul, state),
            Motion::MovePrevParagraph => q_prev_paragraph(mul, state),
            Motion::MoveNextParagraph => q_next_paragraph(mul, state),
            Motion::FindForward(f) => q_find_fwd(mul, *f, state, vi),
            Motion::FindBack(f) => q_find_back(mul, *f, state, vi),
            Motion::FindTillForward(f) => q_till_fwd(mul, *f, state, vi),
            Motion::FindTillBack(f) => q_till_back(mul, *f, state, vi),
            Motion::FindRepeatNext => q_find_repeat_fwd(mul, state, vi),
            Motion::FindRepeatPrev => q_find_repeat_back(mul, state, vi),
            Motion::SearchWordForward => q_search_word_fwd(mul, state, vi)?,
            Motion::SearchWordBackward => q_search_word_back(mul, state, vi)?,
            Motion::SearchForward(term) => q_search_fwd(mul, &term, false, state, vi)?,
            Motion::SearchBack(term) => q_search_back(mul, &term, false, state, vi)?,
            Motion::SearchRepeatNext => q_search_repeat_fwd(mul, state, vi),
            Motion::SearchRepeatPrev => q_search_repeat_back(mul, state, vi),
            Motion::FullLine => q_start_of_next_line(mul, state),
        };
        if let Some(end) = end {
            if start > end {
                Ok(Some(end..start))
            } else {
                Ok(Some(start..end))
            }
        } else {
            Ok(None)
        }
    }

    pub fn delete_text(
        mul: u32,
        motion: &Motion,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Result<(), SearchError> {
        if let Some(range) = change_range(mul, motion, state, vi)? {
            vi.finds.sync = SyncRanges::FromTextArea;
            vi.matches.sync = SyncRanges::FromTextArea;
            state.delete_range(range);
        }
        Ok(())
    }

    pub fn join_line(mut mul: u32, state: &mut TextAreaState, vi: &mut VI) -> TextOutcome {
        vi.finds.sync = SyncRanges::FromTextArea;
        vi.matches.sync = SyncRanges::FromTextArea;

        while mul > 0 {
            let range = q_line_break_and_leading_space(state);
            if !range.is_empty() {
                state.set_selection(range.start, range.end);
                state.insert_char(' ');
            }

            mul -= 1;
        }
        TextOutcome::TextChanged
    }

    pub fn undo(mut mul: u32, state: &mut TextAreaState, vi: &mut VI) -> TextOutcome {
        vi.finds.sync = SyncRanges::FromTextArea;
        vi.matches.sync = SyncRanges::FromTextArea;

        while mul > 0 {
            if !state.undo() {
                break;
            }

            mul -= 1;
        }
        TextOutcome::TextChanged
    }

    pub fn redo(mut mul: u32, state: &mut TextAreaState, vi: &mut VI) -> TextOutcome {
        vi.finds.sync = SyncRanges::FromTextArea;
        vi.matches.sync = SyncRanges::FromTextArea;

        while mul > 0 {
            if !state.redo() {
                break;
            }

            mul -= 1;
        }
        TextOutcome::TextChanged
    }
}

mod query {
    use crate::vi_state::{Direction, Finds, Matches, SyncRanges};
    use crate::{SearchError, VI, ctrl};
    use rat_text::text_area::TextAreaState;
    use rat_text::{Cursor, Grapheme, TextPosition, TextRange, upos_type};
    use regex_cursor::engines::dfa::{Regex, find_iter};
    use regex_cursor::{Input, RopeyCursor};
    use std::cmp::min;

    pub fn q_move_left(mul: u32, state: &mut TextAreaState) -> Option<TextPosition> {
        let mut pos = state.cursor();
        if pos.x == 0 {
            if pos.y > 0 {
                pos.y = pos.y.saturating_sub(1);
                pos.x = state.line_width(pos.y);
            }
        } else {
            pos.x = pos.x.saturating_sub(mul as upos_type);
        }
        Some(pos)
    }

    pub fn q_move_right(mul: u32, state: &mut TextAreaState) -> Option<TextPosition> {
        let mut pos = state.cursor();
        let c_line_width = state.line_width(pos.y);
        if pos.x == c_line_width {
            if pos.y + 1 < state.len_lines() {
                pos.y += 1;
                pos.x = 0;
            }
        } else {
            pos.x = min(pos.x + mul as upos_type, c_line_width)
        }
        Some(pos)
    }

    pub fn q_move_up(mul: u32, state: &mut TextAreaState) -> Option<TextPosition> {
        let pos = state.cursor();
        if let Some(mut scr_cursor) = state.pos_to_relative_screen(pos) {
            scr_cursor.1 -= mul as i16;
            if let Some(npos) = state.relative_screen_to_pos(scr_cursor) {
                Some(npos)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn q_move_down(mul: u32, state: &mut TextAreaState) -> Option<TextPosition> {
        let pos = state.cursor();
        if let Some(mut scr_cursor) = state.pos_to_relative_screen(pos) {
            scr_cursor.1 += mul as i16;

            if let Some(npos) = state.relative_screen_to_pos(scr_cursor) {
                Some(npos)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn q_col(mul: u32, state: &TextAreaState) -> Option<TextPosition> {
        let c = state.cursor();
        if mul as upos_type <= state.line_width(c.y) {
            Some(TextPosition::new(mul as upos_type, c.y))
        } else {
            None
        }
    }

    pub fn q_line(mul: u32, state: &TextAreaState) -> Option<TextPosition> {
        let line = min(
            mul.saturating_sub(1) as upos_type,
            state.len_lines().saturating_sub(1),
        );
        Some(TextPosition::new(0, line))
    }

    pub fn q_line_percent(mul: u32, state: &TextAreaState) -> Option<TextPosition> {
        let len = state.len_lines() as u64;
        let pc = min(mul.saturating_sub(1), 100) as u64;
        let line = ((len * pc) / 100) as u32;
        Some(TextPosition::new(0, line))
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

    pub fn q_mark_pos(mark: char, marks: &[Option<TextPosition>; 26]) -> Option<TextPosition> {
        if let Some(mark) = q_mark_idx(mark) {
            marks[mark]
        } else {
            None
        }
    }

    pub fn q_mark_idx(mark: char) -> Option<usize> {
        let mark = mark.to_ascii_lowercase();
        if mark >= 'a' && mark <= 'z' {
            Some(mark as usize - 'a' as usize)
        } else {
            None
        }
    }

    pub fn q_start_of_file() -> Option<TextPosition> {
        Some(TextPosition::new(0, 0))
    }

    pub fn q_end_of_file(state: &mut TextAreaState) -> Option<TextPosition> {
        let y = state.len_lines().saturating_sub(1);
        Some(TextPosition::new(state.line_width(y), y))
    }

    pub fn q_next_word_start(mut mul: u32, state: &TextAreaState) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        while mul > 0 {
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
        }

        Some(state.byte_pos(it.text_offset()))
    }

    pub fn q_prev_word_start(mut mul: u32, state: &TextAreaState) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        while mul > 0 {
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
        }

        Some(state.byte_pos(it.text_offset()))
    }

    pub fn q_next_word_end(mut mul: u32, state: &TextAreaState) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        while mul > 0 {
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
        }

        Some(state.byte_pos(it.text_offset()))
    }

    pub fn q_prev_word_end(mut mul: u32, state: &TextAreaState) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        while mul > 0 {
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
        }

        Some(state.byte_pos(it.text_offset()))
    }

    pub fn q_next_bigword_start(mut mul: u32, state: &TextAreaState) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        while mul > 0 {
            let Some(sample) = it.peek_next() else {
                return None;
            };
            if !sample.is_whitespace() {
                skip_nonwhite(&mut it);
            }
            skip_white(&mut it);

            mul -= 1;
        }

        Some(state.byte_pos(it.text_offset()))
    }

    pub fn q_prev_bigword_start(mut mul: u32, state: &TextAreaState) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        while mul > 0 {
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
        }

        Some(state.byte_pos(it.text_offset()))
    }

    pub fn q_next_bigword_end(mut mul: u32, state: &TextAreaState) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        while mul > 0 {
            skip_white(&mut it);

            let Some(sample) = it.peek_next() else {
                return None;
            };
            if !sample.is_whitespace() {
                skip_nonwhite(&mut it);
            }

            mul -= 1;
        }

        Some(state.byte_pos(it.text_offset()))
    }

    pub fn q_prev_bigword_end(mut mul: u32, state: &TextAreaState) -> Option<TextPosition> {
        let mut it = state.text_graphemes(state.cursor());

        while mul > 0 {
            let Some(sample) = it.peek_prev() else {
                return None;
            };
            if !sample.is_whitespace() {
                pskip_nonwhite(&mut it);
            }
            pskip_white(&mut it);

            mul -= 1;
        }

        Some(state.byte_pos(it.text_offset()))
    }

    pub fn q_start_of_line(state: &mut TextAreaState) -> Option<TextPosition> {
        Some(TextPosition::new(0, state.cursor().y))
    }

    pub fn q_start_of_next_line(mul: u32, state: &mut TextAreaState) -> Option<TextPosition> {
        let y = min(state.cursor().y + mul as upos_type, state.len_lines());
        Some(TextPosition::new(0, y))
    }

    pub fn q_end_of_line(mul: u32, state: &mut TextAreaState) -> Option<TextPosition> {
        let y = min(
            state.cursor().y + mul.saturating_sub(1) as upos_type,
            state.len_lines().saturating_sub(1),
        );
        Some(TextPosition::new(state.line_width(y), y))
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

    pub fn q_end_of_text(mul: u32, state: &mut TextAreaState) -> Option<TextPosition> {
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

    pub fn q_prev_paragraph(mut mul: u32, state: &mut TextAreaState) -> Option<TextPosition> {
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

    pub fn q_next_paragraph(mut mul: u32, state: &mut TextAreaState) -> Option<TextPosition> {
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

    pub fn q_find_fwd(
        mul: u32,
        term: char,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Option<TextPosition> {
        q_find(&mut vi.finds, term, Direction::Forward, false, state);
        q_find_idx(&mut vi.finds, mul, Direction::Forward, state);

        if let Some(i) = vi.finds.idx {
            let pos = state.byte_pos(vi.finds.list[i].0.end);
            Some(pos)
        } else {
            None
        }
    }

    pub fn q_find_back(
        mul: u32,
        term: char,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Option<TextPosition> {
        q_find(&mut vi.finds, term, Direction::Backward, false, state);
        q_find_idx(&mut vi.finds, mul, Direction::Forward, state);

        if let Some(i) = vi.finds.idx {
            let pos = state.byte_pos(vi.finds.list[i].0.start);
            Some(pos)
        } else {
            None
        }
    }

    pub fn q_till_fwd(
        mul: u32,
        term: char,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Option<TextPosition> {
        q_find(&mut vi.finds, term, Direction::Forward, true, state);
        q_find_idx(&mut vi.finds, mul, Direction::Forward, state);

        if let Some(i) = vi.finds.idx {
            let pos = state.byte_pos(vi.finds.list[i].0.start);
            Some(pos)
        } else {
            None
        }
    }

    pub fn q_till_back(
        mul: u32,
        term: char,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Option<TextPosition> {
        q_find(&mut vi.finds, term, Direction::Backward, true, state);
        q_find_idx(&mut vi.finds, mul, Direction::Forward, state);

        if let Some(i) = vi.finds.idx {
            let pos = state.byte_pos(vi.finds.list[i].0.end);
            Some(pos)
        } else {
            None
        }
    }

    pub fn q_find_repeat_fwd(
        mul: u32,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Option<TextPosition> {
        let Some(last_term) = vi.finds.term else {
            return None;
        };

        let last_dir = vi.finds.dir;
        let last_till = vi.finds.till;

        q_find(&mut vi.finds, last_term, last_dir, last_till, state);
        q_find_idx(&mut vi.finds, mul, Direction::Forward, state);

        let dir = vi.finds.dir.mul(Direction::Forward);

        if let Some(idx) = vi.finds.idx {
            let pos = if vi.finds.till {
                if dir == Direction::Backward {
                    state.byte_pos(vi.finds.list[idx].0.end)
                } else {
                    state.byte_pos(vi.finds.list[idx].0.start)
                }
            } else {
                if dir == Direction::Backward {
                    state.byte_pos(vi.finds.list[idx].0.start)
                } else {
                    state.byte_pos(vi.finds.list[idx].0.end)
                }
            };
            Some(pos)
        } else {
            None
        }
    }

    pub fn q_find_repeat_back(
        mul: u32,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Option<TextPosition> {
        let Some(last_term) = vi.finds.term else {
            return None;
        };

        let last_dir = vi.finds.dir;
        let last_till = vi.finds.till;

        q_find(&mut vi.finds, last_term, last_dir, last_till, state);
        q_find_idx(&mut vi.finds, mul, Direction::Backward, state);

        let dir = vi.finds.dir.mul(Direction::Backward);

        if let Some(idx) = vi.finds.idx {
            let pos = if vi.finds.till {
                if dir == Direction::Backward {
                    state.byte_pos(vi.finds.list[idx].0.end)
                } else {
                    state.byte_pos(vi.finds.list[idx].0.start)
                }
            } else {
                if dir == Direction::Backward {
                    state.byte_pos(vi.finds.list[idx].0.start)
                } else {
                    state.byte_pos(vi.finds.list[idx].0.end)
                }
            };
            Some(pos)
        } else {
            None
        }
    }

    pub fn q_find_idx(finds: &mut Finds, mul: u32, dir: Direction, state: &mut TextAreaState) {
        let mut c = state.byte_at(state.cursor()).start;

        let dir = finds.dir.mul(dir);
        let mul = (mul as usize).saturating_sub(1);

        if dir == Direction::Forward {
            finds.idx = finds.list.iter().position(|(v, _)| v.start > c);

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
                    let r = finds.list[i].0.clone();
                    if c == r.end {
                        c = r.start;
                    }
                }
            }

            finds.idx = finds
                .list
                .iter()
                .enumerate()
                .filter_map(|(i, (v, _))| if v.end < c { Some(i) } else { None })
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
            finds.sync = SyncRanges::ToTextArea;

            let cursor = state.cursor();
            let start = TextPosition::new(0, cursor.y);
            let end = TextPosition::new(state.line_width(cursor.y), cursor.y);
            let mut it = state.graphemes(TextRange::new(start, end), start);
            loop {
                let Some(c) = it.next() else {
                    break;
                };
                if c == term {
                    finds.list.push((c.text_bytes(), 998));
                }
            }
        } else {
            finds.row = state.cursor().y;
            finds.dir = dir;
            finds.till = till;
            finds.idx = None;
        }
    }

    pub fn q_search_word_fwd(
        mul: u32,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Result<Option<TextPosition>, SearchError> {
        let start = qq_word_start(state);
        let end = qq_word_end(state);
        let term = state.str_slice(TextRange::from(start..end)).to_string();

        q_search(&mut vi.matches, &term, Direction::Forward, false, state)?;
        q_search_idx(&mut vi.matches, mul, Direction::Forward, state);
        Ok(q_current_search_idx(state, vi))
    }

    pub fn q_search_word_back(
        mul: u32,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Result<Option<TextPosition>, SearchError> {
        let start = qq_word_start(state);
        let end = qq_word_end(state);
        let term = state.str_slice(TextRange::from(start..end)).to_string();

        q_search(&mut vi.matches, &term, Direction::Backward, false, state)?;
        q_search_idx(&mut vi.matches, mul, Direction::Forward, state);
        Ok(q_current_search_idx(state, vi))
    }

    fn qq_word_start(state: &TextAreaState) -> TextPosition {
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

    fn qq_word_end(state: &TextAreaState) -> TextPosition {
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

    pub fn q_search_back(
        mul: u32,
        term: &str,
        tmp: bool,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Result<Option<TextPosition>, SearchError> {
        q_search(&mut vi.matches, term, Direction::Backward, tmp, state)?;
        q_search_idx(&mut vi.matches, mul, Direction::Forward, state);
        Ok(q_current_search_idx(state, vi))
    }

    pub fn q_search_fwd(
        mul: u32,
        term: &str,
        tmp: bool,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Result<Option<TextPosition>, SearchError> {
        q_search(&mut vi.matches, term, Direction::Forward, tmp, state)?;
        q_search_idx(&mut vi.matches, mul, Direction::Forward, state);
        Ok(q_current_search_idx(state, vi))
    }

    pub fn q_search_repeat_back(
        mul: u32,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Option<TextPosition> {
        if vi.matches.term.is_none() {
            return None;
        }
        q_search_idx(&mut vi.matches, mul, Direction::Backward, state);
        q_current_search_idx(state, vi)
    }

    pub fn q_search_repeat_fwd(
        mul: u32,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Option<TextPosition> {
        if vi.matches.term.is_none() {
            return None;
        }
        q_search_idx(&mut vi.matches, mul, Direction::Forward, state);
        q_current_search_idx(state, vi)
    }

    pub fn q_search(
        matches: &mut Matches,
        term: &str,
        dir: Direction,
        tmp: bool,
        state: &mut TextAreaState,
    ) -> Result<(), SearchError> {
        if matches
            .term
            .as_ref()
            .map(|v| v.as_str() != term)
            .unwrap_or(true)
        {
            matches.term = Some(term.to_string());
            matches.dir = dir;
            matches.tmp = tmp;
            matches.idx = None;
            matches.list.clear();
            matches.sync = SyncRanges::ToTextArea;

            if let Ok(re) = Regex::new(matches.term.as_ref().expect("term")) {
                let cursor = RopeyCursor::new(state.rope().byte_slice(..));
                let input = Input::new(cursor);

                for m in find_iter(&re, input) {
                    matches.list.push((m.start()..m.end(), 999));
                }
            }
        } else {
            matches.dir = dir;
            matches.tmp = tmp;
            matches.idx = None;
        }
        Ok(())
    }

    pub fn q_search_idx(
        matches: &mut Matches,
        mul: u32,
        dir: Direction,
        state: &mut TextAreaState,
    ) {
        let c = state.byte_at(state.cursor()).start;
        let dir = matches.dir.mul(dir);
        let mul = (mul as usize).saturating_sub(1);

        if dir == Direction::Forward {
            matches.idx = matches.list.iter().position(|(v, _)| v.start > c);

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
                .filter_map(|(i, (v, _))| if v.end < c { Some(i) } else { None })
                .last();

            matches.idx = if let Some(idx) = matches.idx {
                if idx >= mul { Some(idx - mul) } else { None }
            } else {
                None
            }
        }
    }

    pub fn q_current_search_idx(state: &mut TextAreaState, vi: &mut VI) -> Option<TextPosition> {
        if let Some(idx) = vi.matches.idx {
            let pos = state.byte_pos(vi.matches.list[idx].0.start);
            Some(pos)
        } else {
            None
        }
    }

    pub fn q_line_break_and_leading_space(state: &mut TextAreaState) -> TextRange {
        let c = state.cursor();
        let width = state.line_width(c.y);

        let start = TextPosition::new(width, c.y);

        let mut it = state.text_graphemes(start);
        skip_white(&mut it);
        let end = state.byte_pos(it.text_offset());

        TextRange::new(start, end)
    }

    #[inline]
    fn pskip_alpha<'a, C: Cursor<Item = Grapheme<'a>>>(it: &mut C) {
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

    #[inline]
    fn pskip_sample<'a, C: Cursor<Item = Grapheme<'a>>>(it: &mut C, sample: Grapheme) {
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

    #[inline]
    fn pskip_white<'a, C: Cursor<Item = Grapheme<'a>>>(it: &mut C) {
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

    #[inline]
    fn pskip_nonwhite<'a, C: Cursor<Item = Grapheme<'a>>>(it: &mut C) {
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

    #[inline]
    fn skip_alpha<'a, C: Cursor<Item = Grapheme<'a>>>(it: &mut C) {
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

    #[inline]
    fn skip_sample<'a, C: Cursor<Item = Grapheme<'a>>>(it: &mut C, sample: Grapheme) {
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

    #[inline]
    fn skip_white<'a, C: Cursor<Item = Grapheme<'a>>>(it: &mut C) {
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

    #[inline]
    fn skip_nonwhite<'a, C: Cursor<Item = Grapheme<'a>>>(it: &mut C) {
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

    pub fn q_prepend_line_str(v: &str, state: &mut TextAreaState) {
        let c = state.cursor();
        state.begin_undo_seq();
        state.set_cursor(TextPosition::new(0, c.y), false);
        state.insert_newline();
        state.set_cursor(TextPosition::new(0, c.y), false);
        for c in v.chars() {
            q_insert(c, state);
        }
        state.end_undo_seq();
    }

    pub fn q_append_line_str(v: &str, state: &mut TextAreaState) {
        let c = state.cursor();
        let width = state.line_width(c.y);
        state.begin_undo_seq();
        state.set_cursor(TextPosition::new(width, c.y), false);
        state.insert_newline();
        for c in v.chars() {
            q_insert(c, state);
        }
        state.end_undo_seq();
    }

    pub fn q_append_str(v: &str, state: &mut TextAreaState) {
        state.begin_undo_seq();
        state.move_right(1, false);
        for c in v.chars() {
            q_insert(c, state);
        }
        state.end_undo_seq();
    }

    pub fn q_insert_str(v: &str, state: &mut TextAreaState) {
        state.begin_undo_seq();
        for c in v.chars() {
            q_insert(c, state);
        }
        state.end_undo_seq();
    }

    pub fn q_insert(cc: char, state: &mut TextAreaState) {
        _ = match cc {
            '\n' => state.insert_newline(),
            '\t' => state.insert_tab(),
            ctrl::BS => state.delete_prev_char(),
            ctrl::DEL => state.delete_next_char(),
            _ => state.insert_char(cc),
        };
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
        if self.focus.gained() && matches!(event, ct_event!(keycode press Tab)) {
            return Ok(TextOutcome::Unchanged);
        }

        let r = if vi.mode == Mode::Normal {
            match event {
                ct_event!(keycode press Esc) | ct_event!(key press CONTROL-'c') => {
                    reset_motion(vi);
                    self.scroll_cursor_to_visible();
                    TextOutcome::Changed
                }

                ct_event!(key press c)
                | ct_event!(key press SHIFT-c)
                | ct_event!(key press CONTROL_ALT-c) => eval_normal(*c, self, vi)?,
                ct_event!(keycode press Enter) => eval_normal('\n', self, vi)?,
                ct_event!(keycode press Backspace) => eval_normal(ctrl::BS, self, vi)?,
                ct_event!(key press CONTROL-cc) => eval_normal(ctrl::ctrl(*cc), self, vi)?,

                _ => TextOutcome::Continue,
            }
        } else if vi.mode == Mode::Insert {
            match event {
                ct_event!(keycode press Esc) | ct_event!(key press CONTROL-'c') => {
                    end_insert_mode(self, vi)
                }

                ct_event!(key press c)
                | ct_event!(key press SHIFT-c)
                | ct_event!(key press CONTROL_ALT-c) => eval_insert(*c, self, vi),
                ct_event!(keycode press Tab) => eval_insert('\t', self, vi),
                ct_event!(keycode press Enter) => eval_insert('\n', self, vi),
                ct_event!(keycode press Backspace) => eval_insert(ctrl::BS, self, vi),
                ct_event!(keycode press Delete) => eval_insert(ctrl::DEL, self, vi),

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
