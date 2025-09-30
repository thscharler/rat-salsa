#![allow(dead_code)]

use crate::mini_salsa::text_input_mock::{TextInputMock, TextInputMockState};
use crate::mini_salsa::{MiniSalsaState, mock_init, run_ui, setup_logging};
use log::debug;
use rat_event::{HandleEvent, Regular, ct_event, try_flow};
use rat_focus::{Focus, FocusBuilder, FocusFlag};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_text::HasScreenCursor;
use rat_widget::event::{FormOutcome, Outcome};
use rat_widget::form::{Form, FormState};
use rat_widget::layout::{FormLabel, FormWidget, LayoutForm};
use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::text::Span;
use ratatui::widgets::{Padding, Widget};
use std::array;

mod mini_salsa;

const HUN: usize = 100;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        flex: Default::default(),
        columns: 1,
        line_spacing: 1,
        form: FormState::default(),
        hundred: array::from_fn(|_| Default::default()),
        menu: Default::default(),
    };
    state.menu.focus.set(true);

    run_ui("pager1", mock_init, event, render, &mut data, &mut state)
}

struct Data {}

struct State {
    flex: Flex,
    columns: u8,
    line_spacing: u16,
    form: FormState<FocusFlag>,
    hundred: [TextInputMockState; HUN],
    menu: MenuLineState,
}

fn render(
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
    let form = Form::new() //
        .styles(istate.theme.form_style());

    // maybe rebuild layout
    let layout_size = form.layout_size(l2[1]);

    if !state.form.valid_layout(layout_size) {
        let mut form = LayoutForm::new() //
            .border(Padding::new(2, 2, 1, 1))
            .line_spacing(state.line_spacing)
            .columns(state.columns)
            .flex(state.flex);

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

        state.form.set_layout(form.build_paged(layout_size));
    }

    // set current layout and prepare rendering.
    let mut form = form.into_buffer(l2[1], frame.buffer_mut(), &mut state.form);

    // render the input fields.
    for i in 0..state.hundred.len() {
        // render manual label
        form.render_label(
            state.hundred[i].focus.clone(), //
            |_, a, b| Span::from("<<?>>").render(a, b),
        );

        // map our widget area.
        form.render(
            state.hundred[i].focus.clone(),
            || {
                TextInputMock::default()
                    .sample(format!("{:?}", i))
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

fn event(
    event: &crossterm::event::Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let mut focus = focus(state);

    istate.focus_outcome = focus.handle(event, Regular);

    try_flow!(match state.form.handle(event, Regular) {
        FormOutcome::Page => {
            state.form.focus_first(&focus);
            Outcome::Changed
        }
        r => {
            state.form.show_focused(&focus);
            r.into()
        }
    });

    try_flow!(match event {
        ct_event!(keycode press F(1)) => {
            debug!("{:#?}", state.form.layout());
            Outcome::Unchanged
        }
        ct_event!(keycode press F(2)) => flip_flex(state),
        ct_event!(keycode press F(3)) => flip_spacing(state),
        ct_event!(keycode press F(4)) => flip_columns(state),
        _ => Outcome::Continue,
    });

    try_flow!(match state.menu.handle(event, Regular) {
        MenuOutcome::Activated(0) => flip_flex(state),
        MenuOutcome::Activated(1) => flip_spacing(state),
        MenuOutcome::Activated(2) => flip_columns(state),
        MenuOutcome::Activated(3) => {
            istate.quit = true;
            Outcome::Changed
        }
        r => r.into(),
    });

    Ok(Outcome::Continue)
}

fn flip_flex(state: &mut State) -> Outcome {
    state.form.clear();
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
    state.form.clear();
    state.line_spacing = match state.line_spacing {
        0 => 1,
        1 => 2,
        2 => 3,
        _ => 0,
    };
    Outcome::Changed
}

fn flip_columns(state: &mut State) -> Outcome {
    state.form.clear();
    state.columns = match state.columns {
        1 => 2,
        2 => 3,
        3 => 4,
        4 => 5,
        _ => 1,
    };
    Outcome::Changed
}
