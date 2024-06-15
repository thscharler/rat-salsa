use crate::event::RepaintEvent::{Repaint, Timer};
use crate::timer::{TimerHandle, Timers};
use crate::{Control, RepaintEvent, TimeOut, TimerDef, TimerEvent};
use crossbeam::channel::{bounded, unbounded, Receiver, SendError, Sender, TryRecvError};
use crossterm::cursor::{DisableBlinking, EnableBlinking, SetCursorStyle};
use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use log::debug;
use ratatui::backend::CrosstermBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::Terminal;
use std::cell::RefCell;
use std::cmp::min;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::io::{stdout, Stdout};
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;
use std::{io, mem, thread};

///
/// A trait for application level widgets.
///
/// This trait is an anlog to ratatui's StatefulWidget, and
/// does only the rendering part. It's extended with all the
/// extras needed in an application.
///
#[allow(unused_variables)]
pub trait AppWidget<Global, Action, Error> {
    /// Type of the State.
    type State: AppEvents<Global, Action, Error> + Debug;

    /// Renders an application widget.
    fn render(
        &self,
        event: &RepaintEvent,
        area: Rect,
        buf: &mut Buffer,
        state: &mut Self::State,
        ctx: &mut RenderContext<'_, Global, Action, Error>,
    ) -> Result<(), Error>;
}

///
/// Eventhandling for application level widgets.
///
/// This one collects all currently defined events.
/// Implement this one on the state struct.
///
#[allow(unused_variables)]
pub trait AppEvents<Global, Action, Error> {
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

/// Type for a background task.
type Task<Action, Error> = Box<
    dyn FnOnce(Cancel, &Sender<Result<Control<Action>, Error>>) -> Result<Control<Action>, Error>
        + Send,
>;

/// Type for cancelation.
type Cancel = Arc<Mutex<bool>>;

/// A collection of context data used by the application.
#[derive(Debug)]
pub struct AppContext<'a, Global, Action, Error> {
    /// Some global state for the application.
    pub g: &'a mut Global,
    /// Application timers.
    timers: &'a Timers,
    /// Start background tasks.
    tasks: Tasks<'a, Action, Error>,
    /// Queue foreground tasks.
    queue: &'a ControlQueue<Action, Error>,
}

impl<'a, Global, Action, Error> AppContext<'a, Global, Action, Error> {
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
    /// let cancel = ctx.spawn(Box::new(|cancel, send| {
    ///     // ... do stuff
    ///     Ok(Control::Continue)
    /// }));
    /// ```
    #[inline]
    pub fn spawn(&self, task: Task<Action, Error>) -> Result<Cancel, SendError<()>>
    where
        Action: 'static + Send,
        Error: 'static + Send,
    {
        self.tasks.send(task)
    }

    /// Queue additional results.
    #[inline]
    pub fn queue(&self, ctrl: impl Into<Control<Action>>) {
        let v: Control<Action> = ctrl.into();
        self.queue.push(v.into())
    }
}

/// A collection of context data used for rendering.
#[derive(Debug)]
pub struct RenderContext<'a, Global, Action, Error> {
    /// Some global state for the application.
    pub g: &'a mut Global,
    /// Application timers.
    timers: &'a Timers,
    /// Start background tasks.
    tasks: Tasks<'a, Action, Error>,
    /// Queue foreground tasks.
    queue: &'a ControlQueue<Action, Error>,

    /// Frame counter.
    pub counter: usize,
    /// Frame area.
    pub area: Rect,
    /// Output cursor position. Set after rendering is complete.
    pub cursor: Option<(u16, u16)>,
}

impl<'a, Global, Action, Error> RenderContext<'a, Global, Action, Error> {
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
    #[inline]
    pub fn spawn(&self, task: Task<Action, Error>) -> Result<Cancel, SendError<()>>
    where
        Action: 'static + Send,
        Error: 'static + Send,
    {
        self.tasks.send(task)
    }

    /// Queue additional results.
    #[inline]
    pub fn queue(&self, ctrl: impl Into<Control<Action>>) {
        let v: Control<Action> = ctrl.into();
        self.queue.push(v.into())
    }
}

/// Captures some parameters for [run_tui()].
#[derive(Debug)]
pub struct RunConfig {
    /// How many worker threads are wanted?
    /// Most of the time 1 should be sufficient to offload any gui-blocking tasks.
    pub n_threats: usize,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self { n_threats: 1 }
    }
}

