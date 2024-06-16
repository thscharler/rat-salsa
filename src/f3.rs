use crate::f3::app::{AppEvents, AppWidget};
use crate::f3::context::AppContext;
use crate::f3::poll::{EventPoll, PollCrossterm, PollTasks, PollTimers};
use crate::f3::queue::ControlQueue;
use crate::f3::render::{RenderCrossterm, RenderUI};
use crate::f3::threadpool::ThreadPool;
use crate::timer::Timers;
use crate::Control;
use crossbeam::channel::{SendError, TryRecvError};
use std::cell::RefCell;
use std::cmp::min;
use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::time::Duration;
use std::{io, thread};

pub mod app {
    use crate::f3::context::{AppContext, RenderContext};
    use crate::{Control, TimeOut};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use std::fmt::Debug;

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
        Action: 'static + Send,
        Error: 'static + Send,
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
        Action: 'static + Send,
        Error: 'static + Send,
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
}

pub mod context {
    use crate::f3::queue::ControlQueue;
    use crate::f3::threadpool::{Cancel, Task, ThreadPool};
    use crate::timer::Timers;
    use crate::{Control, TimeOut, TimerDef, TimerHandle};
    use crossbeam::channel::SendError;
    use ratatui::layout::Rect;

    /// A collection of context data used by the application.
    #[derive(Debug)]
    pub struct AppContext<'a, Global, Action, Error>
    where
        Action: 'static + Send,
        Error: 'static + Send,
    {
        /// Some global state for the application.
        pub g: &'a mut Global,
        /// Current timeout, if any.
        pub timeout: Option<TimeOut>,

        /// Application timers.
        timers: &'a Timers,
        /// Background tasks.
        tasks: &'a ThreadPool<Action, Error>,
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

    impl<'a, Global> RenderContext<'a, Global> {
        pub fn new(g: &'a mut Global, counter: usize, area: Rect) -> Self {
            RenderContext {
                g,
                timeout: None,
                counter,
                area,
                cursor: None,
            }
        }
    }

    impl<'a, Global, Action, Error> AppContext<'a, Global, Action, Error>
    where
        Action: 'static + Send,
        Error: 'static + Send,
    {
        pub fn new(
            g: &'a mut Global,
            timers: &'a Timers,
            tasks: &'a ThreadPool<Action, Error>,
            queue: &'a ControlQueue<Action, Error>,
        ) -> Self {
            Self {
                g,
                timeout: None,
                timers,
                tasks,
                queue,
            }
        }
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

        #[inline]
        pub fn timers(&self) -> &'a Timers {
            self.timers
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

        #[inline]
        pub fn tasks(&self) -> &'a ThreadPool<Action, Error> {
            self.tasks
        }

        /// Queue additional results.
        #[inline]
        pub fn queue(&self, ctrl: Result<Control<Action>, Error>) {
            self.queue.push(ctrl)
        }

        #[inline]
        pub fn ctrl(&self) -> &'a ControlQueue<Action, Error> {
            self.queue
        }
    }
}

