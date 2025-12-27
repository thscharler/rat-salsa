use crate::mini_salsa::{MiniSalsaState, mock_init, run_ui};
use rat_event::{Outcome, try_flow};
use rat_menu::event::MenuOutcome;
use rat_menu::menubar::{Menubar, MenubarState};
use rat_menu::{MenuStyle, StaticMenu, menubar};
use rat_popup::{Placement, PopupStyle};
use rat_theme4::StyleName;
use rat_theme4::palette::Colors;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::style::Style;
use ratatui_core::symbols::border;
use ratatui_core::symbols::border::{
    QUADRANT_BOTTOM_HALF, QUADRANT_LEFT_HALF, QUADRANT_RIGHT_HALF, QUADRANT_TOP_HALF,
    QUADRANT_TOP_LEFT_BOTTOM_LEFT_BOTTOM_RIGHT, QUADRANT_TOP_RIGHT_BOTTOM_LEFT_BOTTOM_RIGHT,
};
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::{Block, Padding};
use ratatui_widgets::borders::Borders;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    mini_salsa::setup_logging()?;

    let mut state = State::new();
    state.menu.bar.focus.set(true);

    run_ui("menubar1", mock_init, event, render, &mut state)
}

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
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([
        Constraint::Length(2),
        Constraint::Fill(1), //
    ])
    .split(area);

    let theme = &ctx.theme;
    let st = MenuStyle {
        title: Some(theme.p.fg_style(Colors::Black, 0)),
        style: theme.p.fg_bg_style(Colors::TextDark, 0, Colors::Gray, 2),
        focus: Some(theme.p.fg_bg_style(Colors::TextLight, 0, Colors::Gray, 0)),
        right: Some(theme.p.fg_style(Colors::BlueGreen, 2)),
        disabled: Some(theme.p.fg_style(Colors::Gray, 2)),
        highlight: Some(Style::default().underlined()),
        menu_block: Some(
            Block::bordered()
                .borders(Borders::BOTTOM)
                .border_set(border::QUADRANT_INSIDE),
        ),
        border_style: Some(theme.p.fg_bg_style(Colors::Gray, 0, Colors::None, 0)),
        title_style: None,

        popup: PopupStyle {
            alignment: None,
            placement: Some(Placement::AboveOrBelow),
            offset: Some((0, 0)),
            ..Default::default()
        },
        popup_style: Some(theme.p.fg_bg_style(Colors::White, 0, Colors::Gray, 2)),
        popup_separator: Some(theme.p.fg_style(Colors::TextDark, 0)),
        popup_focus: Some(theme.style(Style::FOCUS)),
        popup_highlight: Some(Style::default().underlined()),
        popup_disabled: Some(theme.p.fg_style(Colors::Gray, 2)),
        popup_right: Some(theme.p.fg_style(Colors::BlueGreen, 2)),
        popup_block: Some(
            Block::bordered()
                .padding(Padding::new(0, 0, 0, 0))
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM | Borders::TOP)
                .border_set(border::Set {
                    top_left: QUADRANT_TOP_RIGHT_BOTTOM_LEFT_BOTTOM_RIGHT,
                    top_right: QUADRANT_TOP_LEFT_BOTTOM_LEFT_BOTTOM_RIGHT,
                    bottom_left: QUADRANT_TOP_LEFT_BOTTOM_LEFT_BOTTOM_RIGHT,
                    bottom_right: QUADRANT_TOP_RIGHT_BOTTOM_LEFT_BOTTOM_RIGHT,
                    vertical_left: QUADRANT_LEFT_HALF,
                    vertical_right: QUADRANT_RIGHT_HALF,
                    horizontal_top: QUADRANT_TOP_HALF,
                    horizontal_bottom: QUADRANT_BOTTOM_HALF,
                }),
        ),
        popup_border: Some(theme.p.fg_bg_style(Colors::Gray, 0, Colors::None, 0)),
        popup_title: None,

        ..Default::default()
    };

    let (menu, menu_popup) = Menubar::new(&MENU) //
        .styles(st)
        .title("⋱⋰⋱⋰⋱")
        .into_widgets();
    menu.render(l1[0], buf, &mut state.menu);

    // todo: render something for the background ...

    menu_popup.render(l1[0], buf, &mut state.menu);

    Ok(())
}

fn event(
    event: &Event,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    try_flow!(
        match menubar::handle_popup_events(&mut state.menu, true, event) {
            MenuOutcome::MenuSelected(v, w) => {
                ctx.status[0] = format!("Selected {}-{}", v, w);
                Outcome::Changed
            }
            MenuOutcome::MenuActivated(0, 3) => {
                ctx.quit = true;
                Outcome::Changed
            }
            MenuOutcome::MenuActivated(v, w) => {
                ctx.status[0] = format!("Activated {}-{}", v, w);
                state.menu.set_popup_active(false);
                Outcome::Changed
            }
            MenuOutcome::Selected(v) => {
                ctx.status[0] = format!("Selected {}", v);
                Outcome::Changed
            }
            MenuOutcome::Activated(v) => {
                ctx.status[0] = format!("Activated {}", v);
                Outcome::Changed
            }
            r => r.into(),
        }
    );

    Ok(Outcome::Continue)
}
