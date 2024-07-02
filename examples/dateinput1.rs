use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use rat_event::flow_ok;
use rat_widget::date_input;
use rat_widget::date_input::{DateInput, DateInputState};
use rat_widget::event::Outcome;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Span;
use ratatui::Frame;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        input: DateInputState::new("%x")?,
    };

    run_ui(handle_input, repaint_input, &mut data, &mut state)
}

struct Data {}

struct State {
    pub(crate) input: DateInputState,
}

fn repaint_input(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l0 = Layout::horizontal([
        Constraint::Length(14),
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

    let input1 = DateInput::default().style(Style::default().black().on_green());
    frame.render_stateful_widget(input1, l1[1], &mut state.input);
    if let Some((x, y)) = state.input.screen_cursor() {
        frame.set_cursor(x, y);
    }

    let txt1 = Span::from(format!("{:?}", state.input.value()));
    frame.render_widget(txt1, l1[2]);

    Ok(())
}

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    flow_ok!(date_input::handle_events(&mut state.input, true, event));
    Ok(Outcome::NotUsed)
}
