#![allow(dead_code)]

use crate::adapter::table::{TableS, TableSState};
use crate::mini_salsa::{MiniSalsaState, mock_init, run_ui, setup_logging};
use rat_event::{HandleEvent, MouseOnly, Outcome, try_flow};
use rat_scrolled::Scroll;
use rat_theme4::StyleName;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{Block, Cell, Row, StatefulWidget};
use std::iter::repeat_with;

mod adapter;
mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut counter = 0;

    let mut state = State {
        sample1: repeat_with(|| {
            counter += 1;
            counter
        })
        .take(2000)
        .collect::<Vec<i32>>(),
        table1: Default::default(),
        table2: Default::default(),
    };

    run_ui("stable", mock_init, event, render, &mut state)
}

struct State {
    sample1: Vec<i32>,
    table1: TableSState,
    table2: TableSState,
}

fn render(
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)])
        .spacing(1)
        .split(area);

    TableS::new(
        state.sample1.iter().map(|v| {
            Row::new([
                Cell::new(Text::from(v.to_string())),
                Cell::new(Text::from(v.to_string())),
                Cell::new(Text::from(v.to_string())),
                Cell::new(Text::from(v.to_string())),
                Cell::new(Text::from(v.to_string())),
                Cell::new(Text::from(v.to_string())),
                Cell::new(Text::from(v.to_string())),
                Cell::new(Text::from(v.to_string())),
                Cell::new(Text::from(v.to_string())),
            ])
        }),
        [
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
        ],
    )
    .highlight_style(ctx.theme.p.primary(2))
    .scroll_selection()
    .block(Block::bordered().style(ctx.theme.style_style(Style::CONTAINER_BASE)))
    .scroll(Scroll::new().style(ctx.theme.style_style(Style::CONTAINER_BORDER_FG)))
    .style(ctx.theme.style_style(Style::DOCUMENT_BASE))
    .render(l[0], buf, &mut state.table1);

    TableS::new(
        state.sample1.iter().map(|v| {
            Row::new([
                Cell::new(Text::from(v.to_string())),
                Cell::new(Text::from(v.to_string())),
                Cell::new(Text::from(v.to_string())),
                Cell::new(Text::from(v.to_string())),
                Cell::new(Text::from(v.to_string())),
                Cell::new(Text::from(v.to_string())),
                Cell::new(Text::from(v.to_string())),
                Cell::new(Text::from(v.to_string())),
                Cell::new(Text::from(v.to_string())),
            ])
        }),
        [
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
        ],
    )
    .highlight_style(ctx.theme.p.primary(2))
    .block(Block::bordered().style(ctx.theme.style_style(Style::CONTAINER_BASE)))
    .scroll(Scroll::new().style(ctx.theme.style_style(Style::CONTAINER_BORDER_FG)))
    .style(ctx.theme.style_style(Style::DOCUMENT_BASE))
    .render(l[1], buf, &mut state.table2);

    Ok(())
}

fn event(
    event: &crossterm::event::Event,
    _ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    try_flow!(Outcome::from(state.table1.handle(event, MouseOnly)));
    try_flow!(Outcome::from(state.table2.handle(event, MouseOnly)));
    Ok(Outcome::Continue)
}
