#![allow(dead_code)]

use crate::mini_salsa::text_input_mock::{TextInputMock, TextInputMockState};
use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use rat_event::{ConsumedEvent, HandleEvent, Regular};
use rat_focus::{Focus, FocusBuilder, HasFocus};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_scrolled::Scroll;
use rat_text::HasScreenCursor;
use rat_widget::event::Outcome;
use rat_widget::view::clipper::{AreaHandle, Clipper, ClipperLayout, ClipperState};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Span;
use ratatui::widgets::{Block, StatefulWidget};
use ratatui::Frame;
use std::array;
use std::cmp::max;

mod mini_salsa;

const HUN: usize = 100;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        layout: None,
        clipper: ClipperState::default(),
        hundred: array::from_fn(|_| Default::default()),
        hundred_areas: [Default::default(); HUN],
        menu: Default::default(),
    };
    state.menu.focus.set(true);
    state.menu.select(Some(0));

    run_ui("pager1", handle_input, repaint_input, &mut data, &mut state)
}

struct Data {}

struct State {
    layout: Option<ClipperLayout>,
    clipper: ClipperState,

    hundred: [TextInputMockState; HUN],
    hundred_areas: [AreaHandle; HUN],

    menu: MenuLineState,
}

fn repaint_input(
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

    if state.layout.is_none() {
        // the inner layout is fixed, need to init only once.
        let mut pl = ClipperLayout::new();
        let mut row = 0;
        for i in 0..state.hundred.len() {
            let h = if i % 3 == 0 {
                2
            } else if i % 5 == 0 {
                5
            } else {
                1
            };

            let area = Rect::new(10, row, 15, h);
            state.hundred_areas[i] = pl.add(area);

            row += h + 1;
        }
        pl.add(Rect::new(90, 0, 10, 1));
        state.layout = Some(pl.clone());
    };

    let mut clip_buf = Clipper::new()
        .layout(state.layout.clone().expect("layout"))
        .block(Block::bordered())
        .hscroll(Scroll::new().scroll_by(1))
        .vscroll(Scroll::new().scroll_by(1))
        .into_buffer(l2[1], &mut state.clipper);

    // render the input fields.
    for i in 0..state.hundred.len() {
        // map an additional ad hoc area.
        let v_area = clip_buf.layout_area(state.hundred_areas[i]);
        let w_area = Rect::new(5, v_area.y, 5, 1);
        clip_buf.render_widget(Span::from(format!("{:?}:", i)), w_area);

        // render widget
        clip_buf.render_stateful_handle(
            TextInputMock::default()
                .sample(format!("{:?}", state.hundred_areas[i]))
                .style(THEME.limegreen(0))
                .focus_style(THEME.limegreen(2)),
            state.hundred_areas[i],
            &mut state.hundred[i],
        );
    }

    clip_buf.render_stateful(
        TextInputMock::default()
            .sample("__outlier__")
            .style(THEME.orange(0))
            .focus_style(THEME.orange(2)),
        Rect::new(90, 0, 10, 1),
        &mut TextInputMockState::default(),
    );

    clip_buf
        .into_widget()
        .render(l2[1], frame.buffer_mut(), &mut state.clipper);

    let menu1 = MenuLine::new()
        .title("#.#")
        .item_parsed("_Quit")
        .styles(THEME.menu_style());
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
    fb.start(Some(state.clipper.c_focus.clone()), Default::default());
    for i in 0..state.hundred.len() {
        // Focus wants __all__ areas.
        fb.widget(&state.hundred[i]);
    }
    fb.end(Some(state.clipper.c_focus.clone()));
    fb.build()
}

fn focus_by_handle(state: &State, handle: Option<AreaHandle>) {
    if let Some(handle) = handle {
        for i in 0..state.hundred.len() {
            if state.hundred_areas[i] == handle {
                focus(state).focus(&state.hundred[i]);
            }
        }
    }
}

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let mut focus = focus(state);
    let f = focus.handle(event, Regular);

    // set the page from focus.
    for i in 0..state.hundred.len() {
        if state.hundred[i].gained_focus() {
            state.clipper.show_handle(state.hundred_areas[i])
        }
    }

    let r = HandleEvent::handle(&mut state.clipper, event, Regular);

    let r = r.or_else(|| match state.menu.handle(event, Regular) {
        MenuOutcome::Activated(0) => {
            istate.quit = true;
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    Ok(max(f, r))
}
