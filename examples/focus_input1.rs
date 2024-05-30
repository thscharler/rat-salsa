use crate::adapter::textinputf::{TextInputF, TextInputFState};
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
use rat_event::ConsumedEvent;
use rat_focus::{Focus, HasFocusFlag};
use rat_input::event::{FocusKeys, HandleEvent, Outcome};
use rat_input::layout_edit::{layout_edit, EditConstraint};
use rat_input::statusline::{StatusLine, StatusLineState};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Span;
use ratatui::{Frame, Terminal};
use std::fs;
use std::io::{stdout, Stdout};
use std::time::{Duration, SystemTime};

pub mod adapter;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        input1: Default::default(),
        input2: Default::default(),
        input3: Default::default(),
        input4: Default::default(),
        status: Default::default(),
    };
    state.input1.focus.set();
    state.status.status(0, "Ctrl+Q to quit.");

    run_ui(&mut data, &mut state)
}

fn setup_logging() -> Result<(), anyhow::Error> {
    fs::remove_file("log.log")?;
    fern::Dispatch::new()
        .format(|out, message, _record| out.finish(format_args!("{}", message)))
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("log.log")?)
        .apply()?;
    Ok(())
}

struct Data {}

struct State {
    pub(crate) input1: TextInputFState,
    pub(crate) input2: TextInputFState,
    pub(crate) input3: TextInputFState,
    pub(crate) input4: TextInputFState,
    pub(crate) status: StatusLineState,
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
    let t0 = SystemTime::now();
    let area = frame.size();

    let l1 = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

    repaint_input(frame, l1[0], data, state);

    let status1 = StatusLine::new()
        .layout([
            Constraint::Fill(1),
            Constraint::Length(17),
            Constraint::Length(17),
        ])
        .styles([
            Style::default().black().on_dark_gray(),
            Style::default().white().on_blue(),
            Style::default().white().on_light_blue(),
        ]);

    let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
    state
        .status
        .status(1, format!("Render {:?}", el).to_string());
    frame.render_stateful_widget(status1, l1[1], &mut state.status);
}

fn handle_event(
    event: crossterm::event::Event,
    data: &mut Data,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let t0 = SystemTime::now();

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

    let r = handle_input(&event, data, state)?;
    debug!("handle_input {:?}", r);

    let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
    state
        .status
        .status(2, format!("Handle {:?}", el).to_string());

    Ok(r)
}

fn repaint_input(frame: &mut Frame<'_>, area: Rect, _data: &mut Data, state: &mut State) {
    let l0 = Layout::horizontal([Constraint::Length(25), Constraint::Fill(1)]).split(area);

    let le = layout_edit(
        l0[0],
        &[
            EditConstraint::Label("Text 1"),
            EditConstraint::Widget(15),
            EditConstraint::Label("Text 2"),
            EditConstraint::Widget(15),
            EditConstraint::Label("Text 3"),
            EditConstraint::Widget(15),
            EditConstraint::Label("Text 4"),
            EditConstraint::Widget(15),
        ],
    );
    let mut le = le.iter();

    frame.render_widget(Span::from("Text 1"), le.label());
    let input1 = TextInputF::default()
        .style(Style::default().black().on_green())
        .focus_style(Style::default().black().on_light_blue());
    frame.render_stateful_widget(input1, le.widget(), &mut state.input1);
    if let Some((x, y)) = state.input1.screen_cursor() {
        if state.input1.is_focused() {
            frame.set_cursor(x, y);
        }
    }

    frame.render_widget(Span::from("Text 2"), le.label());
    let input2 = TextInputF::default()
        .style(Style::default().black().on_green())
        .focus_style(Style::default().black().on_light_blue());
    frame.render_stateful_widget(input2, le.widget(), &mut state.input2);
    if let Some((x, y)) = state.input2.screen_cursor() {
        if state.input2.is_focused() {
            frame.set_cursor(x, y);
        }
    }

    frame.render_widget(Span::from("Text 3"), le.label());
    let input3 = TextInputF::default()
        .style(Style::default().black().on_green())
        .focus_style(Style::default().black().on_light_blue());
    frame.render_stateful_widget(input3, le.widget(), &mut state.input3);
    if let Some((x, y)) = state.input3.screen_cursor() {
        if state.input3.is_focused() {
            frame.set_cursor(x, y);
        }
    }

    frame.render_widget(Span::from("Text 4"), le.label());
    let input4 = TextInputF::default()
        .style(Style::default().black().on_green())
        .focus_style(Style::default().black().on_light_blue());
    frame.render_stateful_widget(input4, le.widget(), &mut state.input4);
    if let Some((x, y)) = state.input4.screen_cursor() {
        if state.input4.is_focused() {
            frame.set_cursor(x, y);
        }
    }
}

fn focus_input(state: &mut State) -> Focus<'_> {
    Focus::new(&[&state.input1, &state.input2, &state.input3, &state.input4])
}

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut Data,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let f = focus_input(state).handle(event, FocusKeys);

    let mut r: Outcome = state.input1.handle(event, FocusKeys).into();
    if r.is_consumed() {
        return Ok(r | f);
    }
    r = state.input2.handle(event, FocusKeys).into();
    if r.is_consumed() {
        return Ok(r | f);
    }
    r = state.input3.handle(event, FocusKeys).into();
    if r.is_consumed() {
        return Ok(r | f);
    }
    r = state.input4.handle(event, FocusKeys).into();
    if r.is_consumed() {
        return Ok(r | f);
    }

    Ok(r | f)
}
