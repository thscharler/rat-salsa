use crate::{AppContext, AppState, Control, PollEvents};
use std::any::Any;
use std::time::Duration;

/// Processes crossterm events.
#[derive(Debug)]
pub struct PollCrossterm;

impl<Global, State, Event, Error> PollEvents<Global, State, Event, Error> for PollCrossterm
where
    State: AppState<Global, Event, Error> + ?Sized,
    Event: 'static + Send + From<crossterm::event::Event>,
    Error: 'static + Send + From<std::io::Error>,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn poll(&mut self, _ctx: &mut AppContext<'_, Global, Event, Error>) -> Result<bool, Error> {
        Ok(crossterm::event::poll(Duration::from_millis(0))?)
    }

    fn read_exec(
        &mut self,
        state: &mut State,
        ctx: &mut AppContext<'_, Global, Event, Error>,
    ) -> Result<Control<Event>, Error> {
        match crossterm::event::read() {
            Ok(event) => state.event(&event.into(), ctx),
            Err(e) => Err(e.into()),
        }
    }
}
