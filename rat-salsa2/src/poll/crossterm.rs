use crate::poll::PollEvents;
use crate::Control;
use std::any::Any;
use std::time::Duration;

/// Processes crossterm events.
#[derive(Debug)]
pub struct PollCrossterm;

impl<Event, Error> PollEvents<Event, Error> for PollCrossterm
where
    Event: 'static + Send + From<crossterm::event::Event>,
    Error: 'static + Send + From<std::io::Error>,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn poll(&mut self) -> Result<bool, Error> {
        Ok(crossterm::event::poll(Duration::from_millis(0))?)
    }

    fn read(&mut self) -> Result<Control<Event>, Error> {
        Ok(crossterm::event::read().map(|v| Control::Event(v.into()))?)
    }
}
