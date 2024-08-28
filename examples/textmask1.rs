use crate::mini_salsa::{layout_grid, run_ui, setup_logging, MiniSalsaState};
use log::debug;
#[allow(unused_imports)]
use rat_event::{ct_event, try_flow, Outcome};
use rat_event::{flow, ConsumedEvent, HandleEvent, Regular};
use rat_focus::{Focus, HasFocusFlag};
use rat_text::text_input::{TextInput, TextInputState};
use rat_text::text_input_mask::{MaskedInput, MaskedInputState};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Widget};
use ratatui::Frame;
use std::fmt;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        info: true,
        sample1: Default::default(),
        sample2: Default::default(),
        masked: Default::default(),
        mask_idx: 0,
    };

    run_ui(handle_input, repaint_input, &mut data, &mut state)
}

struct Data {}

struct State {
    pub(crate) info: bool,
    pub(crate) sample1: TextInputState,
    pub(crate) sample2: TextInputState,
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
    let ll: [[Rect; 6]; 4] = layout_grid(
        area,
        Layout::horizontal([
            Constraint::Length(25),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(25),
        ]),
        Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ]),
    );

    TextInput::new()
        .style(Style::default().white().on_dark_gray())
        .select_style(Style::default().black().on_yellow())
        .focus_style(Style::default().black().on_cyan())
        .render(ll[1][1], frame.buffer_mut(), &mut state.sample1);

    MaskedInput::new()
        .block(Block::bordered().style(Style::default().gray().on_dark_gray()))
        .style(Style::default().white().on_dark_gray())
        .focus_style(Style::default().black().on_cyan())
        .select_style(Style::default().black().on_yellow())
        .text_style([
            Style::new().red(),
            Style::new().underlined(),
            Style::new().green(),
            Style::new().on_yellow(),
        ])
        .render(ll[1][2], frame.buffer_mut(), &mut state.masked);

    TextInput::new()
        .style(Style::default().white().on_dark_gray())
        .select_style(Style::default().black().on_yellow())
        .focus_style(Style::default().black().on_cyan())
        .render(ll[1][3], frame.buffer_mut(), &mut state.sample2);

    if let Some((cx, cy)) = state
        .masked
        .screen_cursor()
        .or_else(|| state.sample1.screen_cursor())
        .or_else(|| state.sample2.screen_cursor())
    {
        frame.set_cursor(cx, cy);
    }

    let info_area = Rect::new(ll[0][1].x + 1, ll[0][1].y + 1, ll[0][1].width - 2, 1);
    let info = Line::from("F2 next mask").black().on_cyan();
    info.render(info_area, frame.buffer_mut());

    let mask_area = Rect::new(ll[0][2].x + 1, ll[0][2].y + 2, ll[0][2].width - 2, 1);
    let mask = Line::from(state.masked.mask()).black().on_cyan();
    mask.render(mask_area, frame.buffer_mut());

    if state.info {
        use fmt::Write;
        let mut stats = String::new();
        _ = writeln!(&mut stats);
        _ = writeln!(&mut stats, "cursor: {:?}", state.masked.cursor(),);
        _ = writeln!(&mut stats, "anchor: {:?}", state.masked.anchor());

        _ = writeln!(&mut stats, "navig: {:?}", state.masked.navigable());

        let sel = state.masked.selection();
        _ = writeln!(&mut stats, "selection: {:?}", sel.clone());
        _ = writeln!(
            &mut stats,
            "curr-sec: {:?}",
            state.masked.value.section_range(state.masked.cursor())
        );
        _ = writeln!(
            &mut stats,
            "prev-sec: {:?}",
            state.masked.value.prev_section_range(sel.start)
        );
        _ = writeln!(
            &mut stats,
            "next-sec: {:?}",
            state.masked.value.next_section_range(sel.end)
        );

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
        let lx = ll[3][1].union(ll[3][2]).union(ll[3][3]).union(ll[3][4]);
        frame.render_widget(dbg, lx);
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
    let mut f = Focus::new_list(&[&state.sample1, &state.masked, &state.sample2]);

    let r = f.handle(event, Regular);
    let r = r.and(|| {
        flow!(state.sample1.handle(event, Regular));
        flow!(state.masked.handle(event, Regular));
        flow!(state.sample2.handle(event, Regular));
        flow!(match event {
            ct_event!(key press ALT-'0') => {
                state.info = !state.info;
                Outcome::Changed
            }
            ct_event!(keycode press F(2)) => next_mask(state),
            ct_event!(keycode press SHIFT-F(2)) => prev_mask(state),
            _ => Outcome::Continue,
        });
        Outcome::Continue
    });

    Ok(r)
}

static MASKS: [&str; 36] = [
    "",
    //
    "##/##/####",
    "€ ###,##0.0##+",
    "HH HH HH",
    "dd\\°dd\\'dd\\\"",
    "90\\°90\\'90\\\"",
    //
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
    "990.000-",
    "990.000+",
    "-990.000-",
    "+990.000+",
    "###,##0.0##",
    "###,##0.0##-",
    "###,##0.0##+",
    "HHH",
    "dddd dddd dddd dddd \\c\\v\\c ddd",
    "\\C\\C aaaa \\R aaaa \\I ddd",
    "llllll",
    "aaaaaa",
    "cccccc",
    "______",
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
