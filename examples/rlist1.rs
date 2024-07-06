use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{layout_grid, MiniSalsaState};
use anyhow::anyhow;
use rat_event::{flow_ok, FocusKeys, HandleEvent, Outcome};
use rat_scrolled::Scroll;
use rat_widget::list::selection::RowSelection;
use rat_widget::list::{RList, RListState};
use rat_widget::menubar;
use rat_widget::menubar::{MenuBar, MenuBarState, MenuPopup, StaticMenu};
use rat_widget::menuline::MenuOutcome;
use rat_widget::popup_menu::Placement;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, ListItem, StatefulWidget};
use ratatui::Frame;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    mini_salsa::setup_logging()?;

    let mut data = Data {
        data: vec![
            "1999".into(),
            "2000".into(),
            "2001".into(),
            "2002".into(),
            "2003".into(),
            "2004".into(),
            "2005".into(),
        ],
    };
    let mut state = State::default();
    state.menu.bar.focus.set(true);

    mini_salsa::run_ui(handle_input, repaint_input, &mut data, &mut state)
}

#[derive(Default)]
struct Data {
    pub data: Vec<String>,
}

#[derive(Default)]
struct State {
    pub(crate) list1: RListState<RowSelection>,
    pub(crate) menu: MenuBarState,
}

static MENU: StaticMenu = StaticMenu {
    menu: &[("Quit", &[])],
};

fn repaint_input(
    frame: &mut Frame<'_>,
    area: Rect,
    data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

    MenuBar::new()
        .title("Sample")
        .menu(&MENU)
        .title_style(Style::default().black().on_yellow())
        .style(Style::default().black().on_dark_gray())
        .focus_style(Style::default().black().on_cyan())
        .render(l1[1], frame.buffer_mut(), &mut state.menu);

    let l_grid = layout_grid::<2, 1>(
        l1[0],
        Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).spacing(5),
        Layout::vertical([Constraint::Fill(1)]),
    );

    RList::default()
        .items(data.data.iter().map(|v| ListItem::from(v)))
        .styles(THEME.list_styles())
        .scroll(Scroll::new())
        .render(l_grid[0][1], frame.buffer_mut(), &mut state.list1);

    MenuPopup::new()
        .menu(&MENU)
        .block(Block::bordered())
        .width(15)
        .style(Style::default().black().on_dark_gray())
        .focus_style(Style::default().black().on_cyan())
        .placement(Placement::Top)
        .render(l1[1], frame.buffer_mut(), &mut state.menu);

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

    flow_ok!(state.list1.handle(event, FocusKeys));

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
