use crate::main_ui::MainUI;
use anyhow::Error;
use dirs::cache_dir;
use rat_salsa2::poll::{PollCrossterm, PollRendered, PollTasks, PollTimers};
use rat_salsa2::rendered::RenderedEvent;
use rat_salsa2::timer::TimeOut;
use rat_salsa2::{run_tui, Control, RunConfig, SalsaAppContext, SalsaContext};
use rat_theme2::palettes::IMPERIAL;
use rat_theme2::DarkTheme;
use rat_widget::event::{ct_event, ConsumedEvent, Dialog, HandleEvent, Regular};
use rat_widget::focus::FocusBuilder;
use rat_widget::layout::layout_middle;
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::statusline::{StatusLine, StatusLineState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::StatefulWidget;
use std::fs;
use std::fs::create_dir_all;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

fn main() -> Result<(), Error> {
    setup_logging()?;

    let rt = tokio::runtime::Runtime::new()?;

    let config = Config::default();
    let theme = DarkTheme::new("Imperial".into(), IMPERIAL);
    let mut global = Global::new(config, theme);
    let mut state = Scenery::default();

    run_tui(
        init, //
        render,
        event,
        error,
        &mut global,
        &mut state,
        RunConfig::default()?
            .poll(PollCrossterm)
            .poll(PollTimers::default())
            .poll(PollTasks::default())
            .poll(PollRendered)
            .poll(rat_salsa2::poll::PollTokio::new(rt)),
    )?;

    Ok(())
}

/// Globally accessible data/state.
#[derive(Debug)]
pub struct Global {
    pub ctx: SalsaAppContext<AppEvent, Error>,
    pub cfg: Config,
    pub theme: Rc<DarkTheme>,
}

impl SalsaContext<AppEvent, Error> for Global {
    fn set_salsa_ctx(&mut self, app_ctx: SalsaAppContext<AppEvent, Error>) {
        self.ctx = app_ctx;
    }

    fn salsa_ctx(&self) -> &SalsaAppContext<AppEvent, Error> {
        &self.ctx
    }
}

impl Global {
    pub fn new(cfg: Config, theme: DarkTheme) -> Self {
        Self {
            ctx: Default::default(),
            cfg,
            theme: Rc::new(theme),
        }
    }
}

/// Configuration.
#[derive(Debug, Default)]
pub struct Config {}

/// Application wide messages.
#[derive(Debug)]
pub enum AppEvent {
    Timer(TimeOut),
    Event(crossterm::event::Event),
    Rendered,
    Message(String),
    Status(usize, String),
    AsyncMsg(String),
    AsyncTick(u32),
}

impl From<RenderedEvent> for AppEvent {
    fn from(_: RenderedEvent) -> Self {
        Self::Rendered
    }
}

impl From<TimeOut> for AppEvent {
    fn from(value: TimeOut) -> Self {
        Self::Timer(value)
    }
}

impl From<crossterm::event::Event> for AppEvent {
    fn from(value: crossterm::event::Event) -> Self {
        Self::Event(value)
    }
}

#[derive(Debug, Default)]
pub struct Scenery {
    pub async1: MainUI,
    pub status: StatusLineState,
    pub error_dlg: MsgDialogState,
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut Scenery,
    ctx: &mut Global,
) -> Result<(), Error> {
    let t0 = SystemTime::now();

    // forward
    main_ui::render(area, buf, &mut state.async1, ctx)?;

    let layout = Layout::vertical([
        Constraint::Fill(1), //
        Constraint::Length(1),
    ])
    .split(area);

    if state.error_dlg.active() {
        MsgDialog::new()
            .styles(ctx.theme.msg_dialog_style())
            .render(
                layout_middle(
                    layout[0],
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ),
                buf,
                &mut state.error_dlg,
            );
    }

    let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
    state.status.status(1, format!("R {:.0?}", el).to_string());

    let status_layout = Layout::horizontal([
        Constraint::Fill(61), //
        Constraint::Fill(39),
    ])
    .split(layout[1]);

    StatusLine::new()
        .layout([
            Constraint::Fill(1),
            Constraint::Length(8),
            Constraint::Length(8),
        ])
        .styles(ctx.theme.statusline_style())
        .render(status_layout[1], buf, &mut state.status);

    Ok(())
}

pub fn init(state: &mut Scenery, ctx: &mut Global) -> Result<(), Error> {
    ctx.set_focus(FocusBuilder::build_for(&state.async1));
    main_ui::init(&mut state.async1, ctx)?;
    Ok(())
}

pub fn event(
    event: &AppEvent,
    state: &mut Scenery,
    ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    let t0 = SystemTime::now();

    let mut r = match event {
        AppEvent::Event(event) => {
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
        AppEvent::Rendered => {
            ctx.set_focus(FocusBuilder::rebuild_for(&state.async1, ctx.take_focus()));
            Control::Continue
        }
        AppEvent::Message(s) => {
            state.error_dlg.append(&*s);
            Control::Changed
        }
        AppEvent::Status(n, s) => {
            state.status.status(*n, s);
            Control::Changed
        }
        _ => Control::Continue,
    };

    r = r.or_else_try(|| main_ui::event(event, &mut state.async1, ctx))?;

    let el = t0.elapsed()?;
    state.status.status(2, format!("E {:.0?}", el).to_string());

    Ok(r)
}

pub fn error(
    event: Error,
    state: &mut Scenery,
    _ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    state.error_dlg.append(format!("{:?}", &*event).as_str());
    Ok(Control::Changed)
}

pub mod main_ui {
    use crate::AppEvent;
    use crate::Global;
    use anyhow::Error;
    use rat_focus::impl_has_focus;
    use rat_salsa2::{Control, SalsaContext};
    use rat_widget::event::{HandleEvent, MenuOutcome, Regular};
    use rat_widget::menu::{MenuLine, MenuLineState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Layout, Rect};
    use ratatui::widgets::StatefulWidget;
    use std::time::Duration;

    #[derive(Debug, Default)]
    pub struct MainUI {
        pub menu: MenuLineState,
    }

    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        state: &mut MainUI,
        ctx: &mut Global,
    ) -> Result<(), Error> {
        // TODO: repaint_mask
        let r = Layout::new(
            Direction::Vertical,
            [
                Constraint::Fill(1), //
                Constraint::Length(1),
            ],
        )
        .split(area);

        let menu = MenuLine::new()
            .styles(ctx.theme.menu_style())
            .item_parsed("_Simple Async")
            .item_parsed("_Long Running")
            .item_parsed("_Quit");
        menu.render(r[1], buf, &mut state.menu);

        Ok(())
    }

    impl_has_focus!(menu for MainUI);

    pub fn init(
        _state: &mut MainUI, //
        ctx: &mut Global,
    ) -> Result<(), Error> {
        ctx.focus().first();
        Ok(())
    }

    pub fn event(
        event: &AppEvent,
        state: &mut MainUI,
        ctx: &mut Global,
    ) -> Result<Control<AppEvent>, Error> {
        let r = match event {
            AppEvent::Event(event) => match state.menu.handle(event, Regular) {
                MenuOutcome::Activated(0) => {
                    // spawn async task ...
                    ctx.spawn_async(async {
                        // to some awaiting
                        tokio::time::sleep(Duration::from_secs(2)).await;

                        Ok(Control::Event(AppEvent::AsyncMsg(
                            "result of async computation".into(),
                        )))
                    });
                    // that's it.
                    Control::Continue
                }
                MenuOutcome::Activated(1) => {
                    // spawn async task ...
                    ctx.spawn_async_ext(|chan| async move {
                        // to some awaiting
                        let period = Duration::from_secs_f32(1.0 / 60.0);
                        let mut interval = tokio::time::interval(period);

                        for i in 0..1200 {
                            interval.tick().await;
                            // send back intermediate results.
                            _ = chan.send(Ok(Control::Event(AppEvent::AsyncTick(i)))).await
                        }

                        Ok(Control::Event(AppEvent::AsyncTick(300)))
                    });
                    // that's it.
                    Control::Continue
                }
                MenuOutcome::Activated(2) => Control::Quit,
                v => v.into(),
            },
            AppEvent::AsyncMsg(s) => {
                // receive result from async operation
                Control::Event(AppEvent::Message(s.clone()))
            }
            AppEvent::AsyncTick(n) => Control::Event(AppEvent::Status(0, format!("--- {} ---", n))),
            _ => Control::Continue,
        };

        Ok(r)
    }
}

fn setup_logging() -> Result<(), Error> {
    if let Some(cache) = cache_dir() {
        let log_path = cache.join("rat-salsa");
        if !log_path.exists() {
            create_dir_all(&log_path)?;
        }

        let log_file = log_path.join("async1.log");
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
