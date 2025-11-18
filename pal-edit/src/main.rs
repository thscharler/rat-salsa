mod base46;
mod clipboard;
mod color_span;
mod configparser_ext;
mod message;
mod palette_edit;
mod sample_data_input;
mod sample_or_base46;
mod sample_other;
mod sample_readability;
mod show_sample;

use crate::clipboard::CliClipboard;
use crate::configparser_ext::ConfigParserExt;
use crate::message::{MsgState, msg_event, msg_render};
use crate::palette_edit::PaletteEdit;
use crate::sample_or_base46::ShowOrBase46;
use anyhow::{Error, anyhow};
use configparser::ini::Ini;
use dirs::config_dir;
use log::error;
use pure_rust_locales::Locale;
use rat_salsa::dialog_stack::DialogStack;
use rat_salsa::dialog_stack::file_dialog::{file_dialog_event, file_dialog_render};
use rat_salsa::event::RenderedEvent;
use rat_salsa::poll::{PollCrossterm, PollRendered};
use rat_salsa::{Control, RunConfig, SalsaAppContext, SalsaContext, run_tui};
use rat_theme4::{
    ColorIdx, Colors, Palette, RatWidgetColor, SalsaTheme, WidgetStyle, create_theme, dark_theme,
    rat_widget_color_names, shell_theme,
};
use rat_widget::event::{HandleEvent, MenuOutcome, Outcome, Regular, ct_event, event_flow};
use rat_widget::file_dialog::FileDialogState;
use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_widget::layout::LayoutOuter;
use rat_widget::menu::{MenuLine, MenuLineState};
use rat_widget::statusline_stacked::StatusLineStacked;
use rat_widget::text::HasScreenCursor;
use rat_widget::text::clipboard::set_global_clipboard;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{StatefulWidget, Widget};
use std::cell::RefCell;
use std::env::args;
use std::fs::{File, create_dir_all};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;
use std::{array, fs};
use try_as_traits::TryAsRef;

fn main() -> Result<(), Error> {
    let mut extra_alias = Vec::new();
    let mut args = args();
    _ = args.next();
    if let Some(attr_file) = args.next() {
        let path = PathBuf::from(attr_file);
        let path = path.canonicalize()?;

        let mut buf = String::new();

        let mut f = File::open(path)?;
        f.read_to_string(&mut buf)?;

        for l in buf.lines() {
            extra_alias.push(l.trim().to_string());
        }
    }

    setup_logging()?;
    set_global_clipboard(CliClipboard::default());
    let config = Config::load()?;

    let theme = create_theme("Shell");

    let mut global = Global::new(config, theme);
    let mut state = Scenery::new(&global.cfg);

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
    pub show_theme: SalsaTheme,

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
            show_theme: Default::default(),
            status_frame: 0,
            status: Default::default(),
        }
    }
}

/// Configuration.
#[derive(Debug, Default)]
pub struct Config {
    pub loc: Locale,
    pub extra_alias: Vec<String>,
}

impl Config {
    pub fn aliases(&self) -> Vec<String> {
        let mut r = Vec::new();
        r.extend(rat_widget_color_names().iter().map(|v| v.to_string()));
        r.extend(self.extra_alias.iter().cloned());
        r
    }

    pub fn load() -> Result<Config, Error> {
        let loc = sys_locale::get_locale().expect("locale");
        let loc = loc.replace("-", "_");
        let loc = Locale::try_from(loc.as_str()).expect("locale");

        let extra_alias = if let Some(cfg_dir) = config_dir() {
            let mut aliases = Vec::new();

            let cfg_path = cfg_dir.join("pal-edit");
            let cfg_file = cfg_path.join("pal-edit.ini");
            if cfg_file.exists() {
                let mut ini = Ini::new();
                match ini.load(cfg_file) {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(anyhow!(e));
                    }
                }

                if let Some(map) = ini.get_map_ref().get("aliases") {
                    for alias in map.keys() {
                        aliases.push(alias.trim().into());
                    }
                }
            } else {
                create_dir_all(cfg_path)?;
                let mut f = File::create_new(cfg_file)?;
                f.write_all(
                    b"[aliases]
# add extra color aliases here.
# just names, no values needed.
# sample
",
                )?;
            }

            aliases
        } else {
            Vec::new()
        };

