#![allow(dead_code)]

use crate::adapter::list::{ListS, ListSState};
use anyhow::anyhow;
use crossterm::cursor::{DisableBlinking, EnableBlinking, SetCursorStyle};
use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, KeyCode,
    KeyEvent, KeyEventKind, KeyModifiers,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use log::debug;
use rat_event::{HandleEvent, MouseOnly};
use rat_scrolled::{Scrolled, ScrolledState};
use ratatui::backend::CrosstermBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{ListDirection, StatefulWidget};
use ratatui::{Frame, Terminal};
use std::fs;
use std::io::{stdout, Stdout};
use std::iter::repeat_with;
use std::time::Duration;

pub mod adapter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The given event was not handled at all.
    NotUsed,
    /// The event was handled, no repaint necessary.
    Unchanged,
    /// The event was handled, repaint necessary.
    Changed,
}

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut counter = 0;

    let mut data = Data {
        sample1: repeat_with(|| {
            counter += 1;
            counter
        })
        .take(2000)
        .collect::<Vec<i32>>(),
        sample2: vec![
            "Lorem",
            "ipsum",
            "dolor",
            "sit",
            "amet,",
            "consetetur",
            "sadipscing",
            "elitr,",
            "sed",
            "diam",
            "nonumy",
            "eirmod",
            "tempor",
            "invidunt",
            "ut",
            "labore",
            "et",
            "dolore",
            "magna",
            "aliquyam",
            "erat,",
            "sed",
            "diam",
            "voluptua.",
            "At",
            "vero",
            "eos",
            "et",
            "accusam",
            "et",
        ],
    };

    let mut state = State {
        list1: Default::default(),
        list2: Default::default(),
        list3: Default::default(),
        list4: Default::default(),
    };

    run_ui(&mut data, &mut state)
}

fn setup_logging() -> Result<(), anyhow::Error> {
    fs::remove_file("log.log")?;
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}]\n",
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("log.log")?)
        .apply()?;
    Ok(())
}

impl From<rat_scrolled::event::Outcome<adapter::Outcome>> for Outcome {
    fn from(value: rat_scrolled::event::Outcome<adapter::Outcome>) -> Self {
        match value {
            rat_scrolled::event::Outcome::Inner(i) => match i {
                adapter::Outcome::NotUsed => Outcome::NotUsed,
                adapter::Outcome::Unchanged => Outcome::Unchanged,
                adapter::Outcome::Changed => Outcome::Changed,
            },
            rat_scrolled::event::Outcome::NotUsed => Outcome::NotUsed,
            rat_scrolled::event::Outcome::Unchanged => Outcome::Unchanged,
            rat_scrolled::event::Outcome::Changed => Outcome::Changed,
        }
    }
}

struct Data {
    pub(crate) sample1: Vec<i32>,
    pub(crate) sample2: Vec<&'static str>,
}

struct State {
    pub(crate) list1: ScrolledState<ListSState>,
    pub(crate) list2: ScrolledState<ListSState>,
    pub(crate) list3: ScrolledState<ListSState>,
    pub(crate) list4: ScrolledState<ListSState>,
}

fn run_ui(data: &mut Data, state: &mut State) -> Result<(), anyhow::Error> {
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;
    stdout().execute(EnableBlinking)?;
    stdout().execute(SetCursorStyle::BlinkingBar)?;
    stdout().execute(EnableBracketedPaste)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    repaint_ui(&mut terminal, data, state)?;

    let r = 'l: loop {
        let o = match crossterm::event::poll(Duration::from_millis(10)) {
            Ok(true) => {
                let event = match crossterm::event::read() {
                    Ok(v) => v,
                    Err(e) => break 'l Err(anyhow!(e)),
                };
                match handle_event(event, data, state) {
                    Ok(v) => v,
                    Err(e) => break 'l Err(e),
                }
            }
            Ok(false) => continue,
            Err(e) => break 'l Err(anyhow!(e)),
        };

        match o {
            Outcome::Changed => {
                match repaint_ui(&mut terminal, data, state) {
                    Ok(_) => {}
                    Err(e) => break 'l Err(e),
                };
            }
            _ => {
                // noop
            }
        }
    };

    disable_raw_mode()?;
    stdout().execute(DisableBracketedPaste)?;
    stdout().execute(SetCursorStyle::DefaultUserShape)?;
    stdout().execute(DisableBlinking)?;
    stdout().execute(DisableMouseCapture)?;
    stdout().execute(LeaveAlternateScreen)?;

    r
}

fn repaint_ui(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    data: &mut Data,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    terminal.hide_cursor()?;

    _ = terminal.draw(|frame| {
        repaint_tui(frame, data, state);
    });

    Ok(())
}

fn repaint_tui(frame: &mut Frame<'_>, data: &mut Data, state: &mut State) {
    let area = frame.size();
    let buffer = frame.buffer_mut();

    repaint_lists(area, buffer, data, state);
}

fn handle_event(
    event: crossterm::event::Event,
    data: &mut Data,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    use crossterm::event::Event;
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            ..
        }) => {
            return Err(anyhow!("quit"));
        }
        Event::Resize(_, _) => return Ok(Outcome::Changed),
        _ => {}
    }

    let r = handle_lists(&event, data, state)?;

    Ok(r)
}

fn repaint_lists(area: Rect, buf: &mut Buffer, data: &mut Data, state: &mut State) {
    let l = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Fill(1),
    ])
    .split(area);

    let list1 = Scrolled::new(
        ListS::new(data.sample1.iter().map(|v| v.to_string()))
            .highlight_style(Style::default().reversed()),
    );
    list1.render(l[0], buf, &mut state.list1);

    let list2 = Scrolled::new(
        ListS::new(data.sample2.iter().map(|v| v.to_string()))
            .highlight_style(Style::default().reversed()),
    );
    list2.render(l[1], buf, &mut state.list2);

    let list3 = Scrolled::new(
        ListS::new(data.sample1.iter().map(|v| v.to_string()))
            .highlight_symbol("&")
            .highlight_style(Style::default().on_red())
            .scroll_selection()
            .scroll_padding(2),
    );
    list3.render(l[2], buf, &mut state.list3);

    let list4 = Scrolled::new(
        ListS::new(data.sample2.iter().map(|v| v.to_string()))
            .highlight_style(Style::default().reversed())
            .direction(ListDirection::BottomToTop),
    );
    list4.render(l[3], buf, &mut state.list4);
}

fn handle_lists(
    event: &crossterm::event::Event,
    _data: &mut Data,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    debug!("before {:?}", state.list1);
    match state.list1.handle(event, MouseOnly).into() {
        Outcome::NotUsed => {}
        r => return Ok(r),
    };
    debug!("after {:?}", state.list1);
    match state.list2.handle(event, MouseOnly).into() {
        Outcome::NotUsed => {}
        r => return Ok(r),
    };
    match state.list3.handle(event, MouseOnly).into() {
        Outcome::NotUsed => {}
        r => return Ok(r),
    };
    match state.list4.handle(event, MouseOnly).into() {
        Outcome::NotUsed => {}
        r => return Ok(r),
    };

    Ok(Outcome::NotUsed)
}
