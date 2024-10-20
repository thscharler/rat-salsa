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
use rat_widget::clipper::{AreaHandle, Clipper, ClipperState, PageLayout};
use rat_widget::event::Outcome;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Span;
use ratatui::widgets::{Block, StatefulWidget, Widget};
use ratatui::Frame;
use std::array;
use std::cmp::max;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        layout: None,
        clipper: ClipperState::default(),
        hundred: array::from_fn(|_| Default::default()),
        hundred_areas: [Default::default(); 100],
        menu: Default::default(),
    };
    state.menu.focus.set(true);
    state.menu.select(Some(0));

    run_ui("pager1", handle_input, repaint_input, &mut data, &mut state)
}

struct Data {}

struct State {
    layout: Option<PageLayout>,
    clipper: ClipperState,

    hundred: [TextInputMockState; 100],
    hundred_areas: [AreaHandle; 100],

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
        istate.status[0] = "Ctrl-Q to quit. F4/F5 navigate page.".into();
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
        let mut pl = PageLayout::new();
        for i in 0..100 {
            let area = Rect::new(10, 2 * i as u16, 15, 1);
            state.hundred_areas[i] = pl.add(area);
        }
        state.layout = Some(pl.clone());
    };

    let mut clipper = Clipper::new()
        .layout(state.layout.clone().expect("layout"))
        .block(Block::bordered())
        .vscroll(Scroll::new())
        .into_widget(l2[1], &mut state.clipper);

    // render the input fields. relocate and render to tmp buffer.
    for i in 0..100 {
        // map an additional ad hoc area.
        if let Some(area) = clipper.relocate(Rect::new(5, 2 * i, 15, 1)) {
            Span::from(format!("{:?}:", i)).render(area, clipper.buffer_mut());
        }
        // map our widget area.
        if let Some(area) = clipper.relocate_handle(state.hundred_areas[i as usize]) {
            TextInputMock::default()
                .style(THEME.limegreen(0))
                .focus_style(THEME.limegreen(2))
                .render(area, clipper.buffer_mut(), &mut state.hundred[i as usize]);
        } else {
            // __Fallacy 1__
            // If the area is not reset here, it will be used by focus.
            // Can't do this inside the widget though.
            // I'm sure that's a nice little trap ... :(
            state.hundred[i as usize].clear_areas();
        }
    }

    clipper.render(l2[1], frame.buffer_mut(), &mut state.clipper);

    let menu1 = MenuLine::new()
        .title("#.#")
        .item_parsed("_Quit")
        .styles(THEME.menu_style());
    frame.render_stateful_widget(menu1, l1[3], &mut state.menu);

    for i in 0..100 {
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
    for i in 0..100 {
        // Focus wants __all__ areas.
        fb.widget(&state.hundred[i]);
    }
    fb.end(Some(state.clipper.c_focus.clone()));
    fb.build()
}

fn focus_by_handle(state: &State, handle: Option<AreaHandle>) {
    if let Some(handle) = handle {
        for i in 0..100 {
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
    for i in 0..100 {
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
