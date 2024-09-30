//! Current status: BETA
//!
#![doc = include_str!("../readme.md")]

use crossbeam::channel::{SendError, Sender};
use rat_widget::button::ButtonOutcome;
use rat_widget::event::{
    CalOutcome, ConsumedEvent, DoubleClickOutcome, EditOutcome, FileOutcome, MenuOutcome, Outcome,
    ScrollOutcome, TabbedOutcome, TextOutcome,
};
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
use crate::timer::{TimeOut, TimerDef, TimerHandle, Timers};
use rat_widget::focus::Focus;

pub use framework::*;
pub use run_config::*;
pub use threadpool::Cancel;

/// Result of event-handling.
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
pub enum Control<Message> {
    /// Continue with event-handling.
    /// In the event-loop this waits for the next event.
    Continue,
    /// Break event-handling without repaint.
    /// In the event-loop this waits for the next event.
    Unchanged,
    /// Break event-handling and repaints/renders the application.
    /// In the event-loop this calls `render`.
    Changed,
    /// Handle an application defined event. This calls `message`
    /// to distribute the message throughout the application.
    ///
    /// This helps with interactions between parts of the
    /// application.
    Message(Message),
    /// Quit the application.
    Quit,
}

impl<Message> Eq for Control<Message> {}

impl<Message> PartialEq for Control<Message> {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}

