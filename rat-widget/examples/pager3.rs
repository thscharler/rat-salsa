#![allow(dead_code)]

use crate::mini_salsa::text_input_mock::{TextInputMock, TextInputMockState};
use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{MiniSalsaState, run_ui, setup_logging};
use log::debug;
use rat_event::{HandleEvent, Regular, ct_event, try_flow};
use rat_focus::{Focus, FocusBuilder, FocusFlag, HasFocus};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_reloc::RelocatableState;
use rat_text::HasScreenCursor;
use rat_widget::event::{Outcome, PagerOutcome};
use rat_widget::layout::{FormLabel, FormWidget, GenericLayout, LayoutForm};
use rat_widget::pager::{PageNavigation, PageNavigationState, Pager};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, StatefulWidget};
use std::array;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::SystemTime;

mod mini_salsa;

const HUN: usize = 1213;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        t_focus: 0.0,
        n_focus: 0.0,
        focus: Default::default(),
        flex: Default::default(),
        line_spacing: 1,
        layout: Default::default(),
        page_nav: Default::default(),
        hundred: array::from_fn(|n| TextInputMockState::named(format!("{}", n).as_str())),
        menu: Default::default(),
    };
    state.menu.focus.set(true);

    run_ui(
        "pager3",
        |_, _, _| {},
        handle_input,
        repaint_input,
        &mut data,
        &mut state,
    )
}

struct Data {}

struct State {
    t_focus: f64,
    n_focus: f64,
    focus: Option<Focus>,
    flex: Flex,
    line_spacing: u16,
    layout: Rc<RefCell<GenericLayout<FocusFlag>>>,
    page_nav: PageNavigationState,
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
        istate.status[0] = "Ctrl-Q to quit. F2 flex. F3 line sp. F4/F5 navigate page.".into();
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
        .pages(2)
        .block(
            Block::bordered()
                .borders(Borders::TOP | Borders::BOTTOM)
                .title_top(Line::from(format!("{:?}", state.flex)).alignment(Alignment::Center)),
        )
        .styles(THEME.pager_style());

    let layout_size = nav.layout_size(l2[1]);

    // rebuild layout
    if state.layout.borrow().size_changed(layout_size) {
        let mut form_layout = LayoutForm::new()
            .spacing(1)
            .flex(state.flex)
            .line_spacing(state.line_spacing)
            .min_label(10);

        // generate the layout ...
        let mut tag8 = Default::default();
        let mut tag17 = Default::default();
        for i in 0..state.hundred.len() {
            let h = if i % 3 == 0 {
                2
            } else if i % 5 == 0 {
                5
            } else {
                1
            };
            let w = if i % 4 == 0 {
                12
            } else if i % 7 == 0 {
                19
            } else {
                15
            };

            if i == 17 {
                tag17 = form_layout.start(Some(
                    Block::bordered()
                        .border_type(BorderType::Double)
                        .style(THEME.bluegreen(0)),
                ));
            }
            if i == 20 {
                form_layout.end(tag17);
            }
            if i >= 8 {
                if i % 8 == 0 {
                    tag8 = form_layout.start(Some(Block::bordered()));
                }
                if (i - 4) % 8 == 0 {
                    form_layout.end(tag8);
                }
            }

            if i == 3 || i == 9 || i == 17 {
                form_layout.widget(
                    state.hundred[i].focus.clone(),
                    FormLabel::String(format!("{}", i).to_string()),
                    FormWidget::StretchXY(w, h),
                );
            } else {
                form_layout.widget(
                    state.hundred[i].focus.clone(),
                    FormLabel::String(format!("{}", i).to_string()),
                    FormWidget::Size(w, h),
                );
            }

            if i == 17 {
                form_layout.page_break();
            }
        }

        state.layout = Rc::new(RefCell::new(form_layout.build_paged(layout_size)));
        state
            .page_nav
            .set_page_count(state.layout.borrow().page_count());
    }

    // Render navigation
    nav.render(l2[1], frame.buffer_mut(), &mut state.page_nav);

