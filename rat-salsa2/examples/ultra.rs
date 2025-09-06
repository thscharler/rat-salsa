use anyhow::{anyhow, Error};
use rat_salsa2::poll::PollCrossterm;
use rat_salsa2::{mock, run_tui, Control, RunConfig, SalsaAppContext, SalsaContext};
use rat_widget::event::ct_event;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

fn main() -> Result<(), Error> {
    run_tui(
        mock::init,
        render,
        event,
        error,
        &mut Global::default(),
        &mut Ultra,
        RunConfig::default()?.poll(PollCrossterm),
    )
}

#[derive(Debug, Default)]
pub struct Global {
    ctx: SalsaAppContext<UltraEvent, Error>,
    pub err_cnt: u32,
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UltraEvent {
    Event(crossterm::event::Event),
}

impl From<crossterm::event::Event> for UltraEvent {
    fn from(value: crossterm::event::Event) -> Self {
        Self::Event(value)
    }
}

pub struct Ultra;

fn render(area: Rect, buf: &mut Buffer, _state: &mut Ultra, ctx: &mut Global) -> Result<(), Error> {
    Line::from_iter([Span::from("'q' to quit, 'e' for error, 'r' for repair")])
        .render(Rect::new(area.x, area.y, area.width, 1), buf);
    Line::from_iter([
        Span::from("Hello world!").green(),
        Span::from(" Status: "),
        if ctx.err_cnt > 0 {
            Span::from(&ctx.err_msg).red().underlined()
        } else {
            Span::from(&ctx.err_msg).cyan().underlined()
        },
    ])
    .render(Rect::new(area.x, area.y + 2, area.width, 1), buf);
    Ok(())
}

fn event(
    event: &UltraEvent,
    _state: &mut Ultra,
    ctx: &mut Global,
) -> Result<Control<UltraEvent>, Error> {
    match event {
        UltraEvent::Event(event) => match event {
            ct_event!(key press 'q') => Ok(Control::Quit),
            ct_event!(key press 'e') => return Err(anyhow!("An error occured.")),
            ct_event!(key press 'r') => {
                if ctx.err_cnt > 1 {
                    ctx.err_cnt -= 1;
                    ctx.err_msg = format!("#{}# One error repaired.", ctx.err_cnt).to_string();
                } else if ctx.err_cnt == 1 {
                    ctx.err_cnt -= 1;
                    ctx.err_msg = "All within norms.".to_string();
                } else {
                    ctx.err_cnt = 1;
                    ctx.err_msg = format!("#{}# Over-repaired.", ctx.err_cnt).to_string();
                }
                Ok(Control::Changed)
            }
            _ => Ok(Control::Continue),
        },
    }
}

fn error(event: Error, _state: &mut Ultra, ctx: &mut Global) -> Result<Control<UltraEvent>, Error> {
    ctx.err_cnt += 1;
    ctx.err_msg = format!("#{}# {}", ctx.err_cnt, event).to_string();
    Ok(Control::Changed)
}
