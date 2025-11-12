use crate::configparser_ext::ConfigParserExt;
use crate::palette_edit::PaletteEdit;
use crate::showcase::ShowCase;
use anyhow::{anyhow, Error};
use configparser::ini::Ini;
use log::{error, warn};
use rat_event::{try_flow, Outcome, Popup};
use rat_focus::{impl_has_focus, FocusFlag, HasFocus};
use rat_salsa::dialog_stack::file_dialog::{file_dialog_event, file_dialog_render};
use rat_salsa::dialog_stack::DialogStack;
use rat_salsa::event::RenderedEvent;
use rat_salsa::poll::{PollCrossterm, PollRendered};
use rat_salsa::{run_tui, Control, RunConfig, SalsaAppContext, SalsaContext};
use rat_theme4::{
    create_palette, create_theme, dark_theme, fallback_theme, salsa_palettes, shell_theme, Palette,
    SalsaTheme, WidgetStyle,
};
use rat_widget::choice::{Choice, ChoiceState};
use rat_widget::color_input::{ColorInput, ColorInputState};
use rat_widget::dialog_frame::{DialogFrame, DialogFrameState, DialogOutcome};
use rat_widget::event::{ct_event, ChoiceOutcome, Dialog, HandleEvent, MenuOutcome, Regular};
use rat_widget::file_dialog::FileDialogState;
use rat_widget::focus::FocusBuilder;
use rat_widget::layout::LayoutOuter;
use rat_widget::menu::{MenuLine, MenuLineState};
use rat_widget::paragraph::{Paragraph, ParagraphState};
use rat_widget::reloc::RelocatableState;
use rat_widget::statusline_stacked::StatusLineStacked;
use rat_widget::text::clipboard::{set_global_clipboard, Clipboard, ClipboardError};
use rat_widget::text::HasScreenCursor;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{StatefulWidget, Widget};
use std::any::Any;
use std::cell::RefCell;
use std::fs;
use std::fs::File;
use std::iter::once;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use try_as_traits::TryAsRef;

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

    pub status_frame: usize,
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
            status_frame: 0,
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
    Save(PathBuf),
    Load(PathBuf),
    Export(PathBuf),
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
    pub themes: ChoiceState<String>,

    pub file_dlg: Rc<RefCell<FileDialogState>>,

    pub edit: PaletteEdit,
    pub show: ShowCase,
    pub menu: MenuLineState,
}

impl Default for Scenery {
    fn default() -> Self {
        Self {
            defined: ChoiceState::named("palette"),
            themes: ChoiceState::named("themes"),
            file_dlg: Rc::new(RefCell::new(FileDialogState::default())),
            edit: PaletteEdit::default(),
            show: ShowCase::default(),
            menu: MenuLineState::named("menu"),
        }
    }
}

impl HasFocus for Scenery {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget(&self.defined);
        builder.widget(&self.themes);
        builder.widget(&self.edit);
        builder.widget(&self.show);
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
        Constraint::Length(50), //
    ])
    .horizontal_margin(1)
    .split(l1[2]);

    // main
    palette_edit::render(l2[0], buf, &mut state.edit, ctx)?;
    showcase::render(l2[1], buf, &mut state.show, ctx)?;
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
    ctx.dlg.clone().render(area, buf, ctx);

    Ok(())
}

fn render_function(
    area: Rect,
    buf: &mut Buffer,
    state: &mut Scenery,
    ctx: &mut Global,
) -> Result<(), Error> {
    let l_function = Layout::horizontal([
        Constraint::Length(20), //
        Constraint::Length(15),
    ])
    .split(area);

    let (choice, choice_popup) = Choice::new()
        .items(
            once("")
                .chain(salsa_palettes())
                .map(|v| (v.to_string(), v.to_string())),
        )
        .styles(ctx.theme.style(WidgetStyle::CHOICE))
        .into_widgets();
    choice.render(l_function[0], buf, &mut state.defined);

    let (choice, choice_theme) = Choice::new()
        .items(
            once("")
                .chain(["Dark", "Shell", "Fallback"])
                .map(|v| (v.to_string(), v.to_string())),
        )
        .styles(ctx.theme.style(WidgetStyle::CHOICE))
        .into_widgets();
    choice.render(l_function[1], buf, &mut state.themes);

    choice_popup.render(l_function[0], buf, &mut state.defined);
    choice_theme.render(l_function[1], buf, &mut state.themes);

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
        .title(Line::from_iter([
            Span::from(" P ").white().on_red(),
            Span::from(" A ").white().on_green(),
            Span::from(" L ").white().on_blue(),
        ]))
        .item_parsed("_Load")
        .item_parsed("_Save")
        .item_parsed("_Export")
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

    if !ctx.status.is_empty() {
        if ctx.status_frame == 0 {
            ctx.status_frame = ctx.count();
        } else if ctx.status_frame + 4 < ctx.count() {
            ctx.status_frame = 0;
            ctx.status = String::default();
        }
    }

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
    let sc = state.edit.screen_cursor().or(state.show.screen_cursor());
    ctx.set_screen_cursor(sc);
}

