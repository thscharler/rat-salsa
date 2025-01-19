//!
//! Defines the trait for event-sources.
//!

use crate::Control;
use std::any::Any;

/// Trait for an event-source.
///
/// If you need to add your own do the following:
///
/// * Implement this trait for a struct that fits.
///
pub trait PollEvents<Event, Error>: Any
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    fn as_any(&self) -> &dyn Any;

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
    fn read(&mut self) -> Result<Control<Event>, Error>;
}
