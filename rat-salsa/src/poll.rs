/// Trait for an event-source.
///
/// If you need to add your own do the following:
///
/// * Implement this trait for a struct that fits.
///
pub trait PollEvents<Event, Error>: std::any::Any
where
    Event: 'static,
    Error: 'static,
{
    fn as_any(&self) -> &dyn std::any::Any;

    /// Preferred sleep time for this event-source.
    fn sleep_time(&self) -> Option<Duration> {
        None
    }

    /// Poll for a new event.
    ///
    /// Events are not processed immediately when they occur. Instead,
    /// all event sources are polled, the poll state is put into a queue.
    /// Then the queue is emptied one by one and `read_execute()` is called.
    ///
    /// This prevents issues with poll-ordering of multiple sources, and
    /// one source cannot just flood the app with events.
    fn poll(&mut self) -> Result<bool, Error>;

    /// Read the event and distribute it.
    ///
    /// If you add a new event, that doesn't fit into AppEvents, you'll
    /// have to define a new trait for your AppState and use that.
    fn read(&mut self) -> Result<crate::Control<Event>, Error>;
}

mod crossterm;
mod quit;
mod rendered;
mod thread_pool;
mod timer;
#[cfg(feature = "async")]
mod tokio_tasks;

use std::time::Duration;
pub use crossterm::PollCrossterm;
pub use quit::PollQuit;
pub use rendered::PollRendered;
pub use thread_pool::PollTasks;
pub use timer::PollTimers;
#[cfg(feature = "async")]
pub use tokio_tasks::PollTokio;