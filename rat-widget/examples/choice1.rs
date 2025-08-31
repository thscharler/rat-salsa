use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{layout_grid, run_ui, setup_logging, MiniSalsaState};
use log::debug;
use rat_event::{try_flow, HandleEvent, Popup, Regular};
use rat_focus::{Focus, FocusBuilder};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_widget::choice::{Choice, ChoiceState};
use rat_widget::event::{ChoiceOutcome, Outcome};
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, StatefulWidget};
use ratatui::Frame;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        c1: ChoiceState::named("c1"),
        c2: ChoiceState::named("c2"),
        c3: ChoiceState::named("c3"),
        menu: MenuLineState::named("menu"),
    };

    run_ui(
        "choice1",
        |_| {},
        handle_input,
        repaint_input,
        &mut data,
        &mut state,
    )
}

struct Data {}

struct State {
    c1: ChoiceState,
    c2: ChoiceState,
    c3: ChoiceState<Option<usize>>,
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
            Constraint::Length(25), //
            Constraint::Fill(1),
            Constraint::Length(25),
        ])
        .flex(Flex::Start),
        Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(3),
        ])
        .spacing(1),
    );

    let (w, p1) = Choice::new()
        .styles(THEME.choice_style())
        .auto_item("Carrots 🥕")
        .auto_item("Potatoes 🥔")
        .auto_item("Onions 🧅")
        .auto_item("Peas")
        .auto_item("Beans")
        .auto_item(Line::from_iter([
            Span::from("T").red(),
            Span::from("omatoes 🍅"),
        ]))
        .auto_item(Line::from_iter([
            Span::from("Aubergines "),
            Span::from("🍆"),
        ]))
        .auto_item("Chili")
        .auto_item("Äpfel 🍎")
        .auto_item("...")
        .popup_boundary(l1[0])
        .into_widgets();
    w.render(lg[1][1], frame.buffer_mut(), &mut state.c1);

    let (w, p2) = Choice::new()
        .styles(THEME.choice_style())
        .auto_item("wine")
        .auto_item("beer")
        .auto_item("water")
        .popup_boundary(l1[0])
        .into_widgets();
    w.render(lg[1][2], frame.buffer_mut(), &mut state.c2);

    let (w, p3) = Choice::<Option<usize>>::new()
        .styles(THEME.choice_style())
        .item(None, "red")
        .item(Some(0), "blue")
        .item(Some(1), "green")
        .block(Block::bordered().border_type(BorderType::Rounded))
        .popup_block(Block::bordered().border_type(BorderType::Rounded))
        .popup_boundary(l1[0])
        .into_widgets();
    w.render(lg[1][3], frame.buffer_mut(), &mut state.c3);

    p1.render(lg[1][1], frame.buffer_mut(), &mut state.c1);
    p2.render(lg[1][2], frame.buffer_mut(), &mut state.c2);
    p3.render(lg[1][3], frame.buffer_mut(), &mut state.c3);

    let menu1 = MenuLine::new()
        .title("a|b|c")
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

    // popup handling first
    try_flow!(state.c1.handle(event, Popup));
    try_flow!(state.c2.handle(event, Popup));
    try_flow!(match state.c3.handle(event, Popup) {
        ChoiceOutcome::Value => {
            debug!("c3 {:?}", state.c3.value());
            Outcome::Changed
        }
        r => r.into(),
    });

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
