use anyhow::Error;
use crossterm::event::Event;
use rat_salsa::{run_tui, AppState, AppWidget, Control, RunConfig};
use rat_theme::{dark_theme::DarkTheme, scheme::IMPERIAL};
use rat_widget::event::ct_event;
use ratatui::prelude::{Buffer, Rect, Widget};

type AppContext<'a> = rat_salsa::AppContext<'a, GlobalState, UltraMsg, Error>;
type RenderContext<'a> = rat_salsa::RenderContext<'a, GlobalState>;

fn main() -> Result<(), Error> {
    setup_logging()?;
    run_tui(
        Ultra,
        &mut GlobalState::new(DarkTheme::new("Imperial".into(), IMPERIAL)),
        &mut UltraState::default(),
        RunConfig::default()?,
    )
}

#[derive(Debug)]
pub struct GlobalState {
    pub theme: DarkTheme,
    pub err_msg: String,
}

impl GlobalState {
    pub fn new(theme: DarkTheme) -> Self {
        Self {
            theme,
            err_msg: Default::default(),
        }
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
        area: Rect,
        buf: &mut Buffer,
        _state: &mut Self::State,
        ctx: &mut RenderContext<'_>,
    ) -> Result<(), Error> {
        ctx.g.err_msg.as_str().render(area, buf);
        Ok(())
    }
}

impl AppState<GlobalState, UltraMsg, Error> for UltraState {
    fn crossterm(
        &mut self,
        event: &Event,
        _ctx: &mut AppContext<'_>,
    ) -> Result<Control<UltraMsg>, Error> {
        let r = match event {
            ct_event!(key press 'q') => Control::Quit,
            ct_event!(key press CONTROL-'q') => Control::Quit,
            _ => Control::Continue,
        };
        Ok(r)
    }

    fn error(&self, event: Error, ctx: &mut AppContext<'_>) -> Result<Control<UltraMsg>, Error> {
        ctx.g.err_msg = format!("{:?}", event).to_string();
        Ok(Control::Continue)
    }
}

fn setup_logging() -> Result<(), Error> {
    fern::Dispatch::new()
        .format(|out, message, _| out.finish(format_args!("{}", message)))
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("log.log")?)
        .apply()?;
    Ok(())
}
