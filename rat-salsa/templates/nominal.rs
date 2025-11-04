use crate::nominal::{Nominal, NominalState};
use anyhow::Error;
use rat_event::try_flow;
use rat_focus::impl_has_focus;
use rat_salsa::event::RenderedEvent;
use rat_salsa::poll::{PollCrossterm, PollRendered, PollTasks, PollTimers};
use rat_salsa::timer::TimeOut;
use rat_salsa::{run_tui, Control, RunConfig, SalsaAppContext, SalsaContext};
use rat_theme4::{create_theme, SalsaTheme, WidgetStyle};
use rat_widget::event::{ct_event, Dialog, HandleEvent};
use rat_widget::focus::FocusBuilder;
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::statusline::StatusLineState;
use rat_widget::statusline_stacked::StatusLineStacked;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{StatefulWidget, Widget};
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let config = Config::default();
    let theme = create_theme("Imperial Dark").expect("theme");
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
pub struct Global {
    // the salsa machinery
    ctx: SalsaAppContext<AppEvent, Error>,

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
    Timer(TimeOut),
    Event(crossterm::event::Event),
    Rendered,
    Message(String),
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
    pub nominal: NominalState,
    pub status: StatusLineState,
    pub error_dlg: MsgDialogState,
}

impl_has_focus!(nominal for Scenery);

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut Scenery,
    ctx: &mut Global,
) -> Result<(), Error> {
    Nominal.render(area, buf, &mut state.nominal, ctx)?;

    let layout = Layout::vertical([
        Constraint::Fill(1), //
        Constraint::Length(1),
    ])
    .split(area);

    // Error
    if state.error_dlg.active() {
        MsgDialog::new()
            .styles(ctx.theme.style(WidgetStyle::MSG_DIALOG))
            .render(layout[0], buf, &mut state.error_dlg);
    }

    // Status
    let status_layout = Layout::horizontal([
        Constraint::Fill(61), //
        Constraint::Fill(39),
    ])
    .split(layout[1]);

    let palette = &ctx.theme.p;
    let status_color_1 = palette
        .normal_contrast(palette.white[0])
        .bg(palette.blue[3]);
    let status_color_2 = palette
        .normal_contrast(palette.white[0])
        .bg(palette.blue[2]);
    let last_render = format!(
        " R({:03}){:>5} ",
        ctx.count(),
        format!("{:.0?}", ctx.last_render())
    )
    .to_string();
    let last_event = format!(" E{:>5} ", format!("{:.0?}", ctx.last_event())).to_string();

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

pub fn init(state: &mut Scenery, ctx: &mut Global) -> Result<(), Error> {
    ctx.set_focus(FocusBuilder::build_for(state));
    state.nominal.init(ctx)?;
    Ok(())
}

pub fn event(
    event: &AppEvent,
    state: &mut Scenery,
    ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    if let AppEvent::Event(event) = event {
        try_flow!(match &event {
            ct_event!(resized) => Control::Changed,
            ct_event!(key press CONTROL-'q') => Control::Quit,
            _ => Control::Continue,
        });

        try_flow!(if state.error_dlg.active() {
            state.error_dlg.handle(event, Dialog).into()
        } else {
            Control::Continue
        });

        ctx.handle_focus(event);
    }

    match event {
        AppEvent::Rendered => try_flow!({
            ctx.set_focus(FocusBuilder::rebuild_for(state, ctx.take_focus()));
            Control::Continue
        }),
        AppEvent::Message(s) => try_flow!({
            state.error_dlg.append(s.as_str());
            Control::Changed
        }),
        _ => {}
    }

    state.nominal.event(event, ctx)
}

pub fn error(
    event: Error,
    state: &mut Scenery,
    _ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    state.error_dlg.append(format!("{:?}", &*event).as_str());
    Ok(Control::Changed)
}

pub mod nominal {
    use crate::{AppEvent, Global};
    use anyhow::Error;
    use rat_salsa::timer::TimerDef;
    use rat_salsa::{Control, SalsaContext};
    use rat_theme4::WidgetStyle;
    use rat_widget::event::{try_flow, HandleEvent, MenuOutcome, Regular};
    use rat_widget::focus::impl_has_focus;
    use rat_widget::menu::{MenuLine, MenuLineState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Layout, Rect};
    use ratatui::widgets::StatefulWidget;
    use std::thread::sleep;
    use std::time::Duration;

    pub struct Nominal;

    #[derive(Debug, Default)]
    pub struct NominalState {
        pub menu: MenuLineState,
    }

    impl Nominal {
        pub fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut NominalState,
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

            MenuLine::new()
                .styles(ctx.theme.style(WidgetStyle::MENU))
                .title("-?-")
                .item_parsed("_Thread")
                .item_parsed("_Timer")
                .item_parsed("_Quit")
                .render(r[1], buf, &mut state.menu);

            Ok(())
        }
    }

    impl_has_focus!(menu for NominalState);

    impl NominalState {
        pub fn init(
            &mut self, //
            ctx: &mut Global,
        ) -> Result<(), Error> {
            ctx.focus().first();
            Ok(())
        }

        #[allow(unused_variables)]
        pub fn event(
            &mut self,
            event: &AppEvent,
            ctx: &mut Global,
        ) -> Result<Control<AppEvent>, Error> {
            if let AppEvent::Event(event) = event {
                match self.menu.handle(event, Regular) {
                    MenuOutcome::Activated(0) => try_flow!({
                        ctx.spawn(|| {
                            sleep(Duration::from_secs(5));
                            Ok(Control::Event(AppEvent::Message(
                                "waiting is over".to_string(),
                            )))
                        })?;
                        Control::Changed
                    }),
                    MenuOutcome::Activated(1) => try_flow!({
                        ctx.add_timer(TimerDef::new().repeat(21).timer(Duration::from_millis(500)));
                        Control::Changed
                    }),
                    MenuOutcome::Activated(2) => try_flow!({
                        Control::Quit //
                    }),
                    v => try_flow!(v),
                }
            }

            match event {
                AppEvent::Timer(t) => {
                    ctx.status = format!("TICK-{}", t.counter);
                    Ok(Control::Changed)
                }
                _ => Ok(Control::Continue),
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