pub mod render {
    use crate::f3::app::AppWidget;
    use crate::f3::context::{AppContext, RenderContext};
    use crossterm::cursor::{DisableBlinking, EnableBlinking, SetCursorStyle};
    use crossterm::event::{
        DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
    };
    use crossterm::terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    };
    use crossterm::ExecutableCommand;
    use ratatui::backend::CrosstermBackend;
    use ratatui::Terminal;
    use std::io;
    use std::io::{stdout, Stdout};

    /// Encapsulates Terminal and Backend.
    ///
    /// This is used as dyn Trait to hide the Background type parameter.
    ///
    /// If you want to send other than the default Commands to the backend,
    /// implement this trait.
    pub trait RenderUI<App, Global, Action, Error>
    where
        App: AppWidget<Global, Action, Error>,
        Action: 'static + Send,
        Error: 'static + Send,
    {
        /// Terminal init.
        fn init(&mut self) -> Result<(), Error>
        where
            Error: From<io::Error>;

        /// Terminal shutdown.
        fn shutdown(&mut self) -> Result<(), Error>
        where
            Error: From<io::Error>;

        /// Render the app widget.
        ///
        /// Creates the render-context, fetches the frame and calls render.
        fn render<'a, 'b>(
            &mut self,
            app: &mut App,
            state: &mut App::State,
            ctx: &'b mut AppContext<'a, Global, Action, Error>,
        ) -> Result<(), Error>
        where
            Error: From<io::Error>;
    }

    /// Default RenderUI for crossterm.
    #[derive(Debug)]
    pub struct RenderCrossterm {
        term: Terminal<CrosstermBackend<Stdout>>,
    }

    impl RenderCrossterm {
        pub fn new() -> Result<Self, io::Error> {
            Ok(Self {
                term: Terminal::new(CrosstermBackend::new(stdout()))?,
            })
        }
    }

    impl<App, Global, Action, Error> RenderUI<App, Global, Action, Error> for RenderCrossterm
    where
        App: AppWidget<Global, Action, Error>,
        Action: 'static + Send,
        Error: 'static + Send,
    {
        fn init(&mut self) -> Result<(), Error>
        where
            Error: From<io::Error>,
        {
            stdout().execute(EnterAlternateScreen)?;
            stdout().execute(EnableMouseCapture)?;
            stdout().execute(EnableBracketedPaste)?;
            stdout().execute(EnableBlinking)?;
            stdout().execute(SetCursorStyle::BlinkingBar)?;
            enable_raw_mode()?;

            self.term.clear()?;
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), Error>
        where
            Error: From<io::Error>,
        {
            disable_raw_mode()?;
            stdout().execute(SetCursorStyle::DefaultUserShape)?;
            stdout().execute(DisableBlinking)?;
            stdout().execute(DisableBracketedPaste)?;
            stdout().execute(DisableMouseCapture)?;
            stdout().execute(LeaveAlternateScreen)?;
            Ok(())
        }

        fn render<'a, 'b>(
            &mut self,
            app: &mut App,
            state: &mut App::State,
            ctx: &'b mut AppContext<'a, Global, Action, Error>,
        ) -> Result<(), Error>
        where
            App: AppWidget<Global, Action, Error>,
            Error: From<io::Error>,
        {
            let mut res = Ok(());

            _ = self.term.hide_cursor();

            self.term.draw(|frame| {
                let mut ctx = RenderContext::new(ctx.g, frame.count(), frame.size());

                let frame_area = frame.size();
                res = app.render(frame_area, frame.buffer_mut(), state, &mut ctx);

                if let Some((cursor_x, cursor_y)) = ctx.cursor {
                    frame.set_cursor(cursor_x, cursor_y);
                }
            })?;

            res
        }
    }
}

pub mod poll {
    use crate::f3::app::{AppEvents, AppWidget};
    use crate::f3::context::AppContext;
    use crate::f3::render::RenderUI;
    use crate::{Control, TimerEvent};
    use crossbeam::channel::TryRecvError;
    use std::time::Duration;