/// Run the event-loop
pub fn run_tui<Widget, Global, Action, Error>(
    app: Widget,
    global: &mut Global,
    state: &mut Widget::State,
    cfg: RunConfig,
) -> Result<(), Error>
where
    Widget: AppWidget<Global, Action, Error>,
    Action: Send + 'static,
    Error: Send + 'static + From<TryRecvError> + From<io::Error> + From<SendError<()>>,
{
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;
    stdout().execute(EnableBracketedPaste)?;
    stdout().execute(EnableBlinking)?;
    stdout().execute(SetCursorStyle::BlinkingBar)?;
    enable_raw_mode()?;

    let r = match catch_unwind(AssertUnwindSafe(|| _run_tui(app, global, state, cfg))) {
        Ok(v) => v,
        Err(e) => {
            _ = disable_raw_mode();
            _ = stdout().execute(SetCursorStyle::DefaultUserShape);
            _ = stdout().execute(DisableBlinking);
            _ = stdout().execute(DisableBracketedPaste);
            _ = stdout().execute(DisableMouseCapture);
            _ = stdout().execute(LeaveAlternateScreen);

            resume_unwind(e);
        }
    };

    disable_raw_mode()?;
    stdout().execute(SetCursorStyle::DefaultUserShape)?;
    stdout().execute(DisableBlinking)?;
    stdout().execute(DisableBracketedPaste)?;
    stdout().execute(DisableMouseCapture)?;
    stdout().execute(LeaveAlternateScreen)?;

    r
}

fn _run_tui<App, Global, Action, Error>(
    mut app: App,
    global: &mut Global,
    state: &mut App::State,
    cfg: RunConfig,
) -> Result<(), Error>
where
    App: AppWidget<Global, Action, Error>,
    Action: Send + 'static,
    Error: Send + 'static + From<TryRecvError> + From<io::Error> + From<SendError<()>>,
{
    let mut term = Terminal::new(CrosstermBackend::new(stdout()))?;
    term.clear()?;

    let timers = Timers::default();
    let worker = ThreadPool::<Action, Error>::build(cfg.n_threats);
    let queue = ControlQueue::default();
    let poll = PollQueue::default();

    let mut appctx = AppContext {
        g: global,
        timers: &timers,
        tasks: Tasks { send: &worker.send },
        queue: &queue,
    };

    let mut sleep = Duration::from_millis(10);

    // init
    state.init(&mut appctx)?;

    // initial repaint.
    _ = repaint_tui(&mut app, Repaint, state, &mut term, &mut appctx)?;

    let nice = 'ui: loop {
        // panic on worker panic
        if !worker.check_liveness() {
            break 'ui false;
        }

        if queue.is_empty() {
            if poll.is_empty() {
                if poll_timers(&timers) {
                    poll.push(Poll::Timers);
                }
                if poll_workers(&worker) {
                    poll.push(Poll::Workers);
                }
                if poll_crossterm()? {
                    poll.push(Poll::Crossterm);
                }
            }
            if poll.is_empty() {
                let t = calculate_sleep(&timers, sleep);
                thread::sleep(t);
                if sleep < Duration::from_millis(10) {
                    // Back off slowly.
                    sleep += Duration::from_micros(100);
                }
            } else {
                // Shorter sleep immediately after an event.
                sleep = Duration::from_micros(100);
            }
        }

        if queue.is_empty() {
            match poll.take() {
                Some(Poll::Timers) => {
                    if let Some(t) = read_timers(&timers) {
                        queue.push(ControlB::Timer(t));
                    }
                }
                Some(Poll::Workers) => queue.push(read_workers(&worker).into()),
                Some(Poll::Crossterm) => match read_crossterm() {
                    Ok(event) => queue.push(ControlB::Crossterm(event)),
                    Err(err) => queue.push(ControlB::Error(err.into())),
                },
                None => {}
            }
        }

        match queue.take() {
            None => {}
            Some(ControlB::Continue) => {}
            Some(ControlB::Break) => {}
            Some(ControlB::Repaint) => {
                queue.push(repaint_tui(&mut app, Repaint, state, &mut term, &mut appctx).into());
            }
            Some(ControlB::Action(mut action)) => {
                queue.push(state.action(&mut action, &mut appctx).into());
            }
            Some(ControlB::Quit) => {
                break 'ui true;
            }
            Some(ControlB::Error(err)) => {
                queue.push(state.error(err, &mut appctx).into());
            }
            Some(ControlB::Timer(TimerEvent::Repaint(timeout))) => {
                queue.push(
                    repaint_tui(&mut app, Timer(timeout), state, &mut term, &mut appctx).into(),
                );
            }
            Some(ControlB::Timer(TimerEvent::Application(timeout))) => {
                queue.push(state.timer(&timeout, &mut appctx).into());
            }
            Some(ControlB::Crossterm(event)) => {
                queue.push(state.crossterm(&event, &mut appctx).into());
            }
        }
    };

    if nice {
        worker.stop_and_join();
    } else {
        worker.stop_and_join();
        panic!("worker panicked");
    }

    Ok(())
}

