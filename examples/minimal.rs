#![allow(unused_variables)]

use crate::mask0::{Mask0, Mask0State};
use anyhow::Error;
use crossterm::event::Event;
#[allow(unused_imports)]
use log::debug;
use rat_salsa::timer::TimeOut;
use rat_salsa::{run_tui, AppEvents, AppWidget, Control, RunConfig};
use rat_theme::dark_theme::DarkTheme;
use rat_theme::scheme::IMPERIAL;
use rat_widget::event::{ct_event, flow_ok, Dialog, HandleEvent};
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::statusline::{StatusLine, StatusLineState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::StatefulWidget;
use std::fs;
use std::time::{Duration, SystemTime};

type AppContext<'a> = rat_salsa::AppContext<'a, GlobalState, MinimalAction, Error>;
type RenderContext<'a> = rat_salsa::RenderContext<'a, GlobalState>;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let config = MinimalConfig::default();
    let theme = DarkTheme::new("Imperial".into(), IMPERIAL);
    let mut global = GlobalState::new(config, theme);

    let app = MinimalApp;
    let mut state = MinimalState::default();

    run_tui(
        app,
        &mut global,
        &mut state,
        RunConfig::default()?.threads(1),
    )?;

    Ok(())
}

// -----------------------------------------------------------------------

#[derive(Debug)]
pub struct GlobalState {
    pub cfg: MinimalConfig,
    pub theme: DarkTheme,
    pub status: StatusLineState,
    pub error_dlg: MsgDialogState,
}

impl GlobalState {
    fn new(cfg: MinimalConfig, theme: DarkTheme) -> Self {
        Self {
            cfg,
            theme,
            status: Default::default(),
            error_dlg: Default::default(),
        }
    }
}

// -----------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct MinimalConfig {}

#[derive(Debug)]
pub enum MinimalAction {
    Message(String),
}

// -----------------------------------------------------------------------

#[derive(Debug)]
pub struct MinimalApp;

#[derive(Debug, Default)]
pub struct MinimalState {
    pub mask0: Mask0State,
}

impl AppWidget<GlobalState, MinimalAction, Error> for MinimalApp {
    type State = MinimalState;

    fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut Self::State,
        ctx: &mut RenderContext<'_>,
    ) -> Result<(), Error> {
        let t0 = SystemTime::now();

        let r = Layout::new(
            Direction::Vertical,
            [Constraint::Fill(1), Constraint::Length(1)],
        )
        .split(area);

        Mask0.render(r[0], buf, &mut state.mask0, ctx)?;

        if ctx.g.error_dlg.active() {
            let err = MsgDialog::new().styles(ctx.g.theme.msg_dialog_style());
            err.render(r[0], buf, &mut ctx.g.error_dlg);
        }

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        ctx.g.status.status(1, format!("R {:.3?}", el).to_string());

        let status = StatusLine::new()
            .layout([
                Constraint::Fill(1),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Length(12),
            ])
            .styles(ctx.g.theme.statusline_style());
        status.render(r[1], buf, &mut ctx.g.status);

        Ok(())
    }
}

impl AppEvents<GlobalState, MinimalAction, Error> for MinimalState {
    fn init(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
        Ok(())
    }

    fn timer(
        &mut self,
        event: &TimeOut,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<MinimalAction>, Error> {
        Ok(Control::Continue)
    }

    fn crossterm(
        &mut self,
        event: &Event,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<MinimalAction>, Error> {
        use crossterm::event::*;

        let t0 = SystemTime::now();

        flow_ok!(match &event {
            Event::Resize(_, _) => Control::Changed,
            ct_event!(key press CONTROL-'q') => Control::Quit,
            _ => Control::Continue,
        });

        flow_ok!({
            if ctx.g.error_dlg.active() {
                ctx.g.error_dlg.handle(&event, Dialog).into()
            } else {
                Control::Continue
            }
        });

        flow_ok!(self.mask0.crossterm(&event, ctx)?);

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        ctx.g.status.status(2, format!("H {:.3?}", el).to_string());

        Ok(Control::Continue)
    }

    fn message(
        &mut self,
        event: &mut MinimalAction,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<MinimalAction>, Error> {
        let t0 = SystemTime::now();

        // TODO: actions
        flow_ok!(match event {
            MinimalAction::Message(s) => {
                ctx.g.status.status(0, &*s);
                Control::Changed
            }
        });

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        ctx.g.status.status(3, format!("A {:.3?}", el).to_string());

        Ok(Control::Continue)
    }

    fn error(
        &self,
        event: Error,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<MinimalAction>, Error> {
        ctx.g.error_dlg.append(format!("{:?}", &*event).as_str());
        Ok(Control::Changed)
    }
}

mod mask0 {
    use crate::{AppContext, GlobalState, MinimalAction, RenderContext};
    use anyhow::Error;
    use crossterm::event::Event;
    #[allow(unused_imports)]
    use log::debug;
    use rat_salsa::{AppEvents, AppWidget, Control};
    use rat_widget::event::{flow_ok, HandleEvent, Regular};
    use rat_widget::focus::HasFocusFlag;
    use rat_widget::menuline::{MenuLine, MenuLineState, MenuOutcome};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Layout, Rect};
    use ratatui::widgets::StatefulWidget;

    #[derive(Debug)]
    pub(crate) struct Mask0;

    #[derive(Debug)]
    pub struct Mask0State {
        pub menu: MenuLineState,
    }

    impl Default for Mask0State {
        fn default() -> Self {
            let s = Self {
                menu: Default::default(),
            };
            s.menu.focus().set(true);
            s
        }
    }

    impl AppWidget<GlobalState, MinimalAction, Error> for Mask0 {
        type State = Mask0State;

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
                [Constraint::Fill(1), Constraint::Length(1)],
            )
            .split(area);

            let menu = MenuLine::new()
                .styles(ctx.g.theme.menu_style())
                .add_str("One")
                .add_str("Two")
                .add_str("Three")
                .add_str("_Quit");
            menu.render(r[1], buf, &mut state.menu);

            Ok(())
        }
    }

    impl AppEvents<GlobalState, MinimalAction, Error> for Mask0State {
        fn crossterm(
            &mut self,
            event: &Event,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MinimalAction>, Error> {
            // TODO: handle_mask
            flow_ok!(match self.menu.handle(event, Regular) {
                MenuOutcome::Activated(0) => {
                    _ = ctx.spawn(|cancel, send| {
                        Ok(Control::Message(MinimalAction::Message(
                            "hello from the other side".into(),
                        )))
                    });
                    Control::Unchanged
                }
                MenuOutcome::Activated(1) => {
                    _ = ctx.spawn(|cancel, send| {
                        Ok(Control::Message(MinimalAction::Message(
                            "another background task finished ...".into(),
                        )))
                    });
                    Control::Unchanged
                }
                MenuOutcome::Activated(2) => {
                    Control::Continue
                }
                MenuOutcome::Activated(3) => {
                    Control::Quit
                }
                v => {
                    let w = v.into();
                    w
                }
            });

            Ok(Control::Continue)
        }
    }
}

fn setup_logging() -> Result<(), Error> {
    _ = fs::remove_file("log.log");
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
