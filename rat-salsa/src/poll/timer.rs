use crate::timer::{TimeOut, TimerEvent};
use crate::{AppContext, AppState, Control, PollEvents};
use std::any::Any;

/// Processes timers.
#[derive(Debug, Default)]
pub struct PollTimers;

impl<Global, State, Event, Error> PollEvents<Global, State, Event, Error> for PollTimers
where
    State: AppState<Global, Event, Error> + ?Sized,
    Event: 'static + Send + From<TimeOut>,
    Error: 'static + Send + From<std::io::Error>,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn poll(&mut self, ctx: &mut AppContext<'_, Global, Event, Error>) -> Result<bool, Error> {
        if let Some(timers) = ctx.timers {
            Ok(timers.poll())
        } else {
            Ok(false)
        }
    }

    fn read_exec(
        &mut self,
        state: &mut State,
        ctx: &mut AppContext<'_, Global, Event, Error>,
    ) -> Result<Control<Event>, Error> {
        if let Some(timers) = ctx.timers {
            match timers.read() {
                None => Ok(Control::Continue),
                Some(TimerEvent(t)) => state.event(&t.into(), ctx),
            }
        } else {
            Ok(Control::Continue)
        }
    }
}
