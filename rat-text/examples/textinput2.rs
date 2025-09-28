use crate::mini_salsa::{MiniSalsaState, fill_buf_area, mock_init, run_ui, setup_logging};
use log::{debug, warn};
use rat_event::{HandleEvent, Outcome, Regular, ct_event, try_flow};
use rat_focus::{Focus, FocusBuilder, match_focus};
use rat_text::HasScreenCursor;
use rat_text::clipboard::{Clipboard, ClipboardError, set_global_clipboard};
use rat_text::event::TextOutcome;
use rat_text::text_input::{TextInput, TextInputState};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Widget};
use std::cell::RefCell;
use std::fmt;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    set_global_clipboard(CliClipboard::default());

    let mut data = Data {};

    let mut state = State {
        info: true,
        help: false,
        textinput1: Default::default(),
        textinput2: Default::default(),
        textinput3: Default::default(),
    };
    insert_text_1(&mut state);

    run_ui(
        "textinput2",
        mock_init,
        event,
        render,
        &mut data,
        &mut state,
    )
}

struct Data {}

struct State {
    pub(crate) info: bool,
    pub(crate) help: bool,
    pub(crate) textinput1: TextInputState,
    pub(crate) textinput2: TextInputState,
    pub(crate) textinput3: TextInputState,
}

fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([
        Constraint::Length(1), //
        Constraint::Fill(1),
    ])
    .split(area);
    let l2 = Layout::horizontal([
        Constraint::Length(15),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(25),
    ])
    .split(area);
    let l_txt = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(l2[1]);

    TextInput::new()
        .block(Block::bordered())
        .styles(istate.theme.text_style())
        .text_style([
            Style::new().red(),
            Style::new().underlined(),
            Style::new().green(),
            Style::new().on_yellow(),
        ])
        .render(l_txt[1], frame.buffer_mut(), &mut state.textinput1);

    TextInput::new().styles(istate.theme.text_style()).render(
        l_txt[3],
        frame.buffer_mut(),
        &mut state.textinput2,
    );

    TextInput::new().styles(istate.theme.text_style()).render(
        l_txt[5],
        frame.buffer_mut(),
        &mut state.textinput3,
    );

    if let Some(c) = state
        .textinput1
        .screen_cursor()
        .or(state.textinput2.screen_cursor())
        .or(state.textinput3.screen_cursor())
    {
        frame.set_cursor_position(c);
    }

    fill_buf_area(
        frame.buffer_mut(),
        l1[0],
        " ",
        istate
            .theme
            .palette()
            .normal_contrast(istate.theme.palette().orange[0]),
    );
    "F1 toggle help | Ctrl+Q quit".render(l1[0], frame.buffer_mut());

    if state.help {
        let help_area = l1[1].intersection(l2[1]);
        fill_buf_area(
            frame.buffer_mut(),
            help_area,
            " ",
            istate
                .theme
                .palette()
                .normal_contrast(istate.theme.palette().bluegreen[1]),
        );
        Paragraph::new(
            r#"
    ** HELP **            
            
    ALT-1..4 Sample text
    ALT-0    toggle info
    ALT-c    show ctrl
    ALT-d    dump cache (to log)
"#,
        )
        .style(
            istate
                .theme
                .palette()
                .normal_contrast(istate.theme.palette().bluegreen[1]),
        )
        .render(help_area, frame.buffer_mut());
    }

    if let Some(textinput) = match_focus!(
        state.textinput1 => Some(&state.textinput1),
        state.textinput2 => Some(&state.textinput2),
        state.textinput3 => Some(&state.textinput3),
        else => None
    ) {
        if state.info {
            use fmt::Write;
            let mut stats = String::new();
            _ = writeln!(&mut stats);
            _ = writeln!(&mut stats, "cursor: {:?}", textinput.cursor(),);
            _ = writeln!(&mut stats, "anchor: {:?}", textinput.anchor());
            if let Some((scx, scy)) = textinput.screen_cursor() {
                _ = writeln!(&mut stats, "screen: {}:{}", scx, scy);
            } else {
                _ = writeln!(&mut stats, "screen: None",);
            }
            _ = writeln!(&mut stats, "width: {:?} ", textinput.line_width());

            _ = writeln!(
                &mut stats,
                "next word: {:?} {:?}",
                textinput.next_word_start(textinput.cursor()),
                textinput.next_word_end(textinput.cursor())
            );
            _ = writeln!(
                &mut stats,
                "prev word: {:?} {:?}",
                textinput.prev_word_start(textinput.cursor()),
                textinput.prev_word_end(textinput.cursor())
            );

            _ = write!(&mut stats, "cursor-styles: ",);
            let mut styles = Vec::new();
            let cursor_byte = textinput.byte_at(textinput.cursor());
            textinput.styles_at(cursor_byte.start, &mut styles);
            for (_r, s) in styles {
                _ = write!(&mut stats, "{}, ", s);
            }
            _ = writeln!(&mut stats);

            if let Some(st) = textinput.value.styles() {
                _ = writeln!(&mut stats, "text-styles: {}", st.count());
            }
            if let Some(st) = textinput.value.styles() {
                for r in st.take(20) {
                    _ = writeln!(&mut stats, "    {:?}", r);
                }
            }
            let dbg = Paragraph::new(stats);
            frame.render_widget(dbg, l1[1].intersection(l2[3]));
        }

        let ccursor = textinput.selection();
        istate.status[0] = format!("{}-{}", ccursor.start, ccursor.end);
    } else {
        istate.status[0] = Default::default();
    }

    Ok(())
}

