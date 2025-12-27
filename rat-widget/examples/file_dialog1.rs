use crate::mini_salsa::{MiniSalsaState, mock_init};
use rat_event::{Dialog, HandleEvent, Outcome, try_flow};
use rat_menu::event::MenuOutcome;
use rat_menu::menubar::{Menubar, MenubarState};
use rat_menu::{StaticMenu, menubar};
use rat_popup::Placement;
use rat_text::HasScreenCursor;
use rat_theme4::WidgetStyle;
use rat_widget::event::FileOutcome;
use rat_widget::file_dialog::{FileDialog, FileDialogState};
use rat_widget::layout::layout_middle;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::Block;
use std::path::PathBuf;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    mini_salsa::setup_logging()?;

    let mut state = State::default();
    state.menu.bar.focus.set(true);

    mini_salsa::run_ui("filedialog1", mock_init, event, render, &mut state)
}

#[derive(Debug, Default)]
pub struct State {
    pub(crate) file_open: FileDialogState,
    pub(crate) menu: MenubarState,
}

static MENU: StaticMenu = StaticMenu {
    menu: &[
        ("File", &["Choose Dir", "Open", "Open multiple", "Save"]), //
        ("Quit", &[]),
    ],
};

fn render(
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

    let (menu, menu_popup) = Menubar::new(&MENU)
        .title("Wha!")
        .popup_block(Block::bordered())
        .popup_placement(Placement::Above)
        .styles(ctx.theme.style(WidgetStyle::MENU))
        .into_widgets();
    menu.render(l1[1], buf, &mut state.menu);

    if state.file_open.active {
        let l = layout_middle(
            l1[0],
            Constraint::Length(state.menu.bar.item_areas[0].x),
            Constraint::Percentage(39),
            Constraint::Percentage(39),
            Constraint::Length(0),
        );

        FileDialog::new()
            .styles(ctx.theme.style(WidgetStyle::FILE_DIALOG)) //
            .render(l, buf, &mut state.file_open);

        if let Some(cursor) = state.file_open.screen_cursor() {
            ctx.cursor = Some((cursor.0, cursor.1));
        }
    }

    menu_popup.render(l1[1], buf, &mut state.menu);

    Ok(())
}

fn event(
    event: &Event,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    try_flow!(match state.file_open.handle(event, Dialog)? {
        FileOutcome::Ok(path) => {
            state.file_open = Default::default();
            ctx.status[0] = format!("Selected file {:?}", path);
            Outcome::Changed
        }
        FileOutcome::OkList(paths) => {
            state.file_open = Default::default();
            ctx.status[0] = format!("Selected {} files", paths.len());
            Outcome::Changed
        }
        FileOutcome::Cancel => {
            state.file_open = Default::default();
            ctx.status[0] = "Select file cancelled.".to_string();
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
                state.file_open.open_many_dialog(&PathBuf::from("."))?;
                Outcome::Changed
            }
            MenuOutcome::MenuActivated(0, 3) => {
                state.file_open.save_dialog(".", "sample.txt")?;
                Outcome::Changed
            }
            MenuOutcome::Activated(1) => {
                ctx.quit = true;
                Outcome::Changed
            }
            r => r.into(),
        }
    );

    Ok(Outcome::Continue)
}
