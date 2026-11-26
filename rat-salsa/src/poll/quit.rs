use crate::Control;
use crate::event::QuitEvent;
use crate::poll::PollEvents;

///
/// Sends an event before finally terminating the app.
///
#[derive(Debug, Default)]
pub struct PollQuit;

impl<Event, Error> PollEvents<Event, Error> for PollQuit
where
    Event: 'static + From<QuitEvent>,
    Error: 'static,
{
    fn poll(&mut self) -> Result<bool, Error> {
        Ok(false)
    }

    fn read(&mut self) -> Result<Control<Event>, Error> {
        Ok(Control::Event(Event::from(QuitEvent)))
    }
}
