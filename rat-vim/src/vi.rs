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

pub mod change_op;
pub mod display;
pub mod mark_op;
pub mod modes_op;
pub mod motion_op;
pub mod move_op;
pub mod partial_op;
pub mod query;
pub mod scroll_op;
pub mod state_machine;
pub mod visual_op;

/// State for VI editing.
pub struct VI {
    /// vi mode
    pub mode: Mode,

    /// normal commands
    pub co_normal: Coroutine<'static, char, Vim, Vim>,
    /// visual commands
    pub co_visual: Coroutine<'static, char, Vim, Vim>,

    /// summary text for the command
    pub command_display: Rc<RefCell<String>>,

    /// Last repeatable command.
    pub command: Vim,
    /// Text for the last command.
    pub text: String,

    /// f and t matches
    pub finds: Finds,
    /// / matches
    pub matches: Matches,
    /// visual selection
    pub visual: Visual,
    /// yank data
    pub yank: Yank,
    /// text marks
    pub marks: Marks,
    /// pagelen for ctrl-d/u
    pub page: (u32, u32),
}

/// VI mode.
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    Visual,
}

/// Search direction.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    #[default]
    Forward,
    Backward,
}

impl Direction {
    /// Multiplies two directions.
    pub fn mul(self, d: Direction) -> Direction {
        match (self, d) {
            (Direction::Forward, Direction::Forward) => Direction::Forward,
            (Direction::Forward, Direction::Backward) => Direction::Backward,
            (Direction::Backward, Direction::Forward) => Direction::Backward,
            (Direction::Backward, Direction::Backward) => Direction::Forward,
        }
    }
}

/// Flag to sync any ranges with TextArea styles.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum SyncRanges {
    #[default]
    None,
    ToTextArea,
    FromTextArea,
}

/// Marks
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mark {
    Char(char),
    Insert,
    VisualAnchor,
    VisualLead,
    ChangeStart,
    ChangeEnd,
    Jump,
}

/// Mark_ data
#[derive(Debug, Default)]
pub struct Marks {
    /// a-z marks
    pub list: [Option<TextPosition>; 26],

    /// last insert position
    pub insert: Option<TextPosition>,
    /// last visual selection
    pub visual_anchor: Option<TextPosition>,
    /// last visual selection
    pub visual_lead: Option<TextPosition>,
    /// last change
    pub change: Vec<(TextPosition, TextPosition)>,
    pub change_idx: usize,
    /// last jump
    pub jump: Vec<TextPosition>,
    pub jump_idx: usize,

    /// sync
    pub sync: SyncRanges,
}

/// Yank data
#[derive(Debug, Default)]
pub struct Yank {
    /// Yanked text.
    /// Only contains more than one string if this was a visual block.
    pub list: Vec<String>,
}

/// Visual selection
#[derive(Debug, Default)]
pub struct Visual {
    /// regular or block selection
    pub block: bool,
    /// start position
    pub anchor: TextPosition,
    /// end position
    pub lead: TextPosition,
    /// list of affected ranges
    pub list: Vec<(Range<usize>, usize)>,
    /// sync
    pub sync: SyncRanges,
}

impl Visual {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.sync = SyncRanges::ToTextArea;
        self.block = Default::default();
        self.anchor = Default::default();
        self.lead = Default::default();
        self.list.clear();
    }
}

/// Results of f or t
#[derive(Debug, Default)]
pub struct Finds {
    /// Search term.
    pub term: Option<char>,
    /// Valid for this row.
    pub row: upos_type,
    /// Search direction.
    pub dir: Direction,
    /// t instead of f
    pub till: bool,
    /// Last selected idx.
    pub idx: Option<usize>,
    /// List of matching char-ranges
    pub list: Vec<(Range<usize>, usize)>,
    /// Sync
    pub sync: SyncRanges,
}

impl Finds {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.sync = SyncRanges::ToTextArea;
        self.term = Default::default();
        self.row = Default::default();
        self.dir = Default::default();
        self.till = Default::default();
        self.idx = Default::default();
        self.list.clear();
    }
}

/// Search matches
#[derive(Debug, Default)]
pub struct Matches {
    /// Search term
    pub term: Option<String>,
    /// Direction
    pub dir: Direction,
    /// Temporary search. Search occurs with each character input.
    /// Unless confirmed with Enter it is temporary.
    pub tmp: bool,
    /// Last selected idx.
    pub idx: Option<usize>,
    /// List of matching text ranges.
    pub list: Vec<(Range<usize>, usize)>,
    /// Sync
    pub sync: SyncRanges,
}

