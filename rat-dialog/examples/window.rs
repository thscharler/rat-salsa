use crate::moving::MovingState;
use anyhow::Error;
use rat_dialog::DialogStack;
use rat_salsa::event::RenderedEvent;
use rat_salsa::poll::{PollCrossterm, PollRendered, PollTasks, PollTimers};
use rat_salsa::timer::TimeOut;
use rat_salsa::{Control, RunConfig, SalsaAppContext, SalsaContext, run_tui};
use rat_theme2::DarkTheme;
use rat_theme2::palettes::IMPERIAL;
use rat_widget::event::{ConsumedEvent, Dialog, HandleEvent, Regular, ct_event};
use rat_widget::focus::FocusBuilder;
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::statusline::{StatusLine, StatusLineState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::StatefulWidget;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

fn main() -> Result<(), Error> {
    setup_logging()?;

    let config = Config::default();
    let theme = DarkTheme::new("Imperial".into(), IMPERIAL);
    let mut global = Global::new(config, theme);
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
            .poll(PollTasks::default())
            .poll(PollRendered),
    )?;

    Ok(())
}

/// Globally accessible data/state.
#[derive(Debug)]
pub struct Global {
    ctx: SalsaAppContext<AppEvent, Error>,
    pub cfg: Config,
    pub theme: DarkTheme,
    pub dialogs: DialogStack<AppEvent, Global, Error>,
}

impl SalsaContext<AppEvent, Error> for Global {
    fn set_salsa_ctx(&mut self, app_ctx: SalsaAppContext<AppEvent, Error>) {
        self.ctx = app_ctx;
    }

    #[inline(always)]
    fn salsa_ctx(&self) -> &SalsaAppContext<AppEvent, Error> {
        &self.ctx
    }
}

impl Global {
    pub fn new(cfg: Config, theme: DarkTheme) -> Self {
        Self {
            ctx: Default::default(),
            cfg,
            theme,
            dialogs: Default::default(),
        }
    }
}

/// Configuration.
#[derive(Debug, Default)]
pub struct Config {}

/// Application wide messages.
#[derive(Debug)]
pub enum AppEvent {
    Timer(TimeOut),
    Event(crossterm::event::Event),
    Rendered,
    Message(String),
    Status(usize, String),
}

impl From<RenderedEvent> for AppEvent {
    fn from(_: RenderedEvent) -> Self {
        Self::Rendered
    }
}

impl From<TimeOut> for AppEvent {
    fn from(value: TimeOut) -> Self {
        Self::Timer(value)
    }
}

impl From<crossterm::event::Event> for AppEvent {
    fn from(value: crossterm::event::Event) -> Self {
        Self::Event(value)
    }
}

#[derive(Debug, Default)]
pub struct Scenery {
    pub nominal: MovingState,
    pub status: StatusLineState,
    pub error_dlg: MsgDialogState,
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut Scenery,
    ctx: &mut Global,
) -> Result<(), Error> {
    let t0 = SystemTime::now();

    moving::render(area, buf, &mut state.nominal, ctx)?;

    let layout = Layout::vertical([
        Constraint::Fill(1), //
        Constraint::Length(1),
    ])
    .split(area);

    ctx.dialogs.clone().render(layout[0], buf, ctx);

    if state.error_dlg.active() {
        MsgDialog::new()
            .styles(ctx.theme.msg_dialog_style())
            .render(layout[0], buf, &mut state.error_dlg);
    }

    let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
    state.status.status(2, format!("R {:.0?}", el).to_string());

    let status_layout = Layout::horizontal([
        Constraint::Length(20), //
        Constraint::Fill(1),
    ])
    .split(layout[1]);

    StatusLine::new()
        .layout([
            Constraint::Fill(1),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
        ])
        .styles(ctx.theme.statusline_style())
        .render(status_layout[1], buf, &mut state.status);

    Ok(())
}

pub fn init(state: &mut Scenery, ctx: &mut Global) -> Result<(), Error> {
    ctx.set_focus(FocusBuilder::build_for(&state.nominal));
    moving::init(&mut state.nominal, ctx)?;
    Ok(())
}

pub fn event(
    event: &AppEvent,
    state: &mut Scenery,
    ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    let t0 = SystemTime::now();

    let mut r = match event {
        AppEvent::Event(event) => {
            let mut r = match &event {
                ct_event!(resized) => Control::Changed,
                ct_event!(key press CONTROL-'q') => Control::Quit,
                _ => Control::Continue,
            };

            r = r.or_else(|| {
                if state.error_dlg.active() {
                    state.error_dlg.handle(event, Dialog).into()
                } else {
                    Control::Continue
                }
            });

            let f = ctx.focus_mut().handle(event, Regular);
            ctx.queue(f);

            r
        }
        AppEvent::Rendered => {
            ctx.set_focus(FocusBuilder::rebuild_for(&state.nominal, ctx.take_focus()));
            Control::Continue
        }
        AppEvent::Message(s) => {
            state.error_dlg.append(s.as_str());
            Control::Changed
        }
        AppEvent::Status(n, s) => {
            state.status.status(*n, s);
            Control::Changed
        }
        _ => Control::Continue,
    };

    r = r.or_else_try(|| {
        ctx.dialogs
            .clone()
            .handle(event, ctx)
            .map(|v| Control::from(v))
    })?;

    r = r.or_else_try(|| moving::event(event, &mut state.nominal, ctx))?;

    let el = t0.elapsed()?;
    state.status.status(3, format!("E {:.0?}", el).to_string());

    Ok(r)
}

