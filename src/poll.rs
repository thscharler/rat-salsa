//!
//! Defines the trait for event-sources.
//!

use crate::terminal::Terminal;
use crate::timer::TimerEvent;
use crate::{AppContext, AppState, AppWidget, Control};
use crossbeam::channel::TryRecvError;
use std::fmt::Debug;
use std::time::Duration;

/// Trait for an event-source.
///
/// If you need to add your own do the following:
///
/// * Implement this trait for a struct that fits.
///     TODO: try this
///
pub trait PollEvents<App, Global, Message, Error>
where
    App: AppWidget<Global, Message, Error>,
    Message: 'static + Send + Debug,
    Error: 'static + Send + Debug,
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
        ctx: &mut AppContext<'_, Global, Message, Error>,
    ) -> Result<bool, Error>;

    /// Read the event and distribute it.
    ///
    /// If you add a new event, that doesn't fit into AppEvents, you'll
    /// have to define a new trait for your AppState and use that.
    fn read_exec(
        &mut self,
        app: &mut App,
        state: &mut App::State,
        term: &mut dyn Terminal<App, Global, Message, Error>,
        ctx: &mut AppContext<'_, Global, Message, Error>,
    ) -> Result<Control<Message>, Error>;
}

/// Processes results from background tasks.
#[derive(Debug)]
pub struct PollTasks;

impl<App, Global, Message, Error> PollEvents<App, Global, Message, Error> for PollTasks
where
    App: AppWidget<Global, Message, Error>,
    Message: 'static + Send + Debug,
    Error: 'static + Send + Debug + From<TryRecvError> + Debug,
{
    fn poll(&mut self, ctx: &mut AppContext<'_, Global, Message, Error>) -> Result<bool, Error> {
        Ok(!ctx.tasks.is_empty())
    }

    fn read_exec(
        &mut self,
        _app: &mut App,
        _state: &mut App::State,
        _term: &mut dyn Terminal<App, Global, Message, Error>,
        ctx: &mut AppContext<'_, Global, Message, Error>,
    ) -> Result<Control<Message>, Error> {
        ctx.tasks.try_recv()
    }
}

/// Processes timers.
#[derive(Debug)]
pub struct PollTimers;

impl<App, Global, Message, Error> PollEvents<App, Global, Message, Error> for PollTimers
where
    App: AppWidget<Global, Message, Error>,
    Message: 'static + Send + Debug,
    Error: 'static + Send + Debug + From<std::io::Error>,
{
    fn poll(&mut self, ctx: &mut AppContext<'_, Global, Message, Error>) -> Result<bool, Error> {
        Ok(ctx.timers.poll())
    }

    fn read_exec(
        &mut self,
        app: &mut App,
        state: &mut App::State,
        term: &mut dyn Terminal<App, Global, Message, Error>,
        ctx: &mut AppContext<'_, Global, Message, Error>,
    ) -> Result<Control<Message>, Error> {
        match ctx.timers.read() {
            None => Ok(Control::Continue),
            Some(TimerEvent::Repaint(t)) => {
                if let Err(e) = term.render(app, state, Some(t), ctx) {
                    Err(e)
                } else {
                    Ok(Control::Continue)
                }
            }
            Some(TimerEvent::Application(t)) => {
                let r = state.timer(&t, ctx);
                r
            }
        }
    }
}

/// Processes crossterm events.
#[derive(Debug)]
pub struct PollCrossterm;

impl<App, Global, Message, Error> PollEvents<App, Global, Message, Error> for PollCrossterm
where
    App: AppWidget<Global, Message, Error>,
    Message: 'static + Send + Debug,
    Error: 'static + Send + Debug + From<std::io::Error>,
{
    fn poll(&mut self, _ctx: &mut AppContext<'_, Global, Message, Error>) -> Result<bool, Error> {
        Ok(crossterm::event::poll(Duration::from_millis(0))?)
    }

    fn read_exec(
        &mut self,
        _app: &mut App,
        state: &mut App::State,
        _term: &mut dyn Terminal<App, Global, Message, Error>,
        ctx: &mut AppContext<'_, Global, Message, Error>,
    ) -> Result<Control<Message>, Error> {
        match crossterm::event::read() {
            // NOTODO: can this be abstracted out too? sure.
            // but it's not worth it.
            Ok(event) => state.crossterm(&event, ctx),
            Err(e) => Err(e.into()),
        }
    }
}
