use crate::mini_salsa::{MiniSalsaState, layout_grid, mock_init, run_ui, setup_logging};
use rat_event::{HandleEvent, Regular, try_flow};
use rat_focus::{Focus, FocusBuilder};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_theme4::WidgetStyle;
use rat_widget::checkbox::{Checkbox, CheckboxState};
use rat_widget::event::Outcome;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Flex, Layout, Rect};
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::Block;
use ratatui_widgets::borders::BorderType;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State {
        c1: CheckboxState::named("c1"),
        c2: CheckboxState::named("c2"),
        c3: CheckboxState::named("c3"),
        menu: MenuLineState::named("menu"),
    };
    state.c2.set_value(true);

    run_ui("checkbox1", mock_init, event, render, &mut state)
}

struct State {
    c1: CheckboxState,
    c2: CheckboxState,
    c3: CheckboxState,
    menu: MenuLineState,
}

fn render(
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

    let lg = layout_grid::<2, 4>(
        l1[0],
        Layout::horizontal([
            Constraint::Length(15), //
            Constraint::Length(15),
        ])
        .flex(Flex::Start),
        Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .spacing(1),
    );

    Checkbox::new()
        .text("Carrots 🥕")
        .styles(ctx.theme.style(WidgetStyle::CHECKBOX))
        .render(lg[1][1], buf, &mut state.c1);

    Checkbox::new()
        .text("Potatoes 🥔\nTomatoes 🍅")
        .styles(ctx.theme.style(WidgetStyle::CHECKBOX))
        .render(lg[1][2], buf, &mut state.c2);

    Checkbox::new()
        .text("Onions 🧅")
        .styles(ctx.theme.style(WidgetStyle::CHECKBOX))
        .block(Block::bordered().border_type(BorderType::Rounded))
        .render(lg[1][3], buf, &mut state.c3);

    MenuLine::new()
        .title("x x x")
        .item_parsed("_Quit")
        .styles(ctx.theme.style(WidgetStyle::MENU))
        .render(l1[1], buf, &mut state.menu);

    Ok(())
}

fn focus(state: &mut State) -> Focus {
    let mut fb = FocusBuilder::new(None);
    fb.widget(&state.menu);
    fb.widget(&state.c1);
    fb.widget(&state.c2);
    fb.widget(&state.c3);
    let f = fb.build();
    f.enable_log();
    f
}

fn event(
    event: &Event,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let mut focus = focus(state);
    ctx.focus_outcome = focus.handle(event, Regular);

    try_flow!(state.c1.handle(event, Regular));
    try_flow!(state.c2.handle(event, Regular));
    try_flow!(state.c3.handle(event, Regular));
    try_flow!(match state.menu.handle(event, Regular) {
        MenuOutcome::Activated(v) => {
            match v {
                0 => {
                    ctx.quit = true;
                    Outcome::Changed
                }
                _ => Outcome::Changed,
            }
        }
        r => r.into(),
    });

    Ok(Outcome::Continue)
}
