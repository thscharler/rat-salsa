#![allow(dead_code)]

use crate::mini_salsa::{MiniSalsaState, run_ui, setup_logging};
use chrono::{Datelike, Local, Months, NaiveDate};
use pure_rust_locales::Locale;
use rat_event::{HandleEvent, Regular, ct_event, try_flow};
use rat_focus::{Focus, FocusBuilder};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_theme4::WidgetStyle;
use rat_widget::button::{Button, ButtonState};
use rat_widget::calendar::selection::RangeSelection;
use rat_widget::calendar::{Calendar3, CalendarState, TodayPolicy};
use rat_widget::event::{ButtonOutcome, CalOutcome, Outcome};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui_core::style::Style;
use ratatui_core::text::Line;
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::{Event, KeyCode, KeyEvent, MediaKeyCode};
use ratatui_widgets::block::Block;
use std::collections::HashMap;
use std::str::FromStr;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State::new();
    state.menu.focus.set(true);

    run_ui("calendar3", init, event, render, &mut state)
}

struct State {
    locale: Locale,

    direction: Direction,
    cal_style: HashMap<NaiveDate, Style>,
    calendar: CalendarState<3, RangeSelection>,

    prev: ButtonState,
    next: ButtonState,

    menu: MenuLineState,
}

impl State {
    fn new() -> Self {
        let locale = sys_locale::get_locale().unwrap_or("POSIX".to_string());
        let locale = locale.replace('-', "_");
        let locale = Locale::from_str(&locale).expect("locale");

        let mut s = Self {
            locale,
            direction: Default::default(),
            cal_style: Default::default(),
            calendar: Default::default(),
            prev: Default::default(),
            next: Default::default(),
            menu: Default::default(),
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

fn init(ctx: &mut MiniSalsaState, state: &mut State) -> Result<(), anyhow::Error> {
    rebuild_cal_style(state, ctx);
    Ok(())
}

fn render(
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let vertical_areas = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .spacing(1)
    .split(area);

    let button_areas = Layout::horizontal([
        Constraint::Length(5),
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Length(5),
    ])
    .spacing(1)
    .split(vertical_areas[0]);

    let mut main_area = Layout::new(
        state.direction, //
        [
            Constraint::Fill(1),
            if state.direction == Direction::Horizontal {
                Constraint::Length(3 * (8 * 3 + 2))
            } else {
                Constraint::Length(3 * (8 + 2))
            },
            Constraint::Fill(1),
        ],
    )
    .split(vertical_areas[1])[1];
    // dead centered
    match state.direction {
        Direction::Horizontal => {
            main_area.y = area.y + area.height.saturating_sub(8) / 2;
        }
        Direction::Vertical => {
            main_area.x = area.x + area.width.saturating_sub(8 * 3) / 2;
        }
    }

    Button::new("<<<")
        .styles(ctx.theme.style(WidgetStyle::BUTTON))
        .render(button_areas[1], buf, &mut state.prev);

    Line::from(year_title(state))
        .alignment(Alignment::Center)
        .style(ctx.theme.p.limegreen(2))
        .render(button_areas[2], buf);

    Button::new(">>>")
        .styles(ctx.theme.style(WidgetStyle::BUTTON))
        .render(button_areas[3], buf, &mut state.next);

    Calendar3::new()
        .direction(state.direction)
        .locale(state.locale)
        .styles(ctx.theme.style(WidgetStyle::MONTH))
        .title_align(Alignment::Left)
        .day_styles(&state.cal_style)
        .show_weekdays()
        .block(Block::bordered())
        .render(main_area, buf, &mut state.calendar);

    MenuLine::new()
        .title("|/\\|")
        .item_parsed("_Quit")
        .title_style(Style::default().black().on_yellow())
        .style(Style::default().black().on_dark_gray())
        .render(vertical_areas[2], buf, &mut state.menu);

    Ok(())
}

fn year_title(state: &mut State) -> String {
    if state.calendar.months[0].start_date().year() != state.calendar.months[2].start_date().year()
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
    }
}

fn focus(state: &State) -> Focus {
    let mut builder = FocusBuilder::default();
    builder.widget(&state.calendar);
    builder.widget(&state.menu);
    builder.build()
}

fn event(
    event: &Event,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    ctx.status[0] = "Ctrl+Q to quit. F1 horizontal | F2 vertical".into();

    let mut focus = focus(state);
    ctx.focus_outcome = focus.handle(event, Regular);

    // hack in mini_salsa to get notification for a theme-change.
    if matches!(
        event,
        Event::Key(KeyEvent {
            code: KeyCode::Media(MediaKeyCode::Play),
            ..
        })
    ) {
        rebuild_cal_style(state, ctx);
    }

    try_flow!(match state.calendar.handle(event, Regular) {
        CalOutcome::Selected | CalOutcome::Changed => {
            rebuild_cal_style(state, ctx);
            Outcome::Changed
        }
        r => r.into(),
    });

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
            rebuild_cal_style(state, ctx);
            Outcome::Changed
        }
        r => r.into(),
    });
    try_flow!(match state.next.handle(event, Regular) {
        ButtonOutcome::Pressed => {
            state.next_month();
            rebuild_cal_style(state, ctx);
            Outcome::Changed
        }
        r => r.into(),
    });

    try_flow!(match event {
        ct_event!(keycode press F(1)) => {
            state.direction = Direction::Horizontal;
            Outcome::Changed
        }
        ct_event!(keycode press F(2)) => {
            state.direction = Direction::Vertical;
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    Ok(Outcome::Continue)
}

fn rebuild_cal_style(state: &mut State, ctx: &mut MiniSalsaState) {
    state.cal_style.clear();

    let mut date = state.calendar.start_date();
    for _ in 0..3 {
        state
            .cal_style
            .insert(date.with_day(10).expect("date"), ctx.theme.p.redpink(0));
        state
            .cal_style
            .insert(date.with_day(20).expect("date"), ctx.theme.p.redpink(0));
        if let Some(d30) = date.with_day(30) {
            state.cal_style.insert(d30, ctx.theme.p.redpink(0));
        } else {
            state
                .cal_style
                .insert(state.calendar.end_date(), ctx.theme.p.redpink(0));
        }

        date = date + Months::new(1);
    }
}
