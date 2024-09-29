use crate::control_queue::ControlQueue;
use crate::poll_queue::PollQueue;
use crate::run_config::RunConfig;
use crate::threadpool::ThreadPool;
use crate::timer::Timers;
use crate::{AppContext, AppState, AppWidget, Control, RenderContext};
use crossbeam::channel::{SendError, TryRecvError};
use std::cmp::min;
use std::fmt::Debug;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::time::Duration;
use std::{io, thread};

fn _run_tui<App, Global, Message, Error>(
    app: App,
    global: &mut Global,
    state: &mut App::State,
    cfg: &mut RunConfig<App, Global, Message, Error>,
) -> Result<(), Error>
where
    App: AppWidget<Global, Message, Error>,
    Message: Send + 'static + Debug,
    Error: Send + 'static + Debug + From<TryRecvError> + From<io::Error> + From<SendError<()>>,
{
    let term = cfg.term.as_mut();
    let poll = cfg.poll.as_mut_slice();

    let timers = Timers::default();
    let tasks = ThreadPool::new(cfg.n_threats);
    let queue = ControlQueue::default();

    let mut appctx = AppContext {
        g: global,
        focus: None,
        timers: &timers,
        tasks: &tasks,
        queue: &queue,
    };

    let poll_queue = PollQueue::default();
    let mut poll_sleep = Duration::from_millis(10);

    // init state
    state.init(&mut appctx)?;

    // initial render
    term.render(
        &mut |frame, timeout| {
            let mut ctx = RenderContext {
                g: appctx.g,
                timeout,
                timers: appctx.timers,
                count: frame.count(),
                cursor: None,
            };
            let frame_area = frame.area();
            app.render(frame_area, frame.buffer_mut(), state, &mut ctx)?;
            if let Some((cursor_x, cursor_y)) = ctx.cursor {
                frame.set_cursor_position((cursor_x, cursor_y));
            }
            Ok(())
        },
        None,
    )?;

    'ui: loop {
        // panic on worker panic
        if !tasks.check_liveness() {
            dbg!("worker panicked");
            break 'ui;
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
                            appctx.queue.push(Err(e), None);
                        }
                    }
                }
            }

            // Sleep regime.
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

        // All the fall-out of the last event has cleared.
        // Run the next event.
        if queue.is_empty() {
            if let Some(h) = poll_queue.take() {
                let r = poll[h].read_exec(state, &mut appctx);
                let t = poll[h].timeout();
                appctx.queue.push(r, t);
            }
        }

        // Result of event-handling.
        if let Some(n) = queue.take() {
            match n.ctrl {
                Err(e) => {
                    let r = state.error(e, &mut appctx);
                    appctx.queue.push(r, None);
                }
                Ok(Control::Continue) => {}
                Ok(Control::Unchanged) => {}
                Ok(Control::Changed) => {
                    if let Err(e) = term.render(
                        &mut |frame, timeout| {
                            let mut ctx = RenderContext {
                                g: appctx.g,
                                timeout,
                                timers: appctx.timers,
                                count: frame.count(),
                                cursor: None,
                            };
                            let frame_area = frame.area();
                            app.render(frame_area, frame.buffer_mut(), state, &mut ctx)?;
                            if let Some((cursor_x, cursor_y)) = ctx.cursor {
                                frame.set_cursor_position((cursor_x, cursor_y));
                            }
                            Ok(())
                        },
                        n.timeout,
                    ) {
                        appctx.queue.push(Err(e), None);
                    }
                }
                Ok(Control::Message(mut a)) => {
                    let r = state.message(&mut a, &mut appctx);
                    appctx.queue.push(r, None);
                }
                Ok(Control::Quit) => {
                    break 'ui;
                }
            }
        }

        // reset some appctx data
        appctx.focus = None;
    }

    Ok(())
}

/// Run the event-loop
///
/// The shortest version I can come up with:
/// ```rust no_run
/// use crossterm::event::Event;
/// use rat_salsa::event::{ct_event, try_flow};
/// use rat_salsa::{run_tui, AppContext, AppState, AppWidget, Control, RenderContext, RunConfig};
/// use ratatui::buffer::Buffer;
/// use ratatui::layout::Rect;
/// use ratatui::style::Stylize;
/// use ratatui::text::Span;
/// use ratatui::widgets::Widget;
///
/// #[derive(Debug)]
/// struct MainApp;
///
/// #[derive(Debug)]
/// struct MainState;
///
/// impl AppWidget<(), (), anyhow::Error> for MainApp {
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
/// impl AppState<(), (), anyhow::Error> for MainState {
///     fn crossterm(
///         &mut self,
///         event: &Event,
///         _ctx: &mut AppContext<'_, (), (), anyhow::Error>,
///     ) -> Result<Control<()>, anyhow::Error> {
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
///     run_tui(MainApp, &mut (), &mut MainState, RunConfig::default()?)?;
///     Ok(())
/// }
///
/// ```
///
/// Maybe `examples/minimal.rs` is more useful.
///
pub fn run_tui<Widget, Global, Message, Error>(
    app: Widget,
    global: &mut Global,
    state: &mut Widget::State,
    mut cfg: RunConfig<Widget, Global, Message, Error>,
) -> Result<(), Error>
where
    Widget: AppWidget<Global, Message, Error>,
    Message: Send + 'static + Debug,
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
