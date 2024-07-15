use crate::mini_salsa::MiniSalsaState;
use anyhow::anyhow;
use rat_event::{flow_ok, Outcome};
use rat_widget::menubar;
use rat_widget::menubar::{MenuBarState, Menubar, StaticMenu};
use rat_widget::menuline::MenuOutcome;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::StatefulWidget;
use ratatui::Frame;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    mini_salsa::setup_logging()?;

    let mut data = Data::default();
    let mut state = State::default();
    state.menu.bar.focus.set(true);

    mini_salsa::run_ui(handle_input, repaint_input, &mut data, &mut state)
}

#[derive(Default)]
struct Data {}

#[derive(Default)]
struct State {
    pub(crate) menu: MenuBarState,
}

static MENU: StaticMenu = StaticMenu {
    menu: &[
        ("Alpha", &["One", "Two", "Three"]),
        ("Beta", &["Ex", "Why", "Sed"]),
        ("Gamma", &["No", "more", "ideas"]),
        ("Quit", &[]),
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
        .title("Sample")
        .title_style(Style::default().black().on_yellow())
        .style(Style::default().black().on_dark_gray())
        .focus_style(Style::default().black().on_cyan())
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
    flow_ok!(
        match menubar::handle_popup_events(&mut state.menu, true, event) {
            MenuOutcome::MenuSelected(v, w) => {
                istate.status[0] = format!("Selected {}-{}", v, w);
                Outcome::Changed
            }
            MenuOutcome::MenuActivated(v, w) => {
                istate.status[0] = format!("Activated {}-{}", v, w);
                state.menu.set_popup_active(false);
                Outcome::Changed
            }
            r => r.into(),
        }
    );

    flow_ok!(match menubar::handle_events(&mut state.menu, true, event) {
        MenuOutcome::Selected(v) => {
            istate.status[0] = format!("Selected {}", v);
            Outcome::Changed
        }
        MenuOutcome::Activated(v) => {
            istate.status[0] = format!("Activated {}", v);
            match v {
                3 => return Err(anyhow!("Quit")),
                _ => {}
            }
            Outcome::Changed
        }
        r => {
            r.into()
        }
    });

    Ok(Outcome::NotUsed)
}
