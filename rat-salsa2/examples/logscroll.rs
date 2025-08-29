#![allow(dead_code)]

use crate::event::LogScrollEvent;
use crate::file_watcher::{FileWatcher, PollFileWatcher};
use crate::scenery::{error, event, init, render, SceneryState};
use anyhow::Error;
use rat_salsa2::poll::PollCrossterm;
use rat_salsa2::{run_tui, RunConfig};
use rat_theme2::palettes::IMPERIAL;
use rat_theme2::DarkTheme;
use std::env::args;
use std::fs;
use std::path::PathBuf;

type AppContext<'a> = rat_salsa2::AppContext<'a, GlobalState, LogScrollEvent, Error>;
type RenderContext<'a> = rat_salsa2::RenderContext<'a, GlobalState>;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let mut args = args();
    args.next();
    let Some(current) = args.next() else {
        eprintln!("usage: logscroll <file>");
        return Ok(());
    };
    let current = PathBuf::from(current);
    if !current.exists() {
        eprintln!("file {:?} does not exist.", current);
        return Ok(());
    }

    let config = LogScrollConfig::default();

    let theme = DarkTheme::new("Imperial".into(), IMPERIAL);

    let (filewatch_poll, filewatch) = PollFileWatcher::new()?;
    filewatch.watch(&current, notify::RecursiveMode::NonRecursive)?;

    let mut global = GlobalState::new(config, theme, filewatch);
    global.current = current;

    let mut state = SceneryState::default();

    run_tui(
        init,
        render,
        event,
        error,
        &mut global,
        &mut state,
        RunConfig::default()?
            .poll(PollCrossterm)
            .poll(filewatch_poll),
    )?;

    Ok(())
}

mod file_watcher {
    use anyhow::Error;
    use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
    use rat_salsa2::{Control, PollEvents};
    use std::any::Any;
    use std::cell::RefCell;
    use std::path::Path;
    use std::rc::Rc;
    use std::sync::mpsc;
    use std::sync::mpsc::TryRecvError;

    #[derive(Debug)]
    pub struct PollFileWatcher {
        channel: Rc<RefCell<mpsc::Receiver<Result<Event, notify::Error>>>>,
        watcher: Rc<RefCell<RecommendedWatcher>>,
        event: Option<Result<Event, notify::Error>>,
    }

    #[derive(Debug)]
    pub struct FileWatcher {
        watcher: Rc<RefCell<RecommendedWatcher>>,
    }

    impl PollFileWatcher {
        pub fn new() -> Result<(Self, FileWatcher), Error> {
            let (s, r) = mpsc::channel();
            let z = Self {
                channel: Rc::new(RefCell::new(r)),
                watcher: Rc::new(RefCell::new(notify::recommended_watcher(s)?)),
                event: None,
            };
            let c = FileWatcher {
                watcher: z.watcher.clone(),
            };

            Ok((z, c))
        }
    }

    impl FileWatcher {
        pub fn watch(&self, path: &Path, recursive: RecursiveMode) -> Result<(), Error> {
            self.watcher.borrow_mut().watch(path, recursive)?;
            Ok(())
        }

        pub fn unwatch(&self, path: &Path) -> Result<(), Error> {
            self.watcher.borrow_mut().unwatch(path)?;
            Ok(())
        }
    }

    impl<Event, Error> PollEvents<Event, Error> for PollFileWatcher
    where
        Event: 'static + Send + From<notify::Event>,
        Error: 'static + Send + From<mpsc::TryRecvError> + From<notify::Error>,
    {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn poll(&mut self) -> Result<bool, Error> {
            match self.channel.borrow_mut().try_recv() {
                Ok(v) => {
                    self.event = Some(v);
                    Ok(true)
                }
                Err(TryRecvError::Empty) => Ok(false),
                Err(TryRecvError::Disconnected) => Err(TryRecvError::Disconnected.into()),
            }
        }

        fn read(&mut self) -> Result<Control<Event>, Error> {
            if let Some(event) = self.event.take() {
                Ok(Control::Event(Event::from(event?)))
            } else {
                Ok(Control::Continue)
            }
        }
    }
}

mod event {
    #[derive(Debug)]
    pub enum LogScrollEvent {
        Event(crossterm::event::Event),
        File(notify::Event),
        Message(String),
        Status(usize, String),
    }

    impl From<crossterm::event::Event> for LogScrollEvent {
        fn from(value: crossterm::event::Event) -> Self {
            Self::Event(value)
        }
    }

    impl From<notify::Event> for LogScrollEvent {
        fn from(value: notify::Event) -> Self {
            Self::File(value)
        }
    }
}

mod scenery {
    use crate::event::LogScrollEvent;
    use crate::logscroll::LogScrollState;
    use crate::{logscroll, AppContext, RenderContext};
    use anyhow::Error;
    use log::debug;
    use rat_salsa2::Control;
    use rat_widget::event::{ct_event, ConsumedEvent, Dialog, HandleEvent, Regular};
    use rat_widget::focus::FocusBuilder;
    use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
    use rat_widget::statusline::{StatusLine, StatusLineState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Layout, Rect};
    use ratatui::widgets::StatefulWidget;
    use std::time::{Duration, SystemTime};