pub fn init(state: &mut Scenery, ctx: &mut Global) -> Result<(), Error> {
    ctx.set_focus(FocusBuilder::build_for(state));
    // ctx.focus().enable_log();
    ctx.focus().first();

    let theme = create_edit_theme(state);
    state.show.set_theme(theme);

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
                state.show.form.show_focused(&ctx.focus());
            }
            _ => {}
        }

        try_flow!(match state.defined.handle(event, Popup) {
            ChoiceOutcome::Value => {
                if let Some(palette) = create_palette(state.defined.value().as_str()) {
                    state.edit.set_palette(palette);
                }
                let theme = create_edit_theme(state);
                state.show.set_theme(theme);
                Control::Changed
            }
            r => r.into(),
        });
        try_flow!(match state.themes.handle(event, Popup) {
            ChoiceOutcome::Value => {
                let theme = create_edit_theme(state);
                state.show.set_theme(theme);
                Control::Changed
            }
            r => r.into(),
        });

        try_flow!(match palette_edit::event(event, &mut state.edit, ctx)? {
            Outcome::Changed => {
                let theme = create_edit_theme(state);
                state.show.set_theme(theme);
                Outcome::Changed
            }
            r => r.into(),
        });
        try_flow!(showcase::event(event, &mut state.show, ctx)?);

        try_flow!(match state.menu.handle(event, Regular) {
            MenuOutcome::Activated(0) => load_pal(state, ctx)?,
            MenuOutcome::Activated(1) => save_pal(state, ctx)?,
            MenuOutcome::Activated(2) => export_pal(state, ctx)?,
            MenuOutcome::Activated(3) => Control::Quit,
            v => v.into(),
        });
    }

    match event {
        PalEvent::Rendered => {
            ctx.set_focus(FocusBuilder::rebuild_for(state, ctx.take_focus()));
            // ctx.focus().enable_log();
            Ok(Control::Continue)
        }
        PalEvent::Message(s) => {
            show_error(s, ctx);
            Ok(Control::Changed)
        }
        PalEvent::Save(p) => save_pal_file(&p, state, ctx),
        PalEvent::Load(p) => load_pal_file(&p, state, ctx),
        PalEvent::Export(p) => export_pal_file(&p, state, ctx),
        _ => Ok(Control::Continue),
    }
}

fn export_pal(state: &mut Scenery, ctx: &mut Global) -> Result<Control<PalEvent>, Error> {
    let s = state.file_dlg.clone();
    s.borrow_mut()
        .save_dialog_ext(".", state.edit.name.text().to_lowercase(), "rs")?;
    ctx.dlg.push(
        file_dialog_render(
            LayoutOuter::new()
                .left(Constraint::Percentage(19))
                .right(Constraint::Percentage(19))
                .top(Constraint::Length(4))
                .bottom(Constraint::Length(4)),
            ctx.theme.style(WidgetStyle::FILE_DIALOG),
        ),
        file_dialog_event(|p| match p {
            Ok(p) => PalEvent::Export(p),
            Err(_) => PalEvent::NoOp,
        }),
        s,
    );
    Ok(Control::Changed)
}

fn export_pal_file(
    path: &Path,
    state: &mut Scenery,
    _ctx: &mut Global,
) -> Result<Control<PalEvent>, Error> {
    use std::io::Write;

    let palette = state.edit.palette();

    let c32 = Palette::color2u32;

    let mut wr = File::create(path)?;
    writeln!(wr, "use crate::Palette;")?;
    writeln!(wr, "")?;
    writeln!(wr, "// {}", palette.name)?;
    writeln!(wr, "const DARKNESS: u8 = 63;")?;
    writeln!(wr, "")?;
    writeln!(
        wr,
        "pub const {}: Palette = Palette {{",
        palette.name.to_uppercase()
    )?;
    writeln!(wr, "    name: \"{}\", ", palette.name)?;
    writeln!(wr, "")?;
    writeln!(
        wr,
        "    text_dark: Palette::color32({:#08x}), ",
        c32(palette.text_dark)
    )?;
    writeln!(
        wr,
        "    text_black: Palette::color32({:#08x}), ",
        c32(palette.text_black)
    )?;
    writeln!(
        wr,
        "    text_light: Palette::color32({:#08x}), ",
        c32(palette.text_light)
    )?;
    writeln!(
        wr,
        "    text_bright: Palette::color32({:#08x}), ",
        c32(palette.text_bright)
    )?;
    writeln!(wr, "")?;
    writeln!(
        wr,
        "    primary: Palette::interpolate({:#08x}, {:#08x}, DARKNESS), ",
        c32(palette.primary[0]),
        c32(palette.primary[3])
    )?;
    writeln!(
        wr,
        "    secondary: Palette::interpolate({:#08x}, {:#08x}, DARKNESS), ",
        c32(palette.secondary[0]),
        c32(palette.secondary[3])
    )?;
    writeln!(wr, "")?;
    writeln!(
        wr,
        "    white: Palette::interpolate({:#08x}, {:#08x}, DARKNESS), ",
        c32(palette.white[0]),
        c32(palette.white[3])
    )?;
    writeln!(
        wr,
        "    black: Palette::interpolate({:#08x}, {:#08x}, DARKNESS), ",
        c32(palette.black[0]),
        c32(palette.black[3])
    )?;
    writeln!(
        wr,
        "    gray: Palette::interpolate({:#08x}, {:#08x}, DARKNESS), ",
        c32(palette.gray[0]),
        c32(palette.gray[3])
    )?;
    writeln!(
        wr,
        "    red: Palette::interpolate({:#08x}, {:#08x}, DARKNESS), ",
        c32(palette.red[0]),
        c32(palette.red[3])
    )?;
    writeln!(
        wr,
        "    orange: Palette::interpolate({:#08x}, {:#08x}, DARKNESS), ",
        c32(palette.orange[0]),
        c32(palette.orange[3])
    )?;
    writeln!(
        wr,
        "    yellow: Palette::interpolate({:#08x}, {:#08x}, DARKNESS), ",
        c32(palette.yellow[0]),
        c32(palette.yellow[3])
    )?;
    writeln!(
        wr,
        "    limegreen: Palette::interpolate({:#08x}, {:#08x}, DARKNESS), ",
        c32(palette.limegreen[0]),
        c32(palette.limegreen[3])
    )?;
    writeln!(
        wr,
        "    green: Palette::interpolate({:#08x}, {:#08x}, DARKNESS), ",
        c32(palette.green[0]),
        c32(palette.green[3])
    )?;
    writeln!(
        wr,
        "    bluegreen: Palette::interpolate({:#08x}, {:#08x}, DARKNESS), ",
        c32(palette.bluegreen[0]),
        c32(palette.bluegreen[3])
    )?;
    writeln!(
        wr,
        "    cyan: Palette::interpolate({:#08x}, {:#08x}, DARKNESS), ",
        c32(palette.cyan[0]),
        c32(palette.cyan[3])
    )?;
    writeln!(
        wr,
        "    blue: Palette::interpolate({:#08x}, {:#08x}, DARKNESS), ",
        c32(palette.blue[0]),
        c32(palette.blue[3])
    )?;
    writeln!(
        wr,
        "    deepblue: Palette::interpolate({:#08x}, {:#08x}, DARKNESS), ",
        c32(palette.deepblue[0]),
        c32(palette.deepblue[3])
    )?;
    writeln!(
        wr,
        "    purple: Palette::interpolate({:#08x}, {:#08x}, DARKNESS), ",
        c32(palette.purple[0]),
        c32(palette.purple[3])
    )?;
    writeln!(
        wr,
        "    magenta: Palette::interpolate({:#08x}, {:#08x}, DARKNESS), ",
        c32(palette.magenta[0]),
        c32(palette.magenta[3])
    )?;
    writeln!(
        wr,
        "    redpink: Palette::interpolate({:#08x}, {:#08x}, DARKNESS), ",
        c32(palette.redpink[0]),
        c32(palette.redpink[3])
    )?;
    writeln!(wr, "}};")?;

    Ok(Control::Changed)
}

