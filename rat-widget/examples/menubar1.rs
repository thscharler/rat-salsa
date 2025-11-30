use crate::mini_salsa::{MiniSalsaState, mock_init};
use rat_event::{Outcome, try_flow};
use rat_menu::event::MenuOutcome;
use rat_menu::menubar::{Menubar, MenubarState};
use rat_menu::{MenuStyle, StaticMenu, menubar};
use rat_popup::{Placement, PopupStyle};
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::style::{Color, Style};
use ratatui_core::symbols::border;
use ratatui_core::terminal::Frame;
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::{Block, Padding};
use ratatui_widgets::borders::Borders;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    mini_salsa::setup_logging()?;

    let mut data = Data::default();
    let mut state = State::new();
    state.menu.bar.focus.set(true);

    mini_salsa::run_ui("menubar1", mock_init, event, render, &mut data, &mut state)
}

#[derive(Default)]
struct Data {}

struct State {
    pub menu: MenubarState,
}

impl State {
    pub fn new() -> Self {
        Self {
            menu: MenubarState::default(),
        }
    }
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

fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([
        Constraint::Length(2),
        Constraint::Fill(1), //
    ])
    .split(area);

    let theme = &istate.theme;
    let st = MenuStyle {
        title: Some(theme.fg_style(theme.p.black[0])),
        style: Style::new().fg(theme.p.text_black).bg(theme.p.gray[1]),
        focus: Some(Style::new().fg(theme.p.text_light).bg(theme.p.gray[1])),
        right: Some(istate.theme.fg_style(theme.p.bluegreen[2])),
        disabled: Some(theme.fg_style(theme.p.gray[2])),
        highlight: Some(Style::default().underlined()),
        menu_block: Some(
            Block::bordered()
                .borders(Borders::BOTTOM)
                .border_set(border::QUADRANT_INSIDE),
        ),
        border_style: Some(Style::new().fg(theme.p.gray[0]).bg(Color::Reset)),
        title_style: None,

        popup: PopupStyle {
            alignment: None,
            placement: Some(Placement::AboveOrBelow),
            offset: Some((0, 0)),
            ..Default::default()
        },
        popup_style: Some(Style::new().fg(theme.p.white[0]).bg(theme.p.gray[1])),
        popup_separator: Some(theme.fg_style(theme.p.text_dark)),
        popup_focus: Some(theme.focus()),
        popup_highlight: Some(Style::default().underlined()),
        popup_disabled: Some(theme.fg_style(theme.p.gray[2])),
        popup_right: Some(istate.theme.fg_style(theme.p.bluegreen[2])),
        popup_block: Some(
            Block::bordered()
                .padding(Padding::new(0, 0, 0, 0))
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM | Borders::TOP)
                .border_set(border::QUADRANT_INSIDE),
        ),
        popup_border: Some(Style::new().fg(theme.p.gray[0]).bg(Color::Reset)),
        popup_title: None,

        ..Default::default()
    };

    let (menu, menu_popup) = Menubar::new(&MENU) //
        .styles(st)
        .title("⋱⋰⋱⋰⋱")
        .into_widgets();
    menu.render(l1[0], frame.buffer_mut(), &mut state.menu);

    // todo: render something for the background ...

    menu_popup.render(l1[0], frame.buffer_mut(), &mut state.menu);

    Ok(())
}

fn event(
    event: &Event,
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
