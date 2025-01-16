use crate::rendered::RenderedEvent;
use crate::{AppContext, AppState, Control, PollEvents};
use std::any::Any;

/// Sends an event after a render of the UI.
#[derive(Debug, Default)]
pub struct PollRendered;

impl<Global, State, Event, Error> PollEvents<Global, State, Event, Error> for PollRendered
where
    State: AppState<Global, Event, Error> + ?Sized,
    Event: 'static + Send + From<RenderedEvent>,
    Error: 'static + Send + From<std::io::Error>,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn poll(&mut self, _ctx: &mut AppContext<'_, Global, Event, Error>) -> Result<bool, Error> {
        // doesn't poll. it's triggered by a repaint.
        Ok(false)
    }

    fn read_exec(
        &mut self,
        state: &mut State,
        ctx: &mut AppContext<'_, Global, Event, Error>,
    ) -> Result<Control<Event>, Error> {
        state.event(&RenderedEvent.into(), ctx)
    }
}
