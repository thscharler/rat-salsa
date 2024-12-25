#![allow(dead_code)]

use crate::mini_salsa::text_input_mock::{TextInputMock, TextInputMockState};
use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use log::debug;
use rat_event::{ct_event, ConsumedEvent, HandleEvent, Regular};
use rat_focus::{Focus, FocusBuilder, FocusFlag, HasFocus};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_text::HasScreenCursor;
use rat_widget::event::{Outcome, PagerOutcome};
use rat_widget::layout::{GenericLayout, Label, LayoutForm, Widget};
use rat_widget::pager::{PageNavigation, PageNavigationState, Pager, PagerState};
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::widgets::{Block, Padding, StatefulWidget};
use ratatui::Frame;
use std::array;
use std::cmp::max;
use std::rc::Rc;
use std::time::SystemTime;

mod mini_salsa;

const HUN: usize = 100;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        layout: Default::default(),
        page_nav: Default::default(),
        pager: Default::default(),
        hundred: array::from_fn(|_| Default::default()),
        menu: Default::default(),
    };
    state.menu.focus.set(true);
    state.menu.select(Some(0));

    run_ui("pager3", handle_input, repaint_input, &mut data, &mut state)
}

struct Data {}

struct State {
    layout: GenericLayout<()>,
    page_nav: PageNavigationState,
    pager: PagerState<FocusFlag>,
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

    // Prepare navigation.
    let nav = PageNavigation::new()
        .pages(1)
        .block(Block::bordered())
        .styles(THEME.pager_style());

    let layout_size = nav.layout_size(l2[1]);

    // rebuild layout
    if state.pager.layout.size_changed(layout_size) {
        let mut form_layout = LayoutForm::new().spacing(1).flex(Flex::Start);

        for i in 0..state.hundred.len() {
            let h = if i % 3 == 0 {
                2
            } else if i % 5 == 0 {
                5
            } else {
                1
            };

            form_layout.widget(
                state.hundred[i].focus.clone(),
                Label::Str(format!("{}", i).to_string().into()),
                Widget::Size(15, h),
            );

            // if i > 0 && i % 17 == 0 {
            //     form_layout.page_break();
            // }
        }

        let et = SystemTime::now();
        state.pager.layout = Rc::new(form_layout.layout(layout_size, Padding::default()));
        debug!("elapsed {:?}", et.elapsed()?);
        state.page_nav.set_page_count(state.pager.layout.page_count);
    }

    // Render navigation
    nav.render(l2[1], frame.buffer_mut(), &mut state.page_nav);

    // set up pager
    let mut pager = Pager::new() //
        .page(state.page_nav.page())
        .styles(THEME.pager_style())
        .into_buffer(
            state.page_nav.widget_areas[0],
            frame.buffer_mut(),
            &mut state.pager,
        );

    // render the fields.
    for i in 0..state.hundred.len() {
        pager.render(
            &state.hundred[i].focus.clone(),
            || {
                TextInputMock::default()
                    .sample(format!("... {} ...", i))
                    .style(THEME.limegreen(0))
                    .focus_style(THEME.limegreen(2))
            },
            &mut state.hundred[i],
        );
    }

    // pager done.
    _ = pager.into_widget();

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

    // set the page from focus.
    if f == Outcome::Changed {
        if let Some(ff) = focus.focused() {
            if let Some(page) = state.pager.page_of(&ff) {
                state.page_nav.set_page(page);
            }
        }
    }

    let mut r = match state.page_nav.handle(event, Regular) {
        PagerOutcome::Page(p) => {
            if let Some(w) = state.pager.first(p) {
                focus.focus_flag(w.clone());
            }
            Outcome::Changed
        }
        r => r.into(),
    };

    let r = r.or_else(|| match event {
        ct_event!(keycode press F(4)) => {
            if state.page_nav.prev_page() {
                if let Some(w) = state.pager.first(state.page_nav.page) {
                    focus.focus_flag(w.clone());
                }
                Outcome::Changed
            } else {
                Outcome::Unchanged
            }
        }
        ct_event!(keycode press F(5)) => {
            if state.page_nav.next_page() {
                if let Some(w) = state.pager.first(state.page_nav.page) {
                    focus.focus_flag(w.clone());
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