        Ok(Config { loc, extra_alias })
    }
}

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
    Base46(PathBuf),
    ContainerBase(ColorIdx),
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
    pub file_load_dlg: Rc<RefCell<FileDialogState>>,
    pub file_save_dlg: Rc<RefCell<FileDialogState>>,
    pub file_dlg_export: Rc<RefCell<FileDialogState>>,
    pub file_dlg_import: Rc<RefCell<FileDialogState>>,
    pub file_path: Option<PathBuf>,

    pub edit: PaletteEdit,
    pub detail: ShowOrBase46,
    pub menu: MenuLineState,
    pub menu_return_focus: Option<FocusFlag>,
}

impl Scenery {
    pub fn new(cfg: &Config) -> Self {
        Self {
            file_load_dlg: Rc::new(RefCell::new(FileDialogState::default())),
            file_save_dlg: Rc::new(RefCell::new(FileDialogState::default())),
            file_dlg_export: Rc::new(RefCell::new(FileDialogState::default())),
            file_dlg_import: Rc::new(RefCell::new(FileDialogState::default())),
            file_path: None,
            edit: PaletteEdit::new(cfg),
            detail: ShowOrBase46::new(cfg),
            menu: MenuLineState::named("menu"),
            menu_return_focus: Default::default(),
        }
    }
}

impl HasFocus for Scenery {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget(&self.edit);
        builder.widget(&self.detail);
        builder.widget_navigate(&self.menu, Navigation::Leave);
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
        Constraint::Fill(1), //
        Constraint::Length(1),
    ])
    .split(area);
    let l2 = Layout::horizontal([
        Constraint::Length(69), //
        Constraint::Fill(1),    //
    ])
    .horizontal_margin(1)
    .flex(Flex::Center)
    .split(l1[0]);

    // main
    palette_edit::render(l2[0], buf, &mut state.edit, ctx)?;
    sample_or_base46::render(l2[1], buf, &mut state.detail, ctx)?;
    screen_cursor(state, ctx);

    // menu & status
    let status_layout = Layout::horizontal([
        Constraint::Fill(61), //
        Constraint::Fill(39),
    ])
    .split(l1[1]);

    render_menu(status_layout[0], buf, state, ctx)?;
    render_status(status_layout[1], buf, ctx)?;

    // dialog windows
    ctx.dlg.clone().render(area, buf, ctx);

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
        .item_parsed("_New")
        .item_parsed("_Load")
        .item_parsed("_Save")
        .item_parsed("_Save as")
        .item_parsed("_Export")
        .item_parsed("_Base46")
        .item_parsed("_Use46")
        .item_parsed("_Quit")
        .render(area, buf, &mut state.menu);
    Ok(())
}

fn render_status(area: Rect, buf: &mut Buffer, ctx: &mut Global) -> Result<(), Error> {
    let palette = &ctx.theme.p;
    let status_color_1 = palette
        .normal_contrast(palette.color(Colors::White, 0))
        .bg(palette.color(Colors::Blue, 3));
    let status_color_2 = palette
        .normal_contrast(palette.color(Colors::White, 0))
        .bg(palette.color(Colors::Blue, 2));
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
    let sc = state.edit.screen_cursor().or(state.detail.screen_cursor());
    ctx.set_screen_cursor(sc);
}

pub fn init(state: &mut Scenery, ctx: &mut Global) -> Result<(), Error> {
    ctx.set_focus(FocusBuilder::build_for(state));
    // ctx.focus().enable_log();
    ctx.focus().first();

    ctx.show_theme = create_edit_theme(state);

    Ok(())
}

