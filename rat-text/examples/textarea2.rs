use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use rat_event::{ct_event, try_flow, Outcome};
use rat_scrolled::{Scroll, ScrollbarPolicy};
use rat_text::text_area::{TextArea, TextAreaState, TextBreak};
use rat_text::{text_area, HasScreenCursor};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Paragraph, StatefulWidget};
use ratatui::Frame;
use std::fmt;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        info: true,
        textarea: Default::default(),
    };
    state.textarea.set_auto_indent(false);
    state.textarea.set_text("1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 XXXX_\n");
    state.textarea.set_text_break(TextBreak::Shift);
    state.textarea.set_show_ctrl(true);
    state.textarea.set_cursor((56, 0), false);

    run_ui(
        "textarea2",
        handle_input,
        repaint_input,
        &mut data,
        &mut state,
    )
}

struct Data {}

struct State {
    pub(crate) info: bool,
    pub(crate) textarea: TextAreaState,
}

fn repaint_input(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([
        Constraint::Length(7),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);

    let l2 = Layout::horizontal([
        Constraint::Length(15),
        Constraint::Length(25),
        Constraint::Fill(1),
        Constraint::Length(15),
    ])
    .split(l1[1]);

    let txt_area = l2[2];

    let textarea = TextArea::new()
        .block(Block::bordered())
        .vscroll(
            Scroll::new()
                .scroll_by(1)
                .overscroll_by(50)
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
    // debug!("{:#?}", textarea);
    // debug!("{:#?}", state.textarea);
    textarea.render(txt_area, frame.buffer_mut(), &mut state.textarea);

    if let Some((cx, cy)) = state.textarea.screen_cursor() {
        frame.set_cursor_position((cx, cy));
    }

    if state.info {
        use fmt::Write;
        let mut stats = String::new();
        _ = writeln!(&mut stats);
        _ = writeln!(
            &mut stats,
            "offset: {:?} {:?}",
            state.textarea.offset(),
            state.textarea.sub_row_offset
        );
        _ = writeln!(&mut stats, "cursor: {:?}", state.textarea.cursor(),);
        _ = writeln!(&mut stats, "anchor: {:?}", state.textarea.anchor());
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
        frame.render_widget(dbg, l2[1]);
    }

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
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    try_flow!(text_area::handle_events(&mut state.textarea, true, event));

    try_flow!(match event {
        ct_event!(key press ALT-'0') => {
            state.info = !state.info;
            Outcome::Changed
        }
        ct_event!(key press ALT-'1') => {
            let focus = state.textarea.focus.clone();

            state.textarea = TextAreaState::new();
            state.textarea.focus = focus;
            state.textarea.set_text("1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n");
            state.textarea.set_text_break(TextBreak::Shift);
            state.textarea.set_show_ctrl(true);

            Outcome::Changed
        }
        ct_event!(key press ALT-'2') => {
            let focus = state.textarea.focus.clone();

            state.textarea = TextAreaState::new();
            state.textarea.focus = focus;
            state.textarea.set_text("1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n");
            state.textarea.set_text_break(TextBreak::Word(8));
            state.textarea.set_show_ctrl(true);

            Outcome::Changed
        }
        ct_event!(key press ALT-'3') => {
            let focus = state.textarea.focus.clone();

            state.textarea = TextAreaState::new();
            state.textarea.focus = focus;
            state.textarea.set_text("1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n");
            state.textarea.set_text_break(TextBreak::Hard);
            state.textarea.set_show_ctrl(true);

            Outcome::Changed
        }
        ct_event!(key press ALT-'4') => {
            let focus = state.textarea.focus.clone();

            state.textarea = TextAreaState::new();
            state.textarea.focus = focus;
            state.textarea.set_text("1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 XXXX_\n");
            state.textarea.set_text_break(TextBreak::Shift);
            state.textarea.set_show_ctrl(true);

            Outcome::Changed
        }
        ct_event!(key press ALT-'5') => {
            let focus = state.textarea.focus.clone();

            state.textarea = TextAreaState::new();
            state.textarea.focus = focus;
            state.textarea.set_text("1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 XXXX_\n");
            state.textarea.set_text_break(TextBreak::Word(8));
            state.textarea.set_show_ctrl(true);

            Outcome::Changed
        }
        ct_event!(key press ALT-'6') => {
            let focus = state.textarea.focus.clone();

            state.textarea = TextAreaState::new();
            state.textarea.focus = focus;
            state.textarea.set_text("1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 XXXX_\n");
            state.textarea.set_text_break(TextBreak::Hard);
            state.textarea.set_show_ctrl(true);

            Outcome::Changed
        }
        ct_event!(key press ALT-'c') => {
            state.textarea.set_show_ctrl(!state.textarea.show_ctrl());
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    Ok(Outcome::Continue)
}
