#![allow(dead_code)]

use crate::mini_salsa::text_input_mock::{TextInputMock, TextInputMockState};
use crate::mini_salsa::{MiniSalsaState, run_ui, setup_logging};
use log::debug;
use rat_event::{HandleEvent, Regular, ct_event, try_flow};
use rat_focus::{Focus, FocusBuilder, FocusFlag};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_text::HasScreenCursor;
use rat_widget::event::{Outcome, PagerOutcome};
use rat_widget::layout::{FormLabel, FormWidget, LayoutForm};
use rat_widget::pager::{DualPager, DualPagerState};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::widgets::block::{Position, Title};
use ratatui::widgets::{Block, Padding};
use std::array;

mod mini_salsa;

const HUN: usize = 100;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        flex: Default::default(),
        line_spacing: 1,
        columns: 1,
        pager: Default::default(),
        hundred: array::from_fn(|_| Default::default()),
        menu: Default::default(),
    };
    state.menu.focus.set(true);

    run_ui(
        "pager2",
        |_, _, _| {},
        handle_input,
        repaint_input,
        &mut data,
        &mut state,
    )
}

struct Data {}

struct State {
    flex: Flex,
    line_spacing: u16,
    columns: u8,
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
        .auto_label(true)
        .block(
            Block::bordered().title(
                Title::from(format!("{:?}", state.flex))
                    .alignment(Alignment::Center)
                    .position(Position::Top),
            ),
        )
        .styles(istate.theme.pager_style());

    // maybe rebuild layout
    let layout_size = pager.layout_size(l2[1]);
    if !state.pager.valid_layout(layout_size) {
        let mut form = LayoutForm::new()
            .mirror_odd_border()
            .border(Padding::new(4, 2, 0, 0))
            .line_spacing(state.line_spacing)
            .columns(state.columns)
            .flex(state.flex);

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
                FormLabel::String(format!("label {}", i).to_string()),
                FormWidget::WideStretchX(10, h),
            );

            if i == 17 {
                form.page_break();
            }
        }
        state.pager.set_layout(form.build_paged(layout_size));
    }

    // set current layout and prepare rendering.
    let mut pager = pager.into_buffer(l2[1], frame.buffer_mut(), &mut state.pager);

    // render the input fields.
    for i in 0..state.hundred.len() {
        // map our widget area.
        pager.render(
            state.hundred[i].focus.clone(),
            || {
                TextInputMock::default()
                    .sample(format!("text {}", i))
                    .style(istate.theme.limegreen(0))
                    .focus_style(istate.theme.limegreen(2))
            },
            &mut state.hundred[i],
        );
    }

    let menu1 = MenuLine::new()
        .title("#.#")
        .item_parsed("_Flex|F2")
        .item_parsed("_Spacing|F3")
        .item_parsed("_Columns|F4")
        .item_parsed("_Next|F8")
        .item_parsed("_Prev|F9")
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
    if istate.focus_outcome == Outcome::Changed {
        if let Some(ff) = focus.focused() {
            state.pager.show(ff);
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
        ct_event!(keycode press F(1)) => {
            debug!("{:#?}", state.pager.layout.borrow());
            Outcome::Unchanged
        }
        ct_event!(keycode press F(2)) => flip_flex(state),
        ct_event!(keycode press F(3)) => flip_spacing(state),
        ct_event!(keycode press F(4)) => flip_columns(state),
        ct_event!(keycode press F(8)) => prev_page(state, &focus),
        ct_event!(keycode press F(9)) => next_page(state, &focus),
        _ => Outcome::Continue,
    });

    try_flow!(match state.menu.handle(event, Regular) {
        MenuOutcome::Activated(0) => flip_flex(state),
        MenuOutcome::Activated(1) => flip_spacing(state),
        MenuOutcome::Activated(2) => flip_columns(state),
        MenuOutcome::Activated(3) => next_page(state, &focus),
        MenuOutcome::Activated(4) => prev_page(state, &focus),
        MenuOutcome::Activated(5) => {
            istate.quit = true;
            Outcome::Changed
        }
        r => r.into(),
    });

    Ok(Outcome::Continue)
}

fn flip_flex(state: &mut State) -> Outcome {
    state.pager.clear();
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

fn flip_spacing(state: &mut State) -> Outcome {
    state.pager.clear();
    state.line_spacing = match state.line_spacing {
        0 => 1,
        1 => 2,
        2 => 3,
        _ => 0,
    };
    Outcome::Changed
}

fn flip_columns(state: &mut State) -> Outcome {
    state.pager.clear();
    state.columns = match state.columns {
        1 => 2,
        2 => 3,
        3 => 4,
        4 => 5,
        _ => 1,
    };
    Outcome::Changed
}

fn prev_page(state: &mut State, focus: &Focus) -> Outcome {
    if state.pager.prev_page() {
        if let Some(widget) = state.pager.first(state.pager.page()) {
            focus.focus(&widget);
        }
        Outcome::Changed
    } else {
        Outcome::Unchanged
    }
}

fn next_page(state: &mut State, focus: &Focus) -> Outcome {
    if state.pager.next_page() {
        if let Some(widget) = state.pager.first(state.pager.page()) {
            focus.focus(&widget);
        }
        Outcome::Changed
    } else {
        Outcome::Unchanged
    }
}
