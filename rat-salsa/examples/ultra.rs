use anyhow::{Error, anyhow};
use rat_salsa::poll::PollCrossterm;
use rat_salsa::{Control, RunConfig, SalsaAppContext, mock, run_tui};
use rat_widget::event::ct_event;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Stylize;
use ratatui_core::text::{Line, Span};
use ratatui_core::widgets::Widget;
use ratatui_crossterm::crossterm::event::Event;

type Global = SalsaAppContext<UltraEvent, Error>;

fn main() -> Result<(), Error> {
    run_tui(
        mock::init,
        render,
        event,
        error,
        &mut Global::default(),
        &mut Ultra::default(),
        RunConfig::default()?.poll(PollCrossterm),
    )
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UltraEvent {
    Event(Event),
}

impl From<Event> for UltraEvent {
    fn from(value: Event) -> Self {
        Self::Event(value)
    }
}

#[derive(Default)]
pub struct Ultra {
    pub err_cnt: u32,
    pub err_msg: String,
}

fn render(area: Rect, buf: &mut Buffer, state: &mut Ultra, _ctx: &mut Global) -> Result<(), Error> {
    Line::from_iter([Span::from("'q' to quit, 'e' for error, 'r' for repair")])
        .render(Rect::new(area.x, area.y, area.width, 1), buf);
    Line::from_iter([
        Span::from("Hello world!").green(),
        Span::from(" Status: "),
        if state.err_cnt > 0 {
            Span::from(&state.err_msg).red().underlined()
        } else {
            Span::from(&state.err_msg).cyan().underlined()
        },
    ])
    .render(Rect::new(area.x, area.y + 2, area.width, 1), buf);
    Ok(())
}

fn event(
    event: &UltraEvent,
    state: &mut Ultra,
    _ctx: &mut Global,
) -> Result<Control<UltraEvent>, Error> {
    match event {
        UltraEvent::Event(event) => match event {
            ct_event!(key press 'q') => Ok(Control::Quit),
            ct_event!(key press 'e') => Err(anyhow!("An error occured.")),
            ct_event!(key press 'r') => {
                if state.err_cnt > 1 {
                    state.err_cnt -= 1;
                    state.err_msg = format!("#{}# One error repaired.", state.err_cnt).to_string();
                } else if state.err_cnt == 1 {
                    state.err_cnt -= 1;
                    state.err_msg = "All within norms.".to_string();
                } else {
                    state.err_cnt = 1;
                    state.err_msg = format!("#{}# Over-repaired.", state.err_cnt).to_string();
                }
                Ok(Control::Changed)
            }
            _ => Ok(Control::Continue),
        },
    }
}

fn error(event: Error, state: &mut Ultra, _ctx: &mut Global) -> Result<Control<UltraEvent>, Error> {
    state.err_cnt += 1;
    state.err_msg = format!("#{}# {}", state.err_cnt, event).to_string();
    Ok(Control::Changed)
}
