use crate::mini_salsa::{MiniSalsaState, STATUS, fill_buf_area, run_ui, setup_logging};
use crate::text_samples::{
    add_range_styles, sample_bosworth_1, sample_irish, sample_long, sample_lorem_ipsum,
    sample_medium, sample_rust, sample_scott_1,
};
use log::{debug, warn};
use rat_event::{HandleEvent, Outcome, Regular, ct_event, try_flow};
use rat_focus::{Focus, FocusBuilder};
use rat_scrolled::{Scroll, ScrollbarPolicy};
use rat_text::clipboard::{Clipboard, ClipboardError, set_global_clipboard};
use rat_text::line_number::{LineNumberState, LineNumbers};
use rat_text::text_area::{TextArea, TextAreaState, TextWrap};
use rat_text::{HasScreenCursor, TextPosition, upos_type};
use rat_vim::VI;
use rat_vim::vi_status_line::VIStatusLine;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::style::Style;
use ratatui_core::terminal::Frame;
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::paragraph::Paragraph;
use ropey::Rope;
use std::cell::RefCell;
use std::fs::File;
use std::io::BufReader;
use std::sync::atomic::Ordering;
use std::time::{Duration, SystemTime};

mod mini_salsa;
mod text_samples;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    set_global_clipboard(CliClipboard::default());

    let mut data = Data {};

    let mut state = State {
        line_nr: true,
        relative_line_nr: false,
        textarea_vim: Default::default(),
        textarea: Default::default(),
        line_numbers: Default::default(),
        render_dur: Default::default(),
        event_dur: Default::default(),
        help: false,
    };
    state.textarea.focus.set(true);
    state.textarea.set_auto_indent(false);
    state.textarea.set_text_wrap(TextWrap::Word(2));
    state.textarea.clear();

    STATUS.store(false, Ordering::Release);

    run_ui(
        "vi core",
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
    pub line_nr: bool,
    pub relative_line_nr: bool,
    pub textarea_vim: VI,
    pub textarea: TextAreaState,
    pub line_numbers: LineNumberState,

    pub render_dur: Duration,
    pub event_dur: Duration,

    pub help: bool,
}

#[allow(dead_code)]
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
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .split(area);

    let line_number_width = if state.line_nr {
        LineNumbers::width_for(state.textarea.vertical_offset(), 0, (0, 1), 0)
    } else {
        1
    };

    let l22 = Layout::horizontal([
        Constraint::Length(0),
        Constraint::Length(line_number_width),
        Constraint::Fill(1),
        Constraint::Length(0),
    ])
    .split(l1[1]);

    let l23 = Layout::horizontal([
        Constraint::Length(0),
        Constraint::Fill(1),
        Constraint::Length(0),
    ])
    .split(l1[2]);

    let pal = istate.theme.palette();

    let h_scroll = Scroll::horizontal()
        .begin_symbol(Some("◀"))
        .end_symbol(Some("▶"))
        .track_symbol(Some("─"))
        .thumb_symbol("▄")
        .min_symbol(Some("─"));
    let v_scroll = Scroll::vertical()
        .begin_symbol(Some("▲"))
        .end_symbol(Some("▼"))
        .track_symbol(Some("│"))
        .thumb_symbol("█")
        .min_symbol(Some("│"));

    let textarea = TextArea::new()
        .vscroll(v_scroll.scroll_by(1).policy(ScrollbarPolicy::Always))
        .hscroll(h_scroll.policy(ScrollbarPolicy::Collapse))
        .styles(istate.theme.textview_style())
        .set_horizontal_max_offset(256)
        .text_style([
            Style::new().red(),
            Style::new().underlined(),
            Style::new().green(),
            Style::new().on_yellow(),
        ])
        .text_style_idx(999, pal.normal_contrast(pal.bluegreen[1]))
        .text_style_idx(998, pal.normal_contrast(pal.green[1]))
        .text_style_idx(997, pal.normal_contrast(pal.limegreen[1]));
    let t = SystemTime::now();
    textarea.render(l22[2], frame.buffer_mut(), &mut state.textarea);
    state.render_dur = t.elapsed().expect("timinig");

    VIStatusLine::new()
        .style(istate.theme.status_base())
        .name(" ≡vi-core≡ ")
        .name_style(pal.high_contrast(pal.blue[1]))
        .normal_style(pal.high_contrast(pal.limegreen[2]))
        .insert_style(pal.high_contrast(pal.orange[2]))
        .visual_style(pal.high_contrast(pal.yellow[2]))
        .pos_style(pal.high_contrast(pal.gray[0]))
        .render(
            l23[1],
            frame.buffer_mut(),
            &mut (&mut state.textarea, &mut state.textarea_vim),
        );

    if state.line_nr {
        let mut l_line = l22[1];
        // no autocorrection for hidden scrollbar.
        if state.textarea.hscroll.max_offset > 0 {
            l_line.height -= 1;
        }
        LineNumbers::new()
            .margin((0, 1))
            .styles(istate.theme.line_nr_style())
            .style(istate.theme.gray(5))
            .with_textarea(&state.textarea)
            .relative(state.relative_line_nr)
            .render(l_line, frame.buffer_mut(), &mut state.line_numbers);
    }

    fill_buf_area(
        frame.buffer_mut(),
        l1[0],
        " ",
        pal.normal_contrast(pal.blue[7]),
    );
    format!(
        "F1 toggle help | Ctrl+Q quit | R{} | R{:.0?} | E{:.0?}",
        frame.count(),
        state.render_dur,
        state.event_dur
    )
    .to_string()
    .render(l1[0], frame.buffer_mut());

    let screen_cursor = if !state.help {
        state.textarea.screen_cursor()
    } else {
        None
    };
    if let Some((cx, cy)) = screen_cursor {
        frame.set_cursor_position((cx, cy));
    }

    if state.help {
        fill_buf_area(
            frame.buffer_mut(),
            l22[2],
            " ",
            pal.normal_contrast(pal.bluegreen[0]),
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
    Alt-n    toggle line nr
    Alt-m    toggle absolute/relative line nr
"#,
        )
        .style(pal.normal_contrast(pal.bluegreen[0]))
        .render(l22[2], frame.buffer_mut());
    }

    istate.status[0] = format!("wrap {:?}", state.textarea.text_wrap());

    Ok(())
}

fn focus(state: &mut State) -> Focus {
    let mut ff = FocusBuilder::new(None);
    ff.widget(&state.textarea);
    ff.build()
}

fn event(
    event: &Event,
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
            state.event_dur = t.elapsed().expect("timing");
            r
        });
    }

    try_flow!(match event {
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
            let (text, styles) = sample_rust();
            state.textarea.set_rope(text);
            add_range_styles(&mut state.textarea, styles);
            Outcome::Changed
        }
        ct_event!(key press ALT-'5') => {
            let (text, styles) = sample_lorem_ipsum();
            state.textarea.set_rope(text);
            add_range_styles(&mut state.textarea, styles);
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

        ct_event!(key press ALT-'n') => {
            state.line_nr = !state.line_nr;
            Outcome::Changed
        }
        ct_event!(key press ALT-'m') => {
            state.line_nr = true;
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
