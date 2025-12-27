#![allow(dead_code)]

use crate::mini_salsa::text_input_mock::{TextInputMock, TextInputMockState};
use crate::mini_salsa::{MiniSalsaState, mock_init, run_ui, setup_logging};
use log::debug;
use rat_event::{HandleEvent, Regular, ct_event, try_flow};
use rat_focus::{Focus, FocusBuilder, FocusFlag, HasFocus};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_text::HasScreenCursor;
use rat_theme4::WidgetStyle;
use rat_widget::event::{FormOutcome, Outcome};
use rat_widget::form::{Form, FormState};
use rat_widget::layout::{FormLabel, FormWidget, LayoutForm};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui_core::text::Line;
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::Block;
use ratatui_widgets::borders::{BorderType, Borders};
use std::array;
use std::time::SystemTime;

mod mini_salsa;

const HUN: usize = 52;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State {
        t_focus: 0.0,
        n_focus: 0.0,
        focus: Default::default(),
        flex: Default::default(),
        line_spacing: 1,
        columns: 1,
        form: Default::default(),
        hundred: array::from_fn(|n| TextInputMockState::named(format!("{}", n).as_str())),
        menu: Default::default(),
    };
    state.menu.focus.set(true);

    run_ui("pager3", mock_init, event, render, &mut state)
}

struct Data {}

struct State {
    t_focus: f64,
    n_focus: f64,
    focus: Option<Focus>,
    flex: Flex,
    line_spacing: u16,
    columns: u8,
    form: FormState<FocusFlag>,
    hundred: [TextInputMockState; HUN],
    menu: MenuLineState,
}

fn render(
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
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

    // Prepare navigation.
    let form = Form::new()
        .block(
            Block::bordered()
                .borders(Borders::TOP | Borders::BOTTOM)
                .title_top(Line::from(format!("{:?}", state.flex)).alignment(Alignment::Center)),
        )
        .styles(ctx.theme.style(WidgetStyle::FORM));

    let layout_size = form.layout_size(l2[1]);

    // rebuild layout
    if state.form.layout().size_changed(layout_size) {
        let mut form_layout = LayoutForm::new()
            .spacing(1)
            .flex(state.flex)
            .line_spacing(state.line_spacing)
            .columns(state.columns)
            .min_label(10);

        // generate the layout ...
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

            if i >= 17 {
                if i % 17 == 0 {
                    form_layout.start(Some(
                        Block::bordered()
                            .border_type(BorderType::Double)
                            .style(ctx.theme.p.bluegreen(0)),
                    ));
                }
                if (i - 8) % 17 == 0 {
                    form_layout.end_all();
                }
            }
            if i >= 8 {
                if i % 8 == 0 {
                    form_layout.start(Some(Block::bordered()));
                }
                if (i - 4) % 8 == 0 {
                    form_layout.end_all();
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
        form_layout.end_all();

        state.form.set_layout(form_layout.build_paged(layout_size));
    }

    let mut form = form.into_buffer(l2[1], buf, &mut state.form);

    // render the fields.
    for i in 0..state.hundred.len() {
        form.render(
            state.hundred[i].focus(),
            || {
                // lazy render
                TextInputMock::default()
                    .style(ctx.theme.p.limegreen(0))
                    .focus_style(ctx.theme.p.limegreen(2))
            },
            &mut state.hundred[i],
        );
    }

    let menu1 = MenuLine::new()
        .title("#.#")
        .item_parsed("_Flex|F2")
        .item_parsed("_Spacing|F3")
        .item_parsed("_Columns|F4")
        .item_parsed("_Next|F9")
        .item_parsed("_Prev|F10")
        .item_parsed("_Quit")
        .styles(ctx.theme.style(WidgetStyle::MENU));
    menu1.render(l1[3], buf, &mut state.menu);

    for i in 0..state.hundred.len() {
        if let Some(cursor) = state.hundred[i].screen_cursor() {
            ctx.cursor = Some(cursor);
        }
    }

    Ok(())
}

fn focus(state: &mut State) -> Focus {
    let mut fb = FocusBuilder::new(state.focus.take());
    fb.widget(&state.menu);

    let tag = fb.start(&state.form);
    for i in 0..state.hundred.len() {
        // Focus wants __all__ areas.
        fb.widget(&state.hundred[i]);
    }
    fb.end(tag);

    fb.build()
}

fn event(
    event: &Event,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let et = SystemTime::now();
    let mut focus = focus(state);

    let tt = et.elapsed()?;
    state.t_focus += tt.as_secs_f64();
    state.n_focus += 1f64;

    ctx.focus_outcome = focus.handle(event, Regular);
    // set the page from focus.
    if ctx.focus_outcome == Outcome::Changed {
        state.form.show_focused(&focus);
    }

    try_flow!(match state.form.handle(event, Regular) {
        FormOutcome::Page => {
            state.form.focus_first(&focus);
            Outcome::Changed
        }
        r => r.into(),
    });

    try_flow!(match event {
        ct_event!(keycode press F(1)) => {
            debug!("{:#?}", state.form.layout());
            Outcome::Unchanged
        }
        ct_event!(keycode press F(2)) => flip_flex(state),
        ct_event!(keycode press F(3)) => flip_spacing(state),
        ct_event!(keycode press F(4)) => flip_columns(state),
        ct_event!(keycode press F(9)) => prev_page(state, &focus),
        ct_event!(keycode press F(10)) => next_page(state, &focus),
        _ => Outcome::Continue,
    });

    try_flow!(match state.menu.handle(event, Regular) {
        MenuOutcome::Activated(0) => flip_flex(state),
        MenuOutcome::Activated(1) => flip_spacing(state),
        MenuOutcome::Activated(2) => flip_columns(state),
        MenuOutcome::Activated(3) => next_page(state, &focus),
        MenuOutcome::Activated(4) => prev_page(state, &focus),
        MenuOutcome::Activated(5) => {
            ctx.quit = true;
            Outcome::Changed
        }
        r => r.into(),
    });

    state.focus = Some(focus);

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
        Flex::SpaceAround => Flex::SpaceEvenly,
        Flex::SpaceEvenly => Flex::Legacy,
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

fn prev_page(state: &mut State, focus: &Focus) -> Outcome {
    if state.form.prev_page() {
        state.form.focus_first(focus);
        Outcome::Changed
    } else {
        Outcome::Unchanged
    }
}

fn next_page(state: &mut State, focus: &Focus) -> Outcome {
    if state.form.next_page() {
        state.form.focus_first(focus);
        Outcome::Changed
    } else {
        Outcome::Unchanged
    }
}
