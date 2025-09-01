use crate::scenery::Scenery;
use anyhow::Error;
use rat_salsa2::poll::{PollCrossterm, PollRendered, PollTasks, PollTimers};
use rat_salsa2::rendered::RenderedEvent;
use rat_salsa2::timer::TimeOut;
use rat_salsa2::{run_tui, RunConfig};
use rat_theme2::palettes::IMPERIAL;
use rat_theme2::DarkTheme;
use std::fs;

type AppContext<'a> = rat_salsa2::AppContext<'a, GlobalState, MinimalEvent, Error>;
type RenderContext<'a> = rat_salsa2::RenderContext<'a, GlobalState>;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let config = MinimalConfig::default();
    let theme = DarkTheme::new("Imperial".into(), IMPERIAL);
    let mut global = GlobalState::new(config, theme);

    let mut state = Scenery::default();

    run_tui(
        scenery::init,
        scenery::render,
        scenery::event,
        scenery::error,
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
pub struct GlobalState {
    pub cfg: MinimalConfig,
    pub theme: DarkTheme,
}

impl GlobalState {
    pub fn new(cfg: MinimalConfig, theme: DarkTheme) -> Self {
        Self { cfg, theme }
    }
}

/// Configuration.
#[derive(Debug, Default)]
pub struct MinimalConfig {}

/// Application wide messages.

#[derive(Debug)]
pub enum MinimalEvent {
    Timer(TimeOut),
    Event(crossterm::event::Event),
    Rendered,
    Message(String),
    Status(usize, String),
}

impl From<RenderedEvent> for MinimalEvent {
    fn from(_: RenderedEvent) -> Self {
        Self::Rendered
    }
}

impl From<TimeOut> for MinimalEvent {
    fn from(value: TimeOut) -> Self {
        Self::Timer(value)
    }
}

impl From<crossterm::event::Event> for MinimalEvent {
    fn from(value: crossterm::event::Event) -> Self {
        Self::Event(value)
    }
}

pub mod scenery {
    use crate::minimal::Minimal;
    use crate::{minimal, AppContext, MinimalEvent, RenderContext};
    use anyhow::Error;
    use rat_salsa2::Control;
    use rat_widget::event::{ct_event, ConsumedEvent, Dialog, HandleEvent, Regular};
    use rat_widget::focus::FocusBuilder;
    use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
    use rat_widget::statusline::{StatusLine, StatusLineState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Layout, Rect};
    use ratatui::widgets::StatefulWidget;
    use std::time::{Duration, SystemTime};

    #[derive(Debug, Default)]
    pub struct Scenery {
        pub minimal: Minimal,
        pub status: StatusLineState,
        pub error_dlg: MsgDialogState,
    }

    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        state: &mut Scenery,
        ctx: &mut RenderContext<'_>,
    ) -> Result<(), Error> {
        let t0 = SystemTime::now();

        let layout = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

        minimal::render(area, buf, &mut state.minimal, ctx)?;

        if state.error_dlg.active() {
            let err = MsgDialog::new().styles(ctx.g.theme.msg_dialog_style());
            err.render(layout[0], buf, &mut state.error_dlg);
        }

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        state.status.status(1, format!("R {:.0?}", el).to_string());

        let status_layout =
            Layout::horizontal([Constraint::Fill(61), Constraint::Fill(39)]).split(layout[1]);
        let status = StatusLine::new()
            .layout([
                Constraint::Fill(1),
                Constraint::Length(8),
                Constraint::Length(8),
            ])
            .styles(ctx.g.theme.statusline_style());
        status.render(status_layout[1], buf, &mut state.status);

        Ok(())
    }

    pub fn init(state: &mut Scenery, ctx: &mut AppContext<'_>) -> Result<(), Error> {
        ctx.focus = Some(FocusBuilder::build_for(&state.minimal));
        minimal::init(&mut state.minimal, ctx)?;
        Ok(())
    }

    pub fn event(
        event: &MinimalEvent,
        state: &mut Scenery,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<MinimalEvent>, Error> {
        let t0 = SystemTime::now();

        let mut r = match event {
            MinimalEvent::Event(event) => {
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
            MinimalEvent::Rendered => {
                ctx.focus = Some(FocusBuilder::rebuild_for(&state.minimal, ctx.focus.take()));
                Control::Continue
            }
            MinimalEvent::Message(s) => {
                state.error_dlg.append(s.as_str());
                Control::Changed
            }
            MinimalEvent::Status(n, s) => {
                state.status.status(*n, s);
                Control::Changed
            }
            _ => Control::Continue,
        };

        r = r.or_else_try(|| minimal::event(event, &mut state.minimal, ctx))?;

        let el = t0.elapsed()?;
        state.status.status(2, format!("E {:.0?}", el).to_string());

        Ok(r)
    }

    pub fn error(
        event: Error,
        state: &mut Scenery,
        _ctx: &mut AppContext<'_>,
    ) -> Result<Control<MinimalEvent>, Error> {
        state.error_dlg.append(format!("{:?}", &*event).as_str());
        Ok(Control::Changed)
    }
}

pub mod minimal {
    use crate::{AppContext, MinimalEvent, RenderContext};
    use anyhow::Error;
    use rat_salsa2::Control;
    use rat_widget::event::{try_flow, HandleEvent, MenuOutcome, Regular};
    use rat_widget::focus::impl_has_focus;
    use rat_widget::menu::{MenuLine, MenuLineState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Layout, Rect};
    use ratatui::widgets::StatefulWidget;

    #[derive(Debug, Default)]
    pub struct Minimal {
        pub menu: MenuLineState,
    }

    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        state: &mut Minimal,
        ctx: &mut RenderContext<'_>,
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

        let menu = MenuLine::new()
            .styles(ctx.g.theme.menu_style())
            .item_parsed("_Quit");
        menu.render(r[1], buf, &mut state.menu);

        Ok(())
    }

    impl_has_focus!(menu for Minimal);

    pub fn init(_state: &mut Minimal, ctx: &mut AppContext<'_>) -> Result<(), Error> {
        ctx.focus().first();
        Ok(())
    }

    #[allow(unused_variables)]
    pub fn event(
        event: &MinimalEvent,
        state: &mut Minimal,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<MinimalEvent>, Error> {
        match event {
            MinimalEvent::Event(event) => {
                try_flow!(match state.menu.handle(event, Regular) {
                    MenuOutcome::Activated(0) => Control::Quit,
                    v => v.into(),
                });
                Ok(Control::Continue)
            }
            _ => Ok(Control::Continue),
        }
    }
}

fn setup_logging() -> Result<(), Error> {
    if let Some(cache) = dirs::cache_dir() {
        let log_path = cache.join("rat-salsa");
        if !log_path.exists() {
            fs::create_dir_all(&log_path)?;
        }

        let log_file = log_path.join("minimal.log");
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
