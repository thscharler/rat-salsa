use crate::control_queue::ControlQueue;
use crate::poll::{PollRendered, PollTasks, PollTimers};
use crate::poll_queue::PollQueue;
use crate::run_config::RunConfig;
use crate::threadpool::ThreadPool;
use crate::timer::Timers;
use crate::tokio_tasks::PollTokio;
use crate::{AppContext, AppState, AppWidget, Control, RenderContext};
use crossbeam::channel::{SendError, TryRecvError};
use std::any::TypeId;
use std::cmp::min;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::time::Duration;
use std::{io, thread};

const SLEEP: u64 = 250_000; // µs
const BACKOFF: u64 = 10_000; // µs
const FAST_SLEEP: u64 = 100; // µs

fn _run_tui<App, Global, Event, Error>(
    app: App,
    global: &mut Global,
    state: &mut App::State,
    cfg: &mut RunConfig<App, Global, Event, Error>,
) -> Result<(), Error>
where
    App: AppWidget<Global, Event, Error>,
    App: 'static,
    Global: 'static,
    Event: Send + 'static,
    Error: Send + 'static + From<TryRecvError> + From<io::Error> + From<SendError<()>>,
{
    let term = cfg.term.as_mut();
    let poll = cfg.poll.as_mut_slice();

    let timers = if poll
        .iter()
        .any(|v| v.as_ref().type_id() == TypeId::of::<PollTimers>())
    {
        Some(Timers::default())
    } else {
        None
    };
    let tasks = if poll
        .iter()
        .any(|v| v.as_ref().type_id() == TypeId::of::<PollTasks>())
    {
        Some(ThreadPool::new(cfg.n_threats))
    } else {
        None
    };
    let rendered_event = poll.iter().enumerate().find_map(|(n, v)| {
        if v.as_ref().type_id() == TypeId::of::<PollRendered>() {
            Some(n)
        } else {
            None
        }
    });
    let tokio_spawn = poll.iter().find_map(|v| {
        if let Some(t) = v.as_any().downcast_ref::<PollTokio<Event, Error>>() {
            Some(t.get_spawn())
        } else {
            None
        }
    });
    let queue = ControlQueue::default();

    let mut appctx = AppContext {
        g: global,
        focus: None,
        count: 0,
        timers: &timers,
        tasks: &tasks,
        tokio: &tokio_spawn,
        queue: &queue,
    };

    let poll_queue = PollQueue::default();
    let mut poll_sleep = Duration::from_micros(SLEEP);

    // init state
    state.init(&mut appctx)?;

    // initial render
    appctx.count = term.render(&mut |frame| {
        let mut ctx = RenderContext {
            g: appctx.g,
            count: frame.count(),
            cursor: None,
        };
        let frame_area = frame.area();
        app.render(frame_area, frame.buffer_mut(), state, &mut ctx)?;
        if let Some((cursor_x, cursor_y)) = ctx.cursor {
            frame.set_cursor_position((cursor_x, cursor_y));
        }
        Ok(frame.count())
    })?;
    if let Some(h) = rendered_event {
        let r = poll[h].read_exec(state, &mut appctx);
        queue.push(r);
    }

    'ui: loop {
        // panic on worker panic
        if let Some(tasks) = &tasks {
            if !tasks.check_liveness() {
                dbg!("worker panicked");
                break 'ui;
            }
        }

        // No events queued, check here.
        if queue.is_empty() {
            // The events are not processed immediately, but all
            // notifies are queued in the poll_queue.
            if poll_queue.is_empty() {
                for (n, p) in poll.iter_mut().enumerate() {
                    match p.poll(&mut appctx) {
                        Ok(true) => {
                            poll_queue.push(n);
                        }
                        Ok(false) => {}
                        Err(e) => {
                            queue.push(Err(e));
                        }
                    }
                }
            }

            // Sleep regime.
            if poll_queue.is_empty() {
                let t = if let Some(timers) = &timers {
                    if let Some(timer_sleep) = timers.sleep_time() {
                        min(timer_sleep, poll_sleep)
                    } else {
                        poll_sleep
                    }
                } else {
                    poll_sleep
                };
                thread::sleep(t);
                if poll_sleep < Duration::from_micros(SLEEP) {
                    // Back off slowly.
                    poll_sleep += Duration::from_micros(BACKOFF);
                }
            } else {
                // Shorter sleep immediately after an event.
                poll_sleep = Duration::from_micros(FAST_SLEEP);
            }
        }

        // All the fall-out of the last event has cleared.
        // Run the next event.
        if queue.is_empty() {
            if let Some(h) = poll_queue.take() {
                let r = poll[h].read_exec(state, &mut appctx);
                queue.push(r);
            }
        }

        // Result of event-handling.
        if let Some(ctrl) = queue.take() {
            match ctrl {
                Err(e) => {
                    let r = state.error(e, &mut appctx);
                    queue.push(r);
                }
                Ok(Control::Continue) => {}
                Ok(Control::Unchanged) => {}
                Ok(Control::Changed) => {
                    let r = term.render(&mut |frame| {
                        let mut ctx = RenderContext {
                            g: appctx.g,
                            count: frame.count(),
                            cursor: None,
                        };
                        let frame_area = frame.area();
                        app.render(frame_area, frame.buffer_mut(), state, &mut ctx)?;
                        if let Some((cursor_x, cursor_y)) = ctx.cursor {
                            frame.set_cursor_position((cursor_x, cursor_y));
                        }
                        Ok(frame.count())
                    });
                    match r {
                        Ok(v) => {
                            appctx.count = v;
                            if let Some(h) = rendered_event {
                                let r = poll[h].read_exec(state, &mut appctx);
                                queue.push(r);
                            }
                        }
                        Err(e) => queue.push(Err(e)),
                    }
                }
                Ok(Control::Message(a)) => {
                    let r = state.event(&a, &mut appctx);
                    queue.push(r);
                }
                Ok(Control::Quit) => {
                    break 'ui;
                }
            }
        }
    }

    state.shutdown(&mut appctx)?;

    Ok(())
}

