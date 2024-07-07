use crate::adapter::textinputf::{TextInputF, TextInputFState};
use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{layout_grid, run_ui, setup_logging, MiniSalsaState};
use rat_event::{ConsumedEvent, FocusKeys, HandleEvent, Outcome};
use rat_focus::{Focus, HasFocusFlag};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Span;
use ratatui::Frame;
use std::cmp::max;

mod adapter;
mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        input1: Default::default(),
        input2: Default::default(),
        input3: Default::default(),
        input4: Default::default(),
    };
    state.input1.focus.set(true);

    run_ui(handle_input, repaint_input, &mut data, &mut state)
}

struct Data {}

struct State {
    pub(crate) input1: TextInputFState,
    pub(crate) input2: TextInputFState,
    pub(crate) input3: TextInputFState,
    pub(crate) input4: TextInputFState,
}

fn repaint_input(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
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

    frame.render_widget(Span::from("Text 1"), l_grid[0][0]);
    let input1 = TextInputF::default()
        .style(THEME.text_input())
        .focus_style(THEME.text_input_focus());
    frame.render_stateful_widget(input1, l_grid[1][0], &mut state.input1);

    frame.render_widget(Span::from("Text 2"), l_grid[0][1]);
    let input2 = TextInputF::default()
        .style(THEME.text_input())
        .focus_style(THEME.text_input_focus());
    frame.render_stateful_widget(input2, l_grid[1][1], &mut state.input2);

    frame.render_widget(Span::from("Text 3"), l_grid[0][2]);
    let input3 = TextInputF::default()
        .style(THEME.text_input())
        .focus_style(THEME.text_input_focus());
    frame.render_stateful_widget(input3, l_grid[1][2], &mut state.input3);

    frame.render_widget(Span::from("Text 4"), l_grid[0][3]);
    let input4 = TextInputF::default()
        .style(THEME.text_input())
        .focus_style(THEME.text_input_focus());
    frame.render_stateful_widget(input4, l_grid[1][3], &mut state.input4);

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
        frame.set_cursor(cursor.0, cursor.1);
    }

    Ok(())
}

fn focus_input(state: &mut State) -> Focus {
    Focus::new(&[&state.input1, &state.input2, &state.input3, &state.input4])
}

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let f = focus_input(state).handle(event, FocusKeys);

    let mut r: Outcome = state.input1.handle(event, FocusKeys).into();
    if r.is_consumed() {
        return Ok(max(r, f));
    }
    r = state.input2.handle(event, FocusKeys).into();
    if r.is_consumed() {
        return Ok(max(r, f));
    }
    r = state.input3.handle(event, FocusKeys).into();
    if r.is_consumed() {
        return Ok(max(r, f));
    }
    r = state.input4.handle(event, FocusKeys).into();
    if r.is_consumed() {
        return Ok(max(r, f));
    }

    Ok(max(r, f))
}
