use crate::mini_salsa::{MiniSalsaState, fill_buf_area, run_ui, setup_logging};
use crate::text_samples::{
    add_range_styles, sample_bosworth_1, sample_irish, sample_long, sample_lorem_ipsum,
    sample_medium, sample_scott_1, sample_tabs,
};
use log::{debug, warn};
use rat_event::{HandleEvent, Outcome, Regular, ct_event, try_flow};
use rat_focus::{Focus, FocusBuilder, HasFocus};
use rat_scrolled::{Scroll, ScrollbarPolicy};
use rat_text::clipboard::{Clipboard, ClipboardError, set_global_clipboard};
use rat_text::event::TextOutcome;
use rat_text::line_number::{LineNumberState, LineNumbers};
use rat_text::text_area::{TextArea, TextAreaState, TextWrap};
use rat_text::text_input::{TextInput, TextInputState};
use rat_text::{HasScreenCursor, TextPosition, upos_type};
use rat_vim::{Mode, VI};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::symbols::border;
use ratatui::symbols::border::EMPTY;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, StatefulWidget, Widget};
use ropey::Rope;
use std::cell::RefCell;
use std::fs::File;
use std::io::BufReader;
use std::time::SystemTime;

mod mini_salsa;
mod text_samples;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    set_global_clipboard(CliClipboard::default());

    let mut data = Data {};

    let mut state = State {
        info: true,
        relative_line_nr: false,
        search: Default::default(),
        textarea_vim: Default::default(),
        textarea: Default::default(),
        line_numbers: Default::default(),
        help: false,
    };
    state.textarea.focus.set(true);
    state.textarea.set_auto_indent(false);
    state.textarea.set_text_wrap(TextWrap::Word(2));
    state.textarea.clear();

    run_ui(
        "vi motion",
        |_, istate, _| {
            istate.timing = false;
        },
        event,
        render,
        &mut data,
        &mut state,
    )
}

struct Data {}

struct State {
    pub info: bool,
    pub relative_line_nr: bool,
    pub search: TextInputState,
    pub textarea_vim: VI,
    pub textarea: TextAreaState,
    pub line_numbers: LineNumberState,
    pub help: bool,
}

fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    const BARE: bool = false;

    let l1 = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);

    // let line_number_width = LineNumbers::width_for(state.textarea.offset().1, 0, (0, 0), 0);
    let line_number_width = LineNumbers::width_for(1000, 0, (0, 0), 0);

    let l2 = Layout::horizontal([
        Constraint::Length(2),
        Constraint::Length(line_number_width),
        Constraint::Fill(1),
        Constraint::Length(25),
    ])
    .spacing(1)
    .split(l1[2]);

    let l23 = Layout::horizontal([
        Constraint::Length(2),
        Constraint::Length(line_number_width),
        Constraint::Fill(1),
        Constraint::Length(25),
    ])
    .spacing(1)
    .split(l1[3]);

    let textarea = TextArea::new()
        .block(Block::bordered())
        .vscroll(
            Scroll::new()
                .scroll_by(1)
                // .overscroll_by(50)
                .policy(ScrollbarPolicy::Always),
        )
        .hscroll(Scroll::new().policy(ScrollbarPolicy::Collapse))
        .styles(istate.theme.textarea_style())
        .set_horizontal_max_offset(256)
        .text_style([
            Style::new().red(),
            Style::new().underlined(),
            Style::new().green(),
            Style::new().on_yellow(),
        ])
        .text_style_idx(
            999,
            istate
                .theme
                .palette()
                .normal_contrast(istate.theme.palette().bluegreen[0]),
        );
    let t = SystemTime::now();
    textarea.render(l2[2], frame.buffer_mut(), &mut state.textarea);
    let el = t.elapsed().expect("timinig");

    let mb = state.textarea_vim.motion_buf.borrow();
    Line::from_iter([
        Span::from(format!(" {:?} ", state.textarea_vim.mode)).style(
            match state.textarea_vim.mode {
                Mode::Normal => istate
                    .theme
                    .palette()
                    .high_contrast(istate.theme.palette().limegreen[2]),
                Mode::Insert => istate
                    .theme
                    .palette()
                    .high_contrast(istate.theme.palette().orange[2]),
                Mode::Visual => istate
                    .theme
                    .palette()
                    .high_contrast(istate.theme.palette().yellow[2]),
            },
        ),
        Span::from(" "),
        Span::from(mb.as_str()).style(istate.theme.palette().text_bright),
    ])
    .render(l23[2], frame.buffer_mut());

    if !BARE {
        LineNumbers::new()
            .block(
                Block::new()
                    .borders(Borders::TOP | Borders::BOTTOM)
                    .border_set(EMPTY),
            )
            .styles(istate.theme.line_nr_style())
            .with_textarea(&state.textarea)
            .relative(state.relative_line_nr)
            .render(l2[1], frame.buffer_mut(), &mut state.line_numbers);
    }

    istate.status[1] = format!("R{}|{:.0?}", frame.count(), el,).to_string();

    let l3 = Layout::horizontal([
        Constraint::Length(43),
        Constraint::Fill(1),
        Constraint::Percentage(20),
    ])
    .split(l1[0]);

    fill_buf_area(
        frame.buffer_mut(),
        l1[0],
        " ",
        istate
            .theme
            .palette()
            .normal_contrast(istate.theme.palette().orange[0]),
    );
    "F1 toggle help | Ctrl+Q quit | Alt-F(ind) ".render(l1[0], frame.buffer_mut());

    if !BARE {
        TextInput::new()
            .block(
                Block::new()
                    .borders(Borders::LEFT | Borders::RIGHT)
                    .border_set(border::Set {
                        top_left: "",
                        top_right: "",
                        bottom_left: "",
                        bottom_right: "",
                        vertical_left: "[",
                        vertical_right: "]",
                        horizontal_top: "",
                        horizontal_bottom: "",
                    })
                    .style(istate.theme.container_border()),
            )
            .styles(istate.theme.text_style())
            .render(l3[2], frame.buffer_mut(), &mut state.search);
    }

    let screen_cursor = if !state.help {
        state
            .textarea
            .screen_cursor()
            .or(state.search.screen_cursor())
    } else {
        None
    };
    if let Some((cx, cy)) = screen_cursor {
        frame.set_cursor_position((cx, cy));
    }

    if state.help {
        fill_buf_area(
            frame.buffer_mut(),
            l2[2],
            " ",
            istate
                .theme
                .palette()
                .normal_contrast(istate.theme.palette().bluegreen[0]),
        );
        Paragraph::new(
            r#"
    ** HELP **            
            
    ALT-1..8 Sample text
    ALT-l    open 'log.log'
    ALT-q    no wrap
    ALT-w    word wrap
    ALT-e    hard wrap
    ALT-c    show ctrl
    ALT-x    show breaks
    ALT-d    dump cache (to log)
    Alt-n    toggle absolute/relative line nr
"#,
        )
        .style(
            istate
                .theme
                .palette()
                .normal_contrast(istate.theme.palette().bluegreen[0]),
        )
        .render(l2[2], frame.buffer_mut());
    }

    if state.info {
        use std::fmt::Write;
        let mut stats = String::new();
        _ = writeln!(&mut stats);
        _ = writeln!(&mut stats, "cursor: {:?}", state.textarea.cursor());
        _ = writeln!(&mut stats, "anchor: {:?}", state.textarea.anchor());
        _ = writeln!(&mut stats, "offset: {:?}", state.textarea.offset());
        _ = writeln!(&mut stats, "sub_row: {:?}", state.textarea.sub_row_offset());

        _ = writeln!(
            &mut stats,
            "v_max: {:?}",
            state.textarea.vertical_max_offset()
        );
        _ = writeln!(&mut stats, "v_page: {:?}", state.textarea.vertical_page());

        _ = writeln!(&mut stats, "movecol: {:?}", state.textarea.move_col());
        if let Some((scx, scy)) = state.textarea.screen_cursor() {
            _ = writeln!(&mut stats, "screen: {}:{}", scx, scy);
        } else {
            _ = writeln!(&mut stats, "screen: None",);
        }
        _ = writeln!(
            &mut stats,
            "width: {:?} ",
            state.textarea.line_width(state.textarea.cursor().y)
        );
        _ = writeln!(
            &mut stats,
            "next word: {:?} {:?}",
            state.textarea.next_word_start(state.textarea.cursor()),
            state.textarea.next_word_end(state.textarea.cursor())
        );
        _ = writeln!(
            &mut stats,
            "prev word: {:?} {:?}",
            state.textarea.prev_word_start(state.textarea.cursor()),
            state.textarea.prev_word_end(state.textarea.cursor())
        );

        _ = write!(&mut stats, "cursor-styles: ",);
        let mut styles = Vec::new();
        let cursor_byte = state.textarea.byte_at(state.textarea.cursor());
        state.textarea.styles_at(cursor_byte.start, &mut styles);
        for (_, s) in styles {
            _ = write!(&mut stats, "{}, ", s);
        }
        _ = writeln!(&mut stats);

        if let Some(st) = state.textarea.value.styles() {
            _ = writeln!(&mut stats, "text-styles: {}", st.count());
        }
        if let Some(st) = state.textarea.value.styles() {
            for r in st.take(20) {
                _ = writeln!(&mut stats, "    {:?}", r);
            }
        }
        let dbg = Paragraph::new(stats);
        frame.render_widget(dbg, l2[3]);
    }

    let ccursor = state.textarea.selection();
    istate.status[0] = format!(
        "{}:{} - {}:{} | wrap {:?}",
        ccursor.start.y,
        ccursor.start.x,
        ccursor.end.y,
        ccursor.end.x,
        state.textarea.text_wrap()
    );

    Ok(())
}

