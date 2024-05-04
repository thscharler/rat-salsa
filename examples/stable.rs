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
use rat_scrolled::adapter::table::{TableS, TableSState};
use rat_scrolled::events::{DefaultKeys, HandleEvent, Outcome};
use rat_scrolled::scrolled::{Scrolled, ScrolledState};
use ratatui::backend::CrosstermBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::{Cell, Row, StatefulWidget};
use ratatui::{Frame, Terminal};
use std::fs;
use std::io::{stdout, Stdout};
use std::iter::repeat_with;
use std::time::Duration;

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
        table1: Default::default(),
        table2: Default::default(),
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

struct Data {
    pub(crate) sample1: Vec<i32>,
    pub(crate) sample2: Vec<&'static str>,
}

struct State {
    pub(crate) table1: ScrolledState<TableSState>,
    pub(crate) table2: ScrolledState<TableSState>,
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
    let l = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).split(area);

    let table1 = Scrolled::new(
        TableS::new(
            data.sample1.iter().map(|v| {
                Row::new([
                    Cell::new(Text::from(v.to_string())),
                    Cell::new(Text::from(v.to_string())),
                    Cell::new(Text::from(v.to_string())),
                    Cell::new(Text::from(v.to_string())),
                    Cell::new(Text::from(v.to_string())),
                    Cell::new(Text::from(v.to_string())),
                    Cell::new(Text::from(v.to_string())),
                    Cell::new(Text::from(v.to_string())),
                    Cell::new(Text::from(v.to_string())),
                ])
            }),
            [
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
            ],
        )
        .highlight_style(Style::default().on_red())
        .scroll_selection()
        .scroll_by(1),
    );
    table1.render(l[0], buf, &mut state.table1);

    let table2 = Scrolled::new(TableS::new(
        data.sample2.iter().map(|v| {
            Row::new([
                Cell::new(Text::from(*v)),
                Cell::new(Text::from(*v)),
                Cell::new(Text::from(*v)),
                Cell::new(Text::from(*v)),
                Cell::new(Text::from(*v)),
            ])
        }),
        [
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
        ],
    ));
    table2.render(l[1], buf, &mut state.table2);
}

fn handle_lists(
    event: &crossterm::event::Event,
    _data: &mut Data,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    match HandleEvent::handle(&mut state.table1, event, true, DefaultKeys) {
        Outcome::NotUsed => {}
        r => return Ok(r),
    };
    match HandleEvent::handle(&mut state.table2, event, false, DefaultKeys) {
        Outcome::NotUsed => {}
        r => return Ok(r),
    };

    Ok(Outcome::NotUsed)
}
