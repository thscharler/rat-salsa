use anyhow::Error;
use rat_salsa2::poll::PollCrossterm;
use rat_salsa2::{run_tui, Control, RunConfig, SalsaAppContext, SalsaContext};
use rat_theme2::palettes::IMPERIAL;
use rat_theme2::DarkTheme;
use rat_widget::event::ct_event;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

fn main() -> Result<(), Error> {
    setup_logging()?;
    run_tui(
        |_, _| Ok(()),
        render,
        event,
        error,
        &mut Global::new(DarkTheme::new("Imperial".into(), IMPERIAL)),
        &mut Ultra::default(),
        RunConfig::default()?.poll(PollCrossterm),
    )
}

#[derive(Debug)]
pub struct Global {
    ctx: SalsaAppContext<UltraEvent, Error>,
    pub theme: DarkTheme,
    pub err_msg: String,
}

impl SalsaContext<UltraEvent, Error> for Global {
    fn set_salsa_ctx(&mut self, app_ctx: SalsaAppContext<UltraEvent, Error>) {
        self.ctx = app_ctx;
    }

    fn salsa_ctx(&self) -> &SalsaAppContext<UltraEvent, Error> {
        &self.ctx
    }
}

impl Global {
    pub fn new(theme: DarkTheme) -> Self {
        Self {
            ctx: Default::default(),
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

fn render(area: Rect, buf: &mut Buffer, _state: &mut Ultra, ctx: &mut Global) -> Result<(), Error> {
    ctx.err_msg.as_str().render(area, buf);
    Ok(())
}

fn event(
    event: &UltraEvent,
    _state: &mut Ultra,
    _ctx: &mut Global,
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

fn error(event: Error, _state: &mut Ultra, ctx: &mut Global) -> Result<Control<UltraEvent>, Error> {
    ctx.err_msg = format!("{:?}", event).to_string();
    Ok(Control::Continue)
}

fn setup_logging() -> Result<(), Error> {
    fern::Dispatch::new()
        .format(|out, message, _| out.finish(format_args!("{}", message)))
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("log.log")?)
        .apply()?;
    Ok(())
}
