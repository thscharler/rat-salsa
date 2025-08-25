use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::MiniSalsaState;
use rat_event::{try_flow, Outcome};
use rat_menu::event::MenuOutcome;
use rat_menu::menubar::{Menubar, MenubarState};
use rat_menu::{menubar, StaticMenu};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::StatefulWidget;
use ratatui::Frame;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    mini_salsa::setup_logging()?;

    let mut data = Data::default();
    let mut state = State::default();
    state.menu.bar.focus.set(true);

    mini_salsa::run_ui(
        "menubar1",
        |_| {},
        handle_input,
        repaint_input,
        &mut data,
        &mut state,
    )
}

#[derive(Default)]
struct Data {}

#[derive(Default)]
struct State {
    pub(crate) menu: MenubarState,
}

static MENU: StaticMenu = StaticMenu {
    menu: &[
        (
            "_File",
            &[
                "_New", //
                "_Open...|F3",
                "_Save|F2",
                "\\...",
                "_Quit|Ctrl+Q",
            ],
        ),
        (
            "_Edit",
            &[
                "Undo|Ctrl+Z", //
                "Redo|Ctrl+Shift+Z",
                "\\___",
                "Cu_t|Ctrl+X",
                "_Copy|Ctrl+C",
                "_Paste|Ctrl+V",
            ],
        ),
        (
            "_Help",
            &[
                "_Help", //
                "_Web Help",
                "\\---",
                "_About",
            ],
        ),
    ],
};

fn repaint_input(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

    let (menu, menu_popup) = Menubar::new(&MENU)
        .styles(THEME.menu_style())
        .title("⋱⋰⋱⋰⋱")
        .into_widgets();
    menu.render(l1[1], frame.buffer_mut(), &mut state.menu);

    // todo: render something for the background ...

    menu_popup.render(l1[1], frame.buffer_mut(), &mut state.menu);

    Ok(())
}

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    try_flow!(
        match menubar::handle_popup_events(&mut state.menu, true, event) {
            MenuOutcome::MenuSelected(v, w) => {
                istate.status[0] = format!("Selected {}-{}", v, w);
                Outcome::Changed
            }
            MenuOutcome::MenuActivated(0, 3) => {
                istate.quit = true;
                Outcome::Changed
            }
            MenuOutcome::MenuActivated(v, w) => {
                istate.status[0] = format!("Activated {}-{}", v, w);
                state.menu.set_popup_active(false);
                Outcome::Changed
            }
            MenuOutcome::Selected(v) => {
                istate.status[0] = format!("Selected {}", v);
                Outcome::Changed
            }
            MenuOutcome::Activated(v) => {
                istate.status[0] = format!("Activated {}", v);
                Outcome::Changed
            }
            r => r.into(),
        }
    );

    Ok(Outcome::Continue)
}
