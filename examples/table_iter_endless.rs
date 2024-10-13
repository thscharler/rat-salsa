//!
//! Render an iterator that counts to u128::MAX, and
//! then still doesn't stop.
//!
//! Table stops displaying things at usize::MAX + some.
//!

#![allow(dead_code)]

use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
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

mod data;
mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};
    let mut state = State {
        table: Default::default(),
    };

    run_ui(
        "iter_endless",
        handle_table,
        repaint_table,
        &mut data,
        &mut state,
    )
}

struct Data {}

struct State {
    pub(crate) table: TableState<RowSelection>,
}

fn repaint_table(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l0 = Layout::horizontal([Constraint::Percentage(61)])
        .flex(Flex::Center)
        .split(area);

    struct Count(u128);

    impl Iterator for Count {
        type Item = u128;

        fn next(&mut self) -> Option<Self::Item> {
            self.nth(0)
        }

        // implementing nth is essential here otherwise rendering once
        // every 60 years is a bit too long.
        fn nth(&mut self, n: usize) -> Option<Self::Item> {
            self.0 = self.0.saturating_add(n as u128).saturating_add(1);
            Some(self.0)
        }
    }

    struct DataIter {
        iter: Count,
        item: u128,
    }

    impl<'a> TableDataIter<'a> for DataIter {
        fn rows(&self) -> Option<usize> {
            // unknown number of rows
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

        fn render_cell(&self, _ctx: &TableContext, column: usize, area: Rect, buf: &mut Buffer) {
            match column {
                0 => {
                    Span::from(self.item.to_string()).render(area, buf);
                }
                _ => {}
            };
        }
    }

    Table::default()
        .iter(DataIter {
            iter: Count(0),
            item: 0,
        })
        .no_row_count(true) // don't try to count the nr of rows.
        .widths([Constraint::Length(21)])
        .column_spacing(1)
        .header(Row::new([Cell::from("Nr")]).style(Some(THEME.table_header())))
        .footer(Row::new(["..."]).style(Some(THEME.table_footer())))
        .block(
            Block::bordered()
                .border_type(block::BorderType::Rounded)
                .border_style(THEME.block())
                .title("huge-iterator"),
        )
        .vscroll(Scroll::new().style(THEME.block()))
        .flex(Flex::Center)
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