fn repaint_tui<App, Global, Action, Error>(
    app: &mut App,
    reason: RepaintEvent,
    state: &mut App::State,
    term: &mut Terminal<CrosstermBackend<Stdout>>,
    ctx: &mut AppContext<'_, Global, Action, Error>,
) -> Result<(), Error>
where
    App: AppWidget<Global, Action, Error>,
    Error: From<io::Error>,
{
    let mut res = Ok(());

    _ = term.hide_cursor();

    term.draw(|frame| {
        let frame_area = frame.size();
        let mut ctx = RenderContext {
            g: ctx.g,
            timers: ctx.timers,
            tasks: ctx.tasks.clone(),
            queue: ctx.queue,
            counter: frame.count(),
            area: frame_area,
            cursor: None,
        };

        res = app.render(&reason, frame_area, frame.buffer_mut(), state, &mut ctx);

        if let Some((cursor_x, cursor_y)) = ctx.cursor {
            frame.set_cursor(cursor_x, cursor_y);
        }
    })?;

    res
}

fn poll_timers(timers: &Timers) -> bool {
    timers.poll()
}

fn read_timers(timers: &Timers) -> Option<TimerEvent> {
    timers.read()
}

fn poll_workers<Action, Error>(worker: &ThreadPool<Action, Error>) -> bool
where
    Action: Send + 'static,
    Error: Send + 'static + From<TryRecvError>,
{
    !worker.is_empty()
}

fn read_workers<Action, Error>(worker: &ThreadPool<Action, Error>) -> Result<Control<Action>, Error>
where
    Action: Send + 'static,
    Error: Send + 'static + From<TryRecvError>,
{
    worker.try_recv()
}

fn poll_crossterm() -> Result<bool, io::Error> {
    crossterm::event::poll(Duration::from_millis(0))
}

fn read_crossterm() -> Result<crossterm::event::Event, io::Error> {
    crossterm::event::read()
}

fn calculate_sleep(timers: &Timers, max: Duration) -> Duration {
    if let Some(sleep) = timers.sleep_time() {
        min(sleep, max)
    } else {
        max
    }
}

#[derive(Debug)]
enum Poll {
    Timers,
    Workers,
    Crossterm,
}

/// Queue for storing successful polls.
#[derive(Debug, Default)]
struct PollQueue {
    queue: RefCell<VecDeque<Poll>>,
}

impl PollQueue {
    fn is_empty(&self) -> bool {
        self.queue.borrow().is_empty()
    }

    fn take(&self) -> Option<Poll> {
        self.queue.borrow_mut().pop_front()
    }

    fn push(&self, poll: Poll) {
        self.queue.borrow_mut().push_back(poll);
    }
}

/// Queue for event-handling results.
#[derive(Debug)]
struct ControlQueue<Action, Error> {
    queue: RefCell<VecDeque<ControlB<Action, Error>>>,
}

#[derive(Debug)]
enum ControlB<Action, Error> {
    Continue,
    Break,
    Repaint,
    Action(Action),
    Quit,
    Error(Error),
    Timer(TimerEvent),
    Crossterm(crossterm::event::Event),
}

impl<Action, Error> From<Control<Action>> for ControlB<Action, Error> {
    fn from(value: Control<Action>) -> Self {
        match value {
            Control::Continue => ControlB::Continue,
            Control::Break => ControlB::Break,
            Control::Repaint => ControlB::Repaint,
            Control::Action(a) => ControlB::Action(a),
            Control::Quit => ControlB::Quit,
        }
    }
}

impl<Action, Error> From<Result<(), Error>> for ControlB<Action, Error> {
    fn from(value: Result<(), Error>) -> Self {
        match value {
            Ok(_) => ControlB::Continue,
            Err(e) => ControlB::Error(e),
        }
    }
}

impl<Action, Error> From<Result<Control<Action>, Error>> for ControlB<Action, Error> {
    fn from(value: Result<Control<Action>, Error>) -> Self {
        match value {
            Ok(Control::Continue) => ControlB::Continue,
            Ok(Control::Break) => ControlB::Break,
            Ok(Control::Repaint) => ControlB::Repaint,
            Ok(Control::Action(a)) => ControlB::Action(a),
            Ok(Control::Quit) => ControlB::Quit,
            Err(e) => ControlB::Error(e),
        }
    }
}

