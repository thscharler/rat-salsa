//!
//! Single row selection.
//!

use crate::mini_salsa::{MiniSalsaState, run_ui, setup_logging};
use crate::mini_salsa::{THEME, mock_init};
use format_num_pattern::NumberFormat;
use rat_ftable::event::Outcome;
use rat_ftable::selection::{RowSelection, rowselection};
use rat_ftable::textdata::{Cell, Row};
use rat_ftable::{Table, TableContext, TableData, TableState};
use rat_scrolled::Scroll;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Flex, Layout, Rect};
use ratatui_core::terminal::Frame;
use ratatui_core::text::Span;
use ratatui_core::widgets::{ StatefulWidget, Widget,};
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::Block;
use ratatui_widgets::borders::BorderType;

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
    };

    run_ui(
        "row_selection",
        mock_init,
        event,
        render,
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
}

fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l0 = Layout::horizontal([Constraint::Percentage(61)])
        .flex(Flex::Center)
        .split(area);

    struct DataSlice<'a>(&'a [Sample]);

    impl<'a> TableData<'a> for DataSlice<'a> {
        fn rows(&self) -> usize {
            self.0.len()
        }

        fn render_cell(
            &self,
            _ctx: &TableContext,
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

    Table::default()
        .data(DataSlice(&data.table_data))
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
                .border_type(BorderType::Rounded)
                .border_style(THEME.container_border())
                .title("row-selection"),
        )
        .vscroll(Scroll::new().style(THEME.container_border()))
        .flex(Flex::End)
        .styles(THEME.table_style())
        .select_row_style(Some(THEME.gray(3)))
        .render(l0[0], frame.buffer_mut(), &mut state.table);
    Ok(())
}

fn event(
    event: &Event,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let r = rowselection::handle_events(&mut state.table, true, event);
    Ok(r.into())
}
