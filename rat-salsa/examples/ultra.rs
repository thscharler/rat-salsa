use anyhow::Error;
use rat_salsa::poll::PollCrossterm;
use rat_salsa::{run_tui, AppState, AppWidget, Control, RunConfig};
use rat_theme2::palettes::IMPERIAL;
use rat_theme2::DarkTheme;
use rat_widget::event::ct_event;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

type AppContext<'a> = rat_salsa::AppContext<'a, GlobalState, UltraEvent, Error>;
type RenderContext<'a> = rat_salsa::RenderContext<'a, GlobalState>;

fn main() -> Result<(), Error> {
    setup_logging()?;
    run_tui(
        Ultra,
        &mut GlobalState::new(DarkTheme::new("Imperial".into(), IMPERIAL)),
        &mut UltraState::default(),
        RunConfig::default()?.poll(PollCrossterm),
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
pub enum UltraEvent {
    Event(crossterm::event::Event),
}

impl From<crossterm::event::Event> for UltraEvent {
    fn from(value: crossterm::event::Event) -> Self {
        Self::Event(value)
    }
}

#[derive(Debug, Default)]
pub struct Ultra;

#[derive(Debug, Default)]
pub struct UltraState;

impl AppWidget<GlobalState, UltraEvent, Error> for Ultra {
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

impl AppState<GlobalState, UltraEvent, Error> for UltraState {
    fn event(
        &mut self,
        event: &UltraEvent,
        _ctx: &mut AppContext<'_>,
    ) -> Result<Control<UltraEvent>, Error> {
        let r = match event {
            UltraEvent::Event(event) => match event {
                ct_event!(key press 'q') => Control::Quit,
                ct_event!(key press CONTROL-'q') => Control::Quit,
                _ => Control::Continue,
            },
        };
        Ok(r)
    }

    fn error(&self, event: Error, ctx: &mut AppContext<'_>) -> Result<Control<UltraEvent>, Error> {
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
