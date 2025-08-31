#![allow(dead_code)]

use crate::mini_salsa::text_input_mock::{TextInputMock, TextInputMockState};
use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use rat_event::{ct_event, try_flow, HandleEvent, Regular};
use rat_focus::{Focus, FocusBuilder, FocusFlag};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_text::HasScreenCursor;
use rat_widget::event::{Outcome, PagerOutcome};
use rat_widget::layout::{FormLabel, FormWidget, LayoutForm};
use rat_widget::pager::{SinglePager, SinglePagerState};
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::text::Span;
use ratatui::widgets::{Padding, Widget};
use ratatui::Frame;
use std::array;

mod mini_salsa;

const HUN: usize = 100;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        pager: SinglePagerState::default(),
        hundred: array::from_fn(|_| Default::default()),
        menu: Default::default(),
    };
    state.menu.focus.set(true);

    run_ui(
        "pager1",
        |_, _, _| {},
        handle_input,
        repaint_input,
        &mut data,
        &mut state,
    )
}

struct Data {}

struct State {
    pager: SinglePagerState<FocusFlag>,
    hundred: [TextInputMockState; HUN],
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

    // set up pager
    let pager = SinglePager::new() //
        .styles(THEME.pager_style());

    // maybe rebuild layout
    let layout_size = pager.layout_size(l2[1]);

    if !state.pager.valid_layout(layout_size) {
        let mut form = LayoutForm::new() //
            .spacing(1)
            .line_spacing(1)
            .flex(Flex::Legacy);
        for i in 0..state.hundred.len() {
            let h = if i % 3 == 0 {
                2
            } else if i % 5 == 0 {
                5
            } else {
                1
            };

            form.widget(
                state.hundred[i].focus.clone(),
                FormLabel::Width(5),
                FormWidget::Size(15, h),
            );
            if i == 17 {
                form.page_break();
            }
        }

        state
            .pager
            .set_layout(form.paged(layout_size, Padding::new(2, 2, 1, 1)));
    }

    // set current layout and prepare rendering.
    let mut pager = pager.into_buffer(l2[1], frame.buffer_mut(), &mut state.pager);

    // render the input fields.
    for i in 0..state.hundred.len() {
        // render manual label
        pager.render_label(
            state.hundred[i].focus.clone(), //
            |_, a, b| Span::from("<<?>>").render(a, b),
        );

        // map our widget area.
        pager.render(
            state.hundred[i].focus.clone(),
            || {
                TextInputMock::default()
                    .sample(format!("{:?}", i))
                    .style(THEME.limegreen(0))
                    .focus_style(THEME.limegreen(2))
            },
            &mut state.hundred[i],
        );
    }

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
    for i in 0..state.hundred.len() {
        // Focus wants __all__ areas.
        fb.widget(&state.hundred[i]);
    }
    fb.build()
}

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let mut focus = focus(state);

    istate.focus_outcome = focus.handle(event, Regular);
    // set the page from focus.
    if istate.focus_outcome == Outcome::Changed {
        if let Some(ff) = focus.focused() {
            if let Some(page) = state.pager.page_of(ff) {
                state.pager.set_page(page);
            }
        }
    }

    try_flow!(match state.pager.handle(event, Regular) {
        PagerOutcome::Page(p) => {
            if let Some(first) = state.pager.first(p) {
                focus.focus(&first);
            }
            Outcome::Changed
        }
        r => r.into(),
    });

    try_flow!(match event {
        ct_event!(keycode press F(4)) => {
            if state.pager.prev_page() {
                if let Some(widget) = state.pager.first(state.pager.page()) {
                    focus.focus(&widget);
                }
                Outcome::Changed
            } else {
                Outcome::Unchanged
            }
        }
        ct_event!(keycode press F(5)) => {
            if state.pager.next_page() {
                if let Some(widget) = state.pager.first(state.pager.page()) {
                    focus.focus(&widget);
                }
                Outcome::Changed
            } else {
                Outcome::Unchanged
            }
        }
        _ => Outcome::Continue,
    });

    try_flow!(match state.menu.handle(event, Regular) {
        MenuOutcome::Activated(0) => {
            istate.quit = true;
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    Ok(Outcome::Continue)
}
