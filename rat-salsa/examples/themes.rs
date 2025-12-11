#![allow(unused_variables)]

use crate::themes::Themes;
use anyhow::Error;
use crossterm::event::Event;
use rat_focus::FocusBuilder;
use rat_salsa::Control;
use rat_salsa::poll::{PollCrossterm, PollTasks, PollTimers};
use rat_salsa::timer::TimeOut;
use rat_salsa::{RunConfig, SalsaAppContext, SalsaContext, run_tui};
use rat_theme4::theme::SalsaTheme;
use rat_theme4::{WidgetStyle, create_salsa_theme};
use rat_widget::event::{Dialog, HandleEvent, ct_event, try_flow};
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
    let theme = create_salsa_theme("Imperial Dark");
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
    pub theme: SalsaTheme,
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
    fn new(cfg: Config, theme: SalsaTheme) -> Self {
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
    mask0: Themes,
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
    themes::render(area, buf, &mut state.mask0, ctx)?;

    let layout = Layout::new(
        Direction::Vertical,
        [Constraint::Fill(1), Constraint::Length(1)],
    )
    .split(area);

    if state.error_dlg.active() {
        MsgDialog::new()
            .styles(ctx.theme.style(WidgetStyle::MSG_DIALOG))
            .render(layout[0], buf, &mut state.error_dlg);
    }

    let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
    state.status.status(1, format!("R {:.3?}", el).to_string());

    let layout_status = Layout::horizontal([
        Constraint::Percentage(61), //
        Constraint::Percentage(39),
    ])
    .split(layout[1]);
    StatusLine::new()
        .layout([
            Constraint::Fill(1),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(12),
        ])
        .styles(ctx.theme.style(WidgetStyle::STATUSLINE))
        .render(layout_status[1], buf, &mut state.status);

    Ok(())
}

pub fn init(state: &mut Scenery, ctx: &mut GlobalState) -> Result<(), Error> {
    ctx.set_focus(FocusBuilder::rebuild_for(&state.mask0, ctx.take_focus()));
    ctx.focus().first();
    state.mask0.themes.select(Some(0));
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

            ctx.set_focus(FocusBuilder::rebuild_for(&state.mask0, ctx.take_focus()));
            ctx.handle_focus(event);

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

    try_flow!(themes::event(&event, &mut state.mask0, ctx)?);

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

pub mod themes {
    use crate::show_scheme::ShowScheme;
    use crate::{GlobalState, ThemesEvent};
    use anyhow::Error;
    use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
    use rat_salsa::Control;
    use rat_theme4::{WidgetStyle, create_salsa_theme, salsa_themes};
    use rat_widget::checkbox::{Checkbox, CheckboxState};
    use rat_widget::event::{HandleEvent, MenuOutcome, Popup, Regular, TableOutcome, try_flow};
    use rat_widget::menu::{MenuBuilder, MenuStructure, Menubar, MenubarState};
    use rat_widget::popup::Placement;
    use rat_widget::scrolled::Scroll;
    use rat_widget::table::selection::RowSelection;
    use rat_widget::table::{Table, TableContext, TableDataIter, TableState};
    use rat_widget::view::{View, ViewState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Layout, Rect};
    use ratatui::text::Span;
    use ratatui::widgets::{Block, Padding, StatefulWidget, Widget};
    use std::fmt::Debug;
    use std::slice;

    #[derive(Debug)]
    pub struct Themes {
        pub contrast: CheckboxState,
        pub themes: TableState<RowSelection>,
        pub scroll: ViewState,
        pub menu: MenubarState,

        pub focus: FocusFlag,
    }

    impl Default for Themes {
        fn default() -> Self {
            let s = Self {
                contrast: Default::default(),
                menu: Default::default(),
                themes: Default::default(),
                scroll: Default::default(),
                focus: Default::default(),
            };
            s.menu.bar.focus.set(true);
            s
        }
    }

    impl HasFocus for Themes {
        fn build(&self, builder: &mut FocusBuilder) {
            builder.widget(&self.contrast);
            builder.widget(&self.themes);
            builder.widget(&self.scroll);
            builder.widget(&self.menu);
        }

        fn focus(&self) -> FocusFlag {
            todo!()
        }

        fn area(&self) -> Rect {
            todo!()
        }
    }

    #[derive(Debug)]
    struct Menu;

    impl<'a> MenuStructure<'a> for Menu {
        fn menus(&'a self, menu: &mut MenuBuilder<'a>) {
            menu.item_str("Quit");
        }

        fn submenu(&'a self, n: usize, submenu: &mut MenuBuilder<'a>) {}
    }

    struct IterThemes<'a> {
        it: slice::Iter<'a, &'a str>,
        item: Option<&'a &'a str>,
    }

    impl<'a> TableDataIter<'a> for IterThemes<'a> {
        fn rows(&self) -> Option<usize> {
            None
        }

        fn nth(&mut self, n: usize) -> bool {
            self.item = self.it.nth(n);
            self.item.is_some()
        }

        fn widths(&self) -> Vec<Constraint> {
            vec![Constraint::Fill(1)]
        }

        fn render_cell(&self, ctx: &TableContext, column: usize, area: Rect, buf: &mut Buffer) {
            let Some(item) = self.item else { return };
            match column {
                0 => Span::from(*item).render(area, buf),
                _ => {}
            }
        }
    }

    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        state: &mut Themes,
        ctx: &mut GlobalState,
    ) -> Result<(), Error> {
        let l0 = Layout::vertical([
            Constraint::Fill(1), //
            Constraint::Length(1),
        ])
        .split(area);
        let l1 = Layout::horizontal([
            Constraint::Length(20),
            Constraint::Length(92),
            Constraint::Fill(1),
        ])
        .spacing(2)
        .split(l0[0]);
        let l2 = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .split(l1[0]);

        Checkbox::new()
            .text("high contrast")
            .styles(ctx.theme.style(WidgetStyle::CHECKBOX))
            .render(l2[0], buf, &mut state.contrast);

        // theme list
        let salsa_themes = salsa_themes();
        let it = IterThemes {
            it: salsa_themes.iter(),
            item: Default::default(),
        };
        Table::new()
            .iter(it)
            .block(Block::new().padding(Padding::new(0, 0, 1, 1)))
            .vscroll(Scroll::new())
            .styles(ctx.theme.style(WidgetStyle::TABLE))
            .render(l2[2], buf, &mut state.themes);

        let mut view = View::new()
            .block(Block::bordered())
            .hscroll(Scroll::new())
            .vscroll(Scroll::new())
            .styles(ctx.theme.style(WidgetStyle::VIEW))
            .view_width(90)
            .view_height(34)
            .into_buffer(l1[1], &mut state.scroll);

        view.render_widget(
            ShowScheme::new(state.contrast.value(), &ctx.theme, &ctx.theme.p),
            Rect::new(0, 0, view.layout().width, 34),
        );

        view.finish(buf, &mut state.scroll);

        let layout_menu = Layout::horizontal([
            Constraint::Percentage(61), //
            Constraint::Percentage(39),
        ])
        .split(l0[1]);

        let (menu, menu_popup) = Menubar::new(&Menu)
            .styles(ctx.theme.style(WidgetStyle::MENU))
            .popup_placement(Placement::Above)
            .into_widgets();
        menu.render(layout_menu[0], buf, &mut state.menu);
        menu_popup.render(layout_menu[0], buf, &mut state.menu);

        Ok(())
    }

    pub fn event(
        event: &ThemesEvent,
        state: &mut Themes,
        ctx: &mut GlobalState,
    ) -> Result<Control<ThemesEvent>, Error> {
        if let ThemesEvent::Event(event) = event {
            try_flow!(match state.menu.handle(event, Popup) {
                MenuOutcome::Activated(0) => {
                    Control::Quit
                }
                r => r.into(),
            });
            try_flow!(state.contrast.handle(event, Regular));
            try_flow!(match state.themes.handle(event, Regular) {
                TableOutcome::Selected => {
                    if let Some(idx) = state.themes.selected_checked() {
                        let theme = salsa_themes()[idx];
                        ctx.theme = create_salsa_theme(theme);
                    }
                    Control::Changed
                }
                r => r.into(),
            });
            try_flow!(state.scroll.handle(event, Regular));
        }

        Ok(Control::Continue)
    }
}

