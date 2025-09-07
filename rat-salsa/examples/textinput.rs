use crate::event::TextinputEvent;
use crate::textinput::{BasicText, BasicTextState};
use anyhow::Error;
use rat_salsa::poll::{PollCrossterm, PollRendered};
use rat_salsa::{run_tui, RunConfig};
use rat_theme2::palettes::MONOCHROME;
use rat_theme2::DarkTheme;

type AppContext<'a> = rat_salsa::AppContext<'a, Global, TextinputEvent, Error>;
type RenderContext<'a> = rat_salsa::RenderContext<'a, Global>;

fn main() -> Result<(), Error> {
    let theme = DarkTheme::new("Monochrome".into(), MONOCHROME);
    let mut global = Global::new(theme);

    let mut state = BasicTextState::default();

    run_tui(
        BasicText,
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

#[derive(Debug)]
pub struct Global {
    pub theme: DarkTheme,
}

impl Global {
    pub fn new(theme: DarkTheme) -> Self {
        Self { theme }
    }
}

/// Application wide messages.
pub mod event {
    use rat_salsa::rendered::RenderedEvent;

    #[derive(Debug)]
    pub enum TextinputEvent {
        Event(crossterm::event::Event),
        Rendered,
    }

    impl From<RenderedEvent> for TextinputEvent {
        fn from(_: RenderedEvent) -> Self {
            Self::Rendered
        }
    }

    impl From<crossterm::event::Event> for TextinputEvent {
        fn from(value: crossterm::event::Event) -> Self {
            Self::Event(value)
        }
    }
}

pub mod textinput {
    use crate::event::TextinputEvent;
    use crate::{AppContext, Global, RenderContext};
    use anyhow::Error;
    use rat_event::try_flow;
    use rat_focus::impl_has_focus;
    use rat_salsa::{AppState, AppWidget, Control};
    use rat_widget::button::{Button, ButtonState};
    use rat_widget::event::{ct_event, ButtonOutcome, HandleEvent, Regular};
    use rat_widget::focus::FocusBuilder;
    use rat_widget::text::HasScreenCursor;
    use rat_widget::text_input::{TextInput, TextInputState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Layout, Rect};
    use ratatui::widgets::StatefulWidget;

    #[derive(Debug)]
    pub struct BasicText;

    #[derive(Debug, Default)]
    pub struct BasicTextState {
        pub accept: bool,
        pub textinput1: TextInputState,
        pub textinput2: TextInputState,
        pub textinput3: TextInputState,
        pub button1: ButtonState,
        pub button2: ButtonState,
    }

    impl AppWidget<Global, TextinputEvent, Error> for BasicText {
        type State = BasicTextState;

        fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut Self::State,
            ctx: &mut RenderContext<'_>,
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

            TextInput::new().styles(ctx.g.theme.text_style()).render(
                ll[1][1],
                buf,
                &mut state.textinput1,
            );
            TextInput::new().styles(ctx.g.theme.text_style()).render(
                ll[1][3],
                buf,
                &mut state.textinput2,
            );
            TextInput::new().styles(ctx.g.theme.text_style()).render(
                ll[1][5],
                buf,
                &mut state.textinput3,
            );

            let lb = layout_half(ll[1][7]);
            Button::new("Accept")
                .styles(ctx.g.theme.button_style())
                .render(lb.0, buf, &mut state.button1);
            Button::new("Decline")
                .styles(ctx.g.theme.button_style())
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

    impl_has_focus!(textinput1, textinput2, textinput3, button1, button2 for BasicTextState);

    impl AppState<Global, TextinputEvent, Error> for BasicTextState {
        fn init(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            ctx.focus = Some(FocusBuilder::build_for(self));
            ctx.focus().first();
            Ok(())
        }

        fn event(
            &mut self,
            event: &TextinputEvent,
            ctx: &mut rat_salsa::AppContext<'_, Global, TextinputEvent, Error>,
        ) -> Result<Control<TextinputEvent>, Error> {
            match event {
                TextinputEvent::Event(event) => {
                    match &event {
                        ct_event!(resized) => try_flow!(Control::Changed),
                        _ => {}
                    };

                    ctx.focus_event(event);

                    try_flow!(self.textinput1.handle(event, Regular));
                    try_flow!(self.textinput2.handle(event, Regular));
                    try_flow!(self.textinput3.handle(event, Regular));

                    try_flow!(match self.button1.handle(event, Regular) {
                        ButtonOutcome::Pressed => {
                            self.accept = true;
                            Control::Quit
                        }
                        r => r.into(),
                    });
                    try_flow!(match self.button2.handle(event, Regular) {
                        ButtonOutcome::Pressed => {
                            self.accept = false;
                            Control::Quit
                        }
                        r => r.into(),
                    });
                }
                TextinputEvent::Rendered => {
                    ctx.focus = Some(FocusBuilder::rebuild_for(self, ctx.focus.take()));
                }
            }

            Ok(Control::Continue)
        }

        fn error(
            &self,
            event: Error,
            _ctx: &mut AppContext<'_>,
        ) -> Result<Control<TextinputEvent>, Error> {
            eprintln!("{:?}", event);
            Ok(Control::Changed)
        }
    }
}
