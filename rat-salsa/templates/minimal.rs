use anyhow::Error;
use log::error;
use rat_event::try_flow;
use rat_focus::impl_has_focus;
use rat_salsa::event::RenderedEvent;
use rat_salsa::poll::{PollCrossterm, PollRendered};
use rat_salsa::{run_tui, Control, RunConfig, SalsaAppContext, SalsaContext};
use rat_theme3::{create_theme, SalsaTheme};
use rat_widget::event::{ct_event, Dialog, HandleEvent, MenuOutcome, Regular};
use rat_widget::focus::FocusBuilder;
use rat_widget::menu::{MenuLine, MenuLineState};
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::statusline::{StatusLine, StatusLineState};
use rat_widget::statusline_stacked::StatusLineStacked;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{StatefulWidget, Widget};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

fn main() -> Result<(), Error> {
    setup_logging()?;

    let config = Config::default();
    let theme = create_theme("Imperial Shell").expect("theme");
    let mut global = Global::new(config, theme);
    let mut state = Minimal::default();

    run_tui(
        init,
        render,
        event,
        error,
        &mut global,
        &mut state,
        RunConfig::default()?
            .poll(PollCrossterm) //
            .poll(PollRendered),
    )?;

    Ok(())
}

/// Globally accessible data/state.
pub struct Global {
    // the salsa machinery
    ctx: SalsaAppContext<AppEvent, Error>,

    pub cfg: Config,
    pub theme: Box<dyn SalsaTheme>,
    pub status: String,
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
    pub fn new(cfg: Config, theme: Box<dyn SalsaTheme>) -> Self {
        Self {
            ctx: Default::default(),
            cfg,
            theme,
            status: Default::default(),
        }
    }
}

/// Configuration.
#[derive(Debug, Default)]
pub struct Config {}

/// Application wide messages.
#[derive(Debug)]
pub enum AppEvent {
    Event(crossterm::event::Event),
    Rendered,
    Message(String),
}

impl From<RenderedEvent> for AppEvent {
    fn from(_: RenderedEvent) -> Self {
        Self::Rendered
    }
}

impl From<crossterm::event::Event> for AppEvent {
    fn from(value: crossterm::event::Event) -> Self {
        Self::Event(value)
    }
}

#[derive(Debug, Default)]
pub struct Minimal {
    pub menu: MenuLineState,

    pub error_dlg: MsgDialogState,
}

impl_has_focus!(menu for Minimal);

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut Minimal,
    ctx: &mut Global,
) -> Result<(), Error> {
    let layout = Layout::vertical([
        Constraint::Fill(1), //
        Constraint::Length(1),
    ])
    .split(area);

    let status_layout = Layout::horizontal([
        Constraint::Fill(61), //
        Constraint::Fill(39),
    ])
    .split(layout[1]);

    MenuLine::new()
        .styles(ctx.theme.menu_style())
        .title("-!-")
        .item_parsed("_Quit")
        .render(status_layout[0], buf, &mut state.menu);

    if state.error_dlg.active() {
        MsgDialog::new()
            .styles(ctx.theme.msg_dialog_style())
            .render(layout[0], buf, &mut state.error_dlg);
    }

    // Status
    let palette = ctx.theme.palette();
    let status_color_1 = palette
        .normal_contrast(palette.white[0])
        .bg(palette.blue[3]);
    let status_color_2 = palette
        .normal_contrast(palette.white[0])
        .bg(palette.blue[2]);
    let last_render = format!(
        " R({:03}){:05} ",
        ctx.count(),
        format!("{:.0?}", ctx.last_render())
    )
    .to_string();
    let last_event = format!(" E{:05} ", format!("{:.0?}", ctx.last_event())).to_string();

    StatusLineStacked::new()
        .center_margin(1)
        .center(Line::from(ctx.status.as_str()))
        .end(
            Span::from(last_render).style(status_color_1),
            Span::from(" "),
        )
        .end_bare(Span::from(last_event).style(status_color_2))
        .render(status_layout[1], buf);

    Ok(())
}

pub fn init(state: &mut Minimal, ctx: &mut Global) -> Result<(), Error> {
    ctx.set_focus(FocusBuilder::build_for(state));
    ctx.focus().first();
    Ok(())
}

pub fn event(
    event: &AppEvent,
    state: &mut Minimal,
    ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    if let AppEvent::Event(event) = event {
        try_flow!(match &event {
            ct_event!(resized) => Control::Changed,
            ct_event!(key press CONTROL-'q') => Control::Quit,
            _ => Control::Continue,
        });

        try_flow!({
            if state.error_dlg.active() {
                state.error_dlg.handle(event, Dialog).into()
            } else {
                Control::Continue
            }
        });

        ctx.handle_focus(event);

        try_flow!(match state.menu.handle(event, Regular) {
            MenuOutcome::Activated(0) => Control::Quit,
            v => v.into(),
        });
    }

    match event {
        AppEvent::Rendered => {
            ctx.set_focus(FocusBuilder::rebuild_for(state, ctx.take_focus()));
            Ok(Control::Continue)
        }
        AppEvent::Message(s) => {
            state.error_dlg.append(s.as_str());
            Ok(Control::Changed)
        }
        _ => Ok(Control::Continue),
    }
}

pub fn error(
    event: Error,
    state: &mut Minimal,
    _ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    error!("{:?}", event);
    state.error_dlg.append(format!("{:?}", &*event).as_str());
    Ok(Control::Changed)
}

fn setup_logging() -> Result<(), Error> {
    let log_path = PathBuf::from("../..");
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
