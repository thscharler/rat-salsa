use crate::data::render_tablestate::render_tablestate;
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use format_num_pattern::NumberFormat;
use rat_event::ct_event;
use rat_ftable::event::Outcome;
use rat_ftable::selection::{noselection, NoSelection};
use rat_ftable::textdata::{Cell, Row};
use rat_ftable::{FTable, FTableContext, FTableState, TableDataIter};
use rat_input::layout::{layout_edit, EditConstraint, LayoutEdit};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::prelude::Widget;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::Span;
use ratatui::Frame;
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

    run_ui(handle_table, repaint_table, &mut data, &mut state)
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
    pub(crate) table: FTableState<NoSelection>,
    pub(crate) report_rows: Option<usize>,
    pub(crate) no_row_count: bool,
    pub(crate) edit: LayoutEdit,
}

fn repaint_table(
    frame: &mut Frame<'_>,
    area: Rect,
    data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l0 = Layout::horizontal([
        Constraint::Length(20),
        Constraint::Fill(1),
        Constraint::Length(35),
    ])
    .split(area);

    state.edit = layout_edit(
        area,
        &[
            EditConstraint::TitleLabel,
            EditConstraint::Widget(20),
            EditConstraint::Widget(20),
            EditConstraint::Widget(20),
            EditConstraint::Widget(20),
            EditConstraint::Widget(20),
            EditConstraint::Empty,
            EditConstraint::Widget(20),
            EditConstraint::Empty,
            EditConstraint::Widget(20),
        ],
    );
    let mut lb = state.edit.iter();

    "rows() reports".render(lb.label(), frame.buffer_mut());
    let mut b_none = Span::from("None").white().on_dark_gray();
    if state.report_rows == None {
        b_none = b_none.on_gray();
    }
    frame.render_widget(b_none, lb.widget());
    let mut b_none = Span::from("Too few").white().on_dark_gray();
    if state.report_rows == Some(SMALLER) {
        b_none = b_none.on_gray();
    }
    frame.render_widget(b_none, lb.widget());
    let mut b_none = Span::from("Circa").white().on_dark_gray();
    if state.report_rows == Some(CIRCA) {
        b_none = b_none.on_gray();
    }
    frame.render_widget(b_none, lb.widget());
    let mut b_none = Span::from("Exact").white().on_dark_gray();
    if state.report_rows == Some(EXACT) {
        b_none = b_none.on_gray();
    }
    frame.render_widget(b_none, lb.widget());
    let mut b_none = Span::from("Too many").white().on_dark_gray();
    if state.report_rows == Some(GREATER) {
        b_none = b_none.on_gray();
    }
    frame.render_widget(b_none, lb.widget());

    let mut nocount = Span::from("no_row_count").white().on_light_blue();
    if state.no_row_count {
        nocount = nocount.on_red();
    }
    frame.render_widget(nocount, lb.widget());

    let goto = Span::from("GOTO 1_000_000").white().on_light_blue();
    frame.render_widget(goto, lb.widget());

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

        fn render_cell(&self, _ctx: &FTableContext, column: usize, area: Rect, buf: &mut Buffer) {
            let row = self.item.expect("data");
            match column {
                0 => {
                    let row_fmt = NumberFormat::new("000000").expect("fmt");
                    let span = Span::from(row_fmt.fmt_u(row.0));
                    buf.set_style(area, Style::new().black().bg(Color::from_u32(0xe7c787)));
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

    let table1 = FTable::default()
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
            .style(Some(Style::new().black().bg(Color::from_u32(0x98c379)))),
        )
        .footer(
            Row::new(["a", "b", "c", "d", "e"])
                .style(Some(Style::new().black().bg(Color::from_u32(0x98c379)))),
        )
        .flex(Flex::End)
        .style(Style::default().bg(Color::Rgb(25, 25, 25)));
    frame.render_stateful_widget(table1, l0[1], &mut state.table);

    render_tablestate(&state.table, l0[2], frame.buffer_mut());

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
            ct_event!(mouse down Left for x,y) => match state.edit.widget_at((*x, *y)) {
                Some(0) => {
                    state.report_rows = None;
                    break 'f Outcome::Changed;
                }
                Some(1) => {
                    state.report_rows = Some(SMALLER);
                    break 'f Outcome::Changed;
                }
                Some(2) => {
                    state.report_rows = Some(CIRCA);
                    break 'f Outcome::Changed;
                }
                Some(3) => {
                    state.report_rows = Some(EXACT);
                    break 'f Outcome::Changed;
                }
                Some(4) => {
                    state.report_rows = Some(GREATER);
                    break 'f Outcome::Changed;
                }
                Some(5) => {
                    state.no_row_count = !state.no_row_count;
                    break 'f Outcome::Changed;
                }
                Some(6) => {
                    state.table.vscroll.core.set_raw_offset(99900); // 1_000_000;
                    break 'f Outcome::Changed;
                }
                _ => {
                    break 'f Outcome::NotUsed;
                }
            },
            _ => Outcome::NotUsed,
        }
    };

    let r1 = noselection::handle_events(&mut state.table, true, event);
    Ok(r0 | r1)
}
