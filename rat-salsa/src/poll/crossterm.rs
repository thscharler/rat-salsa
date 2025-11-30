use crate::Control;
use crate::poll::PollEvents;
use std::any::Any;
use std::time::Duration;

/// Processes crossterm events.
#[derive(Debug)]
pub struct PollCrossterm;

impl<Event, Error> PollEvents<Event, Error> for PollCrossterm
where
    Event: 'static + From<ratatui_crossterm::crossterm::event::Event>,
    Error: 'static + From<std::io::Error>,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn poll(&mut self) -> Result<bool, Error> {
        Ok(ratatui_crossterm::crossterm::event::poll(
            Duration::from_millis(0),
        )?)
    }

    fn read(&mut self) -> Result<Control<Event>, Error> {
        Ok(ratatui_crossterm::crossterm::event::read().map(|v| Control::Event(v.into()))?)
    }
}
