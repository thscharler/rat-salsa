//!
//! Defines the trait for event-sources.
//!

use crate::timer::{TimeOut, TimerEvent};
use crate::{AppContext, AppState, Control};
use crossbeam::channel::TryRecvError;
use std::any::Any;
use std::fmt::Debug;
use std::time::Duration;

/// Trait for an event-source.
///
/// If you need to add your own do the following:
///
/// * Implement this trait for a struct that fits.
///
pub trait PollEvents<Global, State, Event, Error>: Any
where
    State: AppState<Global, Event, Error> + ?Sized,
    Event: 'static + Send,
    Error: 'static + Send,
{
    /// Poll for a new event.
    ///
    /// Events are not processed immediately when they occur. Instead,
    /// all event sources are polled, the poll state is put into a queue.
    /// Then the queue is emptied one by one and `read_execute()` is called.
    ///
    /// This prevents issues with poll-ordering of multiple sources, and
    /// one source cannot just flood the app with events.
    fn poll(
        &mut self, //
        ctx: &mut AppContext<'_, Global, Event, Error>,
    ) -> Result<bool, Error>;

    /// Read the event and distribute it.
    ///
    /// If you add a new event, that doesn't fit into AppEvents, you'll
    /// have to define a new trait for your AppState and use that.
    fn read_exec(
        &mut self,
        state: &mut State,
        ctx: &mut AppContext<'_, Global, Event, Error>,
    ) -> Result<Control<Event>, Error>;
}

/// Processes results from background tasks.
#[derive(Debug)]
pub struct PollTasks;

impl<Global, State, Event, Error> PollEvents<Global, State, Event, Error> for PollTasks
where
    State: AppState<Global, Event, Error> + ?Sized,
    Event: 'static + Send,
    Error: 'static + Send + From<TryRecvError>,
{
    fn poll(&mut self, ctx: &mut AppContext<'_, Global, Event, Error>) -> Result<bool, Error> {
        if let Some(tasks) = ctx.tasks {
            Ok(!tasks.is_empty())
        } else {
            Ok(false)
        }
    }

    fn read_exec(
        &mut self,
        _state: &mut State,
        ctx: &mut AppContext<'_, Global, Event, Error>,
    ) -> Result<Control<Event>, Error> {
        if let Some(tasks) = ctx.tasks {
            tasks.try_recv()
        } else {
            Ok(Control::Continue)
        }
    }
}

/// Processes timers.
#[derive(Debug, Default)]
pub struct PollTimers;

impl<Global, State, Event, Error> PollEvents<Global, State, Event, Error> for PollTimers
where
    State: AppState<Global, Event, Error> + ?Sized,
    Event: 'static + Send + From<TimeOut>,
    Error: 'static + Send + From<std::io::Error>,
{
    fn poll(&mut self, ctx: &mut AppContext<'_, Global, Event, Error>) -> Result<bool, Error> {
        if let Some(timers) = ctx.timers {
            Ok(timers.poll())
        } else {
            Ok(false)
        }
    }

    fn read_exec(
        &mut self,
        state: &mut State,
        ctx: &mut AppContext<'_, Global, Event, Error>,
    ) -> Result<Control<Event>, Error> {
        if let Some(timers) = ctx.timers {
            match timers.read() {
                None => Ok(Control::Continue),
                Some(TimerEvent(t)) => state.event(&t.into(), ctx),
            }
        } else {
            Ok(Control::Continue)
        }
    }
}

/// Processes crossterm events.
#[derive(Debug)]
pub struct PollCrossterm;

impl<Global, State, Event, Error> PollEvents<Global, State, Event, Error> for PollCrossterm
where
    State: AppState<Global, Event, Error> + ?Sized,
    Event: 'static + Send + From<crossterm::event::Event>,
    Error: 'static + Send + From<std::io::Error>,
{
    fn poll(&mut self, _ctx: &mut AppContext<'_, Global, Event, Error>) -> Result<bool, Error> {
        Ok(crossterm::event::poll(Duration::from_millis(0))?)
    }

    fn read_exec(
        &mut self,
        state: &mut State,
        ctx: &mut AppContext<'_, Global, Event, Error>,
    ) -> Result<Control<Event>, Error> {
        match crossterm::event::read() {
            Ok(event) => state.event(&event.into(), ctx),
            Err(e) => Err(e.into()),
        }
    }
}
