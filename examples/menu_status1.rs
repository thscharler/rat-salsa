use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use anyhow::anyhow;
use rat_widget::event::Outcome;
use rat_widget::menuline;
use rat_widget::menuline::{MenuLine, MenuLineState, MenuOutcome};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::Frame;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        menu: Default::default(),
    };

    run_ui(handle_input, repaint_input, &mut data, &mut state)
}

struct Data {}

struct State {
    pub(crate) menu: MenuLineState,
}

fn repaint_input(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

    let menu1 = MenuLine::new()
        .title("Sample")
        .add_str("Choose1")
        .add_str("Choose2")
        .add_str("Choose3")
        .add_str("Message")
        .add_str("_Quit")
        .title_style(Style::default().black().on_yellow())
        .style(Style::default().black().on_dark_gray());
    frame.render_stateful_widget(menu1, l1[1], &mut state.menu);

    Ok(())
}

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let r = menuline::handle_events(&mut state.menu, true, event);
    match r {
        MenuOutcome::Selected(v) => {
            istate.status.status(0, format!("Selected {}", v));
        }
        MenuOutcome::Activated(v) => {
            istate.status.status(0, format!("Activated {}", v));
            match v {
                3 => {
                    istate.msg.append("Hello world!");
                    istate.msg.active = true;
                    return Ok(Outcome::Changed);
                }
                4 => return Err(anyhow!("Quit")),
                _ => {}
            }
        }
        _ => {}
    };

    Ok(r.into())
}
