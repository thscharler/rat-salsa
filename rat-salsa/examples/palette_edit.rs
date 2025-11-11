use crate::palette_edit::PaletteEdit;
use anyhow::Error;
use log::{error, warn};
use rat_event::{try_flow, Outcome, Popup};
use rat_focus::{impl_has_focus, FocusFlag, HasFocus};
use rat_salsa::dialog_stack::DialogStack;
use rat_salsa::event::RenderedEvent;
use rat_salsa::poll::{PollCrossterm, PollRendered};
use rat_salsa::{run_tui, Control, RunConfig, SalsaAppContext, SalsaContext};
use rat_theme4::{create_palette, create_theme, salsa_palettes, Palette, SalsaTheme, WidgetStyle};
use rat_widget::choice::{Choice, ChoiceState};
use rat_widget::color_input::{ColorInput, ColorInputState};
use rat_widget::dialog_frame::{DialogFrame, DialogFrameState, DialogOutcome};
use rat_widget::event::{ct_event, ChoiceOutcome, Dialog, HandleEvent, MenuOutcome, Regular};
use rat_widget::focus::FocusBuilder;
use rat_widget::menu::{MenuLine, MenuLineState};
use rat_widget::paragraph::{Paragraph, ParagraphState};
use rat_widget::reloc::RelocatableState;
use rat_widget::statusline_stacked::StatusLineStacked;
use rat_widget::text::clipboard::{set_global_clipboard, Clipboard, ClipboardError};
use rat_widget::text::HasScreenCursor;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{StatefulWidget, Widget};
use std::any::Any;
use std::cell::RefCell;
use std::fs;
use std::iter::once;
use try_as::traits::TryAsRef;

fn main() -> Result<(), Error> {
    setup_logging()?;
    set_global_clipboard(CliClipboard::default());

    let config = Config::default();
    let theme = create_theme("Imperial Shell").expect("theme");
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
            .poll(PollCrossterm) //
            .poll(PollRendered),
    )?;

    Ok(())
}

/// Globally accessible data/state.
pub struct Global {
    // the salsa machinery
    ctx: SalsaAppContext<PalEvent, Error>,
    dlg: DialogStack<PalEvent, Global, Error>,

    pub cfg: Config,
    pub theme: SalsaTheme,
    pub status: String,
}

impl SalsaContext<PalEvent, Error> for Global {
    fn set_salsa_ctx(&mut self, app_ctx: SalsaAppContext<PalEvent, Error>) {
        self.ctx = app_ctx;
    }

    #[inline(always)]
    fn salsa_ctx(&self) -> &SalsaAppContext<PalEvent, Error> {
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
pub enum PalEvent {
    NoOp,
    Event(crossterm::event::Event),
    Rendered,
    Message(String),
}

impl From<RenderedEvent> for PalEvent {
    fn from(_: RenderedEvent) -> Self {
        Self::Rendered
    }
}

impl TryAsRef<crossterm::event::Event> for PalEvent {
    fn try_as_ref(&self) -> Option<&crossterm::event::Event> {
        match self {
            PalEvent::Event(e) => Some(e),
            _ => None,
        }
    }
}

impl From<crossterm::event::Event> for PalEvent {
    fn from(value: crossterm::event::Event) -> Self {
        Self::Event(value)
    }
}

#[derive(Debug)]
pub struct Scenery {
    pub defined: ChoiceState<String>,
    pub edit: PaletteEdit,
    pub menu: MenuLineState,
}

impl Default for Scenery {
    fn default() -> Self {
        Self {
            defined: ChoiceState::named("palette"),
            edit: PaletteEdit::default(),
            menu: MenuLineState::named("menu"),
        }
    }
}

impl HasFocus for Scenery {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget(&self.defined);
        builder.widget(&self.edit);
        builder.widget(&self.menu);
    }

    fn focus(&self) -> FocusFlag {
        unimplemented!("not defined")
    }

    fn area(&self) -> Rect {
        unimplemented!("not defined")
    }
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut Scenery,
    ctx: &mut Global,
) -> Result<(), Error> {
    let l1 = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1), //
        Constraint::Length(1),
    ])
    .split(area);

    let l2 = Layout::horizontal([
        Constraint::Length(80), //
    ])
    .horizontal_margin(1)
    .split(l1[2]);

    // main
    palette_edit::render(l2[0], buf, &mut state.edit, ctx)?;
    screen_cursor(state, ctx);

    // functions
    render_function(l1[0], buf, state, ctx)?;

    // menu & status
    let status_layout = Layout::horizontal([
        Constraint::Fill(61), //
        Constraint::Fill(39),
    ])
    .split(l1[3]);

    render_menu(status_layout[0], buf, state, ctx)?;
    render_status(status_layout[1], buf, ctx)?;

    // dialog windows
    ctx.dlg.clone().render(l1[0], buf, ctx);

    Ok(())
}

