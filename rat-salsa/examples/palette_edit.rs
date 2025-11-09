use anyhow::Error;
use log::error;
use rat_event::{break_flow, try_flow, Popup};
use rat_focus::{impl_has_focus, FocusFlag, HasFocus};
use rat_salsa::dialog_stack::DialogStack;
use rat_salsa::event::RenderedEvent;
use rat_salsa::poll::{PollCrossterm, PollRendered};
use rat_salsa::{run_tui, Control, RunConfig, SalsaAppContext, SalsaContext};
use rat_theme4::{create_theme, SalsaTheme, WidgetStyle};
use rat_widget::choice::ChoiceState;
use rat_widget::clipper::{Clipper, ClipperState};
use rat_widget::color_input::{ColorInput, ColorInputState};
use rat_widget::dialog_frame::{DialogFrame, DialogFrameState, DialogOutcome};
use rat_widget::event::{ct_event, Dialog, HandleEvent, MenuOutcome, Regular, TextOutcome};
use rat_widget::focus::FocusBuilder;
use rat_widget::form::{Form, FormState};
use rat_widget::layout::LayoutForm;
use rat_widget::menu::{MenuLine, MenuLineState};
use rat_widget::paired::{PairSplit, Paired, PairedState};
use rat_widget::paragraph::{Paragraph, ParagraphState};
use rat_widget::statusline_stacked::StatusLineStacked;
use rat_widget::text::HasScreenCursor;
use rat_widget::text_input::{TextInput, TextInputState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::widgets::{StatefulWidget, Widget};
use std::any::Any;
use std::fs;
use try_as::traits::TryAsRef;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let config = Config::default();
    let theme = create_theme("Imperial Shell").expect("theme");
    let mut global = Global::new(config, theme);
    let mut state = PaletteEdit::default();

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
pub struct PaletteEdit {
    pub form: ClipperState,

    pub name: TextInputState,

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
    pub primary: ChoiceState,
    pub secondary: ChoiceState,

    pub menu: MenuLineState,
}

impl Default for PaletteEdit {
    fn default() -> Self {
        let mut z = Self {
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
            menu: MenuLineState::named("menu"),
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

impl HasFocus for PaletteEdit {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget(&self.name);
        for v in colors(self) {
            builder.widget(v);
        }
        builder.widget(&self.primary);
        builder.widget(&self.secondary);
        builder.widget(&self.menu);
    }

    fn focus(&self) -> FocusFlag {
        unimplemented!("not defined")
    }

    fn area(&self) -> Rect {
        unimplemented!("not defined")
    }
}

fn paired_colors_mut(
    state: &mut PaletteEdit,
) -> [(&mut ColorInputState, &mut ColorInputState); 17] {
    [
        (&mut state.text_light, &mut state.text_bright),
        (&mut state.text_dark, &mut state.text_black),
        (&mut state.white0, &mut state.white3),
        (&mut state.black0, &mut state.black3),
        (&mut state.gray0, &mut state.gray3),
        (&mut state.red0, &mut state.red3),
        (&mut state.orange0, &mut state.orange3),
        (&mut state.yellow0, &mut state.yellow3),
        (&mut state.limegreen0, &mut state.limegreen3),
        (&mut state.green0, &mut state.green3),
        (&mut state.bluegreen0, &mut state.bluegreen3),
        (&mut state.cyan0, &mut state.cyan3),
        (&mut state.blue0, &mut state.blue3),
        (&mut state.deepblue0, &mut state.deepblue3),
        (&mut state.purple0, &mut state.purple3),
        (&mut state.magenta0, &mut state.magenta3),
        (&mut state.redpink0, &mut state.redpink3),
    ]
}

fn colors(state: &PaletteEdit) -> [&ColorInputState; 34] {
    [
        &state.text_light,
        &state.text_bright,
        &state.text_dark,
        &state.text_black,
        &state.white0,
        &state.white3,
        &state.black0,
        &state.black3,
        &state.gray0,
        &state.gray3,
        &state.red0,
        &state.red3,
        &state.orange0,
        &state.orange3,
        &state.yellow0,
        &state.yellow3,
        &state.limegreen0,
        &state.limegreen3,
        &state.green0,
        &state.green3,
        &state.bluegreen0,
        &state.bluegreen3,
        &state.cyan0,
        &state.cyan3,
        &state.blue0,
        &state.blue3,
        &state.deepblue0,
        &state.deepblue3,
        &state.purple0,
        &state.purple3,
        &state.magenta0,
        &state.magenta3,
        &state.redpink0,
        &state.redpink3,
    ]
}

fn colors_mut(state: &mut PaletteEdit) -> [&mut ColorInputState; 34] {
    [
        &mut state.text_light,
        &mut state.text_bright,
        &mut state.text_dark,
        &mut state.text_black,
        &mut state.white0,
        &mut state.white3,
        &mut state.black0,
        &mut state.black3,
        &mut state.gray0,
        &mut state.gray3,
        &mut state.red0,
        &mut state.red3,
        &mut state.orange0,
        &mut state.orange3,
        &mut state.yellow0,
        &mut state.yellow3,
        &mut state.limegreen0,
        &mut state.limegreen3,
        &mut state.green0,
        &mut state.green3,
        &mut state.bluegreen0,
        &mut state.bluegreen3,
        &mut state.cyan0,
        &mut state.cyan3,
        &mut state.blue0,
        &mut state.blue3,
        &mut state.deepblue0,
        &mut state.deepblue3,
        &mut state.purple0,
        &mut state.purple3,
        &mut state.magenta0,
        &mut state.magenta3,
        &mut state.redpink0,
        &mut state.redpink3,
    ]
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut PaletteEdit,
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

    let mut form = Clipper::new() //
        .styles(ctx.theme.style(WidgetStyle::CLIPPER));
    let layout_size = form.layout_size(layout[0], &mut state.form);
    if !state.form.valid_layout(layout_size) {
        use rat_widget::layout::{FormLabel as L, FormWidget as W};
        let mut layout = LayoutForm::<usize>::new()
            .spacing(1)
            .line_spacing(1)
            // .columns(2)
            .flex(Flex::Start);
        layout.widget(state.name.id(), L::Str("Name"), W::Width(20));
        layout.widget(state.text_light.id(), L::Str("Text light"), W::Width(33));
        layout.widget(state.text_dark.id(), L::Str("Text dark"), W::Width(33));
        layout.widget(state.white0.id(), L::Str("White"), W::Width(33));
        layout.widget(state.black0.id(), L::Str("Black"), W::Width(33));
        layout.widget(state.gray0.id(), L::Str("Gray"), W::Width(33));
        layout.widget(state.red0.id(), L::Str("Red"), W::Width(33));
        layout.widget(state.orange0.id(), L::Str("Orange"), W::Width(33));
        layout.widget(state.yellow0.id(), L::Str("Yellow"), W::Width(33));
        layout.widget(state.limegreen0.id(), L::Str("Limegreen"), W::Width(33));
        layout.widget(state.green0.id(), L::Str("Green"), W::Width(33));
        layout.widget(state.bluegreen0.id(), L::Str("Bluegreen"), W::Width(33));
        layout.widget(state.cyan0.id(), L::Str("Cyan"), W::Width(33));
        layout.widget(state.blue0.id(), L::Str("Blue"), W::Width(33));
        layout.widget(state.deepblue0.id(), L::Str("Deepblue"), W::Width(33));
        layout.widget(state.purple0.id(), L::Str("Purple"), W::Width(33));
        layout.widget(state.magenta0.id(), L::Str("Magenta"), W::Width(33));
        layout.widget(state.redpink0.id(), L::Str("RedPink"), W::Width(33));
        layout.widget(state.primary.id(), L::Str("P/S"), W::Width(33));
        form = form.layout(layout.build_endless(layout_size.width));
    }
    let mut form = form.into_buffer(layout[0], &mut state.form);

    form.render(
        state.name.id(),
        || TextInput::new().styles(ctx.theme.style(WidgetStyle::TEXT)),
        &mut state.name,
    );
    for (a, b) in paired_colors_mut(state) {
        form.render(
            a.id(),
            || {
                Paired::new(
                    ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
                    ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
                )
            },
            &mut PairedState::new(a, b),
        );
    }
    form.finish(buf, &mut state.form);

    MenuLine::new()
        .styles(ctx.theme.style(WidgetStyle::MENU))
        .title("PAL")
        .item_parsed("_Quit")
        .render(status_layout[0], buf, &mut state.menu);

    let mut sc = None;
    for a in colors(state) {
        if let Some(s) = a.screen_cursor() {
            sc = Some(s);
            break;
        }
    }
    ctx.set_screen_cursor(state.name.screen_cursor().or(sc));

    ctx.dlg.clone().render(layout[0], buf, ctx);

    // Status
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
        .render(status_layout[1], buf);

    Ok(())
}

pub fn init(state: &mut PaletteEdit, ctx: &mut Global) -> Result<(), Error> {
    ctx.set_focus(FocusBuilder::build_for(state));
    ctx.focus().enable_log();
    ctx.focus().first();
    Ok(())
}

pub fn event(
    event: &PalEvent,
    state: &mut PaletteEdit,
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
        ctx.handle_focus(event);

        try_flow!(color_event(event, state, ctx)?);

        try_flow!(match state.menu.handle(event, Regular) {
            MenuOutcome::Activated(0) => Control::Event(PalEvent::Message("message!".to_string())),
            MenuOutcome::Activated(1) => Control::Quit,
            v => v.into(),
        });
    }

    match event {
        PalEvent::Rendered => {
            ctx.set_focus(FocusBuilder::rebuild_for(state, ctx.take_focus()));
            ctx.focus().enable_log();
            Ok(Control::Continue)
        }
        PalEvent::Message(s) => {
            show_error(s, ctx);
            Ok(Control::Changed)
        }
        _ => Ok(Control::Continue),
    }
}

fn color_event(
    event: &crossterm::event::Event,
    state: &mut PaletteEdit,
    _ctx: &mut Global,
) -> Result<Control<PalEvent>, Error> {
    let mut mode_change = None;
    let r = 'f: {
        break_flow!('f: state.primary.handle(event, Popup));
        break_flow!('f: state.secondary.handle(event, Popup));
        break_flow!('f: state.name.handle(event, Regular));

        for v in colors_mut(state) {
            break_flow!('f: {
                let mode = v.mode();
                let r = v.handle(event, Regular);
                if v.mode() != mode {
                    mode_change = Some(v.mode());
                }
                r
            });
        }

        break_flow!('f: state.form.handle(event, Regular));

        Control::Continue
    };

    if let Some(mode_change) = mode_change {
        for v in colors_mut(state) {
            v.set_mode(mode_change);
        }
    }

    Ok(r)
}

pub fn error(
    event: Error,
    _state: &mut PaletteEdit,
    ctx: &mut Global,
) -> Result<Control<PalEvent>, Error> {
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
