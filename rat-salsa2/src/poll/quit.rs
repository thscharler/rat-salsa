use crate::event::QuitEvent;
use crate::poll::PollEvents;
use crate::Control;
use std::any::Any;

///
/// Sends an event before finally terminating the app.
///
#[derive(Debug, Default)]
pub struct PollQuit;

impl<Event, Error> PollEvents<Event, Error> for PollQuit
where
    Event: 'static + Send + From<QuitEvent>,
    Error: 'static + Send,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn poll(&mut self) -> Result<bool, Error> {
        Ok(false)
    }

    fn read(&mut self) -> Result<Control<Event>, Error> {
        Ok(Control::Event(Event::from(QuitEvent)))
    }
}
