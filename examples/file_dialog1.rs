use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::MiniSalsaState;
use anyhow::anyhow;
use rat_event::{flow_ok, Dialog, HandleEvent, Outcome};
use rat_widget::event::FileOutcome;
use rat_widget::file_dialog::{FileOpen, FileOpenState};
use rat_widget::layout::layout_middle;
use rat_widget::menubar;
use rat_widget::menubar::{MenuBar, MenuBarState, MenuPopup, StaticMenu};
use rat_widget::menuline::MenuOutcome;
use rat_widget::popup_menu::Placement;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{Block, StatefulWidget};
use ratatui::Frame;
use std::path::PathBuf;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    mini_salsa::setup_logging()?;

    let mut state = State::default();
    state.menu.bar.focus.set(true);

    mini_salsa::run_ui(handle_input, repaint_input, &mut (), &mut state)
}

#[derive(Debug, Default)]
pub struct State {
    pub(crate) file_open: FileOpenState,
    pub(crate) menu: MenuBarState,
}

static MENU: StaticMenu = StaticMenu {
    menu: &[
        ("File", &["Open"]), //
        ("Quit", &[]),
    ],
};

fn repaint_input(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut (),
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

    MenuBar::new()
        .title("Wha!")
        .menu(&MENU)
        .styles(THEME.menu_style())
        .render(l1[1], frame.buffer_mut(), &mut state.menu);

    if state.file_open.active {
        let l = layout_middle(
            l1[0],
            Constraint::Length(state.menu.bar.item_areas[0].x),
            Constraint::Percentage(39),
            Constraint::Percentage(39),
            Constraint::Length(0),
        );

        FileOpen::new()
            .armed_style(THEME.armed_style())
            .button_style(THEME.button_style())
            .focus_style(THEME.focus())
            .style(THEME.dialog_style())
            .list_style(THEME.list_style())
            .select_style(THEME.select())
            .path_style(THEME.text_input())
            .render(l, frame.buffer_mut(), &mut state.file_open);

        if let Some(cursor) = state.file_open.screen_cursor() {
            frame.set_cursor(cursor.0, cursor.1);
        }
    }

    MenuPopup::new()
        .menu(&MENU)
        .block(Block::bordered())
        .styles(THEME.menu_style())
        .placement(Placement::Top)
        .render(l1[1], frame.buffer_mut(), &mut state.menu);

    Ok(())
}

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut (),
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    flow_ok!(match state.file_open.handle(event, Dialog)? {
        FileOutcome::Ok(path) => {
            state.file_open = Default::default();
            istate.status[0] = format!("Selected file {:?}", path);
            Outcome::Changed
        }
        FileOutcome::Cancel => {
            state.file_open = Default::default();
            istate.status[0] = "Select file cancelled.".to_string();
            Outcome::Changed
        }
        r => r.into(),
    });

    flow_ok!(
        match menubar::handle_popup_events(&mut state.menu, true, event) {
            MenuOutcome::MenuActivated(v, w) => {
                state.file_open.open(&PathBuf::from("."))?;
                state.menu.set_popup_active(false);
                Outcome::Changed
            }
            r => r.into(),
        }
    );

    flow_ok!(match menubar::handle_events(&mut state.menu, true, event) {
        MenuOutcome::Activated(v) => {
            if v == 3 {
                return Err(anyhow!("Quit"));
            }
            Outcome::Changed
        }
        r => r.into(),
    });

    Ok(Outcome::NotUsed)
}