pub mod show_scheme {
    use rat_theme4::StyleName;
    use rat_theme4::palette::{Colors, Palette};
    use rat_theme4::theme::SalsaTheme;
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Layout, Rect};
    use ratatui::style::{Color, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::Widget;

    #[derive(Debug)]
    pub struct ShowScheme<'a> {
        high_contrast: bool,
        theme: &'a SalsaTheme,
        palette: &'a Palette,
    }

    impl<'a> ShowScheme<'a> {
        pub fn new(high_contrast: bool, theme: &'a SalsaTheme, palette: &'a Palette) -> Self {
            Self {
                high_contrast,
                theme,
                palette,
            }
        }
    }

    impl<'a> Widget for ShowScheme<'a> {
        fn render(self, area: Rect, buf: &mut Buffer) {
            buf.set_style(area, self.theme.style_style(Style::CONTAINER_BASE));
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
                ],
            )
            .split(area);

            let pal = self.palette;
            for (i, (n, c)) in [
                ("primary", pal.color[Colors::Primary as usize]),
                ("secondary", pal.color[Colors::Secondary as usize]),
                ("white", pal.color[Colors::White as usize]),
                ("black", pal.color[Colors::Black as usize]),
                ("gray", pal.color[Colors::Gray as usize]),
                ("red", pal.color[Colors::Red as usize]),
                ("orange", pal.color[Colors::Orange as usize]),
                ("yellow", pal.color[Colors::Yellow as usize]),
                ("limegreen", pal.color[Colors::LimeGreen as usize]),
                ("green", pal.color[Colors::Green as usize]),
                ("bluegreen", pal.color[Colors::BlueGreen as usize]),
                ("cyan", pal.color[Colors::Cyan as usize]),
                ("blue", pal.color[Colors::Blue as usize]),
                ("deepblue", pal.color[Colors::DeepBlue as usize]),
                ("purple", pal.color[Colors::Purple as usize]),
                ("magenta", pal.color[Colors::Magenta as usize]),
                ("redpink", pal.color[Colors::RedPink as usize]),
            ]
            .iter()
            .enumerate()
            {
                let ccc = |c: Color| {
                    if self.high_contrast {
                        pal.high_contrast(c)
                    } else {
                        pal.normal_contrast(c)
                    }
                };

                Line::from(vec![
                    Span::from(format!("{:10}", n)),
                    Span::from("  FG-0  ").style(ccc(c[0])),
                    Span::from("  FG-1  ").style(ccc(c[1])),
                    Span::from("  FG-2  ").style(ccc(c[2])),
                    Span::from("  FG-3  ").style(ccc(c[3])),
                    Span::from("  BG-0   ").style(ccc(c[4])),
                    Span::from("  BG-1  ").style(ccc(c[5])),
                    Span::from("  BG-3  ").style(ccc(c[6])),
                    Span::from("  BG-4  ").style(ccc(c[7])),
                    Span::from("  grayscale  ").style(pal.high_contrast(Palette::grayscale(c[3]))),
                ])
                .render(l1[i], buf);
            }
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
