use crate::Global;
use crate::sample::sample_custom::SampleCustom;
use crate::sample::sample_data_input::SampleDataInput;
use crate::sample::sample_dialog::{SampleDialog, SampleDialogState};
use crate::sample::sample_list::SampleList;
use crate::sample::sample_readability::SampleReadability;
use crate::sample::sample_split::SampleSplit;
use crate::sample::sample_table::SampleTable;
use crate::sample::{
    sample_custom, sample_data_input, sample_list, sample_readability, sample_split, sample_table,
};
use anyhow::Error;
use pure_rust_locales::Locale;
use rat_theme4::themes::{create_dark, create_fallback, create_shell};
use rat_theme4::{StyleName, WidgetStyle};
use rat_widget::choice::{Choice, ChoiceState};
use rat_widget::event::{ChoiceOutcome, HandleEvent, Outcome, Popup, Regular, event_flow};
use rat_widget::focus::{Focus, FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_widget::menu::{Menubar, MenubarState, StaticMenu};
use rat_widget::popup::Placement;
use rat_widget::statusline::{StatusLine, StatusLineState};
use rat_widget::tabbed::{TabPlacement, Tabbed, TabbedState};
use rat_widget::text::HasScreenCursor;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Position, Rect};
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{Block, BorderType, Borders, StatefulWidget, Widget};
use std::iter::once;

#[derive(Debug)]
pub struct ShowSample {
    pub themes: ChoiceState<String>,
    pub menu: MenubarState,
    pub status: StatusLineState,

    pub tabs: TabbedState,
    pub input: SampleDataInput,
    pub readability: SampleReadability,
    pub dialog: SampleDialogState,
    pub split: SampleSplit,
    pub table: SampleTable,
    pub doc_table: SampleTable,
    pub list: SampleList,
    pub custom: SampleCustom,
}

impl ShowSample {
    pub fn new(loc: Locale) -> Self {
        let mut z = Self {
            themes: ChoiceState::named("themes"),
            menu: Default::default(),
            status: Default::default(),
            tabs: Default::default(),
            input: SampleDataInput::new(loc),
            readability: SampleReadability::default(),
            dialog: Default::default(),
            split: Default::default(),
            table: Default::default(),
            doc_table: Default::default(),
            list: Default::default(),
            custom: Default::default(),
        };
        z.status.status(0, "... something ...");
        z.status.status(1, "[join]");
        z.status.status(2, "[conn]");
        z.status.status(3, "[sync]");
        z
    }

    pub fn show_focused(&mut self, focus: &Focus) {
        match self.tabs.selected() {
            Some(0) => {
                self.input.form.show_focused(focus);
            }
            Some(1) => { /*noop*/ }
            Some(2) => { /*noop*/ }
            Some(3) => { /*noop*/ }
            Some(4) => { /*noop*/ }
            Some(5) => { /*noop*/ }
            Some(6) => { /*noop*/ }
            Some(7) => { /*noop*/ }
            _ => {}
        }
    }
}

impl HasFocus for ShowSample {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget(&self.themes);
        builder.widget(&self.menu);
        builder.widget_navigate(&self.tabs, Navigation::Regular);
        match self.tabs.selected() {
            Some(0) => {
                builder.widget(&self.input);
            }
            Some(1) => {
                builder.widget(&self.readability);
            }
            Some(2) => {
                builder.widget(&self.dialog);
            }
            Some(3) => {
                builder.widget(&self.split);
            }
            Some(4) => {
                builder.widget(&self.table);
            }
            Some(5) => {
                builder.widget(&self.doc_table);
            }
            Some(6) => {
                builder.widget(&self.list);
            }
            Some(7) => {
                builder.widget(&self.custom);
            }
            _ => {}
        }
    }

    fn focus(&self) -> FocusFlag {
        unimplemented!("not available")
    }

    fn area(&self) -> Rect {
        unimplemented!("not available")
    }
}

