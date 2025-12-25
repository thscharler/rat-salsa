use crate::framework::control_queue::ControlQueue;
#[cfg(feature = "async")]
use crate::poll::PollTokio;
use crate::poll::{PollQuit, PollRendered, PollTasks, PollTimers};
use crate::run_config::{RunConfig, TermInit};
use crate::{Control, SalsaAppContext, SalsaContext};
use crossterm::ExecutableCommand;
use crossterm::cursor::{DisableBlinking, EnableBlinking, SetCursorStyle};
use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, SetTitle, disable_raw_mode, enable_raw_mode,
};
use poll_queue::PollQueue;
use rat_event::util::set_have_keyboard_enhancement;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::any::TypeId;
use std::cmp::min;
use std::io::stdout;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::time::{Duration, SystemTime};
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
    cfg: RunConfig<Event, Error>,
) -> Result<(), Error>
where
    Global: SalsaContext<Event, Error>,
    Event: 'static,
    Error: 'static + From<io::Error>,
{
    let term = cfg.term;
    let mut poll = cfg.poll;

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
    let rendered_event = poll
        .iter()
        .position(|v| v.as_ref().type_id() == TypeId::of::<PollRendered>());
    let quit = poll
        .iter()
        .position(|v| v.as_ref().type_id() == TypeId::of::<PollQuit>());
    #[cfg(feature = "async")]
    let tokio = poll.iter().find_map(|v| {
        v.as_any()
            .downcast_ref::<PollTokio<Event, Error>>()
            .map(|t| t.get_tasks())
    });

    global.set_salsa_ctx(SalsaAppContext {
        focus: Default::default(),
        count: Default::default(),
        cursor: Default::default(),
        term: Some(term.clone()),
        window_title: Default::default(),
        clear_terminal: Default::default(),
        insert_before: Default::default(),
        last_render: Default::default(),
        last_event: Default::default(),
        timers,
        tasks,
        #[cfg(feature = "async")]
        tokio,
        queue: ControlQueue::default(),
    });

    let poll_queue = PollQueue::default();
    let mut poll_sleep = Duration::from_micros(SLEEP);
    let mut was_changed = false;

    // init state
    init(state, global)?;

    // initial render
    {
        let ib = global.salsa_ctx().insert_before.take();
        if ib.height > 0 {
            term.borrow_mut().insert_before(ib.height, ib.draw_fn)?;
        }
        if let Some(title) = global.salsa_ctx().window_title.replace(None) {
            stdout().execute(SetTitle(title))?;
        }
        let mut r = Ok(());
        term.borrow_mut().draw(&mut |frame: &mut Frame| -> () {
            let frame_area = frame.area();
            let ttt = SystemTime::now();

            r = render(frame_area, frame.buffer_mut(), state, global);

            global
                .salsa_ctx()
                .last_render
                .set(ttt.elapsed().unwrap_or_default());
            if let Some((cursor_x, cursor_y)) = global.salsa_ctx().cursor.get() {
                frame.set_cursor_position((cursor_x, cursor_y));
            }
            global.salsa_ctx().count.set(frame.count());
            global.salsa_ctx().cursor.set(None);
        })?;
        r?;
        if let Some(idx) = rendered_event {
            global.salsa_ctx().queue.push(poll[idx].read());
        }
    }

    'ui: loop {
        // panic on worker panic
        if let Some(tasks) = &global.salsa_ctx().tasks {
            if !tasks.check_liveness() {
                dbg!("worker panicked");
                break 'ui;
            }
        }

        // No events queued, check here.
        if global.salsa_ctx().queue.is_empty() {
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
                            global.salsa_ctx().queue.push(Err(e));
                        }
                    }
                }
            }

            // Sleep regime.
            if poll_queue.is_empty() {
                let mut t = poll_sleep;
                for p in poll.iter_mut() {
                    if let Some(timer_sleep) = p.sleep_time() {
                        t = min(timer_sleep, t);
                    }
                }
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
        if global.salsa_ctx().queue.is_empty() {
            if let Some(h) = poll_queue.take() {
                global.salsa_ctx().queue.push(poll[h].read());
            }
        }

        // Result of event-handling.
        if let Some(ctrl) = global.salsa_ctx().queue.take() {
            // filter out double Changed events.
            // no need to render twice in a row.
            if matches!(ctrl, Ok(Control::Changed)) {
                if was_changed {
                    continue;
                }
                was_changed = true;
            } else {
                was_changed = false;
            }

            match ctrl {
                Err(e) => {
                    let r = error(e, state, global);
                    global.salsa_ctx().queue.push(r);
                }
                Ok(Control::Continue) => {}
                Ok(Control::Unchanged) => {}
                Ok(Control::Changed) => {
                    if global.salsa_ctx().clear_terminal.get() {
                        global.salsa_ctx().clear_terminal.set(false);
                        if let Err(e) = term.borrow_mut().clear() {
                            global.salsa_ctx().queue.push(Err(e.into()));
                        }
                    }
                    let ib = global.salsa_ctx().insert_before.take();
                    if ib.height > 0 {
                        term.borrow_mut().insert_before(ib.height, ib.draw_fn)?;
                    }
                    if let Some(title) = global.salsa_ctx().window_title.replace(None) {
                        stdout().execute(SetTitle(title))?;
                    }
                    let mut r = Ok(());
                    term.borrow_mut().draw(&mut |frame: &mut Frame| -> () {
                        let frame_area = frame.area();
                        let ttt = SystemTime::now();

                        r = render(frame_area, frame.buffer_mut(), state, global);

                        global
                            .salsa_ctx()
                            .last_render
                            .set(ttt.elapsed().unwrap_or_default());
                        if let Some((cursor_x, cursor_y)) = global.salsa_ctx().cursor.get() {
                            frame.set_cursor_position((cursor_x, cursor_y));
                        }
                        global.salsa_ctx().count.set(frame.count());
                        global.salsa_ctx().cursor.set(None);
                    })?;
                    match r {
                        Ok(_) => {
                            if let Some(h) = rendered_event {
                                global.salsa_ctx().queue.push(poll[h].read());
                            }
                        }
                        Err(e) => global.salsa_ctx().queue.push(Err(e)),
                    }
                }
                #[cfg(feature = "dialog")]
                Ok(Control::Close(a)) => {
                    // close probably demands a repaint.
                    global.salsa_ctx().queue.push(Ok(Control::Event(a)));
                    global.salsa_ctx().queue.push(Ok(Control::Changed));
                }
                Ok(Control::Event(a)) => {
                    let ttt = SystemTime::now();
                    let r = event(&a, state, global);
                    global
                        .salsa_ctx()
                        .last_event
                        .set(ttt.elapsed().unwrap_or_default());
                    global.salsa_ctx().queue.push(r);
                }
                Ok(Control::Quit) => {
                    if let Some(quit) = quit {
                        match poll[quit].read() {
                            Ok(Control::Event(a)) => {
                                match event(&a, state, global) {
                                    Ok(Control::Quit) => { /* really quit now */ }
                                    v => {
                                        global.salsa_ctx().queue.push(v);
                                        continue;
                                    }
                                }
                            }
                            Err(_) => unreachable!(),
                            Ok(_) => unreachable!(),
                        }
                    }
                    break 'ui;
                }
            }
        }
    }

    if cfg.term_init.clear_area {
        term.borrow_mut().clear()?;
    }

    Ok(())
}