    /// Trait for an event-source.
    ///
    /// If you need to add your own do the following:
    ///
    /// * Implement this trait for a struct that fits.
    ///     TODO: try this
    ///
    pub trait EventPoll<App, Global, Action, Error>
    where
        App: AppWidget<Global, Action, Error>,
        Action: 'static + Send,
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
            ctx: &mut AppContext<'_, Global, Action, Error>,
        ) -> Result<bool, Error>;

        /// Read the event and distribute it.
        ///
        /// If you add a new event, that doesn't fit into AppEvents, you'll
        /// have to define a new trait for your AppState and use that.
        fn read_execute(
            &mut self,
            app: &mut App,
            state: &mut App::State,
            render: &mut dyn RenderUI<App, Global, Action, Error>,
            ctx: &mut AppContext<'_, Global, Action, Error>,
        ) -> Result<Control<Action>, Error>;
    }

    /// Processes results from background tasks.
    #[derive(Debug)]
    pub struct PollTasks;

    impl<App, Global, Action, Error> EventPoll<App, Global, Action, Error> for PollTasks
    where
        App: AppWidget<Global, Action, Error>,
        Action: 'static + Send,
        Error: 'static + Send + From<TryRecvError>,
    {
        fn poll(&mut self, ctx: &mut AppContext<'_, Global, Action, Error>) -> Result<bool, Error> {
            Ok(ctx.tasks().is_empty())
        }

        fn read_execute(
            &mut self,
            _app: &mut App,
            _state: &mut App::State,
            _render: &mut dyn RenderUI<App, Global, Action, Error>,
            ctx: &mut AppContext<'_, Global, Action, Error>,
        ) -> Result<Control<Action>, Error> {
            ctx.tasks().try_recv()
        }
    }

    /// Processes timers.
    #[derive(Debug)]
    pub struct PollTimers;

    impl<App, Global, Action, Error> EventPoll<App, Global, Action, Error> for PollTimers
    where
        App: AppWidget<Global, Action, Error>,
        Action: 'static + Send,
        Error: 'static + Send + From<std::io::Error>,
    {
        fn poll(&mut self, ctx: &mut AppContext<'_, Global, Action, Error>) -> Result<bool, Error> {
            Ok(ctx.timers().poll())
        }

        fn read_execute(
            &mut self,
            app: &mut App,
            state: &mut App::State,
            render: &mut dyn RenderUI<App, Global, Action, Error>,
            ctx: &mut AppContext<'_, Global, Action, Error>,
        ) -> Result<Control<Action>, Error> {
            match ctx.timers().read() {
                None => Ok(Control::Continue),
                Some(TimerEvent::Repaint(t)) => {
                    ctx.timeout = Some(t);
                    if let Err(e) = render.render(app, state, ctx) {
                        ctx.timeout = None;
                        Err(e)
                    } else {
                        ctx.timeout = None;
                        Ok(Control::Continue)
                    }
                }
                Some(TimerEvent::Application(t)) => {
                    ctx.timeout = Some(t);
                    let r = state.timer(&t, ctx);
                    ctx.timeout = None;
                    r
                }
            }
        }
    }

    /// Processes crossterm events.
    #[derive(Debug)]
    pub struct PollCrossterm;

    impl<App, Global, Action, Error> EventPoll<App, Global, Action, Error> for PollCrossterm
    where
        App: AppWidget<Global, Action, Error>,
        Action: 'static + Send,
        Error: 'static + Send + From<std::io::Error>,
    {
        fn poll(
            &mut self,
            _ctx: &mut AppContext<'_, Global, Action, Error>,
        ) -> Result<bool, Error> {
            Ok(crossterm::event::poll(Duration::from_millis(0))?)
        }

        fn read_execute(
            &mut self,
            _app: &mut App,
            state: &mut App::State,
            _render: &mut dyn RenderUI<App, Global, Action, Error>,
            ctx: &mut AppContext<'_, Global, Action, Error>,
        ) -> Result<Control<Action>, Error> {
            match crossterm::event::read() {
                // TODO: can this be abstracted out too? sure.
                Ok(event) => state.crossterm(&event, ctx),
                Err(e) => Err(e.into()),
            }
        }
    }
}

pub mod threadpool {
    use crate::Control;
    use crossbeam::channel::{bounded, unbounded, Receiver, SendError, Sender, TryRecvError};
    use log::debug;
    use std::sync::{Arc, Mutex};
    use std::thread::JoinHandle;
    use std::{mem, thread};

    /// Type for a background task.
    pub type Task<Action, Error> = Box<
        dyn FnOnce(
                Cancel,
                &Sender<Result<Control<Action>, Error>>,
            ) -> Result<Control<Action>, Error>
            + Send,
    >;

    /// Type for cancelation.
    pub type Cancel = Arc<Mutex<bool>>;

