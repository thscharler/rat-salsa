use crate::app::{Scenery, SceneryState};
use crate::config::TurboConfig;
use crate::global::GlobalState;
use crate::message::TurboMsg;

use crate::theme::TurboTheme;
use anyhow::Error;
use rat_salsa::{run_tui, RunConfig};
use rat_theme::scheme::BASE16;
use std::time::SystemTime;

type AppContext<'a> = rat_salsa::AppContext<'a, GlobalState, TurboMsg, Error>;
type RenderContext<'a> = rat_salsa::RenderContext<'a, GlobalState>;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let config = TurboConfig::default();
    let theme = TurboTheme::new("Base16".into(), BASE16);
    let mut global = GlobalState::new(config, theme);

    let app = Scenery;
    let mut state = SceneryState::default();

    run_tui(
        app,
        &mut global,
        &mut state,
        RunConfig::default()?.threads(1),
    )?;

    Ok(())
}

/// Globally accessible data/state.
pub mod global {
    use crate::config::TurboConfig;
    use crate::theme::TurboTheme;
    use rat_widget::msgdialog::MsgDialogState;
    use rat_widget::statusline::StatusLineState;

    #[derive(Debug)]
    pub struct GlobalState {
        pub cfg: TurboConfig,
        pub theme: TurboTheme,
        pub status: StatusLineState,
        pub error_dlg: MsgDialogState,
    }

    impl GlobalState {
        pub fn new(cfg: TurboConfig, theme: TurboTheme) -> Self {
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
    pub struct TurboConfig {}
}

/// Application wide messages.
pub mod message {
    #[derive(Debug)]
    pub enum TurboMsg {
        Message(String),
    }
}

pub mod app {
    use crate::global::GlobalState;
    use crate::message::TurboMsg;
    use crate::turbo::{Turbo, TurboState};
    use crate::{AppContext, RenderContext};
    use anyhow::Error;
    use crossterm::event::Event;
    use rat_salsa::event::ct_event;
    use rat_salsa::timer::TimeOut;
    use rat_salsa::{AppState, AppWidget, Control};
    use rat_widget::event::{ConsumedEvent, Dialog, HandleEvent};
    use rat_widget::focus::{build_focus, rebuild_focus};
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
        pub minimal: TurboState,
    }

    impl AppWidget<GlobalState, TurboMsg, Error> for Scenery {
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
                Constraint::Fill(1),
                Constraint::Length(1),
            ])
            .split(area);

            Turbo.render(area, buf, &mut state.minimal, ctx)?;

            if ctx.g.error_dlg.active() {
                let layout_error = layout_middle(
                    layout[1],
                    Constraint::Percentage(19),
                    Constraint::Percentage(19),
                    Constraint::Length(2),
                    Constraint::Length(2),
                );
                let err = MsgDialog::new().styles(ctx.g.theme.msg_dialog_style());
                err.render(layout_error, buf, &mut ctx.g.error_dlg);
            }

            let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
            ctx.g.status.status(1, format!("R {:.0?}", el).to_string());

            let status = StatusLine::new()
                .layout([
                    Constraint::Fill(1),
                    Constraint::Length(8),
                    Constraint::Length(8),
                ])
                .styles(ctx.g.theme.statusline_style());
            status.render(layout[2], buf, &mut ctx.g.status);