    #[derive(Debug, Default)]
    pub struct SceneryState {
        pub logscroll: LogScrollState,
        pub status: StatusLineState,
        pub error_dlg: MsgDialogState,
    }

    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        state: &mut SceneryState,
        ctx: &mut RenderContext<'_>,
    ) -> Result<(), Error> {
        let t0 = SystemTime::now();

        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1), //
            Constraint::Length(1),
        ])
        .split(area);

        logscroll::render(area, buf, &mut state.logscroll, ctx)?;

        if state.error_dlg.active() {
            let err = MsgDialog::new().styles(ctx.g.theme.msg_dialog_style());
            err.render(area, buf, &mut state.error_dlg);
        }

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        state
            .status
            .status(1, format!("R{} {:.0?}", ctx.count, el).to_string());

        let status = StatusLine::new()
            .layout([
                Constraint::Fill(1),
                Constraint::Length(12),
                Constraint::Length(12),
            ])
            .styles(ctx.g.theme.statusline_style());
        status.render(layout[2], buf, &mut state.status);

        Ok(())
    }

    pub fn init(state: &mut SceneryState, ctx: &mut AppContext<'_>) -> Result<(), Error> {
        ctx.focus = Some(FocusBuilder::build_for(&state.logscroll));
        logscroll::init(&mut state.logscroll, ctx)?;
        Ok(())
    }

    pub fn event(
        event: &LogScrollEvent,
        state: &mut SceneryState,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<LogScrollEvent>, Error> {
        let t0 = SystemTime::now();

        let mut r = match event {
            LogScrollEvent::Event(event) => {
                ctx.focus = Some(FocusBuilder::rebuild_for(
                    &state.logscroll,
                    ctx.focus.take(),
                ));

                let mut r = match &event {
                    ct_event!(resized) => Control::Changed,
                    ct_event!(key press CONTROL-'q') => Control::Quit,
                    _ => Control::Continue,
                };

                r = r.or_else(|| {
                    if state.error_dlg.active() {
                        state.error_dlg.handle(event, Dialog).into()
                    } else {
                        Control::Continue
                    }
                });

                let f = ctx.focus_mut().handle(event, Regular);
                ctx.queue(f);

                r
            }
            LogScrollEvent::Message(s) => {
                state.error_dlg.append(s.as_str());
                Control::Changed
            }
            LogScrollEvent::Status(n, s) => {
                state.status.status(*n, s);
                Control::Changed
            }
            _ => Control::Continue,
        };

        r = r.or_else_try(|| logscroll::event(event, &mut state.logscroll, ctx))?;

        let el = t0.elapsed()?;
        state.status.status(2, format!("E {:.0?}", el).to_string());

        Ok(r)
    }

    pub fn error(
        event: Error,
        _state: &mut SceneryState,
        _ctx: &mut AppContext<'_>,
    ) -> Result<Control<LogScrollEvent>, Error> {
        debug!("ERROR {:#?}", event);
        // self.error_dlg.append(format!("{:?}", &*event).as_str());
        Ok(Control::Changed)
    }
}

mod logscroll {
    use crate::event::LogScrollEvent;
    use crate::{AppContext, RenderContext};
    use anyhow::Error;
    use rat_salsa2::Control;
    use rat_widget::event::{try_flow, HandleEvent, ReadOnly};
    use rat_widget::focus::impl_has_focus;
    use rat_widget::text::impl_screen_cursor;
    use rat_widget::textarea::{TextArea, TextAreaState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Layout, Rect};
    use ratatui::widgets::StatefulWidget;

    #[derive(Debug)]
    pub(crate) struct LogScroll;

    #[derive(Debug, Default)]
    pub struct LogScrollState {
        log_text: TextAreaState,
    }

    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        state: &mut LogScrollState,
        ctx: &mut RenderContext<'_>,
    ) -> Result<(), Error> {
        let l0 = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1), //
            Constraint::Length(1),
        ])
        .split(area);

        let l1 = Layout::horizontal([Constraint::Fill(1)]).split(l0[1]);

        TextArea::new().styles(ctx.g.theme.textarea_style()).render(
            l1[0],
            buf,
            &mut state.log_text,
        );

        // TODO

        Ok(())
    }

    impl_has_focus!(log_text for LogScrollState);
    impl_screen_cursor!(log_text for LogScrollState);

    pub fn init(_state: &mut LogScrollState, ctx: &mut AppContext<'_>) -> Result<(), Error> {
        ctx.focus().first();
        Ok(())
    }

    #[allow(unused_variables)]
    pub fn event(
        event: &LogScrollEvent,
        state: &mut LogScrollState,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<LogScrollEvent>, Error> {
        let r = match event {
            LogScrollEvent::Event(event) => {
                try_flow!(state.log_text.handle(event, ReadOnly));

                Control::Continue
            }
            _ => Control::Continue,
        };

        // TODO

        Ok(r)
    }
}

#[derive(Debug, Default)]
pub struct LogScrollConfig {}

#[derive(Debug)]
pub struct GlobalState {
    pub cfg: LogScrollConfig,
    pub theme: DarkTheme,
    pub filewatch: FileWatcher,
    pub current: PathBuf,
}

impl GlobalState {
    pub fn new(cfg: LogScrollConfig, theme: DarkTheme, filewatch: FileWatcher) -> Self {
        Self {
            cfg,
            theme,
            filewatch,
            current: Default::default(),
        }
    }
}

fn setup_logging() -> Result<(), Error> {
    if let Some(cache) = dirs::cache_dir() {
        let log_path = if cfg!(debug_assertions) {
            PathBuf::from(".")
        } else {
            let log_path = cache.join("rat-salsa");
            if !log_path.exists() {
                fs::create_dir_all(&log_path)?;
            }
            log_path
        };

        let log_file = log_path.join("logscroll.log");
        _ = fs::remove_file(&log_file);
        fern::Dispatch::new()
            .format(|out, message, _record| {
                out.finish(format_args!("{}", message)) //
            })
            .level(log::LevelFilter::Debug)
            .chain(fern::log_file(&log_file)?)
            .apply()?;
    }
    Ok(())
}
