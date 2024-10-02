//!
//! Example for [TableDataIter] used with [StatefulWidgetRef]
//!

use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use format_num_pattern::NumberFormat;
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
use std::iter::Enumerate;
use std::slice::Iter;

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
            .take(100_000)
            .collect(),
    };

    let mut state = State {
        table: Default::default(),
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
    pub(crate) table: TableState<RowSelection>,
}

fn repaint_table(
    frame: &mut Frame<'_>,
    area: Rect,
    data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l0 = Layout::horizontal([Constraint::Percentage(61)])
        .flex(Flex::Center)
        .split(area);

    #[derive(Clone)]
    struct DataIter<'a> {
        iter: Enumerate<Iter<'a, Sample>>,
        item: Option<(usize, &'a Sample)>,
    }

    impl<'a> TableDataIter<'a> for DataIter<'a> {
        /// StatefulWidgetRef needs a clone of the iterator for every render.
        /// For StatefulWidget this is not needed at all. So this defaults to
        /// None and warns at runtime.
        fn cloned(&self) -> Option<Box<dyn TableDataIter<'a> + 'a>> {
            let a = self.clone();
            let c: Box<dyn TableDataIter<'a>> = Box::new(a);
            Some(c)
        }

        fn rows(&self) -> Option<usize> {
            None
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
        .iter(DataIter {
            iter: data.table_data.iter().enumerate(),
            item: None,
        })
        .widths([
            Constraint::Length(6),
            Constraint::Length(20),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(13),
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
                .title("tabledata-iter + render_ref"),
        )
        .vscroll(Scroll::new().style(THEME.block()))
        .flex(Flex::End)
        .style(THEME.table())
        .select_row_style(Some(THEME.gray(3)))
        .render(l0[0], frame.buffer_mut(), &mut state.table);
    Ok(())
}

fn handle_table(
    event: &crossterm::event::Event,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let r = rowselection::handle_events(&mut state.table, true, event);
    Ok(r)
}