/// Run the event-loop
///
/// The shortest version I can come up with:
/// ```rust no_run
/// use anyhow::{anyhow, Error};
/// use rat_salsa::poll::PollCrossterm;
/// use rat_salsa::{mock, run_tui, Control, RunConfig, SalsaAppContext, SalsaContext};
/// use rat_widget::event::ct_event;
/// use ratatui::buffer::Buffer;
/// use ratatui::layout::Rect;
/// use ratatui::style::Stylize;
/// use ratatui::text::{Line, Span};
/// use ratatui::widgets::Widget;
///
/// fn main() -> Result<(), Error> {
///     run_tui(
///         mock::init,
///         render,
///         event,
///         error,
///         &mut Global::default(),
///         &mut Ultra,
///         RunConfig::default()?.poll(PollCrossterm),
///     )
/// }
///
/// #[derive(Debug, Default)]
/// pub struct Global {
///     ctx: SalsaAppContext<UltraEvent, Error>,
///     pub err_cnt: u32,
///     pub err_msg: String,
/// }
///
/// impl SalsaContext<UltraEvent, Error> for Global {
///     fn set_salsa_ctx(&mut self, app_ctx: SalsaAppContext<UltraEvent, Error>) {
///         self.ctx = app_ctx;
///     }
///
///     fn salsa_ctx(&self) -> &SalsaAppContext<UltraEvent, Error> {
///         &self.ctx
///     }
/// }
///
/// #[derive(Debug, PartialEq, Eq, Clone)]
/// pub enum UltraEvent {
///     Event(crossterm::event::Event),
/// }
///
/// impl From<crossterm::event::Event> for UltraEvent {
///     fn from(value: crossterm::event::Event) -> Self {
///         Self::Event(value)
///     }
/// }
///
/// pub struct Ultra;
///
/// fn render(area: Rect, buf: &mut Buffer, _state: &mut Ultra, ctx: &mut Global) -> Result<(), Error> {
///     Line::from_iter([Span::from("'q' to quit, 'e' for error, 'r' for repair")])
///         .render(Rect::new(area.x, area.y, area.width, 1), buf);
///     Line::from_iter([
///         Span::from("Hello world!").green(),
///         Span::from(" Status: "),
///         if ctx.err_cnt > 0 {
///             Span::from(&ctx.err_msg).red().underlined()
///         } else {
///             Span::from(&ctx.err_msg).cyan().underlined()
///         },
///     ])
///     .render(Rect::new(area.x, area.y + 2, area.width, 1), buf);
///     Ok(())
/// }
///
/// fn event(
///     event: &UltraEvent,
///     _state: &mut Ultra,
///     ctx: &mut Global,
/// ) -> Result<Control<UltraEvent>, Error> {
///     match event {
///         UltraEvent::Event(event) => match event {
///             ct_event!(key press 'q') => Ok(Control::Quit),
///             ct_event!(key press 'e') => return Err(anyhow!("An error occured.")),
///             ct_event!(key press 'r') => {
///                 if ctx.err_cnt > 1 {
///                     ctx.err_cnt -= 1;
///                     ctx.err_msg = format!("#{}# One error repaired.", ctx.err_cnt).to_string();
///                 } else if ctx.err_cnt == 1 {
///                     ctx.err_cnt -= 1;
///                     ctx.err_msg = "All within norms.".to_string();
///                 } else {
///                     ctx.err_cnt = 1;
///                     ctx.err_msg = format!("#{}# Over-repaired.", ctx.err_cnt).to_string();
///                 }
///                 Ok(Control::Changed)
///             }
///             _ => Ok(Control::Continue),
///         },
///     }
/// }
///
/// fn error(event: Error, _state: &mut Ultra, ctx: &mut Global) -> Result<Control<UltraEvent>, Error> {
///     ctx.err_cnt += 1;
///     ctx.err_msg = format!("#{}# {}", ctx.err_cnt, event).to_string();
///     Ok(Control::Changed)
/// }
/// ```
///
/// Maybe `templates/minimal.rs` is more useful.
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
    cfg: RunConfig<Event, Error>,
) -> Result<(), Error>
where
    Global: SalsaContext<Event, Error>,
    Event: 'static,
    Error: 'static + From<io::Error>,
{
    let t = cfg.term_init;

    if !t.manual {
        init_terminal(t)?;
    }

    let r = match catch_unwind(AssertUnwindSafe(|| {
        _run_tui(init, render, event, error, global, state, cfg)
    })) {
        Ok(v) => v,
        Err(e) => {
            if !t.manual {
                _ = shutdown_terminal(t);
            }
            resume_unwind(e);
        }
    };

    if !t.manual {
        shutdown_terminal(t)?;
    }

    r
}

