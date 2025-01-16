use crate::poll_events::PollEvents;
use crate::terminal::{CrosstermTerminal, Terminal};
use crate::AppWidget;
use crossbeam::channel::TryRecvError;
use std::fmt::{Debug, Formatter};
use std::io;

/// Captures some parameters for [crate::run_tui()].
pub struct RunConfig<App, Global, Event, Error>
where
    App: AppWidget<Global, Event, Error>,
    Event: 'static + Send,
    Error: 'static + Send,
{
    /// How many worker threads are wanted?
    /// Most of the time 1 should be sufficient to offload any gui-blocking tasks.
    pub(crate) n_threats: usize,
    /// This is the renderer that connects to the backend, and calls out
    /// for rendering the application.
    ///
    /// Defaults to RenderCrossterm.
    pub(crate) term: Box<dyn Terminal<Error>>,
    /// List of all event-handlers for the application.
    ///
    /// Defaults to PollTimers, PollCrossterm, PollTasks. Add yours here.
    pub(crate) poll: Vec<Box<dyn PollEvents<Global, App::State, Event, Error>>>,
}

impl<App, Global, Event, Error> Debug for RunConfig<App, Global, Event, Error>
where
    App: AppWidget<Global, Event, Error>,
    Event: 'static + Send,
    Error: 'static + Send,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunConfig")
            .field("n_threads", &self.n_threats)
            .field("render", &"...")
            .field("events", &"...")
            .finish()
    }
}

impl<App, Global, Event, Error> RunConfig<App, Global, Event, Error>
where
    App: AppWidget<Global, Event, Error>,
    Event: 'static + Send,
    Error: 'static + Send + From<io::Error> + From<TryRecvError>,
{
    /// New configuration with some defaults.
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Result<Self, Error> {
        Ok(Self {
            n_threats: 1,
            term: Box::new(CrosstermTerminal::new()?),
            poll: Default::default(),
        })
    }

    /// Number of background-threads.
    /// Default is 1.
    pub fn threads(mut self, n: usize) -> Self {
        self.n_threats = n;
        self
    }

    /// Terminal is a rat-salsa::terminal::Terminal not a ratatui::Terminal.
    pub fn term(mut self, term: impl Terminal<Error> + 'static) -> Self {
        self.term = Box::new(term);
        self
    }

    /// Add one more poll impl.
    pub fn poll(
        mut self,
        poll: impl PollEvents<Global, App::State, Event, Error> + 'static,
    ) -> Self {
        self.poll.push(Box::new(poll));
        self
    }
}
