//!
//! Cell selection
//!

use crate::mini_salsa::mock_init;
use crate::mini_salsa::{MiniSalsaState, run_ui, setup_logging};
use format_num_pattern::NumberFormat;
use rat_ftable::event::Outcome;
use rat_ftable::selection::{CellSelection, cellselection};
use rat_ftable::textdata::{Cell, Row};
use rat_ftable::{Table, TableContext, TableData, TableState};
use rat_scrolled::{Scroll, ScrollStyle};
use rat_theme4::theme::SalsaTheme;
use rat_theme4::{RatWidgetColor, StyleName};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, StatefulWidget, Widget, block};

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
            .collect(),
        table: Default::default(),
    };

    run_ui("cell_selection", mock_init, event, render, &mut state)
}

struct Sample {
    pub(crate) text: &'static str,
    pub(crate) num1: f32,
    pub(crate) num2: f32,
    pub(crate) check: bool,
}

struct State {
    table_data: Vec<Sample>,
    table: TableState<CellSelection>,
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
        .data(DataSlice(&state.table_data))
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
                .title("cell-selection"),
        )
        .vscroll(Scroll::new())
        .flex(Flex::End)
        .styles(table(&ctx.theme))
        .select_row_style(Some(ctx.theme.p.primary(1)))
        .select_column_style(Some(ctx.theme.p.style_alias(Color::DOCUMENT_BASE_BG)))
        .select_cell_style(Some(ctx.theme.p.primary(3)))
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

fn event(
    event: &crossterm::event::Event,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let r = cellselection::handle_events(&mut state.table, true, event);
    Ok(r.into())
}
