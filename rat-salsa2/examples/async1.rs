use crate::event::Async1Event;
use crate::scenery::SceneryState;
use anyhow::Error;
use dirs::cache_dir;
use rat_salsa2::poll::{PollCrossterm, PollRendered, PollTasks, PollTimers};
use rat_salsa2::{run_tui, RunConfig};
use rat_theme2::palettes::IMPERIAL;
use rat_theme2::DarkTheme;
use std::fs;
use std::fs::create_dir_all;
use std::rc::Rc;

type AppContext<'a> = rat_salsa2::AppContext<'a, GlobalState, Async1Event, Error>;
type RenderContext<'a> = rat_salsa2::RenderContext<'a, GlobalState>;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let rt = tokio::runtime::Runtime::new()?;

    let config = MinimalConfig::default();
    let theme = DarkTheme::new("Imperial".into(), IMPERIAL);
    let mut global = GlobalState::new(config, theme);

    let mut state = SceneryState::default();

    let mut run_cfg = RunConfig::default()?
        .poll(PollCrossterm)
        .poll(PollTimers::default())
        .poll(PollTasks::default())
        .poll(PollRendered);
    if cfg!(feature = "async") {
        run_cfg = run_cfg.poll(rat_salsa2::poll::PollTokio::new(rt));
    }

    run_tui(
        scenery::init,
        scenery::render,
        scenery::event,
        scenery::error,
        &mut global,
        &mut state,
        run_cfg,
    )?;

    Ok(())
}

/// Globally accessible data/state.
#[derive(Debug)]
pub struct GlobalState {
    pub cfg: MinimalConfig,
    pub theme: Rc<DarkTheme>,
}

impl GlobalState {
    pub fn new(cfg: MinimalConfig, theme: DarkTheme) -> Self {
        Self {
            cfg,
            theme: Rc::new(theme),
        }
    }
}

/// Configuration.
#[derive(Debug, Default)]
pub struct MinimalConfig {}

/// Application wide messages.
pub mod event {
    use rat_salsa2::rendered::RenderedEvent;
    use rat_salsa2::timer::TimeOut;

    #[derive(Debug)]
    pub enum Async1Event {
        Timer(TimeOut),
        Event(crossterm::event::Event),
        Rendered,
        Message(String),
        Status(usize, String),
        FromAsync(String),
        AsyncTick(u32),
    }

    impl From<RenderedEvent> for Async1Event {
        fn from(_: RenderedEvent) -> Self {
            Self::Rendered
        }
    }

    impl From<TimeOut> for Async1Event {
        fn from(value: TimeOut) -> Self {
            Self::Timer(value)
        }
    }

    impl From<crossterm::event::Event> for Async1Event {
        fn from(value: crossterm::event::Event) -> Self {
            Self::Event(value)
        }
    }
}

pub mod scenery {
    use crate::async1;
    use crate::async1::Async1State;
    use crate::event::Async1Event;
    use crate::{AppContext, RenderContext};
    use anyhow::Error;
    use rat_salsa2::Control;
    use rat_widget::event::{ct_event, ConsumedEvent, Dialog, HandleEvent, Regular};
    use rat_widget::focus::FocusBuilder;
    use rat_widget::layout::layout_middle;
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
        pub async1: Async1State,
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
        let theme = ctx.g.theme.clone();

        let layout = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

        async1::render(area, buf, &mut state.async1, ctx)?;

