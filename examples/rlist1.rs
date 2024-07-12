use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{layout_grid, MiniSalsaState};
use anyhow::anyhow;
#[allow(unused_imports)]
use log::Log;
use rat_event::{flow_ok, HandleEvent, Outcome, Popup, Regular};
use rat_focus::{Focus, FocusFlag, HasFocus, HasFocusFlag};
use rat_ftable::event::EditOutcome;
use rat_scrolled::Scroll;
use rat_widget::edit_list::{EditRList, EditRListState};
use rat_widget::input::{TextInput, TextInputState};
use rat_widget::list::List;
use rat_widget::menubar::{MenuBar, MenuBarState, MenuPopup, StaticMenu};
use rat_widget::menuline::MenuOutcome;
use rat_widget::popup_menu::Placement;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, ListItem, StatefulWidget, StatefulWidgetRef, Widget};
use ratatui::Frame;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    mini_salsa::setup_logging()?;

    let mut data = Data {
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
    };
    let mut state = State::default();
    focus(&state).enable_log(true).initial();

    mini_salsa::run_ui(handle_input, repaint_input, &mut data, &mut state)
}

#[derive(Default)]
struct Data {
    pub(crate) data: Vec<String>,
}

struct State {
    pub(crate) list1: EditRListState<EditEntryState>,
    pub(crate) menu: MenuBarState,
}

impl Default for State {
    fn default() -> Self {
        let mut s = Self {
            list1: Default::default(),
            menu: Default::default(),
        };
        s.menu.focus().set_name("menu");
        s.menu.bar.focus().set_name("menu-bar");
        s.menu.popup.focus().set_name("menu-popup");
        s.menu.bar.select(Some(0));
        s.list1.focus().set_name("list1-edit");
        s.list1.list.focus().set_name("list1");
        s.list1.list.select(Some(0));
        s
    }
}

static MENU: StaticMenu = StaticMenu {
    menu: &[
        ("Quit", &[]), //
        ("Zero", &["Zero", "Zero", "Seven"]),
    ],
};

#[derive(Debug, Default)]
struct EditEntry;

#[derive(Debug)]
struct EditEntryState {
    insert: bool,
    edit: TextInputState,
}

impl StatefulWidgetRef for EditEntry {
    type State = EditEntryState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        TextInput::new()
            .styles(THEME.input_style())
            .render_ref(area, buf, &mut state.edit);
    }
}

impl Default for EditEntryState {
    fn default() -> Self {
        let s = Self {
            insert: false,
            edit: Default::default(),
        };
        s.edit.focus().set(true);
        s.edit.focus().set_name("edit");
        s
    }
}

impl EditEntryState {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        self.edit.screen_cursor()
    }
}

impl HasFocusFlag for EditEntryState {
    fn focus(&self) -> &FocusFlag {
        self.edit.focus()
    }

    fn area(&self) -> Rect {
        self.edit.area
    }
}

impl HandleEvent<crossterm::event::Event, Regular, EditOutcome> for EditEntryState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> EditOutcome {
        Outcome::from(self.edit.handle(event, Regular)).into()
    }
}

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
        .styles(THEME.menu_style())
        .render(l1[1], frame.buffer_mut(), &mut state.menu);

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

    Text::from(vec![
        Line::from("Enter: edit"),
        Line::from("Tab: edit next"),
        Line::from("Esc: cancel"),
    ])
    .render(l_grid[0][0], frame.buffer_mut());

    EditRList::new(
        List::default()
            .items(data.data.iter().map(|v| ListItem::from(v.as_str())))
            .styles(THEME.list_styles())
            .scroll(Scroll::new()),
        EditEntry,
    )
    .render(l_grid[1][0], frame.buffer_mut(), &mut state.list1);
    if let Some(edit) = &state.list1.edit {
        if let Some(cursor) = edit.screen_cursor() {
            frame.set_cursor(cursor.0, cursor.1)
        }
    }

    MenuPopup::new()
        .menu(&MENU)
        .block(Block::bordered())
        .width(15)
        .styles(THEME.menu_style())
        .placement(Placement::Top)
        .render(l1[1], frame.buffer_mut(), &mut state.menu);

    Ok(())
}

