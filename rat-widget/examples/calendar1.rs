#![allow(dead_code)]

use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use chrono::{Datelike, Local, Months, NaiveDate};
use pure_rust_locales::Locale;
use rat_event::{ct_event, ConsumedEvent, HandleEvent, Regular};
use rat_focus::{Focus, FocusBuilder};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_widget::button::{Button, ButtonState};
use rat_widget::calendar::{Month, MonthState, MultiMonth};
use rat_widget::event::{ButtonOutcome, Outcome};
use rat_widget::statusline::StatusLineState;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, StatefulWidget, Widget};
use ratatui::Frame;
use std::cmp::max;
use std::collections::HashMap;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State::new();
    state.menu.focus.set(true);

    run_ui(
        "calendar1",
        handle_input,
        repaint_input,
        &mut (),
        &mut state,
    )
}

struct State {
    months: [MonthState; 3],

    prev: ButtonState,
    next: ButtonState,

    menu: MenuLineState,
    status: StatusLineState,
}

impl State {
    fn new() -> Self {
        let mut s = Self {
            months: Default::default(),
            prev: Default::default(),
            next: Default::default(),
            menu: Default::default(),
            status: Default::default(),
        };

        let today = Local::now().date_naive().with_day(1).expect("date");
        s.months[0].set_start_date(today.checked_sub_months(Months::new(1)).expect("date"));
        s.months[1].set_start_date(today);
        s.months[2].set_start_date(today.checked_add_months(Months::new(1)).expect("date"));
        s
    }

    fn set_start_date(&mut self, date: NaiveDate) {
        self.months[0].set_start_date(date.checked_sub_months(Months::new(1)).expect("date"));
        self.months[1].set_start_date(date);
        self.months[2].set_start_date(date.checked_add_months(Months::new(1)).expect("date"));
    }

    fn start_date(&self) -> NaiveDate {
        self.months[1].start_date()
    }

    fn prev_month(&mut self) {
        let prev = self
            .start_date()
            .checked_sub_months(Months::new(1))
            .expect("date");
        self.set_start_date(prev);
    }

    fn next_month(&mut self) {
        let prev = self
            .start_date()
            .checked_add_months(Months::new(1))
            .expect("date");
        self.set_start_date(prev);
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

    let title = if state.months[0].start_date().year() != state.months[2].start_date().year() {
        format!(
            "{} / {}",
            state.months[0].start_date().format("%Y").to_string(),
            state.months[2].start_date().format("%Y").to_string()
        )
    } else {
        state.months[0].start_date().format("%Y").to_string()
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
        .day_selection()
        .week_selection()
        .show_weekdays()
        .block(Block::bordered().borders(Borders::TOP))
        .render(l2[1], frame.buffer_mut(), &mut state.months[0]);

    Month::new()
        .locale(Locale::de_AT_euro)
        .styles(THEME.month_style())
        .title_align(Alignment::Left)
        .day_styles(&date_styles)
        .day_selection()
        .week_selection()
        .show_weekdays()
        .block(Block::bordered().borders(Borders::TOP))
        .render(l2[2], frame.buffer_mut(), &mut state.months[1]);

    Month::new()
        .locale(Locale::de_AT_euro)
        .styles(THEME.month_style())
        .title_align(Alignment::Left)
        .day_styles(&date_styles)
        .day_selection()
        .week_selection()
        .show_weekdays()
        .block(Block::bordered().borders(Borders::TOP))
        .render(l2[3], frame.buffer_mut(), &mut state.months[2]);

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
    let mut fb = FocusBuilder::default();
    fb.widget(&state.months[0])
        .widget(&state.months[1])
        .widget(&state.months[2])
        .widget(&state.menu);
    fb.build()
}

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut (),
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let mut focus = focus(state);
    let f = focus.handle(event, Regular);

    let r: Outcome = state
        .months
        .as_mut_slice()
        .handle(event, MultiMonth(&focus, 1))
        .into();

    let r = r.or_else(|| match state.menu.handle(event, Regular) {
        MenuOutcome::Activated(0) => {
            istate.quit = true;
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    let r = r.or_else(|| match state.prev.handle(event, Regular) {
        ButtonOutcome::Pressed => {
            state.prev_month();
            Outcome::Changed
        }
        r => r.into(),
    });
    let r = r.or_else(|| match state.next.handle(event, Regular) {
        ButtonOutcome::Pressed => {
            state.next_month();
            Outcome::Changed
        }
        r => r.into(),
    });
    let r = r.or_else(|| match event {
        ct_event!(keycode press PageUp) => {
            state.prev_month();
            Outcome::Changed
        }
        ct_event!(keycode press PageDown) => {
            state.next_month();
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    Ok(max(f, r))
}
