//!
//! This tests some boundary conditions.
//! More to test things than an example.
//!

use crate::data::render_tablestate::render_tablestate;
use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{layout_grid, run_ui, setup_logging, MiniSalsaState};
use format_num_pattern::NumberFormat;
use rat_event::ct_event;
use rat_event::util::item_at;
use rat_ftable::event::Outcome;
use rat_ftable::selection::{rowselection, RowSelection};
use rat_ftable::textdata::{Cell, Row};
use rat_ftable::{Table, TableContext, TableDataIter, TableState};
use rat_scrolled::Scroll;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::text::Span;
use ratatui::widgets::{block, Block, StatefulWidget, Widget};
use ratatui::Frame;
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

    let mut data = Data {
        table_data: data::SMALL_DATA
            .iter()
            .map(|v| Sample {
                text: *v,
                num1: rand::random(),
                num2: rand::random(),
                check: rand::random(),
            })
            .collect(),
    };

    let mut state = State {
        table: Default::default(),
        report_rows: None,
        no_row_count: false,
        edit: Default::default(),
    };

    run_ui(
        "insane_offset",
        handle_table,
        repaint_table,
        &mut data,
        &mut state,
    )
}

struct Sample {
    pub(crate) text: &'static str,
    pub(crate) num1: f32,
    pub(crate) num2: f32,
    pub(crate) check: bool,
}

struct Data {
    pub(crate) table_data: Vec<Sample>,
}

struct State {
    pub(crate) table: TableState<RowSelection>,
    pub(crate) report_rows: Option<usize>,
    pub(crate) no_row_count: bool,
    pub(crate) edit: [[Rect; 10]; 1],
}

fn repaint_table(
    frame: &mut Frame<'_>,
    area: Rect,
    data: &mut Data,
    _istate: &mut MiniSalsaState,
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

    "rows() reports".render(state.edit[0][0], frame.buffer_mut());
    let mut b_none = Span::from("None").style(THEME.deepblue(0));
    if state.report_rows == None {
        b_none = b_none.style(THEME.deepblue(3));
    }
    frame.render_widget(b_none, state.edit[0][1]);
    let mut b_none = Span::from("Too few").style(THEME.deepblue(0));
    if state.report_rows == Some(SMALLER) {
        b_none = b_none.style(THEME.deepblue(3));
    }
    frame.render_widget(b_none, state.edit[0][2]);
    let mut b_none = Span::from("Circa").style(THEME.deepblue(0));
    if state.report_rows == Some(CIRCA) {
        b_none = b_none.style(THEME.deepblue(3));
    }
    frame.render_widget(b_none, state.edit[0][3]);
    let mut b_none = Span::from("Exact").style(THEME.deepblue(0));
    if state.report_rows == Some(EXACT) {
        b_none = b_none.style(THEME.deepblue(3));
    }
    frame.render_widget(b_none, state.edit[0][4]);
    let mut b_none = Span::from("Too many").style(THEME.deepblue(0));
    if state.report_rows == Some(GREATER) {
        b_none = b_none.style(THEME.deepblue(3));
    }
    frame.render_widget(b_none, state.edit[0][5]);

    let mut nocount = Span::from("no_row_count").style(THEME.deepblue(0));
    if state.no_row_count {
        nocount = nocount.style(THEME.deepblue(0).fg(THEME.red[3]));
    }
    frame.render_widget(nocount, state.edit[0][7]);

    let goto = Span::from("GOTO 1_000_000").style(THEME.deepblue(0));
    frame.render_widget(goto, state.edit[0][9]);

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
            iter: data.table_data.iter().enumerate(),
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
        .header(
            Row::new([
                Cell::from("Nr"),
                Cell::from("Text"),
                Cell::from("Val1"),
                Cell::from("Val2"),
                Cell::from("State"),
            ])
            .style(Some(THEME.table_header())),
        )
        .footer(Row::new(["a", "b", "c", "d", "e"]).style(Some(THEME.table_footer())))
        .block(
            Block::bordered()
                .border_type(block::BorderType::Rounded)
                .border_style(THEME.block())
                .title("offsets"),
        )
        .vscroll(Scroll::new().style(THEME.block()))
        .flex(Flex::End)
        .styles(THEME.table_style())
        .select_row_style(Some(THEME.gray(3)))
        .render(l0[1], frame.buffer_mut(), &mut state.table);

    render_tablestate(&state.table, l1[2], frame.buffer_mut());

    Ok(())
}

fn handle_table(
    event: &crossterm::event::Event,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
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
