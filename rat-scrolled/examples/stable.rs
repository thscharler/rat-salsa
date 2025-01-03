#![allow(dead_code)]

use crate::adapter::table::{TableS, TableSState};
use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use rat_event::{HandleEvent, MouseOnly, Outcome};
use rat_scrolled::Scroll;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Text;
use ratatui::widgets::{Block, Cell, Row, StatefulWidget};
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
    };

    let mut state = State {
        table1: Default::default(),
        table2: Default::default(),
    };

    run_ui("stable", handle_lists, repaint_lists, &mut data, &mut state)
}

struct Data {
    pub(crate) sample1: Vec<i32>,
}

struct State {
    pub(crate) table1: TableSState,
    pub(crate) table2: TableSState,
}

fn repaint_lists(
    frame: &mut Frame<'_>,
    area: Rect,
    data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)])
        .spacing(1)
        .split(area);

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
    .highlight_style(THEME.primary(2))
    .scroll_selection()
    .block(Block::bordered().style(THEME.block()))
    .scroll(Scroll::new().style(THEME.block()))
    .style(THEME.table())
    .render(l[0], frame.buffer_mut(), &mut state.table1);

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
    .highlight_style(THEME.primary(2))
    .block(Block::bordered().style(THEME.block()))
    .scroll(Scroll::new().style(THEME.block()))
    .style(THEME.table())
    .render(l[1], frame.buffer_mut(), &mut state.table2);

    Ok(())
}

fn handle_lists(
    event: &crossterm::event::Event,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    match Outcome::from(state.table1.handle(event, MouseOnly)) {
        Outcome::Continue => {}
        r => return Ok(r),
    };
    match Outcome::from(state.table2.handle(event, MouseOnly)) {
        Outcome::Continue => {}
        r => return Ok(r),
    };

    Ok(Outcome::Continue)
}
