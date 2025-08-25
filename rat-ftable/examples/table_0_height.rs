//!
//! Render rows with row-height 0.
//!

use crate::data::render_tablestate::render_tablestate_row;
use crate::data::SMALL_DATA;
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
use std::cell::RefCell;
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
                row_height: rand::random::<u16>() % 3,
            })
            .take(100_000)
            .collect(),
    };

    let mut state = State {
        table: Default::default(),
    };

    run_ui(
        "0_height",
        |_| {},
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
    pub(crate) row_height: u16,
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
    let l0 = Layout::horizontal([Constraint::Percentage(61), Constraint::Percentage(39)])
        .flex(Flex::Center)
        .split(area);

    struct DataIter<'a> {
        rows: usize,
        iter: Enumerate<Iter<'a, Sample>>,
        item: Option<(usize, &'a Sample)>,

        fmt1: NumberFormat,
        fmt2: NumberFormat,
        txt: RefCell<String>,
    }

    impl<'a> TableDataIter<'a> for DataIter<'a> {
        fn rows(&self) -> Option<usize> {
            Some(self.rows)
        }

        fn nth(&mut self, n: usize) -> bool {
            self.item = self.iter.nth(n);
            self.item.is_some()
        }

        fn row_height(&self) -> u16 {
            self.item.map(|v| v.1.row_height).unwrap_or_default()
        }

        fn render_cell(&self, ctx: &TableContext, column: usize, area: Rect, buf: &mut Buffer) {
            let row = self.item.expect("data");
            match column {
                0 => {
                    let span = Span::from(self.fmt1.fmt_u(row.0));
                    span.render(area, buf);
                }
                1 => {
                    let span = Span::from(row.1.text);
                    span.render(area, buf);
                }
                2 => {
                    let span = Span::from(self.fmt2.fmt_u(row.1.num1));
                    span.render(area, buf);
                }
                3 => {
                    let span = Span::from(self.fmt2.fmt_u(row.1.num2));
                    span.render(area, buf);
                }
                4 => {
                    let cc = if row.1.check { "\u{2622}" } else { "\u{2623}" };
                    let span = Span::from(cc);
                    span.render(area, buf);
                }
                _ => {}
            }

            let mut extra = ctx.row_area;
            if extra.height > 1 {
                extra.x = 7;
                extra.y = 1;
                for r in 1..extra.height {
                    let mut txt = self.txt.borrow_mut();
                    txt.clear();

                    for w in 0..7 {
                        txt.push_str(SMALL_DATA[(r * 7 + w) as usize]);
                        txt.push(' ');
                    }
                    txt.as_str().render(extra, buf);

                    extra.y += 1;
                }
            }
        }
    }

    Table::default()
        .iter(DataIter {
            rows: data.table_data.len(),
            iter: data.table_data.iter().enumerate(),
            item: None,
            fmt1: NumberFormat::new("000000").expect("fmt"),
            fmt2: NumberFormat::new("####0.00").expect("fmt"),
            txt: Default::default(),
        })
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
                .title("0-height rows"),
        )
        .vscroll(Scroll::new().style(THEME.block()))
        .flex(Flex::End)
        .styles(THEME.table_style())
        .select_row_style(Some(THEME.gray(3)))
        .render(l0[0], frame.buffer_mut(), &mut state.table);

    render_tablestate_row(&state.table, l0[1], frame.buffer_mut());

    Ok(())
}

fn handle_table(
    event: &crossterm::event::Event,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let r = rowselection::handle_events(&mut state.table, true, event);
    Ok(r.into())
}
