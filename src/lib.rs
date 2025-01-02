//! Current status: BETA
//!
#![doc = include_str!("../readme.md")]

use crossbeam::channel::{SendError, Sender};
use rat_widget::event::{ConsumedEvent, Outcome};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::mem;

pub(crate) mod control_queue;
mod framework;
pub mod poll;
pub(crate) mod poll_queue;
mod run_config;
pub mod terminal;
mod threadpool;
pub mod timer;

use crate::control_queue::ControlQueue;
use crate::threadpool::ThreadPool;
use crate::timer::{TimerDef, TimerHandle, Timers};
use rat_widget::focus::Focus;

pub use framework::*;
pub use run_config::*;
pub use threadpool::Cancel;

/// Result enum for event handling.
///
/// The result of an event is processed immediately after the
/// function returns, before polling new events. This way an action
/// can trigger another action which triggers the repaint without
/// other events intervening.
///
/// If you ever need to return more than one result from event-handling,
/// you can hand it to AppContext/RenderContext::queue(). Events
/// in the queue are processed in order, and the return value of
/// the event-handler comes last. If an error is returned, everything
/// send to the queue will be executed nonetheless.
///
/// __See__
///
/// - [flow!](rat_widget::event::flow)
/// - [try_flow!](rat_widget::event::try_flow)
/// - [ConsumedEvent]
#[derive(Debug, Clone, Copy)]
#[must_use]
#[non_exhaustive]
pub enum Control<Event> {
    /// Continue with event-handling.
    /// In the event-loop this waits for the next event.
    Continue,
    /// Break event-handling without repaint.
    /// In the event-loop this waits for the next event.
    Unchanged,
    /// Break event-handling and repaints/renders the application.
    /// In the event-loop this calls `render`.
    Changed,
    /// Eventhandling can cause secondary application specific events.
    /// One common way is to return this `Control::Message(my_event)`
    /// to reenter the event-loop with your own secondary event.
    ///
    /// This acts quite like a message-queue to communicate between
    /// disconnected parts of your application. And indeed there is
    /// a hidden message-queue as part of the event-loop.
    ///
    /// The other way is to call [AppContext::queue] to initiate such
    /// events.
    Message(Event),
    /// Quit the application.
    Quit,
}

impl<Event> Eq for Control<Event> {}

impl<Event> PartialEq for Control<Event> {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}

impl<Event> Ord for Control<Event> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Control::Continue => match other {
                Control::Continue => Ordering::Equal,
                Control::Unchanged => Ordering::Less,
                Control::Changed => Ordering::Less,
                Control::Message(_) => Ordering::Less,
                Control::Quit => Ordering::Less,
            },
            Control::Unchanged => match other {
                Control::Continue => Ordering::Greater,
                Control::Unchanged => Ordering::Equal,
                Control::Changed => Ordering::Less,
                Control::Message(_) => Ordering::Less,
                Control::Quit => Ordering::Less,
            },
            Control::Changed => match other {
                Control::Continue => Ordering::Greater,
                Control::Unchanged => Ordering::Greater,
                Control::Changed => Ordering::Equal,
                Control::Message(_) => Ordering::Less,
                Control::Quit => Ordering::Less,
            },
            Control::Message(_) => match other {
                Control::Continue => Ordering::Greater,
                Control::Unchanged => Ordering::Greater,
                Control::Changed => Ordering::Greater,
                Control::Message(_) => Ordering::Equal,
                Control::Quit => Ordering::Less,
            },
            Control::Quit => match other {
                Control::Continue => Ordering::Greater,
                Control::Unchanged => Ordering::Greater,
                Control::Changed => Ordering::Greater,
                Control::Message(_) => Ordering::Greater,
                Control::Quit => Ordering::Equal,
            },
        }
    }
}

impl<Event> PartialOrd for Control<Event> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Event> ConsumedEvent for Control<Event> {
    fn is_consumed(&self) -> bool {
        !matches!(self, Control::Continue)
    }
}

impl<Event, T: Into<Outcome>> From<T> for Control<Event> {
    fn from(value: T) -> Self {
        let r = value.into();
        match r {
            Outcome::Continue => Control::Continue,
            Outcome::Unchanged => Control::Unchanged,
            Outcome::Changed => Control::Changed,
        }
    }
}

///
/// AppWidget mimics StatefulWidget and adds a [RenderContext]
///
pub trait AppWidget<Global, Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    /// Type of the State.
    type State: AppState<Global, Event, Error> + ?Sized;

    /// Renders an application widget.
    fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut Self::State,
        ctx: &mut RenderContext<'_, Global>,
    ) -> Result<(), Error>;
}

///
pub struct RenderedEvent;

