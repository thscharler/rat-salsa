use crate::rendered::RenderedEvent;
use crate::{Control, PollEvents};
use std::any::Any;

/// Sends an event after a render of the UI.
#[derive(Debug, Default)]
pub struct PollRendered;

impl<Event, Error> PollEvents<Event, Error> for PollRendered
where
    Event: 'static + Send + From<RenderedEvent>,
    Error: 'static + Send + From<std::io::Error>,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn poll(&mut self) -> Result<bool, Error> {
        // doesn't poll. it's triggered by a repaint.
        Ok(false)
    }

    fn read(&mut self) -> Result<Control<Event>, Error> {
        Ok(Control::Event(RenderedEvent.into()))
    }
}
