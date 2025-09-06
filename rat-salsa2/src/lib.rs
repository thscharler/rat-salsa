#![doc = include_str!("../readme.md")]

use crate::framework::control_queue::ControlQueue;
use crate::thread_pool::{Cancel, ThreadPool};
use crate::timer::{TimerDef, TimerHandle, Timers};
#[cfg(feature = "async")]
use crate::tokio_tasks::TokioTasks;
use crossbeam::channel::{SendError, Sender};
use rat_event::{ConsumedEvent, HandleEvent, Outcome, Regular};
use rat_focus::Focus;
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
#[cfg(feature = "async")]
use std::future::Future;
use std::mem;
use std::rc::Rc;
#[cfg(feature = "async")]
use tokio::task::AbortHandle;

mod framework;
mod poll_events;
pub mod rendered;
mod run_config;
pub mod tasks;
pub mod terminal;
pub mod thread_pool;
pub mod timer;
#[cfg(feature = "async")]
mod tokio_tasks;

/// Event sources.
pub mod poll {
    mod crossterm;
    mod rendered;
    mod thread_pool;
    mod timer;
    #[cfg(feature = "async")]
    mod tokio_tasks;

    pub use crossterm::PollCrossterm;
    pub use rendered::PollRendered;
    pub use thread_pool::PollTasks;
    pub use timer::PollTimers;
    #[cfg(feature = "async")]
    pub use tokio_tasks::PollTokio;
}

use crate::tasks::Liveness;
pub use framework::run_tui;
pub mod mock {
    use crate::Control;

    /// Empty placeholder for [run_tui].
    pub fn init<State, Global, Error>(_state: &mut State, _ctx: &mut Global) -> Result<(), Error> {
        Ok(())
    }