    /// Basic threadpool
    #[derive(Debug)]
    pub struct ThreadPool<Action, Error>
    where
        Action: 'static + Send,
        Error: 'static + Send,
    {
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
        pub fn new(n_worker: usize) -> Self {
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

        /// Start a background task.
        ///
        /// The background task gets a `Arc<Mutex<bool>>` for cancellation support,
        /// and a channel to communicate back to the event loop.
        ///
        /// A clone of the `Arc<Mutex<bool>>` is returned by this function.
        ///
        /// If you need more, create an extra channel for communication to the background task.
        pub fn send(&self, task: Task<Action, Error>) -> Result<Cancel, SendError<()>> {
            let cancel = Arc::new(Mutex::new(false));
            match self.send.send((Arc::clone(&cancel), task)) {
                Ok(_) => Ok(cancel),
                Err(_) => Err(SendError(())),
            }
        }

        /// Check the workers for liveness.
        pub fn check_liveness(&self) -> bool {
            for h in &self.handles {
                if h.is_finished() {
                    return false;
                }
            }
            true
        }

        /// Is the receive-channel empty?
        pub fn is_empty(&self) -> bool {
            self.recv.is_empty()
        }

        /// Receive a result.
        pub fn try_recv(&self) -> Result<Control<Action>, Error>
        where
            Error: From<TryRecvError>,
        {
            match self.recv.try_recv() {
                Ok(v) => v,
                Err(TryRecvError::Empty) => Ok(Control::Continue),
                Err(e) => Err(e.into()),
            }
        }
    }

    impl<Action, Error> Drop for ThreadPool<Action, Error>
    where
        Action: 'static + Send,
        Error: 'static + Send,
    {
        fn drop(&mut self) {
            // dropping the channel will be noticed by the threads running the
            // background tasks.
            drop(mem::replace(&mut self.send, bounded(0).0));
            for h in self.handles.drain(..) {
                _ = h.join();
            }
        }
    }
}

pub mod queue {
    use crate::Control;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    /// Queue for event-handling results.
    #[derive(Debug)]
    pub struct ControlQueue<Action, Error>
    where
        Action: 'static + Send,
        Error: 'static + Send,
    {
        queue: RefCell<VecDeque<Result<Control<Action>, Error>>>,
    }

    impl<Action, Error> Default for ControlQueue<Action, Error>
    where
        Action: 'static + Send,
        Error: 'static + Send,
    {
        fn default() -> Self {
            Self {
                queue: RefCell::new(VecDeque::default()),
            }
        }
    }

    impl<Action, Error> ControlQueue<Action, Error>
    where
        Action: 'static + Send,
        Error: 'static + Send,
    {
        pub fn new() -> Self {
            Self {
                queue: RefCell::new(Default::default()),
            }
        }

        pub fn is_empty(&self) -> bool {
            self.queue.borrow().is_empty()
        }

        pub fn take(&self) -> Option<Result<Control<Action>, Error>> {
            self.queue.borrow_mut().pop_front()
        }

        pub fn push(&self, ctrl: Result<Control<Action>, Error>) {
            self.queue.borrow_mut().push_back(ctrl);
        }
    }
}

/// Captures some parameters for [crate::run_tui()].
pub struct RunConfig<App, Global, Action, Error>
where
    App: AppWidget<Global, Action, Error>,
    Action: 'static + Send,
    Error: 'static + Send,
{
    /// How many worker threads are wanted?
    /// Most of the time 1 should be sufficient to offload any gui-blocking tasks.
    pub n_threats: usize,
    /// This is the renderer that connects to the backend, and calls out
    /// for rendering the application.
    ///
    /// Defaults to RenderCrossterm.
    pub render: Box<dyn RenderUI<App, Global, Action, Error>>,
    /// List of all event-handlers for the application.
    ///
    /// Defaults to PollTimers, PollCrossterm, PollTasks. Add yours here.
    pub events: Vec<Box<dyn EventPoll<App, Global, Action, Error>>>,
}

impl<App, Global, Action, Error> Debug for RunConfig<App, Global, Action, Error>
where
    App: AppWidget<Global, Action, Error>,
    Action: 'static + Send,
    Error: 'static + Send,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunConfig")
            .field("n_threads", &self.n_threats)
            .field("render", &"...")
            .field("events", &"...")
            .finish()
    }
}

impl<App, Global, Action, Error> RunConfig<App, Global, Action, Error>
where
    App: AppWidget<Global, Action, Error>,
    Action: 'static + Send,
    Error: 'static + Send + From<io::Error> + From<TryRecvError>,
{
    /// New configuration with some defaults.
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            n_threats: 1,
            render: Box::new(RenderCrossterm::new()?),
            events: vec![
                Box::new(PollTimers),
                Box::new(PollCrossterm),
                Box::new(PollTasks),
            ],
        })
    }
}

