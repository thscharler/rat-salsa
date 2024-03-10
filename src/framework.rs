//! This implements an event-loop.
//!
//! It uses ControlUI as it's central control-flow construct.
//!

use crate::ControlUI;
use crossbeam::channel::{unbounded, Receiver, SendError, Sender, TryRecvError};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, ExecutableCommand};
use log::debug;
use ratatui::backend::CrosstermBackend;
use ratatui::{Frame, Terminal};
use std::io::{stdout, Stdout};
use std::thread::JoinHandle;
use std::time::Duration;
use std::{io, thread};

/// Describes the requisites of a TuiApp.
///
/// It holds no state of its own, just a way to collect the types and operations.
///
pub trait TuiApp {
    /// Application data.
    type Data;
    /// UI state.
    type State;
    /// Task-type for the worker thread.
    type Task;
    /// Action type.
    type Action;
    /// Error type.
    type Error;

    /// Repaint the ui.
    fn repaint(
        &self,
        frame: &mut Frame<'_>,
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
    ) -> ControlUI<Self::Action, Self::Error>;

    /// Create and start a task.
    fn start_task(
        &self,
        action: Self::Action,
        data: &Self::Data,
        uistate: &Self::State,
        worker: &ThreadPool<Self>,
    ) -> ControlUI<Self::Action, Self::Error>;

    /// Called by the worker thread to run a Task.
    fn run_task(
        &self,
        task: Self::Task,
        send: &TaskSender<Self>,
    ) -> ControlUI<Self::Action, Self::Error>;

    /// Error reporting.
    fn report_error(
        &self,
        error: Self::Error,
        data: &mut Self::Data,
        uistate: &mut Self::State,
    ) -> ControlUI<Self::Action, Self::Error>;
}

/// Run the event-loop.
pub fn run_tui<App: TuiApp>(
    app: &'static App,
    data: &mut App::Data,
    uistate: &mut App::State,
    n_worker: usize,
) -> Result<(), anyhow::Error>
where
    App::Action: Send + 'static,
    App::Error: Send + From<TryRecvError> + From<io::Error> + From<SendError<()>> + 'static,
    App::Task: Send + 'static,
    App: Sync,
{
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let worker = ThreadPool::<App>::build(&app, n_worker);

    let mut flow;

    // initial repaint.
    flow = repaint_tui(&mut terminal, app, data, uistate);

    'ui: loop {
        flow = flow.or_else(|| worker.try_recv());

        flow = flow.or_else(|| match event::poll(Duration::from_millis(10)) {
            Ok(true) => match event::read() {
                Ok(evt) => app.handle_event(evt, data, uistate),
                Err(err) => ControlUI::Err(err.into()),
            },
            Ok(false) => ControlUI::Continue,
            Err(err) => ControlUI::Err(err.into()),
        });

        flow = match flow {
            ControlUI::Continue => ControlUI::Continue,
            ControlUI::Unchanged => ControlUI::Continue,
            ControlUI::Changed => repaint_tui(&mut terminal, app, data, uistate),
            ControlUI::Action(action) => app.run_action(action, data, uistate),
            ControlUI::Spawn(action) => app.start_task(action, data, uistate, &worker),
            ControlUI::Err(err) => app.report_error(err, data, uistate),
            ControlUI::Break => break 'ui,
        }
    }

    worker.stop_and_join()?;

    stdout().execute(DisableMouseCapture)?;
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}

fn repaint_tui<App: TuiApp>(
    term: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &App,
    data: &mut App::Data,
    uistate: &mut App::State,
) -> ControlUI<App::Action, App::Error>
where
    App::Error: From<io::Error>,
{
    let mut flow = ControlUI::Continue;

    let result = term.draw(|frame| {
        flow = app.repaint(frame, data, uistate);
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
pub struct ThreadPool<App: TuiApp + ?Sized> {
    send: Sender<TaskArgs<App::Task>>,
    recv: Receiver<ControlUI<App::Action, App::Error>>,
    handles: Vec<JoinHandle<()>>,
}

/// Send results.
#[derive(Debug)]
pub struct TaskSender<App: TuiApp + ?Sized> {
    pub send: Sender<ControlUI<App::Action, App::Error>>,
}

// internal
enum TaskArgs<Task> {
    Break,
    Task(Task),
}

impl<App: TuiApp> ThreadPool<App>
where
    App: Sync,
    App::Task: 'static + Send,
    App::Action: 'static + Send,
    App::Error: 'static + Send,
{
    /// New threadpool with the given task executor.
    pub fn build(app: &'static App, n_worker: usize) -> Self {
        let (send, t_recv) = unbounded::<TaskArgs<App::Task>>();
        let (t_send, recv) = unbounded::<ControlUI<App::Action, App::Error>>();

        let mut handles = Vec::new();

        for _ in 0..n_worker {
            let t_recv = t_recv.clone();
            let t_send = t_send.clone();

            let handle = thread::spawn(move || {
                let t_recv = t_recv;
                let t_send = TaskSender { send: t_send };
                let app = app;

                'l: loop {
                    match t_recv.recv() {
                        Ok(TaskArgs::Task(task)) => {
                            let flow = app.run_task(task, &t_send);
                            if let Err(err) = t_send.send(flow) {
                                debug!("{:?}", err);
                                break 'l;
                            }
                        }
                        Ok(TaskArgs::Break) => {
                            //
                            break 'l;
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

    /// Send a task.
    pub fn send(&self, t: App::Task) -> ControlUI<App::Action, App::Error>
    where
        App::Error: From<SendError<()>>,
    {
        match self.send.send(TaskArgs::Task(t)) {
            Ok(_) => ControlUI::Continue,
            Err(_) => ControlUI::Err(SendError(()).into()),
        }
    }

    /// Receive a result.
    pub fn try_recv(&self) -> ControlUI<App::Action, App::Error>
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
    pub fn stop_and_join(mut self) -> Result<(), SendError<()>> {
        for _ in 0..self.handles.len() {
            if let Err(_) = self.send.send(TaskArgs::Break) {
                return Err(SendError(()));
            }
        }

        for h in self.handles.drain(..) {
            _ = h.join();
        }

        Ok(())
    }
}

impl<App: TuiApp + ?Sized> Drop for ThreadPool<App> {
    fn drop(&mut self) {
        for _ in 0..self.handles.len() {
            // drop is just a fallback to stop_and_join().
            // so dropping these results might be ok.
            _ = self.send.send(TaskArgs::Break);
        }
        for h in self.handles.drain(..) {
            _ = h.join();
        }
    }
}

impl<App: TuiApp> TaskSender<App>
where
    App::Action: 'static + Send,
    App::Error: 'static + Send,
{
    pub fn send(&self, msg: ControlUI<App::Action, App::Error>) -> Result<(), SendError<()>> {
        match self.send.send(msg) {
            Ok(v) => Ok(v),
            Err(_) => Err(SendError(())),
        }
    }
}
