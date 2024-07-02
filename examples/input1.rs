use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use rat_event::{flow_ok, Outcome};
use rat_widget::input;
use rat_widget::input::{TextInput, TextInputState};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::Frame;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        input: Default::default(),
    };
    state
        .input
        .set_value("asdf jklö 1234 (()) // öö 👩🏾‍🏫 👮🏾‍♀️ 💂🏾‍♀️ 👷🏾 🧔🏾‍♀️ 👩🏾‍");

    run_ui(handle_input, repaint_input, &mut data, &mut state)
}

struct Data {}

struct State {
    pub(crate) input: TextInputState,
}

fn repaint_input(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l0 = Layout::horizontal([
        Constraint::Length(45),
        Constraint::Fill(1),
        Constraint::Fill(1),
    ])
    .split(area);

    let l1 = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .split(l0[0]);

    let input1 = TextInput::default().style(Style::default().black().on_dark_gray());
    frame.render_stateful_widget(input1, l1[1], &mut state.input);
    if let Some((x, y)) = state.input.screen_cursor() {
        frame.set_cursor(x, y);
    }

    Ok(())
}

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    flow_ok!(input::handle_events(&mut state.input, true, event));
    Ok(Outcome::Unchanged)
}