pub fn event(
    event: &PalEvent,
    state: &mut Scenery,
    ctx: &mut Global,
) -> Result<Control<PalEvent>, Error> {
    if let PalEvent::Event(event) = event {
        event_flow!(match &event {
            ct_event!(resized) => Control::Changed,
            ct_event!(key press CONTROL-'q') => Control::Quit,
            _ => Control::Continue,
        });
    }

    event_flow!(ctx.dlg.clone().handle(event, ctx)?);

    if let PalEvent::Event(event) = event {
        match ctx.handle_focus(event) {
            Outcome::Changed => {
                state.edit.form.show_focused(&ctx.focus());
                state.detail.show_focused(&ctx.focus());
            }
            _ => {}
        }

        event_flow!(match palette_edit::event(event, &mut state.edit, ctx)? {
            Outcome::Changed => {
                ctx.show_theme = create_edit_theme(state);
                Outcome::Changed
            }
            r => r.into(),
        });
        event_flow!(sample_or_base46::event(event, &mut state.detail, ctx)?);

        event_flow!(match state.menu.handle(event, Regular) {
            MenuOutcome::Activated(0) => new_pal(state, ctx)?,
            MenuOutcome::Activated(1) => load_pal(state, ctx)?,
            MenuOutcome::Activated(2) => save_pal(state, ctx)?,
            MenuOutcome::Activated(3) => saveas_pal(state, ctx)?,
            MenuOutcome::Activated(4) => export_pal(state, ctx)?,
            MenuOutcome::Activated(5) => import_base46(state, ctx)?,
            MenuOutcome::Activated(6) => use_base46(state, ctx)?,
            MenuOutcome::Activated(7) => Control::Quit,
            v => v.into(),
        });

        event_flow!(match event {
            ct_event!(keycode press Esc) => {
                if state.menu.is_focused() {
                    let last = state
                        .menu_return_focus
                        .clone()
                        .unwrap_or(state.edit.name.focus());
                    ctx.focus().focus(&last);
                } else {
                    state.menu_return_focus = ctx.focus().focused();
                    ctx.focus().focus(&state.menu);
                }

                Control::Changed
            }
            _ => Control::Continue,
        })
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
        PalEvent::ContainerBase(c) => {
            state.detail.show.readability.colors.set_value(*c);
            Ok(Control::Changed)
        }
        PalEvent::Save(p) => save_pal_file(&p, state, ctx),
        PalEvent::Load(p) => {
            _ = load_pal_file(&p, state, ctx)?;
            if let Some(c) = state.edit.color_ext.get(Color::CONTAINER_BASE) {
                state.detail.show.readability.colors.set_value(c.value());
            }
            Ok(Control::Changed)
        }
        PalEvent::Export(p) => export_pal_file(&p, state, ctx),
        PalEvent::Base46(p) => import_base46_file(&p, state, ctx),
        _ => Ok(Control::Continue),
    }
}

pub fn pal_choice(pal: Palette) -> Vec<(ColorIdx, Line<'static>)> {
    const COLOR_X_8: usize = Colors::LEN * 8 + 1;
    let pal_choice = array::from_fn::<_, COLOR_X_8, _>(|n| {
        if n == Colors::LEN * 8 {
            let c = Colors::None;
            let n = 0;
            (c, n)
        } else {
            let c = Colors::array()[n / 8];
            let n = n % 8;
            (c, n)
        }
    });
    pal_choice
        .iter()
        .map(|(c, n)| {
            (
                ColorIdx(*c, *n),
                Line::from(format!("{}-{}", c, n)).style(pal.style(*c, *n)),
            )
        })
        .collect::<Vec<_>>()
}

fn import_base46(state: &mut Scenery, ctx: &mut Global) -> Result<Control<PalEvent>, Error> {
    let s = state.file_dlg_import.clone();
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
            Ok(p) => PalEvent::Base46(p),
            Err(_) => PalEvent::NoOp,
        }),
        s,
    );
    Ok(Control::Changed)
}

