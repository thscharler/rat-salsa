use crate::mini_salsa::{fill_buf_area, run_ui, setup_logging, MiniSalsaState};
use crate::text_samples::{
    add_range_styles, sample_bosworth_1, sample_emoji, sample_long, sample_lorem, sample_medium,
    sample_scott_1, sample_short, sample_tabs,
};
use log::debug;
use rat_event::{ct_event, try_flow, Outcome};
use rat_scrolled::{Scroll, ScrollbarPolicy};
use rat_text::text_area::{TextArea, TextAreaState, TextWrap};
use rat_text::{text_area, HasScreenCursor};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Widget};
use ratatui::Frame;
use std::time::SystemTime;

mod mini_salsa;
mod text_samples;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        info: true,
        textarea: Default::default(),
        help: false,
    };
    state.textarea.set_auto_indent(false);
    state.textarea.set_text_wrap(TextWrap::Shift);

    let (text, styles) = sample_scott_1();
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
        Constraint::Length(2),
        Constraint::Fill(1),
        Constraint::Length(2),
        Constraint::Length(1),
    ])
    .split(area);

    let l2 = Layout::horizontal([
        Constraint::Length(0),
        Constraint::Length(2),
        Constraint::Fill(1),
        Constraint::Length(2),
    ])
    .split(l1[1]);

    let txt_area = l2[2];

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
        ]);

    let t = SystemTime::now();
    textarea.render(txt_area, frame.buffer_mut(), &mut state.textarea);
    let el = t.elapsed().expect("timinig");
    istate.status[1] = format!("R{}|{:.0?}", frame.count(), el).to_string();

    let screen_cursor = state.textarea.screen_cursor();
    if let Some((cx, cy)) = screen_cursor {
        frame.set_cursor_position((cx, cy));
    }

    "F1 toggle help | Ctrl+Q quit".render(l1[0], frame.buffer_mut());

    if state.help {
        fill_buf_area(
            frame.buffer_mut(),
            l2[2],
            " ",
            Style::new()
                .bg(istate.theme.bluegreen[1])
                .fg(istate.theme.text_color(istate.theme.bluegreen[1])),
        );
        Paragraph::new(
            r#"
    ALT-0..8 Sample text
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
        .render(l2[2], frame.buffer_mut());
    }

    // if state.info {
    //     use fmt::Write;
    //     let mut stats = String::new();
    //     _ = writeln!(&mut stats);
    //     _ = writeln!(
    //         &mut stats,
    //         "offset: {:?} {:?}",
    //         state.textarea.offset(),
    //         state.textarea.sub_row_offset
    //     );
    //     _ = writeln!(&mut stats, "cursor: {:?}", state.textarea.cursor(),);
    //     _ = writeln!(&mut stats, "anchor: {:?}", state.textarea.anchor());
    //     if let Some((scx, scy)) = screen_cursor {
    //         _ = writeln!(&mut stats, "screen: {}:{}", scx, scy);
    //     } else {
    //         _ = writeln!(&mut stats, "screen: None",);
    //     }
    //     _ = writeln!(
    //         &mut stats,
    //         "width: {:?} ",
    //         state.textarea.line_width(state.textarea.cursor().y)
    //     );
    //     _ = writeln!(
    //         &mut stats,
    //         "next word: {:?} {:?}",
    //         state.textarea.next_word_start(state.textarea.cursor()),
    //         state.textarea.next_word_end(state.textarea.cursor())
    //     );
    //     _ = writeln!(
    //         &mut stats,
    //         "prev word: {:?} {:?}",
    //         state.textarea.prev_word_start(state.textarea.cursor()),
    //         state.textarea.prev_word_end(state.textarea.cursor())
    //     );
    //
    //     _ = write!(&mut stats, "cursor-styles: ",);
    //     let mut styles = Vec::new();
    //     let cursor_byte = state.textarea.byte_at(state.textarea.cursor());
    //     state.textarea.styles_at(cursor_byte.start, &mut styles);
    //     for (_, s) in styles {
    //         _ = write!(&mut stats, "{}, ", s);
    //     }
    //     _ = writeln!(&mut stats);
    //
    //     if let Some(st) = state.textarea.value.styles() {
    //         _ = writeln!(&mut stats, "text-styles: {}", st.count());
    //     }
    //     if let Some(st) = state.textarea.value.styles() {
    //         for r in st.take(20) {
    //             _ = writeln!(&mut stats, "    {:?}", r);
    //         }
    //     }
    //     let dbg = Paragraph::new(stats);
    //     frame.render_widget(dbg, l2[1]);
    // }

    let ccursor = state.textarea.selection();
    istate.status[0] = format!(
        "{}:{} - {}:{}",
        ccursor.start.y, ccursor.start.x, ccursor.end.y, ccursor.end.x,
    );

    Ok(())
}

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    if !state.help {
        try_flow!({
            let t = SystemTime::now();
            let r = text_area::handle_events(&mut state.textarea, true, event);
            let el = t.elapsed().expect("timing");
            istate.status[2] = format!("H{}|{:?}", istate.event_cnt, el).to_string();
            r
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
            let (text, styles) = sample_lorem();
            state.textarea.set_rope(text);
            state.textarea.set_styles(styles);
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
        ct_event!(key press ALT-'q') => {
            state.textarea.set_text_wrap(TextWrap::Shift);
            Outcome::Changed
        }
        ct_event!(key press ALT-'w') => {
            state.textarea.set_text_wrap(TextWrap::Word(10));
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
