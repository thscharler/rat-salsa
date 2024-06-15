#![allow(unused_variables)]

use crate::mask0::{Mask0, Mask0State};
use anyhow::Error;
use crossterm::event::Event;
#[allow(unused_imports)]
use log::debug;
use rat_salsa::event::RepaintEvent;
use rat_salsa::{run_tui, AppEvents, AppWidget, Control, RunConfig, TimeOut};
use rat_theme::dark_theme::DarkTheme;
use rat_theme::imperial::IMPERIAL;
use rat_widget::event::{ct_event, flow_ok, FocusKeys, HandleEvent};
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::statusline::{StatusLine, StatusLineState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::StatefulWidget;
use std::cell::RefCell;
use std::fmt::Debug;
use std::fs;
use std::time::{Duration, SystemTime};

type AppContext<'a> = rat_salsa::AppContext<'a, GlobalState, MinimalAction, Error>;
type RenderContext<'a> = rat_salsa::RenderContext<'a, GlobalState, MinimalAction, Error>;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let config = MinimalConfig::default();
    let theme = DarkTheme::new("ImperialDark".into(), IMPERIAL);
    let mut global = GlobalState::new(config, theme);

    let app = MinimalApp;
    let mut state = MinimalState::default();

    run_tui(
        app,
        &mut global,
        &mut state,
        RunConfig {
            n_threats: 1,
            ..RunConfig::default()
        },
    )?;

    Ok(())
}

// -----------------------------------------------------------------------

#[derive(Debug)]
pub struct GlobalState {
    pub cfg: MinimalConfig,
    pub theme: DarkTheme,
    pub status: RefCell<StatusLineState>,
    pub error_dlg: RefCell<MsgDialogState>,
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
struct MinimalApp;

#[derive(Debug, Default)]
struct MinimalState {
    mask0: Mask0State,
}

impl AppWidget<GlobalState, MinimalAction, Error> for MinimalApp {
    type State = MinimalState;

    fn render(
        &mut self,
        event: &RepaintEvent,
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

        Mask0.render(event, r[0], buf, &mut state.mask0, ctx)?;

        if ctx.g.error_dlg.borrow().active {
            let err = MsgDialog::new().styles(ctx.g.theme.msg_dialog_style());
            err.render(r[0], buf, &mut ctx.g.error_dlg.borrow_mut());
        }

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        ctx.g
            .status
            .borrow_mut()
            .status(1, format!("R {:.3?}", el).to_string());

        let status = StatusLine::new()
            .layout([
                Constraint::Fill(1),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Length(12),
            ])
            .styles(ctx.g.theme.statusline_style());
        status.render(r[1], buf, &mut ctx.g.status.borrow_mut());

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
            Event::Resize(_, _) => Control::Repaint,
            ct_event!(key press CONTROL-'q') => Control::Quit,
            _ => Control::Continue,
        });

        flow_ok!({
            if ctx.g.error_dlg.borrow().active {
                ctx.g
                    .error_dlg
                    .borrow_mut()
                    .handle(&event, FocusKeys)
                    .into()
            } else {
                Control::Continue
            }
        });

        flow_ok!(self.mask0.crossterm(&event, ctx)?);

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        ctx.g
            .status
            .borrow_mut()
            .status(2, format!("H {:.3?}", el).to_string());