impl Matches {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.sync = SyncRanges::ToTextArea;
        self.term = Default::default();
        self.dir = Default::default();
        self.tmp = Default::default();
        self.idx = Default::default();
        self.list.clear();
    }
}

/// Text object modifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxtObj {
    A,
    I,
}

/// Vi motions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Motion {
    Visual,

    Left,
    Right,
    Up,
    Down,

    HalfPageUp,
    HalfPageDown,

    ToTopOfScreen,
    ToMiddleOfScreen,
    ToBottomOfScreen,

    ToCol,
    ToLine,
    ToLinePercent,
    ToMatchingBrace,
    ToMark(Mark, bool),

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
    PrevSentence,
    NextSentence,
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

    Word(TxtObj),
    WORD(TxtObj),
    Sentence(TxtObj),
    Paragraph(TxtObj),
    Bracket(TxtObj),
    Parenthesis(TxtObj),
    Angled(TxtObj),
    Tagged(TxtObj),
    Brace(TxtObj),
    Quoted(char, TxtObj),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum History {
    PrevJump,
    NextJump,
    PrevChange,
    NextChange,
}

/// Vi scrolling commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scrolling {
    Up,
    Down,
    PageUp,
    PageDown,
    MiddleOfScreen,
    TopOfScreen,
    BottomOfScreen,
}

/// Vi command.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Vim {
    #[default]
    Invalid,

    Repeat(u32),

    /// Partial movement. Only used for '/' search.
    Partial(u32, Motion),
    Move(u32, Motion),
    History(u32, History),
    Scroll(u32, Scrolling),
    Mark(Mark),

    Visual(bool),
    VisualSwapLead,
    VisualSwapDiagonal,

    Undo(u32),
    Redo(u32),

    JoinLines(u32),
    Insert(u32),
    Append(u32),
    AppendLine(u32),
    PrependLine(u32),
    Delete(u32, Motion),
    Change(u32, Motion),
    Yank(u32, Motion),
    CopyClipboard(u32, Motion),
    Paste(u32, bool),
    PasteClipboard(u32, bool),
    Replace(u32, char),
    Dedent,
    Indent,
}

fn is_visual_memo(vim: &Vim) -> bool {
    match vim {
        Vim::Change(_, Motion::Visual) => true,
        _ => false,
    }
}

