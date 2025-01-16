use crate::config::MinimalConfig;
use crate::event::Async1Event;
use crate::global::GlobalState;
use crate::scenery::{Scenery, SceneryState};
use anyhow::Error;
#[cfg(feature = "async")]
use rat_salsa::poll::PollTokio;
use rat_salsa::poll::{PollCrossterm, PollRendered, PollTasks, PollTimers};
use rat_salsa::{run_tui, RunConfig};
use rat_theme::dark_theme::DarkTheme;
use rat_theme::scheme::IMPERIAL;
use std::time::SystemTime;

type AppContext<'a> = rat_salsa::AppContext<'a, GlobalState, Async1Event, Error>;
type RenderContext<'a> = rat_salsa::RenderContext<'a, GlobalState>;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let rt = tokio::runtime::Runtime::new()?;

    let config = MinimalConfig::default();
    let theme = DarkTheme::new("Imperial".into(), IMPERIAL);
    let mut global = GlobalState::new(config, theme);

    let app = Scenery;
    let mut state = SceneryState::default();

    run_tui(
        app,
        &mut global,
        &mut state,
        RunConfig::default()?
            .poll(PollCrossterm)
            .poll(PollTimers)
            .poll(PollTasks)
            .poll(PollRendered)
            .poll(PollTokio::new(rt)),
    )?;

    Ok(())
}

/// Globally accessible data/state.
pub mod global {
    use crate::config::MinimalConfig;
    use rat_theme::dark_theme::DarkTheme;
    use rat_widget::msgdialog::MsgDialogState;
    use rat_widget::statusline::StatusLineState;

    #[derive(Debug)]
    pub struct GlobalState {
        pub cfg: MinimalConfig,
        pub theme: DarkTheme,
        pub status: StatusLineState,
        pub error_dlg: MsgDialogState,
    }

    impl GlobalState {
        pub fn new(cfg: MinimalConfig, theme: DarkTheme) -> Self {
            Self {
                cfg,
                theme,
                status: Default::default(),
                error_dlg: Default::default(),
            }
        }
    }
}

/// Configuration.
pub mod config {
    #[derive(Debug, Default)]
    pub struct MinimalConfig {}
}

/// Application wide messages.
pub mod event {
    use rat_salsa::rendered::RenderedEvent;
    use rat_salsa::timer::TimeOut;

    #[derive(Debug)]
    pub enum Async1Event {
        Timer(TimeOut),
        Event(crossterm::event::Event),
        Rendered,
        Message(String),
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
    use crate::async1::{Async1, Async1State};
    use crate::event::Async1Event;
    use crate::global::GlobalState;
    use crate::{AppContext, RenderContext};
    use anyhow::Error;
    use rat_salsa::{AppState, AppWidget, Control};
    use rat_widget::event::{ct_event, ConsumedEvent, Dialog, HandleEvent, Regular};
    use rat_widget::focus::FocusBuilder;
    use rat_widget::layout::layout_middle;
    use rat_widget::msgdialog::MsgDialog;
    use rat_widget::statusline::StatusLine;
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Layout, Rect};
    use ratatui::widgets::StatefulWidget;
    use std::time::{Duration, SystemTime};

    #[derive(Debug)]
    pub struct Scenery;

    #[derive(Debug, Default)]
    pub struct SceneryState {
        pub async1: Async1State,
    }

    impl AppWidget<GlobalState, Async1Event, Error> for Scenery {
        type State = SceneryState;

        fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut Self::State,
            ctx: &mut RenderContext<'_>,
        ) -> Result<(), Error> {
            let t0 = SystemTime::now();

            let layout = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

            Async1.render(area, buf, &mut state.async1, ctx)?;

            if ctx.g.error_dlg.active() {
                let err = MsgDialog::new().styles(ctx.g.theme.msg_dialog_style());
                err.render(
                    layout_middle(
                        layout[0],
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                        Constraint::Length(1),
                        Constraint::Length(1),
                    ),
                    buf,
                    &mut ctx.g.error_dlg,
                );
            }

            let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
            ctx.g.status.status(1, format!("R {:.0?}", el).to_string());

            let status_layout =
                Layout::horizontal([Constraint::Fill(61), Constraint::Fill(39)]).split(layout[1]);
            let status = StatusLine::new()
                .layout([
                    Constraint::Fill(1),
                    Constraint::Length(8),
                    Constraint::Length(8),
                ])
                .styles(ctx.g.theme.statusline_style());
            status.render(status_layout[1], buf, &mut ctx.g.status);

            Ok(())
        }
    }

    impl AppState<GlobalState, Async1Event, Error> for SceneryState {
        fn init(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            ctx.focus = Some(FocusBuilder::build_for(&self.async1));
            self.async1.init(ctx)?;
            Ok(())
        }

        fn event(
            &mut self,
            event: &Async1Event,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, Async1Event, Error>,
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
                        if ctx.g.error_dlg.active() {
                            ctx.g.error_dlg.handle(event, Dialog).into()
                        } else {
                            Control::Continue
                        }
                    });

                    let f = ctx.focus_mut().handle(event, Regular);
                    ctx.queue(f);

                    r
                }
                Async1Event::Rendered => {
                    ctx.focus = Some(FocusBuilder::rebuild_for(&self.async1, ctx.focus.take()));
                    Control::Continue
                }
                Async1Event::Message(s) => {
                    ctx.g.status.status(0, &*s);
                    Control::Changed
                }
                _ => Control::Continue,
            };

            r = r.or_else_try(|| self.async1.event(event, ctx))?;

            let el = t0.elapsed()?;
            ctx.g.status.status(2, format!("E {:.0?}", el).to_string());

            Ok(r)
        }

        fn error(
            &self,
            event: Error,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<Async1Event>, Error> {
            ctx.g.error_dlg.append(format!("{:?}", &*event).as_str());
            Ok(Control::Changed)
        }
    }
}

pub mod async1 {
    use crate::{Async1Event, GlobalState, RenderContext};
    use anyhow::Error;
    use rat_salsa::{AppState, AppWidget, Control};
    use rat_widget::event::{HandleEvent, MenuOutcome, Regular};
    use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus};
    use rat_widget::menu::{MenuLine, MenuLineState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Layout, Rect};
    use ratatui::widgets::StatefulWidget;
    use std::time::Duration;

    #[derive(Debug)]
    pub(crate) struct Async1;

    #[derive(Debug, Default)]
    pub struct Async1State {
        pub menu: MenuLineState,
    }

    impl AppWidget<GlobalState, Async1Event, Error> for Async1 {
        type State = Async1State;

        fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut Self::State,
            ctx: &mut RenderContext<'_>,
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
                .styles(ctx.g.theme.menu_style())
                .item_parsed("_Simple Async")
                .item_parsed("_Long Running")
                .item_parsed("_Quit");
            menu.render(r[1], buf, &mut state.menu);

            Ok(())
        }
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

    impl AppState<GlobalState, Async1Event, Error> for Async1State {
        fn init(
            &mut self,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, Async1Event, Error>,
        ) -> Result<(), Error> {
            ctx.focus().first();
            self.menu.select(Some(0));
            Ok(())
        }

        #[allow(unused_variables)]
        fn event(
            &mut self,
            event: &Async1Event,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, Async1Event, Error>,
        ) -> Result<Control<Async1Event>, Error> {
            let r = match event {
                Async1Event::Event(event) => match self.menu.handle(event, Regular) {
                    MenuOutcome::Activated(0) => {
                        // spawn async task ...
                        ctx.spawn_async(async {
                            // to some awaiting
                            tokio::time::sleep(Duration::from_secs(2)).await;

                            Ok(Control::Message(Async1Event::FromAsync(
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
                                    .send(Ok(Control::Message(Async1Event::AsyncTick(i))))
                                    .await
                            }

                            Ok(Control::Message(Async1Event::AsyncTick(300)))
                        });
                        // that's it.
                        Control::Continue
                    }
                    MenuOutcome::Activated(2) => Control::Quit,
                    v => v.into(),
                },
                Async1Event::FromAsync(s) => {
                    // receive result from async operation
                    ctx.g.error_dlg.append(s);
                    Control::Changed
                }
                Async1Event::AsyncTick(n) => {
                    ctx.g.status.status(0, format!("--- {} ---", n));
                    Control::Changed
                }
                _ => Control::Continue,
            };

            Ok(r)
        }
    }
}

fn setup_logging() -> Result<(), Error> {
    // _ = fs::remove_file("log.log");
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}]\n        {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("log.log")?)
        .apply()?;
    Ok(())
}