fn render_function(
    area: Rect,
    buf: &mut Buffer,
    state: &mut Scenery,
    ctx: &mut Global,
) -> Result<(), Error> {
    let l_function = Layout::horizontal([Constraint::Length(20)]).split(area);
    let (choice, choice_popup) = Choice::new()
        .items(
            once("")
                .chain(salsa_palettes())
                .map(|v| (v.to_string(), v.to_string())),
        )
        .styles(ctx.theme.style(WidgetStyle::CHOICE))
        .into_widgets();
    choice.render(l_function[0], buf, &mut state.defined);
    choice_popup.render(l_function[0], buf, &mut state.defined);
    Ok(())
}

fn render_menu(
    area: Rect,
    buf: &mut Buffer,
    state: &mut Scenery,
    ctx: &mut Global,
) -> Result<(), Error> {
    MenuLine::new()
        .styles(ctx.theme.style(WidgetStyle::MENU))
        .title("PAL")
        .item_parsed("_Quit")
        .render(area, buf, &mut state.menu);
    Ok(())
}

fn render_status(area: Rect, buf: &mut Buffer, ctx: &mut Global) -> Result<(), Error> {
    let palette = &ctx.theme.p;
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
        .render(area, buf);
    Ok(())
}

fn screen_cursor(state: &mut Scenery, ctx: &mut Global) {
    let sc = state.edit.screen_cursor();
    ctx.set_screen_cursor(sc);
}

pub fn init(state: &mut Scenery, ctx: &mut Global) -> Result<(), Error> {
    ctx.set_focus(FocusBuilder::build_for(state));
    ctx.focus().first();
    Ok(())
}

pub fn event(
    event: &PalEvent,
    state: &mut Scenery,
    ctx: &mut Global,
) -> Result<Control<PalEvent>, Error> {
    if let PalEvent::Event(event) = event {
        try_flow!(match &event {
            ct_event!(resized) => Control::Changed,
            ct_event!(key press CONTROL-'q') => Control::Quit,
            _ => Control::Continue,
        });
    }

    try_flow!(ctx.dlg.clone().handle(event, ctx)?);

    if let PalEvent::Event(event) = event {
        match ctx.handle_focus(event) {
            Outcome::Changed => {
                state.edit.form.show_focused(&ctx.focus());
            }
            _ => {}
        }

        try_flow!(match state.defined.handle(event, Popup) {
            ChoiceOutcome::Value => {
                if let Some(palette) = create_palette(state.defined.value().as_str()) {
                    state.edit.set_palette(palette);
                }
                Control::Changed
            }
            r => r.into(),
        });

        try_flow!(palette_edit::event(event, &mut state.edit, ctx)?);

        try_flow!(match state.menu.handle(event, Regular) {
            MenuOutcome::Activated(0) => Control::Quit,
            v => v.into(),
        });
    }

    match event {
        PalEvent::Rendered => {
            ctx.set_focus(FocusBuilder::rebuild_for(state, ctx.take_focus()));
            Ok(Control::Continue)
        }
        PalEvent::Message(s) => {
            show_error(s, ctx);
            Ok(Control::Changed)
        }
        _ => Ok(Control::Continue),
    }
}

pub fn error(
    event: Error,
    _state: &mut Scenery,
    ctx: &mut Global,
) -> Result<Control<PalEvent>, Error> {
    error!("{:?}", event);
    show_error(format!("{:?}", &*event).as_str(), ctx);
    Ok(Control::Changed)
}

