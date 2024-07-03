#![allow(dead_code)]

use crate::adapter::list::{ListS, ListSState};
use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use rat_event::{HandleEvent, MouseOnly, Outcome};
use rat_scrolled::Scroll;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{ListDirection, StatefulWidget};
use ratatui::Frame;
use std::iter::repeat_with;

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
        sample2: vec![
            "Lorem",
            "ipsum",
            "dolor",
            "sit",
            "amet,",
            "consetetur",
            "sadipscing",
            "elitr,",
            "sed",
            "diam",
            "nonumy",
            "eirmod",
            "tempor",
            "invidunt",
            "ut",
            "labore",
            "et",
            "dolore",
            "magna",
            "aliquyam",
            "erat,",
            "sed",
            "diam",
            "voluptua.",
            "At",
            "vero",
            "eos",
            "et",
            "accusam",
            "et",
        ],
    };

    let mut state = State {
        list1: Default::default(),
        list2: Default::default(),
        list3: Default::default(),
        list4: Default::default(),
    };

    run_ui(handle_lists, repaint_lists, &mut data, &mut state)
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

fn repaint_lists(
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
        .highlight_style(THEME.secondary(2))
        .scroll(Scroll::new().style(THEME.block()))
        .render(l[0], frame.buffer_mut(), &mut state.list1);

    ListS::new(data.sample2.iter().map(|v| v.to_string()))
        .highlight_style(THEME.secondary(2))
        .scroll(Scroll::new().style(THEME.block()))
        .render(l[1], frame.buffer_mut(), &mut state.list2);

    ListS::new(data.sample1.iter().map(|v| v.to_string()))
        .highlight_symbol("&")
        .highlight_style(THEME.secondary(2))
        .scroll(Scroll::new().style(THEME.block()))
        .scroll_selection()
        .scroll_padding(2)
        .render(l[2], frame.buffer_mut(), &mut state.list3);

    ListS::new(data.sample2.iter().map(|v| v.to_string()))
        .highlight_style(THEME.secondary(2))
        .direction(ListDirection::BottomToTop)
        .scroll(Scroll::new().style(THEME.block()))
        .render(l[3], frame.buffer_mut(), &mut state.list4);

    Ok(())
}

fn handle_lists(
    event: &crossterm::event::Event,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    match state.list1.handle(event, MouseOnly).into() {
        Outcome::NotUsed => {}
        r => return Ok(r),
    };
    match state.list2.handle(event, MouseOnly).into() {
        Outcome::NotUsed => {}
        r => return Ok(r),
    };
    match state.list3.handle(event, MouseOnly).into() {
        Outcome::NotUsed => {}
        r => return Ok(r),
    };
    match state.list4.handle(event, MouseOnly).into() {
        Outcome::NotUsed => {}
        r => return Ok(r),
    };

    Ok(Outcome::NotUsed)
}
