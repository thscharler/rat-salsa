use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use log::debug;
#[allow(unused_imports)]
use rat_event::{ct_event, flow_ok, Outcome};
use rat_text::text_input::{TextInput, TextInputState};
use rat_text::text_input_mask::{MaskedInput, MaskedInputState};
use rat_text::{text_input, text_input_mask};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Widget};
use ratatui::Frame;
use std::fmt;
use std::fmt::Error;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        info: true,
        masked: Default::default(),
        mask_idx: 0,
    };
    insert_text_1(&mut state);

    run_ui(handle_input, repaint_input, &mut data, &mut state)
}

struct Data {}

struct State {
    pub(crate) info: bool,
    pub(crate) masked: MaskedInputState,
    pub(crate) mask_idx: usize,
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

    MaskedInput::new()
        .block(Block::bordered().style(Style::default().gray().on_dark_gray()))
        .style(Style::default().white().on_dark_gray())
        .select_style(Style::default().black().on_yellow())
        .text_style([
            Style::new().red(),
            Style::new().underlined(),
            Style::new().green(),
            Style::new().on_yellow(),
        ])
        .render(l2[1], frame.buffer_mut(), &mut state.masked);

    if let Some((cx, cy)) = state.masked.screen_cursor() {
        frame.set_cursor(cx, cy);
    }

    let info_area = Rect::new(l2[0].x + 1, l2[0].y + 1, l2[0].width - 2, 1);
    let info = Line::from("F2 next mask").black().on_cyan();
    info.render(info_area, frame.buffer_mut());

    let mask_area = Rect::new(l2[0].x + 1, l2[0].y + 2, l2[0].width - 2, 1);
    let mask = Line::from(state.masked.mask()).black().on_cyan();
    mask.render(mask_area, frame.buffer_mut());

    if state.info {
        use fmt::Write;
        let mut stats = String::new();
        _ = writeln!(&mut stats);
        _ = writeln!(&mut stats, "cursor: {:?}", state.masked.cursor(),);
        _ = writeln!(&mut stats, "anchor: {:?}", state.masked.anchor());
        if let Some((scx, scy)) = state.masked.screen_cursor() {
            _ = writeln!(&mut stats, "screen: {}:{}", scx, scy);
        } else {
            _ = writeln!(&mut stats, "screen: None",);
        }
        _ = writeln!(&mut stats, "width: {:?} ", state.masked.line_width());

        _ = writeln!(&mut stats, "mask: {:?}", state.masked.mask(),);
        _ = writeln!(&mut stats, "text: {:?}", state.masked.text(),);

        _ = write!(&mut stats, "cursor-styles: ",);
        let mut styles = Vec::new();
        let cursor_byte = state.masked.byte_at(state.masked.cursor()).expect("cursor");
        state.masked.styles_at(cursor_byte.start, &mut styles);
        for s in styles {
            _ = write!(&mut stats, "{}, ", s);
        }
        _ = writeln!(&mut stats);

        if let Some(st) = state.masked.value.styles() {
            _ = writeln!(&mut stats, "text-styles: {}", st.count());
        }
        if let Some(st) = state.masked.value.styles() {
            for r in st.take(20) {
                _ = writeln!(&mut stats, "    {:?}", r);
            }
        }
        let dbg = Paragraph::new(stats);
        frame.render_widget(dbg, l2[3]);
    }

    let ccursor = state.masked.selection();
    istate.status[0] = format!("{}-{}", ccursor.start, ccursor.end);

    Ok(())
}

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    flow_ok!(text_input_mask::handle_events(
        &mut state.masked,
        true,
        event
    ));

    flow_ok!(match event {
        ct_event!(key press ALT-'0') => {
            state.info = !state.info;
            Outcome::Changed
        }
        ct_event!(keycode press F(2)) => next_mask(state),
        ct_event!(keycode press SHIFT-F(2)) => prev_mask(state),
        _ => Outcome::Continue,
    });

    Ok(Outcome::Continue)
}

static MASKS: [&str; 29] = [
    "",
    "#",
    "##",
    "###",
    "##0",
    "#00",
    "000",
    "###.#",
    "###.##",
    "###.###",
    "###.0",
    "###.0##",
    "###.00",
    "###.00#",
    "###.000",
    "##0.000",
    "#00.000",
    "##/##/####",
    "###,##0.0##",
    "###,##0.0##-",
    "###,##0.0##+",
    "€ ###,##0.0##+",
    "HHH",
    "HH HH HH",
    "llllll",
    "aaaaaa",
    "cccccc",
    "______",
    "dd°dd'dd\"",
];

fn next_mask(state: &mut State) -> Outcome {
    state.mask_idx = (state.mask_idx + 1) % MASKS.len();

    match state.masked.set_mask(MASKS[state.mask_idx]) {
        Ok(_) => {}
        Err(e) => {
            debug!("{:?}", e)
        }
    };
    Outcome::Changed
}

fn prev_mask(state: &mut State) -> Outcome {
    if state.mask_idx == 0 {
        state.mask_idx = MASKS.len() - 1;
    } else {
        state.mask_idx -= 1;
    }

    match state.masked.set_mask(MASKS[state.mask_idx]) {
        Ok(_) => {}
        Err(e) => {
            debug!("{:?}", e)
        }
    };
    Outcome::Changed
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

    state.masked.set_text(buf);
    state.masked.set_styles(style);

    Outcome::Changed
}

pub(crate) fn insert_text_2(state: &mut State) -> Outcome {
    state.masked.set_text("");
    Outcome::Changed
}

pub(crate) fn insert_text_1(state: &mut State) -> Outcome {
    let str = "w🤷‍♂️x w🤷‍♀️x w🤦‍♂️x w❤️x w🤦‍♀️x w💕x w🙍🏿‍♀️x";
    state.masked.set_text(str);
    Outcome::Changed
}

pub(crate) fn insert_text_0(state: &mut State) -> Outcome {
    state.masked.set_text(
        "Sir Ridley Scott GBE[1] (* 30. November 1937 in South Shields, England) ist ein
britischer Filmregisseur und Filmproduzent. Er gilt heute als einer der",
    );

    state.masked.add_range_style(4..16, 0).unwrap();

    Outcome::Changed
}
