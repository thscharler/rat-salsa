use crate::event::AppEvent;
use anyhow::Error;
use rat_event::try_flow;
use rat_focus::impl_has_focus;
use rat_salsa::poll::{PollCrossterm, PollRendered};
use rat_salsa::{Control, RunConfig, SalsaAppContext, SalsaContext, run_tui};
use rat_theme4::theme::SalsaTheme;
use rat_theme4::{WidgetStyle, create_theme};
use rat_widget::button::{Button, ButtonState};
use rat_widget::event::{ButtonOutcome, HandleEvent, Regular, ct_event};
use rat_widget::focus::FocusBuilder;
use rat_widget::text::HasScreenCursor;
use rat_widget::text_input::{TextInput, TextInputState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::StatefulWidget;

fn main() -> Result<(), Error> {
    let theme = create_theme("Monochrome Dark");
    let mut global = Global::new(theme);

    let mut state = BasicText::default();

    run_tui(
        init,
        render,
        event,
        error,
        &mut global,
        &mut state,
        RunConfig::default()? //
            .poll(PollCrossterm)
            .poll(PollRendered),
    )?;

    if state.accept {
        println!("accepted");
        println!("text1: {}", state.textinput1.text());
        println!("text2: {}", state.textinput2.text());
        println!("text3: {}", state.textinput3.text());
    } else {
        println!("declined");
    }

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
pub mod event {
    use rat_salsa::event::RenderedEvent;

    #[derive(Debug)]
    pub enum AppEvent {
        Event(crossterm::event::Event),
        Rendered,
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
}

#[derive(Debug, Default)]
pub struct BasicText {
    pub accept: bool,
    pub textinput1: TextInputState,
    pub textinput2: TextInputState,
    pub textinput3: TextInputState,
    pub button1: ButtonState,
    pub button2: ButtonState,
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut BasicText,
    ctx: &mut Global,
) -> Result<(), Error> {
    let ll = layout_grid::<3, 8>(
        area,
        Layout::horizontal([
            Constraint::Length(5),
            Constraint::Length(20),
            Constraint::Fill(1),
        ]),
        Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1), // input
            Constraint::Length(1),
            Constraint::Length(1), // input
            Constraint::Length(1),
            Constraint::Length(1), // input
            Constraint::Length(1),
            Constraint::Length(1), // buttons
        ]),
    );

    TextInput::new()
        .styles(ctx.theme.style(WidgetStyle::TEXT))
        .render(ll[1][1], buf, &mut state.textinput1);
    TextInput::new()
        .styles(ctx.theme.style(WidgetStyle::TEXT))
        .render(ll[1][3], buf, &mut state.textinput2);
    TextInput::new()
        .styles(ctx.theme.style(WidgetStyle::TEXT))
        .render(ll[1][5], buf, &mut state.textinput3);

    let lb = layout_half(ll[1][7]);
    Button::new("Accept")
        .styles(ctx.theme.style(WidgetStyle::BUTTON))
        .render(lb.0, buf, &mut state.button1);
    Button::new("Decline")
        .styles(ctx.theme.style(WidgetStyle::BUTTON))
        .render(lb.1, buf, &mut state.button2);

    ctx.set_screen_cursor(
        state
            .textinput1
            .screen_cursor()
            .or(state.textinput2.screen_cursor())
            .or(state.textinput3.screen_cursor()),
    );

    Ok(())
}

fn layout_half(area: Rect) -> (Rect, Rect) {
    (
        Rect::new(
            area.x, //
            area.y,
            area.width / 2 - 1,
            area.height,
        ),
        Rect::new(
            area.x + area.width / 2,
            area.y,
            area.width - area.width / 2,
            area.height,
        ),
    )
}

fn layout_grid<const X: usize, const Y: usize>(
    area: Rect,
    horizontal: Layout,
    vertical: Layout,
) -> [[Rect; Y]; X] {
    let hori = horizontal.split(Rect::new(area.x, 0, area.width, 0));
    let vert = vertical.split(Rect::new(0, area.y, 0, area.height));

    let mut res = [[Rect::default(); Y]; X];
    for x in 0..X {
        let coldata = &mut res[x];
        for y in 0..Y {
            coldata[y].x = hori[x].x;
            coldata[y].width = hori[x].width;
            coldata[y].y = vert[y].y;
            coldata[y].height = vert[y].height;
        }
    }

    res
}

impl_has_focus!(textinput1, textinput2, textinput3, button1, button2 for BasicText);

pub fn init(state: &mut BasicText, ctx: &mut Global) -> Result<(), Error> {
    ctx.set_focus(FocusBuilder::build_for(state));
    ctx.focus().first();
    Ok(())
}

pub fn event(
    event: &AppEvent,
    state: &mut BasicText,
    ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    match event {
        AppEvent::Event(event) => {
            match &event {
                ct_event!(resized) => try_flow!(Control::Changed),
                _ => {}
            };

            ctx.handle_focus(event);

            try_flow!(state.textinput1.handle(event, Regular));
            try_flow!(state.textinput2.handle(event, Regular));
            try_flow!(state.textinput3.handle(event, Regular));

            try_flow!(match state.button1.handle(event, Regular) {
                ButtonOutcome::Pressed => {
                    state.accept = true;
                    Control::Quit
                }
                r => r.into(),
            });
            try_flow!(match state.button2.handle(event, Regular) {
                ButtonOutcome::Pressed => {
                    state.accept = false;
                    Control::Quit
                }
                r => r.into(),
            });
        }
        AppEvent::Rendered => {
            ctx.set_focus(FocusBuilder::rebuild_for(state, ctx.take_focus()));
        }
    }

    Ok(Control::Continue)
}

pub fn error(
    event: Error,
    _state: &mut BasicText,
    _ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    eprintln!("{:?}", event);
    Ok(Control::Changed)
}
