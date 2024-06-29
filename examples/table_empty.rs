use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use rat_ftable::event::Outcome;
use rat_ftable::selection::{noselection, NoSelection};
use rat_ftable::{FTable, FTableState};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::Frame;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        table: Default::default(),
    };

    run_ui(handle_table, repaint_table, &mut data, &mut state)
}

struct Data {}

struct State {
    pub(crate) table: FTableState<NoSelection>,
}

fn repaint_table(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l0 = Layout::horizontal([
        Constraint::Length(10),
        Constraint::Fill(1),
        Constraint::Length(10),
    ])
    .split(area);

    let table1 = FTable::default().style(Style::default().bg(Color::Rgb(25, 25, 25)));
    frame.render_stateful_widget(table1, l0[1], &mut state.table);

    Ok(())
}

fn handle_table(
    event: &crossterm::event::Event,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let r = noselection::handle_events(&mut state.table, true, event);
    Ok(r)
}