///
/// AppState executes events and has some init and error-handling.
///
/// There is no separate shutdown, handle this case with an application
/// specific event.
///
#[allow(unused_variables)]
pub trait AppState<Global, Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    /// Initialize the application. Runs before the first repaint.
    fn init(
        &mut self, //
        ctx: &mut AppContext<'_, Global, Event, Error>,
    ) -> Result<(), Error> {
        Ok(())
    }

    /// Shutdown the application. Runs after the event-loop has ended.
    ///
    /// __Panic__
    ///
    /// Doesn't run if a panic occurred.
    ///
    ///__Errors__
    /// Any errors will be returned to main().
    fn shutdown(&mut self, ctx: &mut AppContext<'_, Global, Event, Error>) -> Result<(), Error> {
        Ok(())
    }

    /// Handle an event.
    fn event(
        &mut self,
        event: &Event,
        ctx: &mut AppContext<'_, Global, Event, Error>,
    ) -> Result<Control<Event>, Error> {
        Ok(Control::Continue)
    }

    /// Do error handling.
    fn error(
        &self,
        event: Error,
        ctx: &mut AppContext<'_, Global, Event, Error>,
    ) -> Result<Control<Event>, Error> {
        Ok(Control::Continue)
    }
}

///
/// Application context for event handling.
///
#[derive(Debug)]
pub struct AppContext<'a, Global, Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    /// Global state for the application.
    pub g: &'a mut Global,
    /// Can be set to hold a Focus, if needed.
    pub focus: Option<Focus>,
    /// Last frame count rendered.
    pub count: usize,

    /// Application timers.
    pub(crate) timers: &'a Option<Timers>,
    /// Background tasks.
    pub(crate) tasks: &'a Option<ThreadPool<Event, Error>>,
    /// Queue foreground tasks.
    pub(crate) queue: &'a ControlQueue<Event, Error>,
}

///
/// Application context for rendering.
///
#[derive(Debug)]
pub struct RenderContext<'a, Global> {
    /// Some global state for the application.
    pub g: &'a mut Global,
    /// Frame counter.
    pub count: usize,
    /// Output cursor position. Set after rendering is complete.
    pub cursor: Option<(u16, u16)>,
}

impl<'a, Global, Event, Error> AppContext<'a, Global, Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    /// Add a timer.
    ///
    /// __Panic__
    ///
    /// Panics if no timer support is configured.
    #[inline]
    pub fn add_timer(&self, t: TimerDef) -> TimerHandle {
        self.timers
            .as_ref()
            .expect("No timers configured. In main() add RunConfig::default()?.poll(PollTimers)")
            .add(t)
    }

    /// Remove a timer.
    ///
    /// __Panic__
    ///
    /// Panics if no timer support is configured.
    #[inline]
    pub fn remove_timer(&self, tag: TimerHandle) {
        self.timers
            .as_ref()
            .expect("No timers configured. In main() add RunConfig::default()?.poll(PollTimers)")
            .remove(tag);
    }

    /// Replace a timer.
    /// Remove the old timer and create a new one.
    /// If the old timer no longer exists it just creates the new one.
    ///
    /// __Panic__
    ///
    /// Panics if no timer support is configured.
    #[inline]
    pub fn replace_timer(&self, h: Option<TimerHandle>, t: TimerDef) -> TimerHandle {
        if let Some(h) = h {
            self.remove_timer(h);
        }
        self.add_timer(t)
    }

    /// Add a background worker task.
    ///
    /// ```rust ignore
    /// let cancel = ctx.spawn(|cancel, send| {
    ///     // ... do stuff
    ///     Ok(Control::Continue)
    /// });
    /// ```
    ///
    /// __Panic__
    ///
    /// Panics if no worker-thread support is configured.
    #[inline]
    pub fn spawn(
        &self,
        task: impl FnOnce(Cancel, &Sender<Result<Control<Event>, Error>>) -> Result<Control<Event>, Error>
            + Send
            + 'static,
    ) -> Result<Cancel, SendError<()>>
    where
        Event: 'static + Send,
        Error: 'static + Send,
    {
        self.tasks
            .as_ref()
            .expect(
                "No thread-pool configured. In main() add RunConfig::default()?.poll(PollTasks)",
            )
            .send(Box::new(task))
    }

    /// Queue additional results.
    #[inline]
    pub fn queue(&self, ctrl: impl Into<Control<Event>>) {
        self.queue.push(Ok(ctrl.into()));
    }

    /// Queue an error.
    #[inline]
    pub fn queue_err(&self, err: Error) {
        self.queue.push(Err(err));
    }

    /// Access the focus-field.
    ///
    /// __Panic__
    ///
    /// Panics if no focus has been set.
    #[inline]
    pub fn focus(&self) -> &Focus {
        self.focus.as_ref().expect("focus")
    }

    /// Access the focus-field.
    ///
    /// __Panic__
    ///
    /// Panics if no focus has been set.
    #[inline]
    pub fn focus_mut(&mut self) -> &mut Focus {
        self.focus.as_mut().expect("focus")
    }
}

impl<'a, Global> RenderContext<'a, Global> {
    /// Set the cursor, if the given value is Some.
    pub fn set_screen_cursor(&mut self, cursor: Option<(u16, u16)>) {
        if let Some(c) = cursor {
            self.cursor = Some(c);
        }
    }
}
