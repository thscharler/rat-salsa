use crate::mini_salsa::{MiniSalsaState, layout_grid, mock_init, run_ui, setup_logging};
use log::warn;
use rat_event::{HandleEvent, Regular, event_flow};
use rat_event::{Outcome, ct_event};
use rat_focus::{FocusBuilder, HasFocus};
use rat_text::HasScreenCursor;
use rat_text::text_input_mask::{MaskedInput, MaskedInputState};
use rat_theme4::StyleName;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Widget};
use std::fmt;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State {
        info: true,
        masked: Default::default(),
        mask_idx: 0,
    };

    run_ui("textmask1", mock_init, event, render, &mut state)
}

struct State {
    pub(crate) info: bool,
    pub(crate) masked: MaskedInputState,
    pub(crate) mask_idx: usize,
}

fn render(
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
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

    MaskedInput::new()
        .block(Block::bordered().style(ctx.theme.style_style(Style::INPUT)))
        .style(ctx.theme.style_style(Style::INPUT))
        .focus_style(ctx.theme.style_style(Style::INPUT_FOCUS))
        .select_style(ctx.theme.style_style(Style::INPUT_SELECT))
        .text_style([
            Style::new().red(),
            Style::new().underlined(),
            Style::new().green(),
            Style::new().on_yellow(),
        ])
        .render(ll[1][2], buf, &mut state.masked);

    if let Some((cx, cy)) = state.masked.screen_cursor() {
        ctx.cursor = Some((cx, cy));
    }

    let info_area = Rect::new(ll[0][1].x + 1, ll[0][1].y + 1, ll[0][1].width - 2, 1);
    let info = Line::from("F2 next mask").black().on_cyan();
    info.render(info_area, buf);

    let mask_area = Rect::new(ll[0][2].x + 1, ll[0][2].y + 2, ll[0][2].width - 2, 1);
    let mask = Line::from(state.masked.mask()).black().on_cyan();
    mask.render(mask_area, buf);

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
            state.masked.section_range(state.masked.cursor())
        );
        _ = writeln!(
            &mut stats,
            "prev-sec: {:?}",
            state.masked.prev_section_range(sel.start)
        );
        _ = writeln!(
            &mut stats,
            "next-sec: {:?}",
            state.masked.next_section_range(sel.end)
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
        let cursor_byte = state.masked.byte_at(state.masked.cursor());
        state.masked.styles_at(cursor_byte.start, &mut styles);
        for (_, s) in styles {
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

        let lx = ll[3][1].union(ll[3][2]).union(ll[3][3]).union(ll[3][4]);
        Paragraph::new(stats).render(lx, buf);
    }

    let ccursor = state.masked.selection();
    ctx.status[0] = format!("{}-{}", ccursor.start, ccursor.end);

    Ok(())
}

fn event(
    event: &crossterm::event::Event,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let mut focus = {
        let mut fb = FocusBuilder::default();
        fb.widget(&state.masked);
        fb.build()
    };
    ctx.focus_outcome = focus.handle(event, Regular);

    event_flow!(state.masked.handle(event, Regular));

    match event {
        ct_event!(key press ALT-'0') => event_flow!({
            state.info = !state.info;
            Outcome::Changed
        }),
        ct_event!(keycode press F(2)) => event_flow!(next_mask(state)),
        ct_event!(keycode press SHIFT-F(2)) => event_flow!(prev_mask(state)),
        _ => {}
    }

    Ok(Outcome::Continue)
}

static MASKS: [&str; 39] = [
    "",
    "##\\/##\\/####",
    "##\\.##\\.####",
    "\\€ ###,##0.0#+",
    "\\€ ###,##0.0#-",
    "HH HH HH",
    "dd\\°dd\\'dd\\\"",
    "90\\°90\\'90\\\"",
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
    "aaaddaaa",
];

fn next_mask(state: &mut State) -> Outcome {
    state.mask_idx = (state.mask_idx + 1) % MASKS.len();

    match state.masked.set_mask(MASKS[state.mask_idx]) {
        Ok(_) => {}
        Err(e) => {
            warn!("{:?}", e)
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
            warn!("{:?}", e)
        }
    };
    Outcome::Changed
}
