use crate::mini_salsa::{MiniSalsaState, layout_grid};
use rat_event::{HandleEvent, Outcome, Popup, Regular, try_flow};
use rat_focus::{Focus, FocusBuilder, HasFocus};
use rat_menu::StaticMenu;
use rat_menu::event::MenuOutcome;
use rat_menu::menubar::{Menubar, MenubarState};
use rat_popup::Placement;
use rat_scrolled::Scroll;
use rat_text::HasScreenCursor;
use rat_theme4::WidgetStyle;
use rat_widget::list::selection::RowSelection;
use rat_widget::list::{List, ListSelection, ListState};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::text::{Line, Text};
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::Block;
use ratatui_widgets::list::ListItem;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    mini_salsa::setup_logging()?;

    let mut state = State {
        data: vec![
            "1962 Dr. No".into(),
            "1963 From Russia with Love".into(),
            "1964 Goldfinger".into(),
            "1965 Thunderball".into(),
            "1967 You Only Live Twice".into(),
            "1969 On Her Majesty's Secret Service".into(),
            "1971 Diamonds Are Forever".into(),
            "1973 Live and Let Die".into(),
            "1974 The Man with the Golden Gun".into(),
            "1977 The Spy Who Loved Me".into(),
            "1979 Moonraker".into(),
            "1981 For Your Eyes Only".into(),
            "1983 Octopussy".into(),
            "1985 A View to a Kill".into(),
            "1987 The Living Daylights".into(),
            "1989 Licence to Kill".into(),
            "1995 GoldenEye".into(),
            "1997 Tomorrow Never Dies".into(),
            "1999 The World Is Not Enough".into(),
            "2002 Die Another Day".into(),
            "2006 Casino Royale".into(),
            "2008 Quantum of Solace".into(),
            "2012 Skyfall".into(),
            "2015 Spectre".into(),
            "2021 No Time to Die".into(),
        ],
        list1: ListState::named("list1"),
        menu: MenubarState::named("menu"),
    };
    state.list1.scroll.set_overscroll_by(Some(10));

    mini_salsa::run_ui("rlist1", init, event, render, &mut state)
}

struct State {
    pub(crate) data: Vec<String>,
    pub(crate) list1: ListState<RowSelection>,
    pub(crate) menu: MenubarState,
}

static MENU: StaticMenu = StaticMenu {
    menu: &[
        ("Quit", &[]), //
        ("Zero", &["Zero", "Zero", "Seven"]),
    ],
};

fn init(_ctx: &mut MiniSalsaState, state: &mut State) -> Result<(), anyhow::Error> {
    focus(state).first();
    state.list1.select(Some(0));
    Ok(())
}

fn render(
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

    let (menu, menu_popup) = Menubar::new(&MENU)
        .title("Sample")
        .popup_block(Block::bordered())
        .popup_width(15)
        .popup_placement(Placement::Above)
        .styles(ctx.theme.style(WidgetStyle::MENU))
        .into_widgets();
    menu.render(l1[1], buf, &mut state.menu);

    let l_grid = layout_grid::<3, 1>(
        l1[0],
        Layout::horizontal([
            Constraint::Length(15),
            Constraint::Fill(1),
            Constraint::Length(15),
        ])
        .spacing(5),
        Layout::vertical([Constraint::Fill(1)]),
    );

    List::default()
        .items(state.data.iter().map(|v| {
            let mut ll = v.clone();
            ll.push('\n');
            ll.push_str(v);
            ListItem::new(ll)
        }))
        .styles(ctx.theme.style(WidgetStyle::LIST))
        .scroll(Scroll::new())
        .render(l_grid[1][0], buf, &mut state.list1);

    Text::from(vec![
        Line::from(format!("page-len {}", state.list1.scroll.page_len)),
        Line::from(format!("max_offset {}", state.list1.scroll.max_offset)),
        Line::from(format!("offset {}", state.list1.scroll.offset)),
        Line::from(format!(
            "selection {}",
            state.list1.selection.lead_selection().unwrap_or_default()
        )),
    ])
    .render(l_grid[0][0], buf);

    if let Some(cursor) = state.list1.screen_cursor() {
        ctx.cursor = Some((cursor.0, cursor.1))
    }

    menu_popup.render(l1[1], buf, &mut state.menu);

    Ok(())
}

fn focus(state: &State) -> Focus {
    let mut fb = FocusBuilder::default();
    fb.widget(&state.list1).widget(&state.menu);
    fb.build()
}

fn event(
    event: &Event,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    ctx.focus_outcome = focus(state).handle(event, Regular);

    try_flow!(match state.menu.handle(event, Popup) {
        MenuOutcome::MenuSelected(v, w) => {
            ctx.status[0] = format!("Selected {}-{}", v, w);
            Outcome::Changed
        }
        MenuOutcome::MenuActivated(v, w) => {
            ctx.status[0] = format!("Activated {}-{}", v, w);
            state.menu.set_popup_active(false);
            Outcome::Changed
        }
        MenuOutcome::Activated(0) => {
            ctx.quit = true;
            Outcome::Changed
        }
        r => r.into(),
    });

    try_flow!(state.list1.handle(event, Regular));

    Ok(Outcome::Continue)
}