fn focus(state: &mut State) -> Focus {
    let mut builder = FocusBuilder::new(None);
    builder.widget(&state.textinput1);
    builder.widget(&state.textinput2);
    builder.widget(&state.textinput3);
    builder.build()
}

fn event(
    event: &crossterm::event::Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let mut focus = focus(state);

    istate.focus_outcome = focus.handle(event, Regular);

    try_flow!({
        match state.textinput1.handle(event, Regular) {
            TextOutcome::TextChanged => {
                state.textinput1.invalid = state.textinput1.text() == "42";
                TextOutcome::Changed
            }
            r => r,
        }
    });
    try_flow!(state.textinput2.handle(event, Regular));
    try_flow!(state.textinput3.handle(event, Regular));

    try_flow!(match event {
        ct_event!(key press ALT-'0') => {
            state.info = !state.info;
            Outcome::Changed
        }
        ct_event!(key press ALT-'1') => insert_text_0(state),
        ct_event!(key press ALT-'2') => insert_text_1(state),
        ct_event!(key press ALT-'3') => insert_text_2(state),
        ct_event!(key press ALT-'4') => insert_text_3(state),

        ct_event!(key press ALT-'c') => {
            state
                .textinput1
                .set_show_ctrl(!state.textinput1.show_ctrl());
            Outcome::Changed
        }
        ct_event!(key press ALT-'d') => {
            debug!("{:#?}", state.textinput1.value.cache());
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

pub(crate) fn insert_text_3(state: &mut State) -> Outcome {
    let l = lorem_rustum::LoremRustum::new(1_000);

    let mut style = Vec::new();

    let mut buf = String::new();
    let mut pos = 0;
    let mut width = 0;
    for p in l.body {
        buf.push_str(p);
        buf.push_str(" ");
        width += p.len() + 1;

        if p == "macro" {
            style.push((pos..pos + p.len(), 0));
        } else if p == "assert!" {
            style.push((pos..pos + p.len(), 1));
        } else if p == "<'a>" {
            style.push((pos..pos + p.len(), 2));
        } else if p == "await" {
            style.push((pos..pos + p.len(), 3));
        }

        pos += p.len() + 1;

        if width > 66 {
            buf.push_str("\n");
            width = 0;
            pos += 1;
        }
    }

    state.textinput1.set_text(buf);
    state.textinput1.set_styles(style);

    Outcome::Changed
}

pub(crate) fn insert_text_2(state: &mut State) -> Outcome {
    state.textinput1.set_text("");
    Outcome::Changed
}

pub(crate) fn insert_text_1(state: &mut State) -> Outcome {
    let str = "w🤷‍♂️x w🤷‍♀️x w🤦‍♂️x w❤️x w🤦‍♀️x w💕x w🙍🏿‍♀️x";
    // let str = "word word word word word word word";
    state.textinput1.set_text(str);
    Outcome::Changed
}

pub(crate) fn insert_text_0(state: &mut State) -> Outcome {
    state.textinput1.set_text(
        "Sir Ridley Scott GBE[1] (* 30. November 1937 in South Shields, England) ist ein
britischer Filmregisseur und Filmproduzent.",
    );

    state.textinput1.add_range_style(4..16, 0).unwrap();

    Outcome::Changed
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
