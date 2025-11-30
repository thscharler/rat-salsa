use crate::moving::MovingState;
use anyhow::Error;
use rat_dialog::{DialogStack, WindowControl, handle_dialog_stack};
use rat_event::{Outcome, Popup, break_flow, try_flow};
use rat_focus::impl_has_focus;
use rat_salsa::event::RenderedEvent;
use rat_salsa::poll::{PollCrossterm, PollRendered, PollTasks, PollTimers};
use rat_salsa::timer::{TimeOut, TimerDef};
use rat_salsa::{Control, RunConfig, SalsaAppContext, SalsaContext, run_tui};
use rat_theme4::WidgetStyle;
use rat_theme4::palettes::dark::IMPERIAL;
use rat_theme4::theme::SalsaTheme;
use rat_theme4::themes::create_dark;
use rat_widget::event::{Dialog, HandleEvent, MenuOutcome, ct_event};
use rat_widget::focus::FocusBuilder;
use rat_widget::menu::{Menubar, MenubarState, StaticMenu};
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::statusline::{StatusLine, StatusLineState};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::Event;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use try_as::traits::TryAsRef;

type AppResult = Result<Control<AppEvent>, Error>;
type AppDialogResult = Result<WindowControl<AppEvent>, Error>;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let theme = create_dark("Imperial Dark", IMPERIAL);
    let mut global = Global::new(theme);
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
    pub theme: SalsaTheme,
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
    pub fn new(theme: SalsaTheme) -> Self {
        Self {
            ctx: Default::default(),
            theme,
            dialogs: Default::default(),
        }
    }
}