fn save_pal(state: &mut Scenery, ctx: &mut Global) -> Result<Control<PalEvent>, Error> {
    let s = state.file_dlg.clone();
    s.borrow_mut()
        .save_dialog_ext(".", state.edit.name.text().to_lowercase(), "pal")?;
    ctx.dlg.push(
        file_dialog_render(
            LayoutOuter::new()
                .left(Constraint::Percentage(19))
                .right(Constraint::Percentage(19))
                .top(Constraint::Length(4))
                .bottom(Constraint::Length(4)),
            ctx.theme.style(WidgetStyle::FILE_DIALOG),
        ),
        file_dialog_event(|p| match p {
            Ok(p) => PalEvent::Save(p),
            Err(_) => PalEvent::NoOp,
        }),
        s,
    );
    Ok(Control::Changed)
}

fn save_pal_file(
    path: &Path,
    state: &mut Scenery,
    _ctx: &mut Global,
) -> Result<Control<PalEvent>, Error> {
    let palette = state.edit.palette();

    let mut ff = Ini::new_std();
    ff.set_text("palette", "name", palette.name);
    ff.set_text("palette", "text_dark", palette.text_dark);
    ff.set_text("palette", "text_black", palette.text_black);
    ff.set_text("palette", "text_light", palette.text_light);
    ff.set_text("palette", "text_bright", palette.text_bright);
    ff.set_array("palette", "white", palette.white);
    ff.set_array("palette", "black", palette.black);
    ff.set_array("palette", "gray", palette.gray);
    ff.set_array("palette", "red", palette.red);
    ff.set_array("palette", "orange", palette.orange);
    ff.set_array("palette", "yellow", palette.yellow);
    ff.set_array("palette", "limegreen", palette.limegreen);
    ff.set_array("palette", "green", palette.green);
    ff.set_array("palette", "bluegreen", palette.bluegreen);
    ff.set_array("palette", "cyan", palette.cyan);
    ff.set_array("palette", "blue", palette.blue);
    ff.set_array("palette", "deepblue", palette.deepblue);
    ff.set_array("palette", "purple", palette.purple);
    ff.set_array("palette", "magenta", palette.magenta);
    ff.set_array("palette", "redpink", palette.redpink);
    ff.set_array("palette", "primary", palette.primary);
    ff.set_array("palette", "secondary", palette.secondary);
    ff.write_std(path)?;

    Ok(Control::Changed)
}

fn load_pal(state: &mut Scenery, ctx: &mut Global) -> Result<Control<PalEvent>, Error> {
    let s = state.file_dlg.clone();
    s.borrow_mut().open_dialog(".")?;
    ctx.dlg.push(
        file_dialog_render(
            LayoutOuter::new()
                .left(Constraint::Percentage(19))
                .right(Constraint::Percentage(19))
                .top(Constraint::Length(4))
                .bottom(Constraint::Length(4)),
            ctx.theme.style(WidgetStyle::FILE_DIALOG),
        ),
        file_dialog_event(|p| match p {
            Ok(p) => PalEvent::Load(p),
            Err(_) => PalEvent::NoOp,
        }),
        s,
    );
    Ok(Control::Changed)
}

fn load_pal_file(
    path: &Path,
    state: &mut Scenery,
    _ctx: &mut Global,
) -> Result<Control<PalEvent>, Error> {
    let mut ff = Ini::new_std();
    match ff.load(path) {
        Ok(_) => {}
        Err(e) => return Err(anyhow!(e)),
    };

    let mut palette = Palette::default();
    let name = Box::from(ff.get_text("palette", "name", ""));
    let name = Box::leak(name);
    palette.name = name;
    palette.text_dark = ff.parse_val("palette", "text_dark", Color::default());
    palette.text_black = ff.parse_val("palette", "text_black", Color::default());
    palette.text_light = ff.parse_val("palette", "text_light", Color::default());
    palette.text_bright = ff.parse_val("palette", "text_bright", Color::default());
    palette.white = ff.parse_array("palette", "white", Color::default());
    palette.black = ff.parse_array("palette", "black", Color::default());
    palette.gray = ff.parse_array("palette", "gray", Color::default());
    palette.red = ff.parse_array("palette", "red", Color::default());
    palette.orange = ff.parse_array("palette", "orange", Color::default());
    palette.yellow = ff.parse_array("palette", "yellow", Color::default());
    palette.limegreen = ff.parse_array("palette", "limegreen", Color::default());
    palette.green = ff.parse_array("palette", "green", Color::default());
    palette.bluegreen = ff.parse_array("palette", "bluegreen", Color::default());
    palette.cyan = ff.parse_array("palette", "cyan", Color::default());
    palette.blue = ff.parse_array("palette", "blue", Color::default());
    palette.deepblue = ff.parse_array("palette", "deepblue", Color::default());
    palette.purple = ff.parse_array("palette", "purple", Color::default());
    palette.magenta = ff.parse_array("palette", "magenta", Color::default());
    palette.redpink = ff.parse_array("palette", "redpink", Color::default());
    palette.primary = ff.parse_array("palette", "primary", Color::default());
    palette.secondary = ff.parse_array("palette", "secondary", Color::default());

    state.edit.set_palette(palette);
    let theme = create_edit_theme(state);
    state.show.set_theme(theme);

    Ok(Control::Changed)
}

