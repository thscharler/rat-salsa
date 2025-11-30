use crate::mini_salsa::{MiniSalsaState, mock_init, run_ui, setup_logging};
use rat_event::try_flow;
use rat_text::HasScreenCursor;
use rat_widget::date_input;
use rat_widget::date_input::{DateInput, DateInputState};
use rat_widget::event::Outcome;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::terminal::Frame;
use ratatui_core::text::Span;
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::Event;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        input: DateInputState::new().with_pattern("%x")?,
    };

    run_ui(
        "dateinput1",
        mock_init,
        event,
        render,
        &mut data,
        &mut state,
    )
}

struct Data {}

struct State {
    pub(crate) input: DateInputState,
}

fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l0 = Layout::horizontal([
        Constraint::Length(25),
        Constraint::Length(20),
        Constraint::Fill(1),
        Constraint::Fill(1),
    ])
    .split(area);

    let l1 = Layout::vertical([
        Constraint::Length(7),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .split(l0[0]);

    let l2 = Layout::vertical([
        Constraint::Length(7),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .split(l0[1]);

    DateInput::new() //
        .styles(istate.theme.text_style())
        .render(l1[1], frame.buffer_mut(), &mut state.input);
    if let Some((x, y)) = state.input.screen_cursor() {
        frame.set_cursor_position((x, y));
    }

    Span::from(format!("{:?}", state.input.value())) //
        .render(l2[1], frame.buffer_mut());

    Ok(())
}

fn event(
    event: &Event,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    try_flow!(date_input::handle_events(&mut state.input, true, event));

    Ok(Outcome::Continue)
}
