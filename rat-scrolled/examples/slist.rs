#![allow(dead_code)]

use crate::adapter::list::{ListS, ListSState};
use crate::mini_salsa::{MiniSalsaState, mock_init, run_ui, setup_logging};
use rat_event::{HandleEvent, MouseOnly, Outcome, try_flow};
use rat_scrolled::Scroll;
use rat_theme4::StyleName;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, ListDirection, StatefulWidget};
use std::iter::repeat_with;

mod adapter;
mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut counter = 0;

    let mut state = State { sample1: repeat_with(|| {
        counter += 1;
        counter
    })
        .take(2000)
        .collect::<Vec<i32>>(),
        sample2:
        "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et"
            .split(" ").collect()
        ,
        list1: Default::default(),
        list2: Default::default(),
        list3: Default::default(),
        list4: Default::default(),
    };

    run_ui("slist", mock_init, event, render, &mut state)
}

struct State {
    sample1: Vec<i32>,
    sample2: Vec<&'static str>,
    list1: ListSState,
    list2: ListSState,
    list3: ListSState,
    list4: ListSState,
}

fn render(
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Fill(1),
    ])
    .split(area);

    ListS::new(state.sample1.iter().map(|v| v.to_string()))
        .block(Block::new().style(ctx.theme.style_style(Style::CONTAINER_BASE)))
        .scroll(Scroll::new().style(ctx.theme.style_style(Style::CONTAINER_BORDER_FG)))
        .highlight_style(ctx.theme.p.secondary(2))
        .render(l[0], buf, &mut state.list1);

    ListS::new(state.sample2.iter().map(|v| v.to_string()))
        .block(Block::new().style(ctx.theme.style_style(Style::CONTAINER_BASE)))
        .scroll(Scroll::new().style(ctx.theme.style_style(Style::CONTAINER_BORDER_FG)))
        .highlight_style(ctx.theme.p.secondary(2))
        .render(l[1], buf, &mut state.list2);

    ListS::new(state.sample1.iter().map(|v| v.to_string()))
        .block(Block::new().style(ctx.theme.style_style(Style::CONTAINER_BASE)))
        .scroll(Scroll::new().style(ctx.theme.style_style(Style::CONTAINER_BORDER_FG)))
        .highlight_style(ctx.theme.p.secondary(2))
        .scroll_selection()
        .scroll_padding(2)
        .highlight_symbol("&")
        .render(l[2], buf, &mut state.list3);

    ListS::new(state.sample2.iter().map(|v| v.to_string()))
        .block(Block::new().style(ctx.theme.style_style(Style::CONTAINER_BASE)))
        .scroll(Scroll::new().style(ctx.theme.style_style(Style::CONTAINER_BORDER_FG)))
        .highlight_style(ctx.theme.p.secondary(2))
        .direction(ListDirection::BottomToTop)
        .render(l[3], buf, &mut state.list4);

    Ok(())
}

fn event(
    event: &crossterm::event::Event,
    _ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    try_flow!(state.list1.handle(event, MouseOnly));
    try_flow!(state.list2.handle(event, MouseOnly));
    try_flow!(state.list3.handle(event, MouseOnly));
    try_flow!(state.list4.handle(event, MouseOnly));
    Ok(Outcome::Continue)
}
