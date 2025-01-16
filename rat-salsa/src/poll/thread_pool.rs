use crate::{AppContext, AppState, Control, PollEvents};
use crossbeam::channel::TryRecvError;
use std::any::Any;

/// Processes results from background tasks.
#[derive(Debug)]
pub struct PollTasks;

impl<Global, State, Event, Error> PollEvents<Global, State, Event, Error> for PollTasks
where
    State: AppState<Global, Event, Error> + ?Sized,
    Event: 'static + Send,
    Error: 'static + Send + From<TryRecvError>,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn poll(&mut self, ctx: &mut AppContext<'_, Global, Event, Error>) -> Result<bool, Error> {
        if let Some(tasks) = ctx.tasks {
            Ok(!tasks.is_empty())
        } else {
            Ok(false)
        }
    }

    fn read_exec(
        &mut self,
        _state: &mut State,
        ctx: &mut AppContext<'_, Global, Event, Error>,
    ) -> Result<Control<Event>, Error> {
        if let Some(tasks) = ctx.tasks {
            tasks.try_recv()
        } else {
            Ok(Control::Continue)
        }
    }
}