fn is_normal_memo(vim: &Vim) -> bool {
    match vim {
        Vim::Invalid => false,
        Vim::Repeat(_) => false,
        Vim::Partial(_, _) => false,
        Vim::Move(_, _) => false,
        Vim::Scroll(_, _) => false,
        Vim::Mark(_) => false,
        Vim::History(_, _) => false,
        Vim::Visual(_) => false,
        Vim::VisualSwapLead => false,
        Vim::VisualSwapDiagonal => false,
        Vim::Undo(_) => false,
        Vim::Redo(_) => false,
        Vim::JoinLines(_) => true,
        Vim::Insert(_) => true,
        Vim::Append(_) => true,
        Vim::AppendLine(_) => true,
        Vim::PrependLine(_) => true,
        Vim::Delete(_, _) => true,
        Vim::Change(_, _) => true,
        Vim::Dedent => true,
        Vim::Indent => true,
        Vim::Replace(_, _) => true,
        Vim::Yank(_, _) => false,
        Vim::CopyClipboard(_, _) => false,
        Vim::Paste(_, _) => true,
        Vim::PasteClipboard(_, _) => true,
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
            yank: Default::default(),
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
        let r = match execute_visual(&vim, state, vi) {
            Ok(v) => {
                if is_visual_memo(&vim) {
                    debug!("visual memo {:?}", vim);
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
                if is_normal_memo(&vim) {
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
        Vim::History(_, _) => unreachable!("unknown"),
        Vim::Scroll(_, _) => unreachable!("unknown"),
        Vim::Visual(_) => unreachable!("unknown"),

        Vim::Move(mul, m) => visual_move(*mul, m, state, vi)?,
        Vim::Mark(_) => unreachable!("unknown"),

        Vim::Partial(mul, Motion::SearchForward(s)) => {
            search_fwd(*mul, s, true, state, vi)?;
            scroll_to_search_idx(state, vi);
        }
        Vim::Partial(mul, Motion::SearchBack(s)) => {
            search_back(*mul, s, true, state, vi)?;
            scroll_to_search_idx(state, vi);
        }
        Vim::Partial(_, _) => unreachable!("unknown"),

        Vim::VisualSwapDiagonal => visual_swap_diagonal(state, vi),
        Vim::VisualSwapLead => visual_swap_lead(state, vi),
        Vim::Delete(_, _) => visual_delete(state, vi),
        Vim::Change(_, _) => visual_change(state, vi),
        Vim::Yank(_, _) => visual_yank(state, vi),
        Vim::CopyClipboard(_, _) => visual_copy_clipboard(state, vi),
        Vim::Undo(_) => unreachable!("unknown"),
        Vim::Redo(_) => unreachable!("unknown"),
        Vim::JoinLines(_) => unreachable!("unknown"),
        Vim::Insert(_) => unreachable!("unknown"),
        Vim::Append(_) => unreachable!("unknown"),
        Vim::AppendLine(_) => unreachable!("unknown"),
        Vim::PrependLine(_) => unreachable!("unknown"),
        Vim::Dedent => unreachable!("unknown"),
        Vim::Indent => unreachable!("unknown"),
        Vim::Paste(_, _) => unreachable!("unknown"),
        Vim::PasteClipboard(_, _) => unreachable!("unknown"),
        Vim::Replace(_, _) => unreachable!("unknown"),
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

        Vim::Move(mul, m) => move_cursor(*mul, m, state, vi)?,

        Vim::Partial(mul, Motion::SearchForward(s)) => {
            search_fwd(*mul, s, true, state, vi)?;
            scroll_to_search_idx(state, vi);
        }
        Vim::Partial(mul, Motion::SearchBack(s)) => {
            search_back(*mul, s, true, state, vi)?;
            scroll_to_search_idx(state, vi);
        }
        Vim::Partial(_, _) => unreachable!("unknown partial"),

        Vim::Scroll(mul, Scrolling::PageUp) => scroll_page_up(*mul, state, vi),
        Vim::Scroll(mul, Scrolling::PageDown) => scroll_page_down(*mul, state, vi),
        Vim::Scroll(mul, Scrolling::Up) => scroll_up(*mul, state),
        Vim::Scroll(mul, Scrolling::Down) => scroll_down(*mul, state),
        Vim::Scroll(_, Scrolling::MiddleOfScreen) => scroll_cursor_to_middle(state),
        Vim::Scroll(_, Scrolling::TopOfScreen) => scroll_cursor_to_top(state),
        Vim::Scroll(_, Scrolling::BottomOfScreen) => scroll_cursor_to_bottom(state),

        Vim::Mark(mark) => set_mark(*mark, state, vi),
        Vim::History(mul, History::NextJump) => jump_history(*mul as i32, state, vi),
        Vim::History(mul, History::PrevJump) => jump_history(-(*mul as i32), state, vi),
        Vim::History(mul, History::NextChange) => {
            todo!()
        }
        Vim::History(mul, History::PrevChange) => {
            todo!()
        }

        Vim::Visual(block) => begin_visual(*block, state, vi),
        Vim::VisualSwapLead => unreachable!("unknown"),
        Vim::VisualSwapDiagonal => unreachable!("unknown"),

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
                begin_insert(state, vi);
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
                change_text_str(*mul, m, state, vi)?;
                insert_str(1, state, vi);
                r = TextOutcome::TextChanged;
            } else {
                begin_change_text(*mul, m, state, vi)?;
                r = TextOutcome::TextChanged;
            }
        }
        Vim::Yank(mul, m) => yank_text(*mul, m, state, vi)?,
        Vim::CopyClipboard(mul, m) => copy_clipboard_text(*mul, m, state, vi)?,
        Vim::Paste(mul, before) => paste_text(*mul, *before, state, vi),
        Vim::PasteClipboard(mul, before) => paste_clipboard_text(*mul, *before, state, vi),
        Vim::Delete(mul, m) => {
            delete_text(*mul, m, state, vi)?;
            r = TextOutcome::TextChanged;
        }
        Vim::Replace(mul, c) => {
            replace_text(*mul, *c, state, vi)?;
            r = TextOutcome::TextChanged;
        }
        Vim::Dedent => {
            todo!()
        }
        Vim::Indent => {
            todo!()
        }
    }

    display_matches(state, vi);
    display_finds(state, vi);
    display_visual(state, vi);

    Ok(r)
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
                    display_visual(self, vi);
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
