#![allow(dead_code)]

use crate::mini_salsa::text_input_mock::{TextInputMock, TextInputMockState};
use crate::mini_salsa::{MiniSalsaState, mock_init, run_ui, setup_logging};
use rat_event::{HandleEvent, Regular, try_flow};
use rat_focus::{Focus, FocusBuilder, FocusFlag};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_scrolled::Scroll;
use rat_text::HasScreenCursor;
use rat_widget::clipper::{Clipper, ClipperState};
use rat_widget::event::Outcome;
use rat_widget::layout::GenericLayout;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::Block;
use std::array;

mod mini_salsa;

const HUN: usize = 66;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        clipper: ClipperState::default(),
        hundred: array::from_fn(|_| Default::default()),
        menu: Default::default(),
    };
    state.menu.focus.set(true);

    run_ui("clipper1", mock_init, event, render, &mut data, &mut state)
}

struct Data {}

struct State {
    clipper: ClipperState<FocusFlag>,
    hundred: [TextInputMockState; HUN],
    menu: MenuLineState,
}

fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    if istate.status[0] == "Ctrl-Q to quit." {
        istate.status[0] = "Ctrl-Q to quit.".into();
    }

    let l1 = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);

    let l2 = Layout::horizontal([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(3),
    ])
    .split(l1[1]);

    let clipper = Clipper::new()
        .block(Block::bordered())
        .hscroll(Scroll::new().scroll_by(1))
        .vscroll(Scroll::new().scroll_by(1));

    if state.clipper.layout.borrow().is_empty() {
        // the inner layout is fixed, need to init only once.
        let mut gen_layout = GenericLayout::new();

        let mut row = 5;
        for i in 0..state.hundred.len() {
            gen_layout.add(
                state.hundred[i].focus.clone(),
                Rect::new(5 + row, row, 10, 1),
                None,
                Rect::default(),
            );
            row += 2;
        }

        state.clipper.set_layout(gen_layout);
    }

    let mut clip_buf = clipper.into_buffer(l2[1], &mut state.clipper);

    // render the input fields.
    for i in 0..state.hundred.len() {
        clip_buf.render(
            state.hundred[i].focus.clone(),
            || {
                TextInputMock::default()
                    .sample(format!("{:?}", i))
                    .style(istate.theme.limegreen(0))
                    .focus_style(istate.theme.limegreen(2))
            },
            &mut state.hundred[i],
        );
    }

    clip_buf.finish(frame.buffer_mut(), &mut state.clipper);

    let menu1 = MenuLine::new()
        .title("#.#")
        .item_parsed("_Quit")
        .styles(istate.theme.menu_style());
    frame.render_stateful_widget(menu1, l1[3], &mut state.menu);

    for i in 0..state.hundred.len() {
        if let Some(cursor) = state.hundred[i].screen_cursor() {
            frame.set_cursor_position(cursor);
        }
    }

    Ok(())
}

fn focus(state: &State) -> Focus {
    let mut fb = FocusBuilder::default();
    fb.widget(&state.menu);
    let tag = fb.start(&state.clipper);
    for i in 0..state.hundred.len() {
        // Focus wants __all__ areas.
        fb.widget(&state.hundred[i]);
    }
    fb.end(tag);
    fb.build()
}

fn event(
    event: &crossterm::event::Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let mut focus = focus(state);

    istate.focus_outcome = focus.handle(event, Regular);
    if istate.focus_outcome == Outcome::Changed {
        state.clipper.show_focused(&focus);
    }

    try_flow!(state.clipper.handle(event, Regular));

    try_flow!(match state.menu.handle(event, Regular) {
        MenuOutcome::Activated(0) => {
            istate.quit = true;
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    Ok(Outcome::Continue)
}
