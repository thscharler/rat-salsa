#![allow(unused_variables)]

use crate::mask0::Mask0;
use anyhow::Error;
use crossterm::event::Event;
use rat_salsa::poll::{PollCrossterm, PollTasks, PollTimers};
use rat_salsa::timer::TimeOut;
use rat_salsa::Control;
use rat_salsa::{run_tui, RunConfig, SalsaAppContext, SalsaContext};
use rat_theme3::{create_theme, SalsaTheme};
use rat_widget::event::{ct_event, try_flow, Dialog, HandleEvent};
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::statusline::{StatusLine, StatusLineState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::StatefulWidget;
use std::fmt::Debug;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

fn main() -> Result<(), Error> {
    setup_logging()?;

    let config = Config::default();
    let theme = create_theme("Imperial Dark").expect("theme");
    let mut global = GlobalState::new(config, theme);
    let mut state = Scenery::default();

    run_tui(
        init,
        render,
        event,
        error,
        &mut global,
        &mut state,
        RunConfig::default()?
            .poll(PollCrossterm)
            .poll(PollTimers::default())
            .poll(PollTasks::default()),
    )?;

    Ok(())
}

pub struct GlobalState {
    pub ctx: SalsaAppContext<ThemesEvent, Error>,
    pub cfg: Config,
    pub theme: Box<dyn SalsaTheme>,
}

impl SalsaContext<ThemesEvent, Error> for GlobalState {
    fn set_salsa_ctx(&mut self, app_ctx: SalsaAppContext<ThemesEvent, Error>) {
        self.ctx = app_ctx;
    }

    fn salsa_ctx(&self) -> &SalsaAppContext<ThemesEvent, Error> {
        &self.ctx
    }
}

impl GlobalState {
    fn new(cfg: Config, theme: Box<dyn SalsaTheme>) -> Self {
        Self {
            ctx: Default::default(),
            cfg,
            theme,
        }
    }
}

#[derive(Debug, Default)]
pub struct Config {}

#[derive(Debug)]
pub enum ThemesEvent {
    Event(Event),
    TimeOut(TimeOut),
    Message(String),
    Status(usize, String),
}

impl From<Event> for ThemesEvent {
    fn from(value: Event) -> Self {
        Self::Event(value)
    }
}

impl From<TimeOut> for ThemesEvent {
    fn from(value: TimeOut) -> Self {
        Self::TimeOut(value)
    }
}

#[derive(Debug, Default)]
pub struct Scenery {
    mask0: Mask0,
    status: StatusLineState,
    error_dlg: MsgDialogState,
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut Scenery,
    ctx: &mut GlobalState,
) -> Result<(), Error> {
    let t0 = SystemTime::now();

    // forward
    mask0::render(area, buf, &mut state.mask0, ctx)?;

    let layout = Layout::new(
        Direction::Vertical,
        [Constraint::Fill(1), Constraint::Length(1)],
    )
    .split(area);

    if state.error_dlg.active() {
        MsgDialog::new()
            .styles(ctx.theme.msg_dialog_style())
            .render(layout[0], buf, &mut state.error_dlg);
    }

    let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
    state.status.status(1, format!("R {:.3?}", el).to_string());

    let layout_status =
        Layout::horizontal([Constraint::Percentage(61), Constraint::Percentage(39)])
            .split(layout[1]);
    StatusLine::new()
        .layout([
            Constraint::Fill(1),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(12),
        ])
        .styles(ctx.theme.statusline_style())
        .render(layout_status[1], buf, &mut state.status);

    Ok(())
}

pub fn init(_state: &mut Scenery, _ctx: &mut GlobalState) -> Result<(), Error> {
    Ok(())
}

pub fn event(
    event: &ThemesEvent,
    state: &mut Scenery,
    ctx: &mut GlobalState,
) -> Result<Control<ThemesEvent>, Error> {
    let t0 = SystemTime::now();

    let r = match event {
        ThemesEvent::Event(event) => {
            try_flow!(match &event {
                Event::Resize(_, _) => Control::Changed,
                ct_event!(key press CONTROL-'q') => Control::Quit,
                _ => Control::Continue,
            });

            try_flow!({
                if state.error_dlg.active() {
                    state.error_dlg.handle(&event, Dialog).into()
                } else {
                    Control::Continue
                }
            });

            Control::Continue
        }
        ThemesEvent::Message(s) => {
            state.error_dlg.append(s.as_str());
            Control::Changed
        }
        ThemesEvent::Status(n, s) => {
            state.status.status(*n, s);
            Control::Changed
        }
        _ => Control::Continue,
    };

    try_flow!(mask0::event(&event, &mut state.mask0, ctx)?);

    let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
    state.status.status(3, format!("H {:.3?}", el).to_string());

    Ok(r)
}

pub fn error(
    event: Error,
    state: &mut Scenery,
    ctx: &mut GlobalState,
) -> Result<Control<ThemesEvent>, Error> {
    state.error_dlg.append(format!("{:?}", &*event).as_str());
    Ok(Control::Changed)
}

pub mod mask0 {
    use crate::show_scheme::{ShowScheme, ShowSchemeState};
    use crate::{GlobalState, ThemesEvent};
    use anyhow::Error;
    use rat_salsa::Control;
    use rat_theme3::{create_theme, salsa_themes};
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
    pub struct Mask0 {
        pub menu: MenubarState,
        pub scroll: ViewState,
        pub scheme: ShowSchemeState,
        pub theme: usize,
    }

    impl Default for Mask0 {
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
                    for t in salsa_themes().iter() {
                        submenu.item_str(*t);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        state: &mut Mask0,
        ctx: &mut GlobalState,
    ) -> Result<(), Error> {
        let layout = Layout::new(
            Direction::Vertical,
            [Constraint::Fill(1), Constraint::Length(1)],
        )
        .split(area);

        let view = View::new()
            .block(Block::bordered())
            .vscroll(Scroll::new().styles(ctx.theme.scroll_style()));
        let view_area = view.inner(layout[0], &mut state.scroll);

        let mut v_buf = view
            .layout(Rect::new(0, 0, view_area.width, 38))
            .into_buffer(layout[0], &mut state.scroll);

        v_buf.render_stateful(
            ShowScheme::new(ctx.theme.name(), ctx.theme.palette()),
            Rect::new(0, 0, view_area.width, 38),
            &mut state.scheme,
        );

        v_buf
            .into_widget()
            .render(layout[0], buf, &mut state.scroll);

        let layout_menu =
            Layout::horizontal([Constraint::Percentage(61), Constraint::Percentage(39)])
                .split(layout[1]);
        let (menu, menu_popup) = Menubar::new(&Menu)
            .styles(ctx.theme.menu_style())
            .popup_placement(Placement::Above)
            .into_widgets();
        menu.render(layout_menu[0], buf, &mut state.menu);
        menu_popup.render(layout_menu[0], buf, &mut state.menu);

        Ok(())
    }

    pub fn event(
        event: &ThemesEvent,
        state: &mut Mask0,
        ctx: &mut GlobalState,
    ) -> Result<Control<ThemesEvent>, Error> {
        let r = match event {
            ThemesEvent::Event(event) => {
                try_flow!(match state.menu.handle(event, Popup) {
                    MenuOutcome::MenuSelected(0, n) => {
                        let theme = salsa_themes()[n];
                        ctx.theme = create_theme(theme).expect("theme");
                        Control::Changed
                    }
                    MenuOutcome::MenuActivated(0, n) => {
                        let theme = salsa_themes()[n];
                        ctx.theme = create_theme(theme).expect("theme");
                        Control::Changed
                    }
                    MenuOutcome::Activated(1) => {
                        Control::Quit
                    }
                    r => r.into(),
                });

                try_flow!(state.scroll.handle(event, Regular));

                Control::Continue
            }
            _ => Control::Continue,
        };

        Ok(r)
    }
}

pub mod show_scheme {
    use rat_theme3::{Palette, TextColorRating};
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
        scheme: &'a Palette,
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
        pub fn new(name: &'a str, scheme: &'a Palette) -> Self {
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

            let make_fg = |c| match Palette::rate_text_color(c) {
                None => Color::Reset,
                Some(TextColorRating::Light) => self.scheme.white[3],
                Some(TextColorRating::Dark) => self.scheme.black[0],
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
                    Span::from("  FG-0  ").bg(c[0]).fg(make_fg(c[0])),
                    Span::from("  FG-1  ").bg(c[1]).fg(make_fg(c[1])),
                    Span::from("  FG-2  ").bg(c[2]).fg(make_fg(c[2])),
                    Span::from("  FG-3  ").bg(c[3]).fg(make_fg(c[3])),
                    Span::from("  BG-0   ").bg(c[4]).fg(make_fg(c[4])),
                    Span::from("  BG-1  ").bg(c[5]).fg(make_fg(c[5])),
                    Span::from("  BG-3  ").bg(c[6]).fg(make_fg(c[6])),
                    Span::from("  BG-4  ").bg(c[7]).fg(make_fg(c[7])),
                    Span::from("  grayscale  ")
                        .bg(Palette::grayscale(c[3]))
                        .fg(make_fg(Palette::grayscale(c[3]))),
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
    let log_path = PathBuf::from(".");
    let log_file = log_path.join("log.log");
    _ = fs::remove_file(&log_file);
    fern::Dispatch::new()
        .format(|out, message, _record| {
            out.finish(format_args!("{}", message)) //
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file(&log_file)?)
        .apply()?;
    Ok(())
}