mod palette_edit {
    use crate::{ColorSpan, ColorSpanState, Global, PalEvent};
    use anyhow::Error;
    use rat_event::{break_flow, MouseOnly, Outcome, Popup};
    use rat_focus::{FocusFlag, HasFocus};
    use rat_salsa::Control;
    use rat_theme4::{Palette, WidgetStyle};
    use rat_widget::choice::{Choice, ChoiceState};
    use rat_widget::clipper::{Clipper, ClipperState};
    use rat_widget::color_input::{ColorInput, ColorInputState, Mode};
    use rat_widget::event::{HandleEvent, Regular, ScrollOutcome, TextOutcome};
    use rat_widget::focus::FocusBuilder;
    use rat_widget::layout::LayoutForm;
    use rat_widget::paired::{Paired, PairedState};
    use rat_widget::scrolled::Scroll;
    use rat_widget::text::HasScreenCursor;
    use rat_widget::text_input::{TextInput, TextInputState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Flex, Rect};
    use ratatui::style::Color;
    use ratatui::text::Line;

    #[derive(Debug)]
    pub struct PaletteEdit {
        pub palette: Palette,

        pub form: ClipperState,
        pub name: TextInputState,

        pub primary: ChoiceState,
        pub secondary: ChoiceState,

        pub text_light: ColorInputState,
        pub text_bright: ColorInputState,
        pub text_dark: ColorInputState,
        pub text_black: ColorInputState,

        pub white0: ColorInputState,
        pub white3: ColorInputState,
        pub black0: ColorInputState,
        pub black3: ColorInputState,
        pub gray0: ColorInputState,
        pub gray3: ColorInputState,
        pub red0: ColorInputState,
        pub red3: ColorInputState,
        pub orange0: ColorInputState,
        pub orange3: ColorInputState,
        pub yellow0: ColorInputState,
        pub yellow3: ColorInputState,
        pub limegreen0: ColorInputState,
        pub limegreen3: ColorInputState,
        pub green0: ColorInputState,
        pub green3: ColorInputState,
        pub bluegreen0: ColorInputState,
        pub bluegreen3: ColorInputState,
        pub cyan0: ColorInputState,
        pub cyan3: ColorInputState,
        pub blue0: ColorInputState,
        pub blue3: ColorInputState,
        pub deepblue0: ColorInputState,
        pub deepblue3: ColorInputState,
        pub purple0: ColorInputState,
        pub purple3: ColorInputState,
        pub magenta0: ColorInputState,
        pub magenta3: ColorInputState,
        pub redpink0: ColorInputState,
        pub redpink3: ColorInputState,
    }

