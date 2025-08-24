use crate::mini_salsa::{fill_buf_area, run_ui, setup_logging, MiniSalsaState};
use crate::text_samples::{
    add_range_styles, sample_bosworth_1, sample_emoji, sample_long, sample_lorem_ipsum,
    sample_lorem_rustum, sample_medium, sample_scott_1, sample_short, sample_tabs,
};
use log::{debug, warn};
use rat_event::{ct_event, try_flow, HandleEvent, Outcome, Regular};
use rat_focus::{Focus, FocusBuilder, HasFocus};
use rat_scrolled::{Scroll, ScrollbarPolicy};
use rat_text::clipboard::{set_global_clipboard, Clipboard, ClipboardError};
use rat_text::event::TextOutcome;
use rat_text::text_area::{TextArea, TextAreaState, TextWrap};
use rat_text::text_input::{TextInput, TextInputState};
use rat_text::{text_area, HasScreenCursor};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Widget};
use ratatui::Frame;
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
        search: Default::default(),
        textarea: Default::default(),
        help: false,
    };
    state.textarea.focus.set(true);
    state.textarea.set_auto_indent(false);
    state.textarea.set_text_wrap(TextWrap::Word(2));
    let (text, styles) = sample_bosworth_1();
    state.textarea.set_rope(text);
    // state.textarea.set_styles(styles);
    add_range_styles(&mut state.textarea, styles);

    run_ui(
        "textarea2",
        |istate| {
            istate.timing = false;
        },
        handle_input,
        repaint_input,
        &mut data,
        &mut state,
    )
}

struct Data {}

struct State {
    pub info: bool,
    pub search: TextInputState,
    pub textarea: TextAreaState,
    pub help: bool,
}

fn repaint_input(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(2),
        Constraint::Length(1),
    ])
    .split(area);

    let l2 = Layout::horizontal([
        Constraint::Length(2),
        Constraint::Fill(1),
        Constraint::Length(2),
    ])
    .split(l1[2]);

    let textarea = TextArea::new()
        .block(Block::bordered())
        .vscroll(
            Scroll::new()
                .scroll_by(1)
                // .overscroll_by(50)
                .policy(ScrollbarPolicy::Always),
        )
        .hscroll(Scroll::new().policy(ScrollbarPolicy::Always))
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
            Style::new()
                .bg(istate.theme.bluegreen[0])
                .fg(istate.theme.text_color(istate.theme.bluegreen[0])),
        );

    let t = SystemTime::now();
    textarea.render(l2[1], frame.buffer_mut(), &mut state.textarea);
    let el = t.elapsed().expect("timinig");

    istate.status[1] = format!("R{}|{:.0?}", frame.count(), el,).to_string();

    let l3 = Layout::horizontal([
        Constraint::Length(43),
        Constraint::Fill(1),
        Constraint::Length(10),
    ])
    .split(l1[0]);

    fill_buf_area(
        frame.buffer_mut(),
        l1[0],
        " ",
        Style::new()
            .bg(istate.theme.orange[0])
            .fg(istate.theme.text_color(istate.theme.orange[0])),
    );
    "F1 toggle help | Ctrl+Q quit | Alt-F(ind) ".render(l1[0], frame.buffer_mut());
    TextInput::new().styles(istate.theme.input_style()).render(
        l3[1],
        frame.buffer_mut(),
        &mut state.search,
    );

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
            l2[1],
            " ",
            Style::new()
                .bg(istate.theme.bluegreen[1])
                .fg(istate.theme.text_color(istate.theme.bluegreen[1])),
        );
        Paragraph::new(
            r#"
    ** HELP **            
            
    ALT-0..8 Sample text
    ALT-l    open 'log.log'
    ALT-q    no wrap
    ALT-w    word wrap
    ALT-e    hard wrap
    ALT-c    show ctrl
    ALT-x    show breaks
    ALT-d    dump cache (to log)
"#,
        )
        .style(
            Style::new()
                .bg(istate.theme.bluegreen[1])
                .fg(istate.theme.text_color(istate.theme.bluegreen[1])),
        )
        .render(l2[1], frame.buffer_mut());
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

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let mut focus = focus(state);

    try_flow!(focus.handle(event, Regular));

    if !state.help {
        try_flow!({
            let t = SystemTime::now();
            let r = state.textarea.handle(event, Regular);
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
            let (text, styles) = sample_emoji();
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
        ct_event!(key press ALT-'r') => {
            state.textarea.set_text_wrap(TextWrap::Block);
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