fn import_base46_file(
    path: &Path,
    state: &mut Scenery,
    _ctx: &mut Global,
) -> Result<Control<PalEvent>, Error> {
    let mut buf = String::new();
    {
        let mut f = File::open(path)?;
        f.read_to_string(&mut buf)?;
    }

    // quick and dirty parser
    let mut mode = 0;
    for l in buf.lines() {
        if l.starts_with("M.base_30") {
            mode = 1;
        } else if l.starts_with("M.base_16") {
            mode = 1;
        } else if l.starts_with("}") {
            mode = 0;
        } else if mode == 1 {
            let l = l.trim();
            let mut it = l.split(['=', ',']);
            let Some(name) = it.next() else {
                continue;
            };
            let name = name.trim();
            let Some(color) = it.next() else {
                continue;
            };
            let color = color.trim_matches([' ', '"']);
            let Ok(color) = Color::from_str(color) else {
                continue;
            };

            match name {
                "white" => state.detail.base46.white.set_value(color),
                "darker_black" => state.detail.base46.darker_black.set_value(color),
                "black" => state.detail.base46.black.set_value(color),
                "black2" => state.detail.base46.black2.set_value(color),
                "one_bg" => state.detail.base46.one_bg.set_value(color),
                "one_bg2" => state.detail.base46.one_bg2.set_value(color),
                "one_bg3" => state.detail.base46.one_bg3.set_value(color),
                "grey" => state.detail.base46.grey.set_value(color),
                "grey_fg" => state.detail.base46.grey_fg.set_value(color),
                "grey_fg2" => state.detail.base46.grey_fg2.set_value(color),
                "light_grey" => state.detail.base46.light_grey.set_value(color),
                "red" => state.detail.base46.red.set_value(color),
                "baby_pink" => state.detail.base46.baby_pink.set_value(color),
                "pink" => state.detail.base46.pink.set_value(color),
                "line" => state.detail.base46.line.set_value(color),
                "green" => state.detail.base46.green.set_value(color),
                "vibrant_green" => state.detail.base46.vibrant_green.set_value(color),
                "nord_blue" => state.detail.base46.nord_blue.set_value(color),
                "blue" => state.detail.base46.blue.set_value(color),
                "yellow" => state.detail.base46.yellow.set_value(color),
                "sun" => state.detail.base46.sun.set_value(color),
                "purple" => state.detail.base46.purple.set_value(color),
                "dark_purple" => state.detail.base46.dark_purple.set_value(color),
                "teal" => state.detail.base46.teal.set_value(color),
                "orange" => state.detail.base46.orange.set_value(color),
                "cyan" => state.detail.base46.cyan.set_value(color),
                "statusline_bg" => state.detail.base46.statusline_bg.set_value(color),
                "lightbg" => state.detail.base46.lightbg.set_value(color),
                "pmenu_bg" => state.detail.base46.pmenu_bg.set_value(color),
                "folder_bg" => state.detail.base46.folder_bg.set_value(color),
                "base00" => state.detail.base46.base00.set_value(color),
                "base01" => state.detail.base46.base01.set_value(color),
                "base02" => state.detail.base46.base02.set_value(color),
                "base03" => state.detail.base46.base03.set_value(color),
                "base04" => state.detail.base46.base04.set_value(color),
                "base05" => state.detail.base46.base05.set_value(color),
                "base06" => state.detail.base46.base06.set_value(color),
                "base07" => state.detail.base46.base07.set_value(color),
                "base08" => state.detail.base46.base08.set_value(color),
                "base09" => state.detail.base46.base09.set_value(color),
                "base0A" => state.detail.base46.base0A.set_value(color),
                "base0B" => state.detail.base46.base0B.set_value(color),
                "base0C" => state.detail.base46.base0C.set_value(color),
                "base0D" => state.detail.base46.base0D.set_value(color),
                "base0E" => state.detail.base46.base0E.set_value(color),
                "base0F" => state.detail.base46.base0F.set_value(color),
                _ => {}
            }
        }
    }

    Ok(Control::Changed)
}

