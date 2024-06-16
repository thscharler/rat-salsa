#![allow(unused_variables)]

use crate::mask0::{Mask0, Mask0State};
use crate::theme::Theme;
use anyhow::Error;
use crossterm::event::Event;
#[allow(unused_imports)]
use log::debug;
use rat_salsa::timer::TimeOut;
use rat_salsa::{run_tui, AppEvents, AppWidget, Control, RunConfig};
use rat_widget::event::{ct_event, flow_ok, FocusKeys, HandleEvent};
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::statusline::{StatusLine, StatusLineState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::StatefulWidget;
use std::cell::RefCell;
use std::fs;
use std::time::{Duration, SystemTime};

type AppContext<'a> = rat_salsa::AppContext<'a, GlobalState, MinimalAction, Error>;
type RenderContext<'a> = rat_salsa::RenderContext<'a, GlobalState>;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let config = MinimalConfig::default();
    let mut global = GlobalState::new(config, &theme::ONEDARK);

    let app = MinimalApp;
    let mut state = MinimalState::default();

    run_tui(
        app,
        &mut global,
        &mut state,
        RunConfig {
            n_threats: 1,
            ..RunConfig::default()?
        },
    )?;

    Ok(())
}

// -----------------------------------------------------------------------

#[derive(Debug)]
pub struct GlobalState {
    pub cfg: MinimalConfig,
    pub theme: &'static Theme,
    pub status: RefCell<StatusLineState>,
    pub error_dlg: RefCell<MsgDialogState>,
}

