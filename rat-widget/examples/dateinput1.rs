use crate::mini_salsa::{MiniSalsaState, layout_grid, mock_init, run_ui, setup_logging};
use rat_event::try_flow;
use rat_text::HasScreenCursor;
use rat_theme4::WidgetStyle;
use rat_widget::date_input;
use rat_widget::date_input::{DateInput, DateInputState};
use rat_widget::event::Outcome;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Span;
use ratatui::widgets::{StatefulWidget, Widget};

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State {
        input: DateInputState::new().with_pattern("%x")?,
    };

    run_ui("dateinput1", mock_init, event, render, &mut state)
}

struct State {
    pub(crate) input: DateInputState,
}

fn render(
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l = layout_grid::<5, 4>(
        area,
        Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(15),
            Constraint::Length(25),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .spacing(1),
        Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ]),
    );

    DateInput::new() //
        .styles(ctx.theme.style(WidgetStyle::TEXT))
        .render(l[1][1], buf, &mut state.input);
    if let Some((x, y)) = state.input.screen_cursor() {
        ctx.cursor = Some((x, y));
    }

    Span::from(format!("{:?}", state.input.value())) //
        .render(l[2][1], buf);

    Ok(())
}

fn event(
    event: &crossterm::event::Event,
    _ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    try_flow!(date_input::handle_events(&mut state.input, true, event));

    Ok(Outcome::Continue)
}
