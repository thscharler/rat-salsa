#![allow(dead_code)]

use crate::adapter::table::{TableS, TableSState};
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use rat_event::{HandleEvent, MouseOnly, Outcome};
use rat_scrolled::{Scrolled, ScrolledState};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::{Cell, Row, StatefulWidget};
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
        table1: Default::default(),
        table2: Default::default(),
    };

    run_ui(handle_lists, repaint_lists, &mut data, &mut state)
}

struct Data {
    pub(crate) sample1: Vec<i32>,
    pub(crate) sample2: Vec<&'static str>,
}

struct State {
    pub(crate) table1: ScrolledState<TableSState>,
    pub(crate) table2: ScrolledState<TableSState>,
}

fn repaint_lists(
    frame: &mut Frame<'_>,
    area: Rect,
    data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).split(area);

    let table1 = Scrolled::new(
        TableS::new(
            data.sample1.iter().map(|v| {
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
        .highlight_style(Style::default().on_red())
        .scroll_selection()
        .scroll_by(1),
    );
    table1.render(l[0], frame.buffer_mut(), &mut state.table1);

    let table2 = Scrolled::new(TableS::new(
        data.sample2.iter().map(|v| {
            Row::new([
                Cell::new(Text::from(*v)),
                Cell::new(Text::from(*v)),
                Cell::new(Text::from(*v)),
                Cell::new(Text::from(*v)),
                Cell::new(Text::from(*v)),
            ])
        }),
        [
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
        ],
    ));
    table2.render(l[1], frame.buffer_mut(), &mut state.table2);

    Ok(())
}

fn handle_lists(
    event: &crossterm::event::Event,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    match Outcome::from(state.table1.handle(event, MouseOnly)) {
        Outcome::NotUsed => {}
        r => return Ok(r),
    };
    match Outcome::from(state.table2.handle(event, MouseOnly)) {
        Outcome::NotUsed => {}
        r => return Ok(r),
    };

    Ok(Outcome::NotUsed)
}
