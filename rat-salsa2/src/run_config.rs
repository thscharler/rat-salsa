use crate::poll_events::PollEvents;
use crate::terminal::{CrosstermTerminal, Terminal};
use crossbeam::channel::TryRecvError;
use std::fmt::{Debug, Formatter};
use std::io;

/// Captures some parameters for [crate::run_tui()].
pub struct RunConfig<Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    /// This is the renderer that connects to the backend, and calls out
    /// for rendering the application.
    ///
    /// Defaults to RenderCrossterm.
    pub(crate) term: Box<dyn Terminal<Error>>,
    /// List of all event-handlers for the application.
    ///
    /// Defaults to PollTimers, PollCrossterm, PollTasks. Add yours here.
    pub(crate) poll: Vec<Box<dyn PollEvents<Event, Error>>>,
}

impl<Event, Error> Debug for RunConfig<Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunConfig")
            .field("render", &"...")
            .field("events", &"...")
            .finish()
    }
}

impl<Event, Error> RunConfig<Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send + From<io::Error> + From<TryRecvError>,
{
    /// New configuration with some defaults.
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Result<Self, Error> {
        Ok(Self {
            term: Box::new(CrosstermTerminal::new()?),
            poll: Default::default(),
        })
    }

    /// Terminal is a rat-salsa::terminal::Terminal not a ratatui::Terminal.
    pub fn new(term: impl Terminal<Error> + 'static) -> Self {
        Self {
            term: Box::new(term),
            poll: Default::default(),
        }
    }

    /// Add one more poll impl.
    pub fn poll(mut self, poll: impl PollEvents<Event, Error> + 'static) -> Self {
        self.poll.push(Box::new(poll));
        self
    }
}