impl<Message> Ord for Control<Message> {
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

impl<Message> PartialOrd for Control<Message> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Message> ConsumedEvent for Control<Message> {
    fn is_consumed(&self) -> bool {
        !matches!(self, Control::Continue)
    }
}

impl<Message> From<Outcome> for Control<Message> {
    fn from(value: Outcome) -> Self {
        match value {
            Outcome::Continue => Control::Continue,
            Outcome::Unchanged => Control::Unchanged,
            Outcome::Changed => Control::Changed,
        }
    }
}

impl<Message> From<bool> for Control<Message> {
    fn from(value: bool) -> Self {
        Outcome::from(value).into()
    }
}

impl<Message> From<MenuOutcome> for Control<Message> {
    fn from(value: MenuOutcome) -> Self {
        Outcome::from(value).into()
    }
}

impl<Message> From<ButtonOutcome> for Control<Message> {
    fn from(value: ButtonOutcome) -> Self {
        Outcome::from(value).into()
    }
}

impl<Message> From<TextOutcome> for Control<Message> {
    fn from(value: TextOutcome) -> Self {
        Outcome::from(value).into()
    }
}

impl<Message> From<ScrollOutcome> for Control<Message> {
    fn from(value: ScrollOutcome) -> Self {
        Outcome::from(value).into()
    }
}

impl<Message> From<DoubleClickOutcome> for Control<Message> {
    fn from(value: DoubleClickOutcome) -> Self {
        Outcome::from(value).into()
    }
}

impl<Message> From<EditOutcome> for Control<Message> {
    fn from(value: EditOutcome) -> Self {
        Outcome::from(value).into()
    }
}

impl<Message> From<FileOutcome> for Control<Message> {
    fn from(value: FileOutcome) -> Self {
        Outcome::from(value).into()
    }
}

impl<Message> From<TabbedOutcome> for Control<Message> {
    fn from(value: TabbedOutcome) -> Self {
        Outcome::from(value).into()
    }
}

impl<Message> From<CalOutcome> for Control<Message> {
    fn from(value: CalOutcome) -> Self {
        Outcome::from(value).into()
    }
}

///
/// AppWidget mimics StatefulWidget and adds a [RenderContext]
///
#[allow(unused_variables)]
pub trait AppWidget<Global, Message, Error>
where
    Message: 'static + Send + Debug,
    Error: 'static + Send + Debug,
{
    /// Type of the State.
    type State: AppState<Global, Message, Error> + Debug;

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
/// AppState packs together the currently supported event-handlers.
///
#[allow(unused_variables)]
pub trait AppState<Global, Message, Error>
where
    Message: 'static + Send + Debug,
    Error: 'static + Send + Debug,
{
    /// Initialize the application. Runs before the first repaint.
    fn init(&mut self, ctx: &mut AppContext<'_, Global, Message, Error>) -> Result<(), Error> {
        Ok(())
    }

    /// Timeout event.
    fn timer(
        &mut self,
        event: &TimeOut,
        ctx: &mut AppContext<'_, Global, Message, Error>,
    ) -> Result<Control<Message>, Error> {
        Ok(Control::Continue)
    }

    /// Crossterm event.
    fn crossterm(
        &mut self,
        event: &crossterm::event::Event,
        ctx: &mut AppContext<'_, Global, Message, Error>,
    ) -> Result<Control<Message>, Error> {
        Ok(Control::Continue)
    }

    /// Process a message.
    fn message(
        &mut self,
        event: &mut Message,
        ctx: &mut AppContext<'_, Global, Message, Error>,
    ) -> Result<Control<Message>, Error> {
        Ok(Control::Continue)
    }

    /// Do error handling.
    fn error(
        &self,
        event: Error,
        ctx: &mut AppContext<'_, Global, Message, Error>,
    ) -> Result<Control<Message>, Error> {
        Ok(Control::Continue)
    }
}

///
/// Application context data.
///
#[derive(Debug)]
pub struct AppContext<'a, Global, Message, Error>
where
    Message: 'static + Send + Debug,
    Error: 'static + Send + Debug,
{
    /// Global state for the application.
    pub g: &'a mut Global,
    /// Can be set to hold a Focus, if needed.
    pub focus: Option<Focus>,

    /// Application timers.
    pub(crate) timers: &'a Timers,
    /// Background tasks.
    pub(crate) tasks: &'a ThreadPool<Message, Error>,
    /// Queue foreground tasks.
    pub(crate) queue: &'a ControlQueue<Message, Error>,
}

///
/// Application context data when rendering.
///
#[derive(Debug)]
pub struct RenderContext<'a, Global> {
    /// Some global state for the application.
    pub g: &'a mut Global,
    /// Current timeout that triggered the repaint.
    pub timeout: Option<TimeOut>,
    /// Frame counter.
    pub count: usize,
    /// Output cursor position. Set after rendering is complete.
    pub cursor: Option<(u16, u16)>,

    /// Application timers.
    pub(crate) timers: &'a Timers,
}

impl<'a, Global, Message, Error> AppContext<'a, Global, Message, Error>
where
    Message: 'static + Send + Debug,
    Error: 'static + Send + Debug,
{
    /// Add a timer.
    #[inline]
    pub fn add_timer(&self, t: TimerDef) -> TimerHandle {
        self.timers.add(t)
    }

    /// Remove a timer.
    #[inline]
    pub fn remove_timer(&self, tag: TimerHandle) {
        self.timers.remove(tag);
    }

    /// Replace a timer.
    /// Remove the old timer and create a new one.
    /// If the old timer no longer exists it just creates the new one.
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
    #[inline]
    pub fn spawn(
        &self,
        task: impl FnOnce(
                Cancel,
                &Sender<Result<Control<Message>, Error>>,
            ) -> Result<Control<Message>, Error>
            + Send
            + 'static,
    ) -> Result<Cancel, SendError<()>>
    where
        Message: 'static + Send + Debug,
        Error: 'static + Send + Debug,
    {
        self.tasks.send(Box::new(task))
    }

    /// Queue additional results.
    #[inline]
    pub fn queue(&self, ctrl: impl Into<Control<Message>>) {
        self.queue.push(Ok(ctrl.into()), None);
    }

    /// Queue an error.
    #[inline]
    pub fn queue_err(&self, err: Error) {
        self.queue.push(Err(err), None);
    }

    /// Access the focus-field.
    ///
    /// # Panic
    /// Panics if no focus has been set.
    #[inline]
    pub fn focus(&self) -> &Focus {
        self.focus.as_ref().expect("focus")
    }

    /// Access the focus-field.
    ///
    /// # Panic
    /// Panics if no focus has been set.
    #[inline]
    pub fn focus_mut(&mut self) -> &mut Focus {
        self.focus.as_mut().expect("focus")
    }
}

impl<'a, Global> RenderContext<'a, Global> {
    /// Add a timer.
    #[inline]
    pub fn add_timer(&self, t: TimerDef) -> TimerHandle {
        self.timers.add(t)
    }

    /// Remove a timer.
    #[inline]
    pub fn remove_timer(&self, tag: TimerHandle) {
        self.timers.remove(tag)
    }

    /// Replace a timer.
    #[inline]
    pub fn replace_timer(&self, h: Option<TimerHandle>, t: TimerDef) -> TimerHandle {
        if let Some(h) = h {
            self.remove_timer(h);
        }
        self.add_timer(t)
    }

    /// Set the cursor, if the given value is Some.
    pub fn set_screen_cursor(&mut self, cursor: Option<(u16, u16)>) {
        if let Some(c) = cursor {
            self.cursor = Some(c);
        }
    }
}

///
/// Event-handler traits and Keybindings.
///
pub mod event {
    pub use rat_widget::event::{ct_event, try_flow};
}
