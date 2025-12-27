use crate::adapter::textinputf::{TextInputF, TextInputFState};
use crate::mini_salsa::mock_init;
use crate::mini_salsa::{MiniSalsaState, layout_grid, run_ui, setup_logging};
use rat_event::{ConsumedEvent, HandleEvent, Outcome, Regular};
use rat_focus::{Focus, FocusBuilder, HasFocus};
use rat_theme4::StyleName;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::style::Style;
use ratatui_core::text::Span;
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::Event;
use std::cmp::max;

mod adapter;
mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State {
        focus: None,
        input1: Default::default(),
        input2: Default::default(),
        input3: Default::default(),
        input4: Default::default(),
    };
    state.input1.focus.set(true);

    run_ui("focus_input1", mock_init, event, render, &mut state)
}

struct State {
    focus: Option<Focus>,

    input1: TextInputFState,
    input2: TextInputFState,
    input3: TextInputFState,
    input4: TextInputFState,
}

fn render(
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l0 = Layout::horizontal([Constraint::Length(25), Constraint::Fill(1)]).split(area);

    let l_grid = layout_grid::<2, 4>(
        l0[0],
        Layout::horizontal([Constraint::Length(10), Constraint::Length(20)]),
        Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ]),
    );

    Span::from("Text 1").render(l_grid[0][0], buf);
    TextInputF::default()
        .style(ctx.theme.style_style(Style::INPUT))
        .focus_style(ctx.theme.style_style(Style::FOCUS))
        .render(l_grid[1][0], buf, &mut state.input1);

    Span::from("Text 2").render(l_grid[0][1], buf);
    TextInputF::default()
        .style(ctx.theme.style_style(Style::INPUT))
        .focus_style(ctx.theme.style_style(Style::FOCUS))
        .render(l_grid[1][1], buf, &mut state.input2);

    Span::from("Text 3").render(l_grid[0][2], buf);
    TextInputF::default()
        .style(ctx.theme.style_style(Style::INPUT))
        .focus_style(ctx.theme.style_style(Style::FOCUS))
        .render(l_grid[1][2], buf, &mut state.input3);

    Span::from("Text 4").render(l_grid[0][3], buf);
    TextInputF::default()
        .style(ctx.theme.style_style(Style::INPUT))
        .focus_style(ctx.theme.style_style(Style::FOCUS))
        .render(l_grid[1][3], buf, &mut state.input4);

    let cursor = if state.input1.is_focused() {
        state.input1.screen_cursor()
    } else if state.input2.is_focused() {
        state.input2.screen_cursor()
    } else if state.input3.is_focused() {
        state.input3.screen_cursor()
    } else if state.input4.is_focused() {
        state.input4.screen_cursor()
    } else {
        None
    };
    if let Some(cursor) = cursor {
        ctx.cursor = Some((cursor.0, cursor.1));
    }

    Ok(())
}

fn focus_input(state: &mut State) -> &mut Focus {
    let mut fb = FocusBuilder::new(state.focus.take());
    fb.widget(&state.input1)
        .widget(&state.input2)
        .widget(&state.input3)
        .widget(&state.input4);
    state.focus = Some(fb.build());
    state.focus.as_mut().expect("focus")
}

fn event(
    event: &Event,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    // Handle events for focus.
    let f = focus_input(state).handle(event, Regular);

    // Return early if the outcome is anything but Outcome::Continue.
    // But when returning early take the result of focus into
    // consideration and return max(r, f).
    //
    // This way a Outcome::Changed from focus doesn't get lost.
    let r = state
        .input1
        .handle(event, Regular)
        .or_else(|| state.input2.handle(event, Regular))
        .or_else(|| state.input3.handle(event, Regular))
        .or_else(|| state.input4.handle(event, Regular));

    Ok(max(f, r))
}
