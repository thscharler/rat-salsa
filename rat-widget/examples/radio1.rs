use crate::mini_salsa::{MiniSalsaState, layout_grid, run_ui, setup_logging};
use rat_event::{HandleEvent, Regular, ct_event, try_flow};
use rat_focus::{Focus, FocusBuilder};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_widget::event::Outcome;
use rat_widget::radio::{Radio, RadioLayout, RadioState};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::widgets::{Block, BorderType, StatefulWidget};

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        layout: Default::default(),
        direction: Default::default(),
        c1: RadioState::named("c1"),
        c2: RadioState::named("c2"),
        c3: RadioState::named("c3"),
        menu: MenuLineState::named("menu"),
    };

    run_ui(
        "radio1",
        |_, _, _| {},
        handle_input,
        repaint_input,
        &mut data,
        &mut state,
    )
}

struct Data {}

struct State {
    layout: RadioLayout,
    direction: Direction,

    c1: RadioState<&'static str>,
    c2: RadioState,
    c3: RadioState,
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
        istate.status[0] = "Ctrl-Q to quit. F2/F3 change style/align.".into();
    }

    let l1 = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

    let vc = if state.direction == Direction::Vertical {
        Constraint::Fill(4)
    } else {
        Constraint::Length(4)
    };

    let lg = layout_grid::<2, 4>(
        l1[0],
        Layout::horizontal([
            Constraint::Length(15), //
            Constraint::Fill(1),
            Constraint::Length(15),
        ])
        .flex(Flex::Start),
        Layout::vertical([
            Constraint::Fill(1), //
            vc,
            vc,
            vc,
            Constraint::Fill(1),
        ])
        .spacing(1),
    );

    Radio::new()
        .styles(istate.theme.radio_style())
        .direction(state.direction)
        .layout(state.layout)
        .item("C", "🥕Carrots")
        .item("P", "🥔Potatoes")
        .item("O", "🧅Onions")
        .item("L", "Peas\n&\nLentils")
        .default_value("C")
        .render(lg[1][1], frame.buffer_mut(), &mut state.c1);

    Radio::new()
        .styles(istate.theme.radio_style())
        .direction(state.direction)
        .layout(state.layout)
        .item(0, "wine")
        .item(1, "beer")
        .item(2, "water")
        .render(lg[1][2], frame.buffer_mut(), &mut state.c2);

    Radio::new()
        .styles(istate.theme.radio_style())
        .direction(state.direction)
        .layout(state.layout)
        .item(0, "red")
        .item(1, "blue")
        .item(2, "green")
        .item(3, "pink")
        .block(Block::bordered().border_type(BorderType::Rounded))
        .render(lg[1][3], frame.buffer_mut(), &mut state.c3);

    let menu1 = MenuLine::new()
        .title(":-0")
        .item_parsed("_Quit")
        .styles(istate.theme.menu_style());
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

    istate.focus_outcome = focus.handle(event, Regular);

    try_flow!(match event {
        ct_event!(keycode press SHIFT-F(2)) | ct_event!(keycode press F(2)) => {
            state.layout = match state.layout {
                RadioLayout::Stacked => RadioLayout::Spaced,
                RadioLayout::Spaced => RadioLayout::Stacked,
            };
            Outcome::Changed
        }
        ct_event!(keycode press SHIFT- F(3)) | ct_event!(keycode press F(3)) => {
            state.direction = match state.direction {
                Direction::Horizontal => Direction::Vertical,
                Direction::Vertical => Direction::Horizontal,
            };
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    try_flow!(state.c1.handle(event, Regular));
    try_flow!(state.c2.handle(event, Regular));
    try_flow!(state.c3.handle(event, Regular));
    try_flow!(match state.menu.handle(event, Regular) {
        MenuOutcome::Activated(v) => {
            match v {
                0 => {
                    istate.quit = true;
                    Outcome::Changed
                }
                _ => Outcome::Changed,
            }
        }
        r => r.into(),
    });

    Ok(Outcome::Continue)
}
