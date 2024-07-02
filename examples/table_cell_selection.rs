use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use format_num_pattern::NumberFormat;
use rat_ftable::event::Outcome;
use rat_ftable::selection::{cellselection, CellSelection};
use rat_ftable::textdata::{Cell, Row};
use rat_ftable::{FTable, FTableContext, FTableState, TableData};
use rat_widget::statusline::StatusLineState;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::prelude::Widget;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::Span;
use ratatui::Frame;

mod data;
mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {
        table_data: data::DATA
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
        status: Default::default(),
    };
    state.status.status(0, "Ctrl+Q to quit.");

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
    pub(crate) table: FTableState<CellSelection>,
    pub(crate) status: StatusLineState,
}

fn repaint_table(
    frame: &mut Frame<'_>,
    area: Rect,
    data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l0 = Layout::horizontal([
        Constraint::Length(30),
        Constraint::Fill(1),
        Constraint::Length(30),
    ])
    .split(area);

    struct Data1<'a>(&'a [Sample]);

    impl<'a> TableData<'a> for Data1<'a> {
        fn rows(&self) -> usize {
            self.0.len()
        }

        fn render_cell(
            &self,
            _ctx: &FTableContext,
            column: usize,
            row: usize,
            area: Rect,
            buf: &mut Buffer,
        ) {
            if let Some(d) = self.0.get(row) {
                match column {
                    0 => {
                        let row_fmt = NumberFormat::new("000000").expect("fmt");
                        let span = Span::from(row_fmt.fmt_u(row));
                        span.render(area, buf);
                    }
                    1 => {
                        let span = Span::from(d.text);
                        span.render(area, buf);
                    }
                    2 => {
                        let num1_fmt = NumberFormat::new("####0.00").expect("fmt");
                        let span = Span::from(num1_fmt.fmt_u(d.num1));
                        span.render(area, buf);
                    }
                    3 => {
                        let num2_fmt = NumberFormat::new("####0.00").expect("fmt");
                        let span = Span::from(num2_fmt.fmt_u(d.num2));
                        span.render(area, buf);
                    }
                    4 => {
                        let cc = if d.check { "\u{2622}" } else { "\u{2623}" };
                        let span = Span::from(cc);
                        span.render(area, buf);
                    }
                    _ => {}
                }
            }
        }
    }

    let table1 = FTable::default()
        .data(Data1(&data.table_data))
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
            .style(Some(Style::new().black().bg(Color::Rgb(152, 195, 121)))),
        )
        .footer(
            Row::new(["a", "b", "c", "d", "e"])
                .style(Some(Style::new().black().bg(Color::Rgb(152, 195, 121)))),
        )
        .flex(Flex::End)
        .style(Style::default().bg(Color::Rgb(25, 25, 25)))
        .select_row_style(Some(Style::default().bg(Color::Rgb(50, 50, 50))))
        .select_column_style(Some(Style::default().bg(Color::Rgb(30, 30, 30))))
        .select_cell_style(Some(Style::default().black().bg(Color::Rgb(128, 128, 128))))
        .select_header_style(Some(Style::default().bg(Color::Rgb(172, 215, 141))))
        .select_footer_style(Some(Style::default().bg(Color::Rgb(172, 215, 141))));
    frame.render_stateful_widget(table1, l0[1], &mut state.table);
    Ok(())
}

fn handle_table(
    event: &crossterm::event::Event,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let r = cellselection::handle_events(&mut state.table, true, event);
    Ok(r)
}