    impl Default for PaletteEdit {
        fn default() -> Self {
            let mut z = Self {
                palette: Default::default(),
                form: ClipperState::named("form"),
                name: TextInputState::named("name"),
                text_light: ColorInputState::named("text_light"),
                text_bright: ColorInputState::named("text_bright"),
                text_dark: ColorInputState::named("text_dark"),
                text_black: ColorInputState::named("text_black"),
                white0: ColorInputState::named("white0"),
                white3: ColorInputState::named("white3"),
                black0: ColorInputState::named("black0"),
                black3: ColorInputState::named("black3"),
                gray0: ColorInputState::named("gray0"),
                gray3: ColorInputState::named("gray3"),
                red0: ColorInputState::named("red0"),
                red3: ColorInputState::named("red3"),
                orange0: ColorInputState::named("orange0"),
                orange3: ColorInputState::named("orange3"),
                yellow0: ColorInputState::named("yellow0"),
                yellow3: ColorInputState::named("yellow3"),
                limegreen0: ColorInputState::named("limegreen0"),
                limegreen3: ColorInputState::named("limegreen3"),
                green0: ColorInputState::named("green0"),
                green3: ColorInputState::named("green3"),
                bluegreen0: ColorInputState::named("bluegreen0"),
                bluegreen3: ColorInputState::named("bluegreen3"),
                cyan0: ColorInputState::named("cyan0"),
                cyan3: ColorInputState::named("cyan3"),
                blue0: ColorInputState::named("blue0"),
                blue3: ColorInputState::named("blue3"),
                deepblue0: ColorInputState::named("deepblue0"),
                deepblue3: ColorInputState::named("deepblue3"),
                purple0: ColorInputState::named("purple0"),
                purple3: ColorInputState::named("purple3"),
                magenta0: ColorInputState::named("magenta0"),
                magenta3: ColorInputState::named("magenta3"),
                redpink0: ColorInputState::named("redpink0"),
                redpink3: ColorInputState::named("redpink3"),
                primary: ChoiceState::named("primary"),
                secondary: ChoiceState::named("secondary"),
            };

            z.text_light.set_value(Color::Rgb(255, 255, 255));
            z.text_bright.set_value(Color::Rgb(255, 255, 255));
            z.text_dark.set_value(Color::Rgb(0, 0, 0));
            z.text_black.set_value(Color::Rgb(0, 0, 0));
            z.white0.set_value(Color::Rgb(255, 255, 255));
            z.white3.set_value(Color::Rgb(255, 255, 255));
            z.black0.set_value(Color::Rgb(0, 0, 0));
            z.black3.set_value(Color::Rgb(0, 0, 0));
            z.gray0.set_value(Color::Rgb(85, 85, 85));
            z.gray3.set_value(Color::Rgb(170, 170, 170));

            z.red0.set_value(Color::Rgb(255, 0, 0));
            z.red3.set_value(Color::Rgb(255, 0, 0));
            z.orange0.set_value(Color::Rgb(255, 128, 0));
            z.orange3.set_value(Color::Rgb(255, 128, 0));
            z.yellow0.set_value(Color::Rgb(255, 255, 0));
            z.yellow3.set_value(Color::Rgb(255, 255, 0));
            z.limegreen0.set_value(Color::Rgb(128, 255, 0));
            z.limegreen3.set_value(Color::Rgb(128, 255, 0));
            z.green0.set_value(Color::Rgb(0, 255, 0));
            z.green3.set_value(Color::Rgb(0, 255, 0));
            z.bluegreen0.set_value(Color::Rgb(0, 255, 128));
            z.bluegreen3.set_value(Color::Rgb(0, 255, 128));
            z.cyan0.set_value(Color::Rgb(0, 255, 255));
            z.cyan3.set_value(Color::Rgb(0, 255, 255));
            z.blue0.set_value(Color::Rgb(0, 128, 255));
            z.blue3.set_value(Color::Rgb(0, 128, 255));
            z.deepblue0.set_value(Color::Rgb(0, 0, 255));
            z.deepblue3.set_value(Color::Rgb(0, 0, 255));
            z.purple0.set_value(Color::Rgb(128, 0, 255));
            z.purple3.set_value(Color::Rgb(128, 0, 255));
            z.magenta0.set_value(Color::Rgb(255, 0, 255));
            z.magenta3.set_value(Color::Rgb(255, 0, 255));
            z.redpink0.set_value(Color::Rgb(255, 0, 128));
            z.redpink3.set_value(Color::Rgb(255, 0, 128));
            z
        }
    }

