use anyhow::Error;
use log::error;
use rat_event::try_flow;
use rat_focus::impl_has_focus;
use rat_salsa::dialog_stack::DialogStack;
use rat_salsa::event::RenderedEvent;
use rat_salsa::poll::{PollCrossterm, PollRendered};
use rat_salsa::{Control, RunConfig, SalsaAppContext, SalsaContext, run_tui};
use rat_theme4::palette::Colors;
use rat_theme4::theme::SalsaTheme;
use rat_theme4::{WidgetStyle, create_salsa_theme};
use rat_widget::dialog_frame::{DialogFrame, DialogFrameState, DialogOutcome};
use rat_widget::event::{Dialog, HandleEvent, MenuOutcome, Regular, ct_event};
use rat_widget::focus::FocusBuilder;
use rat_widget::menu::{MenuLine, MenuLineState};
use rat_widget::paragraph::{Paragraph, ParagraphState};
use rat_widget::statusline_stacked::StatusLineStacked;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::text::{Line, Span};
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::Event;
use std::any::Any;
use std::fs;
use try_as_traits::TryAsRef;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let config = Config::default();
    let theme = create_salsa_theme("Imperial Shell");
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
    dlg: DialogStack<AppEvent, Global, Error>,

    pub cfg: Config,
    pub theme: SalsaTheme,
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
    pub fn new(cfg: Config, theme: SalsaTheme) -> Self {
        Self {
            ctx: Default::default(),
            dlg: Default::default(),
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
    NoOp,
    Event(Event),
    Rendered,
    Message(String),
}

impl From<RenderedEvent> for AppEvent {
    fn from(_: RenderedEvent) -> Self {
        Self::Rendered
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
pub struct Minimal {
    pub menu: MenuLineState,
    // pub error_dlg: MsgDialogState,
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
        .styles(ctx.theme.style(WidgetStyle::MENU))
        .title("-!-")
        .item_parsed("_Dialog")
        .item_parsed("_Quit")
        .render(status_layout[0], buf, &mut state.menu);

    ctx.dlg.clone().render(layout[0], buf, ctx);

    // Status
    let palette = &ctx.theme.p;
    let status_color_1 = palette.fg_bg_style(Colors::White, 0, Colors::Blue, 3);
    let status_color_2 = palette.fg_bg_style(Colors::White, 0, Colors::Blue, 2);
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
    }

    try_flow!(ctx.dlg.clone().handle(event, ctx)?);

    if let AppEvent::Event(event) = event {
        ctx.handle_focus(event);

        try_flow!(match state.menu.handle(event, Regular) {
            MenuOutcome::Activated(0) => Control::Event(AppEvent::Message("message!".to_string())),
            MenuOutcome::Activated(1) => Control::Quit,
            v => v.into(),
        });
    }

    match event {
        AppEvent::Rendered => {
            ctx.set_focus(FocusBuilder::rebuild_for(state, ctx.take_focus()));
            Ok(Control::Continue)
        }
        AppEvent::Message(s) => {
            show_error(s, ctx);
            Ok(Control::Changed)
        }
        _ => Ok(Control::Continue),
    }
}

pub fn error(
    event: Error,
    _state: &mut Minimal,
    ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    error!("{:?}", event);
    show_error(format!("{:?}", &*event).as_str(), ctx);
    Ok(Control::Changed)
}

struct MsgState {
    pub dlg: DialogFrameState,
    pub paragraph: ParagraphState,
    pub message: String,
}

impl_has_focus!(dlg, paragraph for MsgState);

fn msg_render(area: Rect, buf: &mut Buffer, state: &mut dyn Any, ctx: &mut Global) {
    let state = state.downcast_mut::<MsgState>().expect("msg-dialog");

    DialogFrame::new()
        .styles(ctx.theme.style(WidgetStyle::DIALOG_FRAME))
        .no_cancel()
        .left(Constraint::Percentage(19))
        .right(Constraint::Percentage(19))
        .top(Constraint::Length(4))
        .bottom(Constraint::Length(4))
        .render(area, buf, &mut state.dlg);

    Paragraph::new(state.message.as_str())
        .styles(ctx.theme.style(WidgetStyle::PARAGRAPH))
        .render(state.dlg.widget_area, buf, &mut state.paragraph);
}

fn msg_event(
    event: &AppEvent,
    state: &mut dyn Any,
    ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    let state = state.downcast_mut::<MsgState>().expect("msg-dialog");

    if let AppEvent::Event(e) = event {
        let mut focus = FocusBuilder::build_for(state);
        ctx.queue(focus.handle(e, Regular));

        try_flow!(state.paragraph.handle(e, Regular));
        try_flow!(match state.dlg.handle(e, Dialog) {
            DialogOutcome::Ok => {
                Control::Close(AppEvent::NoOp)
            }
            DialogOutcome::Cancel => {
                Control::Close(AppEvent::NoOp)
            }
            r => r.into(),
        });
        Ok(Control::Continue)
    } else {
        Ok(Control::Continue)
    }
}

pub fn show_error(txt: &str, ctx: &mut Global) {
    for i in 0..ctx.dlg.len() {
        if ctx.dlg.state_is::<MsgState>(i) {
            let mut msg = ctx.dlg.get_mut::<MsgState>(i).expect("msg-dialog");
            msg.message.push_str(txt);
            return;
        }
    }

    ctx.dlg.push(
        msg_render,
        msg_event,
        MsgState {
            dlg: Default::default(),
            paragraph: Default::default(),
            message: txt.to_string(),
        },
    );
}

fn setup_logging() -> Result<(), Error> {
    // let log_path = PathBuf::from("../..");
    let log_file = "log.log";
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
