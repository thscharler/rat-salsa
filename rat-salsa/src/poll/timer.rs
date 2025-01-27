use crate::timer::{TimeOut, Timers};
use crate::{Control, PollEvents};
use std::any::Any;
use std::rc::Rc;

/// Processes timers.
#[derive(Debug, Default)]
pub struct PollTimers {
    timers: Rc<Timers>,
}

impl PollTimers {
    pub fn new() -> Self {
        Self {
            timers: Rc::new(Timers::default()),
        }
    }

    pub(crate) fn get_timers(&self) -> Rc<Timers> {
        self.timers.clone()
    }
}

impl<Event, Error> PollEvents<Event, Error> for PollTimers
where
    Event: 'static + Send + From<TimeOut>,
    Error: 'static + Send + From<std::io::Error>,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn poll(&mut self) -> Result<bool, Error> {
        Ok(self.timers.poll())
    }

    fn read(&mut self) -> Result<Control<Event>, Error> {
        Ok(self
            .timers
            .read()
            .map(|v| Control::Event(v.0.into()))
            .unwrap_or(Control::Continue))
    }
}