fn focus(state: &mut State) -> Focus {
    let mut ff = FocusBuilder::new(None);
    ff.widget(&state.textarea);
    ff.widget(&state.search);
    ff.build()
}

fn event(
    event: &crossterm::event::Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let mut focus = focus(state);
    // focus.enable_log();

    istate.focus_outcome = focus.handle(event, Regular);

    if !state.help {
        try_flow!({
            let t = SystemTime::now();
            let r = state.textarea.handle(event, &mut state.textarea_vim)?;
            let el = t.elapsed().expect("timing");
            istate.status[2] = format!("H{}|{:?}", istate.event_cnt, el).to_string();
            r
        });

        try_flow!({
            let r = state.search.handle(event, Regular);
            match r {
                TextOutcome::TextChanged => {
                    run_search(state);
                    TextOutcome::Changed
                }
                r => r,
            }
        });
    }

    try_flow!(match event {
        ct_event!(key press ALT-'0') => {
            state.info = !state.info;
            Outcome::Changed
        }
        ct_event!(key press ALT-'1') => {
            state.textarea.clear();
            Outcome::Changed
        }
        ct_event!(key press ALT-'2') => {
            let (text, styles) = sample_scott_1();
            state.textarea.set_rope(text);
            add_range_styles(&mut state.textarea, styles);
            Outcome::Changed
        }
        ct_event!(key press ALT-'3') => {
            let (text, styles) = sample_irish();
            state.textarea.set_rope(text);
            add_range_styles(&mut state.textarea, styles);
            Outcome::Changed
        }
        ct_event!(key press ALT-'4') => {
            let (text, styles) = sample_tabs();
            state.textarea.set_rope(text);
            add_range_styles(&mut state.textarea, styles);
            Outcome::Changed
        }
        ct_event!(key press ALT-'5') => {
            let (text, styles) = sample_lorem_ipsum();
            state.textarea.set_rope(text);
            add_range_styles(&mut state.textarea, styles);
            // let (text, styles) = sample_lorem_rustum();
            // state.textarea.set_rope(text);
            // state.textarea.set_styles(styles);
            Outcome::Changed
        }
        ct_event!(key press ALT-'6') => {
            let (text, styles) = sample_bosworth_1();
            state.textarea.set_rope(text);
            add_range_styles(&mut state.textarea, styles);
            Outcome::Changed
        }
        ct_event!(key press ALT-'7') => {
            let (text, styles) = sample_medium();
            state.textarea.set_rope(text);
            add_range_styles(&mut state.textarea, styles);
            Outcome::Changed
        }
        ct_event!(key press ALT-'8') => {
            let (text, styles) = sample_long();
            state.textarea.set_rope(text);
            add_range_styles(&mut state.textarea, styles);
            Outcome::Changed
        }
        ct_event!(key press ALT-'h') => {
            let hhh = (state.textarea.rendered.height * 8 / 10) as upos_type;
            state.textarea.set_cursor(TextPosition::new(0, hhh), false);
            Outcome::Changed
        }
        ct_event!(key press ALT-'j') => {
            let hhh = (state.textarea.rendered.height * 2 / 10) as upos_type;
            let len = state.textarea.len_lines();
            state
                .textarea
                .set_cursor(TextPosition::new(0, len - hhh), false);
            Outcome::Changed
        }
        ct_event!(key press ALT-'l') => {
            let file = File::open("log.log")?;
            let buf = BufReader::new(file);
            let text = Rope::from_reader(buf)?;
            state.textarea.set_rope(text);
            Outcome::Changed
        }

        ct_event!(key press ALT-'f') => {
            if state.search.is_focused() {
                focus.focus(&state.textarea);
            } else {
                focus.focus(&state.search);
            }
            Outcome::Changed
        }

        ct_event!(key press ALT-'n') => {
            state.relative_line_nr = !state.relative_line_nr;
            Outcome::Changed
        }

        ct_event!(key press ALT-'q') => {
            state.textarea.set_text_wrap(TextWrap::Shift);
            Outcome::Changed
        }
        ct_event!(key press ALT-'w') => {
            state.textarea.set_text_wrap(TextWrap::Word(9));
            Outcome::Changed
        }
        ct_event!(key press ALT-'e') => {
            state.textarea.set_text_wrap(TextWrap::Hard);
            Outcome::Changed
        }
        ct_event!(key press ALT-'c') => {
            state.textarea.set_show_ctrl(!state.textarea.show_ctrl());
            Outcome::Changed
        }
        ct_event!(key press ALT-'x') => {
            state.textarea.set_wrap_ctrl(!state.textarea.wrap_ctrl());
            Outcome::Changed
        }
        ct_event!(key press ALT-'d') => {
            debug!("{:#?}", state.textarea.value.cache());
            Outcome::Changed
        }
        ct_event!(keycode press F(1) ) => {
            state.help = !state.help;
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    Ok(Outcome::Continue)
}

fn run_search(state: &mut State) {
    let search_text = state.search.text();

    // TODO: will kill any sample styling ...
    state.textarea.set_styles(Vec::default());

    if search_text.len() < 1 {
        return;
    }

    // TODO: this is not fast
    let text = state.textarea.text();

    let mut start = 0;
    loop {
        let Some(pos) = text[start..].find(search_text) else {
            break;
        };
        state
            .textarea
            .add_style(start + pos..start + pos + search_text.len(), 999);
        start = start + pos + search_text.len();
    }
}

#[derive(Debug, Default, Clone)]
struct CliClipboard {
    clip: RefCell<String>,
}

impl Clipboard for CliClipboard {
    fn get_string(&self) -> Result<String, ClipboardError> {
        match cli_clipboard::get_contents() {
            Ok(v) => Ok(v),
            Err(e) => {
                warn!("{:?}", e);
                Ok(self.clip.borrow().clone())
            }
        }
    }

    fn set_string(&self, s: &str) -> Result<(), ClipboardError> {
        let mut clip = self.clip.borrow_mut();
        *clip = s.to_string();

        match cli_clipboard::set_contents(s.to_string()) {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("{:?}", e);
                Err(ClipboardError)
            }
        }
    }
}
