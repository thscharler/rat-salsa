use crate::mini_salsa::{MiniSalsaState, mock_init, run_ui, setup_logging};
use rat_event::{Outcome, ct_event, try_flow};
use rat_text::text_input::{TextInput, TextInputState};
use rat_text::{HasScreenCursor, text_input};
use rat_theme4::StyleName;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::style::Style;
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::Block;
use ratatui_widgets::paragraph::Paragraph;
use std::fmt;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State {
        info: true,
        textinput: Default::default(),
    };
    insert_text_1(&mut state);

    run_ui("textinput1", mock_init, event, render, &mut state)
}

struct State {
    info: bool,
    textinput: TextInputState,
}

fn render(
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);

    let l2 = Layout::horizontal([
        Constraint::Length(15),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(25),
    ])
    .split(l1[1]);

    TextInput::new()
        .block(Block::bordered().style(ctx.theme.style_style(Style::INPUT)))
        .style(ctx.theme.style_style(Style::INPUT))
        .select_style(ctx.theme.style_style(Style::SELECT))
        .render(l2[1], buf, &mut state.textinput);

    if let Some((cx, cy)) = state.textinput.screen_cursor() {
        ctx.cursor = Some((cx, cy));
    }

    if state.info {
        use fmt::Write;
        let mut stats = String::new();
        _ = writeln!(&mut stats);
        _ = writeln!(&mut stats, "cursor: {:?}", state.textinput.cursor(),);
        _ = writeln!(&mut stats, "anchor: {:?}", state.textinput.anchor());
        if let Some((scx, scy)) = state.textinput.screen_cursor() {
            _ = writeln!(&mut stats, "screen: {}:{}", scx, scy);
        } else {
            _ = writeln!(&mut stats, "screen: None",);
        }
        _ = writeln!(&mut stats, "width: {:?} ", state.textinput.line_width());

        _ = writeln!(
            &mut stats,
            "next word: {:?} {:?}",
            state.textinput.next_word_start(state.textinput.cursor()),
            state.textinput.next_word_end(state.textinput.cursor())
        );
        _ = writeln!(
            &mut stats,
            "prev word: {:?} {:?}",
            state.textinput.prev_word_start(state.textinput.cursor()),
            state.textinput.prev_word_end(state.textinput.cursor())
        );

        _ = write!(&mut stats, "cursor-styles: ",);
        let mut styles = Vec::new();
        let cursor_byte = state.textinput.byte_at(state.textinput.cursor());
        state.textinput.styles_at(cursor_byte.start, &mut styles);
        for (_r, s) in styles {
            _ = write!(&mut stats, "{}, ", s);
        }
        _ = writeln!(&mut stats);

        if let Some(st) = state.textinput.value.styles() {
            _ = writeln!(&mut stats, "text-styles: {}", st.count());
        }
        if let Some(st) = state.textinput.value.styles() {
            for r in st.take(20) {
                _ = writeln!(&mut stats, "    {:?}", r);
            }
        }
        Paragraph::new(stats).render(l2[3], buf);
    }

    let ccursor = state.textinput.selection();
    ctx.status[0] = format!("{}-{}", ccursor.start, ccursor.end);

    Ok(())
}

fn event(
    event: &Event,
    _ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    try_flow!(text_input::handle_events(&mut state.textinput, true, event));

    try_flow!(match event {
        ct_event!(key press ALT-'0') => {
            state.info = !state.info;
            Outcome::Changed
        }
        ct_event!(key press ALT-'1') => insert_text_0(state),
        ct_event!(key press ALT-'2') => insert_text_1(state),
        ct_event!(key press ALT-'3') => insert_text_2(state),
        ct_event!(key press ALT-'4') => insert_text_3(state),
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

    state.textinput.set_text(buf);
    state.textinput.set_styles(style);

    Outcome::Changed
}

pub(crate) fn insert_text_2(state: &mut State) -> Outcome {
    state.textinput.set_text("");
    Outcome::Changed
}

pub(crate) fn insert_text_1(state: &mut State) -> Outcome {
    // let str = "w🤷‍♂️x w🤷‍♀️x w🤦‍♂️x w❤️x w🤦‍♀️x w💕x w🙍🏿‍♀️x";
    let str = "word word word word word word word";
    state.textinput.set_text(str);
    Outcome::Changed
}

pub(crate) fn insert_text_0(state: &mut State) -> Outcome {
    state.textinput.set_text(
        "Sir Ridley Scott GBE[1] (* 30. November 1937 in South Shields, England) ist ein
britischer Filmregisseur und Filmproduzent. Er gilt heute als einer der",
    );

    state.textinput.add_range_style(4..16, 0).unwrap();

    Outcome::Changed
}
