use anyhow::Error;
use log::error;
use rat_event::try_flow;
use rat_salsa::poll::PollCrossterm;
use rat_salsa::{Control, RunConfig, SalsaAppContext, SalsaContext, run_tui};
use rat_theme4::create_salsa_theme;
use rat_theme4::theme::SalsaTheme;
use rat_widget::event::{HandleEvent, Regular, ct_event};
use rat_widget::scrolled::Scroll;
use rat_widget::view::{View, ViewState};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Position, Rect};
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::Block;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let config = Config::default();
    let theme = create_salsa_theme("Imperial Dark");
    let mut global = Global::new(config, theme);
    let mut state = Minimal::default();

    run_tui(
        init,
        render,
        event,
        error,
        &mut global,
        &mut state,
        RunConfig::default()?.poll(PollCrossterm),
    )?;

    Ok(())
}

/// Globally accessible data/state.
pub struct Global {
    ctx: SalsaAppContext<AppEvent, Error>,
    pub cfg: Config,
    pub theme: SalsaTheme,
}

impl SalsaContext<AppEvent, Error> for Global {
    fn set_salsa_ctx(&mut self, app_ctx: SalsaAppContext<AppEvent, Error>) {
        self.ctx = app_ctx;
    }

    #[inline(always)]
    fn salsa_ctx(&self) -> &SalsaAppContext<AppEvent, Error> {
        &self.ctx
    }
}

impl Global {
    pub fn new(cfg: Config, theme: SalsaTheme) -> Self {
        Self {
            ctx: Default::default(),
            cfg,
            theme,
        }
    }
}

/// Configuration.
#[derive(Debug, Default)]
pub struct Config {}

/// Application wide messages.
#[derive(Debug)]
pub enum AppEvent {
    Event(Event),
}

impl From<Event> for AppEvent {
    fn from(value: Event) -> Self {
        Self::Event(value)
    }
}

#[derive(Debug, Default)]
pub struct Minimal {
    pub view: ViewState,
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut Minimal,
    _ctx: &mut Global,
) -> Result<(), Error> {
    let layout = Layout::vertical([
        Constraint::Length(5),
        Constraint::Fill(1), //
        Constraint::Length(5),
    ])
    .split(area);

    let mut vbuf = View::new()
        .block(Block::bordered())
        .vscroll(Scroll::new())
        .hscroll(Scroll::new())
        .layout(Rect::new(0, 0, 50, 50))
        .into_buffer(layout[1], &mut state.view);

    let link_str = "\u{1B}]8;;https://github.com/ratatui/\u{1B}\\ratatui\u{1B}]8;;\u{1B}\\";

    vbuf.buffer()
        .cell_mut(Position::new(4, 5))
        .expect("cell")
        .set_symbol(link_str);

    for c in 5..12 {
        vbuf.buffer()
            .cell_mut(Position::new(c, 5))
            .expect("cell")
            .skip = true;
    }

    vbuf.finish(buf, &mut state.view);

    Ok(())
}

pub fn init(_state: &mut Minimal, _ctx: &mut Global) -> Result<(), Error> {
    Ok(())
}

pub fn event(
    event: &AppEvent,
    state: &mut Minimal,
    _ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    match event {
        AppEvent::Event(event) => {
            try_flow!(match &event {
                ct_event!(resized) => Control::Changed,
                ct_event!(key press CONTROL-'q') => Control::Quit,
                _ => Control::Continue,
            });

            try_flow!(state.view.handle(event, Regular));

            Ok(Control::Continue)
        }
    }
}

pub fn error(
    event: Error,
    _state: &mut Minimal,
    _ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    error!("{:?}", event);
    Ok(Control::Continue)
}

fn setup_logging() -> Result<(), Error> {
    let log_path = PathBuf::from(".");
    let log_file = log_path.join("log.log");
    _ = fs::remove_file(&log_file);
    fern::Dispatch::new()
        .format(|out, message, _record| {
            out.finish(format_args!("{}", message)) //
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file(&log_file)?)
        .apply()?;
    Ok(())
}
