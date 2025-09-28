use crate::mini_salsa::{MiniSalsaState, mock_init};
use rat_event::{Dialog, HandleEvent, Outcome, try_flow};
use rat_menu::event::MenuOutcome;
use rat_menu::menubar::{Menubar, MenubarState};
use rat_menu::{StaticMenu, menubar};
use rat_popup::Placement;
use rat_text::HasScreenCursor;
use rat_widget::event::FileOutcome;
use rat_widget::file_dialog::{FileDialog, FileDialogState};
use rat_widget::layout::layout_middle;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{Block, StatefulWidget};
use std::path::PathBuf;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    mini_salsa::setup_logging()?;

    let mut state = State::default();
    state.menu.bar.focus.set(true);

    mini_salsa::run_ui("filedialog1", mock_init, event, render, &mut (), &mut state)
}

#[derive(Debug, Default)]
pub struct State {
    pub(crate) file_open: FileDialogState,
    pub(crate) menu: MenubarState,
}

static MENU: StaticMenu = StaticMenu {
    menu: &[
        ("File", &["Choose Dir", "Open", "Save"]), //
        ("Quit", &[]),
    ],
};

fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut (),
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

    let (menu, menu_popup) = Menubar::new(&MENU)
        .title("Wha!")
        .popup_block(Block::bordered())
        .popup_placement(Placement::Above)
        .styles(istate.theme.menu_style())
        .into_widgets();
    menu.render(l1[1], frame.buffer_mut(), &mut state.menu);

    if state.file_open.active {
        let l = layout_middle(
            l1[0],
            Constraint::Length(state.menu.bar.item_areas[0].x),
            Constraint::Percentage(39),
            Constraint::Percentage(39),
            Constraint::Length(0),
        );

        FileDialog::new()
            .styles(istate.theme.file_dialog_style()) //
            .render(l, frame.buffer_mut(), &mut state.file_open);

        if let Some(cursor) = state.file_open.screen_cursor() {
            frame.set_cursor_position((cursor.0, cursor.1));
        }
    }

    menu_popup.render(l1[1], frame.buffer_mut(), &mut state.menu);

    Ok(())
}

fn event(
    event: &crossterm::event::Event,
    _data: &mut (),
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    try_flow!(match state.file_open.handle(event, Dialog)? {
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

    try_flow!(
        match menubar::handle_popup_events(&mut state.menu, true, event) {
            MenuOutcome::MenuActivated(0, 0) => {
                state.file_open.directory_dialog(&PathBuf::from("."))?;
                Outcome::Changed
            }
            MenuOutcome::MenuActivated(0, 1) => {
                state.file_open.open_dialog(&PathBuf::from("."))?;
                Outcome::Changed
            }
            MenuOutcome::MenuActivated(0, 2) => {
                state.file_open.save_dialog(".", "sample.txt")?;
                Outcome::Changed
            }
            MenuOutcome::Activated(1) => {
                istate.quit = true;
                Outcome::Changed
            }
            r => r.into(),
        }
    );

    Ok(Outcome::Continue)
}
