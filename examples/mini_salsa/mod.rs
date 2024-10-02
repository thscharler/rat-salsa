#![allow(unreachable_pub)]
#![allow(dead_code)]

use crate::mini_salsa::theme::THEME;
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
use rat_event::Outcome;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::Widget;
use ratatui::{Frame, Terminal};
use std::fs;
use std::io::{stdout, Stdout};
use std::time::{Duration, SystemTime};

pub struct MiniSalsaState {
    pub status: [String; 3],
    pub quit: bool,
}

impl Default for MiniSalsaState {
    fn default() -> Self {
        let mut s = Self {
            status: Default::default(),
            quit: false,
        };
        s.status[0] = "Ctrl-Q to quit.".into();
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
            Err(e) => {
                istate.status[0] = format!("{}", e);
                Outcome::Changed
            }
        };

        if istate.quit {
            break 'l Ok(());
        }

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
    let area = frame.area();

    let l1 = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

    repaint(frame, l1[0], data, istate, state)?;

    let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
    istate.status[1] = format!("Render {:?}", el).to_string();

    let l_status = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(17),
        Constraint::Length(17),
    ])
    .split(l1[1]);

    Line::from(istate.status[0].as_str())
        .style(THEME.black(2))
        .render(l_status[0], frame.buffer_mut());
    Line::from(istate.status[1].as_str())
        .style(THEME.deepblue(0))
        .render(l_status[1], frame.buffer_mut());
    Line::from(istate.status[2].as_str())
        .style(THEME.deepblue(1))
        .render(l_status[2], frame.buffer_mut());

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

    let r = {
        use crossterm::event::Event;
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                ..
            }) => {
                istate.quit = true;
                return Ok(Outcome::Changed);
            }
            Event::Resize(_, _) => return Ok(Outcome::Changed),
            _ => {}
        }

        let r = handle(&event, data, istate, state)?;

        r
    };

    let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
    istate.status[2] = format!("Handle {:?}", el).to_string();

    Ok(r)
}

pub fn setup_logging() -> Result<(), anyhow::Error> {
    _ = fs::remove_file("log.log");
    fern::Dispatch::new()
        .format(|out, message, _record| out.finish(format_args!("{}", message)))
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("log.log")?)
        .apply()?;
    Ok(())
}

pub fn layout_grid<const X: usize, const Y: usize>(
    area: Rect,
    horizontal: Layout,
    vertical: Layout,
) -> [[Rect; Y]; X] {
    let hori = horizontal.split(Rect::new(area.x, 0, area.width, 0));
    let vert = vertical.split(Rect::new(0, area.y, 0, area.height));

    let mut res = [[Rect::default(); Y]; X];
    for x in 0..X {
        let coldata = &mut res[x];
        for y in 0..Y {
            coldata[y].x = hori[x].x;
            coldata[y].width = hori[x].width;
            coldata[y].y = vert[y].y;
            coldata[y].height = vert[y].height;
        }
    }

    res
}

pub mod theme;
