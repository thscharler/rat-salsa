#![allow(dead_code)]

use crate::mini_salsa::{MiniSalsaState, mock_init, run_ui, setup_logging};
use chrono::{Datelike, Local, Months, NaiveDate};
use pure_rust_locales::Locale;
use rat_event::{HandleEvent, Regular, try_flow};
use rat_focus::{Focus, FocusBuilder};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_theme4::WidgetStyle;
use rat_widget::button::{Button, ButtonState};
use rat_widget::calendar::selection::RangeSelection;
use rat_widget::calendar::{CalendarState, Month, TodayPolicy};
use rat_widget::event::{ButtonOutcome, Outcome};
use rat_widget::statusline::StatusLineState;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Alignment, Constraint, Layout, Rect};
use ratatui_core::style::Style;
use ratatui_core::text::Line;
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::Block;
use ratatui_widgets::borders::Borders;
use std::collections::HashMap;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State::new();
    state.menu.focus.set(true);

    run_ui("calendar1", mock_init, event, render, &mut state)
}

struct State {
    calendar: CalendarState<3, RangeSelection>,

    prev: ButtonState,
    next: ButtonState,

    menu: MenuLineState,
    status: StatusLineState,
}

impl State {
    fn new() -> Self {
        let mut s = Self {
            calendar: Default::default(),
            prev: Default::default(),
            next: Default::default(),
            menu: Default::default(),
            status: Default::default(),
        };

        let today = Local::now().date_naive();
        s.calendar.set_today_policy(TodayPolicy::Index(1));
        s.calendar.set_primary_idx(1);
        s.calendar.set_start_date(today - Months::new(1));
        s.calendar.set_step(1);
        s
    }

    fn start_date(&self) -> NaiveDate {
        self.calendar.start_date()
    }

    fn prev_month(&mut self) {
        self.calendar.scroll_back(1);
    }

    fn next_month(&mut self) {
        self.calendar.scroll_forward(1);
    }
}

fn render(
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);

    let l2 = Layout::horizontal([
        Constraint::Length(5),
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Length(5),
    ])
    .spacing(1)
    .split(l1[3]);

    let l4 = Layout::horizontal([
        Constraint::Length(5),
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Length(5),
    ])
    .spacing(1)
    .split(l1[1]);

    let mut date_styles = HashMap::new();
    date_styles.insert(
        NaiveDate::from_ymd_opt(2024, 9, 1).expect("some"),
        Style::default().red(),
    );
    date_styles.insert(Local::now().date_naive(), ctx.theme.p.redpink(3));

    let title = if state.calendar.months[0].start_date().year()
        != state.calendar.months[2].start_date().year()
    {
        format!(
            "{} / {}",
            state.calendar.months[0]
                .start_date()
                .format("%Y")
                .to_string(),
            state.calendar.months[2]
                .start_date()
                .format("%Y")
                .to_string()
        )
    } else {
        state.calendar.months[0]
            .start_date()
            .format("%Y")
            .to_string()
    };

    Line::from(title)
        .alignment(Alignment::Center)
        .style(ctx.theme.p.limegreen(2))
        .render(l4[2], buf);

    Month::new()
        .locale(Locale::de_AT_euro)
        .styles(ctx.theme.style(WidgetStyle::MONTH))
        .title_align(Alignment::Left)
        .day_styles(&date_styles)
        .show_weekdays()
        .block(Block::bordered().borders(Borders::TOP))
        .render(l2[1], buf, &mut state.calendar.months[0]);

    Month::new()
        .locale(Locale::de_AT_euro)
        .styles(ctx.theme.style(WidgetStyle::MONTH))
        .title_align(Alignment::Left)
        .day_styles(&date_styles)
        .show_weekdays()
        .block(Block::bordered().borders(Borders::TOP))
        .render(l2[2], buf, &mut state.calendar.months[1]);

    Month::new()
        .locale(Locale::de_AT_euro)
        .styles(ctx.theme.style(WidgetStyle::MONTH))
        .title_align(Alignment::Left)
        .day_styles(&date_styles)
        .show_weekdays()
        .block(Block::bordered().borders(Borders::TOP))
        .render(l2[3], buf, &mut state.calendar.months[2]);

    Button::new("<<<")
        .styles(ctx.theme.style(WidgetStyle::BUTTON))
        .render(l4[1], buf, &mut state.prev);

    Button::new(">>>")
        .styles(ctx.theme.style(WidgetStyle::BUTTON))
        .render(l4[3], buf, &mut state.next);

    MenuLine::new()
        .title("|/\\|")
        .item_parsed("_Quit")
        .title_style(Style::default().black().on_yellow())
        .style(Style::default().black().on_dark_gray())
        .render(l1[5], buf, &mut state.menu);

    Ok(())
}

fn focus(state: &State) -> Focus {
    let mut builder = FocusBuilder::default();
    builder.widget(&state.calendar);
    builder.widget(&state.menu);
    let f = builder.build();
    f.enable_log();
    f
}

fn event(
    event: &Event,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let mut focus = focus(state);

    ctx.focus_outcome = focus.handle(event, Regular);

    try_flow!(state.calendar.handle(event, Regular));

    try_flow!(match state.menu.handle(event, Regular) {
        MenuOutcome::Activated(0) => {
            ctx.quit = true;
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    try_flow!(match state.prev.handle(event, Regular) {
        ButtonOutcome::Pressed => {
            state.prev_month();
            Outcome::Changed
        }
        r => r.into(),
    });
    try_flow!(match state.next.handle(event, Regular) {
        ButtonOutcome::Pressed => {
            state.next_month();
            Outcome::Changed
        }
        r => r.into(),
    });

    Ok(Outcome::Continue)
}
