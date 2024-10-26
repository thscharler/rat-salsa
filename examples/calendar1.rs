#![allow(dead_code)]

use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use chrono::{Datelike, Months, NaiveDate};
use pure_rust_locales::Locale;
use rat_event::{ct_event, ConsumedEvent, HandleEvent, Regular};
use rat_focus::{Focus, FocusBuilder};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_widget::button::{Button, ButtonOutcome, ButtonState};
use rat_widget::calendar::{Month, MonthState};
use rat_widget::event::{CalOutcome, Outcome};
use rat_widget::statusline::StatusLineState;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, StatefulWidget, Widget};
use ratatui::Frame;
use std::cmp::max;
use std::collections::HashMap;
use std::mem;
use std::ops::{Add, Sub};

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {
        date: chrono::offset::Local::now().date_naive(),
    };

    let mut state = State {
        cal_b: Default::default(),
        cal0: Default::default(),
        cal1: Default::default(),
        cal2: Default::default(),
        cal_a: Default::default(),
        prev: Default::default(),
        next: Default::default(),
        menu: Default::default(),
        status: Default::default(),
    };
    state.menu.focus.set(true);

    run_ui(
        "calendar1",
        handle_input,
        repaint_input,
        &mut data,
        &mut state,
    )
}

struct Data {
    date: NaiveDate,
}

struct State {
    cal_b: MonthState,
    cal0: MonthState,
    cal1: MonthState,
    cal2: MonthState,
    cal_a: MonthState,

    prev: ButtonState,
    next: ButtonState,

    menu: MenuLineState,
    status: StatusLineState,
}

impl Data {
    fn prev_month(&mut self) {
        self.date = self.date.sub(Months::new(1));
    }

    fn next_month(&mut self) {
        self.date = self.date.add(Months::new(1));
    }
}

impl State {
    fn prev_month(&mut self) {
        mem::swap(&mut self.cal_a, &mut self.cal2);
        mem::swap(&mut self.cal2, &mut self.cal1);
        mem::swap(&mut self.cal1, &mut self.cal0);
        mem::swap(&mut self.cal0, &mut self.cal_b);
        self.cal_b = Default::default();
    }

    fn next_month(&mut self) {
        mem::swap(&mut self.cal_b, &mut self.cal0);
        mem::swap(&mut self.cal0, &mut self.cal1);
        mem::swap(&mut self.cal1, &mut self.cal2);
        mem::swap(&mut self.cal2, &mut self.cal_a);
        self.cal_a = Default::default();
    }
}

fn repaint_input(
    frame: &mut Frame<'_>,
    area: Rect,
    data: &mut Data,
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
    date_styles.insert(chrono::offset::Local::now().date_naive(), THEME.redpink(3));

    let date1 = data.date.with_day(1).expect("date");
    let date_b = date1.sub(Months::new(2));
    let date0 = date1.sub(Months::new(1));
    let date2 = date1.add(Months::new(1));
    let date_a = date1.add(Months::new(2));

    state.cal_b.start_date = date_b.with_day(1).unwrap();
    state.cal_a.start_date = date_a.with_day(1).unwrap();

    let title = if date0.year() != date2.year() {
        format!(
            "{} / {}",
            date0.format("%Y").to_string(),
            date2.format("%Y").to_string()
        )
    } else {
        date0.format("%Y").to_string()
    };

    Line::from(title)
        .alignment(Alignment::Center)
        .style(THEME.limegreen(2))
        .render(l4[2], frame.buffer_mut());

    Month::new()
        .date(date0)
        .locale(Locale::de_AT_euro)
        .styles(THEME.month_style())
        .title_align(Alignment::Left)
        .day_styles(&date_styles)
        .day_selection()
        .week_selection()
        .show_weekdays()
        .block(Block::bordered().borders(Borders::TOP))
        .render(l2[1], frame.buffer_mut(), &mut state.cal0);

    Month::new()
        .date(date1)
        .locale(Locale::de_AT_euro)
        .styles(THEME.month_style())
        .title_align(Alignment::Left)
        .day_styles(&date_styles)
        .day_selection()
        .week_selection()
        .show_weekdays()
        .block(Block::bordered().borders(Borders::TOP))
        .render(l2[2], frame.buffer_mut(), &mut state.cal1);

    Month::new()
        .date(date2)
        .locale(Locale::de_AT_euro)
        .styles(THEME.month_style())
        .title_align(Alignment::Left)
        .day_styles(&date_styles)
        .day_selection()
        .week_selection()
        .show_weekdays()
        .block(Block::bordered().borders(Borders::TOP))
        .render(l2[3], frame.buffer_mut(), &mut state.cal2);

    Button::new("<<<")
        .styles(THEME.button_style_no_border())
        .render(l4[1], frame.buffer_mut(), &mut state.prev);

    Button::new(">>>")
        .styles(THEME.button_style_no_border())
        .render(l4[3], frame.buffer_mut(), &mut state.next);

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
    fb.widget(&state.cal0)
        .widget(&state.cal1)
        .widget(&state.cal2)
        .widget(&state.menu);
    fb.build()
}

fn handle_input(
    event: &crossterm::event::Event,
    data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let mut focus = focus(state);
    let f = focus.handle(event, Regular);

    let mut all_months = [
        &mut state.cal_b,
        &mut state.cal0,
        &mut state.cal1,
        &mut state.cal2,
        &mut state.cal_a,
    ];

    let r: Outcome = match all_months.handle(event, Regular) {
        CalOutcome::Month(0) => {
            data.prev_month();
            state.prev_month();
            // renew focus
            let focus = crate::focus(state);
            focus.focus(&state.cal0);

            Outcome::Changed
        }
        CalOutcome::Month(1) => {
            focus.focus(&state.cal0);
            Outcome::Changed
        }
        CalOutcome::Month(2) => {
            focus.focus(&state.cal1);
            Outcome::Changed
        }
        CalOutcome::Month(3) => {
            focus.focus(&state.cal2);
            Outcome::Changed
        }
        CalOutcome::Month(4) => {
            data.next_month();
            state.next_month();
            // renew focus
            let focus = crate::focus(state);
            focus.focus(&state.cal2);
            Outcome::Changed
        }
        r => r.into(),
    };

    let r = r.or_else(|| match state.menu.handle(event, Regular) {
        MenuOutcome::Activated(0) => {
            istate.quit = true;
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    let r = r.or_else(|| match state.prev.handle(event, Regular) {
        ButtonOutcome::Pressed => {
            data.prev_month();
            state.prev_month();
            Outcome::Changed
        }
        r => r.into(),
    });
    let r = r.or_else(|| match state.next.handle(event, Regular) {
        ButtonOutcome::Pressed => {
            data.next_month();
            state.next_month();
            Outcome::Changed
        }
        r => r.into(),
    });
    let r = r.or_else(|| match event {
        ct_event!(keycode press PageUp) => {
            data.prev_month();
            state.prev_month();
            Outcome::Changed
        }
        ct_event!(keycode press PageDown) => {
            data.next_month();
            state.next_month();
            Outcome::Changed
        }
        ct_event!(scroll up for _x,_y) => {
            data.prev_month();
            state.prev_month();
            Outcome::Changed
        }
        ct_event!(scroll down for _x,_y) => {
            data.next_month();
            state.next_month();
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    Ok(max(f, r))
}
