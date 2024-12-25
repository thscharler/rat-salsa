#![allow(dead_code)]

use crate::mini_salsa::text_input_mock::{TextInputMock, TextInputMockState};
use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use log::debug;
use rat_event::{ct_event, ConsumedEvent, HandleEvent, Regular};
use rat_focus::{Focus, FocusBuilder, FocusFlag};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_reloc::RelocatableState;
use rat_text::HasScreenCursor;
use rat_widget::event::{Outcome, PagerOutcome};
use rat_widget::layout::{FormLabel, FormWidget, GenericLayout, LayoutForm};
use rat_widget::pager::{PageNavigation, PageNavigationState, Pager};
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Style;
use ratatui::symbols::border::EMPTY;
use ratatui::widgets::{Block, Borders, Padding, StatefulWidget};
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
        flex: Default::default(),
        layout: Default::default(),
        page_nav: Default::default(),
        hundred: array::from_fn(|_| Default::default()),
        menu: Default::default(),
    };
    state.menu.focus.set(true);
    state.menu.select(Some(0));

    run_ui("pager3", handle_input, repaint_input, &mut data, &mut state)
}

struct Data {}

struct State {
    flex: Flex,
    layout: Rc<GenericLayout<FocusFlag>>,
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
        istate.status[0] = "Ctrl-Q to quit. F2 flex. F4/F5 navigate page.".into();
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
        .block(Block::bordered().borders(Borders::TOP | Borders::BOTTOM))
        .styles(THEME.pager_style());

    let layout_size = nav.layout_size(l2[1]);

    // rebuild layout
    if state.layout.size_changed(layout_size) {
        let mut form_layout = LayoutForm::new()
            .spacing(1)
            .flex(state.flex)
            .line_spacing(1);

        // generate the layout ...
        let mut c0 = 0;
        for i in 0..state.hundred.len() {
            let h = if i % 3 == 0 {
                2
            } else if i % 5 == 0 {
                5
            } else {
                1
            };

            if i >= 8 {
                if i % 8 == 0 {
                    form_layout.start((), Some(Block::bordered()));
                    c0 += 1;
                }
                if (i - 4) % 8 == 0 {
                    form_layout.end(());
                    c0 -= 1;
                }
            }
            if i == 17 {
                form_layout.start(
                    (),
                    Some(
                        Block::bordered().border_set(EMPTY).style(
                            Style::new()
                                .bg(THEME.purple[0])
                                .fg(THEME.text_color(THEME.purple[0])),
                        ),
                    ),
                );
            }
            if i == 23 {
                form_layout.end(());
            }

            form_layout.widget(
                state.hundred[i].focus.clone(),
                FormLabel::Str(format!("{}", i).to_string().into()),
                FormWidget::Size(15, h),
            );

            if i == 17 {
                form_layout.page_break();
            }
        }
        while c0 > 0 {
            form_layout.end(());
            c0 -= 1;
        }

        let et = SystemTime::now();
        state.layout = Rc::new(form_layout.layout(layout_size, Padding::default()));
        debug!("layout {:?}", et.elapsed()?);
        debug!(
            "page_count {}/{}",
            state.layout.page_count,
            (state.layout.page_count + 1) / 2
        );
        state
            .page_nav
            .set_page_count((state.layout.page_count + 1) / 2);
    }

    // Render navigation
    nav.render(l2[1], frame.buffer_mut(), &mut state.page_nav);

    // reset state areas
    for i in 0..state.hundred.len() {
        state.hundred[i].relocate((0, 0), Rect::default());
    }
    // render 2 pages
    render_page(frame, state.page_nav.page * 2, 0, state)?;
    render_page(frame, state.page_nav.page * 2 + 1, 1, state)?;

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
    let et = SystemTime::now();
    // set up pager
    let mut pager = Pager::new() //
        .layout(state.layout.clone())
        .page(page)
        .styles(THEME.pager_style())
        .into_buffer(state.page_nav.widget_areas[area_idx], frame.buffer_mut());

    // render container areas
    pager.render_container();

    // render the fields.
    for i in 0..state.hundred.len() {
        pager.render(
            &state.hundred[i].focus.clone(),
            || {
                // lazy construction
                TextInputMock::default()
                    .style(THEME.limegreen(0))
                    .focus_style(THEME.limegreen(2))
            },
            &mut state.hundred[i],
        );
    }

    // pager done.
    debug!("render {:?}", et.elapsed()?);

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
            if let Some(page) = state.layout.page_of(&ff) {
                let page = page / 2;
                if page != state.page_nav.page {
                    state.page_nav.set_page(page);
                }
            }
        }
    }

    let mut r = match state.page_nav.handle(event, Regular) {
        PagerOutcome::Page(p) => {
            if let Some(w) = state.layout.first(p * 2) {
                focus.focus_flag(w.clone());
            }
            Outcome::Changed
        }
        r => r.into(),
    };

    r = r.or_else(|| match event {
        ct_event!(keycode press F(4)) => {
            if state.page_nav.prev_page() {
                if let Some(w) = state.layout.first(state.page_nav.page * 2) {
                    focus.focus_flag(w.clone());
                }
                Outcome::Changed
            } else {
                Outcome::Unchanged
            }
        }
        ct_event!(keycode press F(5)) => {
            if state.page_nav.next_page() {
                if let Some(w) = state.layout.first(state.page_nav.page * 2) {
                    focus.focus_flag(w.clone());
                }
                Outcome::Changed
            } else {
                Outcome::Unchanged
            }
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
        _ => Outcome::Continue,
    });

    r = r.or_else(|| match state.menu.handle(event, Regular) {
        MenuOutcome::Activated(0) => {
            istate.quit = true;
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    Ok(max(f, r))
}
