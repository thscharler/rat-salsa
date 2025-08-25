#![allow(dead_code)]

use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use chrono::{Datelike, Local, Months, NaiveDate};
use pure_rust_locales::Locale;
use rat_event::{try_flow, ConsumedEvent, HandleEvent, Regular};
use rat_focus::{Focus, FocusBuilder};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_widget::button::{Button, ButtonState};
use rat_widget::calendar::selection::RangeSelection;
use rat_widget::calendar::{CalendarState, Month, TodayPolicy};
use rat_widget::event::{ButtonOutcome, Outcome};
use rat_widget::statusline::StatusLineState;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, StatefulWidget, Widget};
use ratatui::Frame;
use std::collections::HashMap;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State::new();
    state.menu.focus.set(true);

    run_ui(
        "calendar1",
        |_| {},
        handle_input,
        repaint_input,
        &mut (),
        &mut state,
    )
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

fn repaint_input(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut (),
    _istate: &mut MiniSalsaState,
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
    date_styles.insert(Local::now().date_naive(), THEME.redpink(3));

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
        .style(THEME.limegreen(2))
        .render(l4[2], frame.buffer_mut());

    Month::new()
        .locale(Locale::de_AT_euro)
        .styles(THEME.month_style())
        .title_align(Alignment::Left)
        .day_styles(&date_styles)
        .show_weekdays()
        .block(Block::bordered().borders(Borders::TOP))
        .render(l2[1], frame.buffer_mut(), &mut state.calendar.months[0]);

    Month::new()
        .locale(Locale::de_AT_euro)
        .styles(THEME.month_style())
        .title_align(Alignment::Left)
        .day_styles(&date_styles)
        .show_weekdays()
        .block(Block::bordered().borders(Borders::TOP))
        .render(l2[2], frame.buffer_mut(), &mut state.calendar.months[1]);

    Month::new()
        .locale(Locale::de_AT_euro)
        .styles(THEME.month_style())
        .title_align(Alignment::Left)
        .day_styles(&date_styles)
        .show_weekdays()
        .block(Block::bordered().borders(Borders::TOP))
        .render(l2[3], frame.buffer_mut(), &mut state.calendar.months[2]);

    Button::new("<<<").styles(THEME.button_style()).render(
        l4[1],
        frame.buffer_mut(),
        &mut state.prev,
    );

    Button::new(">>>").styles(THEME.button_style()).render(
        l4[3],
        frame.buffer_mut(),
        &mut state.next,
    );

    MenuLine::new()
        .title("|/\\|")
        .item_parsed("_Quit")
        .title_style(Style::default().black().on_yellow())
        .style(Style::default().black().on_dark_gray())
        .render(l1[5], frame.buffer_mut(), &mut state.menu);

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

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut (),
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let mut focus = focus(state);

    istate.focus_outcome = focus.handle(event, Regular);

    try_flow!(state.calendar.handle(event, Regular));

    try_flow!(match state.menu.handle(event, Regular) {
        MenuOutcome::Activated(0) => {
            istate.quit = true;
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
