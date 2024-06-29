#![allow(dead_code)]
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use rat_ftable::event::Outcome;
use rat_ftable::selection::{noselection, NoSelection};
use rat_ftable::textdata::{Cell, Row};
use rat_ftable::{FTable, FTableContext, FTableState, TableDataIter};
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
}

fn repaint_table(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l0 = Layout::horizontal([
        Constraint::Length(10),
        Constraint::Fill(1),
        Constraint::Length(10),
    ])
    .split(area);

    struct Count(u32);
    impl Iterator for Count {
        type Item = u32;

        fn next(&mut self) -> Option<Self::Item> {
            if self.0 > 200_000 {
                None
            } else {
                self.0 += 1;
                Some(self.0)
            }
        }
    }

    struct RowIter {
        iter: Count,
        item: u32,
    }

    impl<'a> TableDataIter<'a> for RowIter {
        fn rows(&self) -> Option<usize> {
            None
        }

        fn nth(&mut self, n: usize) -> bool {
            if let Some(v) = self.iter.nth(n) {
                self.item = v;
                true
            } else {
                false
            }
        }

        fn render_cell(&self, _ctx: &FTableContext, _column: usize, area: Rect, buf: &mut Buffer) {
            Span::from(self.item.to_string()).render(area, buf);
        }
    }

    let table1 = FTable::default()
        .iter(RowIter {
            iter: Count(0),
            item: 0,
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
            .style(Some(Style::new().black().bg(Color::from_u32(0x98c379)))),
        )
        .footer(
            Row::new(["a", "b", "c", "d", "e"])
                .style(Some(Style::new().black().bg(Color::from_u32(0x98c379)))),
        )
        .flex(Flex::End)
        .style(Style::default().bg(Color::Rgb(25, 25, 25)));
    frame.render_stateful_widget(table1, l0[1], &mut state.table);
    Ok(())
}

fn handle_table(
    event: &crossterm::event::Event,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let r = noselection::handle_events(&mut state.table, true, event);
    Ok(r)
}
