use crate::control_queue::ControlQueue;
use crate::poll::{PollCrossterm, PollEvents, PollTasks, PollTimers};
use crate::terminal::{CrosstermTerminal, Terminal};
use crate::threadpool::ThreadPool;
use crate::timer::Timers;
use crate::{AppContext, AppEvents, AppWidget, Control};
use crossbeam::channel::{SendError, TryRecvError};
use std::cell::RefCell;
use std::cmp::min;
use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::time::Duration;
use std::{io, thread};

/// Captures some parameters for [crate::run_tui()].
pub struct RunConfig<App, Global, Action, Error>
where
    App: AppWidget<Global, Action, Error>,
    Action: 'static + Send + Debug,
    Error: 'static + Send + Debug,
{
    /// How many worker threads are wanted?
    /// Most of the time 1 should be sufficient to offload any gui-blocking tasks.
    pub n_threats: usize,
    /// This is the renderer that connects to the backend, and calls out
    /// for rendering the application.
    ///
    /// Defaults to RenderCrossterm.
    pub term: Box<dyn Terminal<App, Global, Action, Error>>,
    /// List of all event-handlers for the application.
    ///
    /// Defaults to PollTimers, PollCrossterm, PollTasks. Add yours here.
    pub poll: Vec<Box<dyn PollEvents<App, Global, Action, Error>>>,
}

impl<App, Global, Action, Error> Debug for RunConfig<App, Global, Action, Error>
where
    App: AppWidget<Global, Action, Error>,
    Action: 'static + Send + Debug,
    Error: 'static + Send + Debug,
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
    Action: 'static + Send + Debug,
    Error: 'static + Send + Debug + From<io::Error> + From<TryRecvError>,
{
    /// New configuration with some defaults.
    pub fn default() -> Result<Self, Error> {
        Ok(Self {
            n_threats: 1,
            term: Box::new(CrosstermTerminal::new()?),
            poll: vec![
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
    Action: Send + 'static + Debug,
    Error: Send + 'static + Debug + From<TryRecvError> + From<io::Error> + From<SendError<()>>,
{
    let term = cfg.term.as_mut();
    let poll = &mut cfg.poll;

    let timers = Timers::default();
    let tasks = ThreadPool::new(cfg.n_threats);
    let queue = ControlQueue::default();

    let mut appctx = AppContext {
        g: global,
        timeout: None,
        timers: &timers,
        tasks: &tasks,
        queue: &queue,
    };

    let poll_queue = PollQueue::default();
    let mut poll_sleep = Duration::from_millis(10);

    // init state
    state.init(&mut appctx)?;

    // initial render
    term.render(&mut app, state, &mut appctx)?;

    'ui: loop {
        // panic on worker panic
        if !tasks.check_liveness() {
            dbg!("worker panicked");
            break 'ui;
        }

        if queue.is_empty() {
            if poll_queue.is_empty() {
                for (n, p) in poll.iter_mut().enumerate() {
                    match p.poll(&mut appctx) {
                        Ok(true) => {
                            poll_queue.push(PollHandle(n));
                        }
                        Ok(false) => {}
                        Err(e) => {
                            appctx.queue_result(Err(e));
                        }
                    }
                }
            }

            if poll_queue.is_empty() {
                let t = if let Some(timer_sleep) = timers.sleep_time() {
                    min(timer_sleep, poll_sleep)
                } else {
                    poll_sleep
                };
                thread::sleep(t);
                if poll_sleep < Duration::from_millis(10) {
                    // Back off slowly.
                    poll_sleep += Duration::from_micros(100);
                }
            } else {
                // Shorter sleep immediately after an event.
                poll_sleep = Duration::from_micros(100);
            }
        }

        if queue.is_empty() {
            if let Some(h) = poll_queue.take() {
                let r = poll[h.0].read_exec(&mut app, state, term, &mut appctx);
                appctx.queue_result(r);
            }
        }

        let n = queue.take();
        match n {
            None => {}
            Some(Err(e)) => {
                let r = state.error(e, &mut appctx);
                appctx.queue_result(r);
            }
            Some(Ok(Control::Continue)) => {}
            Some(Ok(Control::Break)) => {}
            Some(Ok(Control::Repaint)) => {
                if let Err(e) = term.render(&mut app, state, &mut appctx) {
                    appctx.queue_result(Err(e));
                }
            }
            Some(Ok(Control::Action(mut a))) => {
                let r = state.action(&mut a, &mut appctx);
                appctx.queue_result(r);
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
    Action: Send + 'static + Debug,
    Error: Send + 'static + Debug + From<TryRecvError> + From<io::Error> + From<SendError<()>>,
{
    cfg.term.init()?;

    let r = match catch_unwind(AssertUnwindSafe(|| _run_tui(app, global, state, &mut cfg))) {
        Ok(v) => v,
        Err(e) => {
            _ = cfg.term.shutdown();
            resume_unwind(e);
        }
    };

    cfg.term.shutdown()?;

    r
}
