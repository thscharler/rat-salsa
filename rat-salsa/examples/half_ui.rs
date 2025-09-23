///
/// Inline rendering in the console.
/// Uses only 10 lines for the UI.
///
use anyhow::Error;
use rat_focus::impl_has_focus;
use rat_salsa::event::RenderedEvent;
use rat_salsa::poll::{PollCrossterm, PollRendered};
use rat_salsa::terminal::{CrosstermTerminal, SalsaOptions};
use rat_salsa::timer::TimeOut;
use rat_salsa::{run_tui, Control, RunConfig, SalsaAppContext, SalsaContext};
use rat_theme3::{create_theme, SalsaTheme};
use rat_widget::event::{ct_event, ConsumedEvent, Dialog, HandleEvent, MenuOutcome, Regular};
use rat_widget::focus::FocusBuilder;
use rat_widget::menu::{MenuLine, MenuLineState};
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::statusline::{StatusLine, StatusLineState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::Widget;
use ratatui::text::{Line, Text};
use ratatui::widgets::StatefulWidget;
use ratatui::{TerminalOptions, Viewport};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

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
        RunConfig::new(CrosstermTerminal::with_options(SalsaOptions {
            alternate_screen: false,
            shutdown_clear: true,
            ratatui_options: TerminalOptions {
                viewport: Viewport::Inline(4),
            },
            ..Default::default()
        })?)
        .poll(PollCrossterm)
        .poll(PollRendered),
    )?;

    Ok(())
}

/// Globally accessible data/state.
pub struct Global {
    ctx: SalsaAppContext<AppEvent, Error>,

    pub cfg: Config,
    pub theme: Box<dyn SalsaTheme>,
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
    pub menu: MenuLineState,
    pub status: StatusLineState,
    pub error_dlg: MsgDialogState,
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut Scenery,
    ctx: &mut Global,
) -> Result<(), Error> {
    let layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1), //
    ])
    .split(area);

    if state.error_dlg.active() {
        MsgDialog::new().render(layout[1], buf, &mut state.error_dlg);
    }

    Text::from_iter([
        Line::from("ui ui ui ui "),
        Line::from("ui ui ui ui "),
        Line::from("ui ui ui ui "),
    ])
    .style(ctx.theme.container_base())
    .render(layout[1], buf);

    MenuLine::new()
        .item_parsed("_First")
        .item_parsed("_Second")
        .item_parsed("_Third")
        .item_parsed("_Quit")
        .styles(ctx.theme.menu_style())
        .render(layout[0], buf, &mut state.menu);

    let status_layout = Layout::horizontal([
        Constraint::Fill(61), //
        Constraint::Fill(39),
    ])
    .split(layout[0]);

    StatusLine::new()
        .layout([
            Constraint::Fill(1),
            Constraint::Length(8),
            Constraint::Length(8),
        ])
        .render(status_layout[1], buf, &mut state.status);

    Ok(())
}

impl_has_focus!(menu for Scenery);

pub fn init(state: &mut Scenery, ctx: &mut Global) -> Result<(), Error> {
    ctx.set_focus(FocusBuilder::build_for(state));
    ctx.focus().first();
    Ok(())
}

pub fn event(
    event: &AppEvent,
    state: &mut Scenery,
    ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    let t0 = SystemTime::now();

    let r = match event {
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

            ctx.handle_focus(event);

            r = r.or_else_try(|| match state.menu.handle(event, Regular) {
                MenuOutcome::Activated(0) => dump_raw(TEXT1.into(), ctx),
                MenuOutcome::Activated(1) => dump_raw(TEXT2.into(), ctx),
                MenuOutcome::Activated(2) => dump_raw(TEXT3.into(), ctx),
                MenuOutcome::Activated(3) => Ok(Control::Quit),
                v => Ok(v.into()),
            })?;

            r
        }
        AppEvent::Rendered => {
            ctx.set_focus(FocusBuilder::rebuild_for(state, ctx.take_focus()));
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

    let el = t0.elapsed()?;
    state.status.status(2, format!("E {:.0?}", el).to_string());

    Ok(r)
}

fn dump_raw(p0: String, ctx: &mut Global) -> Result<Control<AppEvent>, Error> {
    let n_lines = p0.split('\n').count() as u16;
    ctx.insert_before(n_lines, move |buf| {
        Text::from(p0).render(buf.area, buf);
    });
    Ok(Control::Changed)
}

static TEXT3: &str = r#"
more bla
bla
bla
bla
bla
"#;

static TEXT2: &str = r#"
...
...
...
...
"#;

static TEXT1: &str = r#"
    |                                                                 ^^^ variant or associated item not found in `StackControl<_>`

error[E0599]: no variant or associated item named `Pop` found for enum `StackControl` in the current scope
   --> rat-salsa\rat-dialog\examples\turbo2.rs:669:46
    |
669 | ...                   Ok(StackControl::Pop)
    |                                        ^^^ variant or associated item not found in `StackControl<_>`

error[E0599]: no variant or associated item named `Pop` found for enum `StackControl` in the current scope
   --> rat-salsa\rat-dialog\examples\turbo2.rs:708:65
    |
708 |                         FileOutcome::Cancel => Ok(StackControl::Pop),
    |                                                                 ^^^ variant or associated item not found in `StackControl<_>`

error[E0599]: no variant or associated item named `Pop` found for enum `StackControl` in the current scope
   --> rat-salsa\rat-dialog\examples\turbo2.rs:711:46
    |
711 | ...                   Ok(StackControl::Pop)
    |                                        ^^^ variant or associated item not found in `StackControl<_>`

For more information about this error, try `rustc --explain E0599`.
error: could not compile `rat-dialog` (example "turbo2") due to 7 previous errors
   Compiling rat-dialog v0.3.0 (C:\Users\stommy\Documents\Workspaces\biosys\rat-salsa\rat-dialog)
warning: unused variable: `ctx`
   --> rat-salsa\rat-dialog\examples\window.rs:386:32
    |
386 |                 |event, state, ctx| {
    |                                ^^^ help: if this is intentional, prefix it with an underscore: `_ctx`
    |
    = note: `#[warn(unused_variables)]` on by default

warning: `rat-dialog` (example "window") generated 1 warning
    Finished `release` profile [optimized] target(s) in 2.27s
     Running `D:\target\release\examples\window.exe`

C:\Users\stommy\Documents\Workspaces\biosys>
    "#;

pub fn error(
    event: Error,
    state: &mut Scenery,
    _ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    state.error_dlg.append(format!("{:?}", &*event).as_str());
    Ok(Control::Changed)
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
