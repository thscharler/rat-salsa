//!
//! This tests some boundary conditions.
//! More to test things than an example.
//!

use crate::mini_salsa::mock_init;
use crate::mini_salsa::{MiniSalsaState, layout_grid, run_ui, setup_logging};
use crate::widget::render_tablestate::render_tablestate;
use format_num_pattern::NumberFormat;
use rat_event::util::item_at;
use rat_event::{HandleEvent, Regular, ct_event, event_flow};
use rat_focus::HasFocus;
use rat_ftable2::event::Outcome;
use rat_ftable2::selection::row::RowSelection;
use rat_ftable2::{Table, TableState};
use rat_scrolled::{Scroll, ScrollStyle};
use rat_theme4::StyleName;
use rat_theme4::palette::Colors;
use rat_theme4::theme::SalsaTheme;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Flex, Layout, Rect};
use ratatui_core::style::Style;
use ratatui_core::text::Span;
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::Block;
use ratatui_widgets::borders::BorderType;

const SMALLER: usize = 90;
const CIRCA: usize = 99;
const EXACT: usize = 100;
const GREATER: usize = 110;

mod data;
mod mini_salsa;
mod widget;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State {
        table_data: data::SMALL_DATA
            .iter()
            .map(|v| Sample {
                text: *v,
                num1: rand::random(),
                num2: rand::random(),
                check: rand::random(),
            })
            .collect(),
        table: Default::default(),
        report_rows: None,
        no_row_count: false,
        edit: Default::default(),
    };

    run_ui("insane_offset", mock_init, event, render, &mut state)
}

struct Sample {
    pub(crate) text: &'static str,
    pub(crate) num1: f32,
    pub(crate) num2: f32,
    pub(crate) check: bool,
}

struct State {
    table_data: Vec<Sample>,
    table: TableState<RowSelection>,
    report_rows: Option<usize>,
    no_row_count: bool,
    edit: [[Rect; 10]; 1],
}

fn render(
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l0 = Layout::horizontal([
        Constraint::Length(35),
        Constraint::Fill(1),
        Constraint::Length(20),
    ])
    .split(area);

    let l1 = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(11),
        Constraint::Length(11),
    ])
    .split(l0[0]);

    state.edit = layout_grid::<1, 10>(
        l1[1],
        Layout::horizontal([Constraint::Length(20)]),
        Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ]),
    );

    "rows() reports".render(state.edit[0][0], buf);
    let mut b_none = Span::from("None").style(ctx.theme.p.deepblue(0));
    if state.report_rows == None {
        b_none = b_none.style(ctx.theme.p.deepblue(3));
    }
    b_none.render(state.edit[0][1], buf);
    let mut b_none = Span::from("Too few").style(ctx.theme.p.deepblue(0));
    if state.report_rows == Some(SMALLER) {
        b_none = b_none.style(ctx.theme.p.deepblue(3));
    }
    b_none.render(state.edit[0][2], buf);
    let mut b_none = Span::from("Circa").style(ctx.theme.p.deepblue(0));
    if state.report_rows == Some(CIRCA) {
        b_none = b_none.style(ctx.theme.p.deepblue(3));
    }
    b_none.render(state.edit[0][3], buf);
    let mut b_none = Span::from("Exact").style(ctx.theme.p.deepblue(0));
    if state.report_rows == Some(EXACT) {
        b_none = b_none.style(ctx.theme.p.deepblue(3));
    }
    b_none.render(state.edit[0][4], buf);
    let mut b_none = Span::from("Too many").style(ctx.theme.p.deepblue(0));
    if state.report_rows == Some(GREATER) {
        b_none = b_none.style(ctx.theme.p.deepblue(3));
    }
    b_none.render(state.edit[0][5], buf);

    let mut nocount = Span::from("no_row_count").style(ctx.theme.p.deepblue(0));
    if state.no_row_count {
        nocount = nocount.style(
            ctx.theme
                .p
                .deepblue(0)
                .fg(ctx.theme.p.color(Colors::Red, 3)),
        );
    }
    nocount.render(state.edit[0][7], buf);

    let goto = Span::from("GOTO 1_000_000").style(ctx.theme.p.deepblue(0));
    goto.render(state.edit[0][9], buf);

    let row_fmt = NumberFormat::new("000000").expect("fmt");
    let num1_fmt = NumberFormat::new("####0.00").expect("fmt");
    let num2_fmt = NumberFormat::new("####0.00").expect("fmt");

    Table::default()
        .data(state.table_data.iter())
        .widths([
            Constraint::Length(6),
            Constraint::Length(20),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(3),
        ])
        .column_spacing(1)
        .header(vec!["Nr", "Text", "Val1", "Val2", "State"])
        .footer(vec!["a", "b", "c", "d", "e"])
        .render_cell(0, |_v, area, buf, ctx| {
            row_fmt.fmt_u(ctx.row).render(area, buf)
        })
        .render_cell(1, |v, area, buf, _ctx| v.text.render(area, buf))
        .render_cell(2, |v, area, buf, _ctx| {
            num1_fmt.fmt_u(v.num1).render(area, buf)
        })
        .render_cell(3, |v, area, buf, _ctx| {
            num2_fmt.fmt_u(v.num2).render(area, buf)
        })
        .render_cell(4, |v, area, buf, _ctx| {
            let cc = if v.check { "\u{2622}" } else { "\u{2623}" };
            cc.render(area, buf);
        })
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title("offsets"),
        )
        .vscroll(Scroll::new())
        .flex(Flex::End)
        .styles(table(&ctx.theme))
        .select_row_style(Some(ctx.theme.p.gray(3)))
        .render(l0[1], buf, &mut state.table);

    render_tablestate(&state.table, l1[2], buf);

    Ok(())
}

