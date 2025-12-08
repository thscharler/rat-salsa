//!
//! This tests some boundary conditions.
//! More to test things than an example.
//!

use crate::data::render_tablestate::render_tablestate;
use crate::mini_salsa::mock_init;
use crate::mini_salsa::{MiniSalsaState, layout_grid, run_ui, setup_logging};
use format_num_pattern::NumberFormat;
use rat_event::ct_event;
use rat_event::util::item_at;
use rat_ftable::event::Outcome;
use rat_ftable::selection::{RowSelection, rowselection};
use rat_ftable::textdata::{Cell, Row};
use rat_ftable::{Table, TableContext, TableDataIter, TableState};
use rat_scrolled::{Scroll, ScrollStyle};
use rat_theme4::StyleName;
use rat_theme4::palette::Colors;
use rat_theme4::theme::SalsaTheme;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{Block, StatefulWidget, Widget, block};
use std::cmp::max;
use std::iter::Enumerate;
use std::slice::Iter;

const SMALLER: usize = 90;
const CIRCA: usize = 99;
const EXACT: usize = 100;
const GREATER: usize = 110;

mod data;
mod mini_salsa;

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

    // table
    struct RowIter1<'a> {
        report_rows: Option<usize>,
        iter: Enumerate<Iter<'a, Sample>>,
        item: Option<(usize, &'a Sample)>,
    }

    impl<'a> TableDataIter<'a> for RowIter1<'a> {
        fn rows(&self) -> Option<usize> {
            self.report_rows
        }

        fn nth(&mut self, n: usize) -> bool {
            self.item = self.iter.nth(n);
            self.item.is_some()
        }

        fn render_cell(&self, _ctx: &TableContext, column: usize, area: Rect, buf: &mut Buffer) {
            let row = self.item.expect("data");
            match column {
                0 => {
                    let row_fmt = NumberFormat::new("000000").expect("fmt");
                    let span = Span::from(row_fmt.fmt_u(row.0));
                    span.render(area, buf);
                }
                1 => {
                    let span = Span::from(row.1.text);
                    span.render(area, buf);
                }
                2 => {
                    let num1_fmt = NumberFormat::new("####0.00").expect("fmt");
                    let span = Span::from(num1_fmt.fmt_u(row.1.num1));
                    span.render(area, buf);
                }
                3 => {
                    let num2_fmt = NumberFormat::new("####0.00").expect("fmt");
                    let span = Span::from(num2_fmt.fmt_u(row.1.num2));
                    span.render(area, buf);
                }
                4 => {
                    let cc = if row.1.check { "\u{2622}" } else { "\u{2623}" };
                    let span = Span::from(cc);
                    span.render(area, buf);
                }
                _ => {}
            }
        }
    }

    Table::default()
        .iter(RowIter1 {
            report_rows: state.report_rows,
            iter: state.table_data.iter().enumerate(),
            item: None,
        })
        .no_row_count(state.no_row_count)
        .widths([
            Constraint::Length(6),
            Constraint::Length(20),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(3),
        ])
        .column_spacing(1)
        .header(Row::new([
            Cell::from("Nr"),
            Cell::from("Text"),
            Cell::from("Val1"),
            Cell::from("Val2"),
            Cell::from("State"),
        ]))
        .footer(Row::new(["a", "b", "c", "d", "e"]))
        .block(
            Block::bordered()
                .border_type(block::BorderType::Rounded)
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

fn table(th: &SalsaTheme) -> rat_ftable::TableStyle {
    rat_ftable::TableStyle {
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
    event: &crossterm::event::Event,
    _ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let r0 = 'f: {
        match event {
            ct_event!(mouse down Left for x,y) => match item_at(&state.edit[0], *x, *y) {
                Some(1) => {
                    state.report_rows = None;
                    break 'f Outcome::Changed;
                }
                Some(2) => {
                    state.report_rows = Some(SMALLER);
                    break 'f Outcome::Changed;
                }
                Some(3) => {
                    state.report_rows = Some(CIRCA);
                    break 'f Outcome::Changed;
                }
                Some(4) => {
                    state.report_rows = Some(EXACT);
                    break 'f Outcome::Changed;
                }
                Some(5) => {
                    state.report_rows = Some(GREATER);
                    break 'f Outcome::Changed;
                }
                Some(7) => {
                    state.no_row_count = !state.no_row_count;
                    break 'f Outcome::Changed;
                }
                Some(9) => {
                    state.table.vscroll.offset = 1_000_000; // 1_000_000;
                    break 'f Outcome::Changed;
                }
                _ => {
                    break 'f Outcome::Continue;
                }
            },
            _ => Outcome::Continue,
        }
    };

    let r1 = rowselection::handle_events(&mut state.table, true, event);
    Ok(max(r0, r1.into()))
}