fn export_pal(state: &mut Scenery, ctx: &mut Global) -> Result<Control<PalEvent>, Error> {
    let s = state.file_dlg_export.clone();
    s.borrow_mut()
        .save_dialog_ext(".", state.edit.file_name(), "rs")?;
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

    let c32 = Palette::color_to_u32;

    let mut wr = File::create(path)?;
    writeln!(wr, "use crate::{{ColorIdx, Colors, Palette}};")?;
    writeln!(wr, "")?;
    writeln!(wr, "/// {}", state.edit.name())?;
    for l in state.edit.docs.text().lines() {
        writeln!(wr, "/// {}", l)?;
    }
    writeln!(
        wr,
        "const DARKNESS: u8 = {};",
        state.edit.dark.value::<u8>().unwrap_or(64)
    )?;
    writeln!(wr, "")?;
    writeln!(
        wr,
        "pub const {}: Palette = Palette {{",
        state.edit.const_name(),
    )?;
    writeln!(wr, "    name: \"{}\", ", state.edit.name())?;
    writeln!(wr, "")?;
    writeln!(wr, "    color: [")?;
    for c in [Colors::TextLight, Colors::TextDark] {
        let c0 = state.edit.color[c as usize].0.value();
        let c3 = state.edit.color[c as usize].3.value();
        writeln!(
            wr,
            "        Palette::interpolate2({:#08x}, {:#08x}, 0x0, 0x0),",
            c32(c0),
            c32(c3)
        )?;
    }
    for c in Colors::array_no_text() {
        let c0 = state.edit.color[c as usize].0.value();
        let c3 = state.edit.color[c as usize].3.value();
        writeln!(
            wr,
            "        Palette::interpolate({:#08x}, {:#08x}, DARKNESS),",
            c32(c0),
            c32(c3)
        )?;
    }
    writeln!(wr, "    ],")?;
    writeln!(wr, "    // must be sorted!")?;
    writeln!(wr, "    aliased: &[")?;
    let aliased = state.edit.aliased();
    for (n, c) in aliased {
        writeln!(
            wr,
            "        ({:?}, ColorIdx(Colors::{:?}, {:?})),",
            n, c.0, c.1
        )?;
    }
    writeln!(wr, "    ],")?;
    writeln!(wr, "}};")?;
    writeln!(wr, "")?;

    Ok(Control::Changed)
}

