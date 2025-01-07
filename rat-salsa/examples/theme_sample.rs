#![allow(unused_variables)]

use crate::mask0::{Mask0, Mask0State};
use anyhow::Error;
use crossterm::event::Event;
use rat_salsa::poll::{PollCrossterm, PollTasks, PollTimers};
use rat_salsa::timer::TimeOut;
use rat_salsa::{run_tui, AppState, AppWidget, Control, RunConfig};
use rat_theme::dark_theme::DarkTheme;
use rat_theme::scheme::IMPERIAL;
use rat_widget::event::{ct_event, try_flow, Dialog, HandleEvent};
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::statusline::{StatusLine, StatusLineState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::StatefulWidget;
use std::fmt::Debug;
use std::fs;
use std::time::{Duration, SystemTime};

type AppContext<'a> = rat_salsa::AppContext<'a, GlobalState, ThemeEvent, Error>;
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
        RunConfig::default()?
            .threads(1)
            .poll(PollCrossterm)
            .poll(PollTimers)
            .poll(PollTasks),
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
pub enum ThemeEvent {
    Event(crossterm::event::Event),
    TimeOut(TimeOut),
    Message(String),
}

impl From<crossterm::event::Event> for ThemeEvent {
    fn from(value: Event) -> Self {
        Self::Event(value)
    }
}

impl From<TimeOut> for ThemeEvent {
    fn from(value: TimeOut) -> Self {
        Self::TimeOut(value)
    }
}

// -----------------------------------------------------------------------

#[derive(Debug)]
struct MinimalApp;

#[derive(Debug, Default)]
struct MinimalState {
    mask0: Mask0State,
}

impl AppWidget<GlobalState, ThemeEvent, Error> for MinimalApp {
    type State = MinimalState;

    fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut Self::State,
        ctx: &mut RenderContext<'_>,
    ) -> Result<(), Error> {
        let t0 = SystemTime::now();

        let layout = Layout::new(
            Direction::Vertical,
            [Constraint::Fill(1), Constraint::Length(1)],
        )
        .split(area);

        Mask0.render(area, buf, &mut state.mask0, ctx)?;

        if ctx.g.error_dlg.active() {
            let err = MsgDialog::new().styles(ctx.g.theme.msg_dialog_style());
            err.render(layout[0], buf, &mut ctx.g.error_dlg);
        }

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        ctx.g.status.status(1, format!("R {:.3?}", el).to_string());

        let layout_status =
            Layout::horizontal([Constraint::Percentage(61), Constraint::Percentage(39)])
                .split(layout[1]);
        let status = StatusLine::new()
            .layout([
                Constraint::Fill(1),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Length(12),
            ])
            .styles(ctx.g.theme.statusline_style());
        status.render(layout_status[1], buf, &mut ctx.g.status);

        Ok(())
    }
}

impl AppState<GlobalState, ThemeEvent, Error> for MinimalState {
    fn init(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
        Ok(())
    }

    fn event(
        &mut self,
        event: &ThemeEvent,
        ctx: &mut rat_salsa::AppContext<'_, GlobalState, ThemeEvent, Error>,
    ) -> Result<Control<ThemeEvent>, Error> {
        let t0 = SystemTime::now();

        let r = match event {
            ThemeEvent::Event(event) => {
                try_flow!(match &event {
                    Event::Resize(_, _) => Control::Changed,
                    ct_event!(key press CONTROL-'q') => Control::Quit,
                    _ => Control::Continue,
                });

                try_flow!({
                    if ctx.g.error_dlg.active() {
                        ctx.g.error_dlg.handle(&event, Dialog).into()
                    } else {
                        Control::Continue
                    }
                });

                Control::Continue
            }
            ThemeEvent::Message(s) => {
                ctx.g.status.status(0, &*s);
                Control::Changed
            }
            _ => Control::Continue,
        };

        try_flow!(self.mask0.event(&event, ctx)?);

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        ctx.g.status.status(3, format!("H {:.3?}", el).to_string());

        Ok(r)
    }

