#![allow(unreachable_pub)]

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
use log::error;
use rat_event::{ConsumedEvent, Outcome};
use rat_input::button::ButtonStyle;
use rat_input::msgdialog;
use rat_input::msgdialog::{MsgDialog, MsgDialogState};
use rat_input::statusline::{StatusLine, StatusLineState};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::Style;
use ratatui::style::Stylize;
use ratatui::{Frame, Terminal};
use std::fs;
use std::io::{stdout, Stdout};
use std::time::{Duration, SystemTime};

pub struct MiniSalsaState {
    pub status: StatusLineState,
    pub msg: MsgDialogState,
}

impl Default for MiniSalsaState {
    fn default() -> Self {
        let mut s = Self {
            status: Default::default(),
            msg: Default::default(),
        };
        s.status.status(0, "Ctrl-Q to quit.");
        s
    }
}

pub fn run_ui<Data, State>(
    handle: fn(
        &crossterm::event::Event,
        data: &mut Data,
        istate: &mut MiniSalsaState,
        state: &mut State,
    ) -> Result<Outcome, anyhow::Error>,
    repaint: fn(
        &mut Frame<'_>,
        Rect,
        &mut Data,
        &mut MiniSalsaState,
        &mut State,
    ) -> Result<(), anyhow::Error>,
    data: &mut Data,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;
    stdout().execute(EnableBlinking)?;
    stdout().execute(SetCursorStyle::BlinkingBar)?;
    stdout().execute(EnableBracketedPaste)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut istate = MiniSalsaState::default();

    repaint_ui(&mut terminal, repaint, data, &mut istate, state)?;

    let r = 'l: loop {
        let o = match crossterm::event::poll(Duration::from_millis(10)) {
            Ok(true) => {
                let event = match crossterm::event::read() {
                    Ok(v) => v,
                    Err(e) => break 'l Err(anyhow!(e)),
                };
                match handle_event(handle, event, data, &mut istate, state) {
                    Ok(v) => v,
                    Err(e) => break 'l Err(e),
                }
            }
            Ok(false) => continue,
            Err(e) => break 'l Err(anyhow!(e)),
        };

        match o {
            Outcome::Changed => {
                match repaint_ui(&mut terminal, repaint, data, &mut istate, state) {
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

fn repaint_ui<Data, State>(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    repaint: fn(
        &mut Frame<'_>,
        Rect,
        &mut Data,
        &mut MiniSalsaState,
        &mut State,
    ) -> Result<(), anyhow::Error>,
    data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    terminal.hide_cursor()?;

    _ = terminal.draw(|frame| {
        match repaint_tui(frame, repaint, data, istate, state) {
            Ok(_) => {}
            Err(e) => {
                error!("{:?}", e)
            }
        };
    });

    Ok(())
}

fn repaint_tui<Data, State>(
    frame: &mut Frame<'_>,
    repaint: fn(
        &mut Frame<'_>,
        Rect,
        &mut Data,
        &mut MiniSalsaState,
        &mut State,
    ) -> Result<(), anyhow::Error>,
    data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let t0 = SystemTime::now();
    let area = frame.size();

    let l1 = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

    repaint(frame, l1[0], data, istate, state)?;

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

    if istate.msg.active {
        let msgd = MsgDialog::default()
            .style(Style::default().white().on_blue())
            .button_style(ButtonStyle {
                style: Style::default().blue().on_white(),
                ..Default::default()
            });
        frame.render_stateful_widget(msgd, area, &mut istate.msg);
    }

    let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
    istate
        .status
        .status(1, format!("Render {:?}", el).to_string());
    frame.render_stateful_widget(status1, l1[1], &mut istate.status);

    Ok(())
}

fn handle_event<Data, State>(
    handle: fn(
        &crossterm::event::Event,
        data: &mut Data,
        istate: &mut MiniSalsaState,
        state: &mut State,
    ) -> Result<Outcome, anyhow::Error>,
    event: crossterm::event::Event,
    data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let t0 = SystemTime::now();

    let r = 'h: {
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

        if istate.msg.active() {
            let r = msgdialog::handle_dialog_events(&mut istate.msg, &event);
            if r.is_consumed() {
                break 'h r;
            }
        }

        let r = handle(&event, data, istate, state)?;

        r
    };

    let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
    istate
        .status
        .status(2, format!("Handle {:?}", el).to_string());

    Ok(r)
}

pub fn setup_logging() -> Result<(), anyhow::Error> {
    _ = fs::remove_file("../../../log.log");
    fern::Dispatch::new()
        .format(|out, message, _record| out.finish(format_args!("{}", message)))
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("../../../log.log")?)
        .apply()?;
    Ok(())
}
