#![allow(dead_code)]

use crate::mini_salsa::text_input_mock::{TextInputMock, TextInputMockState};
use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use log::debug;
use rat_event::{ct_event, ConsumedEvent, HandleEvent, Regular};
use rat_focus::{Focus, FocusBuilder, FocusFlag, HasFocus};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_scrolled::Scroll;
use rat_text::HasScreenCursor;
use rat_widget::clipper::{Clipper, ClipperBuffer, ClipperState};
use rat_widget::event::Outcome;
use rat_widget::layout::{FormLabel, FormWidget, LayoutForm};
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Padding, StatefulWidget};
use ratatui::Frame;
use std::array;
use std::cmp::max;
use std::rc::Rc;
use std::time::SystemTime;

mod mini_salsa;

const HUN: usize = 4448;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        t_focus: 0.0,
        n_focus: 0.0,
        focus: Default::default(),
        flex: Default::default(),
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
    t_focus: f64,
    n_focus: f64,
    focus: Option<Focus>,
    flex: Flex,
    pager: ClipperState<FocusFlag>,
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
    let pager = Clipper::new()
        .vscroll(Scroll::new().styles(THEME.scroll_style()))
        .block(
            Block::bordered()
                .borders(Borders::TOP | Borders::BOTTOM)
                .title_top(Line::from(format!("{:?}", state.flex)).alignment(Alignment::Center)),
        )
        .styles(THEME.clipper_style());

    let layout_size = pager.layout_size(l2[1], &state.pager);
    // rebuild layout
    if state.pager.layout.size_changed(layout_size) {
        let et = SystemTime::now();

        let mut form_layout = LayoutForm::new()
            .spacing(1)
            .flex(state.flex)
            .line_spacing(1)
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

        state.pager.set_layout(Rc::new(
            form_layout.endless(layout_size.width, Padding::default()),
        ));
        debug!(
            "layout {} {:?}",
            state.pager.layout.page_count(),
            et.elapsed()?
        );
    }

    // Render
    let et = SystemTime::now();
    let mut pager = pager.into_buffer(l2[1], &mut state.pager);
    render_page(&mut pager, state)?;
    pager
        .into_widget()
        .render(l2[1], frame.buffer_mut(), &mut state.pager);
    debug!("{:12}{:>12?}", "render", et.elapsed()?);

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
    pager: &mut ClipperBuffer<'_, FocusFlag>,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    // render container areas
    pager.render_block();

    // render the fields.
    for i in 0..state.hundred.len() {
        pager.render(
            state.hundred[i].focus(),
            || {
                // lazy construction
                TextInputMock::default()
                    .style(THEME.limegreen(0))
                    .focus_style(THEME.limegreen(2))
            },
            &mut state.hundred[i],
        );
    }

    Ok(())
}

fn focus(state: &mut State) -> Focus {
    let mut fb = FocusBuilder::new(state.focus.take());
    fb.widget(&state.menu);

    let tag = fb.start_container(Some(state.pager.container.clone()), Rect::default(), 0);
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
    debug!(
        "{:12}{:>12.2?}",
        "focus",
        state.t_focus / state.n_focus * 1e6f64
    );

    let f = focus.handle(event, Regular);

    // set the page from focus.
    if f == Outcome::Changed {
        if let Some(ff) = focus.focused() {
            state.pager.show(ff);
        }
    }

    let mut r = match state.pager.handle(event, Regular) {
        // PagerOutcome::Page(_p) => {
        //     if let Some(first) = state.pager.first(p) {
        //         focus.focus_flag(first.clone());
        //     }
        //     Outcome::Changed
        // }
        r => r,
    };

    r = r.or_else(|| match event {
        // ct_event!(keycode press F(4)) => {
        //     if state.pager.prev_page() {
        //         if let Some(first) = state.pager.first(state.pager.page()) {
        //             focus.focus_flag(first.clone());
        //         }
        //         Outcome::Changed
        //     } else {
        //         Outcome::Unchanged
        //     }
        // }
        // ct_event!(keycode press F(5)) => {
        //     if state.pager.next_page() {
        //         if let Some(first) = state.pager.first(state.pager.page()) {
        //             focus.focus_flag(first.clone());
        //         }
        //         Outcome::Changed
        //     } else {
        //         Outcome::Unchanged
        //     }
        // }
        ct_event!(keycode press F(2)) => {
            state.pager.set_layout(Default::default());
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

    state.focus = Some(focus);

    Ok(max(f, r))
}
