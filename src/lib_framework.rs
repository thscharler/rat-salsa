//! This implements an event-loop.
//!
//! It uses ControlUI as it's central control-flow construct.
//!

use crate::lib_repaint::{Repaint, RepaintEvent};
use crate::lib_timer::{Timed, TimerEvent, Timers};
use crate::ControlUI;
use crossbeam::channel::{bounded, unbounded, Receiver, SendError, Sender, TryRecvError};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, ExecutableCommand};
use log::debug;
use ratatui::backend::CrosstermBackend;
use ratatui::{Frame, Terminal};
use std::cmp::min;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::io::{stdout, Stdout};
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::thread::{sleep, JoinHandle};
use std::time::{Duration, SystemTime};
use std::{io, mem, thread};

/// Describes the requisites of a TuiApp.
///
/// It holds no state of its own, just a way to collect the types and operations.
///
pub trait TuiApp {
    /// Application data.
    type Data;
    /// UI state.
    type State;
    /// Action type.
    type Action;
    /// Error type.
    type Error;

    /// Get the repaint state for this uistate.
    #[allow(unused_variables)]
    fn get_repaint<'b>(&self, uistate: &'b Self::State) -> Option<&'b Repaint> {
        None
    }

    /// Get the timer state for this uistate.
    #[allow(unused_variables)]
    fn get_timers<'b>(&self, uistate: &'b Self::State) -> Option<&'b Timers> {
        None
    }

    /// Do some init immediately before the event-loop starts.
    fn init(
        &self,
        data: &mut Self::Data,
        uistate: &mut Self::State,
        send: &Sender<Self::Action>,
    ) -> Result<(), anyhow::Error>;

    /// Repaint the ui.
    fn repaint(
        &self,
        event: RepaintEvent,
        frame: &mut Frame<'_>,
        data: &mut Self::Data,
        uistate: &mut Self::State,
    ) -> ControlUI<Self::Action, Self::Error>;

    /// Handle a timer.
    fn handle_timer(
        &self,
        event: Timed,
        data: &mut Self::Data,
        uistate: &mut Self::State,
    ) -> ControlUI<Self::Action, Self::Error>;

    /// Handle an event.
    fn handle_event(
        &self,
        event: Event,
        data: &mut Self::Data,
        uistate: &mut Self::State,
    ) -> ControlUI<Self::Action, Self::Error>;

    /// Run an action.
    fn run_action(
        &self,
        action: Self::Action,
        data: &mut Self::Data,
        uistate: &mut Self::State,
        send: &Sender<Self::Action>,
    ) -> ControlUI<Self::Action, Self::Error>;

    /// Called by the worker thread to run a Task.
    fn run_task(
        &self,
        task: Self::Action,
        send: &Sender<ControlUI<Self::Action, Self::Error>>,
    ) -> ControlUI<Self::Action, Self::Error>;

    /// Error reporting.
    fn report_error(
        &self,
        error: Self::Error,
        data: &mut Self::Data,
        uistate: &mut Self::State,
    ) -> ControlUI<Self::Action, Self::Error>;
}

enum PollNext {
    RepaintFlag,
    Timers,
    Workers,
    Crossterm,
}

/// Captures some parameters for [run_tui()].
#[derive(Debug)]
pub struct RunConfig {
    /// How many worker threads are wanted?
    /// Most of the time 1 should be sufficient to offload any gui-blocking tasks.
    pub n_threats: usize,
    /// Logs the run timings for every action.
    pub log_timing: bool,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            n_threats: 1,
            log_timing: false,
        }
    }
}

macro_rules! log_timing {
    ($expr:expr, $event:expr, $cfg:expr) => {{
        let time = if $cfg.log_timing {
            Some((SystemTime::now(), format!("{:?}", $event)))
        } else {
            None
        };

        let r = $expr;

        if $cfg.log_timing {
            if let Some((time, event_msg)) = time {
                if let Ok(elapsed) = time.elapsed() {
                    debug!("{:?} for {}", elapsed, event_msg);
                }
            }
        }

        r
    }};
}

/// Run the event-loop.
pub fn run_tui<App: TuiApp>(
    app: &'static App,
    data: &mut App::Data,
    uistate: &mut App::State,
    cfg: RunConfig,
) -> Result<(), anyhow::Error>
where
    App::Action: Debug + Send + 'static,
    App::Error: Send + 'static + From<TryRecvError> + From<io::Error> + From<SendError<()>>,
    App: Sync,
{
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;
    enable_raw_mode()?;

    let r = match catch_unwind(AssertUnwindSafe(|| _run_tui(app, data, uistate, cfg))) {
        Ok(v) => v,
        Err(e) => {
            stdout().execute(DisableMouseCapture)?;
            stdout().execute(LeaveAlternateScreen)?;
            disable_raw_mode()?;

            resume_unwind(e);
        }
    };

    stdout().execute(DisableMouseCapture)?;
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    r
}

