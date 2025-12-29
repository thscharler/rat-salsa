///
/// Example for inline text-prompts.
/// Uses a bit of a state-machine to model consecutive prompts.
///
use anyhow::Error;
use log::error;
use rat_event::event_flow;
use rat_salsa::poll::PollCrossterm;
use rat_salsa::{Control, RunConfig, SalsaAppContext, SalsaContext, TermInit, run_tui};
use rat_theme4::palette::Colors;
use rat_theme4::theme::SalsaTheme;
use rat_theme4::{WidgetStyle, create_salsa_theme};
use rat_widget::event::{HandleEvent, Regular, ct_event};
use rat_widget::text::cursor::{CursorType, set_cursor_type};
use rat_widget::text_input::{TextInput, TextInputState};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::{Style, Stylize};
use ratatui_core::text::{Span, Text};
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::Event;
use rat_widget::text::clipboard::cli::setup_cli_clipboard;

fn main() -> Result<(), Error> {
    setup_logging()?;

    set_cursor_type(CursorType::RenderedCursor); // use rendered cursor for text-input.
    setup_cli_clipboard();

    let theme = create_salsa_theme("SunriseBreeze Light");
    let mut global = Global::new(theme);
    let mut state = Scenery::new();

    run_tui(
        init,
        render,
        event,
        error,
        &mut global,
        &mut state,
        RunConfig::inline(1, true)?
            .term_init(TermInit {
                raw_mode: false,
                alternate_screen: false,
                mouse_capture: true,
                bracketed_paste: true,
                clear_area: true,
                ..Default::default()
            })
            .poll(PollCrossterm),
    )?;

    Ok(())
}

/// Globally accessible data/state.
pub struct Global {
    ctx: SalsaAppContext<AppEvent, Error>,
    pub theme: SalsaTheme,
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
        }
    }
}

/// Application wide messages.

#[derive(Debug)]
pub enum AppEvent {
    NoOp,
    Event(Event),
}

impl From<Event> for AppEvent {
    fn from(value: Event) -> Self {
        Self::Event(value)
    }
}

#[derive(Debug)]
enum InputState {
    Username { text: TextInputState },
    Passwd { text: TextInputState },
    Done,
}

#[derive(Debug)]
struct Scenery {
    pub username: String,
    pub passwd: String,

    pub state: InputState,
}

impl Scenery {
    fn new() -> Self {
        Self {
            username: Default::default(),
            passwd: Default::default(),
            state: InputState::Username {
                text: TextInputState::new_focused(),
            },
        }
    }
}

fn init(_state: &mut Scenery, ctx: &mut Global) -> Result<(), Error> {
    let style = ctx.theme.p.style(Colors::Orange, 0).bold();
    ctx.insert_before(2, move |buf| {
        Text::from_iter([
            Span::from("WELCOME TO WEYLAND-YUTANI MESSAGE BOARD.").style(style),
            Span::from("ENTER CREDENTIALS NOW").style(style),
        ])
        .render(buf.area, buf);
    });
    Ok(())
}

fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut Scenery,
    ctx: &mut Global,
) -> Result<(), Error> {
    match &mut state.state {
        InputState::Username { text } => {
            Span::from("username: ").render(area, buf);
            let txt_area = Rect::new(area.x + 10, area.y, area.width, area.height);
            TextInput::new()
                .styles(ctx.theme.style(WidgetStyle::TEXT))
                .render(txt_area, buf, text);
        }
        InputState::Passwd { text } => {
            Span::from("password: ").render(area, buf);
            let txt_area = Rect::new(area.x + 10, area.y, area.width, area.height);
            TextInput::new()
                .styles(ctx.theme.style(WidgetStyle::TEXT))
                .passwd()
                .render(txt_area, buf, text);
        }
        InputState::Done => {}
    }

    Ok(())
}

fn event(
    event: &AppEvent,
    state: &mut Scenery,
    ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    if let AppEvent::Event(event) = event {
        if let ct_event!(resized) = event {
            event_flow!(Control::Changed)
        }

        match event {
            ct_event!(keycode press Enter) => match &mut state.state {
                InputState::Username { text } => event_flow!({
                    state.username = text.value();

                    let username = state.username.clone();
                    ctx.insert_before(1, move |buf| {
                        buf.set_style(buf.area, Style::new().italic());
                        Span::from(format!("hello {}, now your password ...", username))
                            .render(buf.area, buf);
                    });

                    state.state = InputState::Passwd {
                        text: TextInputState::new_focused(),
                    };

                    Control::Changed
                }),
                InputState::Passwd { text } => event_flow!({
                    state.passwd = text.value();

                    if state.username == "peter" {
                        ctx.insert_before(3, |buf| {
                            buf.set_style(buf.area, Style::new().italic());
                            Text::from_iter([
                                "password accepted ...",
                                "* there are no new messages for project xenomorph",
                                "-> quit now",
                            ])
                            .render(buf.area, buf);
                        });
                    } else {
                        ctx.insert_before(3, |buf| {
                            buf.set_style(buf.area, Style::new().italic());
                            Text::from_iter([
                                Span::from("wrong password  ...").red(),
                                Span::from("* there are no new messages for you. btw who are you?"),
                                Span::from("-> inform site security"),
                            ])
                            .render(buf.area, buf);
                        });
                    }

                    state.state = InputState::Done;

                    ctx.queue(Control::Changed);
                    Control::Quit
                }),
                InputState::Done => {}
            },
            _ => {}
        }

        match &mut state.state {
            InputState::Username { text } => {
                event_flow!(text.handle(event, Regular));
            }
            InputState::Passwd { text } => {
                event_flow!(text.handle(event, Regular));
            }
            InputState::Done => {}
        }
    }

    Ok(Control::Continue)
}

fn error(
    event: Error,
    _state: &mut Scenery,
    _ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    error!("{:#?}", event);
    Ok(Control::Changed)
}

fn setup_logging() -> Result<(), Error> {
    fern::Dispatch::new()
        .format(|out, message, _record| {
            out.finish(format_args!("{}", message)) //
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stderr())
        .apply()?;
    Ok(())
}
