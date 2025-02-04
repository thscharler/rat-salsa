use crate::config::MinimalConfig;
use crate::event::MinimalEvent;
use crate::global::GlobalState;
use crate::scenery::{Scenery, SceneryState};
use anyhow::Error;
use rat_salsa::poll::{PollCrossterm, PollRendered, PollTasks, PollTimers};
use rat_salsa::{run_tui, RunConfig};
use rat_theme::dark_theme::DarkTheme;
use rat_theme::scheme::IMPERIAL;
use std::time::SystemTime;

type AppContext<'a> = rat_salsa::AppContext<'a, GlobalState, MinimalEvent, Error>;
type RenderContext<'a> = rat_salsa::RenderContext<'a, GlobalState>;

fn main() -> Result<(), Error> {
    setup_logging()?;

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
            .poll(PollTimers::default())
            .poll(PollTasks::default())
            .poll(PollRendered),
    )?;

    Ok(())
}

/// Globally accessible data/state.
pub mod global {
    use crate::config::MinimalConfig;
    use rat_theme::dark_theme::DarkTheme;

    #[derive(Debug)]
    pub struct GlobalState {
        pub cfg: MinimalConfig,
        pub theme: DarkTheme,
    }

    impl GlobalState {
        pub fn new(cfg: MinimalConfig, theme: DarkTheme) -> Self {
            Self { cfg, theme }
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
    pub enum MinimalEvent {
        Timer(TimeOut),
        Event(crossterm::event::Event),
        Rendered,
        Message(String),
        Status(usize, String),
    }

    impl From<RenderedEvent> for MinimalEvent {
        fn from(_: RenderedEvent) -> Self {
            Self::Rendered
        }
    }

    impl From<TimeOut> for MinimalEvent {
        fn from(value: TimeOut) -> Self {
            Self::Timer(value)
        }
    }

    impl From<crossterm::event::Event> for MinimalEvent {
        fn from(value: crossterm::event::Event) -> Self {
            Self::Event(value)
        }
    }
}

pub mod scenery {
    use crate::event::MinimalEvent;
    use crate::global::GlobalState;
    use crate::minimal::{Minimal, MinimalState};
    use crate::{AppContext, RenderContext};
    use anyhow::Error;
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
        pub minimal: MinimalState,
        pub status: StatusLineState,
        pub error_dlg: MsgDialogState,
    }

    impl AppWidget<GlobalState, MinimalEvent, Error> for Scenery {
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

            Minimal.render(area, buf, &mut state.minimal, ctx)?;

            if state.error_dlg.active() {
                let err = MsgDialog::new().styles(ctx.g.theme.msg_dialog_style());
                err.render(layout[0], buf, &mut state.error_dlg);
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
                .styles(ctx.g.theme.statusline_style());
            status.render(status_layout[1], buf, &mut state.status);

            Ok(())
        }
    }

    impl AppState<GlobalState, MinimalEvent, Error> for SceneryState {
        fn init(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            ctx.focus = Some(FocusBuilder::build_for(&self.minimal));
            self.minimal.init(ctx)?;
            Ok(())
        }

        fn event(
            &mut self,
            event: &MinimalEvent,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, MinimalEvent, Error>,
        ) -> Result<Control<MinimalEvent>, Error> {
            let t0 = SystemTime::now();

            let mut r = match event {
                MinimalEvent::Event(event) => {
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
                MinimalEvent::Rendered => {
                    ctx.focus = Some(FocusBuilder::rebuild_for(&self.minimal, ctx.focus.take()));
                    Control::Continue
                }
                MinimalEvent::Message(s) => {
                    self.error_dlg.append(s.as_str());
                    Control::Changed
                }
                MinimalEvent::Status(n, s) => {
                    self.status.status(*n, s);
                    Control::Changed
                }
                _ => Control::Continue,
            };

            r = r.or_else_try(|| self.minimal.event(event, ctx))?;

            let el = t0.elapsed()?;
            self.status.status(2, format!("E {:.0?}", el).to_string());

            Ok(r)
        }

        fn error(
            &self,
            event: Error,
            _ctx: &mut AppContext<'_>,
        ) -> Result<Control<MinimalEvent>, Error> {
            self.error_dlg.append(format!("{:?}", &*event).as_str());
            Ok(Control::Changed)
        }
    }
}

pub mod minimal {
    use crate::{GlobalState, MinimalEvent, RenderContext};
    use anyhow::Error;
    use rat_salsa::{AppState, AppWidget, Control};
    use rat_widget::event::{HandleEvent, MenuOutcome, Regular};
    use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus};
    use rat_widget::menu::{MenuLine, MenuLineState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Layout, Rect};
    use ratatui::widgets::StatefulWidget;

    #[derive(Debug)]
    pub(crate) struct Minimal;

    #[derive(Debug, Default)]
    pub struct MinimalState {
        pub menu: MenuLineState,
    }

    impl AppWidget<GlobalState, MinimalEvent, Error> for Minimal {
        type State = MinimalState;

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
                .item_parsed("_Quit");
            menu.render(r[1], buf, &mut state.menu);

            Ok(())
        }
    }

    impl HasFocus for MinimalState {
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

    impl AppState<GlobalState, MinimalEvent, Error> for MinimalState {
        fn init(
            &mut self,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, MinimalEvent, Error>,
        ) -> Result<(), Error> {
            ctx.focus().first();
            Ok(())
        }

        #[allow(unused_variables)]
        fn event(
            &mut self,
            event: &MinimalEvent,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, MinimalEvent, Error>,
        ) -> Result<Control<MinimalEvent>, Error> {
            let r = match event {
                MinimalEvent::Event(event) => match self.menu.handle(event, Regular) {
                    MenuOutcome::Activated(0) => Control::Quit,
                    v => v.into(),
                },
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