        Ok(Control::Continue)
    }

    fn action(
        &mut self,
        event: &mut MinimalAction,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<MinimalAction>, Error> {
        let t0 = SystemTime::now();

        // TODO: actions
        flow_ok!(match event {
            MinimalAction::Message(s) => {
                ctx.g.status.borrow_mut().status(0, &*s);
                Control::Repaint
            }
        });

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        ctx.g
            .status
            .borrow_mut()
            .status(3, format!("A {:.3?}", el).to_string());

        Ok(Control::Continue)
    }

    fn error(
        &self,
        event: Error,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<MinimalAction>, Error> {
        ctx.g
            .error_dlg
            .borrow_mut()
            .append(format!("{:?}", &*event).as_str());
        Ok(Control::Repaint)
    }
}

pub mod mask0 {
    use crate::show_scheme::{ShowScheme, ShowSchemeState};
    use crate::{AppContext, GlobalState, MinimalAction, RenderContext};
    use anyhow::Error;
    use crossterm::event::Event;
    #[allow(unused_imports)]
    use log::debug;
    use rat_salsa::event::RepaintEvent;
    use rat_salsa::{AppEvents, AppWidget, Control};
    use rat_theme::dark_theme::DarkTheme;
    use rat_theme::imperial::IMPERIAL;
    use rat_theme::radium::RADIUM;
    use rat_widget::event::{flow_ok, FocusKeys, HandleEvent};
    use rat_widget::menuline::{MenuOutcome, RMenuLine, RMenuLineState};
    use rat_widget::scrolled::{Scrolled, ScrolledState, ViewportState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Layout, Rect, Size};
    use ratatui::widgets::StatefulWidget;

    #[derive(Debug)]
    pub struct Mask0;

    #[derive(Debug)]
    pub struct Mask0State {
        pub menu: RMenuLineState,
        pub scroll: ScrolledState<ViewportState<ShowSchemeState>>,
        pub theme: u8,
    }

    impl Default for Mask0State {
        fn default() -> Self {
            let s = Self {
                menu: Default::default(),
                scroll: Default::default(),
                theme: 0,
            };
            s.menu.widget.focus.set();
            s
        }
    }

    impl AppWidget<GlobalState, MinimalAction, Error> for Mask0 {
        type State = Mask0State;

        fn render(
            &mut self,
            event: &RepaintEvent,
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

            Scrolled::new_viewport(ShowScheme::new(ctx.g.theme.scheme()))
                .styles(ctx.g.theme.scrolled_style())
                .view_size(Size::new(area.width - 4, 40))
                .render(r[0], buf, &mut state.scroll);

            let menu = RMenuLine::new()
                .styles(ctx.g.theme.menu_style())
                .add("One")
                .add("Two")
                .add("Three")
                .add("_Quit");
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
            flow_ok!(match self.menu.handle(event, FocusKeys) {
                MenuOutcome::Activated(0) => {
                    _ = ctx.spawn(Box::new(|cancel, send| {
                        Ok(Control::Action(MinimalAction::Message(
                            "hello from the other side".into(),
                        )))
                    }));
                    Control::Break
                }
                MenuOutcome::Activated(1) => {
                    self.theme = (self.theme + 1) % 2;
                    match self.theme {
                        0 => ctx.g.theme = DarkTheme::new("ImperialDark".into(), IMPERIAL),
                        1 => ctx.g.theme = DarkTheme::new("RadiumDark".into(), RADIUM),
                        _ => {}
                    }
                    Control::Repaint
                }
                MenuOutcome::Activated(3) => {
                    Control::Quit
                }
                v => {
                    let w = v.into();
                    w
                }
            });

            flow_ok!(self.scroll.handle(event, FocusKeys));

            Ok(Control::Continue)
        }
    }
}

// -----------------------------------------------------------------------

pub mod show_scheme {
    use rat_theme::Scheme;
    use rat_widget::event::{FocusKeys, HandleEvent, MouseOnly, Outcome};
    use rat_widget::focus::{FocusFlag, HasFocusFlag};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Layout, Rect};
    use ratatui::prelude::{Line, Span, StatefulWidget};
    use ratatui::style::Stylize;
    use ratatui::widgets::Widget;

    #[derive(Debug)]
    pub struct ShowScheme<'a> {
        scheme: &'a Scheme,
    }

    #[derive(Debug, Default)]
    pub struct ShowSchemeState {
        pub focus: FocusFlag,
        pub area: Rect,
    }

    impl HasFocusFlag for ShowSchemeState {
        fn focus(&self) -> &FocusFlag {
            &self.focus
        }

        fn area(&self) -> Rect {
            self.area
        }
    }

    impl<'a> ShowScheme<'a> {
        pub fn new(scheme: &'a Scheme) -> Self {
            Self { scheme }
        }
    }

    impl<'a> StatefulWidget for ShowScheme<'a> {
        type State = ShowSchemeState;

        fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
            state.area = area;

            let l0 = Layout::new(
                Direction::Horizontal,
                [
                    Constraint::Fill(1),
                    Constraint::Length(66),
                    Constraint::Fill(1),
                ],
            )
            .split(area);

            let l1 = Layout::new(
                Direction::Vertical,
                [
                    Constraint::Fill(1),
                    Constraint::Length(2),
                    Constraint::Length(2),
                    Constraint::Length(2),
                    Constraint::Length(2),
                    Constraint::Length(2),
                    Constraint::Length(2),
                    Constraint::Length(2),
                    Constraint::Length(2),
                    Constraint::Length(2),
                    Constraint::Length(2),
                    Constraint::Length(2),
                    Constraint::Length(2),
                    Constraint::Length(2),
                    Constraint::Length(2),
                    Constraint::Length(2),
                    Constraint::Length(2),
                    Constraint::Length(2),
                    Constraint::Fill(1),
                ],
            )
            .split(l0[1]);

            "Theme\ncolors".render(l1[0], buf);

            let sc = self.scheme;
            for (i, (n, c)) in [
                ("primary", sc.primary),
                ("sec\nondary", sc.secondary),
                ("white", sc.white),
                ("black", sc.black),
                ("gray", sc.gray),
                ("red", sc.red),
                ("orange", sc.orange),
                ("yellow", sc.yellow),
                ("limegreen", sc.limegreen),
                ("green", sc.green),
                ("bluegreen", sc.bluegreen),
                ("cyan", sc.cyan),
                ("blue", sc.blue),
                ("deepblue", sc.deepblue),
                ("purple", sc.purple),
                ("magenta", sc.magenta),
                ("redpink", sc.redpink),
            ]
            .iter()
            .enumerate()
            {
                Line::from(vec![
                    Span::from(format!("{:10}", n)),
                    Span::from("     DARK     ")
                        .bg(c[0])
                        .fg(sc.text_color(c[0])),
                    Span::from("     MID1     ")
                        .bg(c[1])
                        .fg(sc.text_color(c[1])),
                    Span::from("     MID2     ")
                        .bg(c[2])
                        .fg(sc.text_color(c[2])),
                    Span::from("     LITE     ")
                        .bg(c[3])
                        .fg(sc.text_color(c[3])),
                ])
                .render(l1[i + 1], buf);
            }
        }
    }

    impl HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for ShowSchemeState {
        fn handle(&mut self, event: &crossterm::event::Event, qualifier: FocusKeys) -> Outcome {
            Outcome::NotUsed
        }
    }

    impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for ShowSchemeState {
        fn handle(&mut self, event: &crossterm::event::Event, qualifier: MouseOnly) -> Outcome {
            Outcome::NotUsed
        }
    }
}

// -----------------------------------------------------------------------

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
