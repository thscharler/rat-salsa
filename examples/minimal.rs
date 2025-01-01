use crate::config::MinimalConfig;
use crate::event::MinimalEvent;
use crate::global::GlobalState;
use crate::scenery::{Scenery, SceneryState};

use anyhow::Error;
use rat_salsa::poll::{PollCrossterm, PollTasks, PollTimers};
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
            .poll(PollTimers)
            .poll(PollTasks),
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
    use crossterm::event::Event;
    use rat_salsa::timer::TimeOut;

    #[derive(Debug)]
    pub enum MinimalEvent {
        Timer(TimeOut),
        Event(crossterm::event::Event),
        Message(String),
    }

    impl From<TimeOut> for MinimalEvent {
        fn from(value: TimeOut) -> Self {
            Self::Timer(value)
        }
    }

    impl From<crossterm::event::Event> for MinimalEvent {
        fn from(value: Event) -> Self {
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
        pub minimal: MinimalState,
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

            if ctx.g.error_dlg.active() {
                let err = MsgDialog::new().styles(ctx.g.theme.msg_dialog_style());
                err.render(layout[0], buf, &mut ctx.g.error_dlg);
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

    impl AppState<GlobalState, MinimalEvent, Error> for SceneryState {
        fn init(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            ctx.focus = Some(FocusBuilder::for_container(&self.minimal));
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
                        if ctx.g.error_dlg.active() {
                            ctx.g.error_dlg.handle(event, Dialog).into()
                        } else {
                            Control::Continue
                        }
                    });

                    r
                }
                MinimalEvent::Message(s) => {
                    ctx.g.status.status(0, &*s);
                    Control::Changed
                }
                _ => Control::Continue,
            };

            // rebuild and handle focus for each event
            r = r.or_else(|| {
                ctx.focus = Some(FocusBuilder::rebuild(&self.minimal, ctx.focus.take()));
                if let MinimalEvent::Event(event) = event {
                    let f = ctx.focus_mut().handle(event, Regular);
                    ctx.queue(f);
                }
                Control::Continue
            });

            r = r.or_else_try(|| self.minimal.event(event, ctx))?;

            let el = t0.elapsed()?;
            ctx.g.status.status(2, format!("E {:.0?}", el).to_string());

            Ok(r)
        }

        fn error(
            &self,
            event: Error,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MinimalEvent>, Error> {
            ctx.g.error_dlg.append(format!("{:?}", &*event).as_str());
            Ok(Control::Changed)
        }
    }
}

pub mod minimal {
    use crate::{GlobalState, MinimalEvent, RenderContext};
    use anyhow::Error;
    use rat_salsa::{AppState, AppWidget, Control};
    use rat_widget::event::{try_flow, HandleEvent, MenuOutcome, Regular};
    use rat_widget::focus::{FocusBuilder, FocusContainer};
    use rat_widget::menu::{MenuLine, MenuLineState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Layout, Rect};
    use ratatui::widgets::StatefulWidget;

    #[derive(Debug)]
    pub(crate) struct Minimal;

    #[derive(Debug)]
    pub struct MinimalState {
        pub menu: MenuLineState,
    }

    impl Default for MinimalState {
        fn default() -> Self {
            let mut s = Self {
                menu: Default::default(),
            };
            s.menu.select(Some(0));
            s
        }
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

    impl FocusContainer for MinimalState {
        fn build(&self, builder: &mut FocusBuilder) {
            builder.widget(&self.menu);
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
            let MinimalEvent::Event(event) = event else {
                return Ok(Control::Continue);
            };

            try_flow!(match self.menu.handle(event, Regular) {
                MenuOutcome::Activated(0) => {
                    Control::Quit
                }
                v => v.into(),
            });

            Ok(Control::Continue)
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
