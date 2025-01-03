use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{layout_grid, run_ui, setup_logging, MiniSalsaState};
use rat_event::{ConsumedEvent, HandleEvent, Regular};
use rat_focus::{Focus, FocusBuilder};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_widget::checkbox::{Checkbox, CheckboxState};
use rat_widget::event::Outcome;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::widgets::{Block, BorderType, StatefulWidget};
use ratatui::Frame;
use std::cmp::max;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        c1: CheckboxState::named("c1"),
        c2: CheckboxState::named("c2"),
        c3: CheckboxState::named("c3"),
        menu: MenuLineState::named("menu"),
    };
    state.c2.set_value(true);
    state.c2.set_default(true);

    run_ui(
        "choice1",
        handle_input,
        repaint_input,
        &mut data,
        &mut state,
    )
}

struct Data {}

struct State {
    c1: CheckboxState,
    c2: CheckboxState,
    c3: CheckboxState,
    menu: MenuLineState,
}

fn repaint_input(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
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
        .default_settable()
        .styles(THEME.checkbox_style())
        .render(lg[1][1], frame.buffer_mut(), &mut state.c1);

    Checkbox::new()
        .text("Potatoes 🥔\nTomatoes 🍅")
        .default_settable()
        .styles(THEME.checkbox_style())
        .render(lg[1][2], frame.buffer_mut(), &mut state.c2);

    Checkbox::new()
        .text("Onions 🧅")
        .styles(THEME.checkbox_style())
        .block(Block::bordered().border_type(BorderType::Rounded))
        .render(lg[1][3], frame.buffer_mut(), &mut state.c3);

    let menu1 = MenuLine::new()
        .title("x x x")
        .item_parsed("_Quit")
        .styles(THEME.menu_style());
    frame.render_stateful_widget(menu1, l1[1], &mut state.menu);

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

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let mut focus = focus(state);
    let f = focus.handle(event, Regular);

    let r = Outcome::Continue;
    let r = r.or_else(|| state.c1.handle(event, Regular));
    let r = r.or_else(|| state.c2.handle(event, Regular));
    let r = r.or_else(|| state.c3.handle(event, Regular));
    let r = r.or_else(|| match state.menu.handle(event, Regular) {
        MenuOutcome::Activated(v) => {
            match v {
                0 => {
                    istate.quit = true;
                    return Outcome::Changed;
                }
                _ => {}
            }
            Outcome::Changed
        }
        r => r.into(),
    });

    Ok(max(f, r))
}
