use crate::adapter::textinputf::{TextInputF, TextInputFState};
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use rat_event::ConsumedEvent;
use rat_focus::{Focus, HasFocusFlag};
use rat_widget::event::{FocusKeys, HandleEvent, Outcome};
use rat_widget::layout::{layout_edit, EditConstraint};
use rat_widget::statusline::StatusLineState;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Span;
use ratatui::Frame;

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
        status: Default::default(),
    };
    state.input1.widget.focus.set(true);
    state.status.status(0, "Ctrl+Q to quit.");

    run_ui(handle_input, repaint_input, &mut data, &mut state)
}

struct Data {}

struct State {
    pub(crate) input1: TextInputFState,
    pub(crate) input2: TextInputFState,
    pub(crate) input3: TextInputFState,
    pub(crate) input4: TextInputFState,
    pub(crate) status: StatusLineState,
}

fn repaint_input(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l0 = Layout::horizontal([Constraint::Length(25), Constraint::Fill(1)]).split(area);

    let le = layout_edit(
        l0[0],
        &[
            EditConstraint::Label("Text 1"),
            EditConstraint::Widget(15),
            EditConstraint::Label("Text 2"),
            EditConstraint::Widget(15),
            EditConstraint::Label("Text 3"),
            EditConstraint::Widget(15),
            EditConstraint::Label("Text 4"),
            EditConstraint::Widget(15),
        ],
    );
    let mut le = le.iter();

    frame.render_widget(Span::from("Text 1"), le.label());
    let input1 = TextInputF::default()
        .style(Style::default().black().on_green())
        .focus_style(Style::default().black().on_light_blue());
    frame.render_stateful_widget(input1, le.widget(), &mut state.input1);
    if let Some((x, y)) = state.input1.screen_cursor() {
        if state.input1.is_focused() {
            frame.set_cursor(x, y);
        }
    }

    frame.render_widget(Span::from("Text 2"), le.label());
    let input2 = TextInputF::default()
        .style(Style::default().black().on_green())
        .focus_style(Style::default().black().on_light_blue());
    frame.render_stateful_widget(input2, le.widget(), &mut state.input2);
    if let Some((x, y)) = state.input2.screen_cursor() {
        if state.input2.is_focused() {
            frame.set_cursor(x, y);
        }
    }

    frame.render_widget(Span::from("Text 3"), le.label());
    let input3 = TextInputF::default()
        .style(Style::default().black().on_green())
        .focus_style(Style::default().black().on_light_blue());
    frame.render_stateful_widget(input3, le.widget(), &mut state.input3);
    if let Some((x, y)) = state.input3.screen_cursor() {
        if state.input3.is_focused() {
            frame.set_cursor(x, y);
        }
    }

    frame.render_widget(Span::from("Text 4"), le.label());
    let input4 = TextInputF::default()
        .style(Style::default().black().on_green())
        .focus_style(Style::default().black().on_light_blue());
    frame.render_stateful_widget(input4, le.widget(), &mut state.input4);
    if let Some((x, y)) = state.input4.screen_cursor() {
        if state.input4.is_focused() {
            frame.set_cursor(x, y);
        }
    }

    Ok(())
}

fn focus_input(state: &mut State) -> Focus<'_> {
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
        return Ok(r | f);
    }
    r = state.input2.handle(event, FocusKeys).into();
    if r.is_consumed() {
        return Ok(r | f);
    }
    r = state.input3.handle(event, FocusKeys).into();
    if r.is_consumed() {
        return Ok(r | f);
    }
    r = state.input4.handle(event, FocusKeys).into();
    if r.is_consumed() {
        return Ok(r | f);
    }

    Ok(r | f)
}
