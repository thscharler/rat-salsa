use crate::mini_salsa::mock_init;
use crate::mini_salsa::{MiniSalsaState, run_ui, setup_logging};
use format_num_pattern::NumberFormat;
use rat_ftable::event::Outcome;
use rat_ftable::selection::{RowSelection, rowselection};
use rat_ftable::textdata::{Cell, Row};
use rat_ftable::{Table, TableState};
use rat_scrolled::{Scroll, ScrollStyle};
use rat_theme4::StyleName;
use rat_theme4::theme::SalsaTheme;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Flex, Layout, Rect};
use ratatui_core::style::Style;
use ratatui_core::text::Text;
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::Block;
use ratatui_widgets::borders::BorderType;

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

    run_ui("text", mock_init, event, render, &mut state)
}

struct Sample {
    pub(crate) text: &'static str,
    pub(crate) num1: f32,
    pub(crate) num2: f32,
    pub(crate) check: bool,
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

    let row_fmt = NumberFormat::new("000000").expect("fmt");
    let num1_fmt = NumberFormat::new("####0.00").expect("fmt");
    let num2_fmt = NumberFormat::new("####0.00").expect("fmt");

    Table::default()
        .rows(
            state
                .table_data
                .iter()
                .take(3000)
                .enumerate()
                .map(|(i, v)| {
                    Row::new([
                        Text::from(row_fmt.fmt_u(i)),
                        v.text.into(),
                        num1_fmt.fmt_u(v.num1).into(),
                        num2_fmt.fmt_u(v.num2).into(),
                        v.check.to_string().into(),
                    ])
                }),
        )
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
                .title("text-tabledata"),
        )
        .vscroll(Scroll::new())
        .flex(Flex::End)
        .styles(table(&ctx.theme))
        .select_row_style(Some(ctx.theme.p.gray(3)))
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
    event: &Event,
    _ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let r = rowselection::handle_events(&mut state.table, true, event);
    Ok(r.into())
}
