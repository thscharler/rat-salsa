use crate::mini_salsa::{MiniSalsaState, layout_grid, mock_init, run_ui, setup_logging};
use rat_event::{HandleEvent, Regular, ct_event, try_flow};
use rat_focus::{Focus, FocusBuilder};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_theme4::WidgetStyle;
use rat_widget::event::Outcome;
use rat_widget::radio::{Radio, RadioLayout, RadioState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::widgets::{Block, BorderType, StatefulWidget};

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State {
        layout: Default::default(),
        direction: Default::default(),
        c1: RadioState::named("c1"),
        c2: RadioState::named("c2"),
        c3: RadioState::named("c3"),
        menu: MenuLineState::named("menu"),
    };

    run_ui("radio1", mock_init, event, render, &mut state)
}

struct State {
    layout: RadioLayout,
    direction: Direction,

    c1: RadioState<&'static str>,
    c2: RadioState,
    c3: RadioState,
    menu: MenuLineState,
}

fn render(
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    if ctx.status[0] == "Ctrl-Q to quit." {
        ctx.status[0] = "Ctrl-Q to quit. F2/F3 change style/align.".into();
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
        .styles(ctx.theme.style(WidgetStyle::RADIO))
        .direction(state.direction)
        .layout(state.layout)
        .item("C", "🥕Carrots")
        .item("P", "🥔Potatoes")
        .item("O", "🧅Onions")
        .default_value("C")
        .render(lg[1][1], buf, &mut state.c1);

    Radio::new()
        .styles(ctx.theme.style(WidgetStyle::RADIO))
        .direction(state.direction)
        .layout(state.layout)
        .item(0, "wine")
        .item(1, "beer")
        .item(2, "water")
        .render(lg[1][2], buf, &mut state.c2);

    Radio::new()
        .styles(ctx.theme.style(WidgetStyle::RADIO))
        .direction(state.direction)
        .layout(state.layout)
        .item(0, "red")
        .item(1, "blue")
        .item(2, "green")
        .item(3, "pink")
        .block(Block::bordered().border_type(BorderType::Rounded))
        .render(lg[1][3], buf, &mut state.c3);

    MenuLine::new()
        .title(":-0")
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
    event: &crossterm::event::Event,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let mut focus = focus(state);

    ctx.focus_outcome = focus.handle(event, Regular);

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