fn saveas_pal(state: &mut Scenery, ctx: &mut Global) -> Result<Control<PalEvent>, Error> {
    let s = state.file_save_dlg.clone();
    s.borrow_mut()
        .save_dialog_ext(".", state.edit.file_name(), "pal")?;
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

fn save_pal(state: &mut Scenery, ctx: &mut Global) -> Result<Control<PalEvent>, Error> {
    if let Some(file_path) = state.file_path.clone() {
        save_pal_file(&file_path, state, ctx)
    } else {
        saveas_pal(state, ctx)
    }
}

fn save_pal_file(
    path: &Path,
    state: &mut Scenery,
    _ctx: &mut Global,
) -> Result<Control<PalEvent>, Error> {
    state.file_path = Some(path.into());

    let mut ff = Ini::new_std();
    ff.set_text("palette", "name", state.edit.name());
    ff.set_text("palette", "docs", state.edit.docs.text());
    ff.set_val(
        "palette",
        "dark",
        state.edit.dark.value::<u8>().unwrap_or(63),
    );
    for c in Colors::array() {
        ff.set_array(
            "color",
            c.name(),
            [
                state.edit.color[c as usize].0.value(),
                state.edit.color[c as usize].3.value(),
            ],
        );
    }

    for (c, s) in state.edit.color_ext.iter() {
        let c_idx = s.value();
        ff.set_val("reference", c, c_idx);
    }

    ff.write_std(path)?;

    Ok(Control::Changed)
}

fn new_pal(state: &mut Scenery, _ctx: &mut Global) -> Result<Control<PalEvent>, Error> {
    state.file_path = None;

    state.edit.name.set_value("pal.name");
    _ = state.edit.dark.set_value(64);

    for c in Colors::array() {
        state.edit.color[c as usize].0.set_value(Color::default());
        state.edit.color[c as usize].3.set_value(Color::default());
    }
    for (_, s) in state.edit.color_ext.iter_mut() {
        s.set_value(ColorIdx(Colors::default(), 0));
    }

    state
        .detail
        .show
        .readability
        .colors
        .set_value(ColorIdx(Colors::default(), 0));

    Ok(Control::Changed)
}

fn load_pal(state: &mut Scenery, ctx: &mut Global) -> Result<Control<PalEvent>, Error> {
    let s = state.file_load_dlg.clone();
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

fn use_base46(state: &mut Scenery, _ctx: &mut Global) -> Result<Control<PalEvent>, Error> {
    let v = state.detail.base46.white.value();
    state.edit.color[Colors::TextLight as usize].0.set_value(v);
    state.edit.color[Colors::TextLight as usize].3.set_value(v);
    let v = state.detail.base46.darker_black.value();
    state.edit.color[Colors::TextDark as usize].0.set_value(v);
    state.edit.color[Colors::TextDark as usize].3.set_value(v);

    let v = state.detail.base46.white.value();
    state.edit.color[Colors::White as usize].0.set_value(v);
    state.edit.color[Colors::White as usize].3.set_value(v);

    let v = state.detail.base46.grey.value();
    state.edit.color[Colors::Gray as usize].0.set_value(v);
    let v = state.detail.base46.light_grey.value();
    state.edit.color[Colors::Gray as usize].3.set_value(v);

    let v = state.detail.base46.darker_black.value();
    state.edit.color[Colors::Black as usize].0.set_value(v);
    let v = state.detail.base46.black2.value();
    state.edit.color[Colors::Black as usize].3.set_value(v);

    let v = state.detail.base46.red.value();
    state.edit.color[Colors::Red as usize].0.set_value(v);
    state.edit.color[Colors::Red as usize].3.set_value(v);
    let v = state.detail.base46.orange.value();
    state.edit.color[Colors::Orange as usize].0.set_value(v);
    state.edit.color[Colors::Orange as usize].3.set_value(v);
    let v = state.detail.base46.yellow.value();
    state.edit.color[Colors::Yellow as usize].0.set_value(v);
    state.edit.color[Colors::Yellow as usize].3.set_value(v);
    let v = state.detail.base46.vibrant_green.value();
    state.edit.color[Colors::LimeGreen as usize].0.set_value(v);
    state.edit.color[Colors::LimeGreen as usize].3.set_value(v);
    let v = state.detail.base46.green.value();
    state.edit.color[Colors::Green as usize].0.set_value(v);
    state.edit.color[Colors::Green as usize].3.set_value(v);
    let v = state.detail.base46.teal.value();
    state.edit.color[Colors::BlueGreen as usize].0.set_value(v);
    state.edit.color[Colors::BlueGreen as usize].3.set_value(v);
    let v = state.detail.base46.cyan.value();
    state.edit.color[Colors::Cyan as usize].0.set_value(v);
    state.edit.color[Colors::Cyan as usize].3.set_value(v);
    let v = state.detail.base46.blue.value();
    state.edit.color[Colors::Blue as usize].0.set_value(v);
    state.edit.color[Colors::Blue as usize].3.set_value(v);
    let v = state.detail.base46.nord_blue.value();
    state.edit.color[Colors::DeepBlue as usize].0.set_value(v);
    state.edit.color[Colors::DeepBlue as usize].3.set_value(v);
    let v = state.detail.base46.dark_purple.value();
    state.edit.color[Colors::Purple as usize].0.set_value(v);
    state.edit.color[Colors::Purple as usize].3.set_value(v);
    let v = state.detail.base46.pink.value();
    state.edit.color[Colors::Magenta as usize].0.set_value(v);
    state.edit.color[Colors::Magenta as usize].3.set_value(v);
    let v = state.detail.base46.baby_pink.value();
    state.edit.color[Colors::RedPink as usize].0.set_value(v);
    state.edit.color[Colors::RedPink as usize].3.set_value(v);
    Ok(Control::Changed)
}

fn load_pal_file(
    path: &Path,
    state: &mut Scenery,
    ctx: &mut Global,
) -> Result<Control<PalEvent>, Error> {
    state.file_path = Some(path.into());

    let mut ff = Ini::new_std();
    match ff.load(path) {
        Ok(_) => {}
        Err(e) => return Err(anyhow!(e)),
    };

    state
        .edit
        .name
        .set_value(ff.get_text("palette", "name", ""));
    state
        .edit
        .docs
        .set_value(ff.get_text("palette", "docs", ""));
    _ = state
        .edit
        .dark
        .set_value(ff.parse_val::<u8, _>("palette", "dark", 63));
    for c in Colors::array() {
        let ccc = ff.parse_array::<2, _, _>("color", c.name(), Color::default());
        state.edit.color[c as usize].0.set_value(ccc[0]);
        state.edit.color[c as usize].3.set_value(ccc[1]);
    }
    for (n, s) in state.edit.color_ext.iter_mut() {
        let c_idx = ff.parse_val("reference", n, ColorIdx::default());
        s.set_value(c_idx);
    }

    ctx.show_theme = create_edit_theme(state);

    Ok(Control::Changed)
}

fn create_edit_theme(state: &Scenery) -> SalsaTheme {
    let palette = state.edit.palette();
    match state.detail.show.themes.value().as_str() {
        "Shell" => shell_theme("Shell", palette),
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