    fn error(&self, event: Error, ctx: &mut AppContext<'_>) -> Result<Control<ThemeEvent>, Error> {
        ctx.g.error_dlg.append(format!("{:?}", &*event).as_str());
        Ok(Control::Changed)
    }
}

pub mod mask0 {
    use crate::show_scheme::{ShowScheme, ShowSchemeState};
    use crate::{GlobalState, RenderContext, ThemeEvent};
    use anyhow::Error;
    use rat_salsa::{AppState, AppWidget, Control};
    use rat_theme::dark_themes;
    use rat_widget::event::{try_flow, HandleEvent, MenuOutcome, Popup, Regular};
    use rat_widget::menu::{MenuBuilder, MenuStructure, Menubar, MenubarState};
    use rat_widget::popup::Placement;
    use rat_widget::scrolled::Scroll;
    use rat_widget::view::{View, ViewState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Layout, Rect};
    use ratatui::widgets::{Block, StatefulWidget};
    use std::fmt::Debug;

    #[derive(Debug)]
    pub struct Mask0;

    #[derive(Debug)]
    pub struct Mask0State {
        pub menu: MenubarState,
        pub scroll: ViewState,
        pub scheme: ShowSchemeState,
        pub theme: usize,
    }

    impl Default for Mask0State {
        fn default() -> Self {
            let s = Self {
                menu: Default::default(),
                scroll: Default::default(),
                scheme: Default::default(),
                theme: 0,
            };
            s.menu.bar.focus.set(true);
            s
        }
    }

    #[derive(Debug)]
    struct Menu;

    impl<'a> MenuStructure<'a> for Menu {
        fn menus(&'a self, menu: &mut MenuBuilder<'a>) {
            menu.item_str("Theme").item_str("Quit");
        }

        fn submenu(&'a self, n: usize, submenu: &mut MenuBuilder<'a>) {
            match n {
                0 => {
                    for t in dark_themes().iter() {
                        submenu.item_string(t.name().into());
                    }
                }
                _ => {}
            }
        }
    }

    impl AppWidget<GlobalState, ThemeEvent, Error> for Mask0 {
        type State = Mask0State;

        fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut Self::State,
            ctx: &mut RenderContext<'_>,
        ) -> Result<(), Error> {
            // TODO: repaint_mask

            let layout = Layout::new(
                Direction::Vertical,
                [Constraint::Fill(1), Constraint::Length(1)],
            )
            .split(area);

            let view = View::new()
                .block(Block::bordered())
                .vscroll(Scroll::new().styles(ctx.g.theme.scroll_style()));
            let view_area = view.inner(layout[0], &mut state.scroll);

            let mut v_buf = view
                .layout(Rect::new(0, 0, view_area.width, 38))
                .into_buffer(layout[0], &mut state.scroll);

            v_buf.render_stateful(
                ShowScheme::new(ctx.g.theme.name(), ctx.g.theme.scheme()),
                Rect::new(0, 0, view_area.width, 38),
                &mut state.scheme,
            );

            v_buf
                .into_widget()
                .render(layout[0], buf, &mut state.scroll);

            let layout_menu =
                Layout::horizontal([Constraint::Percentage(61), Constraint::Percentage(39)])
                    .split(layout[1]);
            let menu = Menubar::new(&Menu)
                .styles(ctx.g.theme.menu_style())
                .popup_placement(Placement::Above)
                .into_widgets();
            menu.0.render(layout_menu[0], buf, &mut state.menu);
            menu.1.render(layout_menu[0], buf, &mut state.menu);

            Ok(())
        }
    }

    impl AppState<GlobalState, ThemeEvent, Error> for Mask0State {
        fn event(
            &mut self,
            event: &ThemeEvent,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, ThemeEvent, Error>,
        ) -> Result<Control<ThemeEvent>, Error> {
            let r = match event {
                ThemeEvent::Event(event) => {
                    try_flow!(match self.menu.handle(event, Popup) {
                        MenuOutcome::MenuSelected(0, n) => {
                            ctx.g.theme = dark_themes()[n].clone();
                            Control::Changed
                        }
                        MenuOutcome::MenuActivated(0, n) => {
                            ctx.g.theme = dark_themes()[n].clone();
                            Control::Changed
                        }
                        r => r.into(),
                    });

                    // TODO: handle_mask

                    try_flow!(match self.menu.handle(event, Regular) {
                        MenuOutcome::Activated(1) => {
                            Control::Quit
                        }
                        r => r.into(),
                    });

                    try_flow!(self.scroll.handle(event, Regular));

                    Control::Continue
                }
                _ => Control::Continue,
            };

            Ok(r)
        }
    }
}