/// Handle for which EventPoll wants to be read.
#[derive(Debug)]
struct PollHandle(usize);

/// Queue for which EventPoll wants to be read.
#[derive(Debug, Default)]
struct PollQueue {
    queue: RefCell<VecDeque<PollHandle>>,
}

impl PollQueue {
    /// Empty
    fn is_empty(&self) -> bool {
        self.queue.borrow().is_empty()
    }

    /// Take the next handle.
    fn take(&self) -> Option<PollHandle> {
        self.queue.borrow_mut().pop_front()
    }

    /// Push a handle to the queue.
    fn push(&self, poll: PollHandle) {
        self.queue.borrow_mut().push_back(poll);
    }
}

fn _run_tui<App, Global, Action, Error>(
    mut app: App,
    global: &mut Global,
    state: &mut App::State,
    cfg: &mut RunConfig<App, Global, Action, Error>,
) -> Result<(), Error>
where
    App: AppWidget<Global, Action, Error>,
    Action: Send + 'static,
    Error: Send + 'static + From<TryRecvError> + From<io::Error> + From<SendError<()>>,
{
    let render = cfg.render.as_mut();
    let events = &mut cfg.events;

    let __timers = Timers::default();
    let __thread_pool = ThreadPool::new(cfg.n_threats);
    let __queue = ControlQueue::default();

    let mut appctx = AppContext::new(global, &__timers, &__thread_pool, &__queue);

    let poll = PollQueue::default();
    let mut sleep = Duration::from_millis(10);

    // init state
    state.init(&mut appctx)?;

    // initial render
    render.render(&mut app, state, &mut appctx)?;

    'ui: loop {
        // panic on worker panic
        if !appctx.tasks().check_liveness() {
            dbg!("worker panicked");
            break 'ui;
        }

        if appctx.ctrl().is_empty() {
            if poll.is_empty() {
                for (n, p) in events.iter_mut().enumerate() {
                    match p.poll(&mut appctx) {
                        Ok(true) => {
                            poll.push(PollHandle(n));
                        }
                        Ok(false) => {}
                        Err(e) => {
                            appctx.queue(Err(e));
                        }
                    }
                }
            }

            if poll.is_empty() {
                let t = if let Some(timer_sleep) = appctx.timers().sleep_time() {
                    min(timer_sleep, sleep)
                } else {
                    sleep
                };
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

        if appctx.ctrl().is_empty() {
            if let Some(h) = poll.take() {
                let r = events[h.0].read_execute(&mut app, state, render, &mut appctx);
                appctx.queue(r);
            }
        }

        let n = appctx.ctrl().take();
        match n {
            None => {}
            Some(Err(e)) => {
                let r = state.error(e, &mut appctx);
                appctx.queue(r);
            }
            Some(Ok(Control::Continue)) => {}
            Some(Ok(Control::Break)) => {}
            Some(Ok(Control::Repaint)) => {
                if let Err(e) = render.render(&mut app, state, &mut appctx) {
                    appctx.queue(Err(e));
                }
            }
            Some(Ok(Control::Action(mut a))) => {
                let r = state.action(&mut a, &mut appctx);
                appctx.queue(r);
            }
            Some(Ok(Control::Quit)) => {
                break 'ui;
            }
        }
    }

    Ok(())
}

/// Run the event-loop
pub fn run_tui<Widget, Global, Action, Error>(
    app: Widget,
    global: &mut Global,
    state: &mut Widget::State,
    mut cfg: RunConfig<Widget, Global, Action, Error>,
) -> Result<(), Error>
where
    Widget: AppWidget<Global, Action, Error>,
    Action: Send + 'static,
    Error: Send + 'static + From<TryRecvError> + From<io::Error> + From<SendError<()>>,
{
    cfg.render.init()?;

    let r = match catch_unwind(AssertUnwindSafe(|| _run_tui(app, global, state, &mut cfg))) {
        Ok(v) => v,
        Err(e) => {
            _ = cfg.render.shutdown();
            resume_unwind(e);
        }
    };

    cfg.render.shutdown()?;

    r
}
