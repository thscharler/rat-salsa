use anyhow::Error;
use crossterm::event::Event;
use rat_salsa::event::ct_event;
use rat_salsa::timer::TimeOut;
use rat_salsa::{run_tui, AppState, AppWidget, Control, RunConfig};
use rat_theme::dark_theme::DarkTheme;
use rat_theme::scheme::IMPERIAL;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::time::SystemTime;

type AppContext<'a> = rat_salsa::AppContext<'a, GlobalState, UltraMsg, Error>;
type RenderContext<'a> = rat_salsa::RenderContext<'a, GlobalState>;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let theme = DarkTheme::new("Imperial".into(), IMPERIAL);
    let mut global = GlobalState::new(theme);
    let mut state = UltraState::default();

    run_tui(
        Ultra,
        &mut global,
        &mut state,
        RunConfig::default()?.threads(1),
    )
}

#[derive(Debug)]
pub struct GlobalState {
    pub theme: DarkTheme,
}

impl GlobalState {
    pub fn new(theme: DarkTheme) -> Self {
        Self { theme }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UltraMsg {}

#[derive(Debug, Default)]
pub struct Ultra;

#[derive(Debug, Default)]
pub struct UltraState;

impl AppWidget<GlobalState, UltraMsg, Error> for Ultra {
    type State = UltraState;

    fn render(
        &self,
        _area: Rect,
        _buf: &mut Buffer,
        _state: &mut Self::State,
        _ctx: &mut RenderContext<'_>,
    ) -> Result<(), Error> {
        Ok(())
    }
}

impl AppState<GlobalState, UltraMsg, Error> for UltraState {
    fn init(&mut self, _ctx: &mut AppContext<'_>) -> Result<(), Error> {
        Ok(())
    }

    fn timer(
        &mut self,
        _event: &TimeOut,
        _ctx: &mut AppContext<'_>,
    ) -> Result<Control<UltraMsg>, Error> {
        Ok(Control::Continue)
    }

    fn crossterm(
        &mut self,
        event: &Event,
        _ctx: &mut AppContext<'_>,
    ) -> Result<Control<UltraMsg>, Error> {
        let r = match event {
            ct_event!(key press 'q') => Control::Quit,
            _ => Control::Continue,
        };

        Ok(r)
    }

    fn message(
        &mut self,
        _event: &mut UltraMsg,
        _ctx: &mut AppContext<'_>,
    ) -> Result<Control<UltraMsg>, Error> {
        Ok(Control::Continue)
    }

    fn error(&self, _event: Error, _ctx: &mut AppContext<'_>) -> Result<Control<UltraMsg>, Error> {
        Ok(Control::Continue)
    }
}

fn setup_logging() -> Result<(), Error> {
    // _ = fs::remove_file("log.log");
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}]\n        {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
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
