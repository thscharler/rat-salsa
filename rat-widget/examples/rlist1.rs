use crate::mini_salsa::{MiniSalsaState, layout_grid};
use rat_event::{HandleEvent, MouseOnly, Outcome, Popup, Regular, try_flow};
use rat_focus::{Focus, FocusBuilder, FocusFlag, HasFocus};
use rat_ftable::event::EditOutcome;
use rat_menu::StaticMenu;
use rat_menu::event::MenuOutcome;
use rat_menu::menubar::{Menubar, MenubarState};
use rat_popup::Placement;
use rat_scrolled::Scroll;
use rat_text::HasScreenCursor;
use rat_text::text_input::{TextInput, TextInputState};
use rat_theme4::WidgetStyle;
use rat_theme4::theme::SalsaTheme;
use rat_widget::list::List;
use rat_widget::list::edit::{EditList, EditListState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, ListItem, StatefulWidget, Widget};
use std::cmp::min;

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
        list1: EditListState::named("list1", EditEntryState::default()),
        menu: MenubarState::named("menu"),
    };

    mini_salsa::run_ui("rlist1", init, event, render, &mut state)
}

struct State {
    pub(crate) data: Vec<String>,
    pub(crate) list1: EditListState<EditEntryState>,
    pub(crate) menu: MenubarState,
}

static MENU: StaticMenu = StaticMenu {
    menu: &[
        ("Quit", &[]), //
        ("Zero", &["Zero", "Zero", "Seven"]),
    ],
};

#[derive(Debug)]
struct EditEntry<'a> {
    theme: &'a SalsaTheme,
}

#[derive(Debug)]
struct EditEntryState {
    text_input: TextInputState,
}

impl<'a> StatefulWidget for EditEntry<'a> {
    type State = EditEntryState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        TextInput::new()
            .styles(self.theme.style(WidgetStyle::TEXT))
            .render(area, buf, &mut state.text_input);
    }
}

impl Default for EditEntryState {
    fn default() -> Self {
        let s = Self {
            text_input: TextInputState::named("edit"),
        };
        s.text_input.focus().set(true);
        s
    }
}

impl HasFocus for EditEntryState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    fn focus(&self) -> FocusFlag {
        self.text_input.focus()
    }

    fn area(&self) -> Rect {
        self.text_input.area
    }
}

impl HasScreenCursor for EditEntryState {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        self.text_input.screen_cursor()
    }
}

impl HandleEvent<crossterm::event::Event, Regular, EditOutcome> for EditEntryState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> EditOutcome {
        Outcome::from(self.text_input.handle(event, Regular)).into()
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, EditOutcome> for EditEntryState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> EditOutcome {
        Outcome::from(self.text_input.handle(event, MouseOnly)).into()
    }
}

fn init(_ctx: &mut MiniSalsaState, state: &mut State) -> Result<(), anyhow::Error> {
    focus(state).first();
    state.list1.list.select(Some(0));
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

    Text::from(vec![
        Line::from("Enter: edit"),
        Line::from("Tab: edit next"),
        Line::from("Esc: cancel"),
    ])
    .render(l_grid[0][0], buf);

    EditList::new(
        List::default()
            .items(state.data.iter().map(|v| ListItem::from(v.as_str())))
            .styles(ctx.theme.style(WidgetStyle::LIST))
            .scroll(Scroll::new()),
        EditEntry { theme: &ctx.theme },
    )
    .render(l_grid[1][0], buf, &mut state.list1);

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
    event: &crossterm::event::Event,
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

    try_flow!({
        fn insert(state: &mut State) -> Outcome {
            if let Some(sel) = state.list1.list.selected() {
                state.data.insert(sel, "".into());
                state.list1.editor.text_input.set_text("");
                state.list1.edit_new(sel);
            }
            Outcome::Changed
        }

        fn remove(state: &mut State) -> Outcome {
            if let Some(sel) = state.list1.list.selected() {
                state.data.remove(sel);
                if state.data.len() == 0 {
                    state.data.push("".into());
                }
                state
                    .list1
                    .list
                    .select(Some(min(sel, state.data.len() - 1)));
            }
            Outcome::Changed
        }

        fn append(state: &mut State) -> Outcome {
            state.data.push("".into());
            state.list1.editor.text_input.set_text("");
            state.list1.edit_new(state.data.len() - 1);
            Outcome::Changed
        }

        fn edit(state: &mut State) -> Outcome {
            if let Some(sel) = state.list1.list.selected() {
                state.list1.editor.text_input.set_text(&state.data[sel]);
                state.list1.edit_new(sel);
            }
            Outcome::Changed
        }

        fn commit(state: &mut State) -> Outcome {
            if let Some(sel) = state.list1.list.selected() {
                let s = state.list1.editor.text_input.text().to_string();
                if !s.is_empty() {
                    state.data[sel] = s;
                    state.list1.commit();
                } else if state.data.len() == 1 {
                    // don't remove last
                    state.list1.commit();
                } else {
                    state.data.remove(sel);
                    state.list1.cancel();
                }
            }
            Outcome::Changed
        }

        fn cancel(state: &mut State) -> Outcome {
            if let Some(sel) = state.list1.list.selected() {
                if state.list1.is_insert() {
                    state.data.remove(sel);
                }
                state.list1.cancel();
            }
            Outcome::Changed
        }

        match state.list1.handle(event, Regular) {
            EditOutcome::Cancel => cancel(state),
            EditOutcome::Commit => commit(state),
            EditOutcome::CommitAndAppend => commit(state) //
                .and_then(|| append(state)),
            EditOutcome::CommitAndEdit => commit(state)
                .and_then(|| state.list1.list.move_down(1).into())
                .and_then(|| edit(state)),
            EditOutcome::Insert => insert(state),
            EditOutcome::Remove => remove(state),
            EditOutcome::Edit => edit(state),
            EditOutcome::Append => append(state),
            r => r.into(),
        }
    });

    Ok(Outcome::Continue)
}