impl GlobalState {
    fn new(cfg: MinimalConfig, theme: &'static Theme) -> Self {
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

        if ctx.g.error_dlg.borrow().active {
            let err = MsgDialog::new().styles(ctx.g.theme.status_dialog_style());
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

mod mask0 {
    use crate::{AppContext, GlobalState, MinimalAction, RenderContext};
    use anyhow::Error;
    use crossterm::event::Event;
    #[allow(unused_imports)]
    use log::debug;
    use rat_salsa::{AppEvents, AppWidget, Control};
    use rat_widget::event::{flow_ok, FocusKeys, HandleEvent};
    use rat_widget::focus::HasFocusFlag;
    use rat_widget::menuline::{MenuOutcome, RMenuLine, RMenuLineState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Layout, Rect};
    use ratatui::widgets::StatefulWidget;

    #[derive(Debug)]
    pub(crate) struct Mask0;

    #[derive(Debug)]
    pub struct Mask0State {
        pub menu: RMenuLineState,
    }

    impl Default for Mask0State {
        fn default() -> Self {
            let s = Self {
                menu: Default::default(),
            };
            s.menu.focus().set();
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
                    _ = ctx.spawn(|cancel, send| {
                        Ok(Control::Action(MinimalAction::Message(
                            "hello from the other side".into(),
                        )))
                    });
                    Control::Break
                }
                MenuOutcome::Activated(1) => {
                    _ = ctx.spawn(|cancel, send| {
                        Ok(Control::Action(MinimalAction::Message(
                            "another background task finished ...".into(),
                        )))
                    });
                    Control::Break
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

// -----------------------------------------------------------------------

mod theme {
    use rat_widget::button::ButtonStyle;
    use rat_widget::input::TextInputStyle;
    use rat_widget::masked_input::MaskedInputStyle;
    use rat_widget::menuline::MenuStyle;
    use rat_widget::msgdialog::MsgDialogStyle;
    use ratatui::style::{Color, Style, Stylize};

    #[derive(Debug, Default)]
    pub struct Theme {
        pub name: &'static str,
        pub dark_theme: bool,

        pub white: Color,
        pub darker_black: Color,
        pub black: Color,
        pub black2: Color,
        pub one_bg: Color,
        pub one_bg2: Color,
        pub one_bg3: Color,
        pub grey: Color,
        pub grey_fg: Color,
        pub grey_fg2: Color,
        pub light_grey: Color,
        pub red: Color,
        pub baby_pink: Color,
        pub pink: Color,
        pub line: Color,
        pub green: Color,
        pub vibrant_green: Color,
        pub nord_blue: Color,
        pub blue: Color,
        pub yellow: Color,
        pub sun: Color,
        pub purple: Color,
        pub dark_purple: Color,
        pub teal: Color,
        pub orange: Color,
        pub cyan: Color,
        pub statusline_bg: Color,
        pub lightbg: Color,
        pub pmenu_bg: Color,
        pub folder_bg: Color,

        pub base00: Color,
        pub base01: Color,
        pub base02: Color,
        pub base03: Color,
        pub base04: Color,
        pub base05: Color,
        pub base06: Color,
        pub base07: Color,
        pub base08: Color,
        pub base09: Color,
        pub base0a: Color,
        pub base0b: Color,
        pub base0c: Color,
        pub base0d: Color,
        pub base0e: Color,
        pub base0f: Color,
    }

    impl Theme {
        pub fn status_style(&self) -> Style {
            Style::default().fg(self.white).bg(self.one_bg3)
        }

        pub fn input_style(&self) -> TextInputStyle {
            TextInputStyle {
                style: Style::default().fg(self.black).bg(self.base05),
                focus: Some(Style::default().fg(self.black).bg(self.green)),
                select: Some(Style::default().fg(self.black).bg(self.base0e)),
                ..TextInputStyle::default()
            }
        }

        pub fn input_mask_style(&self) -> MaskedInputStyle {
            MaskedInputStyle {
                style: Style::default().fg(self.black).bg(self.base05),
                focus: Some(Style::default().fg(self.black).bg(self.green)),
                select: Some(Style::default().fg(self.black).bg(self.base0e)),
                invalid: Some(Style::default().bg(Color::Red)),
                ..Default::default()
            }
        }

        pub fn button_style(&self) -> ButtonStyle {
            ButtonStyle {
                style: Style::default().fg(self.black).bg(self.purple).bold(),
                focus: Some(Style::default().fg(self.black).bg(self.green).bold()),
                armed: Some(Style::default().fg(self.black).bg(self.orange).bold()),
                ..Default::default()
            }
        }

        pub fn statusline_style(&self) -> Vec<Style> {
            vec![
                self.status_style(),
                Style::default().white().on_blue(),
                Style::default().white().on_light_blue(),
                Style::default().white().on_gray(),
            ]
        }

        pub fn status_dialog_style(&self) -> MsgDialogStyle {
            MsgDialogStyle {
                style: self.status_style(),
                button: self.button_style(),
                ..Default::default()
            }
        }

        pub fn menu_style(&self) -> MenuStyle {
            MenuStyle {
                style: Style::default().fg(self.white).bg(self.one_bg3).bold(),
                title: Some(Style::default().fg(self.black).bg(self.base0a).bold()),
                select: Some(Style::default().fg(self.black).bg(self.base0e).bold()),
                focus: Some(Style::default().fg(self.black).bg(self.green).bold()),
                ..Default::default()
            }
        }
    }

    pub(crate) static ONEDARK: Theme = Theme {
        name: "onedark",
        dark_theme: false,

        white: Color::from_u32(0xabb2bf),
        darker_black: Color::from_u32(0x1b1f27),
        black: Color::from_u32(0x1e222a), //  nvim bg
        black2: Color::from_u32(0x252931),
        one_bg: Color::from_u32(0x282c34), // real bg of onedark
        one_bg2: Color::from_u32(0x353b45),
        one_bg3: Color::from_u32(0x373b43),
        grey: Color::from_u32(0x42464e),
        grey_fg: Color::from_u32(0x565c64),
        grey_fg2: Color::from_u32(0x6f737b),
        light_grey: Color::from_u32(0x6f737b),
        red: Color::from_u32(0xe06c75),
        baby_pink: Color::from_u32(0xDE8C92),
        pink: Color::from_u32(0xff75a0),
        line: Color::from_u32(0x31353d), // for lines like vertsplit
        green: Color::from_u32(0x98c379),
        vibrant_green: Color::from_u32(0x7eca9c),
        nord_blue: Color::from_u32(0x81A1C1),
        blue: Color::from_u32(0x61afef),
        yellow: Color::from_u32(0xe7c787),
        sun: Color::from_u32(0xEBCB8B),
        purple: Color::from_u32(0xde98fd),
        dark_purple: Color::from_u32(0xc882e7),
        teal: Color::from_u32(0x519ABA),
        orange: Color::from_u32(0xfca2aa),
        cyan: Color::from_u32(0xa3b8ef),
        statusline_bg: Color::from_u32(0x22262e),
        lightbg: Color::from_u32(0x2d3139),
        pmenu_bg: Color::from_u32(0x61afef),
        folder_bg: Color::from_u32(0x61afef),

        base00: Color::from_u32(0x1e222a),
        base01: Color::from_u32(0x353b45),
        base02: Color::from_u32(0x3e4451),
        base03: Color::from_u32(0x545862),
        base04: Color::from_u32(0x565c64),
        base05: Color::from_u32(0xabb2bf),
        base06: Color::from_u32(0xb6bdca),
        base07: Color::from_u32(0xc8ccd4),
        base08: Color::from_u32(0xe06c75),
        base09: Color::from_u32(0xd19a66),
        base0a: Color::from_u32(0xe5c07b),
        base0b: Color::from_u32(0x98c379),
        base0c: Color::from_u32(0x56b6c2),
        base0d: Color::from_u32(0x61afef),
        base0e: Color::from_u32(0xc678dd),
        base0f: Color::from_u32(0xbe5046),
    };
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