    /// Empty placeholder for [run_tui].
    pub fn error<Global, State, Event, Error>(
        _error: Error,
        _state: &mut State,
        _ctx: &mut Global,
    ) -> Result<Control<Event>, Error> {
        Ok(Control::Continue)
    }
}
pub use poll_events::PollEvents;
pub use run_config::RunConfig;

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
    /// The other way is to call [SalsaAppContext::queue] to initiate such
    /// events.
    Event(Event),
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
                Control::Event(_) => Ordering::Less,
                Control::Quit => Ordering::Less,
            },
            Control::Unchanged => match other {
                Control::Continue => Ordering::Greater,
                Control::Unchanged => Ordering::Equal,
                Control::Changed => Ordering::Less,
                Control::Event(_) => Ordering::Less,
                Control::Quit => Ordering::Less,
            },
            Control::Changed => match other {
                Control::Continue => Ordering::Greater,
                Control::Unchanged => Ordering::Greater,
                Control::Changed => Ordering::Equal,
                Control::Event(_) => Ordering::Less,
                Control::Quit => Ordering::Less,
            },
            Control::Event(_) => match other {
                Control::Continue => Ordering::Greater,
                Control::Unchanged => Ordering::Greater,
                Control::Changed => Ordering::Greater,
                Control::Event(_) => Ordering::Equal,
                Control::Quit => Ordering::Less,
            },
            Control::Quit => match other {
                Control::Continue => Ordering::Greater,
                Control::Unchanged => Ordering::Greater,
                Control::Changed => Ordering::Greater,
                Control::Event(_) => Ordering::Greater,
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

/// This trait gives access to all facilities built into rat-salsa.
///
/// This trait is implemented for the global state struct and gives
/// access to all rat-salsa functions at the same level as your
/// own global stuff.
///
///
///
pub trait SalsaContext<Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    /// The AppContext struct holds all the data for the rat-salsa
    /// functionality. run_tui calls this to set the initialized
    /// struct.
    fn set_salsa_ctx(&mut self, app_ctx: SalsaAppContext<Event, Error>);

    /// Access the AppContext previously set.
    fn salsa_ctx(&self) -> &SalsaAppContext<Event, Error>;

    /// Get the current frame/render-count.
    fn count(&self) -> usize {
        self.salsa_ctx().count.get()
    }

    /// Set the cursor, if the given value is something,
    /// hides it otherwise.
    ///
    /// This should only be set during rendering.
    fn set_screen_cursor(&self, cursor: Option<(u16, u16)>) {
        if let Some(c) = cursor {
            self.salsa_ctx().cursor.set(Some(c));
        }
    }

    /// Add a timer.
    ///
    /// __Panic__
    ///
    /// Panics if no timer support is configured.
    #[inline]
    fn add_timer(&self, t: TimerDef) -> TimerHandle {
        self.salsa_ctx()
            .timers
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
    fn remove_timer(&self, tag: TimerHandle) {
        self.salsa_ctx()
            .timers
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
    fn replace_timer(&self, h: Option<TimerHandle>, t: TimerDef) -> TimerHandle {
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
    ///     if cancel.is_canceled() {
    ///         return; // return early
    ///     }
    ///     Ok(Control::Continue)
    /// });
    /// ```
    ///
    /// - Cancel token
    ///
    /// The cancel token can be used by the application to signal an early
    /// cancellation of a long-running task. This cancellation is cooperative,
    /// the background task must regularly check for cancellation and quit
    /// if needed.
    ///
    /// - Liveness token
    ///
    /// This token is set whenever the given task has finished, be it
    /// regularly or by panicking.
    ///
    /// __Panic__
    ///
    /// Panics if no worker-thread support is configured.
    #[inline]
    fn spawn_ext(
        &self,
        task: impl FnOnce(Cancel, &Sender<Result<Control<Event>, Error>>) -> Result<Control<Event>, Error>
            + Send
            + 'static,
    ) -> Result<(Cancel, Liveness), SendError<()>>
    where
        Event: 'static + Send,
        Error: 'static + Send,
    {
        self.salsa_ctx()
            .tasks
            .as_ref()
            .expect(
                "No thread-pool configured. In main() add RunConfig::default()?.poll(PollTasks)",
            )
            .spawn(Box::new(task))
    }

    /// Add a background worker task.
    ///
    /// ```rust ignore
    /// let cancel = ctx.spawn(|| {
    ///     // ...
    ///     Ok(Control::Continue)
    /// });
    /// ```
    ///
    /// __Panic__
    ///
    /// Panics if no worker-thread support is configured.
    #[inline]
    fn spawn(
        &self,
        task: impl FnOnce() -> Result<Control<Event>, Error> + Send + 'static,
    ) -> Result<(), SendError<()>>
    where
        Event: 'static + Send,
        Error: 'static + Send,
    {
        _ = self
            .salsa_ctx()
            .tasks
            .as_ref()
            .expect(
                "No thread-pool configured. In main() add RunConfig::default()?.poll(PollTasks)",
            )
            .spawn(Box::new(|_, _| task()))?;
        Ok(())
    }

    /// Spawn a future in the executor.
    ///
    /// Panic
    ///
    /// Panics if tokio is not configured.
    #[inline]
    #[cfg(feature = "async")]
    fn spawn_async<F>(&self, future: F)
    where
        F: Future<Output = Result<Control<Event>, Error>> + Send + 'static,
    {
        _ = self.salsa_ctx() //
            .tokio
            .as_ref()
            .expect("No tokio runtime is configured. In main() add RunConfig::default()?.poll(PollTokio::new(rt))")
            .spawn(Box::new(future));
    }

    /// Spawn a future in the executor.
    /// You get an extra channel to send back more than one result.
    ///
    /// - AbortHandle
    ///
    /// The tokio AbortHandle to abort a spawned task.
    ///
    /// - Liveness
    ///
    /// This token is set whenever the given task has finished, be it
    /// regularly or by panicking.
    ///
    /// Panic
    ///
    /// Panics if tokio is not configured.
    #[inline]
    #[cfg(feature = "async")]
    fn spawn_async_ext<C, F>(&self, cr_future: C) -> (AbortHandle, Liveness)
    where
        C: FnOnce(tokio::sync::mpsc::Sender<Result<Control<Event>, Error>>) -> F,
        F: Future<Output = Result<Control<Event>, Error>> + Send + 'static,
    {
        let rt = self
            .salsa_ctx()//
            .tokio
            .as_ref()
            .expect("No tokio runtime is configured. In main() add RunConfig::default()?.poll(PollTokio::new(rt))");
        let future = cr_future(rt.sender());
        rt.spawn(Box::new(future))
    }

    /// Queue an application event.
    #[inline]
    fn queue_event(&self, event: Event) {
        self.salsa_ctx().queue.push(Ok(Control::Event(event)));
    }

    /// Queue additional results.
    #[inline]
    fn queue(&self, ctrl: impl Into<Control<Event>>) {
        self.salsa_ctx().queue.push(Ok(ctrl.into()));
    }

    /// Queue an error.
    #[inline]
    fn queue_err(&self, err: Error) {
        self.salsa_ctx().queue.push(Err(err));
    }

    /// Set the `Focus`.
    #[inline]
    fn set_focus(&self, focus: Focus) {
        self.salsa_ctx().focus.replace(Some(focus));
    }

    /// Take the `Focus` back from the Context.
    #[inline]
    fn take_focus(&self) -> Option<Focus> {
        self.salsa_ctx().focus.take()
    }

    /// Clear the `Focus`.
    #[inline]
    fn clear_focus(&self) {
        self.salsa_ctx().focus.replace(None);
    }

    /// Access the `Focus`.
    ///
    /// __Panic__
    ///
    /// Panics if no focus has been set.
    #[inline]
    fn focus<'a>(&'a self) -> Ref<'a, Focus> {
        let borrow = self.salsa_ctx().focus.borrow();
        Ref::map(borrow, |v| v.as_ref().expect("focus"))
    }

    /// Mutably access the focus-field.
    ///
    /// __Panic__
    ///
    /// Panics if no focus has been set.
    #[inline]
    fn focus_mut<'a>(&'a mut self) -> RefMut<'a, Focus> {
        let borrow = self.salsa_ctx().focus.borrow_mut();
        RefMut::map(borrow, |v| v.as_mut().expect("focus"))
    }

    /// Handle the focus-event and automatically queue the result.
    ///
    /// __Panic__
    ///
    /// Panics if no focus has been set.
    #[inline]
    fn focus_event<E>(&mut self, event: &E)
    where
        Focus: HandleEvent<E, Regular, Outcome>,
    {
        let mut borrow = self.salsa_ctx().focus.borrow_mut();
        let focus = borrow.as_mut().expect("focus");
        let r = focus.handle(event, Regular);
        if r.is_consumed() {
            self.queue(r);
        }
    }
}

