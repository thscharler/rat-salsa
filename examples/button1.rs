use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use rat_event::flow_ok;
use rat_widget::button;
use rat_widget::button::{Button, ButtonOutcome, ButtonState};
use rat_widget::event::Outcome;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::Widget;
use ratatui::style::{Style, Stylize};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, StatefulWidget};
use ratatui::Frame;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {
        p0: false,
        p1: false,
        p2: false,
    };

    let mut state = State {
        button1: Default::default(),
        button2: Default::default(),
        button3: Default::default(),
    };

    run_ui(handle_buttons, repaint_buttons, &mut data, &mut state)
}

struct Data {
    pub(crate) p0: bool,
    pub(crate) p1: bool,
    pub(crate) p2: bool,
}

struct State {
    pub(crate) button1: ButtonState,
    pub(crate) button2: ButtonState,
    pub(crate) button3: ButtonState,
}

fn repaint_buttons(
    frame: &mut Frame<'_>,
    area: Rect,
    data: &mut Data,
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
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(5),
        Constraint::Length(1),
        Constraint::Length(5),
        Constraint::Fill(1),
    ])
    .split(l0[0]);

    let mut button1 = Button::from("Button");
    button1 = button1.block(Block::bordered().border_type(BorderType::Rounded));
    button1 = button1.style(Style::new().on_black().green());
    button1.render(l1[1], frame.buffer_mut(), &mut state.button1);

    let mut button2 = Button::from("Button\nnottuB");
    button2 = button2.block(Block::bordered().border_type(BorderType::Plain));
    button2 = button2.style(Style::new().on_black().blue());
    button2.render(l1[3], frame.buffer_mut(), &mut state.button2);

    let mut button3 = Button::from("Button").style(Style::new().white().on_red());
    button3 = button3.block(Block::bordered().border_type(BorderType::QuadrantInside));
    button3 = button3.style(Style::new().white().on_red());
    button3.render(l1[5], frame.buffer_mut(), &mut state.button3);

    let l2 = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(5),
        Constraint::Length(1),
        Constraint::Length(5),
        Constraint::Fill(1),
    ])
    .split(l0[1]);

    let label1 = Span::from(format!("=> {}", data.p0));
    label1.render(l2[1], frame.buffer_mut());

    let label2 = Span::from(format!("=> {:?}", data.p1));
    label2.render(l2[3], frame.buffer_mut());

    let label3 = if !data.p0 && !data.p1 && data.p2 {
        Span::from("of course")
    } else {
        Span::from(format!("=> {}", data.p2))
    };
    label3.render(l2[5], frame.buffer_mut());

    Ok(())
}

fn handle_buttons(
    event: &crossterm::event::Event,
    data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    flow_ok!(
        match button::handle_mouse_events(&mut state.button1, event) {
            ButtonOutcome::Pressed => {
                data.p0 = !data.p0;
                Outcome::Changed
            }
            r => r.into(),
        }
    );

    flow_ok!(
        match button::handle_mouse_events(&mut state.button2, event) {
            ButtonOutcome::Pressed => {
                data.p1 = !data.p1;
                Outcome::Changed
            }
            r => r.into(),
        }
    );

    flow_ok!(
        match button::handle_mouse_events(&mut state.button3, event) {
            ButtonOutcome::Pressed => {
                data.p2 = !data.p2;
                Outcome::Changed
            }
            r => r.into(),
        }
    );

    Ok(Outcome::Continue)
}
