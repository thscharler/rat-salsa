//!
//! Example for [TableDataIter] used with [StatefulWidget]
//!

use crate::mini_salsa::mock_init;
use crate::mini_salsa::{MiniSalsaState, run_ui, setup_logging};
use format_num_pattern::NumberFormat;
use rat_event::{HandleEvent, Regular};
use rat_focus::{Focus, FocusBuilder, FocusFlag};
use rat_ftable::event::Outcome;
use rat_ftable::selection::RowSelection;
use rat_ftable::textdata::{Cell, Row};
use rat_ftable::{Table, TableContext, TableDataIter, TableState};
use rat_scrolled::{Scroll, ScrollStyle};
use rat_theme4::StyleName;
use rat_theme4::theme::SalsaTheme;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Flex, Layout, Rect};
use ratatui_core::style::Style;
use ratatui_core::text::Span;
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::Block;
use ratatui_widgets::borders::BorderType;
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
            })
            .take(100_000)
            .collect(),
        table: Default::default(),
    };

    run_ui("iter", mock_init, event, render, &mut state)
}

struct Sample {
    text: &'static str,
    num1: f32,
    num2: f32,
    check: bool,
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
    let l0 = Layout::horizontal([Constraint::Percentage(61)])
        .flex(Flex::Center)
        .split(area);

    struct DataIter<'a> {
        size: usize,
        iter: Enumerate<Iter<'a, Sample>>,
        item: Option<(usize, &'a Sample)>,
    }

    impl<'a> TableDataIter<'a> for DataIter<'a> {
        fn rows(&self) -> Option<usize> {
            Some(self.size)
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
            size: state.table_data.len(),
            iter: state.table_data.iter().enumerate(),
            item: None,
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
                .border_type(BorderType::Rounded)
                .title("tabledata-iter (no row-count)"),
        )
        .vscroll(Scroll::new())
        .styles(table(&ctx.theme))
        .render(l0[0], buf, &mut state.table);
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

fn focus(state: &mut State) -> Focus {
    let mut fb = FocusBuilder::new(None);
    fb.widget(&state.table);
    fb.widget(&FocusFlag::new());
    fb.build()
}

fn event(
    event: &Event,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    ctx.focus_outcome = focus(state).handle(event, Regular);

    let r = state.table.handle(event, Regular);

    Ok(r.into())
}