fn table(th: &SalsaTheme) -> rat_ftable2::TableStyle {
    rat_ftable2::TableStyle {
        style: th.style(Style::CONTAINER_BASE),
        select_row: Some(th.style(Style::SELECT)),
        show_row_focus: true,
        focus_style: Some(th.style(Style::FOCUS)),
        border_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        scroll: Some(scroll(th)),
        header: Some(th.style(Style::HEADER)),
        footer: Some(th.style(Style::FOOTER)),
        ..Default::default()
    }
}

fn scroll(th: &SalsaTheme) -> ScrollStyle {
    ScrollStyle {
        thumb_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        track_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        min_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        begin_style: Some(th.style(Style::CONTAINER_ARROW_FG)),
        end_style: Some(th.style(Style::CONTAINER_ARROW_FG)),
        ..Default::default()
    }
}

fn event(
    event: &Event,
    _ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    match event {
        ct_event!(mouse down Left for x,y) => match item_at(&state.edit[0], *x, *y) {
            Some(1) => event_flow!({
                state.report_rows = None;
                Outcome::Changed
            }),
            Some(2) => event_flow!({
                state.report_rows = Some(SMALLER);
                Outcome::Changed
            }),
            Some(3) => event_flow!({
                state.report_rows = Some(CIRCA);
                Outcome::Changed
            }),
            Some(4) => event_flow!({
                state.report_rows = Some(EXACT);
                Outcome::Changed
            }),
            Some(5) => event_flow!({
                state.report_rows = Some(GREATER);
                Outcome::Changed
            }),
            Some(7) => event_flow!({
                state.no_row_count = !state.no_row_count;
                Outcome::Changed
            }),
            Some(9) => event_flow!({
                state.table.core.vscroll.offset = 1_000_000; // 1_000_000;
                Outcome::Changed
            }),
            _ => {}
        },
        _ => {}
    }

    // override focus
    state.table.focus().set(true);
    event_flow!(state.table.handle(event, Regular));

    Ok(Outcome::Continue)
}
