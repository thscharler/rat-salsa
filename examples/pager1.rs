#![allow(dead_code)]

use crate::mini_salsa::text_input_mock::{TextInputMock, TextInputMockState};
use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use rat_event::{ct_event, ConsumedEvent, HandleEvent, Regular};
use rat_focus::{Focus, FocusBuilder, FocusContainer, HasFocus};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_text::HasScreenCursor;
use rat_widget::event::{Outcome, PagerOutcome};
use rat_widget::pager::{AreaHandle, PageLayout, SinglePage, SinglePageState};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
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
        pager: SinglePageState::default(),
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
    layout: Option<PageLayout>,
    pager: SinglePageState,

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

            if i > 0 && i % 17 == 0 {
                pl.break_before(row);
            }

            row += h + 1;
        }
        pl.add(Rect::new(90, 0, 10, 1));
        state.layout = Some(pl.clone());
    };

    let mut pg_buf = SinglePage::new()
        .layout(state.layout.clone().expect("layout"))
        .block(Block::bordered())
        .nav_style(Style::new().fg(THEME.orange[2]))
        .style(THEME.gray(0))
        .into_buffer(l2[1], frame.buffer_mut(), &mut state.pager);

    // render the input fields.
    for i in 0..state.hundred.len() {
        // map an additional ad hoc area.
        let v_area = pg_buf.layout_area(state.hundred_areas[i]);
        let w_area = Rect::new(5, v_area.y, 5, 1);
        pg_buf.render_widget(Span::from(format!("{:?}:", i)), w_area);

        // map our widget area.
        pg_buf.render_stateful_handle(
            TextInputMock::default()
                .sample(format!("{:?}", state.hundred_areas[i]))
                .style(THEME.limegreen(0))
                .focus_style(THEME.limegreen(2)),
            state.hundred_areas[i],
            &mut state.hundred[i],
        );
    }

    pg_buf.render_stateful(
        TextInputMock::default()
            .sample("__outlier__")
            .style(THEME.orange(0))
            .focus_style(THEME.orange(2)),
        Rect::new(90, 0, 10, 1),
        &mut TextInputMockState::default(),
    );

    pg_buf
        .into_widget()
        .render(l2[1], frame.buffer_mut(), &mut state.pager);

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
    fb.start(Some(state.pager.c_focus.clone()), Default::default());
    for i in 0..state.hundred.len() {
        // Focus wants __all__ areas.
        fb.widget(&state.hundred[i]);
    }
    fb.end(Some(state.pager.c_focus.clone()));
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
            state.pager.show_handle(state.hundred_areas[i])
        }
    }

    let r = match HandleEvent::handle(&mut state.pager, event, Regular) {
        PagerOutcome::Page(_) => {
            let h = state.pager.first_handle(state.pager.page);
            focus_by_handle(state, h);
            Outcome::Changed
        }
        r => r.into(),
    };

    let r = r.or_else(|| {
        if state.pager.c_focus.is_container_focused() {
            match event {
                ct_event!(keycode press F(4)) => {
                    if state.pager.prev_page() {
                        let h = state.pager.first_handle(state.pager.page);
                        focus_by_handle(state, h);
                        Outcome::Changed
                    } else {
                        Outcome::Unchanged
                    }
                }
                ct_event!(keycode press F(5)) => {
                    if state.pager.next_page() {
                        let h = state.pager.first_handle(state.pager.page);
                        focus_by_handle(state, h);
                        Outcome::Changed
                    } else {
                        Outcome::Unchanged
                    }
                }
                _ => Outcome::Continue,
            }
        } else {
            Outcome::Continue
        }
    });

    let r = r.or_else(|| match state.menu.handle(event, Regular) {
        MenuOutcome::Activated(0) => {
            istate.quit = true;
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    Ok(max(f, r))
}