    impl PaletteEdit {
        fn paired_colors(&self) -> [(&'static str, &ColorInputState, &ColorInputState); 15] {
            [
                ("White", &self.white0, &self.white3),
                ("Black", &self.black0, &self.black3),
                ("Gray", &self.gray0, &self.gray3),
                ("Red", &self.red0, &self.red3),
                ("Orange", &self.orange0, &self.orange3),
                ("Yellow", &self.yellow0, &self.yellow3),
                ("Limegreen", &self.limegreen0, &self.limegreen3),
                ("Green", &self.green0, &self.green3),
                ("Bluegreen", &self.bluegreen0, &self.bluegreen3),
                ("Cyan", &self.cyan0, &self.cyan3),
                ("Blue", &self.blue0, &self.blue3),
                ("Deepblue", &self.deepblue0, &self.deepblue3),
                ("Purple", &self.purple0, &self.purple3),
                ("Magenta", &self.magenta0, &self.magenta3),
                ("Redpink", &self.redpink0, &self.redpink3),
            ]
        }

        fn paired_colors_mut(
            &mut self,
        ) -> [(&str, &mut ColorInputState, &mut ColorInputState); 15] {
            [
                ("White", &mut self.white0, &mut self.white3),
                ("Black", &mut self.black0, &mut self.black3),
                ("Gray", &mut self.gray0, &mut self.gray3),
                ("Red", &mut self.red0, &mut self.red3),
                ("Orange", &mut self.orange0, &mut self.orange3),
                ("Yellow", &mut self.yellow0, &mut self.yellow3),
                ("Limegreen", &mut self.limegreen0, &mut self.limegreen3),
                ("Green", &mut self.green0, &mut self.green3),
                ("Bluegreen", &mut self.bluegreen0, &mut self.bluegreen3),
                ("Cyan", &mut self.cyan0, &mut self.cyan3),
                ("Blue", &mut self.blue0, &mut self.blue3),
                ("Deepblue", &mut self.deepblue0, &mut self.deepblue3),
                ("Purple", &mut self.purple0, &mut self.purple3),
                ("Magenta", &mut self.magenta0, &mut self.magenta3),
                ("Redpink", &mut self.redpink0, &mut self.redpink3),
            ]
        }

        pub fn palette(&self) -> Palette {
            let mut palette = Palette::default();
            palette.text_light = self.text_light.value();
            palette.text_bright = self.text_bright.value();
            palette.text_dark = self.text_dark.value();
            palette.text_black = self.text_black.value();
            palette.white =
                Palette::interpolate(self.white0.value_u32(), self.white3.value_u32(), 64);
            palette.gray = Palette::interpolate(self.gray0.value_u32(), self.gray3.value_u32(), 64);
            palette.black =
                Palette::interpolate(self.black0.value_u32(), self.black3.value_u32(), 64);
            palette.red = Palette::interpolate(self.red0.value_u32(), self.red3.value_u32(), 64);
            palette.orange =
                Palette::interpolate(self.orange0.value_u32(), self.orange3.value_u32(), 64);
            palette.yellow =
                Palette::interpolate(self.yellow0.value_u32(), self.yellow3.value_u32(), 64);
            palette.limegreen =
                Palette::interpolate(self.limegreen0.value_u32(), self.limegreen3.value_u32(), 64);
            palette.green =
                Palette::interpolate(self.green0.value_u32(), self.green3.value_u32(), 64);
            palette.bluegreen =
                Palette::interpolate(self.bluegreen0.value_u32(), self.bluegreen3.value_u32(), 64);
            palette.cyan = Palette::interpolate(self.cyan0.value_u32(), self.cyan3.value_u32(), 64);
            palette.blue = Palette::interpolate(self.blue0.value_u32(), self.blue3.value_u32(), 64);
            palette.deepblue =
                Palette::interpolate(self.deepblue0.value_u32(), self.deepblue3.value_u32(), 64);
            palette.purple =
                Palette::interpolate(self.purple0.value_u32(), self.purple3.value_u32(), 64);
            palette.magenta =
                Palette::interpolate(self.magenta0.value_u32(), self.magenta3.value_u32(), 64);
            palette.redpink =
                Palette::interpolate(self.redpink0.value_u32(), self.redpink3.value_u32(), 64);

            let colors = self
                .paired_colors()
                .iter()
                .map(|(_, v0, v3)| (v0.value_u32(), v3.value_u32()))
                .collect::<Vec<_>>();

            let n = self.primary.value();
            palette.primary = Palette::interpolate(colors[n].0, colors[n].1, 64);
            let n = self.secondary.value();
            palette.secondary = Palette::interpolate(colors[n].0, colors[n].1, 64);
            palette
        }

        pub fn set_palette(&mut self, palette: Palette) {
            self.primary
                .set_value(palette_idx(&palette, &palette.primary));
            self.secondary
                .set_value(palette_idx(&palette, &palette.secondary));

            self.text_light.set_value(palette.text_light);
            self.text_bright.set_value(palette.text_bright);
            self.text_dark.set_value(palette.text_dark);
            self.text_black.set_value(palette.text_black);
            self.white0.set_value(palette.white[0]);
            self.white3.set_value(palette.white[3]);
            self.black0.set_value(palette.black[0]);
            self.black3.set_value(palette.black[3]);
            self.gray0.set_value(palette.gray[0]);
            self.gray3.set_value(palette.gray[3]);
            self.red0.set_value(palette.red[0]);
            self.red3.set_value(palette.red[3]);
            self.orange0.set_value(palette.orange[0]);
            self.orange3.set_value(palette.orange[3]);
            self.yellow0.set_value(palette.yellow[0]);
            self.yellow3.set_value(palette.yellow[3]);
            self.limegreen0.set_value(palette.limegreen[0]);
            self.limegreen3.set_value(palette.limegreen[3]);
            self.green0.set_value(palette.green[0]);
            self.green3.set_value(palette.green[3]);
            self.bluegreen0.set_value(palette.bluegreen[0]);
            self.bluegreen3.set_value(palette.bluegreen[3]);
            self.cyan0.set_value(palette.cyan[0]);
            self.cyan3.set_value(palette.cyan[3]);
            self.blue0.set_value(palette.blue[0]);
            self.blue3.set_value(palette.blue[3]);
            self.deepblue0.set_value(palette.deepblue[0]);
            self.deepblue3.set_value(palette.deepblue[3]);
            self.purple0.set_value(palette.purple[0]);
            self.purple3.set_value(palette.purple[3]);
            self.magenta0.set_value(palette.magenta[0]);
            self.magenta3.set_value(palette.magenta[3]);
            self.redpink0.set_value(palette.redpink[0]);
            self.redpink3.set_value(palette.redpink[3]);
        }
    }