/// Run the event-loop.
fn _run_tui<App: TuiApp>(
    app: &'static App,
    data: &mut App::Data,
    uistate: &mut App::State,
    cfg: RunConfig,
) -> Result<(), anyhow::Error>
where
    App::Action: Debug + Send + 'static,
    App::Error: Send + 'static + From<TryRecvError> + From<io::Error> + From<SendError<()>>,
    App: Sync,
{
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut worker = ThreadPool::<App>::build(app, cfg.n_threats);

    let mut flow;
    let mut repaint_event = RepaintEvent::Change;

    // to not starve any event source everyone is polled and put in this queue.
    // they are not polled again before the queue is not empty.
    let mut poll_queue = VecDeque::new();

    // init
    app.init(data, uistate, &worker.send)?;

    // initial repaint.
    flow = repaint_tui(&mut terminal, app, data, uistate, repaint_event);

    'ui: loop {
        // panic on worker panic
        worker.check_liveness();

        flow = flow.or_else(|| 'f: {
            if poll_queue.is_empty() {
                if poll_repaint_flag(app, uistate) {
                    poll_queue.push_back(PollNext::RepaintFlag);
                }
                if poll_timers(app, uistate) {
                    poll_queue.push_back(PollNext::Timers);
                }
                if poll_workers(app, &worker) {
                    poll_queue.push_back(PollNext::Workers);
                }
                match poll_crossterm(app) {
                    Ok(true) => {
                        poll_queue.push_back(PollNext::Crossterm);
                    }
                    Err(err) => break 'f ControlUI::Err(err.into()),
                    _ => {}
                }
            }

            if poll_queue.is_empty() {
                let t = calculate_sleep(app, uistate, Duration::from_millis(10));
                sleep(t);
            }

            match poll_queue.pop_front() {
                None => ControlUI::Continue,
                Some(PollNext::RepaintFlag) => read_repaint_flag(app, uistate, &mut repaint_event),
                Some(PollNext::Timers) => read_timers(app, &cfg, data, uistate, &mut repaint_event),
                Some(PollNext::Workers) => read_workers(app, &worker),
                Some(PollNext::Crossterm) => read_crossterm(app, &cfg, data, uistate),
            }
        });

        flow = match flow {
            ControlUI::Continue => ControlUI::Continue,
            ControlUI::NoChange => ControlUI::Continue,
            ControlUI::Change => {
                flow = log_timing!(
                    repaint_tui(&mut terminal, app, data, uistate, repaint_event),
                    repaint_event,
                    cfg
                );
                repaint_event = RepaintEvent::Change;
                flow
            }
            ControlUI::Run(action) => {
                log_timing!(
                    app.run_action(action, data, uistate, &worker.send),
                    action,
                    cfg
                )
            }
            ControlUI::Spawn(action) => worker.send(action),
            ControlUI::Err(err) => app.report_error(err, data, uistate),
            ControlUI::Break => break 'ui,
        };
    }

    worker.stop_and_join();

    Ok(())
}

fn calculate_sleep<App: TuiApp>(app: &App, uistate: &mut App::State, max: Duration) -> Duration {
    if let Some(timers) = app.get_timers(uistate) {
        if let Some(sleep) = timers.sleep_time() {
            min(sleep, max)
        } else {
            max
        }
    } else {
        max
    }
}

fn poll_repaint_flag<App: TuiApp>(app: &App, uistate: &mut App::State) -> bool {
    if let Some(repaint) = app.get_repaint(uistate) {
        repaint.get()
    } else {
        false
    }
}

fn read_repaint_flag<App: TuiApp>(
    app: &App,
    uistate: &mut App::State,
    repaint_event: &mut RepaintEvent,
) -> ControlUI<App::Action, App::Error> {
    if let Some(repaint) = app.get_repaint(uistate) {
        if repaint.get() {
            repaint.reset();
            *repaint_event = RepaintEvent::Change;
            ControlUI::Change
        } else {
            ControlUI::Continue
        }
    } else {
        ControlUI::Continue
    }
}

fn poll_timers<App: TuiApp>(app: &App, uistate: &mut App::State) -> bool {
    if let Some(timers) = app.get_timers(uistate) {
        timers.poll()
    } else {
        false
    }
}

fn read_timers<App: TuiApp>(
    app: &App,
    cfg: &RunConfig,
    data: &mut App::Data,
    uistate: &mut App::State,
    repaint_event: &mut RepaintEvent,
) -> ControlUI<App::Action, App::Error> {
    if let Some(timers) = app.get_timers(uistate) {
        match timers.read() {
            Some(TimerEvent::Repaint(evt)) => {
                *repaint_event = RepaintEvent::Timer(evt);
                ControlUI::Change
            }
            Some(TimerEvent::Application(evt)) => {
                log_timing!(app.handle_timer(evt, data, uistate), evt, cfg)
            }
            None => ControlUI::Continue,
        }
    } else {
        ControlUI::Continue
    }
}

