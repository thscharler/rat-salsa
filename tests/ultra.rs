use crossterm::event::Event;
use rat_salsa::event::{ct_event, flow_ok};
use rat_salsa::{run_tui, AppContext, AppState, AppWidget, Control, RenderContext, RunConfig};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Span;
use ratatui::widgets::Widget;

#[derive(Debug)]
struct MainApp;

#[derive(Debug)]
struct MainState;

impl AppWidget<(), (), anyhow::Error> for MainApp {
    type State = MainState;

    fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        _state: &mut Self::State,
        _ctx: &mut RenderContext<'_, ()>,
    ) -> Result<(), anyhow::Error> {
        Span::from("Hello world")
            .white()
            .on_blue()
            .render(area, buf);
        Ok(())
    }
}

impl AppState<(), (), anyhow::Error> for MainState {
    fn crossterm(
        &mut self,
        event: &Event,
        _ctx: &mut AppContext<'_, (), (), anyhow::Error>,
    ) -> Result<Control<()>, anyhow::Error> {
        flow_ok!(match event {
            ct_event!(key press 'q') => Control::Quit,
            _ => Control::Continue,
        });

        Ok(Control::Continue)
    }
}

#[test]
fn ultra() -> Result<(), anyhow::Error> {
    run_tui(MainApp, &mut (), &mut MainState, RunConfig::default()?)?;
    Ok(())
}