/// Application wide messages.
#[derive(Debug)]
pub enum AppEvent {
    Timer(TimeOut),
    Event(Event),
    Rendered,
    Message(String),
    Status(usize, String),
    NoOp,
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

impl TryAsRef<Event> for AppEvent {
    fn try_as_ref(&self) -> Option<&Event> {
        match self {
            AppEvent::Event(e) => Some(e),
            _ => None,
        }
    }
}

impl From<Event> for AppEvent {
    fn from(value: Event) -> Self {
        Self::Event(value)
    }
}

#[derive(Debug, Default)]
pub struct Scenery {
    pub moving: MovingState,
    pub menu: MenubarState,
    pub status: StatusLineState,
    pub error_dlg: MsgDialogState,
}

impl_has_focus!(menu for Scenery);

static MENU: StaticMenu = StaticMenu {
    menu: &[
        ("_File", &["_Open (moveable)", "O_pen (fixed)"]),
        ("_Quit", &[]),
    ],
};

pub fn init(state: &mut Scenery, ctx: &mut Global) -> Result<(), Error> {
    ctx.set_focus(FocusBuilder::build_for(state));
    ctx.focus().first();

    ctx.add_timer(
        TimerDef::new()
            .repeat_forever()
            .timer(Duration::from_millis(1000)),
    );

    moving::init(&mut state.moving, ctx)?;

    Ok(())
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut Scenery,
    ctx: &mut Global,
) -> Result<(), Error> {
    let t0 = SystemTime::now();

    moving::render(area, buf, &mut state.moving, ctx)?;

    let layout = Layout::vertical([
        Constraint::Fill(1), //
        Constraint::Length(1),
    ])
    .split(area);

    let (menubar, menupopup) = Menubar::new(&MENU)
        .styles(ctx.theme.style(WidgetStyle::MENU))
        .title("rata-tui")
        .into_widgets();
    menubar.render(layout[1], buf, &mut state.menu);
    menupopup.render(layout[1], buf, &mut state.menu);

    ctx.dialogs.clone().render(layout[0], buf, ctx);

    if state.error_dlg.active() {
        MsgDialog::new()
            .styles(ctx.theme.style(WidgetStyle::MSG_DIALOG))
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
        .styles_ext(ctx.theme.style(WidgetStyle::STATUSLINE))
        .render(status_layout[1], buf, &mut state.status);

    Ok(())
}

#[allow(unused_variables)]
pub fn event(event: &AppEvent, state: &mut Scenery, ctx: &mut Global) -> AppResult {
    let t0 = SystemTime::now();

    let r = 'b: {
        break_flow!('b: handle_super_keys(event, state, ctx)?);
        break_flow!('b: handle_error_dlg(event, state, ctx)?);
        break_flow!('b: handle_app_events(event, state, ctx)?);
        break_flow!('b: {
            match handle_dialog_stack(ctx.dialogs.clone(), event, ctx)? {
                WindowControl::Continue => Control::Continue,
                WindowControl::Unchanged => Control::Unchanged,
                WindowControl::Changed => Control::Changed,
                WindowControl::Event(e) => Control::Event(e),
                WindowControl::Close(e) => {
                    ctx.queue_event(e);
                    Control::Changed
                }
            }
        });
        break_flow!('b: {
            if let AppEvent::Event(event) = event {
                ctx.handle_focus(event);
            }
            Control::Continue
        });
        break_flow!('b: handle_menu(event, state, ctx)?);
        break_flow!('b: moving::event(event, &mut state.moving, ctx)?);
        Control::Continue
    };

    let el = t0.elapsed()?;
    state.status.status(3, format!("E {:.0?}", el).to_string());

    Ok(r)
}

fn handle_menu(event: &AppEvent, state: &mut Scenery, ctx: &mut Global) -> AppResult {
    match event {
        AppEvent::Event(event) => {
            try_flow!(match state.menu.handle(event, Popup) {
                MenuOutcome::MenuActivated(0, 0) => {
                    moving::show_moveable_filedlg(ctx)?
                }
                MenuOutcome::MenuActivated(0, 1) => {
                    moving::show_fixed_filedlg(ctx)?
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

fn handle_app_events(event: &AppEvent, state: &mut Scenery, ctx: &mut Global) -> AppResult {
    match event {
        AppEvent::Rendered => {
            ctx.set_focus(FocusBuilder::rebuild_for(state, ctx.take_focus()));
            Ok(Control::Continue)
        }
        AppEvent::Message(s) => {
            state.error_dlg.append(s.as_str());
            Ok(Control::Changed)
        }
        AppEvent::Status(n, s) => {
            state.status.status(*n, s);
            Ok(Control::Changed)
        }
        _ => Ok(Control::Continue),
    }
}

fn handle_super_keys(event: &AppEvent, _state: &mut Scenery, _ctx: &mut Global) -> AppResult {
    match event {
        AppEvent::Event(event) => match &event {
            ct_event!(resized) => Ok(Control::Changed),
            ct_event!(key press CONTROL-'q') => Ok(Control::Quit),
            ct_event!(key press ALT-'x') => Ok(Control::Quit),
            _ => Ok(Control::Continue),
        },
        _ => Ok(Control::Continue),
    }
}

fn handle_error_dlg(event: &AppEvent, state: &mut Scenery, _ctx: &mut Global) -> AppResult {
    if let AppEvent::Event(event) = event {
        try_flow!(if state.error_dlg.active() {
            state.error_dlg.handle(event, Dialog)
        } else {
            Outcome::Continue
        });
    }
    Ok(Control::Continue)
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
    use crate::file_dlg_moveable::FileDlgState;
    use crate::{AppEvent, AppResult, Global, file_dlg_fixed, file_dlg_moveable};
    use anyhow::Error;
    use rat_dialog::decorations::WindowFrameState;
    use rat_salsa::Control;
    use rat_widget::file_dialog::FileDialogState;
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Rect;
    use std::path::PathBuf;

    pub struct Moving;

    #[derive(Debug, Default)]
    pub struct MovingState {}

    pub fn render(
        _area: Rect,
        _buf: &mut Buffer,
        _state: &mut MovingState,
        _ctx: &mut Global,
    ) -> Result<(), Error> {
        Ok(())
    }

    pub fn init(_state: &mut MovingState, _ctx: &mut Global) -> Result<(), Error> {
        Ok(())
    }

    pub fn event(_event: &AppEvent, _state: &mut MovingState, _ctx: &mut Global) -> AppResult {
        Ok(Control::Continue)
    }

    pub fn show_moveable_filedlg(ctx: &mut Global) -> Result<Control<AppEvent>, Error> {
        let mut state = FileDlgState {
            window: WindowFrameState::new(),
            filedlg: FileDialogState::new(),
        };
        state.filedlg.open_dialog(PathBuf::from("."))?;
        ctx.dialogs.push(
            file_dlg_moveable::file_dialog_render,
            file_dlg_moveable::file_dialog_event,
            state,
        );

        Ok(Control::Changed)
    }

    pub fn show_fixed_filedlg(ctx: &mut Global) -> Result<Control<AppEvent>, Error> {
        let mut state = FileDialogState::new();
        state.open_dialog(PathBuf::from("."))?;
        ctx.dialogs.push(
            file_dlg_fixed::file_dialog_render,
            file_dlg_fixed::file_dialog_event,
            state,
        );

        Ok(Control::Changed)
    }
}

pub mod file_dlg_fixed {
    use crate::{AppDialogResult, AppEvent, Global};
    use rat_dialog::WindowControl;
    use rat_event::Dialog;
    use rat_salsa::SalsaContext;
    use rat_theme4::WidgetStyle;
    use rat_widget::event::{FileOutcome, HandleEvent, try_flow};
    use rat_widget::file_dialog::{FileDialog, FileDialogState};
    use rat_widget::layout::layout_middle;
    use rat_widget::text::HasScreenCursor;
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::{Constraint, Rect};
    use ratatui_core::widgets::StatefulWidget;
    use std::any::Any;

    pub fn file_dialog_render(area: Rect, buf: &mut Buffer, state: &mut dyn Any, ctx: &mut Global) {
        let state = state
            .downcast_mut::<FileDialogState>()
            .expect("dialog-state");

        let area = layout_middle(
            area,
            Constraint::Percentage(19),
            Constraint::Percentage(19),
            Constraint::Length(2),
            Constraint::Length(2),
        );

        FileDialog::new()
            .styles(ctx.theme.style(WidgetStyle::FILE_DIALOG))
            .render(area, buf, state);

        ctx.set_screen_cursor(state.screen_cursor());
    }

    pub fn file_dialog_event(
        event: &AppEvent,
        state: &mut dyn Any,
        _ctx: &mut Global,
    ) -> AppDialogResult {
        let state = state.downcast_mut::<FileDialogState>().expect("open-state");

        if let AppEvent::Event(event) = event {
            try_flow!(match state.handle(event, Dialog)? {
                FileOutcome::Cancel => {
                    WindowControl::Close(AppEvent::NoOp) //
                }
                FileOutcome::Ok(f) => {
                    WindowControl::Close(AppEvent::Status(0, format!("Open file {:?}", f)))
                }
                r => r.into(),
            });
            Ok(WindowControl::Continue)
        } else {
            Ok(WindowControl::Continue)
        }
    }
}

pub mod file_dlg_moveable {
    use crate::{AppDialogResult, AppEvent, Global};
    use rat_dialog::decorations::{WindowFrame, WindowFrameState};
    use rat_dialog::{WindowControl, WindowFrameOutcome};
    use rat_event::Dialog;
    use rat_salsa::SalsaContext;
    use rat_theme4::{StyleName, WidgetStyle};
    use rat_widget::event::{FileOutcome, HandleEvent, try_flow};
    use rat_widget::file_dialog::{FileDialog, FileDialogState};
    use rat_widget::layout::layout_middle;
    use rat_widget::text::HasScreenCursor;
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::{Constraint, Rect};
    use ratatui_core::style::Style;
    use ratatui_core::widgets::StatefulWidget;
    use std::any::Any;

    pub struct FileDlgState {
        pub window: WindowFrameState,
        pub filedlg: FileDialogState,
    }

    pub fn file_dialog_render(area: Rect, buf: &mut Buffer, state: &mut dyn Any, ctx: &mut Global) {
        let state = state.downcast_mut::<FileDlgState>().expect("dialog-state");

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

        FileDialog::new()
            .styles(ctx.theme.style(WidgetStyle::FILE_DIALOG))
            .render(state.window.area, buf, &mut state.filedlg);

        WindowFrame::new()
            .no_fill()
            .style(ctx.theme.style_style(Style::DIALOG_BASE))
            .drag_style(ctx.theme.style_style(Style::DIALOG_BASE))
            .hover_style(ctx.theme.p.limegreen(1))
            .render(area, buf, &mut state.window);

        ctx.set_screen_cursor(state.filedlg.screen_cursor());
    }

    pub fn file_dialog_event(
        event: &AppEvent,
        state: &mut dyn Any,
        _ctx: &mut Global,
    ) -> AppDialogResult {
        let state = state.downcast_mut::<FileDlgState>().expect("open-state");

        if let AppEvent::Event(event) = event {
            try_flow!(match state.window.handle(event, Dialog) {
                WindowFrameOutcome::ShouldClose => {
                    WindowControl::Close(AppEvent::NoOp)
                }
                r => r.into(),
            });
            try_flow!(match state.filedlg.handle(event, Dialog)? {
                FileOutcome::Cancel => {
                    WindowControl::Close(AppEvent::NoOp) //
                }
                FileOutcome::Ok(f) => {
                    WindowControl::Close(AppEvent::Status(0, format!("Open file {:?}", f)))
                }
                r => r.into(),
            });
            Ok(WindowControl::Continue)
        } else {
            Ok(WindowControl::Continue)
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
