#![allow(unused_variables)]

use crate::mask0::{Mask0, Mask0State};
use anyhow::Error;
use crossterm::event::Event;
use rat_salsa::poll::{PollCrossterm, PollTasks, PollTimers};
use rat_salsa::timer::TimeOut;
use rat_salsa::{run_tui, AppState, AppWidget, Control, RunConfig};
use rat_theme2::schemes::IMPERIAL;
use rat_theme2::DarkTheme;
use rat_widget::event::{ct_event, try_flow, Dialog, HandleEvent};
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::statusline::{StatusLine, StatusLineState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::StatefulWidget;
use std::fmt::Debug;
use std::fs;
use std::rc::Rc;
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
            .poll(PollCrossterm)
            .poll(PollTimers::default())
            .poll(PollTasks::default()),
    )?;

    Ok(())
}

// -----------------------------------------------------------------------

#[derive(Debug)]
pub struct GlobalState {
    pub cfg: MinimalConfig,
    pub theme: Rc<DarkTheme>,
}

impl GlobalState {
    fn new(cfg: MinimalConfig, theme: DarkTheme) -> Self {
        Self {
            cfg,
            theme: Rc::new(theme),
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
    Status(usize, String),
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
    status: StatusLineState,
    error_dlg: MsgDialogState,
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
        let theme = ctx.g.theme.clone();

        let layout = Layout::new(
            Direction::Vertical,
            [Constraint::Fill(1), Constraint::Length(1)],
        )
        .split(area);

        Mask0.render(area, buf, &mut state.mask0, ctx)?;

        if state.error_dlg.active() {
            let err = MsgDialog::new().styles(theme.msg_dialog_style());
            err.render(layout[0], buf, &mut state.error_dlg);
        }

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        state.status.status(1, format!("R {:.3?}", el).to_string());

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
            .styles(theme.statusline_style());
        status.render(layout_status[1], buf, &mut state.status);

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
                    if self.error_dlg.active() {
                        self.error_dlg.handle(&event, Dialog).into()
                    } else {
                        Control::Continue
                    }
                });

                Control::Continue
            }
            ThemeEvent::Message(s) => {
                self.error_dlg.append(s.as_str());
                Control::Changed
            }
            ThemeEvent::Status(n, s) => {
                self.status.status(*n, s);
                Control::Changed
            }
            _ => Control::Continue,
        };

        try_flow!(self.mask0.event(&event, ctx)?);

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        self.status.status(3, format!("H {:.3?}", el).to_string());

        Ok(r)
    }

    fn error(&self, event: Error, ctx: &mut AppContext<'_>) -> Result<Control<ThemeEvent>, Error> {
        self.error_dlg.append(format!("{:?}", &*event).as_str());
        Ok(Control::Changed)
    }
}

pub mod mask0 {
    use crate::show_scheme::{ShowScheme, ShowSchemeState};
    use crate::{GlobalState, RenderContext, ThemeEvent};
    use anyhow::Error;
    use rat_salsa::{AppState, AppWidget, Control};
    use rat_theme2::dark_themes;
    use rat_widget::event::{try_flow, HandleEvent, MenuOutcome, Popup, Regular};
    use rat_widget::menu::{MenuBuilder, MenuStructure, Menubar, MenubarState};
    use rat_widget::popup::Placement;
    use rat_widget::scrolled::Scroll;
    use rat_widget::view::{View, ViewState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Layout, Rect};
    use ratatui::widgets::{Block, StatefulWidget};
    use std::fmt::Debug;
    use std::rc::Rc;

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

            let theme = ctx.g.theme.clone();

            let layout = Layout::new(
                Direction::Vertical,
                [Constraint::Fill(1), Constraint::Length(1)],
            )
            .split(area);

            let view = View::new()
                .block(Block::bordered())
                .vscroll(Scroll::new().styles(theme.scroll_style()));
            let view_area = view.inner(layout[0], &mut state.scroll);

            let mut v_buf = view
                .layout(Rect::new(0, 0, view_area.width, 38))
                .into_buffer(layout[0], &mut state.scroll);

            v_buf.render_stateful(
                ShowScheme::new(theme.name(), theme.scheme()),
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
                .styles(theme.menu_style())
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
                            ctx.g.theme = Rc::new(dark_themes()[n].clone());
                            Control::Changed
                        }
                        MenuOutcome::MenuActivated(0, n) => {
                            ctx.g.theme = Rc::new(dark_themes()[n].clone());
                            Control::Changed
                        }
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
    use rat_theme2::{Scheme, TextColorRating};
    use rat_widget::event::{HandleEvent, MouseOnly, Outcome, Regular};
    use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus};
    use rat_widget::reloc::{relocate_area, RelocatableState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
    use ratatui::style::{Color, Style, Stylize};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::StatefulWidget;
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
            builder.leaf_widget(self);
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

            let make_fg = |c| match Scheme::rate_text_color(c) {
                None => Color::Reset,
                Some(TextColorRating::Light) => self.scheme.white[0],
                Some(TextColorRating::Dark) => self.scheme.black[3],
            };

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
                    Span::from("  THEME  ").bg(c[0]).fg(make_fg(c[0])),
                    Span::from("  MID-1  ").bg(c[1]).fg(make_fg(c[1])),
                    Span::from("  MID-2  ").bg(c[2]).fg(make_fg(c[2])),
                    Span::from("  LIGHT  ").bg(c[3]).fg(make_fg(c[3])),
                    Span::from("  BACK   ").bg(c[4]).fg(make_fg(c[4])),
                    Span::from("  MID-1  ").bg(c[5]).fg(make_fg(c[5])),
                    Span::from("  MID-2  ").bg(c[6]).fg(make_fg(c[6])),
                    Span::from("  LIGHT  ").bg(c[7]).fg(make_fg(c[7])),
                    Span::from("  GRAY   ")
                        .bg(Scheme::grayscale(c[3]))
                        .fg(make_fg(Scheme::grayscale(c[3]))),
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

fn setup_logging() -> Result<(), Error> {
    if let Some(cache) = dirs::cache_dir() {
        let log_path = cache.join("rat-salsa");
        if !log_path.exists() {
            fs::create_dir_all(&log_path)?;
        }

        let log_file = log_path.join("theme_sample.log");
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