impl HasScreenCursor for ShowSample {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        match self.tabs.selected() {
            Some(0) => self.input.screen_cursor(),
            Some(1) => self.readability.screen_cursor(),
            Some(2) => self.dialog.screen_cursor(),
            Some(3) => self.split.screen_cursor(),
            Some(4) => self.table.screen_cursor(),
            Some(5) => self.doc_table.screen_cursor(),
            Some(6) => self.list.screen_cursor(),
            Some(7) => self.custom.screen_cursor(),
            _ => None,
        }
    }
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut ShowSample,
    ctx: &mut Global,
) -> Result<(), Error> {
    let l0 = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .spacing(1)
    .split(area);

    let base = ctx.show_theme.style_style(Style::CONTAINER_BASE);
    for r in area.top()..area.bottom() {
        for c in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut(Position::new(c, r)) {
                cell.reset();
                cell.set_style(base);
            }
        }
    }

    Text::from("Preview")
        .alignment(Alignment::Center)
        .style(ctx.show_theme.style_style(Style::TITLE))
        .render(l0[1], buf);

    let l_function = Layout::horizontal([
        Constraint::Length(2), //
        Constraint::Length(12),
    ])
    .spacing(1)
    .split(l0[2]);
    let (choice, choice_theme) = Choice::new()
        .items(
            once("")
                .chain([
                    "Dark",     //
                    "Shell",    //
                    "Fallback", //
                ])
                .map(|v| (v.to_string(), v.to_string())),
        )
        .styles(ctx.show_theme.style(WidgetStyle::CHOICE))
        .into_widgets();
    choice.render(l_function[1], buf, &mut state.themes);

    let (menu, menu_popup) = Menubar::new(&StaticMenu {
        menu: &[
            ("_File", &["_Open", "_Save", "\\___", "_Quit"]),
            ("_Help|F1", &["No Help"]),
        ],
    })
    .styles(ctx.show_theme.style(WidgetStyle::MENU))
    .popup_placement(Placement::Below)
    .into_widgets();
    menu.render(l0[3], buf, &mut state.menu);

    Tabbed::new()
        .tabs([
            "Input", "Text", "Dialog", "Split", "Table", "Doc", "List", "Custom",
        ])
        .placement(TabPlacement::Left)
        .block(
            Block::bordered()
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_type(BorderType::Rounded),
        )
        .styles(ctx.show_theme.style(WidgetStyle::TABBED))
        .render(l0[4], buf, &mut state.tabs);

    match state.tabs.selected() {
        Some(0) => {
            let mut area = state.tabs.widget_area;
            area.width += 1;
            sample_data_input::render(area, buf, &mut state.input, ctx)?;
        }
        Some(1) => {
            sample_readability::render(state.tabs.widget_area, buf, &mut state.readability, ctx)?;
        }
        Some(2) => {
            SampleDialog::new(&ctx.show_theme).render(
                state.tabs.widget_area,
                buf,
                &mut state.dialog,
            );
        }
        Some(3) => {
            sample_split::render(state.tabs.widget_area, buf, &mut state.split, ctx)?;
        }
        Some(4) => {
            sample_table::render(false, state.tabs.widget_area, buf, &mut state.table, ctx)?;
        }
        Some(5) => {
            sample_table::render(true, state.tabs.widget_area, buf, &mut state.doc_table, ctx)?;
        }
        Some(6) => {
            sample_list::render(state.tabs.widget_area, buf, &mut state.list, ctx)?;
        }
        Some(7) => {
            sample_custom::render(state.tabs.widget_area, buf, &mut state.custom, ctx)?;
        }
        _ => {}
    };

    StatusLine::new()
        .layout([
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
        ])
        .styles_ext(ctx.show_theme.style(WidgetStyle::STATUSLINE))
        .render(l0[5], buf, &mut state.status);

    choice_theme.render(l_function[1], buf, &mut state.themes);
    menu_popup.render(l0[3], buf, &mut state.menu);

    Ok(())
}

pub fn event(
    event: &crossterm::event::Event,
    state: &mut ShowSample,
    ctx: &mut Global,
) -> Result<Outcome, Error> {
    event_flow!(match state.themes.handle(event, Popup) {
        ChoiceOutcome::Value => {
            let pal = ctx.show_theme.p.clone();
            ctx.show_theme = match state.themes.value().as_str() {
                "Shell" => create_shell("Shell", pal),
                "Fallback" => create_fallback("Fallback", pal),
                _ => create_dark("Dark", pal),
            };
            Outcome::Changed
        }
        r => r.into(),
    });
    event_flow!(state.menu.handle(event, Popup));

    event_flow!(match state.tabs.selected() {
        Some(0) => {
            sample_data_input::event(event, &mut state.input, ctx)?
        }
        Some(1) => {
            sample_readability::event(event, &mut state.readability, ctx)?
        }
        Some(2) => {
            state.dialog.handle(event, Regular)
        }
        Some(3) => {
            sample_split::event(event, &mut state.split, ctx)?
        }
        Some(4) => {
            sample_table::event(event, &mut state.table, ctx)?
        }
        Some(5) => {
            sample_table::event(event, &mut state.doc_table, ctx)?
        }
        Some(6) => {
            sample_list::event(event, &mut state.list, ctx)?
        }
        Some(7) => {
            sample_custom::event(event, &mut state.custom, ctx)?
        }
        _ => {
            Outcome::Continue
        }
    });
    event_flow!(state.tabs.handle(event, Regular));
    Ok(Outcome::Continue)
}
