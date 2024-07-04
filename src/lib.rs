#![doc = include_str!("../readme.md")]

use crossbeam::channel::{SendError, Sender};
use rat_widget::button::ButtonOutcome;
use rat_widget::event::{
    ConsumedEvent, DoubleClickOutcome, EditOutcome, Outcome, ScrollOutcome, TextOutcome,
};
use rat_widget::menuline::MenuOutcome;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::fmt::Debug;

pub(crate) mod control_queue;
mod framework;
pub mod poll;
pub mod terminal;
mod threadpool;
pub mod timer;

use crate::control_queue::ControlQueue;
use crate::threadpool::ThreadPool;
use crate::timer::{TimeOut, TimerDef, TimerHandle, Timers};

pub use framework::{run_tui, RunConfig};
use rat_widget::file_dialog::FileOutcome;
pub use threadpool::Cancel;

/// Result of event-handling.
///
/// The macro
/// [rat-event::flow_ok!](https://docs.rs/rat-event/latest/rat_event/macro.flow_ok.html)
/// provides control-flow using this enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[must_use]
pub enum Control<Action> {
    /// Continue handling the current event.
    /// In the event-loop this goes on waiting for a new event.
    Continue,
    /// Break handling the current event.
    /// In the event-loop this does nothing and just waits for a new event.
    Break,
    /// Triggers a repaint in the event-loop.
    Repaint,
    /// The event-loop calls out the action-handlers to take care of it.
    Action(Action),
    /// Quit the application.
    Quit,
}

impl<Action> ConsumedEvent for Control<Action> {
    fn is_consumed(&self) -> bool {
        !matches!(self, Control::Continue)
    }
}

impl<Action> From<Outcome> for Control<Action> {
    fn from(value: Outcome) -> Self {
        match value {
            Outcome::NotUsed => Control::Continue,
            Outcome::Unchanged => Control::Break,
            Outcome::Changed => Control::Repaint,
        }
    }
}

impl<Action> From<MenuOutcome> for Control<Action> {
    fn from(value: MenuOutcome) -> Self {
        match value {
            MenuOutcome::NotUsed => Control::Continue,
            MenuOutcome::Unchanged => Control::Break,
            MenuOutcome::Changed => Control::Repaint,
            MenuOutcome::Selected(_) => Control::Repaint,
            MenuOutcome::Activated(_) => Control::Repaint,
            MenuOutcome::MenuSelected(_, _) => Control::Repaint,
            MenuOutcome::MenuActivated(_, _) => Control::Repaint,
        }
    }
}

impl<Action> From<ButtonOutcome> for Control<Action> {
    fn from(value: ButtonOutcome) -> Self {
        match value {
            ButtonOutcome::NotUsed => Control::Continue,
            ButtonOutcome::Unchanged => Control::Break,
            ButtonOutcome::Changed => Control::Repaint,
            ButtonOutcome::Pressed => Control::Repaint,
        }
    }
}

impl<Action> From<TextOutcome> for Control<Action> {
    fn from(value: TextOutcome) -> Self {
        match value {
            TextOutcome::NotUsed => Control::Continue,
            TextOutcome::Unchanged => Control::Break,
            TextOutcome::Changed => Control::Repaint,
            TextOutcome::TextChanged => Control::Repaint,
        }
    }
}

impl<Action> From<ScrollOutcome> for Control<Action> {
    fn from(value: ScrollOutcome) -> Self {
        match value {
            ScrollOutcome::NotUsed => Control::Continue,
            ScrollOutcome::Unchanged => Control::Break,
            ScrollOutcome::Changed => Control::Repaint,
            ScrollOutcome::Up(_) => Control::Repaint,
            ScrollOutcome::Down(_) => Control::Repaint,
            ScrollOutcome::Left(_) => Control::Repaint,
            ScrollOutcome::Right(_) => Control::Repaint,
            ScrollOutcome::VPos(_) => Control::Repaint,
            ScrollOutcome::HPos(_) => Control::Repaint,
        }
    }
}

impl<Action> From<DoubleClickOutcome> for Control<Action> {
    fn from(value: DoubleClickOutcome) -> Self {
        match value {
            DoubleClickOutcome::NotUsed => Control::Continue,
            DoubleClickOutcome::Unchanged => Control::Break,
            DoubleClickOutcome::Changed => Control::Repaint,
            DoubleClickOutcome::ClickClick(_, _) => Control::Break,
        }
    }
}

impl<Action> From<EditOutcome> for Control<Action> {
    fn from(value: EditOutcome) -> Self {
        match value {
            EditOutcome::NotUsed => Control::Continue,
            EditOutcome::Unchanged => Control::Break,
            EditOutcome::Changed => Control::Repaint,
            EditOutcome::Insert => Control::Break,
            EditOutcome::Remove => Control::Break,
            EditOutcome::Edit => Control::Break,
            EditOutcome::Append => Control::Break,
            EditOutcome::Cancel => Control::Break,
            EditOutcome::Commit => Control::Break,
            EditOutcome::CommitAndAppend => Control::Break,
            EditOutcome::CommitAndEdit => Control::Break,
        }
    }
}

