//!
//! Render rows with row-height 0.
//!

use crate::data::SMALL_DATA;
use crate::data::render_tablestate::render_tablestate_row;
use crate::mini_salsa::mock_init;
use crate::mini_salsa::{MiniSalsaState, run_ui, setup_logging};
use format_num_pattern::NumberFormat;
use rat_ftable::event::Outcome;
use rat_ftable::selection::{RowSelection, rowselection};
use rat_ftable::textdata::{Cell, Row};
use rat_ftable::{Table, TableContext, TableDataIter, TableState};
use rat_scrolled::{Scroll, ScrollStyle};
use rat_theme4::StyleName;
use rat_theme4::theme::SalsaTheme;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{Block, StatefulWidget, Widget, block};
use std::cell::RefCell;
use std::iter::Enumerate;
use std::slice::Iter;

mod data;
mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State {
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
        table: Default::default(),
    };

    run_ui("0_height", mock_init, event, render, &mut state)
}

struct Sample {
    pub(crate) text: &'static str,
    pub(crate) num1: f32,
    pub(crate) num2: f32,
    pub(crate) check: bool,
    pub(crate) row_height: u16,
}

struct State {
    table_data: Vec<Sample>,
    table: TableState<RowSelection>,
}

fn render(
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
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
            rows: state.table_data.len(),
            iter: state.table_data.iter().enumerate(),
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
                .title("0-height rows"),
        )
        .vscroll(Scroll::new())
        .flex(Flex::End)
        .styles(table(&ctx.theme))
        .render(l0[0], buf, &mut state.table);

    render_tablestate_row(&state.table, l0[1], buf);

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
    let r = rowselection::handle_events(&mut state.table, true, event);
    Ok(r.into())
}
