use crate::poll::PollEvents;
use crate::terminal::{CrosstermTerminal, Terminal};
use crossbeam::channel::TryRecvError;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::io;
use std::rc::Rc;

/// Captures some parameters for [crate::run_tui()].
pub struct RunConfig<Event, Error>
where
    Event: 'static,
    Error: 'static,
{
    /// Don't run init/shutdown.
    pub(crate) manual: bool,
    /// This is the renderer that connects to the backend, and calls out
    /// for rendering the application.
    ///
    /// Defaults to RenderCrossterm.
    pub(crate) term: Rc<RefCell<dyn Terminal<Error>>>,
    /// List of all event-handlers for the application.
    ///
    /// Defaults to PollTimers, PollCrossterm, PollTasks. Add yours here.
    pub(crate) poll: Vec<Box<dyn PollEvents<Event, Error>>>,
}

impl<Event, Error> Debug for RunConfig<Event, Error>
where
    Event: 'static,
    Error: 'static,
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
    Event: 'static,
    Error: 'static + From<io::Error> + From<TryRecvError>,
{
    /// New configuration with some defaults.
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Result<Self, Error> {
        Ok(Self {
            manual: Default::default(),
            term: Rc::new(RefCell::new(CrosstermTerminal::new()?)),
            poll: Default::default(),
        })
    }

    /// Terminal is a rat-salsa::terminal::Terminal not a ratatui::Terminal.
    pub fn new(term: impl Terminal<Error> + 'static) -> Self {
        Self {
            manual: Default::default(),
            term: Rc::new(RefCell::new(term)),
            poll: Default::default(),
        }
    }

    /// Don't run [Terminal::init] and [Terminal::shutdown].
    /// You can run your own terminal initialization/shutdown before/after calling run_tui(),
    pub fn manual_mode(mut self) -> Self {
        self.manual = true;
        self
    }

    /// Add one more poll impl.
    pub fn poll(mut self, poll: impl PollEvents<Event, Error> + 'static) -> Self {
        self.poll.push(Box::new(poll));
        self
    }
}
