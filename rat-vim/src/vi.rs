use crate::SearchError;
use crate::coroutine::{Coroutine, Resume};
use crate::ctrl;
use crate::vi::change_op::*;
use crate::vi::display::*;
use crate::vi::mark_op::*;
use crate::vi::modes_op::*;
use crate::vi::move_op::*;
use crate::vi::partial_op::*;
use crate::vi::scroll_op::*;
use crate::vi::state_machine::*;
use crate::vi::visual_op::*;
use log::debug;
use rat_event::{HandleEvent, ct_event};
use rat_text::event::TextOutcome;
use rat_text::text_area::TextAreaState;
use rat_text::{TextPosition, upos_type};
use std::cell::RefCell;
use std::mem;
use std::ops::Range;
use std::rc::Rc;
use std::time::SystemTime;

pub mod query;

pub struct VI {
    pub mode: Mode,

    pub co_normal: Coroutine<'static, char, Vim, Vim>,
    pub co_visual: Coroutine<'static, char, Vim, Vim>,

    pub command_display: Rc<RefCell<String>>,

    pub command: Vim,
    pub text: String,

    pub finds: Finds,
    pub matches: Matches,
    pub visual: Visual,
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
pub struct Visual {
    pub block: bool,
    pub anchor: TextPosition,
    pub lead: TextPosition,
    pub list: Vec<(Range<usize>, usize)>,
    pub sync: SyncRanges,
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
    Left,
    Right,
    Up,
    Down,

    ToCol,
    ToLine,
    ToLinePercent,
    ToMatchingBrace,
    ToMark(char),

    StartOfFile,
    EndOfFile,