///
/// Application context for event handling.
///
/// Add this to your global state and implement `Context` to
/// access the facilities of rat-salsa. You can Default::default()
/// initialize this field with some dummy values. It will
/// be set correctly when calling `run_tui()`.
///
pub struct SalsaAppContext<Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    /// Can be set to hold a Focus, if needed.
    pub(crate) focus: RefCell<Option<Focus>>,
    /// Last frame count rendered.
    pub(crate) count: Cell<usize>,
    /// Output cursor position. Set to Frame after rendering is complete.
    pub(crate) cursor: Cell<Option<(u16, u16)>>,

    /// Application timers.
    pub(crate) timers: Option<Rc<Timers>>,
    /// Background tasks.
    pub(crate) tasks: Option<Rc<ThreadPool<Event, Error>>>,
    /// Background tasks.
    #[cfg(feature = "async")]
    pub(crate) tokio: Option<Rc<TokioTasks<Event, Error>>>,
    /// Queue foreground tasks.
    pub(crate) queue: ControlQueue<Event, Error>,
}

impl<Event, Error> Debug for SalsaAppContext<Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut ff = f.debug_struct("AppContext");
        ff.field("focus", &self.focus)
            .field("count", &self.count)
            .field("cursor", &self.cursor)
            .field("timers", &self.timers)
            .field("tasks", &self.tasks)
            .field("queue", &self.queue);
        #[cfg(feature = "async")]
        {
            ff.field("tokio", &self.tokio);
        }
        ff.finish()
    }
}

impl<Event, Error> Default for SalsaAppContext<Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    fn default() -> Self {
        Self {
            focus: Default::default(),
            count: Default::default(),
            cursor: Default::default(),
            timers: Default::default(),
            tasks: Default::default(),
            #[cfg(feature = "async")]
            tokio: Default::default(),
            queue: Default::default(),
        }
    }
}

impl<Event, Error> SalsaContext<Event, Error> for SalsaAppContext<Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    #[inline]
    fn set_salsa_ctx(&mut self, app_ctx: SalsaAppContext<Event, Error>) {
        *self = app_ctx;
    }

    #[inline]
    fn salsa_ctx(&self) -> &SalsaAppContext<Event, Error> {
        self
    }
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