    fn palette_idx(palette: &Palette, colors: &[Color; 8]) -> usize {
        if colors == &palette.white {
            0
        } else if colors == &palette.black {
            1
        } else if colors == &palette.gray {
            2
        } else if colors == &palette.red {
            3
        } else if colors == &palette.orange {
            4
        } else if colors == &palette.yellow {
            5
        } else if colors == &palette.limegreen {
            6
        } else if colors == &palette.green {
            7
        } else if colors == &palette.bluegreen {
            8
        } else if colors == &palette.cyan {
            9
        } else if colors == &palette.blue {
            10
        } else if colors == &palette.deepblue {
            11
        } else if colors == &palette.purple {
            12
        } else if colors == &palette.magenta {
            13
        } else if colors == &palette.redpink {
            14
        } else {
            0
        }
    }

    impl HasFocus for PaletteEdit {
        fn build(&self, builder: &mut FocusBuilder) {
            builder.widget(&self.name);
            builder.widget(&self.primary);
            builder.widget(&self.secondary);
            builder.widget(&self.text_light);
            builder.widget(&self.text_bright);
            builder.widget(&self.text_dark);
            builder.widget(&self.text_black);
            for (_, v0, v3) in self.paired_colors() {
                builder.widget(v0);
                builder.widget(v3);
            }
        }

        fn focus(&self) -> FocusFlag {
            unimplemented!("not defined")
        }

        fn area(&self) -> Rect {
            unimplemented!("not defined")
        }
    }

