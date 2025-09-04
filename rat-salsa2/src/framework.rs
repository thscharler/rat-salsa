use crate::framework::control_queue::ControlQueue;
#[cfg(feature = "async")]
use crate::poll::PollTokio;
use crate::poll::{PollRendered, PollTasks, PollTimers};
use crate::run_config::RunConfig;
use crate::{AppContext, Context, Control};
use poll_queue::PollQueue;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::any::TypeId;
use std::cmp::min;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::time::Duration;
use std::{io, thread};

pub(crate) mod control_queue;
mod poll_queue;

const SLEEP: u64 = 250_000; // µs
const BACKOFF: u64 = 10_000; // µs
const FAST_SLEEP: u64 = 100; // µs

fn _run_tui<Global, State, Event, Error>(
    init: fn(
        state: &mut State, //
        ctx: &mut Global,
    ) -> Result<(), Error>,
    render: fn(
        area: Rect, //
        buf: &mut Buffer,
        state: &mut State,
        ctx: &mut Global,
    ) -> Result<(), Error>,
    event: fn(
        event: &Event, //
        state: &mut State,
        ctx: &mut Global,
    ) -> Result<Control<Event>, Error>,
    error: fn(
        error: Error, //
        state: &mut State,
        ctx: &mut Global,
    ) -> Result<Control<Event>, Error>,
    global: &mut Global,
    state: &mut State,
    cfg: &mut RunConfig<Event, Error>,
) -> Result<(), Error>
where
    Global: Context<Event, Error>,
    Event: Send + 'static,
    Error: Send + 'static + From<io::Error>,
{
    let term = cfg.term.as_mut();
    let poll = cfg.poll.as_mut_slice();

    let timers = poll.iter().find_map(|v| {
        v.as_any()
            .downcast_ref::<PollTimers>()
            .map(|t| t.get_timers())
    });
    let tasks = poll.iter().find_map(|v| {
        v.as_any()
            .downcast_ref::<PollTasks<Event, Error>>()
            .map(|t| t.get_tasks())
    });
    let rendered_event = poll.iter().enumerate().find_map(|(n, v)| {
        if v.as_ref().type_id() == TypeId::of::<PollRendered>() {
            Some(n)
        } else {
            None
        }
    });
    #[cfg(feature = "async")]
    let tokio = poll.iter().find_map(|v| {
        v.as_any()
            .downcast_ref::<PollTokio<Event, Error>>()
            .map(|t| t.get_tasks())
    });

    global.set_app_ctx(AppContext {
        focus: Default::default(),
        count: Default::default(),
        cursor: Default::default(),
        timers,
        tasks,
        #[cfg(feature = "async")]
        tokio,
        queue: ControlQueue::default(),
    });

    let poll_queue = PollQueue::default();
    let mut poll_sleep = Duration::from_micros(SLEEP);

    // init state
    init(state, global)?;

    // initial render
    term.render(&mut |frame| {
        let frame_area = frame.area();
        render(frame_area, frame.buffer_mut(), state, global)?;
        if let Some((cursor_x, cursor_y)) = global.app_ctx().cursor.get() {
            frame.set_cursor_position((cursor_x, cursor_y));
        }
        global.app_ctx().count.set(frame.count());
        global.app_ctx().cursor.set(None);
        Ok(())
    })?;
    if let Some(idx) = rendered_event {
        global.app_ctx().queue.push(poll[idx].read());
    }

    'ui: loop {
        // panic on worker panic
        if let Some(tasks) = &global.app_ctx().tasks {
            if !tasks.check_liveness() {
                dbg!("worker panicked");
                break 'ui;
            }
        }

        // No events queued, check here.
        if global.app_ctx().queue.is_empty() {
            // The events are not processed immediately, but all
            // notifies are queued in the poll_queue.
            if poll_queue.is_empty() {
                for (n, p) in poll.iter_mut().enumerate() {
                    match p.poll() {
                        Ok(true) => {
                            poll_queue.push(n);
                        }
                        Ok(false) => {}
                        Err(e) => {
                            global.app_ctx().queue.push(Err(e));
                        }
                    }
                }
            }

            // Sleep regime.
            if poll_queue.is_empty() {
                let t = if let Some(timers) = &global.app_ctx().timers {
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
        if global.app_ctx().queue.is_empty() {
            if let Some(h) = poll_queue.take() {
                global.app_ctx().queue.push(poll[h].read());
            }
        }

        // Result of event-handling.
        if let Some(ctrl) = global.app_ctx().queue.take() {
            match ctrl {
                Err(e) => {
                    let r = error(e, state, global);
                    global.app_ctx().queue.push(r);
                }
                Ok(Control::Continue) => {}
                Ok(Control::Unchanged) => {}
                Ok(Control::Changed) => {
                    let r = term.render(&mut |frame| {
                        let frame_area = frame.area();
                        render(frame_area, frame.buffer_mut(), state, global)?;
                        if let Some((cursor_x, cursor_y)) = global.app_ctx().cursor.get() {
                            frame.set_cursor_position((cursor_x, cursor_y));
                        }
                        global.app_ctx().count.set(frame.count());
                        global.app_ctx().cursor.set(None);
                        Ok(())
                    });
                    match r {
                        Ok(_) => {
                            if let Some(h) = rendered_event {
                                global.app_ctx().queue.push(poll[h].read());
                            }
                        }
                        Err(e) => global.app_ctx().queue.push(Err(e)),
                    }
                }
                Ok(Control::Event(a)) => {
                    let r = event(&a, state, global);
                    global.app_ctx().queue.push(r);
                }
                Ok(Control::Quit) => {
                    break 'ui;
                }
            }
        }
    }

    // state.shutdown(&mut appctx)?;

    Ok(())
}

/// Run the event-loop
///
/// The shortest version I can come up with:
/// ```rust no_run
/// use rat_salsa2::{run_tui, Context, AppContext, Control, RunConfig};
/// use ratatui::buffer::Buffer;
/// use ratatui::layout::Rect;
/// use ratatui::style::Stylize;
/// use ratatui::text::Span;
/// use ratatui::widgets::Widget;
/// use rat_widget::event::{try_flow, ct_event};
///
/// #[derive(Debug)]
/// struct State;
///
/// #[derive(Debug)]
/// enum Event {
///     Event(crossterm::event::Event),
///     Dummy
/// }
///
/// impl From<crossterm::event::Event> for Event {
///     fn from(value: crossterm::event::Event) -> Self {
///         Self::Event(value)
///     }
/// }
///
/// struct Global {
///     app_ctx: Option<AppContext<Event, anyhow::Error>>
/// }
///
/// impl Context<Event, anyhow::Error> for Global {
///     fn set_app_ctx(&mut self, app_ctx: AppContext<Event, anyhow::Error>) {
///         self.app_ctx = Some(app_ctx);
///     }
///
///     #[inline]
///     fn app_ctx(&self) -> &AppContext<Event, anyhow::Error> {
///         self.app_ctx.as_ref().expect("app-ctx")
///     }
/// }
///
/// fn main() -> Result<(), anyhow::Error> {
///     use rat_salsa2::poll::PollCrossterm;
///     run_tui(
///         init,
///         render,
///         event,
///         error,
///         &mut Global {app_ctx: None},
///         &mut State,
///         RunConfig::default()?
///             .poll(PollCrossterm)
///     )?;
///     Ok(())
/// }
///
/// fn init(state: &mut State,
///     ctx: &mut Global) -> Result<(), anyhow::Error> {
///     Ok(())
/// }
///
/// fn render(
///     area: Rect,
///     buf: &mut Buffer,
///     state: &mut State,
///     ctx: &mut Global,
/// ) -> Result<(), anyhow::Error> {
///     Span::from("Hello world").white().on_blue().render(area, buf);
///     Ok(())
/// }
///
/// fn event(
///     event: &Event,
///     state: &mut State,
///     ctx: &mut Global,
/// ) -> Result<Control<Event>, anyhow::Error> {
///     if let Event::Event(event) = event {
///         try_flow!(match event {
///             crossterm::event::Event::Resize(_,_) => Control::Changed,
///             ct_event!(key press 'q') => Control::Quit,
///             _ => Control::Continue,
///         });
///
///         Ok(Control::Continue)
///     } else {
///         Ok(Control::Continue)
///     }
/// }
///
/// fn error(error: anyhow::Error,
///     state: &mut State,
///     ctx: &mut Global) -> Result<Control<Event>, anyhow::Error> {
///     Ok(Control::Continue)
/// }
///
/// ```
///
/// Maybe `examples/minimal.rs` is more useful.
///
pub fn run_tui<Global, State, Event, Error>(
    init: fn(
        state: &mut State, //
        ctx: &mut Global,
    ) -> Result<(), Error>,
    render: fn(
        area: Rect, //
        buf: &mut Buffer,
        state: &mut State,
        ctx: &mut Global,
    ) -> Result<(), Error>,
    event: fn(
        event: &Event, //
        state: &mut State,
        ctx: &mut Global,
    ) -> Result<Control<Event>, Error>,
    error: fn(
        error: Error, //
        state: &mut State,
        ctx: &mut Global,
    ) -> Result<Control<Event>, Error>,
    global: &mut Global,
    state: &mut State,
    mut cfg: RunConfig<Event, Error>,
) -> Result<(), Error>
where
    Global: Context<Event, Error>,
    Event: Send + 'static,
    Error: Send + 'static + From<io::Error>,
{
    cfg.term.init()?;

    let r = match catch_unwind(AssertUnwindSafe(|| {
        _run_tui(init, render, event, error, global, state, &mut cfg)
    })) {
        Ok(v) => v,
        Err(e) => {
            _ = cfg.term.shutdown();
            resume_unwind(e);
        }
    };

    cfg.term.shutdown()?;

    r
}