    NextWordStart,
    PrevWordStart,
    NextWordEnd,
    PrevWordEnd,
    NextWORDStart,
    PrevWORDStart,
    NextWORDEnd,
    PrevWORDEnd,
    StartOfLine,
    EndOfLine,
    StartOfLineText,
    EndOfLineText,
    PrevParagraph,
    NextParagraph,

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Visuals {
    Select,
    SelectBlock,

    SwapLead,
    SwapDiagonal,
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
    Visual(Visuals),

    Undo(u32),
    Redo(u32),

    JoinLines(u32),
    Insert(u32),
    Append(u32),
    AppendLine(u32),
    PrependLine(u32),
    Delete(u32, Motion),
    Change(u32, Motion),
    Replace(u32, char),
}

fn is_memo(vim: &Vim) -> bool {
    match vim {
        Vim::Invalid => false,
        Vim::Repeat(_) => false,
        Vim::Partial(_, _) => false,
        Vim::Move(_, _) => false,
        Vim::Scroll(_, _) => false,
        Vim::Mark(_) => false,
        Vim::Visual(_) => false,
        Vim::Undo(_) => false,
        Vim::Redo(_) => false,
        Vim::JoinLines(_) => true,
        Vim::Insert(_) => true,
        Vim::Append(_) => true,
        Vim::AppendLine(_) => true,
        Vim::PrependLine(_) => true,
        Vim::Delete(_, _) => true,
        Vim::Change(_, _) => true,
        Vim::Replace(_, _) => true,
    }
}

impl Default for VI {
    fn default() -> Self {
        let motion_buf = Rc::new(RefCell::new(String::new()));
        Self {
            mode: Default::default(),
            co_normal: Coroutine::new({
                let motion_buf = motion_buf.clone();
                |c, yp| Box::new(next_normal(c, motion_buf, yp))
            }),
            co_visual: Coroutine::new({
                let motion_buf = motion_buf.clone();
                |c, yp| Box::new(next_visual(c, motion_buf, yp))
            }),
            command_display: motion_buf,
            command: Default::default(),
            text: Default::default(),
            finds: Default::default(),
            matches: Default::default(),
            visual: Default::default(),
            marks: Default::default(),
            page: Default::default(),
        }
    }
}

fn eval_insert(cc: char, state: &mut TextAreaState, vi: &mut VI) -> TextOutcome {
    insert_char(cc, state, vi);
    display_matches(state, vi);
    display_finds(state, vi);
    TextOutcome::TextChanged
}

fn eval_visual(
    cc: char,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Result<TextOutcome, SearchError> {
    let vim = match vi.co_visual.resume(cc) {
        Resume::Yield(v) => v,
        Resume::Return(v) => {
            reset_co_visual(vi);
            v
        }
        Resume::Pending => {
            debug!("VIM ... {:?}", vi.command_display.borrow());
            return Ok(TextOutcome::Changed);
        }
    };

    if let Vim::Repeat(_) = &vim {
        unreachable!("no repeat");
    } else {
        debug!("VIM |> {:?}", vim);
        let tt = SystemTime::now();
        let r = execute_visual(&vim, state, vi);
        debug!("TT {:?}", tt.elapsed());
        r
    }
}

fn eval_normal(
    cc: char,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Result<TextOutcome, SearchError> {
    let vim = match vi.co_normal.resume(cc) {
        Resume::Yield(v) => v,
        Resume::Return(v) => {
            reset_co_normal(vi);
            v
        }
        Resume::Pending => {
            debug!("VIM ... {:?}", vi.command_display.borrow());
            return Ok(TextOutcome::Changed);
        }
    };

    if let Vim::Repeat(mul) = &vim {
        debug!("VIM |> {:?} {:?} {:?}", vim, vi.command, vi.text);
        let tt = SystemTime::now();
        assert!(*mul > 0);
        let vim = mem::take(&mut vi.command);
        let mut mul = *mul;
        let r = loop {
            let rr = match execute_normal(&vim, true, state, vi) {
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
    } else {
        debug!("VIM |> {:?}", vim);
        let tt = SystemTime::now();
        let r = match execute_normal(&vim, false, state, vi) {
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
}

fn execute_visual(
    vim: &Vim,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Result<TextOutcome, SearchError> {
    let mut r = TextOutcome::Changed;

    match vim {
        Vim::Invalid => r = TextOutcome::Unchanged,
        Vim::Repeat(_) => unreachable!("wrong spot for repeat"),

        Vim::Move(mul, m) => {
            visual_cursor(*mul, m, state, vi)?;
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

        Vim::Scroll(_, _) => {}
        Vim::Mark(_) => {}

        Vim::Visual(Visuals::SwapDiagonal) => {
            visual_swap_diagonal(state, vi);
        }
        Vim::Visual(Visuals::SwapLead) => {
            visual_swap_lead(state, vi);
        }
        Vim::Visual(_) => unreachable!("unknown visual"),

        Vim::Undo(_) => {}
        Vim::Redo(_) => {}
        Vim::JoinLines(_) => {}
        Vim::Insert(_) => {}
        Vim::Append(_) => {}
        Vim::AppendLine(_) => {}
        Vim::PrependLine(_) => {}
        Vim::Delete(_, _) => {}
        Vim::Change(_, _) => {}
        Vim::Replace(_, _) => {}
    }

    display_matches(state, vi);
    display_finds(state, vi);
    display_visual(state, vi);

    Ok(r)
}

fn execute_normal(
    vim: &Vim,
    repeat: bool,
    state: &mut TextAreaState,
    vi: &mut VI,
) -> Result<TextOutcome, SearchError> {
    let mut r = TextOutcome::Changed;

    match vim {
        Vim::Invalid => r = TextOutcome::Unchanged,
        Vim::Repeat(_) => unreachable!("unkown repeat"),

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

        Vim::Visual(Visuals::Select) => begin_visual(false, state, vi),
        Vim::Visual(Visuals::SelectBlock) => begin_visual(true, state, vi),
        Vim::Visual(_) => unreachable!("unknown visual"),

        Vim::Undo(mul) => {
            undo(*mul, state, vi);
            r = TextOutcome::TextChanged;
        }
        Vim::Redo(mul) => {
            redo(*mul, state, vi);
            r = TextOutcome::TextChanged;
        }
        Vim::JoinLines(mul) => {
            join_line(*mul, state, vi);
            r = TextOutcome::TextChanged;
        }
        Vim::Insert(mul) => {
            if repeat {
                insert_str(*mul, state, vi);
                r = TextOutcome::TextChanged;
            } else {
                begin_insert(vi);
            }
        }
        Vim::Append(mul) => {
            if repeat {
                append_str(*mul, state, vi);
                r = TextOutcome::TextChanged;
            } else {
                begin_append(state, vi);
            }
        }
        Vim::AppendLine(mul) => {
            if repeat {
                append_line_str(*mul, state, vi);
                r = TextOutcome::TextChanged;
            } else {
                begin_append_line(state, vi);
                r = TextOutcome::TextChanged;
            }
        }
        Vim::PrependLine(mul) => {
            if repeat {
                prepend_line_str(*mul, state, vi);
                r = TextOutcome::TextChanged;
            } else {
                begin_prepend_line(state, vi);
                r = TextOutcome::TextChanged;
            }
        }
        Vim::Change(mul, m) => {
            if repeat {
                change_text(*mul, m, state, vi)?;
                insert_str(1, state, vi);
                r = TextOutcome::TextChanged;
            } else {
                change_text(*mul, m, state, vi)?;
                begin_insert(vi);
                r = TextOutcome::TextChanged;
            }
        }
        Vim::Delete(mul, m) => {
            delete_text(*mul, m, state, vi)?;
            r = TextOutcome::TextChanged;
        }
        Vim::Replace(mul, c) => {
            replace_text(*mul, *c, state, vi)?;
            r = TextOutcome::TextChanged;
        }
    }

    display_matches(state, vi);
    display_finds(state, vi);
    display_visual(state, vi);

    Ok(r)
}

pub mod display {
    use crate::vi::query::{q_find_idx, q_search_idx};
    use crate::vi::{Direction, SyncRanges, VI};
    use rat_text::text_area::TextAreaState;

    pub fn display_visual(state: &mut TextAreaState, vi: &mut VI) {
        match vi.visual.sync {
            SyncRanges::None => {}
            SyncRanges::ToTextArea => {
                vi.visual.sync = SyncRanges::None;
                state.remove_style_fully(997);
                for r in &vi.visual.list {
                    state.add_style(r.0.clone(), 997)
                }
            }
            SyncRanges::FromTextArea => {
                // noop
            }
        }
    }

    pub fn display_matches(state: &mut TextAreaState, vi: &mut VI) {
        match vi.matches.sync {
            SyncRanges::None => {}
            SyncRanges::ToTextArea => {
                vi.matches.sync = SyncRanges::None;
                state.remove_style_fully(999);
                for r in &vi.matches.list {
                    state.add_style(r.0.clone(), 999)
                }
            }
            SyncRanges::FromTextArea => {
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
                vi.finds.sync = SyncRanges::None;
                state.remove_style_fully(998);
                for r in &vi.finds.list {
                    state.add_style(r.0.clone(), 998)
                }
            }
            SyncRanges::FromTextArea => {
                vi.finds.sync = SyncRanges::None;
                vi.finds.list.clear();
                state.styles_in_match(0..state.len_bytes(), 998, &mut vi.finds.list);
                q_find_idx(&mut vi.finds, 1, Direction::Forward, state);
            }
        }
    }
}

pub mod scroll_op {
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

pub mod move_op {
    use crate::SearchError;
    use crate::vi::query::*;
    use crate::vi::{Motion, VI};
    use rat_text::TextPosition;
    use rat_text::text_area::TextAreaState;

    fn move_position(
        mul: u32,
        motion: &Motion,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Result<Option<TextPosition>, SearchError> {
        Ok(match motion {
            Motion::Left => q_move_left(mul, state),
            Motion::Right => q_move_right(mul, state),
            Motion::Up => q_move_up(mul, state),
            Motion::Down => q_move_down(mul, state),
            Motion::ToCol => q_col(mul, state),
            Motion::ToLine => q_line(mul, state),
            Motion::ToLinePercent => q_line_percent(mul, state),
            Motion::ToMatchingBrace => q_matching_brace(state),
            Motion::ToMark(mark) => q_mark_pos(*mark, &vi.marks),
            Motion::StartOfFile => q_start_of_file(),
            Motion::EndOfFile => q_end_of_file(state),
            Motion::NextWordStart => q_next_word_start(mul, state),
            Motion::PrevWordStart => q_prev_word_start(mul, state),
            Motion::NextWordEnd => q_next_word_end(mul, state),
            Motion::PrevWordEnd => q_prev_word_end(mul, state),
            Motion::NextWORDStart => q_next_bigword_start(mul, state),
            Motion::PrevWORDStart => q_prev_bigword_start(mul, state),
            Motion::NextWORDEnd => q_next_bigword_end(mul, state),
            Motion::PrevWORDEnd => q_prev_bigword_end(mul, state),
            Motion::StartOfLine => q_start_of_line(state),
            Motion::EndOfLine => q_end_of_line(mul, state),
            Motion::StartOfLineText => q_start_of_text(state),
            Motion::EndOfLineText => q_end_of_text(mul, state),
            Motion::PrevParagraph => q_prev_paragraph(mul, state),
            Motion::NextParagraph => q_next_paragraph(mul, state),
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

pub mod visual_op {
    use crate::SearchError;
    use crate::vi::query::*;
    use crate::vi::{Motion, VI};
    use rat_text::TextPosition;
    use rat_text::text_area::TextAreaState;

    fn visual_position(
        mul: u32,
        motion: &Motion,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Result<Option<TextPosition>, SearchError> {
        Ok(match motion {
            Motion::Left => q_move_left(mul, state),
            Motion::Right => q_move_right(mul, state),
            Motion::Up => q_move_up(mul, state),
            Motion::Down => q_move_down(mul, state),
            Motion::ToCol => q_col(mul, state),
            Motion::ToLine => q_line(mul, state),
            Motion::ToLinePercent => q_line_percent(mul, state),
            Motion::ToMatchingBrace => q_matching_brace(state),
            Motion::ToMark(mark) => q_mark_pos(*mark, &vi.marks),
            Motion::StartOfFile => q_start_of_file(),
            Motion::EndOfFile => q_end_of_file(state),
            Motion::NextWordStart => q_next_word_start(mul, state),
            Motion::PrevWordStart => q_prev_word_start(mul, state),
            Motion::NextWordEnd => q_next_word_end(mul, state),
            Motion::PrevWordEnd => q_prev_word_end(mul, state),
            Motion::NextWORDStart => q_next_bigword_start(mul, state),
            Motion::PrevWORDStart => q_prev_bigword_start(mul, state),
            Motion::NextWORDEnd => q_next_bigword_end(mul, state),
            Motion::PrevWORDEnd => q_prev_bigword_end(mul, state),
            Motion::StartOfLine => q_start_of_line(state),
            Motion::EndOfLine => q_end_of_line(mul, state),
            Motion::StartOfLineText => q_start_of_text(state),
            Motion::EndOfLineText => q_end_of_text(mul, state),
            Motion::PrevParagraph => q_prev_paragraph(mul, state),
            Motion::NextParagraph => q_next_paragraph(mul, state),
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

    pub fn visual_cursor(
        mul: u32,
        motion: &Motion,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Result<(), SearchError> {
        if let Some(npos) = visual_position(mul, motion, state, vi)? {
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
}

pub mod partial_op {
    use crate::SearchError;
    use crate::vi::query::*;
    use crate::vi::{Direction, VI};
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

pub mod mark_op {
    use crate::VI;
    use crate::vi::query::*;
    use rat_text::text_area::TextAreaState;

    pub fn set_mark(mark: char, state: &mut TextAreaState, vi: &mut VI) {
        if let Some(mark) = q_mark_idx(mark) {
            vi.marks[mark] = Some(state.cursor());
        }
    }
}

pub mod change_op {
    use crate::vi::query::*;
    use crate::vi::{Motion, SyncRanges};
    use crate::{SearchError, VI};
    use rat_text::TextPosition;
    use rat_text::text_area::TextAreaState;
    use std::ops::Range;

    pub fn prepend_line_str(mut mul: u32, state: &mut TextAreaState, vi: &mut VI) {
        if mul > 0 {
            vi.finds.sync = SyncRanges::FromTextArea;
            vi.matches.sync = SyncRanges::FromTextArea;
        }
        while mul > 0 {
            q_prepend_line_str(&vi.text, state);
            mul -= 1;
        }
    }

    pub fn append_line_str(mut mul: u32, state: &mut TextAreaState, vi: &mut VI) {
        if mul > 0 {
            vi.finds.sync = SyncRanges::FromTextArea;
            vi.matches.sync = SyncRanges::FromTextArea;
        }
        while mul > 0 {
            q_append_line_str(&vi.text, state);
            mul -= 1;
        }
    }

    pub fn append_str(mut mul: u32, state: &mut TextAreaState, vi: &mut VI) {
        if mul > 0 {
            vi.finds.sync = SyncRanges::FromTextArea;
            vi.matches.sync = SyncRanges::FromTextArea;
        }
        while mul > 0 {
            q_append_str(&vi.text, state);
            mul -= 1;
        }
    }

    pub fn insert_str(mut mul: u32, state: &mut TextAreaState, vi: &mut VI) {
        if mul > 0 {
            vi.finds.sync = SyncRanges::FromTextArea;
            vi.matches.sync = SyncRanges::FromTextArea;
        }
        while mul > 0 {
            q_insert_str(&vi.text, state);
            mul -= 1;
        }
    }

    pub fn insert_char(cc: char, state: &mut TextAreaState, vi: &mut VI) {
        vi.finds.sync = SyncRanges::FromTextArea;
        vi.matches.sync = SyncRanges::FromTextArea;
        vi.text.push(cc);
        q_insert(cc, state);
    }

    pub fn replace_text(
        mut mul: u32,
        cc: char,
        state: &mut TextAreaState,
        vi: &mut VI,
    ) -> Result<(), SearchError> {
        if let Some(range) = change_range(mul, &Motion::Right, state, vi)? {
            vi.finds.sync = SyncRanges::FromTextArea;
            vi.matches.sync = SyncRanges::FromTextArea;
            state.begin_undo_seq();
            state.delete_range(range);
            while mul > 0 {
                state.insert_char(cc);
                mul -= 1;
            }
            state.end_undo_seq();
        }
        Ok(())
    }

    fn change_range(
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
            Motion::Left => q_move_left(mul, state),
            Motion::Right => q_move_right(mul, state),
            Motion::Up => q_move_up(mul, state),
            Motion::Down => q_move_down(mul, state),
            Motion::ToCol => q_col(mul, state),
            Motion::ToLine => q_line(mul, state),
            Motion::ToLinePercent => q_line_percent(mul, state),
            Motion::ToMatchingBrace => q_matching_brace(state),
            Motion::ToMark(mark) => q_mark_pos(*mark, &vi.marks),
            Motion::StartOfFile => q_start_of_file(),
            Motion::EndOfFile => q_end_of_file(state),
            Motion::NextWordStart => q_next_word_end(mul, state), // DIFF
            Motion::PrevWordStart => q_prev_word_start(mul, state),
            Motion::NextWordEnd => q_next_word_end(mul, state),
            Motion::PrevWordEnd => q_prev_word_end(mul, state),
            Motion::NextWORDStart => q_next_bigword_start(mul, state),
            Motion::PrevWORDStart => q_prev_bigword_start(mul, state),
            Motion::NextWORDEnd => q_next_bigword_end(mul, state),
            Motion::PrevWORDEnd => q_prev_bigword_end(mul, state),
            Motion::StartOfLine => q_start_of_line(state),
            Motion::EndOfLine => q_end_of_line(mul, state),
            Motion::StartOfLineText => q_start_of_text(state),
            Motion::EndOfLineText => q_end_of_text(mul, state),
            Motion::PrevParagraph => q_prev_paragraph(mul, state),
            Motion::NextParagraph => q_next_paragraph(mul, state),
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
            Motion::FullLine => q_end_of_line(mul, state), // DIFF
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

    pub fn change_text(
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

    fn delete_range(
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
            Motion::Left => q_move_left(mul, state),
            Motion::Right => q_move_right(mul, state),
            Motion::Up => q_move_up(mul, state),
            Motion::Down => q_move_down(mul, state),
            Motion::ToCol => q_col(mul, state),
            Motion::ToLine => q_line(mul, state),
            Motion::ToLinePercent => q_line_percent(mul, state),
            Motion::ToMatchingBrace => q_matching_brace(state),
            Motion::ToMark(mark) => q_mark_pos(*mark, &vi.marks),
            Motion::StartOfFile => q_start_of_file(),
            Motion::EndOfFile => q_end_of_file(state),
            Motion::NextWordStart => q_next_word_end(mul, state), // DIFF
            Motion::PrevWordStart => q_prev_word_start(mul, state),
            Motion::NextWordEnd => q_next_word_end(mul, state),
            Motion::PrevWordEnd => q_prev_word_end(mul, state),
            Motion::NextWORDStart => q_next_bigword_start(mul, state),
            Motion::PrevWORDStart => q_prev_bigword_start(mul, state),
            Motion::NextWORDEnd => q_next_bigword_end(mul, state),
            Motion::PrevWORDEnd => q_prev_bigword_end(mul, state),
            Motion::StartOfLine => q_start_of_line(state),
            Motion::EndOfLine => q_end_of_line(mul, state),
            Motion::StartOfLineText => q_start_of_text(state),
            Motion::EndOfLineText => q_end_of_text(mul, state),
            Motion::PrevParagraph => q_prev_paragraph(mul, state),
            Motion::NextParagraph => q_next_paragraph(mul, state),
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
        if let Some(range) = delete_range(mul, motion, state, vi)? {
            vi.finds.sync = SyncRanges::FromTextArea;
            vi.matches.sync = SyncRanges::FromTextArea;
            state.delete_range(range);
        }
        Ok(())
    }

    pub fn join_line(mut mul: u32, state: &mut TextAreaState, vi: &mut VI) {
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
    }

    pub fn undo(mut mul: u32, state: &mut TextAreaState, vi: &mut VI) {
        vi.finds.sync = SyncRanges::FromTextArea;
        vi.matches.sync = SyncRanges::FromTextArea;

        while mul > 0 {
            if !state.undo() {
                break;
            }

            mul -= 1;
        }
    }

    pub fn redo(mut mul: u32, state: &mut TextAreaState, vi: &mut VI) {
        vi.finds.sync = SyncRanges::FromTextArea;
        vi.matches.sync = SyncRanges::FromTextArea;

        while mul > 0 {
            if !state.redo() {
                break;
            }

            mul -= 1;
        }
    }
}

pub mod modes_op {
    use crate::coroutine::Coroutine;
    use crate::vi::change_op::*;
    use crate::vi::state_machine::*;
    use crate::vi::{SyncRanges, Vim};
    use crate::{Mode, VI};
    use log::debug;
    use rat_text::TextPosition;
    use rat_text::text_area::TextAreaState;
    use std::mem;

    pub fn reset_co_normal(vi: &mut VI) {
        vi.command_display.borrow_mut().clear();
        let mb = vi.command_display.clone();
        vi.co_normal = Coroutine::new(|c, yp| Box::new(next_normal(c, mb, yp)));
    }

    pub fn reset_co_visual(vi: &mut VI) {
        vi.command_display.borrow_mut().clear();
        let mb = vi.command_display.clone();
        vi.co_visual = Coroutine::new(|c, yp| Box::new(next_visual(c, mb, yp)));
    }

    pub fn reset_normal_mode(vi: &mut VI) {
        vi.mode = Mode::Normal;

        vi.matches.sync = SyncRanges::ToTextArea;
        vi.matches.term = None;
        vi.matches.idx = None;
        vi.matches.list.clear();
        vi.matches.tmp = Default::default();
        vi.matches.dir = Default::default();

        vi.finds.sync = SyncRanges::ToTextArea;
        vi.finds.term = None;
        vi.finds.idx = None;
        vi.finds.list.clear();
        vi.finds.row = Default::default();
        vi.finds.till = Default::default();
        vi.finds.dir = Default::default();
    }

    pub fn reset_visual_mode(vi: &mut VI) {
        vi.mode = Mode::Normal;

        vi.visual.sync = SyncRanges::ToTextArea;
        vi.visual.block = Default::default();
        vi.visual.anchor = Default::default();
        vi.visual.lead = Default::default();
        vi.visual.list.clear();
    }

    pub fn reset_insert_mode(vi: &mut VI) {
        vi.mode = Mode::Normal;
    }

    pub fn begin_visual(block: bool, state: &mut TextAreaState, vi: &mut VI) {
        vi.mode = Mode::Visual;
        vi.visual.block = block;
        vi.visual.anchor = state.cursor();
        vi.visual.lead = state.cursor();
        vi.visual.sync = SyncRanges::ToTextArea;
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

    pub fn begin_append(state: &mut TextAreaState, vi: &mut VI) {
        vi.mode = Mode::Insert;
        vi.text.clear();

        state.move_right(1, false);
    }

    pub fn begin_insert(vi: &mut VI) {
        vi.mode = Mode::Insert;
        vi.text.clear();
    }

    pub fn end_insert(state: &mut TextAreaState, vi: &mut VI) {
        let command = mem::take(&mut vi.command);
        match &command {
            Vim::Insert(mul) => {
                insert_str(mul.saturating_sub(1), state, vi);
            }
            Vim::Append(mul) => {
                append_str(mul.saturating_sub(1), state, vi);
            }
            Vim::AppendLine(mul) => {
                append_line_str(mul.saturating_sub(1), state, vi);
            }
            Vim::PrependLine(mul) => {
                prepend_line_str(mul.saturating_sub(1), state, vi);
            }
            _ => {}
        };
        vi.command = command;
    }
}

pub mod state_machine {
    use crate::coroutine::Yield;
    use crate::vi::{Motion, Scrolling, Vim, Visuals};
    use crate::{ctrl, yield_};
    use std::cell::RefCell;
    use std::rc::Rc;

    async fn bare_multiplier(
        mut tok: char,
        motion_buf: &RefCell<String>,
        yp: &Yield<char, Vim>,
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
        yp: &Yield<char, Vim>,
    ) -> Result<Vim, char> {
        match tok {
            'h' => Ok(Vim::Move(mul.unwrap_or(1), Motion::Left)),
            'l' => Ok(Vim::Move(mul.unwrap_or(1), Motion::Right)),
            'k' => Ok(Vim::Move(mul.unwrap_or(1), Motion::Up)),
            'j' => Ok(Vim::Move(mul.unwrap_or(1), Motion::Down)),
            '|' => Ok(Vim::Move(mul.unwrap_or(0), Motion::ToCol)),
            'w' => Ok(Vim::Move(mul.unwrap_or(1), Motion::NextWordStart)),
            'b' => Ok(Vim::Move(mul.unwrap_or(1), Motion::PrevWordStart)),
            'e' => Ok(Vim::Move(mul.unwrap_or(1), Motion::NextWordEnd)),
            'g' => {
                let tok = yield_!(yp);
                motion_buf.borrow_mut().push(tok);
                match tok {
                    'e' => Ok(Vim::Move(mul.unwrap_or(1), Motion::PrevWordEnd)),
                    'E' => Ok(Vim::Move(mul.unwrap_or(1), Motion::PrevWORDEnd)),
                    '_' => Ok(Vim::Move(mul.unwrap_or(1), Motion::EndOfLineText)),
                    'g' => {
                        if let Some(mul) = mul {
                            Ok(Vim::Move(mul, Motion::ToLine))
                        } else {
                            Ok(Vim::Move(1, Motion::StartOfFile))
                        }
                    }
                    _ => Ok(Vim::Invalid),
                }
            }
            'W' => Ok(Vim::Move(mul.unwrap_or(1), Motion::NextWORDStart)),
            'B' => Ok(Vim::Move(mul.unwrap_or(1), Motion::PrevWORDStart)),
            'E' => Ok(Vim::Move(mul.unwrap_or(1), Motion::NextWORDEnd)),
            '{' => Ok(Vim::Move(mul.unwrap_or(1), Motion::PrevParagraph)),
            '}' => Ok(Vim::Move(mul.unwrap_or(1), Motion::NextParagraph)),
            'G' => {
                if let Some(mul) = mul {
                    Ok(Vim::Move(mul, Motion::ToLine))
                } else {
                    Ok(Vim::Move(1, Motion::EndOfFile))
                }
            }
            '^' => Ok(Vim::Move(1, Motion::StartOfLineText)),
            '$' => Ok(Vim::Move(mul.unwrap_or(1), Motion::EndOfLine)),
            '%' => {
                if let Some(mul) = mul {
                    Ok(Vim::Move(mul, Motion::ToLinePercent))
                } else {
                    Ok(Vim::Move(1, Motion::ToMatchingBrace))
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
                Ok(Vim::Move(1, Motion::ToMark(tok)))
            }

            _ => Err(tok),
        }
    }

    async fn bare_scroll(
        tok: char,
        mul: Option<u32>,
        motion_buf: &RefCell<String>,
        yp: &Yield<char, Vim>,
    ) -> Result<Vim, char> {
        match tok {
            ctrl::CTRL_U => Ok(Vim::Scroll(mul.unwrap_or(0), Scrolling::HalfPageUp)),
            ctrl::CTRL_D => Ok(Vim::Scroll(mul.unwrap_or(0), Scrolling::HalfPageDown)),
            'z' => {
                let tok = yield_!(yp);
                motion_buf.borrow_mut().push(tok);
                match tok {
                    'z' => Ok(Vim::Scroll(1, Scrolling::MiddleOfScreen)),
                    't' => Ok(Vim::Scroll(1, Scrolling::TopOfScreen)),
                    'b' => Ok(Vim::Scroll(1, Scrolling::BottomOfScreen)),
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

    pub async fn next_normal(
        mut tok: char,
        motion_buf: Rc<RefCell<String>>,
        yp: Yield<char, Vim>,
    ) -> Vim {
        if tok == '0' {
            return Vim::Move(1, Motion::StartOfLine);
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
            'v' => Vim::Visual(Visuals::Select),
            ctrl::CTRL_V => Vim::Visual(Visuals::SelectBlock),

            'r' => {
                tok = yield_!(yp);
                motion_buf.borrow_mut().push(tok);
                Vim::Replace(mul.unwrap_or(1), tok)
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
            'D' => Vim::Delete(mul.unwrap_or(1), Motion::EndOfLine),
            'c' => {
                tok = yield_!(yp);

                let mul2;
                (mul2, tok) = bare_multiplier(tok, &motion_buf, &yp).await;

                motion_buf.borrow_mut().push(tok);
                tok = match bare_motion(tok, mul2, &motion_buf, &yp).await {
                    Ok(Vim::Move(mul2, motion)) => {
                        let mul = mul.unwrap_or(1);
                        return Vim::Change(mul * mul2, motion);
                    }
                    Ok(Vim::Invalid) => return Vim::Invalid,
                    Err(tok) => tok,
                    _ => unreachable!("no"),
                };
                match tok {
                    'c' => Vim::Change(mul.unwrap_or(1), Motion::FullLine),
                    _ => Vim::Invalid,
                }
            }
            'C' => Vim::Change(mul.unwrap_or(1), Motion::EndOfLine),
            's' => Vim::Change(mul.unwrap_or(1), Motion::Right),
            'S' => Vim::Change(mul.unwrap_or(1), Motion::EndOfLine),
            'i' => Vim::Insert(mul.unwrap_or(1)),
            'a' => Vim::Append(mul.unwrap_or(1)),
            'o' => Vim::AppendLine(mul.unwrap_or(1)),
            'O' => Vim::PrependLine(mul.unwrap_or(1)),
            'x' => Vim::Delete(mul.unwrap_or(1), Motion::Right),
            'X' => Vim::Delete(mul.unwrap_or(1), Motion::Left),
            'J' => Vim::JoinLines(mul.unwrap_or(1)),
            'u' => Vim::Undo(mul.unwrap_or(1)),
            ctrl::CTRL_R => Vim::Redo(mul.unwrap_or(1)),

            '.' => Vim::Repeat(mul.unwrap_or(1)),

            _ => Vim::Invalid,
        }
    }

    pub async fn next_visual(
        mut tok: char,
        motion_buf: Rc<RefCell<String>>,
        yp: Yield<char, Vim>,
    ) -> Vim {
        if tok == '0' {
            return Vim::Move(1, Motion::StartOfLine);
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

        match tok {
            'o' => Vim::Visual(Visuals::SwapLead),
            'O' => Vim::Visual(Visuals::SwapDiagonal),

            _ => Vim::Invalid,
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
        if self.focus.gained() && matches!(event, ct_event!(keycode press Tab)) {
            return Ok(TextOutcome::Unchanged);
        }

        let r = if vi.mode == Mode::Normal {
            match event {
                ct_event!(keycode press Esc) | ct_event!(key press CONTROL-'c') => {
                    reset_normal_mode(vi);
                    reset_co_normal(vi);
                    display_matches(self, vi);
                    display_finds(self, vi);
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
                    end_insert(self, vi);
                    reset_insert_mode(vi);
                    display_matches(self, vi);
                    display_finds(self, vi);
                    self.scroll_cursor_to_visible();
                    TextOutcome::TextChanged
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
            match event {
                ct_event!(keycode press Esc) | ct_event!(key press CONTROL-'c') => {
                    reset_visual_mode(vi);
                    reset_co_visual(vi);
                    display_matches(self, vi);
                    display_finds(self, vi);
                    display_visual(self, vi);
                    self.scroll_cursor_to_visible();
                    TextOutcome::Changed
                }

                ct_event!(key press c)
                | ct_event!(key press SHIFT-c)
                | ct_event!(key press CONTROL_ALT-c) => eval_visual(*c, self, vi)?,
                ct_event!(keycode press Enter) => eval_visual('\n', self, vi)?,
                ct_event!(keycode press Backspace) => eval_visual(ctrl::BS, self, vi)?,
                ct_event!(key press CONTROL-cc) => eval_visual(ctrl::ctrl(*cc), self, vi)?,

                _ => TextOutcome::Continue,
            }
        } else {
            TextOutcome::Continue
        };

        Ok(r)
    }
}
