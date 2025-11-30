#![allow(dead_code)]

use crate::adapter::list::{ListS, ListSState};
use crate::mini_salsa::{MiniSalsaState, THEME, mock_init, run_ui, setup_logging};
use rat_event::{HandleEvent, MouseOnly, Outcome, try_flow};
use rat_scrolled::Scroll;
use ratatui_core::layout::{Constraint, Layout, Rect};
use std::iter::repeat_with;
use ratatui_core::terminal::Frame;
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::list::ListDirection;

mod adapter;
mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut counter = 0;

    let mut data = Data {
        sample1: repeat_with(|| {
            counter += 1;
            counter
        })
        .take(2000)
        .collect::<Vec<i32>>(),
        sample2:
            "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et"
                .split(" ").collect()
        ,
    };

    let mut state = State {
        list1: Default::default(),
        list2: Default::default(),
        list3: Default::default(),
        list4: Default::default(),
    };

    run_ui("slist", mock_init, event, render, &mut data, &mut state)
}

struct Data {
    pub(crate) sample1: Vec<i32>,
    pub(crate) sample2: Vec<&'static str>,
}

struct State {
    pub(crate) list1: ListSState,
    pub(crate) list2: ListSState,
    pub(crate) list3: ListSState,
    pub(crate) list4: ListSState,
}

fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Fill(1),
    ])
    .split(area);

    ListS::new(data.sample1.iter().map(|v| v.to_string()))
        .scroll(Scroll::new().style(THEME.block()))
        .highlight_style(THEME.secondary(2))
        .render(l[0], frame.buffer_mut(), &mut state.list1);

    ListS::new(data.sample2.iter().map(|v| v.to_string()))
        .scroll(Scroll::new().style(THEME.block()))
        .highlight_style(THEME.secondary(2))
        .render(l[1], frame.buffer_mut(), &mut state.list2);

    ListS::new(data.sample1.iter().map(|v| v.to_string()))
        .scroll(Scroll::new().style(THEME.block()))
        .scroll_selection()
        .scroll_padding(2)
        .highlight_symbol("&")
        .highlight_style(THEME.secondary(2))
        .render(l[2], frame.buffer_mut(), &mut state.list3);

    ListS::new(data.sample2.iter().map(|v| v.to_string()))
        .scroll(Scroll::new().style(THEME.block()))
        .highlight_style(THEME.secondary(2))
        .direction(ListDirection::BottomToTop)
        .render(l[3], frame.buffer_mut(), &mut state.list4);

    Ok(())
}

fn event(
    event: &Event,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    try_flow!(state.list1.handle(event, MouseOnly));
    try_flow!(state.list2.handle(event, MouseOnly));
    try_flow!(state.list3.handle(event, MouseOnly));
    try_flow!(state.list4.handle(event, MouseOnly));
    Ok(Outcome::Continue)
}
