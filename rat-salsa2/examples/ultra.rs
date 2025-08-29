use anyhow::Error;
use rat_salsa2::poll::PollCrossterm;
use rat_salsa2::{run_tui, Control, RunConfig};
use rat_theme2::palettes::IMPERIAL;
use rat_theme2::DarkTheme;
use rat_widget::event::ct_event;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

type AppContext<'a> = rat_salsa2::AppContext<'a, GlobalState, UltraEvent, Error>;
type RenderContext<'a> = rat_salsa2::RenderContext<'a, GlobalState>;

fn main() -> Result<(), Error> {
    setup_logging()?;
    run_tui(
        |_, _| Ok(()),
        render,
        event,
        error,
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
pub struct UltraState;

fn render(
    area: Rect,
    buf: &mut Buffer,
    _state: &mut UltraState,
    ctx: &mut RenderContext<'_>,
) -> Result<(), Error> {
    ctx.g.err_msg.as_str().render(area, buf);
    Ok(())
}

fn event(
    event: &UltraEvent,
    _state: &mut UltraState,
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

fn error(
    event: Error,
    _state: &mut UltraState,
    ctx: &mut AppContext<'_>,
) -> Result<Control<UltraEvent>, Error> {
    ctx.g.err_msg = format!("{:?}", event).to_string();
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