/// Run the event-loop
///
/// The shortest version I can come up with:
/// ```rust no_run
/// use rat_salsa::{run_tui, AppContext, AppState, AppWidget, Control, RenderContext, RunConfig};
/// use ratatui::buffer::Buffer;
/// use ratatui::layout::Rect;
/// use ratatui::style::Stylize;
/// use ratatui::text::Span;
/// use ratatui::widgets::Widget;
/// use rat_widget::event::{try_flow, ct_event};
///
/// #[derive(Debug)]
/// struct MainApp;
///
/// #[derive(Debug)]
/// struct MainState;
///
/// #[derive(Debug)]
/// enum Event {
///     Event(crossterm::event::Event)
/// }
///
/// impl From<crossterm::event::Event> for Event {
///     fn from(value: crossterm::event::Event) -> Self {
///         Self::Event(value)
///     }
/// }
///
/// impl AppWidget<(), Event, anyhow::Error> for MainApp {
///     type State = MainState;
///
///     fn render(
///         &self,
///         area: Rect,
///         buf: &mut Buffer,
///         _state: &mut Self::State,
///         _ctx: &mut RenderContext<'_, ()>,
///     ) -> Result<(), anyhow::Error> {
///         Span::from("Hello world")
///             .white()
///             .on_blue()
///             .render(area, buf);
///         Ok(())
///     }
/// }
///
/// impl AppState<(), Event, anyhow::Error> for MainState {
///     fn event(
///         &mut self,
///         event: &Event,
///         _ctx: &mut AppContext<'_, (), (), anyhow::Error>,
///     ) -> Result<Control<()>, anyhow::Error> {
///         let Event::Event(event) = event else {
///             return Ok(Control::Continue);
///         };
///
///         try_flow!(match event {
///             ct_event!(key press 'q') => Control::Quit,
///             _ => Control::Continue,
///         });
///
///         Ok(Control::Continue)
///     }
/// }
///
/// fn main() -> Result<(), anyhow::Error> {
///     use rat_salsa::poll::PollCrossterm;
///     run_tui(MainApp,
///         &mut (),
///         &mut MainState,
///         RunConfig::default()?
///             .poll(PollCrossterm)
///     )?;
///     Ok(())
/// }
///
/// ```
///
/// Maybe `examples/minimal.rs` is more useful.
///
pub fn run_tui<Widget, Global, Event, Error>(
    app: Widget,
    global: &mut Global,
    state: &mut Widget::State,
    mut cfg: RunConfig<Widget, Global, Event, Error>,
) -> Result<(), Error>
where
    Widget: AppWidget<Global, Event, Error> + 'static,
    Global: 'static,
    Event: Send + 'static,
    Error: Send + 'static + From<TryRecvError> + From<io::Error> + From<SendError<()>>,
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