fn create_edit_theme(state: &Scenery) -> SalsaTheme {
    let palette = state.edit.palette();
    match state.themes.value().as_str() {
        "Shell" => shell_theme("Shell", palette),
        "Fallback" => fallback_theme("Fallback", palette),
        _ => dark_theme("Dark", palette),
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
    use crate::{ColorSpan, ColorSpanState, Global};
    use anyhow::Error;
    use rat_event::{break_flow, MouseOnly, Outcome};
    use rat_focus::{FocusFlag, HasFocus};
    use rat_theme4::{Palette, WidgetStyle};
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

    #[derive(Debug)]
    pub struct PaletteEdit {
        pub palette: Palette,

        pub form: ClipperState,
        pub name: TextInputState,

        pub primary0: ColorInputState,
        pub primary3: ColorInputState,
        pub secondary0: ColorInputState,
        pub secondary3: ColorInputState,

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
                primary0: ColorInputState::named("primary0"),
                primary3: ColorInputState::named("primary3"),
                secondary0: ColorInputState::named("secondary0"),
                secondary3: ColorInputState::named("secondary3"),
            };

            z.text_light.set_value(Color::Rgb(255, 255, 255));
            z.text_bright.set_value(Color::Rgb(255, 255, 255));
            z.text_dark.set_value(Color::Rgb(0, 0, 0));
            z.text_black.set_value(Color::Rgb(0, 0, 0));

            z.primary0.set_value(Color::Rgb(255, 255, 255));
            z.primary3.set_value(Color::Rgb(255, 255, 255));
            z.secondary0.set_value(Color::Rgb(170, 170, 170));
            z.secondary3.set_value(Color::Rgb(170, 170, 170));

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
        fn paired_colors(&self) -> [(&'static str, &ColorInputState, &ColorInputState); 17] {
            [
                ("Primary", &self.primary0, &self.primary3),
                ("Secondary", &self.secondary0, &self.secondary3),
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
        ) -> [(&str, &mut ColorInputState, &mut ColorInputState); 17] {
            [
                ("Primary", &mut self.primary0, &mut self.primary3),
                ("Secondary", &mut self.secondary0, &mut self.secondary3),
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
            let name = Box::from(self.name.text());
            let name = Box::leak(name);
            palette.name = name;
            palette.text_light = self.text_light.value();
            palette.text_bright = self.text_bright.value();
            palette.text_dark = self.text_dark.value();
            palette.text_black = self.text_black.value();
            palette.primary =
                Palette::interpolate(self.primary0.value_u32(), self.primary3.value_u32(), 64);
            palette.secondary =
                Palette::interpolate(self.secondary0.value_u32(), self.secondary3.value_u32(), 64);
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
            palette
        }

        pub fn set_palette(&mut self, palette: Palette) {
            self.name.set_value(palette.name);
            self.text_light.set_value(palette.text_light);
            self.text_bright.set_value(palette.text_bright);
            self.text_dark.set_value(palette.text_dark);
            self.text_black.set_value(palette.text_black);
            self.primary0.set_value(palette.primary[0]);
            self.primary3.set_value(palette.primary[3]);
            self.secondary0.set_value(palette.secondary[0]);
            self.secondary3.set_value(palette.secondary[3]);
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

    impl HasFocus for PaletteEdit {
        fn build(&self, builder: &mut FocusBuilder) {
            builder.widget(&self.name);
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
            layout.widget(state.text_light.id(), L::Str("Text light"), W::Width(33));
            layout.widget(state.text_dark.id(), L::Str("Text dark"), W::Width(33));
            layout.gap(1);
            layout.widget(state.primary0.id(), L::Str("Primary"), W::Width(50));
            layout.widget(state.secondary0.id(), L::Str("Secondary"), W::Width(50));
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

        form.finish(buf, &mut state.form);

        Ok(())
    }

    pub fn event(
        event: &crossterm::event::Event,
        state: &mut PaletteEdit,
        _ctx: &mut Global,
    ) -> Result<Outcome, Error> {
        let mut mode_change = None;

        let r = 'f: {
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

            Outcome::Continue
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

mod showcase {
    use crate::Global;
    use anyhow::Error;
    use log::debug;
    use pure_rust_locales::{locale_match, Locale};
    use rat_event::{try_flow, HandleEvent, Outcome, Popup, Regular};
    use rat_focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
    use rat_theme4::{dark_theme, SalsaTheme, StyleName, WidgetStyle};
    use rat_widget::button::{Button, ButtonState};
    use rat_widget::calendar::selection::SingleSelection;
    use rat_widget::calendar::{CalendarState, Month};
    use rat_widget::checkbox::{Checkbox, CheckboxState};
    use rat_widget::choice::{Choice, ChoiceState};
    use rat_widget::clipper::{Clipper, ClipperState};
    use rat_widget::combobox::{Combobox, ComboboxState};
    use rat_widget::date_input::{DateInput, DateInputState};
    use rat_widget::event::{ButtonOutcome, ChoiceOutcome};
    use rat_widget::layout::LayoutForm;
    use rat_widget::number_input::{NumberInput, NumberInputState};
    use rat_widget::paired::{PairSplit, Paired, PairedState, PairedWidget};
    use rat_widget::radio::{Radio, RadioLayout, RadioState};
    use rat_widget::scrolled::Scroll;
    use rat_widget::slider::{Slider, SliderState};
    use rat_widget::text::{HasScreenCursor, TextFocusLost};
    use rat_widget::text_input::{TextInput, TextInputState};
    use rat_widget::textarea::{TextArea, TextAreaState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Flex, Rect};
    use ratatui::style::Style;
    use ratatui::symbols::border;
    use ratatui::text::Line;
    use ratatui::widgets::{Block, Borders, Padding};

    #[derive(Debug)]
    pub struct ShowCase {
        pub theme: SalsaTheme,
        pub loc: Locale,

        pub form: ClipperState,

        pub button: ButtonState,
        pub checkbox: CheckboxState,
        pub choice: ChoiceState,
        pub combobox: ComboboxState,
        pub date_input: DateInputState,
        pub number_input: NumberInputState,
        pub radio: RadioState,
        pub slider: SliderState<usize>,
        pub text: TextInputState,
        pub textarea: TextAreaState,
        pub calendar: CalendarState<1, SingleSelection>,
    }

    impl ShowCase {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn set_theme(&mut self, theme: SalsaTheme) {
            self.theme = theme;
        }
    }

    impl HasFocus for ShowCase {
        fn build(&self, builder: &mut FocusBuilder) {
            builder.widget(&self.button);
            builder.widget(&self.checkbox);
            builder.widget(&self.choice);
            builder.widget(&self.combobox);
            builder.widget(&self.date_input);
            builder.widget(&self.number_input);
            builder.widget(&self.radio);
            builder.widget(&self.slider);
            builder.widget(&self.text);
            builder.widget_navigate(&self.textarea, Navigation::Regular);
            builder.widget(&self.calendar);
        }

        fn focus(&self) -> FocusFlag {
            unimplemented!("no available")
        }

        fn area(&self) -> Rect {
            unimplemented!("no available")
        }
    }

    impl HasScreenCursor for ShowCase {
        fn screen_cursor(&self) -> Option<(u16, u16)> {
            self.combobox
                .screen_cursor()
                .or(self.date_input.screen_cursor())
                .or(self.number_input.screen_cursor())
                .or(self.text.screen_cursor())
                .or(self.textarea.screen_cursor())
                .or(self.calendar.screen_cursor())
        }
    }

    impl Default for ShowCase {
        fn default() -> Self {
            let mut z = Self {
                theme: Default::default(),
                loc: Default::default(),
                form: ClipperState::named("show"),
                button: ButtonState::named("button"),
                checkbox: CheckboxState::named("checkbox"),
                choice: ChoiceState::named("choice"),
                combobox: ComboboxState::named("combobox"),
                date_input: DateInputState::named("date_input"),
                number_input: NumberInputState::named("number_input"),
                radio: RadioState::named("radio"),
                slider: SliderState::<usize>::named("slider"),
                text: TextInputState::named("text"),
                textarea: TextAreaState::named("textarea"),
                calendar: CalendarState::named("calendar"),
            };

            let loc = sys_locale::get_locale().expect("locale");
            let loc = loc.replace("-", "_");
            z.loc = Locale::try_from(loc.as_str()).expect("locale");
            let fmt = locale_match!(z.loc => LC_TIME::D_FMT);
            z.date_input
                .set_format_loc(fmt, z.loc)
                .expect("date_format");
            z.number_input
                .set_format_loc("###,##0.00#", z.loc)
                .expect("number_format");
            z.theme = dark_theme("Tundra Dark", rat_theme4::palettes::TUNDRA);
            z.calendar.move_to_today();
            z
        }
    }

    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        state: &mut ShowCase,
        _ctx: &mut Global,
    ) -> Result<(), Error> {
        let mut form = Clipper::new() //
            .vscroll(Scroll::new())
            .buffer_uses_view_size()
            .styles(state.theme.style(WidgetStyle::CLIPPER));

        let layout_size = form.layout_size(area, &mut state.form);

        if !state.form.valid_layout(layout_size) {
            use rat_widget::layout::{FormLabel as L, FormWidget as W};
            let mut layout = LayoutForm::<usize>::new()
                .spacing(1)
                .line_spacing(1)
                .padding(Padding::new(1, 1, 1, 1))
                .flex(Flex::Start);
            layout.widget(state.button.id(), L::Str("Button"), W::Width(11));
            layout.widget(state.checkbox.id(), L::Str("Checkbox"), W::Width(14));
            layout.widget(state.choice.id(), L::Str("Choice"), W::Width(14));
            layout.widget(state.combobox.id(), L::Str("Combobox"), W::Width(14));
            layout.widget(state.date_input.id(), L::Str("DateInput"), W::Width(11));
            layout.widget(state.number_input.id(), L::Str("NumberInput"), W::Width(10));
            layout.widget(state.radio.id(), L::Str("Radio"), W::Width(25));
            layout.widget(state.slider.id(), L::Str("Slider"), W::Width(15));
            layout.widget(state.text.id(), L::Str("TextInput"), W::Width(20));
            layout.widget(state.textarea.id(), L::Str("TextArea"), W::Size(25, 5));
            layout.widget(state.calendar.id(), L::Str("Calendar"), W::Size(25, 8));
            form = form.layout(layout.build_endless(layout_size.width));
        }
        let mut form = form.into_buffer(area, &mut state.form);

        form.render(
            state.button.id(),
            || Button::new("Ok").styles(state.theme.style(WidgetStyle::BUTTON)),
            &mut state.button,
        );
        form.render(
            state.checkbox.id(),
            || {
                Checkbox::new()
                    .text("rat-salsa")
                    .styles(state.theme.style(WidgetStyle::CHECKBOX))
            },
            &mut state.checkbox,
        );
        let choice_popup = form.render2(
            state.choice.id(),
            || {
                Choice::new()
                    .items([
                        (0, "Zero"),
                        (1, "One"),
                        (2, "Two"),
                        (3, "Three"),
                        (4, "Four"),
                    ])
                    // .popup_placement(Placement::Right)
                    .styles(state.theme.style(WidgetStyle::CHOICE))
                    .into_widgets()
            },
            &mut state.choice,
        );
        let combo_popup = form.render2(
            state.combobox.id(),
            || {
                Combobox::new()
                    .items([
                        ("a".to_string(), "Alpha"),
                        ("b".to_string(), "Beta"),
                        ("g".to_string(), "Gamma"),
                    ])
                    .styles(state.theme.style(WidgetStyle::COMBOBOX))
                    .into_widgets()
            },
            &mut state.combobox,
        );
        form.render(
            state.date_input.id(),
            || {
                DateInput::new()
                    .on_focus_lost(TextFocusLost::Position0)
                    .styles(state.theme.style(WidgetStyle::TEXT))
            },
            &mut state.date_input,
        );
        form.render(
            state.number_input.id(),
            || NumberInput::new().styles(state.theme.style(WidgetStyle::TEXT)),
            &mut state.number_input,
        );
        form.render(
            state.radio.id(),
            || {
                Radio::new()
                    .direction(Direction::Horizontal)
                    .layout(RadioLayout::Stacked)
                    .items([(0, "abc"), (1, "def"), (2, "ghi"), (3, "jkl")])
                    .styles(state.theme.style(WidgetStyle::RADIO))
            },
            &mut state.radio,
        );

        let val = format!("{}", state.slider.value());
        form.render(
            state.slider.id(),
            || {
                Paired::new(
                    Slider::new()
                        .range((0, 25))
                        .long_step(4)
                        .styles(state.theme.style(WidgetStyle::SLIDER)),
                    PairedWidget::new(Line::from(val)),
                )
                .split(PairSplit::Constrain(
                    Constraint::Fill(1),
                    Constraint::Length(3),
                ))
            },
            &mut PairedState::new(&mut state.slider, &mut ()),
        );
        form.render(
            state.text.id(),
            || TextInput::new().styles(state.theme.style(WidgetStyle::TEXT)),
            &mut state.text,
        );
        let text_area_focused = state.textarea.is_focused();
        form.render(
            state.textarea.id(),
            || {
                TextArea::new()
                    .vscroll(Scroll::new())
                    .styles(state.theme.style(WidgetStyle::TEXTAREA))
                    .block(if text_area_focused {
                        Block::new()
                            .style(state.theme.style_style(Style::INPUT))
                            .border_style(state.theme.style_style(Style::FOCUS))
                            .borders(Borders::LEFT)
                            .border_set(border::EMPTY)
                    } else {
                        Block::default().style(state.theme.style_style(Style::INPUT))
                        // .border_style(state.theme.style_style(Style::INPUT))
                        // .borders(Borders::LEFT)
                        // .border_set(border::EMPTY)
                    })
            },
            &mut state.textarea,
        );
        form.render(
            state.calendar.id(),
            || {
                Month::new()
                    .locale(state.loc)
                    .styles(state.theme.style(WidgetStyle::MONTH))
            },
            &mut state.calendar.months[0],
        );

        form.render_popup(state.choice.id(), || choice_popup, &mut state.choice);
        form.render_popup(state.combobox.id(), || combo_popup, &mut state.combobox);
        form.finish(buf, &mut state.form);
        Ok(())
    }

    pub fn event(
        event: &crossterm::event::Event,
        state: &mut ShowCase,
        ctx: &mut Global,
    ) -> Result<Outcome, Error> {
        try_flow!(match state.choice.handle(event, Popup) {
            ChoiceOutcome::Changed => {
                debug!("choice changed");
                ChoiceOutcome::Changed
            }
            ChoiceOutcome::Value => {
                debug!("choice value");
                ChoiceOutcome::Value
            }
            r => r,
        });
        try_flow!(state.combobox.handle(event, Popup));

        try_flow!(match state.button.handle(event, Regular) {
            ButtonOutcome::Pressed => {
                ctx.status = "!!OK!!".to_string();
                Outcome::Changed
            }
            r => r.into(),
        });
        try_flow!(state.checkbox.handle(event, Regular));
        try_flow!(state.date_input.handle(event, Regular));
        try_flow!(state.number_input.handle(event, Regular));
        try_flow!(state.radio.handle(event, Regular));
        try_flow!(state.slider.handle(event, Regular));
        try_flow!(state.text.handle(event, Regular));
        try_flow!(state.textarea.handle(event, Regular));
        try_flow!(state.calendar.handle(event, Regular));

        try_flow!(state.form.handle(event, Regular));

        Ok(Outcome::Continue)
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

        let width = (area.width.saturating_sub(33)) / 8;
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

mod configparser_ext {
    use configparser::ini::{IniDefault, WriteOptions};
    use std::mem;
    use std::mem::MaybeUninit;
    use std::path::Path;
    use std::str::FromStr;

    /// Extensions to configparser for ease of use.
    pub(crate) trait ConfigParserExt {
        fn new_std() -> Self;

        /// Parse 'sec|val'
        fn parse_sec1<T: FromStr>(
            &self, //
            sec: &str,
            default: T,
        ) -> T;

        /// Parse 'sec|val1|val2'
        fn parse_sec2<T: FromStr, U: FromStr>(
            &self, //
            sec: &str,
            default1: T,
            default2: U,
        ) -> (T, U);

        /// Parse 'sec|val1|val2'
        fn parse_sec3<T: FromStr, U: FromStr, V: FromStr>(
            &self,
            sec: &str,
            default1: T,
            default2: U,
            default3: V,
        ) -> (T, U, V);

        /// Clean the sec by replacing '|' with '_'.
        /// This is probably not reversible.
        fn clean_sec(&self, key: &str) -> String;

        /// Create section key 'sec|val1'
        fn build_sec1<T: ToString>(
            &self, //
            sec: &str,
            val: T,
        ) -> String;

        /// Create section key 'sec|val1|val2'
        fn build_sec2<T: ToString, U: ToString>(
            &self, //
            sec: &str,
            val1: T,
            val2: U,
        ) -> String;

        /// Create section key 'sec|val1|val2|val3'
        fn build_sec3<T: ToString, U: ToString, V: ToString>(
            &self, //
            sec: &str,
            val1: T,
            val2: U,
            val3: V,
        ) -> String;

        /// Parse 'key.val'
        fn parse_key1<T: FromStr>(
            &self, //
            key: &str,
            default: T,
        ) -> T;

        /// Parse 'key.val1.val2'
        fn parse_key2<T: FromStr, U: FromStr>(
            &self, //
            key: &str,
            default1: T,
            default2: U,
        ) -> (T, U);

        /// Parse 'key.val1.val2'
        fn parse_key3<T: FromStr, U: FromStr, V: FromStr>(
            &self,
            key: &str,
            default1: T,
            default2: U,
            default3: V,
        ) -> (T, U, V);

        /// Clean the key by replacing '.' with '_'.
        /// This is probably not reversible.
        fn clean_key(&self, key: &str) -> String;

        /// Create section key 'key.val1'
        fn build_key1<T: ToString>(
            &self, //
            key: &str,
            val: T,
        ) -> String;

        /// Create section key 'key.val1.val2'
        fn build_key2<T: ToString, U: ToString>(
            &self, //
            key: &str,
            val1: T,
            val2: U,
        ) -> String;

        /// Create section key 'key.val1.val2.val3'
        fn build_key3<T: ToString, U: ToString, V: ToString>(
            &self, //
            key: &str,
            val1: T,
            val2: U,
            val3: V,
        ) -> String;

        /// Get the String value
        fn get_val<S: AsRef<str>, D: Into<String>>(
            &self, //
            sec: S,
            key: &str,
            default: D,
        ) -> String;

        /// Get multiline text.
        fn get_text<S: AsRef<str>, D: Into<String>>(
            &self, //
            sec: S,
            key: &str,
            default: D,
        ) -> String;

        fn parse_array<const N: usize, T: Copy + FromStr, S: AsRef<str>>(
            &mut self,
            sec: S,
            key: &str,
            default: T,
        ) -> [T; N];

        /// Call parse() for the value.
        fn parse_val<T: FromStr, S: AsRef<str>>(
            &self, //
            sec: S,
            key: &str,
            default: T,
        ) -> T;

        /// Call parse() for the value.
        fn parse_val_fallible<T: FromStr, S: AsRef<str>>(
            &self, //
            sec: S,
            key: &str,
            default: T,
        ) -> Result<T, <T as FromStr>::Err>;

        /// Parse a value with default.
        fn parse_val_with<T, S: AsRef<str>>(
            &self, //
            sec: S,
            key: &str,
            with: impl FnOnce(String) -> Option<T>,
            default: T,
        ) -> T;

        fn parse_val_with_fallible<T, E, S: AsRef<str>>(
            &self, //
            sec: S,
            key: &str,
            with: impl FnOnce(String) -> Result<T, E>,
            default: T,
        ) -> Result<T, E>;

        /// Set from some type.
        fn set_val<T: ToString, S: AsRef<str>>(
            &mut self, //
            sec: S,
            key: &str,
            val: T,
        );

        fn set_array<const N: usize, T: Copy + ToString, S: AsRef<str>>(
            &mut self,
            sec: S,
            key: &str,
            val: [T; N],
        );

        fn set_text<T: ToString, S: AsRef<str>>(
            &mut self, //
            sec: S,
            key: &str,
            val: T,
        );

        /// Iterate over one section.
        fn section_iter<S: AsRef<str>>(&self, sec: S) -> SectionIter<'_>;

        /// Write with our standards.
        fn write_std(&self, path: impl AsRef<Path>) -> std::io::Result<()>;
    }

    impl ConfigParserExt for configparser::ini::Ini {
        fn new_std() -> Self {
            let mut def = IniDefault::default();
            def.case_sensitive = true;
            def.multiline = false;
            def.comment_symbols = vec![];

            configparser::ini::Ini::new_from_defaults(def)
        }

        fn parse_sec1<T: FromStr>(&self, sec: &str, default: T) -> T {
            let mut s = sec.split('|');
            s.next();
            let t = match s.next() {
                None => {
                    panic!("invalid section: {:?}", sec)
                }
                Some(v) => v.parse().unwrap_or(default),
            };
            assert_eq!(s.next(), None);
            t
        }

        fn parse_sec2<T: FromStr, U: FromStr>(
            &self,
            sec: &str,
            default1: T,
            default2: U,
        ) -> (T, U) {
            let mut s = sec.split('|');
            s.next();
            let t = match s.next() {
                None => {
                    panic!("invalid section: {:?}", sec)
                }
                Some(v) => v.parse().unwrap_or(default1),
            };
            let u = match s.next() {
                None => {
                    panic!("invalid section: {:?}", sec)
                }
                Some(v) => v.parse().unwrap_or(default2),
            };
            assert_eq!(s.next(), None);
            (t, u)
        }

        fn parse_sec3<T: FromStr, U: FromStr, V: FromStr>(
            &self,
            sec: &str,
            default1: T,
            default2: U,
            default3: V,
        ) -> (T, U, V) {
            let mut s = sec.split('|');
            s.next();
            let t = match s.next() {
                None => {
                    panic!("invalid section: {:?}", sec)
                }
                Some(v) => v.parse().unwrap_or(default1),
            };
            let u = match s.next() {
                None => {
                    panic!("invalid section: {:?}", sec)
                }
                Some(v) => v.parse().unwrap_or(default2),
            };
            let v = match s.next() {
                None => {
                    panic!("invalid section: {:?}", sec)
                }
                Some(v) => v.parse().unwrap_or(default3),
            };
            assert_eq!(s.next(), None);
            (t, u, v)
        }

        fn clean_sec(&self, key: &str) -> String {
            key.replace('|', "_")
        }

        fn build_sec1<T: ToString>(&self, sec: &str, val: T) -> String {
            format!("{}|{}", sec, val.to_string())
        }

        fn build_sec2<T: ToString, U: ToString>(&self, sec: &str, val1: T, val2: U) -> String {
            format!("{}|{}|{}", sec, val1.to_string(), val2.to_string())
        }

        fn build_sec3<T: ToString, U: ToString, V: ToString>(
            &self,
            sec: &str,
            val1: T,
            val2: U,
            val3: V,
        ) -> String {
            format!(
                "{}|{}|{}|{}",
                sec,
                val1.to_string(),
                val2.to_string(),
                val3.to_string()
            )
        }

        fn parse_key1<T: FromStr>(&self, key: &str, default: T) -> T {
            let mut s = key.split('.');
            s.next();
            let t = match s.next() {
                None => {
                    panic!("invalid key: {:?}", key)
                }
                Some(v) => v.parse().unwrap_or(default),
            };
            assert_eq!(s.next(), None);
            t
        }

        fn parse_key2<T: FromStr, U: FromStr>(
            &self,
            key: &str,
            default1: T,
            default2: U,
        ) -> (T, U) {
            let mut s = key.split('.');
            s.next();
            let t = match s.next() {
                None => {
                    panic!("invalid key: {:?}", key)
                }
                Some(v) => v.parse().unwrap_or(default1),
            };
            let u = match s.next() {
                None => {
                    panic!("invalid key: {:?}", key)
                }
                Some(v) => v.parse().unwrap_or(default2),
            };
            assert_eq!(s.next(), None);
            (t, u)
        }

        fn parse_key3<T: FromStr, U: FromStr, V: FromStr>(
            &self,
            key: &str,
            default1: T,
            default2: U,
            default3: V,
        ) -> (T, U, V) {
            let mut s = key.split('.');
            s.next();
            let t = match s.next() {
                None => {
                    panic!("invalid key: {:?}", key)
                }
                Some(v) => v.parse().unwrap_or(default1),
            };
            let u = match s.next() {
                None => {
                    panic!("invalid key: {:?}", key)
                }
                Some(v) => v.parse().unwrap_or(default2),
            };
            let v = match s.next() {
                None => {
                    panic!("invalid key: {:?}", key)
                }
                Some(v) => v.parse().unwrap_or(default3),
            };
            assert_eq!(s.next(), None);
            (t, u, v)
        }

        fn clean_key(&self, key: &str) -> String {
            key.replace('.', "_")
        }

        fn build_key1<T: ToString>(&self, key: &str, val: T) -> String {
            format!("{}.{}", key, val.to_string())
        }

        fn build_key2<T: ToString, U: ToString>(&self, key: &str, val1: T, val2: U) -> String {
            format!("{}.{}.{}", key, val1.to_string(), val2.to_string())
        }

        fn build_key3<T: ToString, U: ToString, V: ToString>(
            &self,
            key: &str,
            val1: T,
            val2: U,
            val3: V,
        ) -> String {
            format!(
                "{}.{}.{}.{}",
                key,
                val1.to_string(),
                val2.to_string(),
                val3.to_string()
            )
        }

        fn get_val<S: AsRef<str>, D: Into<String>>(&self, sec: S, key: &str, default: D) -> String {
            self.get(sec.as_ref(), key).unwrap_or(default.into())
        }

        fn get_text<S: AsRef<str>, D: Into<String>>(
            &self,
            sec: S,
            key: &str,
            default: D,
        ) -> String {
            if let Some(s) = self.get(sec.as_ref(), key) {
                let mut buf = String::new();
                let mut esc = false;
                for c in s.chars() {
                    if c == '\\' {
                        if esc {
                            buf.push('\\');
                            esc = false;
                        } else {
                            esc = true;
                        }
                    } else if esc {
                        match c {
                            'r' => buf.push('\r'),
                            'n' => buf.push('\n'),
                            _ => {
                                buf.push('\\');
                                buf.push(c);
                            }
                        }
                        esc = false;
                    } else {
                        buf.push(c);
                    }
                }
                buf
            } else {
                default.into()
            }
        }

        fn parse_array<const N: usize, T: Copy + FromStr, S: AsRef<str>>(
            &mut self,
            sec: S,
            key: &str,
            default: T,
        ) -> [T; N] {
            let sec = sec.as_ref();
            let mut r = [MaybeUninit::uninit(); N];
            for (i, v) in r.iter_mut().enumerate() {
                v.write(self.parse_val(sec, format!("{}.{}", key, i).as_str(), default));
            }
            // Everything is initialized. Transmute the array to the
            // initialized type.
            unsafe { mem::transmute_copy::<[MaybeUninit<T>; N], [T; N]>(&r) }
        }

        fn parse_val<T: FromStr, S: AsRef<str>>(
            &self, //
            sec: S,
            key: &str,
            default: T,
        ) -> T {
            if let Some(v) = self.get(sec.as_ref(), key) {
                v.parse::<T>().unwrap_or(default)
            } else {
                default
            }
        }

        fn parse_val_fallible<T: FromStr, S: AsRef<str>>(
            &self, //
            sec: S,
            key: &str,
            default: T,
        ) -> Result<T, <T as FromStr>::Err> {
            if let Some(v) = self.get(sec.as_ref(), key) {
                v.parse::<T>()
            } else {
                Ok(default)
            }
        }

        fn parse_val_with<T, S: AsRef<str>>(
            &self, //
            sec: S,
            key: &str,
            with: impl FnOnce(String) -> Option<T>,
            default: T,
        ) -> T {
            if let Some(v) = self.get(sec.as_ref(), key) {
                with(v).unwrap_or(default)
            } else {
                default
            }
        }

        fn parse_val_with_fallible<T, E, S: AsRef<str>>(
            &self, //
            sec: S,
            key: &str,
            with: impl FnOnce(String) -> Result<T, E>,
            default: T,
        ) -> Result<T, E> {
            if let Some(v) = self.get(sec.as_ref(), key) {
                with(v)
            } else {
                Ok(default)
            }
        }

        fn set_val<T: ToString, S: AsRef<str>>(&mut self, sec: S, key: &str, val: T) {
            self.set(sec.as_ref(), key, Some(val.to_string()));
        }

        fn set_array<const N: usize, T: Copy + ToString, S: AsRef<str>>(
            &mut self,
            sec: S,
            key: &str,
            val: [T; N],
        ) {
            let sec = sec.as_ref();
            for i in 0..N {
                self.set_text(sec, format!("{}.{}", key, i).as_str(), val[i]);
            }
        }

        fn set_text<T: ToString, S: AsRef<str>>(&mut self, sec: S, key: &str, val: T) {
            let mut buf = String::new();
            for c in val.to_string().chars() {
                if c == '\r' {
                    // skip
                } else if c == '\n' {
                    buf.push_str("\\n");
                } else {
                    buf.push(c)
                }
            }
            self.set(sec.as_ref(), key, Some(buf));
        }

        fn section_iter<S: AsRef<str>>(&self, sec: S) -> SectionIter<'_> {
            SectionIter {
                it: if let Some(map) = self.get_map_ref().get(sec.as_ref()) {
                    Some(map.iter())
                } else {
                    None
                },
            }
        }

        fn write_std(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
            self.pretty_write(path, &WriteOptions::new_with_params(false, 4, 1))
        }
    }

    pub(crate) struct SectionIter<'a> {
        it: Option<indexmap::map::Iter<'a, String, Option<String>>>,
    }

    impl<'a> Iterator for SectionIter<'a> {
        type Item = (&'a String, &'a Option<String>);

        fn next(&mut self) -> Option<Self::Item> {
            if let Some(it) = &mut self.it {
                it.next()
            } else {
                None
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