        if state.error_dlg.active() {
            let err = MsgDialog::new().styles(theme.msg_dialog_style());
            err.render(
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

        let status_layout =
            Layout::horizontal([Constraint::Fill(61), Constraint::Fill(39)]).split(layout[1]);
        let status = StatusLine::new()
            .layout([
                Constraint::Fill(1),
                Constraint::Length(8),
                Constraint::Length(8),
            ])
            .styles(theme.statusline_style());
        status.render(status_layout[1], buf, &mut state.status);

        Ok(())
    }

    pub fn init(state: &mut SceneryState, ctx: &mut AppContext<'_>) -> Result<(), Error> {
        ctx.focus = Some(FocusBuilder::build_for(&state.async1));
        async1::init(&mut state.async1, ctx)?;
        Ok(())
    }

    pub fn event(
        event: &Async1Event,
        state: &mut SceneryState,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<Async1Event>, Error> {
        let t0 = SystemTime::now();

        let mut r = match event {
            Async1Event::Event(event) => {
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
            Async1Event::Rendered => {
                ctx.focus = Some(FocusBuilder::rebuild_for(&state.async1, ctx.focus.take()));
                Control::Continue
            }
            Async1Event::Message(s) => {
                state.error_dlg.append(&*s);
                Control::Changed
            }
            Async1Event::Status(n, s) => {
                state.status.status(*n, s);
                Control::Changed
            }
            _ => Control::Continue,
        };

        r = r.or_else_try(|| async1::event(event, &mut state.async1, ctx))?;

        let el = t0.elapsed()?;
        state.status.status(2, format!("E {:.0?}", el).to_string());

        Ok(r)
    }

    pub fn error(
        event: Error,
        state: &mut SceneryState,
        _ctx: &mut AppContext<'_>,
    ) -> Result<Control<Async1Event>, Error> {
        state.error_dlg.append(format!("{:?}", &*event).as_str());
        Ok(Control::Changed)
    }
}

pub mod async1 {
    use crate::event::Async1Event;
    use crate::{AppContext, RenderContext};
    use anyhow::Error;
    use rat_salsa2::Control;
    use rat_widget::event::{HandleEvent, MenuOutcome, Regular};
    use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus};
    use rat_widget::menu::{MenuLine, MenuLineState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Layout, Rect};
    use ratatui::widgets::StatefulWidget;
    use std::time::Duration;

    #[derive(Debug, Default)]
    pub struct Async1State {
        pub menu: MenuLineState,
    }

    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        state: &mut Async1State,
        ctx: &mut RenderContext<'_>,
    ) -> Result<(), Error> {
        // TODO: repaint_mask
        let theme = ctx.g.theme.clone();

        let r = Layout::new(
            Direction::Vertical,
            [
                Constraint::Fill(1), //
                Constraint::Length(1),
            ],
        )
        .split(area);

        let menu = MenuLine::new()
            .styles(theme.menu_style())
            .item_parsed("_Simple Async")
            .item_parsed("_Long Running")
            .item_parsed("_Quit");
        menu.render(r[1], buf, &mut state.menu);

        Ok(())
    }

    impl HasFocus for Async1State {
        fn build(&self, builder: &mut FocusBuilder) {
            builder.widget(&self.menu);
        }

        fn focus(&self) -> FocusFlag {
            unimplemented!("not in use, silent container")
        }

        fn area(&self) -> Rect {
            unimplemented!("not in use, silent container")
        }
    }

    pub fn init(
        _state: &mut Async1State, //
        ctx: &mut AppContext<'_>,
    ) -> Result<(), Error> {
        ctx.focus().first();
        Ok(())
    }

    pub fn event(
        event: &Async1Event,
        state: &mut Async1State,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<Async1Event>, Error> {
        let r = match event {
            Async1Event::Event(event) => match state.menu.handle(event, Regular) {
                MenuOutcome::Activated(0) => {
                    // spawn async task ...
                    ctx.spawn_async(async {
                        // to some awaiting
                        tokio::time::sleep(Duration::from_secs(2)).await;

                        Ok(Control::Event(Async1Event::FromAsync(
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
                            _ = chan
                                .send(Ok(Control::Event(Async1Event::AsyncTick(i))))
                                .await
                        }

                        Ok(Control::Event(Async1Event::AsyncTick(300)))
                    });
                    // that's it.
                    Control::Continue
                }
                MenuOutcome::Activated(2) => Control::Quit,
                v => v.into(),
            },
            Async1Event::FromAsync(s) => {
                // receive result from async operation
                Control::Event(Async1Event::Message(s.clone()))
            }
            Async1Event::AsyncTick(n) => {
                Control::Event(Async1Event::Status(0, format!("--- {} ---", n)))
            }
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
