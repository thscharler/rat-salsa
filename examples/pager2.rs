#![allow(dead_code)]

use crate::mini_salsa::text_input_mock::{TextInputMock, TextInputMockState};
use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use rat_event::{ct_event, ConsumedEvent, HandleEvent, Regular};
use rat_focus::{Focus, FocusBuilder, FocusFlag, HasFocus};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_text::HasScreenCursor;
use rat_widget::event::{Outcome, PagerOutcome};
use rat_widget::layout::{FormLabel, FormWidget, GenericLayout, LayoutForm};
use rat_widget::pager::{AreaHandle, DualPager, DualPagerState, PagerLayout};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Span;
use ratatui::widgets::{Padding, StatefulWidget};
use ratatui::Frame;
use std::array;
use std::cmp::max;
use std::rc::Rc;

mod mini_salsa;

const HUN: usize = 100;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        pager: DualPagerState::default(),
        hundred: array::from_fn(|_| Default::default()),
        menu: Default::default(),
    };
    state.menu.focus.set(true);
    state.menu.select(Some(0));

    run_ui("pager2", handle_input, repaint_input, &mut data, &mut state)
}

struct Data {}

struct State {
    pager: DualPagerState<FocusFlag>,
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
    let pager = DualPager::new() //
        .styles(THEME.pager_style());

    // maybe rebuild layout
    let layout_size = pager.layout_size(l2[1]);
    if state.pager.layout.size_changed(layout_size) {
        let mut form = LayoutForm::new().mirror_odd_border();
        form.widget(FocusFlag::new(), FormLabel::None, FormWidget::Measure(40));

        for i in 0..state.hundred.len() {
            let h = if i == 0 {
                1
            } else if i % 3 == 0 {
                2
            } else if i % 5 == 0 {
                5
            } else if i % 11 == 0 {
                22
            } else {
                1
            };

            form.widget(
                state.hundred[i].focus.clone(),
                FormLabel::Str(format!("{}", i).to_string().into()),
                FormWidget::WideStretch(2),
            );

            if i == 17 {
                form.page_break();
            }
        }
        state.pager.layout = Rc::new(form.layout(layout_size, Padding::new(2, 1, 1, 0)));
    }

    // set current layout and prepare rendering.
    let mut pager = pager.into_buffer(l2[1], frame.buffer_mut(), &mut state.pager);

    // render the input fields.
    for i in 0..state.hundred.len() {
        // map our widget area.
        pager.render(
            &state.hundred[i].focus.clone(),
            || {
                TextInputMock::default()
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
    let f = focus.handle(event, Regular);

    if f == Outcome::Changed {
        if let Some(ff) = focus.focused() {
            state.pager.show(&ff);
        }
    }

    let mut r = match state.pager.handle(event, Regular) {
        PagerOutcome::Page(p) => {
            if let Some(first) = state.pager.first(p) {
                focus.focus_flag(first.clone());
            }
            Outcome::Changed
        }
        r => r.into(),
    };

    r = r.or_else(|| match event {
        ct_event!(keycode press F(4)) => {
            if state.pager.prev_page() {
                if let Some(first) = state.pager.first(state.pager.page()) {
                    focus.focus_flag(first.clone());
                }
                Outcome::Changed
            } else {
                Outcome::Unchanged
            }
        }
        ct_event!(keycode press F(5)) => {
            if state.pager.next_page() {
                if let Some(first) = state.pager.first(state.pager.page()) {
                    focus.focus_flag(first.clone());
                }
                Outcome::Changed
            } else {
                Outcome::Unchanged
            }
        }
        _ => Outcome::Continue,
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