fn focus(state: &State) -> Focus {
    let mut f = Focus::default();
    f.add_container(&state.list1);
    f.add(&state.menu);
    f
}

fn handle_input(
    event: &crossterm::event::Event,
    data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let f = focus(state).handle(event, Regular);

    flow_ok!(
        match state.menu.handle(event, Popup) {
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
    , consider f);

    flow_ok!({
        fn insert(data: &mut Data, state: &mut State) -> Outcome {
            if let Some(sel) = state.list1.list.selected() {
                let mut edit = EditEntryState::default();
                edit.insert = true;
                data.data.insert(sel, "".into());
                state.list1.list.items_added(sel, 1);
                state.list1.list.move_to(sel);
                state.list1.edit = Some(edit);
            }
            Outcome::Changed
        }
        fn remove(data: &mut Data, state: &mut State) -> Outcome {
            if let Some(sel) = state.list1.list.selected() {
                data.data.remove(sel);
            }
            Outcome::Changed
        }
        fn append(data: &mut Data, state: &mut State) -> Outcome {
            let mut edit = EditEntryState::default();
            edit.insert = true;
            data.data.push("".into());
            state.list1.list.items_added(data.data.len(), 1);
            state.list1.list.move_to(data.data.len() - 1);
            state.list1.edit = Some(edit);
            Outcome::Changed
        }
        fn edit(data: &mut Data, state: &mut State) -> Outcome {
            if let Some(sel) = state.list1.list.selected() {
                let mut edit = EditEntryState::default();
                edit.edit.set_value(&data.data[sel]);
                state.list1.edit = Some(edit);
            }
            Outcome::Changed
        }
        fn commit(data: &mut Data, state: &mut State) -> Outcome {
            if let Some(sel) = state.list1.list.selected() {
                if let Some(edit) = &state.list1.edit {
                    let s = edit.edit.value().to_string();
                    if !s.is_empty() {
                        data.data[sel] = s;
                    } else {
                        data.data.remove(sel);
                        state.list1.list.items_removed(sel, 1);
                        if data.data.is_empty() {
                            data.data.insert(sel, "".to_string());
                            state.list1.list.items_added(sel, 1);
                        }
                        if sel >= data.data.len() {
                            state.list1.list.select(Some(data.data.len() - 1));
                        }
                    }
                    state.list1.edit = None;
                }
            }
            Outcome::Changed
        }
        fn cancel(data: &mut Data, state: &mut State) -> Outcome {
            if let Some(sel) = state.list1.list.selected() {
                if let Some(edit) = &state.list1.edit {
                    if edit.insert {
                        data.data.remove(sel);
                        state.list1.list.items_removed(sel, 1);
                        if data.data.is_empty() {
                            data.data.insert(sel, "".to_string());
                            state.list1.list.items_added(sel, 1);
                        }
                        if sel >= data.data.len() {
                            state.list1.list.select(Some(data.data.len() - 1));
                        }
                    }
                    state.list1.edit = None;
                }
            }
            Outcome::Changed
        }

        match state.list1.handle(event, Regular) {
            EditOutcome::Cancel => cancel(data, state),
            EditOutcome::Commit => commit(data, state),
            EditOutcome::CommitAndAppend => commit(data, state).then(|| append(data, state)),
            EditOutcome::CommitAndEdit => commit(data, state)
                .then(|| state.list1.list.move_down(1).into())
                .then(|| edit(data, state)),
            EditOutcome::Insert => insert(data, state),
            EditOutcome::Remove => remove(data, state),
            EditOutcome::Edit => edit(data, state),
            EditOutcome::Append => append(data, state),
            r => r.into(),
        }
    }, consider f);

    flow_ok!(match state.menu.handle(event, Regular) {
        MenuOutcome::Activated(0) => {
            return Err(anyhow!("Quit"));
        }
        r => r,
    }, consider f);

    Ok(f.max(Outcome::NotUsed))
}