pub fn error(
    event: Error,
    state: &mut Scenery,
    _ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    state.error_dlg.append(format!("{:?}", &*event).as_str());
    Ok(Control::Changed)
}

pub mod moving {
    use crate::{AppEvent, Global};
    use anyhow::Error;
    use rat_dialog::event::{DialogControl, WindowOutcome};
    use rat_dialog::{Window, WindowState};
    use rat_event::{Dialog, Popup};
    use rat_salsa::timer::TimerDef;
    use rat_salsa::{Control, SalsaContext};
    use rat_widget::event::{FileOutcome, HandleEvent, MenuOutcome, Regular, try_flow};
    use rat_widget::file_dialog::{FileDialog, FileDialogState};
    use rat_widget::focus::impl_has_focus;
    use rat_widget::layout::layout_middle;
    use rat_widget::menu::{Menubar, MenubarState, StaticMenu};
    use rat_widget::text::HasScreenCursor;
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Layout, Rect};
    use ratatui::widgets::{Block, StatefulWidget};
    use std::path::PathBuf;
    use std::time::Duration;

    pub struct Moving;

    #[derive(Debug, Default)]
    pub struct MovingState {
        pub menu: MenubarState,
    }

    static MENU: StaticMenu = StaticMenu {
        menu: &[("_File", &["_Open"]), ("_Quit", &[])],
    };

    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        state: &mut MovingState,
        ctx: &mut Global,
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

        let (menubar, menupopup) = Menubar::new(&MENU)
            .styles(ctx.theme.menu_style())
            .title("rata-tui")
            .into_widgets();
        menubar.render(r[1], buf, &mut state.menu);
        menupopup.render(r[1], buf, &mut state.menu);

        Ok(())
    }

    impl_has_focus!(menu for MovingState);

    pub fn init(_state: &mut MovingState, ctx: &mut Global) -> Result<(), Error> {
        ctx.focus().first();

        ctx.add_timer(
            TimerDef::new()
                .repeat_forever()
                .timer(Duration::from_millis(1000)),
        );

        Ok(())
    }

    #[allow(unused_variables)]
    pub fn event(
        event: &AppEvent,
        state: &mut MovingState,
        ctx: &mut Global,
    ) -> Result<Control<AppEvent>, Error> {
        match event {
            AppEvent::Event(event) => {
                try_flow!(match state.menu.handle(event, Popup) {
                    MenuOutcome::MenuActivated(0, 0) => {
                        show_open(ctx)?
                    }
                    MenuOutcome::Activated(1) => {
                        Control::Quit
                    }
                    v => v.into(),
                });
                Ok(Control::Continue)
            }
            AppEvent::Timer(t) => Ok(Control::Event(AppEvent::Status(
                1,
                format!("up {}s", t.counter),
            ))),
            _ => Ok(Control::Continue),
        }
    }

    fn show_open(ctx: &mut Global) -> Result<Control<AppEvent>, Error> {
        struct OpenState {
            window: WindowState,
            filedlg: FileDialogState,
        }

        let mut state = OpenState {
            window: WindowState::new(),
            filedlg: FileDialogState::new(),
        };
        state.filedlg.open_dialog(PathBuf::from("."))?;

        ctx.dialogs.push(
            |area, buf, state, ctx| {
                let state = state.downcast_mut::<OpenState>().expect("dialog-state");

                if state.window.area.is_empty() {
                    state.window.limit = area;
                    state.window.area = layout_middle(
                        area,
                        Constraint::Percentage(19),
                        Constraint::Percentage(19),
                        Constraint::Length(2),
                        Constraint::Length(2),
                    );
                }

                Window::new()
                    .drag(ctx.theme.dialog_base())
                    .hover(ctx.theme.limegreen(1))
                    .style(ctx.theme.dialog_base())
                    .block(
                        Block::bordered()
                            .border_style(ctx.theme.dialog_border())
                            .title("Open file"),
                    )
                    .render(area, buf, &mut state.window);

                FileDialog::new()
                    .styles(ctx.theme.file_dialog_style())
                    .no_block()
                    .render(state.window.widget_area, buf, &mut state.filedlg);

                ctx.set_screen_cursor(state.filedlg.screen_cursor());
            },
            |event, state, ctx| {
                let state = state.downcast_mut::<OpenState>().expect("open-state");
                match event {
                    AppEvent::Event(event) => {
                        try_flow!(match state.window.handle(event, Regular) {
                            WindowOutcome::ShouldClose => {
                                DialogControl::Close(None)
                            }
                            r => r.into(),
                        });

                        try_flow!(match state.filedlg.handle(event, Dialog)? {
                            FileOutcome::Cancel => DialogControl::Close(None),
                            FileOutcome::Ok(f) => {
                                DialogControl::Close(Some(AppEvent::Status(
                                    0,
                                    format!("Open file {:?}", f),
                                )))
                            }
                            r => r.into(),
                        });

                        Ok(DialogControl::Continue)
                    }
                    _ => Ok(DialogControl::Continue),
                }
            },
            state,
        );

        Ok(Control::Changed)
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
