use crate::framework::control_queue::ControlQueue;
#[cfg(feature = "async")]
use crate::poll::PollTokio;
use crate::poll::{PollRendered, PollTasks, PollTimers};
use crate::run_config::RunConfig;
use crate::{AppContext, Control, RenderContext};
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
        ctx: &mut AppContext<'_, Global, Event, Error>,
    ) -> Result<(), Error>,
    render: fn(
        area: Rect, //
        buf: &mut Buffer,
        state: &mut State,
        ctx: &mut RenderContext<'_, Global>,
    ) -> Result<(), Error>,
    event: fn(
        event: &Event, //
        state: &mut State,
        ctx: &mut AppContext<'_, Global, Event, Error>,
    ) -> Result<Control<Event>, Error>,
    error: fn(
        error: Error, //
        state: &mut State,
        ctx: &mut AppContext<'_, Global, Event, Error>,
    ) -> Result<Control<Event>, Error>,
    global: &mut Global,
    state: &mut State,
    cfg: &mut RunConfig<Event, Error>,
) -> Result<(), Error>
where
    Global: 'static,
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
    let queue = ControlQueue::default();

    let mut appctx = AppContext {
        g: global,
        focus: None,
        count: 0,
        timers,
        tasks,
        #[cfg(feature = "async")]
        tokio,
        queue: &queue,
    };

    let poll_queue = PollQueue::default();
    let mut poll_sleep = Duration::from_micros(SLEEP);

    // init state
    init(state, &mut appctx)?;

    // initial render
    appctx.count = term.render(&mut |frame| {
        let mut ctx = RenderContext {
            g: appctx.g,
            count: frame.count(),
            cursor: None,
        };
        let frame_area = frame.area();
        render(frame_area, frame.buffer_mut(), state, &mut ctx)?;
        if let Some((cursor_x, cursor_y)) = ctx.cursor {
            frame.set_cursor_position((cursor_x, cursor_y));
        }
        Ok(frame.count())
    })?;
    if let Some(idx) = rendered_event {
        queue.push(poll[idx].read());
    }

    'ui: loop {
        // panic on worker panic
        if let Some(tasks) = &appctx.tasks {
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
                    match p.poll() {
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
                let t = if let Some(timers) = &appctx.timers {
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
                queue.push(poll[h].read());
            }
        }

        // Result of event-handling.
        if let Some(ctrl) = queue.take() {
            match ctrl {
                Err(e) => {
                    queue.push(error(e, state, &mut appctx));
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
                        render(frame_area, frame.buffer_mut(), state, &mut ctx)?;
                        if let Some((cursor_x, cursor_y)) = ctx.cursor {
                            frame.set_cursor_position((cursor_x, cursor_y));
                        }
                        Ok(frame.count())
                    });
                    match r {
                        Ok(v) => {
                            appctx.count = v;
                            if let Some(h) = rendered_event {
                                queue.push(poll[h].read());
                            }
                        }
                        Err(e) => queue.push(Err(e)),
                    }
                }
                Ok(Control::Event(a)) => {
                    queue.push(event(&a, state, &mut appctx));
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
/// use rat_salsa2::{run_tui, Control, RunConfig};
/// use ratatui::buffer::Buffer;
/// use ratatui::layout::Rect;
/// use ratatui::style::Stylize;
/// use ratatui::text::Span;
/// use ratatui::widgets::Widget;
/// use rat_widget::event::{try_flow, ct_event};
///
/// type AppContext<'a> = rat_salsa2::AppContext<'a, (), Event, anyhow::Error>;
/// type RenderContext<'a> = rat_salsa2::RenderContext<'a, ()>;
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
/// fn main() -> Result<(), anyhow::Error> {
///     use rat_salsa2::poll::PollCrossterm;
///     run_tui(
///         init,
///         render,
///         event,
///         error,
///         &mut (),
///         &mut State,
///         RunConfig::default()?
///             .poll(PollCrossterm)
///     )?;
///     Ok(())
/// }
///
/// fn init(state: &mut State,
///     ctx: &mut AppContext<'_>) -> Result<(), anyhow::Error> {
///     Ok(())
/// }
///
/// fn render(
///     area: Rect,
///     buf: &mut Buffer,
///     state: &mut State,
///     ctx: &mut RenderContext<'_>,
/// ) -> Result<(), anyhow::Error> {
///     Span::from("Hello world").white().on_blue().render(area, buf);
///     Ok(())
/// }
///
/// fn event(
///     event: &Event,
///     state: &mut State,
///     ctx: &mut AppContext<'_>,
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
///     ctx: &mut AppContext<'_>) -> Result<Control<Event>, anyhow::Error> {
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
        ctx: &mut AppContext<'_, Global, Event, Error>,
    ) -> Result<(), Error>,
    render: fn(
        area: Rect, //
        buf: &mut Buffer,
        state: &mut State,
        ctx: &mut RenderContext<'_, Global>,
    ) -> Result<(), Error>,
    event: fn(
        event: &Event, //
        state: &mut State,
        ctx: &mut AppContext<'_, Global, Event, Error>,
    ) -> Result<Control<Event>, Error>,
    error: fn(
        error: Error, //
        state: &mut State,
        ctx: &mut AppContext<'_, Global, Event, Error>,
    ) -> Result<Control<Event>, Error>,
    global: &mut Global,
    state: &mut State,
    mut cfg: RunConfig<Event, Error>,
) -> Result<(), Error>
where
    Global: 'static,
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