impl<Action, Error> Default for ControlQueue<Action, Error> {
    fn default() -> Self {
        Self {
            queue: RefCell::new(VecDeque::default()),
        }
    }
}

impl<Action, Error> ControlQueue<Action, Error> {
    fn is_empty(&self) -> bool {
        self.queue.borrow().is_empty()
    }

    fn take(&self) -> Option<ControlB<Action, Error>> {
        self.queue.borrow_mut().pop_front()
    }

    fn push(&self, ctrl: ControlB<Action, Error>) {
        self.queue.borrow_mut().push_back(ctrl);
    }
}

/// Initiates a background task given as a boxed closure [Task]
#[derive(Debug)]
struct Tasks<'a, Action, Error> {
    send: &'a Sender<(Cancel, Task<Action, Error>)>,
}

impl<'a, Action, Error> Clone for Tasks<'a, Action, Error> {
    fn clone(&self) -> Self {
        Self { send: self.send }
    }
}

impl<'a, Action, Error> Tasks<'a, Action, Error>
where
    Action: 'static + Send,
    Error: 'static + Send,
{
    /// Start a background task.
    ///
    /// The background task gets a `Arc<Mutex<bool>>` for cancellation support,
    /// and a channel to communicate back to the event loop.
    ///
    /// A clone of the `Arc<Mutex<bool>>` is returned by this function.
    ///
    /// If you need more, create an extra channel for communication to the background task.
    fn send(&self, task: Task<Action, Error>) -> Result<Cancel, SendError<()>> {
        let cancel = Arc::new(Mutex::new(false));
        match self.send.send((Arc::clone(&cancel), task)) {
            Ok(_) => Ok(cancel),
            Err(_) => Err(SendError(())),
        }
    }
}

/// Basic threadpool
#[derive(Debug)]
struct ThreadPool<Action, Error> {
    send: Sender<(Cancel, Task<Action, Error>)>,
    recv: Receiver<Result<Control<Action>, Error>>,
    handles: Vec<JoinHandle<()>>,
}

impl<Action, Error> ThreadPool<Action, Error>
where
    Action: 'static + Send,
    Error: 'static + Send,
{
    /// New thread-pool with the given task executor.
    fn build(n_worker: usize) -> Self {
        let (send, t_recv) = unbounded::<(Cancel, Task<Action, Error>)>();
        let (t_send, recv) = unbounded::<Result<Control<Action>, Error>>();

        let mut handles = Vec::new();

        for _ in 0..n_worker {
            let t_recv = t_recv.clone();
            let t_send = t_send.clone();

            let handle = thread::spawn(move || {
                let t_recv = t_recv;

                'l: loop {
                    match t_recv.recv() {
                        Ok((cancel, task)) => {
                            let flow = task(cancel, &t_send);
                            if let Err(err) = t_send.send(flow) {
                                debug!("{:?}", err);
                                break 'l;
                            }
                        }
                        Err(err) => {
                            debug!("{:?}", err);
                            break 'l;
                        }
                    }
                }
            });

            handles.push(handle);
        }

        Self {
            send,
            recv,
            handles,
        }
    }

    /// Check the workers for liveness.
    fn check_liveness(&self) -> bool {
        for h in &self.handles {
            if h.is_finished() {
                return false;
            }
        }
        true
    }

    /// Is the receive-channel empty?
    fn is_empty(&self) -> bool {
        self.recv.is_empty()
    }

    /// Receive a result.
    fn try_recv(&self) -> Result<Control<Action>, Error>
    where
        Error: From<TryRecvError>,
    {
        match self.recv.try_recv() {
            Ok(v) => v,
            Err(TryRecvError::Empty) => Ok(Control::Continue),
            Err(e) => Err(e.into()),
        }
    }

    /// Stop threads and join.
    fn stop_and_join(mut self) {
        shutdown_thread_pool(&mut self);
    }
}

impl<Action, Error> Drop for ThreadPool<Action, Error> {
    fn drop(&mut self) {
        shutdown_thread_pool(self);
    }
}

fn shutdown_thread_pool<Action, Error>(t: &mut ThreadPool<Action, Error>) {
    // dropping the channel will be noticed by the threads running the
    // background tasks.
    drop(mem::replace(&mut t.send, bounded(0).0));
    for h in t.handles.drain(..) {
        _ = h.join();
    }
}