    impl HasScreenCursor for PaletteEdit {
        fn screen_cursor(&self) -> Option<(u16, u16)> {
            self.name
                .screen_cursor()
                .or(self.text_light.screen_cursor())
                .or(self.text_bright.screen_cursor())
                .or(self.text_dark.screen_cursor())
                .or(self.text_black.screen_cursor())
                .or_else(|| {
                    for (_, v0, v3) in self.paired_colors() {
                        if let Some(s) = v0.screen_cursor() {
                            return Some(s);
                        }
                        if let Some(s) = v3.screen_cursor() {
                            return Some(s);
                        }
                    }
                    None
                })
        }
    }

    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        state: &mut PaletteEdit,
        ctx: &mut Global,
    ) -> Result<(), Error> {
        let mut form = Clipper::new() //
            .vscroll(Scroll::new())
            .styles(ctx.theme.style(WidgetStyle::CLIPPER));

        let layout_size = form.layout_size(area, &mut state.form);

        if !state.form.valid_layout(layout_size) {
            use rat_widget::layout::{FormLabel as L, FormWidget as W};
            let mut layout = LayoutForm::<usize>::new().spacing(1).flex(Flex::Start);
            layout.widget(state.name.id(), L::Str("Name"), W::Width(20));
            layout.gap(1);
            layout.widget(state.primary.id(), L::Str("Primary"), W::Width(16));
            layout.widget(state.secondary.id(), L::Str("Secondary"), W::Width(16));
            layout.gap(1);
            layout.widget(state.text_light.id(), L::Str("Text light"), W::Width(33));
            layout.widget(state.text_dark.id(), L::Str("Text dark"), W::Width(33));
            layout.gap(1);
            layout.widget(state.white0.id(), L::Str("White"), W::Width(50));
            layout.widget(state.black0.id(), L::Str("Black"), W::Width(50));
            layout.widget(state.gray0.id(), L::Str("Gray"), W::Width(50));
            layout.widget(state.red0.id(), L::Str("Red"), W::Width(50));
            layout.widget(state.orange0.id(), L::Str("Orange"), W::Width(50));
            layout.widget(state.yellow0.id(), L::Str("Yellow"), W::Width(50));
            layout.widget(state.limegreen0.id(), L::Str("Limegreen"), W::Width(50));
            layout.widget(state.green0.id(), L::Str("Green"), W::Width(50));
            layout.widget(state.bluegreen0.id(), L::Str("Bluegreen"), W::Width(50));
            layout.widget(state.cyan0.id(), L::Str("Cyan"), W::Width(50));
            layout.widget(state.blue0.id(), L::Str("Blue"), W::Width(50));
            layout.widget(state.deepblue0.id(), L::Str("Deepblue"), W::Width(50));
            layout.widget(state.purple0.id(), L::Str("Purple"), W::Width(50));
            layout.widget(state.magenta0.id(), L::Str("Magenta"), W::Width(50));
            layout.widget(state.redpink0.id(), L::Str("RedPink"), W::Width(50));
            form = form.layout(layout.build_endless(layout_size.width));
        }
        let mut form = form.into_buffer(area, &mut state.form);

        form.render(
            state.name.id(),
            || TextInput::new().styles(ctx.theme.style(WidgetStyle::TEXT)),
            &mut state.name,
        );

        let primary_items = state
            .paired_colors()
            .iter()
            .enumerate()
            .map(|(idx, (n, v0, _v3))| {
                (
                    idx,
                    Line::from(*n).style(ctx.theme.p.normal_contrast(v0.value())),
                )
            })
            .collect::<Vec<_>>();
        let primary_popup = form.render2(
            state.primary.id(),
            || {
                Choice::new()
                    .items(primary_items.iter().cloned())
                    .select_marker('→')
                    .styles(ctx.theme.style(WidgetStyle::CHOICE))
                    .into_widgets()
            },
            &mut state.primary,
        );

        let secondary_items = state
            .paired_colors()
            .iter()
            .enumerate()
            .map(|(idx, (n, v0, _v3))| {
                (
                    idx,
                    Line::from(*n).style(ctx.theme.p.normal_contrast(v0.value())),
                )
            })
            .collect::<Vec<_>>();
        let secondary_popup = form.render2(
            state.secondary.id(),
            || {
                Choice::new()
                    .items(secondary_items.iter().cloned())
                    .select_marker('→')
                    .styles(ctx.theme.style(WidgetStyle::CHOICE))
                    .into_widgets()
            },
            &mut state.secondary,
        );

        form.render(
            state.text_light.id(),
            || {
                Paired::new(
                    ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
                    ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
                )
            },
            &mut PairedState {
                first: &mut state.text_light,
                second: &mut state.text_bright,
            },
        );
        form.render(
            state.text_dark.id(),
            || {
                Paired::new(
                    ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
                    ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
                )
            },
            &mut PairedState {
                first: &mut state.text_dark,
                second: &mut state.text_black,
            },
        );

        for (_, c0, c3) in state.paired_colors_mut() {
            form.render(
                c0.id(),
                || {
                    ColorSpan::new()
                        .color0(ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)))
                        .color3(ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)))
                },
                &mut ColorSpanState {
                    color0: c0,
                    color3: c3,
                },
            );
        }

        form.render_opt(state.primary.id(), || primary_popup, &mut state.primary);
        form.render_opt(
            state.secondary.id(),
            || secondary_popup,
            &mut state.secondary,
        );
        form.finish(buf, &mut state.form);

        Ok(())
    }

    pub fn event(
        event: &crossterm::event::Event,
        state: &mut PaletteEdit,
        _ctx: &mut Global,
    ) -> Result<Control<PalEvent>, Error> {
        let mut mode_change = None;

        let r = 'f: {
            break_flow!('f: state.primary.handle(event, Popup));
            break_flow!('f: state.secondary.handle(event, Popup));

            break_flow!('f: state.name.handle(event, Regular));

            break_flow!('f: handle_color(event, &mut state.text_light, &mut mode_change));
            break_flow!('f: handle_color(event, &mut state.text_bright, &mut mode_change));
            break_flow!('f: handle_color(event, &mut state.text_dark, &mut mode_change));
            break_flow!('f: handle_color(event, &mut state.text_black, &mut mode_change));
            for (_, v0, v3) in state.paired_colors_mut() {
                break_flow!('f: handle_color(event, v0, &mut mode_change));
                break_flow!('f: handle_color(event, v3, &mut mode_change));
            }

            // completely override Clipper event-handling.
            // Need none of that only scrolling with the scrollbar.
            break_flow!('f: match state.form.vscroll.handle(event, MouseOnly) {
                ScrollOutcome::Up(v) => Outcome::from(state.form.scroll_up(v)),
                ScrollOutcome::Down(v) => Outcome::from(state.form.scroll_down(v)),
                ScrollOutcome::VPos(v) => Outcome::from(state.form.set_vertical_offset(v)),
                ScrollOutcome::Left(v) => Outcome::from(state.form.scroll_left(v)),
                ScrollOutcome::Right(v) => Outcome::from(state.form.scroll_right(v)),
                ScrollOutcome::HPos(v) => Outcome::from(state.form.set_horizontal_offset(v)),
                r => r.into(),
            });

            Control::Continue
        };

        if let Some(mode_change) = mode_change {
            state.text_light.set_mode(mode_change);
            state.text_bright.set_mode(mode_change);
            state.text_dark.set_mode(mode_change);
            state.text_black.set_mode(mode_change);

            for (_, v0, v3) in state.paired_colors_mut() {
                v0.set_mode(mode_change);
                v3.set_mode(mode_change);
            }
        }

        Ok(r)
    }

    fn handle_color(
        event: &crossterm::event::Event,
        color: &mut ColorInputState,
        mode_change: &mut Option<Mode>,
    ) -> TextOutcome {
        let mode = color.mode();
        let r = color.handle(event, Regular);
        if color.mode() != mode {
            *mode_change = Some(color.mode());
        }
        r
    }
}

#[derive(Default, Debug)]
struct ColorSpan<'a> {
    color0: ColorInput<'a>,
    color3: ColorInput<'a>,
}

struct ColorSpanState<'a> {
    pub color0: &'a mut ColorInputState,
    pub color3: &'a mut ColorInputState,
}

impl<'a> RelocatableState for ColorSpanState<'a> {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.color0.relocate(shift, clip);
        self.color3.relocate(shift, clip);
    }
}

impl<'a> ColorSpan<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn color0(mut self, color: ColorInput<'a>) -> Self {
        self.color0 = color;
        self
    }

    pub fn color3(mut self, color: ColorInput<'a>) -> Self {
        self.color3 = color;
        self
    }
}

impl<'a> StatefulWidget for ColorSpan<'a> {
    type State = ColorSpanState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.color0
            .render(Rect::new(area.x, area.y, 16, 1), buf, state.color0);
        self.color3
            .render(Rect::new(area.x + 17, area.y, 16, 1), buf, state.color3);

        let width = (area.width - 33) / 8;
        let colors = Palette::interpolate(state.color0.value_u32(), state.color3.value_u32(), 64);
        for i in 0usize..8usize {
            let color_area =
                Rect::new(area.x + 34 + (i as u16) * width, area.y, width, area.height);
            buf.set_style(color_area, Style::new().bg(colors[i]));
        }
    }
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
    event: &PalEvent,
    state: &mut dyn Any,
    ctx: &mut Global,
) -> Result<Control<PalEvent>, Error> {
    let state = state.downcast_mut::<MsgState>().expect("msg-dialog");

    if let PalEvent::Event(e) = event {
        let mut focus = FocusBuilder::build_for(state);
        ctx.queue(focus.handle(e, Regular));

        try_flow!(state.paragraph.handle(e, Regular));
        try_flow!(match state.dlg.handle(e, Dialog) {
            DialogOutcome::Ok => {
                Control::Close(PalEvent::NoOp)
            }
            DialogOutcome::Cancel => {
                Control::Close(PalEvent::NoOp)
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

#[derive(Debug, Default, Clone)]
struct CliClipboard {
    clip: RefCell<String>,
}

impl Clipboard for CliClipboard {
    fn get_string(&self) -> Result<String, ClipboardError> {
        match cli_clipboard::get_contents() {
            Ok(v) => Ok(v),
            Err(e) => {
                warn!("{:?}", e);
                Ok(self.clip.borrow().clone())
            }
        }
    }

    fn set_string(&self, s: &str) -> Result<(), ClipboardError> {
        let mut clip = self.clip.borrow_mut();
        *clip = s.to_string();

        match cli_clipboard::set_contents(s.to_string()) {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("{:?}", e);
                Err(ClipboardError)
            }
        }
    }
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
