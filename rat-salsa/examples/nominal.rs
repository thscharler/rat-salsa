use crate::nominal::{Nominal, NominalState};
use anyhow::Error;
use rat_salsa::event::RenderedEvent;
use rat_salsa::poll::{PollCrossterm, PollRendered, PollTasks, PollTimers};
use rat_salsa::timer::TimeOut;
use rat_salsa::{run_tui, Control, RunConfig, SalsaAppContext, SalsaContext};
use rat_theme2::palettes::IMPERIAL;
use rat_theme2::DarkTheme;
use rat_widget::event::{ct_event, ConsumedEvent, Dialog, HandleEvent, Regular};
use rat_widget::focus::FocusBuilder;
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::statusline::{StatusLine, StatusLineState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::StatefulWidget;
use std::fs;
use std::time::{Duration, SystemTime};

fn main() -> Result<(), Error> {
    setup_logging()?;

    let config = Config::default();
    let theme = DarkTheme::new("Imperial".into(), IMPERIAL);
    let mut global = Global::new(config, theme);
    let mut state = Scenery::default();

    run_tui(
        init,
        render,
        event,
        error,
        &mut global,
        &mut state,
        RunConfig::default()?
            .poll(PollCrossterm)
            .poll(PollTimers::default())
            .poll(PollTasks::default())
            .poll(PollRendered),
    )?;

    Ok(())
}

/// Globally accessible data/state.
#[derive(Debug)]
pub struct Global {
    ctx: SalsaAppContext<AppEvent, Error>,
    pub cfg: Config,
    pub theme: DarkTheme,
}

impl SalsaContext<AppEvent, Error> for Global {
    fn set_salsa_ctx(&mut self, app_ctx: SalsaAppContext<AppEvent, Error>) {
        self.ctx = app_ctx;
    }

    #[inline(always)]
    fn salsa_ctx(&self) -> &SalsaAppContext<AppEvent, Error> {
        &self.ctx
    }
}

impl Global {
    pub fn new(cfg: Config, theme: DarkTheme) -> Self {
        Self {
            ctx: Default::default(),
            cfg,
            theme,
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
    pub nominal: NominalState,
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

    Nominal.render(area, buf, &mut state.nominal, ctx)?;

    let layout = Layout::vertical([
        Constraint::Fill(1), //
        Constraint::Length(1),
    ])
    .split(area);

    if state.error_dlg.active() {
        MsgDialog::new()
            .styles(ctx.theme.msg_dialog_style())
            .render(layout[0], buf, &mut state.error_dlg);
    }

    let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
    state.status.status(2, format!("R {:.0?}", el).to_string());

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
            Constraint::Length(8),
        ])
        .styles(ctx.theme.statusline_style())
        .render(status_layout[1], buf, &mut state.status);

    Ok(())
}

pub fn init(state: &mut Scenery, ctx: &mut Global) -> Result<(), Error> {
    ctx.set_focus(FocusBuilder::build_for(&state.nominal));
    state.nominal.init(ctx)?;
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
            ctx.set_focus(FocusBuilder::rebuild_for(&state.nominal, ctx.take_focus()));
            Control::Continue
        }
        AppEvent::Message(s) => {
            state.error_dlg.append(s.as_str());
            Control::Changed
        }
        AppEvent::Status(n, s) => {
            state.status.status(*n, s);
            Control::Changed
        }
        _ => Control::Continue,
    };

    r = r.or_else_try(|| state.nominal.event(event, ctx))?;

    let el = t0.elapsed()?;
    state.status.status(3, format!("E {:.0?}", el).to_string());

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

pub mod nominal {
    use crate::{AppEvent, Global};
    use anyhow::Error;
    use rat_salsa::timer::TimerDef;
    use rat_salsa::{Control, SalsaContext};
    use rat_widget::event::{try_flow, HandleEvent, MenuOutcome, Regular};
    use rat_widget::focus::impl_has_focus;
    use rat_widget::menu::{MenuLine, MenuLineState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Layout, Rect};
    use ratatui::widgets::StatefulWidget;
    use std::thread::sleep;
    use std::time::Duration;

    pub struct Nominal;

    #[derive(Debug, Default)]
    pub struct NominalState {
        pub menu: MenuLineState,
    }

    impl Nominal {
        pub fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut NominalState,
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

            MenuLine::new()
                .styles(ctx.theme.menu_style())
                .item_parsed("_Thread")
                .item_parsed("_Timer")
                .item_parsed("_Quit")
                .render(r[1], buf, &mut state.menu);

            Ok(())
        }
    }

    impl_has_focus!(menu for NominalState);

    impl NominalState {
        pub fn init(
            &mut self, //
            ctx: &mut Global,
        ) -> Result<(), Error> {
            ctx.focus().first();
            Ok(())
        }

        #[allow(unused_variables)]
        pub fn event(
            &mut self,
            event: &AppEvent,
            ctx: &mut Global,
        ) -> Result<Control<AppEvent>, Error> {
            match event {
                AppEvent::Event(event) => {
                    try_flow!(match self.menu.handle(event, Regular) {
                        MenuOutcome::Activated(0) => {
                            ctx.spawn(|| {
                                sleep(Duration::from_secs(5));
                                Ok(Control::Event(AppEvent::Message(
                                    "waiting is over".to_string(),
                                )))
                            })?;
                            Control::Changed
                        }
                        MenuOutcome::Activated(1) => {
                            ctx.add_timer(
                                TimerDef::new().repeat(21).timer(Duration::from_millis(500)),
                            );
                            Control::Changed
                        }
                        MenuOutcome::Activated(2) => {
                            Control::Quit //
                        }
                        v => v.into(),
                    });
                    Ok(Control::Continue)
                }
                AppEvent::Timer(t) => {
                    Ok(Control::Event(
                        //
                        AppEvent::Status(1, format!("TICK-{}", t.counter)),
                    ))
                }
                _ => Ok(Control::Continue),
            }
        }
    }
}

fn setup_logging() -> Result<(), Error> {
    if let Some(cache) = dirs::cache_dir() {
        let log_path = cache.join("rat-salsa");
        if !log_path.exists() {
            fs::create_dir_all(&log_path)?;
        }

        let log_file = log_path.join("minimal.log");
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
