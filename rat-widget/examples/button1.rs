use crate::mini_salsa::{MiniSalsaState, mock_init, run_ui, setup_logging};
use rat_event::{HandleEvent, Regular, try_flow};
use rat_focus::{Focus, FocusBuilder};
use rat_theme4::WidgetStyle;
use rat_widget::button::{Button, ButtonState};
use rat_widget::event::{ButtonOutcome, Outcome};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Span;
use ratatui::widgets::Widget;
use ratatui::widgets::{Block, StatefulWidget};

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State {
        button1: Default::default(),
        button2: Default::default(),
        button3: Default::default(),
        p0: 0,
        p1: 0,
        p2: 0,
    };

    run_ui("button1", mock_init, event, render, &mut state)
}

struct State {
    button1: ButtonState,
    button2: ButtonState,
    button3: ButtonState,

    p0: u8,
    p1: u8,
    p2: u8,
}

fn render(
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l0 = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(15),
        Constraint::Fill(1),
    ])
    .split(area);

    let l1 = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(3),
        Constraint::Length(5),
        Constraint::Length(5),
        Constraint::Fill(1),
    ])
    .spacing(1)
    .split(l0[1]);

    Button::new("Button")
        .block(Block::bordered())
        .styles(ctx.theme.style(WidgetStyle::BUTTON)) //
        .render(l1[1], buf, &mut state.button1);

    Button::new("Button\nPress\nButton")
        .styles(ctx.theme.style(WidgetStyle::BUTTON)) //
        .render(l1[2], buf, &mut state.button2);

    Button::new("Button")
        .styles(ctx.theme.style(WidgetStyle::BUTTON)) //
        .render(l1[3], buf, &mut state.button3);

    let l2 = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(3),
        Constraint::Length(5),
        Constraint::Length(5),
        Constraint::Fill(1),
    ])
    .split(l0[2]);
    Span::from(format!("  {} | {} | {}", state.p0, state.p1, state.p2)).render(l2[1], buf);

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
    event: &crossterm::event::Event,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let mut focus = focus(state);
    ctx.focus_outcome = focus.handle(event, Regular);

    try_flow!(match state.button1.handle(event, Regular) {
        ButtonOutcome::Pressed => {
            state.p0 += 1;
            Outcome::Changed
        }
        r => r.into(),
    });
    try_flow!(match state.button2.handle(event, Regular) {
        ButtonOutcome::Pressed => {
            state.p1 += 1;
            Outcome::Changed
        }
        r => r.into(),
    });
    try_flow!(match state.button3.handle(event, Regular) {
        ButtonOutcome::Pressed => {
            state.p2 += 1;
            Outcome::Changed
        }
        r => r.into(),
    });

    Ok(Outcome::Continue)
}