fn init_terminal(cfg: TermInit) -> io::Result<()> {
    if cfg.alternate_screen {
        stdout().execute(EnterAlternateScreen)?;
    }
    if cfg.mouse_capture {
        stdout().execute(EnableMouseCapture)?;
    }
    if cfg.bracketed_paste {
        stdout().execute(EnableBracketedPaste)?;
    }
    if cfg.cursor_blinking {
        stdout().execute(EnableBlinking)?;
    }
    stdout().execute(cfg.cursor)?;
    #[cfg(not(windows))]
    {
        stdout().execute(PushKeyboardEnhancementFlags(cfg.keyboard_enhancements))?;
        let enhanced = supports_keyboard_enhancement().unwrap_or_default();
        set_have_keyboard_enhancement(enhanced);
    }
    #[cfg(windows)]
    {
        set_have_keyboard_enhancement(true);
    }

    enable_raw_mode()?;

    Ok(())
}

fn shutdown_terminal(cfg: TermInit) -> io::Result<()> {
    disable_raw_mode()?;

    #[cfg(not(windows))]
    stdout().execute(PopKeyboardEnhancementFlags)?;
    stdout().execute(SetCursorStyle::DefaultUserShape)?;
    if cfg.cursor_blinking {
        stdout().execute(DisableBlinking)?;
    }
    if cfg.bracketed_paste {
        stdout().execute(DisableBracketedPaste)?;
    }
    if cfg.mouse_capture {
        stdout().execute(DisableMouseCapture)?;
    }
    if cfg.alternate_screen {
        stdout().execute(LeaveAlternateScreen)?;
    }

    Ok(())
}
