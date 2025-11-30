use crate::mini_salsa::{MiniSalsaState, mock_init, run_ui, setup_logging};
use rat_event::{HandleEvent, Regular, try_flow};
use rat_focus::{Focus, FocusBuilder};
use rat_widget::button::{Button, ButtonState};
use rat_widget::event::{ButtonOutcome, Outcome};
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::terminal::Frame;
use ratatui_core::text::Span;
use ratatui_core::widgets::StatefulWidget;
use ratatui_core::widgets::Widget;
use ratatui_crossterm::crossterm::event::Event;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {
        p0: 0,
        p1: 0,
        p2: 0,
    };

    let mut state = State {
        button1: Default::default(),
        button2: Default::default(),
        button3: Default::default(),
    };

    run_ui("button1", mock_init, event, render, &mut data, &mut state)
}

struct Data {
    p0: u32,
    p1: u32,
    p2: u32,
}

struct State {
    button1: ButtonState,
    button2: ButtonState,
    button3: ButtonState,
}

fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l0 = Layout::horizontal([
        Constraint::Length(30),
        Constraint::Fill(1),
        Constraint::Length(30),
    ])
    .split(area);

    let l1 = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(3),
        Constraint::Length(5),
        Constraint::Length(5),
        Constraint::Fill(1),
    ])
    .split(l0[1]);

    Button::new("Button")
        .styles(istate.theme.button_style()) //
        .render(l1[1], frame.buffer_mut(), &mut state.button1);

    Button::new("Button\nnottuB")
        .styles(istate.theme.button_style()) //
        .render(l1[2], frame.buffer_mut(), &mut state.button2);

    Button::new("Button")
        .styles(istate.theme.button_style()) //
        .render(l1[3], frame.buffer_mut(), &mut state.button3);

    let l2 = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(3),
        Constraint::Length(5),
        Constraint::Length(5),
        Constraint::Fill(1),
    ])
    .split(l0[2]);
    Span::from(format!("  {} | {} | {}", data.p0, data.p1, data.p2))
        .render(l2[1], frame.buffer_mut());

    Ok(())
}

fn focus(state: &mut State) -> Focus {
    let mut fb = FocusBuilder::new(None);
    fb.widget(&state.button1);
    fb.widget(&state.button2);
    fb.widget(&state.button3);
    fb.build()
}

fn event(
    event: &Event,
    data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let mut focus = focus(state);
    istate.focus_outcome = focus.handle(event, Regular);

    try_flow!(match state.button1.handle(event, Regular) {
        ButtonOutcome::Pressed => {
            data.p0 += 1;
            Outcome::Changed
        }
        r => r.into(),
    });
    try_flow!(match state.button2.handle(event, Regular) {
        ButtonOutcome::Pressed => {
            data.p1 += 1;
            Outcome::Changed
        }
        r => r.into(),
    });
    try_flow!(match state.button3.handle(event, Regular) {
        ButtonOutcome::Pressed => {
            data.p2 += 1;
            Outcome::Changed
        }
        r => r.into(),
    });

    Ok(Outcome::Continue)
}
