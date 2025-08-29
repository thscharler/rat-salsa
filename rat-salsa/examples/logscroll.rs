#![allow(dead_code)]

use crate::event::LogScrollEvent;
use crate::scenery::{Scenery, SceneryState};
use anyhow::Error;
use log::debug;
use rat_salsa::poll::{PollCrossterm, PollTimers};
use rat_salsa::{run_tui, RunConfig};
use rat_theme2::palettes::IMPERIAL;
use rat_theme2::DarkTheme;
use std::env::args;
use std::fs;
use std::path::PathBuf;

type AppContext<'a> = rat_salsa::AppContext<'a, GlobalState, LogScrollEvent, Error>;
type RenderContext<'a> = rat_salsa::RenderContext<'a, GlobalState>;

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

    let mut global = GlobalState::new(config, theme);
    global.current = current;

    let app = Scenery;
    let mut state = SceneryState::default();

    run_tui(
        app,
        &mut global,
        &mut state,
        RunConfig::default()?
            .poll(PollCrossterm)
            .poll(PollTimers::new()),
    )?;

    Ok(())
}

mod event {
    use rat_salsa::timer::TimeOut;

    #[derive(Debug)]
    pub enum LogScrollEvent {
        Event(crossterm::event::Event),
        TimeOut(TimeOut),
        Message(String),
        Status(usize, String),
    }

    impl From<crossterm::event::Event> for LogScrollEvent {
        fn from(value: crossterm::event::Event) -> Self {
            Self::Event(value)
        }
    }

    impl From<TimeOut> for LogScrollEvent {
        fn from(value: TimeOut) -> Self {
            Self::TimeOut(value)
        }
    }
}

mod scenery {
    use crate::event::LogScrollEvent;
    use crate::logscroll::{LogScroll, LogScrollState};
    use crate::{AppContext, GlobalState, RenderContext};
    use anyhow::Error;
    use log::debug;
    use rat_salsa::timer::TimerDef;
    use rat_salsa::{AppState, AppWidget, Control};
    use rat_widget::event::{ct_event, ConsumedEvent, Dialog, HandleEvent, Regular};
    use rat_widget::focus::FocusBuilder;
    use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
    use rat_widget::statusline::{StatusLine, StatusLineState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Layout, Rect};
    use ratatui::widgets::StatefulWidget;
    use std::time::{Duration, SystemTime};

    #[derive(Debug)]
    pub struct Scenery;

    #[derive(Debug, Default)]
    pub struct SceneryState {
        pub logscroll: LogScrollState,
        pub status: StatusLineState,
        pub error_dlg: MsgDialogState,
    }

    impl AppWidget<GlobalState, LogScrollEvent, Error> for Scenery {
        type State = SceneryState;

        fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut Self::State,
            ctx: &mut RenderContext<'_>,
        ) -> Result<(), Error> {
            let t0 = SystemTime::now();

            let layout = Layout::vertical([
                Constraint::Length(1),
                Constraint::Fill(1), //
                Constraint::Length(1),
            ])
            .split(area);

            LogScroll.render(area, buf, &mut state.logscroll, ctx)?;

            if state.error_dlg.active() {
                let err = MsgDialog::new().styles(ctx.g.theme.msg_dialog_style());
                err.render(area, buf, &mut state.error_dlg);
            }

            let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
            state
                .status
                .status(2, format!("R{} {:.0?}", ctx.count, el).to_string());

            let status = StatusLine::new()
                .layout([
                    Constraint::Fill(1),
                    Constraint::Length(12),
                    Constraint::Length(12),
                    Constraint::Length(12),
                ])
                .styles(ctx.g.theme.statusline_style());
            status.render(layout[2], buf, &mut state.status);

            Ok(())
        }
    }

    impl AppState<GlobalState, LogScrollEvent, Error> for SceneryState {
        fn init(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            ctx.focus = Some(FocusBuilder::build_for(&self.logscroll));
            self.logscroll.init(ctx)?;
            Ok(())
        }

        fn event(
            &mut self,
            event: &LogScrollEvent,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, LogScrollEvent, Error>,
        ) -> Result<Control<LogScrollEvent>, Error> {
            let t0 = SystemTime::now();

            let mut r = match event {
                LogScrollEvent::Event(event) => {
                    ctx.focus = Some(FocusBuilder::rebuild_for(&self.logscroll, ctx.focus.take()));

                    let mut r = match &event {
                        ct_event!(resized) => Control::Changed,
                        ct_event!(key press CONTROL-'q') => Control::Quit,
                        _ => Control::Continue,
                    };

                    r = r.or_else(|| {
                        if self.error_dlg.active() {
                            self.error_dlg.handle(event, Dialog).into()
                        } else {
                            Control::Continue
                        }
                    });

                    let f = ctx.focus_mut().handle(event, Regular);
                    ctx.queue(f);

                    r
                }
                LogScrollEvent::Message(s) => {
                    self.error_dlg.append(s.as_str());
                    Control::Changed
                }
                LogScrollEvent::Status(n, s) => {
                    self.status.status(*n, s);
                    Control::Changed
                }
                _ => Control::Continue,
            };

            r = r.or_else_try(|| self.logscroll.event(event, ctx))?;

            let el = t0.elapsed()?;
            self.status.status(3, format!("E {:.0?}", el).to_string());

            Ok(r)
        }

        fn error(
            &self,
            event: Error,
            _ctx: &mut AppContext<'_>,
        ) -> Result<Control<LogScrollEvent>, Error> {
            debug!("ERROR {:#?}", event);
            // self.error_dlg.append(format!("{:?}", &*event).as_str());
            Ok(Control::Changed)
        }
    }
}

mod logscroll {
    use crate::event::LogScrollEvent;
    use crate::{GlobalState, RenderContext};
    use anyhow::Error;
    use log::debug;
    use rat_salsa::timer::{TimeOut, TimerDef, TimerHandle};
    use rat_salsa::{AppState, AppWidget, Control};
    use rat_widget::event::{ct_event, try_flow, HandleEvent, ReadOnly};
    use rat_widget::focus::impl_has_focus;
    use rat_widget::scrolled::Scroll;
    use rat_widget::text::{impl_screen_cursor, HasScreenCursor, TextPosition};
    use rat_widget::textarea::{TextArea, TextAreaState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Layout, Rect};
    use ratatui::widgets::StatefulWidget;
    use ropey::RopeBuilder;
    use std::fs::File;
    use std::io::{BufReader, Read, Seek, SeekFrom};
    use std::path::Path;
    use std::time::Duration;

    #[derive(Debug)]
    pub(crate) struct LogScroll;

    #[derive(Debug, Default)]
    pub struct LogScrollState {
        log_text: TextAreaState,
        pollution: TimerHandle,
    }

    impl AppWidget<GlobalState, LogScrollEvent, Error> for LogScroll {
        type State = LogScrollState;

        fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut Self::State,
            ctx: &mut RenderContext<'_>,
        ) -> Result<(), Error> {
            let l0 = Layout::vertical([
                Constraint::Length(1),
                Constraint::Fill(1), //
                Constraint::Length(1),
            ])
            .split(area);

            let l1 = Layout::horizontal([
                Constraint::Fill(1), //
                Constraint::Length(25),
            ])
            .split(l0[1]);

            TextArea::new()
                .vscroll(Scroll::new())
                .hscroll(Scroll::new())
                .styles(ctx.g.theme.textarea_style())
                .render(l1[0], buf, &mut state.log_text);

            // TODO

            ctx.set_screen_cursor(state.log_text.screen_cursor());

            Ok(())
        }
    }

    impl_has_focus!(log_text for LogScrollState);
    impl_screen_cursor!(log_text for LogScrollState);

    impl LogScrollState {
        fn load_file(&mut self, path: &Path) -> Result<(), Error> {
            let mut f = File::open(path)?;

            let mut buf = String::with_capacity(4096);
            let mut txt = RopeBuilder::new();
            loop {
                match f.read_to_string(&mut buf) {
                    Ok(0) => {
                        break;
                    }
                    Ok(_) => {
                        txt.append(&buf);
                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }
            self.log_text.set_rope(txt.finish());
            self.log_text
                .set_cursor((0, self.log_text.len_lines()), false);
            self.log_text.scroll_cursor_to_visible();

            Ok(())
        }

        fn update_file(&mut self, path: &Path) -> Result<bool, Error> {
            let mut rope = self.log_text.rope().clone();
            let len_bytes = rope.len_bytes();

            if path.metadata()?.len() == len_bytes as u64 {
                return Ok(false);
            }

            let mut f = File::open(path)?;
            f.seek(SeekFrom::Start(len_bytes as u64))?;

            let cursor_at_end = self.log_text.cursor().y == self.log_text.len_lines()
                || self.log_text.cursor().y == self.log_text.len_lines() - 1;

            let mut buf = String::with_capacity(4096);
            loop {
                match f.read_to_string(&mut buf) {
                    Ok(0) => {
                        break;
                    }
                    Ok(_) => {
                        let pos = TextPosition::new(0, self.log_text.len_lines());
                        self.log_text.value.insert_str(pos, &buf).expect("fine");
                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }

            if cursor_at_end {
                self.log_text
                    .set_cursor((0, self.log_text.len_lines()), false);
                self.log_text.scroll_cursor_to_visible();
            }

            Ok(true)
        }
    }

    impl AppState<GlobalState, LogScrollEvent, Error> for LogScrollState {
        fn init(
            &mut self,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, LogScrollEvent, Error>,
        ) -> Result<(), Error> {
            ctx.focus().first();

            self.load_file(&ctx.g.current)?;

            ctx.add_timer(
                TimerDef::new()
                    .repeat_forever()
                    .timer(Duration::from_millis(100)),
            );

            self.pollution = ctx.add_timer(
                TimerDef::new()
                    .repeat_forever()
                    .timer(Duration::from_millis(10)),
            );

            Ok(())
        }

        #[allow(unused_variables)]
        fn event(
            &mut self,
            event: &LogScrollEvent,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, LogScrollEvent, Error>,
        ) -> Result<Control<LogScrollEvent>, Error> {
            let r = match event {
                LogScrollEvent::Event(event) => {
                    try_flow!(self.log_text.handle(event, ReadOnly));

                    debug!("event");
                    try_flow!(match event {
                        ct_event!(keycode press F(1)) => {
                            debug!("{:#?}", ctx);
                            Control::Continue
                        }
                        _ => Control::Continue,
                    });

                    Control::Continue
                }
                LogScrollEvent::TimeOut(t) if t.handle == self.pollution => {
                    debug!("pollution {:?}", t);
                    Control::Continue
                }
                LogScrollEvent::TimeOut(t) if t.handle != self.pollution => {
                    if self.update_file(&ctx.g.current)? {
                        ctx.queue(Control::Event(LogScrollEvent::Status(
                            0,
                            format!("{} lines", self.log_text.len_lines()),
                        )));
                        Control::Changed
                    } else {
                        Control::Continue
                    }
                }
                _ => Control::Continue,
            };

            // TODO

            Ok(r)
        }
    }
}

#[derive(Debug, Default)]
pub struct LogScrollConfig {}

#[derive(Debug)]
pub struct GlobalState {
    pub cfg: LogScrollConfig,
    pub theme: DarkTheme,
    pub current: PathBuf,
}

impl GlobalState {
    pub fn new(cfg: LogScrollConfig, theme: DarkTheme) -> Self {
        Self {
            cfg,
            theme,
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