fn poll_workers<App: TuiApp>(_app: &App, worker: &ThreadPool<App>) -> bool
where
    App::Action: Send + 'static,
    App::Error: Send + 'static + From<TryRecvError>,
    App: Sync,
{
    !worker.is_empty()
}

fn read_workers<App: TuiApp>(
    _app: &App,
    worker: &ThreadPool<App>,
) -> ControlUI<App::Action, App::Error>
where
    App::Action: Send + 'static,
    App::Error: Send + 'static + From<TryRecvError>,
    App: Sync,
{
    worker.try_recv()
}

fn poll_crossterm<App: TuiApp>(_app: &App) -> Result<bool, io::Error> {
    event::poll(Duration::from_millis(0))
}

fn read_crossterm<App: TuiApp>(
    app: &App,
    cfg: &RunConfig,
    data: &mut App::Data,
    uistate: &mut App::State,
) -> ControlUI<<App as TuiApp>::Action, <App as TuiApp>::Error>
where
    App::Error: From<io::Error>,
{
    match event::poll(Duration::from_millis(0)) {
        Ok(true) => match event::read() {
            Ok(evt) => log_timing!(app.handle_event(evt, data, uistate), evt, cfg),
            Err(err) => ControlUI::Err(err.into()),
        },
        Ok(false) => ControlUI::Continue,
        Err(err) => ControlUI::Err(err.into()),
    }
}

fn repaint_tui<App: TuiApp>(
    term: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &App,
    data: &mut App::Data,
    uistate: &mut App::State,
    reason: RepaintEvent,
) -> ControlUI<App::Action, App::Error>
where
    App::Error: From<io::Error>,
{
    let mut flow = ControlUI::Continue;

    _ = term.hide_cursor();

    let result = term.draw(|frame| {
        flow = app.repaint(reason, frame, data, uistate);
    });

    // a draw() error overwrites a possible repaint() error.
    // but that's probably ok.
    if let Err(err) = result {
        flow = ControlUI::Err(err.into())
    }

    flow
}

/// Basic threadpool
#[derive(Debug)]
struct ThreadPool<App: TuiApp> {
    send: Sender<App::Action>,
    recv: Receiver<ControlUI<App::Action, App::Error>>,
    handles: Vec<JoinHandle<()>>,
}

impl<App: TuiApp> ThreadPool<App>
where
    App: Sync,
    App::Action: 'static + Send,
    App::Error: 'static + Send,
{
    /// New threadpool with the given task executor.
    fn build(app: &'static App, n_worker: usize) -> Self {
        let (send, t_recv) = unbounded::<App::Action>();
        let (t_send, recv) = unbounded::<ControlUI<App::Action, App::Error>>();

        let mut handles = Vec::new();

        for _ in 0..n_worker {
            let t_recv = t_recv.clone();
            let t_send = t_send.clone();

            let handle = thread::spawn(move || {
                let t_recv = t_recv;

                'l: loop {
                    match t_recv.recv() {
                        Ok(task) => {
                            let flow = app.run_task(task, &t_send);
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
    ///
    /// Panic:
    /// Panics if any of the workers panicked themselves.
    fn check_liveness(&mut self) {
        let mut all_alive = true;
        for h in &self.handles {
            if h.is_finished() {
                all_alive = false;
            }
        }

        if !all_alive {
            shutdown_thread_pool(self);
            panic!("worker panicked");
        }
    }

    /// Send a task.
    fn send(&self, t: App::Action) -> ControlUI<App::Action, App::Error>
    where
        App::Error: From<SendError<()>>,
    {
        match self.send.send(t) {
            Ok(_) => ControlUI::Continue,
            Err(_) => ControlUI::Err(SendError(()).into()),
        }
    }

    /// Is the channel empty?
    fn is_empty(&self) -> bool {
        self.recv.is_empty()
    }

    /// Receive a result.
    fn try_recv(&self) -> ControlUI<App::Action, App::Error>
    where
        App::Error: From<TryRecvError>,
    {
        match self.recv.try_recv() {
            Ok(v) => v,
            Err(TryRecvError::Empty) => ControlUI::Continue,
            Err(e) => ControlUI::Err(e.into()),
        }
    }

    /// Stop threads and join.
    fn stop_and_join(mut self) {
        shutdown_thread_pool(&mut self);
    }
}

impl<App: TuiApp> Drop for ThreadPool<App> {
    fn drop(&mut self) {
        shutdown_thread_pool(self);
    }
}

fn shutdown_thread_pool<App: TuiApp>(t: &mut ThreadPool<App>) {
    drop(mem::replace(&mut t.send, bounded(0).0));
    for h in t.handles.drain(..) {
        _ = h.join();
    }
}