    // reset state areas
    for i in 0..state.hundred.len() {
        state.hundred[i].relocate((0, 0), Rect::default());
    }
    // render 2 pages
    render_page(frame, state.page_nav.page, 0, state)?;
    render_page(frame, state.page_nav.page + 1, 1, state)?;

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

fn render_page(
    frame: &mut Frame<'_>,
    page: usize,
    area_idx: usize,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    // set up pager
    let mut pager = Pager::new() //
        .layout(state.layout.clone())
        .page(page)
        .label_alignment(Alignment::Right)
        .styles(THEME.pager_style())
        .into_buffer(
            state.page_nav.widget_areas[area_idx],
            Rc::new(RefCell::new(frame.buffer_mut())),
        );

    // render container areas
    pager.render_block();

    // render the fields.
    for i in 0..state.hundred.len() {
        let idx = pager.widget_idx(state.hundred[i].focus()).expect("fine");
        if pager.is_visible(idx) {
            if let Some(idx) = pager.widget_idx(state.hundred[i].focus.clone()) {
                pager.render_auto_label(idx);
                pager.render(
                    idx,
                    || {
                        // lazy render
                        TextInputMock::default()
                            .style(THEME.limegreen(0))
                            .focus_style(THEME.limegreen(2))
                    },
                    &mut state.hundred[i],
                );
            }
        }
    }

    // pager done.

    Ok(())
}

fn focus(state: &mut State) -> Focus {
    let mut fb = FocusBuilder::new(state.focus.take());
    fb.widget(&state.menu);

    let tag = fb.start_with_flags(state.page_nav.container.clone(), Rect::default(), 0);
    for i in 0..state.hundred.len() {
        // Focus wants __all__ areas.
        fb.widget(&state.hundred[i]);
    }
    fb.end(tag);

    fb.build()
}

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let et = SystemTime::now();
    let mut focus = focus(state);

    let tt = et.elapsed()?;
    state.t_focus += tt.as_secs_f64();
    state.n_focus += 1f64;

    istate.focus_outcome = focus.handle(event, Regular);
    // set the page from focus.
    if istate.focus_outcome == Outcome::Changed {
        if let Some(ff) = focus.focused() {
            if let Some(page) = state.layout.borrow().page_of(ff) {
                if page != state.page_nav.page {
                    state.page_nav.set_page((page / 2) * 2);
                }
            }
        }
    }

    try_flow!(match state.page_nav.handle(event, Regular) {
        PagerOutcome::Page(p) => {
            if let Some(w) = state.layout.borrow().first(p) {
                focus.focus(&w);
            }
            Outcome::Changed
        }
        r => r.into(),
    });

    try_flow!(match event {
        ct_event!(keycode press F(1)) => {
            debug!("{:#?}", state.layout.borrow());
            Outcome::Unchanged
        }
        ct_event!(keycode press F(2)) => {
            state.layout = Default::default();
            state.flex = match state.flex {
                Flex::Legacy => Flex::Start,
                Flex::Start => Flex::End,
                Flex::End => Flex::Center,
                Flex::Center => Flex::SpaceBetween,
                Flex::SpaceBetween => Flex::SpaceAround,
                Flex::SpaceAround => Flex::Legacy,
            };
            Outcome::Changed
        }
        ct_event!(keycode press F(3)) => {
            state.layout = Default::default();
            state.line_spacing = match state.line_spacing {
                0 => 1,
                1 => 2,
                2 => 3,
                _ => 0,
            };
            Outcome::Changed
        }
        ct_event!(keycode press F(4)) => {
            if state.page_nav.prev_page() {
                if let Some(w) = state.layout.borrow().first(state.page_nav.page) {
                    focus.focus(&w);
                }
                Outcome::Changed
            } else {
                Outcome::Unchanged
            }
        }
        ct_event!(keycode press F(5)) => {
            if state.page_nav.next_page() {
                if let Some(w) = state.layout.borrow().first(state.page_nav.page) {
                    focus.focus(&w);
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

    state.focus = Some(focus);

    Ok(Outcome::Continue)
}