// -----------------------------------------------------------------------

pub mod show_scheme {
    use rat_theme::Scheme;
    use rat_widget::event::{HandleEvent, MouseOnly, Outcome, Regular};
    use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus};
    use rat_widget::reloc::{relocate_area, RelocatableState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
    use ratatui::prelude::{Line, Span, StatefulWidget};
    use ratatui::style::{Style, Stylize};
    use ratatui::widgets::Widget;

    #[derive(Debug)]
    pub struct ShowScheme<'a> {
        name: &'a str,
        scheme: &'a Scheme,
    }

    #[derive(Debug, Default)]
    pub struct ShowSchemeState {
        pub focus: FocusFlag,
        pub area: Rect,
    }

    impl RelocatableState for ShowSchemeState {
        fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
            self.area = relocate_area(self.area, shift, clip);
        }
    }

    impl HasFocus for ShowSchemeState {
        fn build(&self, builder: &mut FocusBuilder) {
            builder.append_leaf(self);
        }

        fn focus(&self) -> FocusFlag {
            self.focus.clone()
        }

        fn area(&self) -> Rect {
            self.area
        }
    }

    impl<'a> ShowScheme<'a> {
        pub fn new(name: &'a str, scheme: &'a Scheme) -> Self {
            Self { name, scheme }
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
                    Constraint::Length(90),
                    Constraint::Fill(1),
                ],
            )
            .split(area);

            let l1 = Layout::new(
                Direction::Vertical,
                [
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
                    Constraint::Length(2),
                ],
            )
            .flex(Flex::Center)
            .split(l0[1]);

            Span::from(format!("{:10}{}", "", self.name))
                .style(Style::new().fg(self.scheme.secondary[3]))
                .render(l1[0], buf);

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
                    Span::from("  DARK  ").bg(c[0]).fg(sc.text_color(c[0])),
                    Span::from("  MID1  ").bg(c[1]).fg(sc.text_color(c[1])),
                    Span::from("  MID2  ").bg(c[2]).fg(sc.text_color(c[2])),
                    Span::from("  LITE  ").bg(c[3]).fg(sc.text_color(c[3])),
                    Span::from("  GRAY  ")
                        .bg(sc.grey_color(c[3]))
                        .fg(sc.text_color(sc.grey_color(c[3]))),
                    Span::from("  DARK  ")
                        .bg(sc.true_dark_color(c[0]))
                        .fg(sc.text_color(sc.true_dark_color(c[0]))),
                    Span::from("  MID1  ")
                        .bg(sc.true_dark_color(c[1]))
                        .fg(sc.text_color(sc.true_dark_color(c[1]))),
                    Span::from("  MID2  ")
                        .bg(sc.true_dark_color(c[2]))
                        .fg(sc.text_color(sc.true_dark_color(c[2]))),
                    Span::from("  LITE  ")
                        .bg(sc.true_dark_color(c[3]))
                        .fg(sc.text_color(sc.true_dark_color(c[3]))),
                ])
                .render(l1[i + 1], buf);
            }
        }
    }

    impl HandleEvent<crossterm::event::Event, Regular, Outcome> for ShowSchemeState {
        fn handle(&mut self, event: &crossterm::event::Event, qualifier: Regular) -> Outcome {
            Outcome::Continue
        }
    }

    impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for ShowSchemeState {
        fn handle(&mut self, event: &crossterm::event::Event, qualifier: MouseOnly) -> Outcome {
            Outcome::Continue
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