impl<Action> From<FileOutcome> for Control<Action> {
    fn from(value: FileOutcome) -> Self {
        match value {
            FileOutcome::NotUsed => Control::Continue,
            FileOutcome::Unchanged => Control::Break,
            FileOutcome::Changed => Control::Repaint,
            FileOutcome::Cancel => Control::Repaint,
            FileOutcome::Ok(_) => Control::Repaint,
        }
    }
}

///
/// A trait for application level widgets.
///
/// This trait is an anlog to ratatui's StatefulWidget, and
/// does only the rendering part. It's extended with all the
/// extras needed in an application.
///
#[allow(unused_variables)]
pub trait AppWidget<Global, Action, Error>
where
    Action: 'static + Send + Debug,
    Error: 'static + Send + Debug,
{
    /// Type of the State.
    type State: AppEvents<Global, Action, Error> + Debug;

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
/// Eventhandling for application level widgets.
///
/// This one collects all currently defined events.
/// Implement this one on the state struct.
///
#[allow(unused_variables)]
pub trait AppEvents<Global, Action, Error>
where
    Action: 'static + Send + Debug,
    Error: 'static + Send + Debug,
{
    /// Initialize the application. Runs before the first repaint.
    fn init(&mut self, ctx: &mut AppContext<'_, Global, Action, Error>) -> Result<(), Error> {
        Ok(())
    }

    /// Timeout event.
    fn timer(
        &mut self,
        event: &TimeOut,
        ctx: &mut AppContext<'_, Global, Action, Error>,
    ) -> Result<Control<Action>, Error> {
        Ok(Control::Continue)
    }

    /// Crossterm event.
    fn crossterm(
        &mut self,
        event: &crossterm::event::Event,
        ctx: &mut AppContext<'_, Global, Action, Error>,
    ) -> Result<Control<Action>, Error> {
        Ok(Control::Continue)
    }

    /// Run an action.
    fn action(
        &mut self,
        event: &mut Action,
        ctx: &mut AppContext<'_, Global, Action, Error>,
    ) -> Result<Control<Action>, Error> {
        Ok(Control::Continue)
    }

    /// Do error handling.
    fn error(
        &self,
        event: Error,
        ctx: &mut AppContext<'_, Global, Action, Error>,
    ) -> Result<Control<Action>, Error> {
        Ok(Control::Continue)
    }
}

/// A collection of context data used by the application.
#[derive(Debug)]
pub struct AppContext<'a, Global, Action, Error>
where
    Action: 'static + Send + Debug,
    Error: 'static + Send + Debug,
{
    /// Some global state for the application.
    pub g: &'a mut Global,
    /// Current timeout, if any.
    pub timeout: Option<TimeOut>,

    /// Application timers.
    pub(crate) timers: &'a Timers,
    /// Background tasks.
    pub(crate) tasks: &'a ThreadPool<Action, Error>,
    /// Queue foreground tasks.
    queue: &'a ControlQueue<Action, Error>,
}

/// A collection of context data used for rendering.
#[derive(Debug)]
pub struct RenderContext<'a, Global> {
    /// Some global state for the application.
    pub g: &'a mut Global,
    /// Current timeout that triggered the repaint.
    pub timeout: Option<TimeOut>,

    /// Frame counter.
    pub counter: usize,
    /// Frame area.
    pub area: Rect,
    /// Output cursor position. Set after rendering is complete.
    pub cursor: Option<(u16, u16)>,
}

impl<'a, Global, Action, Error> AppContext<'a, Global, Action, Error>
where
    Action: 'static + Send + Debug,
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
        self.timers.remove(tag)
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
                &Sender<Result<Control<Action>, Error>>,
            ) -> Result<Control<Action>, Error>
            + Send
            + 'static,
    ) -> Result<Cancel, SendError<()>>
    where
        Action: 'static + Send + Debug,
        Error: 'static + Send + Debug,
    {
        self.tasks.send(task)
    }

    /// Queue additional results.
    ///
    /// X
    #[inline]
    pub fn queue(&self, ctrl: impl Into<Control<Action>>) {
        self.queue.push(Ok(ctrl.into()))
    }

    /// Queue additional results.
    #[inline]
    pub fn queue_result(&self, ctrl: Result<Control<Action>, Error>) {
        self.queue.push(ctrl)
    }
}

///
/// Event-handler traits and Keybindings.
///
pub mod event {
    pub use rat_widget::event::{ct_event, flow_ok};
}