            Ok(())
        }
    }

    /// Calculate the middle Rect inside a given area.
    fn layout_middle(
        area: Rect,
        left: Constraint,
        right: Constraint,
        top: Constraint,
        bottom: Constraint,
    ) -> Rect {
        let h_layout = Layout::horizontal([
            left, //
            Constraint::Fill(1),
            right,
        ])
        .split(area);
        let v_layout = Layout::vertical([
            top, //
            Constraint::Fill(1),
            bottom,
        ])
        .split(h_layout[1]);
        v_layout[1]
    }

    impl AppState<GlobalState, TurboMsg, Error> for SceneryState {
        fn init(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            ctx.focus = Some(build_focus(&self.minimal));
            self.minimal.init(ctx)?;
            ctx.g.status.status(0, "Ctrl-Q to quit.");
            Ok(())
        }

        fn timer(
            &mut self,
            event: &TimeOut,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<TurboMsg>, Error> {
            let t0 = SystemTime::now();

            ctx.focus = Some(rebuild_focus(&self.minimal, ctx.focus.take()));
            let r = self.minimal.timer(event, ctx)?;

            let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
            ctx.g.status.status(2, format!("T {:.0?}", el).to_string());

            Ok(r)
        }

        fn crossterm(
            &mut self,
            event: &Event,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<TurboMsg>, Error> {
            let t0 = SystemTime::now();

            let mut r = match &event {
                ct_event!(resized) => Control::Changed,
                ct_event!(key press CONTROL-'q') => Control::Quit,
                _ => Control::Continue,
            };

            r = r.or_else(|| {
                if ctx.g.error_dlg.active() {
                    ctx.g.error_dlg.handle(&event, Dialog).into()
                } else {
                    Control::Continue
                }
            });

            r = r.or_else_try(|| {
                ctx.focus = Some(rebuild_focus(&self.minimal, ctx.focus.take()));
                self.minimal.crossterm(&event, ctx)
            })?;

            let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
            ctx.g.status.status(2, format!("H {:.0?}", el).to_string());

            Ok(r)
        }

        fn message(
            &mut self,
            event: &mut TurboMsg,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<TurboMsg>, Error> {
            let t0 = SystemTime::now();

            #[allow(unreachable_patterns)]
            let r = match event {
                TurboMsg::Message(s) => {
                    ctx.g.status.status(0, &*s);
                    Control::Changed
                }
                _ => {
                    ctx.focus = Some(rebuild_focus(&self.minimal, ctx.focus.take()));
                    self.minimal.message(event, ctx)?
                }
            };

            let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
            ctx.g.status.status(2, format!("A {:.0?}", el).to_string());

            Ok(r)
        }

        fn error(
            &self,
            event: Error,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<TurboMsg>, Error> {
            ctx.g.error_dlg.append(format!("{:?}", &*event).as_str());
            Ok(Control::Changed)
        }
    }
}

pub mod turbo {
    use crate::{AppContext, GlobalState, RenderContext, TurboMsg};
    use anyhow::Error;
    use crossterm::event::Event;
    use rat_salsa::{AppState, AppWidget, Control};
    use rat_widget::event::{try_flow, HandleEvent, MenuOutcome, Popup, Regular};
    use rat_widget::focus::{FocusBuilder, HasFocus};
    use rat_widget::menu::{MenuBarState, Menubar, Placement, StaticMenu};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Layout, Rect};
    use ratatui::widgets::{Block, StatefulWidget};

    #[derive(Debug)]
    pub(crate) struct Turbo;

    #[derive(Debug)]
    pub struct TurboState {
        pub menu: MenuBarState,
    }

    static MENU: StaticMenu = StaticMenu {
        menu: &[
            ("_File", &["_Exit"]),
            ("_Edit", &[]),
            ("_Search", &[]),
            (
                "_Run",
                &[
                    "_Run",
                    "_Step over",
                    "_Trace into",
                    "_Goto cursor",
                    "_Program reset",
                    "P_arameters...",
                ],
            ),
            ("_Compile", &[]),
            ("_Debug", &[]),
            (
                "_Tools",
                &[
                    "_Messages",
                    "Go to _next|Alt+F8",
                    "Go to _previous|Alt+F7",
                    "____",
                    "_Grep",
                    "Clear/Refresh screen DOS",
                    "About TPWDB",
                ],
            ),
            ("_Options", &[]),
            ("_Window", &[]),
            ("_Help", &[]),
        ],
    };

    impl Default for TurboState {
        fn default() -> Self {
            let mut s = Self {
                menu: Default::default(),
            };
            s.menu.bar.select(Some(0));
            s
        }
    }

    impl AppWidget<GlobalState, TurboMsg, Error> for Turbo {
        type State = TurboState;

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
                    Constraint::Length(1),
                    Constraint::Fill(1),
                    Constraint::Length(1),
                ],
            )
            .split(area);

            let (menubar, popup) = Menubar::new(&MENU)
                .styles(ctx.g.theme.menu_style())
                .title("  ")
                .popup_placement(Placement::Bottom)
                .popup_block(Block::bordered())
                .into_widgets();
            menubar.render(r[0], buf, &mut state.menu);
            popup.render(r[0], buf, &mut state.menu);

            Ok(())
        }
    }

    impl HasFocus for TurboState {
        fn build(&self, builder: &mut FocusBuilder) {
            builder.widget(&self.menu);
        }
    }

    impl AppState<GlobalState, TurboMsg, Error> for TurboState {
        fn init(
            &mut self,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, TurboMsg, Error>,
        ) -> Result<(), Error> {
            ctx.focus().first();
            Ok(())
        }

        #[allow(unused_variables)]
        fn crossterm(
            &mut self,
            event: &Event,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<TurboMsg>, Error> {
            // TODO: handle_mask

            let f = ctx.focus_mut().handle(event, Regular);
            ctx.queue(f);

            try_flow!(match self.menu.handle(event, Popup) {
                MenuOutcome::MenuActivated(0, 0) => {
                    Control::Quit
                }
                MenuOutcome::MenuActivated(6, 0) => {
                    for i in 0..50 {
                        ctx.g.error_dlg.append("Hello!");
                    }
                    Control::Changed
                }
                v => v.into(),
            });
            try_flow!(self.menu.handle(event, Regular));

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

#[allow(dead_code)]
pub mod theme {
    use rat_theme::Scheme;
    use rat_widget::button::ButtonStyle;
    use rat_widget::file_dialog::FileDialogStyle;
    use rat_widget::line_number::LineNumberStyle;
    use rat_widget::list::ListStyle;
    use rat_widget::menu::MenuStyle;
    use rat_widget::msgdialog::MsgDialogStyle;
    use rat_widget::scrolled::{ScrollStyle, ScrollSymbols};
    use rat_widget::splitter::SplitStyle;
    use rat_widget::tabbed::TabbedStyle;
    use rat_widget::table::TableStyle;
    use rat_widget::text_input::TextInputStyle;
    use rat_widget::textarea::TextAreaStyle;
    use ratatui::prelude::Style;
    use ratatui::style::Stylize;

    #[derive(Debug, Clone)]
    pub struct TurboTheme {
        s: Scheme,
        name: String,
    }

    impl TurboTheme {
        pub fn new(name: String, s: Scheme) -> Self {
            Self { s, name }
        }
    }

    impl TurboTheme {
        /// Some display name.
        pub fn name(&self) -> &str {
            &self.name
        }

        /// Hint at dark.
        pub fn dark_theme(&self) -> bool {
            false
        }

        /// The underlying scheme.
        pub fn scheme(&self) -> &Scheme {
            &self.s
        }

        /// Create a style from the given white shade.
        /// n is `0..=3`
        pub fn white(&self, n: usize) -> Style {
            self.s.style(self.s.white[n])
        }

        /// Create a style from the given black shade.
        /// n is `0..=3`
        pub fn black(&self, n: usize) -> Style {
            self.s.style(self.s.black[n])
        }

        /// Create a style from the given gray shade.
        /// n is `0..=3`
        pub fn gray(&self, n: usize) -> Style {
            self.s.style(self.s.gray[n])
        }

        /// Create a style from the given red shade.
        /// n is `0..=3`
        pub fn red(&self, n: usize) -> Style {
            self.s.style(self.s.red[n])
        }

        /// Create a style from the given orange shade.
        /// n is `0..=3`
        pub fn orange(&self, n: usize) -> Style {
            self.s.style(self.s.orange[n])
        }

        /// Create a style from the given yellow shade.
        /// n is `0..=3`
        pub fn yellow(&self, n: usize) -> Style {
            self.s.style(self.s.yellow[n])
        }

        /// Create a style from the given limegreen shade.
        /// n is `0..=3`
        pub fn limegreen(&self, n: usize) -> Style {
            self.s.style(self.s.limegreen[n])
        }

        /// Create a style from the given green shade.
        /// n is `0..=3`
        pub fn green(&self, n: usize) -> Style {
            self.s.style(self.s.green[n])
        }

        /// Create a style from the given bluegreen shade.
        /// n is `0..=3`
        pub fn bluegreen(&self, n: usize) -> Style {
            self.s.style(self.s.bluegreen[n])
        }

        /// Create a style from the given cyan shade.
        /// n is `0..=3`
        pub fn cyan(&self, n: usize) -> Style {
            self.s.style(self.s.cyan[n])
        }

        /// Create a style from the given blue shade.
        /// n is `0..=3`
        pub fn blue(&self, n: usize) -> Style {
            self.s.style(self.s.blue[n])
        }

        /// Create a style from the given deepblue shade.
        /// n is `0..=3`
        pub fn deepblue(&self, n: usize) -> Style {
            self.s.style(self.s.deepblue[n])
        }

        /// Create a style from the given purple shade.
        /// n is `0..=3`
        pub fn purple(&self, n: usize) -> Style {
            self.s.style(self.s.purple[n])
        }

        /// Create a style from the given magenta shade.
        /// n is `0..=3`
        pub fn magenta(&self, n: usize) -> Style {
            self.s.style(self.s.magenta[n])
        }

        /// Create a style from the given redpink shade.
        /// n is `0..=3`
        pub fn redpink(&self, n: usize) -> Style {
            self.s.style(self.s.redpink[n])
        }

        /// Create a style from the given primary shade.
        /// n is `0..=3`
        pub fn primary(&self, n: usize) -> Style {
            self.s.style(self.s.primary[n])
        }

        /// Create a style from the given secondary shade.
        /// n is `0..=3`
        pub fn secondary(&self, n: usize) -> Style {
            self.s.style(self.s.secondary[n])
        }

        /// Focus style
        pub fn focus(&self) -> Style {
            let fg = self.s.black[0];
            let bg = self.s.primary[2];
            Style::default().fg(fg).bg(bg)
        }

        /// Selection style
        pub fn select(&self) -> Style {
            let fg = self.s.black[0];
            let bg = self.s.secondary[1];
            Style::default().fg(fg).bg(bg)
        }

        /// Text field style.
        pub fn text_input(&self) -> Style {
            Style::default().fg(self.s.black[0]).bg(self.s.gray[3])
        }

        /// Focused text field style.
        pub fn text_focus(&self) -> Style {
            let fg = self.s.black[0];
            let bg = self.s.primary[0];
            Style::default().fg(fg).bg(bg)
        }

        /// Text selection style.
        pub fn text_select(&self) -> Style {
            let fg = self.s.black[0];
            let bg = self.s.secondary[0];
            Style::default().fg(fg).bg(bg)
        }

        /// Data display style. Used for lists, tables, ...
        pub fn data(&self) -> Style {
            Style::default().fg(self.s.black[0]).bg(self.s.gray[3])
        }

        /// Background for dialogs.
        pub fn dialog_style(&self) -> Style {
            Style::default().fg(self.s.black[0]).bg(self.s.gray[3])
        }

        /// Style for the status line.
        pub fn status_style(&self) -> Style {
            Style::default().fg(self.s.black[0]).bg(self.s.gray[3])
        }

        /// Style for LineNumbers.
        pub fn line_nr_style(&self) -> LineNumberStyle {
            LineNumberStyle {
                style: self.data().fg(self.s.gray[0]),
                cursor_style: Some(self.text_select()),
                ..LineNumberStyle::default()
            }
        }

        /// Complete TextAreaStyle
        pub fn textarea_style(&self) -> TextAreaStyle {
            TextAreaStyle {
                style: self.data(),
                focus: Some(self.focus()),
                select: Some(self.text_select()),
                ..TextAreaStyle::default()
            }
        }

        /// Complete TextInputStyle
        pub fn input_style(&self) -> TextInputStyle {
            TextInputStyle {
                style: self.text_input(),
                focus: Some(self.text_focus()),
                select: Some(self.text_select()),
                invalid: Some(Style::default().bg(self.s.red[3])),
                ..TextInputStyle::default()
            }
        }

        /// Complete MenuStyle
        pub fn menu_style(&self) -> MenuStyle {
            let menu = Style::default().fg(self.s.black[0]).bg(self.s.gray[3]);
            MenuStyle {
                style: menu,
                title: Some(Style::default().fg(self.s.black[0]).bg(self.s.gray[3])),
                select: Some(self.select()),
                focus: Some(self.focus()),
                highlight: Some(Style::default().fg(self.s.red[2])),
                disabled: Some(Style::default().fg(self.s.black[3])),
                right: Some(Style::default().italic()),
                ..Default::default()
            }
        }

        /// Complete FTableStyle
        pub fn table_style(&self) -> TableStyle {
            TableStyle {
                style: self.data(),
                select_row_style: Some(self.select()),
                show_row_focus: true,
                focus_style: Some(self.focus()),
                ..Default::default()
            }
        }

        /// Complete ListStyle
        pub fn list_style(&self) -> ListStyle {
            ListStyle {
                style: self.data(),
                select_style: Some(self.select()),
                focus_style: Some(self.focus()),
                ..Default::default()
            }
        }

        /// Complete ButtonStyle
        pub fn button_style(&self) -> ButtonStyle {
            ButtonStyle {
                style: Style::default()
                    .fg(self.s.text_color(self.s.primary[0]))
                    .bg(self.s.primary[0]),
                focus: Some(
                    Style::default()
                        .fg(self.s.text_color(self.s.primary[3]))
                        .bg(self.s.primary[3]),
                ),
                armed: Some(Style::default().fg(self.s.black[0]).bg(self.s.secondary[0])),
                ..Default::default()
            }
        }

        /// Complete ScrolledStyle
        pub fn scroll_style(&self) -> ScrollStyle {
            let style = Style::default().fg(self.s.black[3]);
            let arrow_style = Style::default().fg(self.s.black[3]).bg(self.s.secondary[0]);
            ScrollStyle {
                thumb_style: Some(style),
                track_style: Some(style),
                min_style: Some(style),
                begin_style: Some(arrow_style),
                end_style: Some(arrow_style),
                vertical: Some(ScrollSymbols {
                    track: "\u{2592}",
                    thumb: "█",
                    begin: "▲",
                    end: "▼",
                    min: "\u{2591}",
                }),
                horizontal: Some(ScrollSymbols {
                    track: "\u{2592}",
                    thumb: "█",
                    begin: "◄",
                    end: "►",
                    min: "\u{2591}",
                }),
                ..Default::default()
            }
        }

        /// Complete Split style
        pub fn split_style(&self) -> SplitStyle {
            let style = Style::default().fg(self.s.gray[0]).bg(self.s.black[1]);
            let arrow_style = Style::default().fg(self.s.secondary[0]).bg(self.s.black[1]);
            SplitStyle {
                style,
                arrow_style: Some(arrow_style),
                drag_style: Some(self.focus()),
                ..Default::default()
            }
        }

        /// Complete Tabbed style
        pub fn tabbed_style(&self) -> TabbedStyle {
            let style = Style::default().fg(self.s.gray[0]).bg(self.s.black[1]);
            TabbedStyle {
                style,
                tab_style: Some(self.gray(1)),
                select_style: Some(self.gray(3)),
                focus_style: Some(self.focus()),
                ..Default::default()
            }
        }

        /// Complete StatusLineStyle for a StatusLine with 3 indicator fields.
        /// This is what I need for the
        /// [minimal](https://github.com/thscharler/rat-salsa/blob/master/examples/minimal.rs)
        /// example, which shows timings for Render/Event/Action.
        pub fn statusline_style(&self) -> Vec<Style> {
            let s = &self.s;
            vec![
                self.status_style(),
                Style::default().fg(s.text_color(s.white[0])).bg(s.blue[3]),
                Style::default().fg(s.text_color(s.white[0])).bg(s.blue[2]),
                Style::default().fg(s.text_color(s.white[0])).bg(s.blue[1]),
            ]
        }

        /// Complete MsgDialogStyle.
        pub fn msg_dialog_style(&self) -> MsgDialogStyle {
            MsgDialogStyle {
                style: self.dialog_style(),
                button: self.button_style(),
                scroll: Some(self.scroll_style()),
                ..Default::default()
            }
        }

        pub fn file_dialog_style(&self) -> FileDialogStyle {
            FileDialogStyle {
                style: self.dialog_style(),
                list: Some(self.data()),
                path: Some(self.text_input()),
                name: Some(self.text_input()),
                new: Some(self.text_input()),
                invalid: Some(Style::new().fg(self.s.red[3]).bg(self.s.gray[2])),
                select: Some(self.select()),
                focus: Some(self.focus()),
                button: Some(self.button_style()),
                ..Default::default()
            }
        }
    }
}
