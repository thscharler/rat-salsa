use crate::mini_salsa::{MiniSalsaState, mock_init, run_ui, setup_logging};
use log::debug;
use rat_event::{HandleEvent, Regular};
use rat_focus::{Focus, FocusBuilder};
use rat_widget::event::Outcome;
use rat_widget::hyperlink::Hyperlink;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::style::Style;
use ratatui_core::widgets::Widget;
use ratatui_crossterm::crossterm::event::Event;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State {};

    run_ui("link1", mock_init, event, render, &mut state)
}

struct State {}

fn render(
    buf: &mut Buffer,
    area: Rect,
    _ctx: &mut MiniSalsaState,
    _state: &mut State,
) -> Result<(), anyhow::Error> {
    let l0 = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
        .spacing(1)
        .split(area);

    let l1 = Layout::horizontal([
        Constraint::Length(15),
        Constraint::Fill(1),
        Constraint::Length(11),
    ])
        .split(l0[1]);

    debug!("first link {:?}", l1[1]);
    buf.set_style(l1[1], Style::new().on_gray());
    Hyperlink::new("link", "https://github.com/ratatui/ratatui/issues/1227") //
        .style(Style::new().red())
        .render(l1[1], buf);

    debug!("second link {:?}", l1[2]);
    buf.set_style(l1[2], Style::new().on_blue());
    Hyperlink::new(
        "link link link",
        "https://github.com/ratatui/ratatui/issues/1227",
    )
        .style(Style::new().green())
        .render(l1[2], buf);

    debug!("{:#?}", buf);

    Ok(())
}

fn focus(_state: &mut State) -> Focus {
    let fb = FocusBuilder::new(None);
    fb.build()
}

fn event(
    event: &Event,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let mut focus = focus(state);
    ctx.focus_outcome = focus.handle(event, Regular);

    Ok(Outcome::Continue)
}
