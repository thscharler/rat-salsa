use crate::mini_salsa::{MiniSalsaState, mock_init, run_ui, setup_logging};
use rat_event::{HandleEvent, Outcome, Regular, try_flow};
use rat_focus::{Focus, FocusBuilder, HasFocus};
use rat_scrolled::Scroll;
use rat_widget::view;
use rat_widget::view::{View, ViewState};
use ratatui_core::layout::{Constraint, Direction, Layout, Rect};
use ratatui_core::style::Style;
use ratatui_core::terminal::Frame;
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::barchart::{Bar, BarChart, BarGroup};
use ratatui_widgets::block::Block;
use ratatui_widgets::borders::BorderType;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        view_state: Default::default(),
    };

    run_ui(
        "view-barchart1",
        mock_init,
        event,
        render,
        &mut data,
        &mut state,
    )
}

struct Data {}

struct State {
    view_state: ViewState,
}

fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l = Layout::horizontal([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .split(area);

    let mut view_buf = View::new()
        .layout(Rect::new(0, 0, 200, 30))
        .block(Block::bordered().border_type(BorderType::Rounded))
        .vscroll(Scroll::new())
        .hscroll(Scroll::new())
        .into_buffer(l[1], &mut state.view_state);

    view_buf.render_widget(
        BarChart::default()
            .block(Block::bordered().title("BarChart"))
            .direction(Direction::Horizontal)
            .bar_width(3)
            .bar_gap(1)
            .group_gap(3)
            .bar_style(Style::new().yellow().on_red())
            .value_style(Style::new().red().bold())
            .label_style(Style::new().white())
            .data(&[("B0", 0), ("B1", 2), ("B2", 4), ("B3", 3)])
            .data(BarGroup::default().bars(&[Bar::default().value(10), Bar::default().value(20)]))
            .max(4),
        Rect::new(0, 0, 100, 15),
    );

    view_buf.finish(frame.buffer_mut(), &mut state.view_state);

    Ok(())
}

fn focus(state: &mut State) -> Focus {
    let mut fb = FocusBuilder::new(None);
    fb.widget(&state.view_state);
    fb.build()
}

fn event(
    event: &Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    istate.focus_outcome = focus(state).handle(event, Regular);

    // handle inner first.
    let view_focused = state.view_state.is_focused();
    try_flow!(view::handle_events(
        &mut state.view_state,
        view_focused,
        event
    ));
    // try_flow!(state.view_state.handle(event, Regular));

    Ok(Outcome::Continue)
}
