use crate::event::AppEvent;
use crate::three_inputs::{ThreeInputs, ThreeInputsState};
use anyhow::Error;
use rat_event::try_flow;
use rat_focus::impl_has_focus;
use rat_salsa::poll::{PollCrossterm, PollRendered};
use rat_salsa::{Control, RunConfig, SalsaAppContext, SalsaContext, run_tui};
use rat_theme4::{SalsaTheme, WidgetStyle, create_theme};
use rat_widget::button::{Button, ButtonState};
use rat_widget::event::{ButtonOutcome, HandleEvent, Regular, ct_event};
use rat_widget::focus::FocusBuilder;
use rat_widget::text::{HasScreenCursor, impl_screen_cursor};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{Block, StatefulWidget};

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

    pub three: ThreeInputsState,

    pub button1: ButtonState,
    pub button2: ButtonState,
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut BasicText,
    ctx: &mut Global,
) -> Result<(), Error> {
    let ll = layout_grid::<3, 4>(
        area,
        Layout::horizontal([
            Constraint::Length(5),
            Constraint::Length(60),
            Constraint::Fill(1),
        ]),
        Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3), // input
            Constraint::Length(1),
            Constraint::Length(1), // buttons
        ]),
    );

    ThreeInputs::new()
        .text_style(ctx.theme.style(WidgetStyle::TEXT))
        .block(Block::bordered())
        .render(ll[1][1], buf, &mut state.three);

    let lb = layout_half(ll[1][3]);
    Button::new("Accept")
        .styles(ctx.theme.style(WidgetStyle::BUTTON))
        .render(lb.0, buf, &mut state.button1);
    Button::new("Decline")
        .styles(ctx.theme.style(WidgetStyle::BUTTON))
        .render(lb.1, buf, &mut state.button2);

    // text cursor
    ctx.set_screen_cursor(state.three.screen_cursor());

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

impl_has_focus!(three, button1, button2 for BasicText);
impl_screen_cursor!(three, button1, button2 for BasicText);

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
                ct_event!(key press CONTROL-'q') => try_flow!(Control::Quit),
                _ => {}
            };

            ctx.handle_focus(event);

            try_flow!(state.three.handle(event, Regular));

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

#[allow(dead_code)]
mod three_inputs {
    use crossterm::event::Event;
    use rat_event::{HandleEvent, MouseOnly, Regular, flow};
    use rat_focus::impl_has_focus;
    use rat_widget::event::TextOutcome;
    use rat_widget::reloc::impl_relocatable_state;
    use rat_widget::text::{TextStyle, impl_screen_cursor};
    use rat_widget::text_input::{TextInput, TextInputState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Layout, Rect};
    use ratatui::prelude::BlockExt;
    use ratatui::style::Style;
    use ratatui::widgets::{Block, StatefulWidget, Widget};

    #[derive(Debug, Default)]
    pub struct ThreeInputs<'a> {
        style: Style,
        block: Option<Block<'a>>,
        text_style: TextStyle,
    }

    #[derive(Debug, Default)]
    pub struct ThreeInputsState {
        pub area: Rect,
        pub inner: Rect,
        pub input1: TextInputState,
        pub input2: TextInputState,
        pub input3: TextInputState,
    }

    impl<'a> ThreeInputs<'a> {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn text_style(mut self, text_style: TextStyle) -> Self {
            self.text_style = text_style;
            self
        }

        pub fn style(mut self, style: Style) -> Self {
            self.style = style;
            self
        }

        pub fn block(mut self, block: Block<'a>) -> Self {
            self.block = Some(block);
            self
        }
    }

    impl<'a> StatefulWidget for ThreeInputs<'a> {
        type State = ThreeInputsState;

        fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
            state.area = area;
            state.inner = self.block.inner_if_some(area);

            let l = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ])
            .spacing(1)
            .split(state.inner);

            if let Some(block) = self.block {
                block.render(area, buf);
            } else {
                buf.set_style(area, self.style);
            }

            TextInput::new()
                .styles(self.text_style.clone())
                .render(l[0], buf, &mut state.input1);
            TextInput::new()
                .styles(self.text_style.clone())
                .render(l[1], buf, &mut state.input2);
            TextInput::new()
                .styles(self.text_style.clone())
                .render(l[2], buf, &mut state.input3);
        }
    }

    impl_has_focus!(input1, input2, input3 for ThreeInputsState);
    impl_screen_cursor!(input1, input2, input3 for ThreeInputsState);
    impl_relocatable_state!(input1, input2, input3 for ThreeInputsState);

    impl ThreeInputsState {
        pub fn new() -> Self {
            Self::default()
        }

        // ... more ...
    }

    impl HandleEvent<Event, Regular, TextOutcome> for ThreeInputsState {
        fn handle(&mut self, event: &Event, _qualifier: Regular) -> TextOutcome {
            flow!(self.input1.handle(event, Regular));
            flow!(self.input2.handle(event, Regular));
            flow!(self.input3.handle(event, Regular));
            // no need to forward to our mousehandler here.
            TextOutcome::Continue
        }
    }

    impl HandleEvent<Event, MouseOnly, TextOutcome> for ThreeInputsState {
        fn handle(&mut self, event: &Event, _qualifier: MouseOnly) -> TextOutcome {
            flow!(self.input1.handle(event, MouseOnly));
            flow!(self.input2.handle(event, MouseOnly));
            flow!(self.input3.handle(event, MouseOnly));
            TextOutcome::Continue
        }
    }
}

pub fn error(
    event: Error,
    _state: &mut BasicText,
    _ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    eprintln!("{:?}", event);
    Ok(Control::Changed)
}
