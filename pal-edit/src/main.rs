mod foreign;
mod palette_edit;
mod proc;
mod sample;
mod sample_or_base46;
mod util;
mod widget;

#[cfg(feature = "term")]
pub(crate) use rat_salsa;
#[cfg(all(feature = "wgpu", not(feature = "term")))]
pub(crate) use rat_salsa_wgpu as rat_salsa;

use crate::palette_edit::PaletteEdit;
use crate::sample_or_base46::ShowOrBase46;
use anyhow::{Error, anyhow};
use configparser::ini::Ini;
use dirs::config_dir;
use log::{debug, error};
use pure_rust_locales::Locale;
use rat_salsa::dialog_stack::DialogStack;
use rat_salsa::dialog_stack::file_dialog::{
    file_dialog_event, file_dialog_event2, file_dialog_render,
};
use rat_salsa::dialog_stack::msgdialog::msg_dialog_event;
use rat_salsa::event::RenderedEvent;
#[cfg(all(feature = "wgpu", not(feature = "term")))]
use rat_salsa::event_type::convert_crossterm::ConvertCrossterm;
#[cfg(feature = "term")]
use rat_salsa::poll::PollCrossterm;
use rat_salsa::poll::PollRendered;
use rat_salsa::{Control, RunConfig, SalsaAppContext, SalsaContext, run_tui};
use rat_theme4::palette::{ColorIdx, Colors};
use rat_theme4::theme::SalsaTheme;
use rat_theme4::themes::create_fallback;
use rat_theme4::{
    RatWidgetColor, StyleName, WidgetStyle, create_palette_theme, create_salsa_theme,
};
use rat_widget::event::{
    FileOutcome, HandleEvent, MenuOutcome, Outcome, Popup, Regular, SliderOutcome, ct_event,
    event_flow,
};
use rat_widget::file_dialog::FileDialogState;
use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_widget::layout::LayoutOuter;
use rat_widget::menu::{Menubar, MenubarState, StaticMenu};
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::slider::{Slider, SliderState};
use rat_widget::statusline_stacked::StatusLineStacked;
use rat_widget::text::HasScreenCursor;
use rat_widget::text::clipboard::set_global_clipboard;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui_core::style::{Color, Style, Stylize};
use ratatui_core::text::{Line, Span};
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::Event;
use std::cell::RefCell;
use std::env::args;
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;
use std::rc::Rc;
use std::{fs, mem};
use try_as_traits::TryAsRef;
use util::clipboard::CliClipboard;
use util::message::{MsgState, msg_event, msg_render};

fn main() -> Result<(), Error> {
    let arg = parse_arg();

    setup_logging()?;
    set_global_clipboard(CliClipboard::default());

    let config = Config::load(arg.0, arg.1)?;
    let theme = create_salsa_theme("Shell");
    let mut global = Global::new(config, theme);
    let mut state = Scenery::new(&global.cfg);

    run_tui(
        init,
        render,
        event,
        error,
        &mut global,
        &mut state,
        #[cfg(all(feature = "wgpu", not(feature = "term")))]
        RunConfig::new(ConvertCrossterm::new())?
            .font_family("FiraCode Nerd Font Mono")
            .font_size(20.)
            .poll(PollRendered),
        #[cfg(feature = "term")]
        RunConfig::default()?.poll(PollCrossterm).poll(PollRendered),
    )?;

    Ok(())
}

fn parse_arg() -> (Vec<PathBuf>, Option<PathBuf>) {
    let mut open_pal_path = Vec::new();
    let mut extra_alias_path = None;

    let mut args = args();
    _ = args.next();

    enum S {
        Start,
        Alias,
        Fail,
    }
    let mut s = S::Start;
    for arg in args {
        match s {
            S::Start => {
                if arg == "--alias" {
                    s = S::Alias
                } else if arg == "--help" {
                    eprintln!("pal-edit [pal-file] [--alias aliases.ini]");
                    eprintln!();
                    eprintln!("pal-file can be .pal or .json files");
                    eprintln!();
                    eprintln!("aliases are a list of additional color-aliases.");
                    exit(0);
                } else {
                    open_pal_path.push(arg.into());
                }
            }
            S::Alias => {
                if extra_alias_path.is_none() {
                    extra_alias_path = Some(arg.into());
                } else {
                    s = S::Fail;
                }
            }
            S::Fail => {
                eprintln!("pal-edit palette.pal [--alias custom_aliases]");
                exit(1)
            }
        }
    }

    (open_pal_path, extra_alias_path)
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
    pub open_path: Vec<PathBuf>,
    pub extra_alias: Vec<String>,
}

impl Config {
    pub fn extra_aliased_vec(&self) -> Vec<String> {
        self.extra_alias.clone()
    }

    pub fn aliased_vec(&self) -> Vec<String> {
        let mut r = Vec::new();
        r.extend(proc::rat_widget_color_names().iter().map(|v| v.to_string()));
        r.extend(self.extra_alias.iter().cloned());
        r
    }

    pub fn load(
        open_path: Vec<PathBuf>,
        extra_alias_path: Option<PathBuf>,
    ) -> Result<Config, Error> {
        let loc = sys_locale::get_locale().expect("locale");
        let loc = loc.replace("-", "_");
        let loc = Locale::try_from(loc.as_str()).expect("locale");

        let mut extra_alias = Vec::new();

        if let Some(cfg_dir) = config_dir() {
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
                        extra_alias.push(alias.trim().into());
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
        };

        if let Some(extra_alias_path) = extra_alias_path {
            let mut ini = Ini::new();
            match ini.load(extra_alias_path) {
                Ok(_) => {}
                Err(e) => {
                    return Err(anyhow!(e));
                }
            }

            if let Some(map) = ini.get_map_ref().get("aliases") {
                for alias in map.keys() {
                    extra_alias.push(alias.trim().into());
                }
            }
            if let Some(map) = ini.get_map_ref().get("default") {
                for alias in map.keys() {
                    extra_alias.push(alias.trim().into());
                }
            }
        };

        Ok(Config {
            loc,
            open_path,
            extra_alias,
        })
    }
}

/// Application wide messages.
#[derive(Debug)]
pub enum PalEvent {
    NoOp,
    Event(Event),
    Rendered,
    Message(String),
    Save(PathBuf),
    LoadVec(Vec<PathBuf>),
    Load(PathBuf),
    ExportRs(PathBuf),
    PatchPath(PathBuf),
    ExportPatch(PathBuf),
    ImportColors(PathBuf),
    ContainerBase(ColorIdx),
}

impl From<RenderedEvent> for PalEvent {
    fn from(_: RenderedEvent) -> Self {
        Self::Rendered
    }
}

impl TryAsRef<Event> for PalEvent {
    fn try_as_ref(&self) -> Option<&Event> {
        match self {
            PalEvent::Event(e) => Some(e),
            _ => None,
        }
    }
}

impl From<Event> for PalEvent {
    fn from(value: Event) -> Self {
        Self::Event(value)
    }
}

#[derive(Debug)]
pub struct Scenery {
    pub file_load_dlg: Rc<RefCell<FileDialogState>>,
    pub file_save_dlg: Rc<RefCell<FileDialogState>>,
    pub file_dlg_export: Rc<RefCell<FileDialogState>>,
    pub file_dlg_import: Rc<RefCell<FileDialogState>>,

    pub file_slider: SliderState,
    pub files: Vec<PathBuf>,
    pub patch_path: Option<PathBuf>,
    pub file_path: Option<PathBuf>,

    pub edit: PaletteEdit,
    pub detail: ShowOrBase46,
    pub menu: MenubarState,
    pub menu_return_focus: Option<FocusFlag>,
}

impl Scenery {
    pub fn new(cfg: &Config) -> Self {
        Self {
            file_load_dlg: Rc::new(RefCell::new(FileDialogState::default())),
            file_save_dlg: Rc::new(RefCell::new(FileDialogState::default())),
            file_dlg_export: Rc::new(RefCell::new(FileDialogState::default())),
            file_dlg_import: Rc::new(RefCell::new(FileDialogState::default())),
            file_path: Default::default(),
            patch_path: None,
            file_slider: SliderState::<usize>::named("files"),
            files: Default::default(),
            edit: PaletteEdit::new(cfg),
            detail: ShowOrBase46::new(cfg),
            menu: MenubarState::named("menu"),
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
        Constraint::Length(1),
        Constraint::Fill(1), //
        Constraint::Length(1),
    ])
    .split(area);
    let edit_width = PaletteEdit::width(&ctx.cfg);
    let l2 = Layout::horizontal([
        Constraint::Length(edit_width), //
        Constraint::Fill(1),            //
    ])
    .horizontal_margin(1)
    .flex(Flex::Center)
    .split(l1[1]);

    let l_tool = Layout::horizontal([
        Constraint::Length(5),
        Constraint::Length(15), //
        Constraint::Fill(1),
    ])
    .horizontal_margin(1)
    .spacing(1)
    .split(l1[0]);

    // tool
    buf.set_style(l1[0], ctx.theme.style_style(Style::MENU_BASE));
    if state.patch_path.is_some() {
        Span::from("PATCH").render(l_tool[0], buf);
    }
    if state.files.len() > 0 {
        Slider::new()
            .styles(ctx.theme.style(WidgetStyle::SLIDER))
            .direction(Direction::Horizontal)
            .lower_bound(format!(
                "{}/{} [",
                state.file_slider.value(),
                state.files.len().saturating_sub(1),
            ))
            .upper_bound("]")
            .render(l_tool[1], buf, &mut state.file_slider);

        if let Some(file_path) = &state.file_path {
            let current = file_path.file_name().unwrap_or_default();
            Span::from(current.to_string_lossy()).render(l_tool[2], buf);
        }
    }

    // main
    palette_edit::render(l2[0], buf, &mut state.edit, ctx)?;
    sample_or_base46::render(l2[1], buf, &mut state.detail, ctx)?;
    screen_cursor(state, ctx);

    // menu & status
    let status_layout = Layout::horizontal([
        Constraint::Fill(61), //
        Constraint::Fill(39),
    ])
    .split(l1[2]);

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
    static MENU: StaticMenu = StaticMenu {
        menu: &[
            (
                "P_alette",
                &[
                    "_New",
                    "_Load..",
                    "_Save|F12",
                    "_Save as..",
                    "_Export .rs..|Ctrl+E",
                ],
            ),
            (
                "_Patch", //
                &["_Auto-Load from..", "_Export .rs..|Ctrl+P"],
            ),
            (
                "_Extern", //
                &["_Import Colors..", "Use Base46 colors"],
            ),
            (
                "_List", //
                &["_Next|F8", "Prev|F7"],
            ),
            (
                "_Quit", //
                &[],
            ),
        ],
    };

    Menubar::new(&MENU)
        .styles(ctx.theme.style(WidgetStyle::MENU))
        .title(Line::from_iter([
            Span::from(" P ").white().on_red(),
            Span::from(" A ").white().on_green(),
            Span::from(" L ").white().on_blue(),
        ]))
        .render(area, buf, &mut state.menu);
    Ok(())
}

fn render_status(area: Rect, buf: &mut Buffer, ctx: &mut Global) -> Result<(), Error> {
    let palette = &ctx.theme.p;
    let status_color_1 = palette.high_style(Colors::Blue, 0);
    let status_color_2 = palette.high_style(Colors::Blue, 2);
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

    let pal = state.edit.palette();
    ctx.show_theme = create_palette_theme(pal).unwrap_or_else(|p| create_fallback(p));

    let open_path = mem::take(&mut ctx.cfg.open_path);
    if !open_path.is_empty() {
        ctx.queue_event(PalEvent::LoadVec(open_path));
    }

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

        event_flow!(match state.menu.handle(event, Popup) {
            MenuOutcome::MenuActivated(0, 0) => {
                proc::new_pal(state, ctx)?;
                Control::Changed
            }
            MenuOutcome::MenuActivated(0, 1) => load_pal_dlg(state, ctx)?,
            MenuOutcome::MenuActivated(0, 2) => save_pal_dlg(state, ctx)?,
            MenuOutcome::MenuActivated(0, 3) => save_as_pal_dlg(state, ctx)?,
            MenuOutcome::MenuActivated(0, 4) => export_rs_dlg(state, ctx)?,
            MenuOutcome::MenuActivated(1, 0) => load_patch_from_dlg(state, ctx)?,
            MenuOutcome::MenuActivated(1, 1) => export_patch_dlg(state, ctx)?,
            MenuOutcome::MenuActivated(2, 0) => import_colors_dlg(state, ctx)?,
            MenuOutcome::MenuActivated(2, 1) => {
                proc::use_base46(state, ctx)?;
                Control::Changed
            }
            MenuOutcome::MenuActivated(3, 0) => open_next_pal(state, ctx)?,
            MenuOutcome::MenuActivated(3, 1) => open_prev_pal(state, ctx)?,
            MenuOutcome::Activated(4) => Control::Quit,
            v => v.into(),
        });

        event_flow!(match event {
            ct_event!(keycode press F(12)) => save_pal_dlg(state, ctx)?,
            ct_event!(key press CONTROL-'e') => export_rs_dlg(state, ctx)?,
            ct_event!(key press CONTROL-'p') => export_patch_dlg(state, ctx)?,
            ct_event!(keycode press F(7)) => open_prev_pal(state, ctx)?,
            ct_event!(keycode press F(8)) => open_next_pal(state, ctx)?,
            ct_event!(keycode press F(1)) => help(ctx)?,
            _ => Control::Continue,
        });

        event_flow!(match state.file_slider.handle(event, Regular) {
            SliderOutcome::Value => {
                let p = &state.files[state.file_slider.value()];
                Control::Event(PalEvent::Load(p.clone()))
            }
            r => r.into(),
        });

        event_flow!(match palette_edit::event(event, &mut state.edit, ctx)? {
            Outcome::Changed => {
                let pal = state.edit.palette();
                ctx.show_theme = create_palette_theme(pal).unwrap_or_else(|p| create_fallback(p));
                Outcome::Changed
            }
            r => r.into(),
        });
        event_flow!(sample_or_base46::event(event, &mut state.detail, ctx)?);

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
            state.detail.show.readability.bg_color.set_value(*c);
            Ok(Control::Changed)
        }
        PalEvent::Save(p) => {
            debug!("save_pal {:?}", p);
            proc::save_pal(&p, state, ctx)?;
            Ok(Control::Changed)
        }
        PalEvent::LoadVec(p) => {
            state.files = p.clone();
            state.file_slider.set_value(0);
            state
                .file_slider
                .set_range((0, state.files.len().saturating_sub(1)));
            if let Some(p) = p.first() {
                Ok(Control::Event(PalEvent::Load(p.clone())))
            } else {
                Ok(Control::Changed)
            }
        }
        PalEvent::Load(p) => {
            if state.patch_path.is_some() {
                _ = save_pal_patch(state, ctx)?;
            } else {
                if let Some(p) = state.file_path.clone() {
                    proc::save_pal(&p, state, ctx)?;
                }
            }

            proc::load_pal(p, state, ctx)?;
            if state.patch_path.is_some() {
                _ = load_pal_patch(state, ctx)?;
            }
            if let Some(c) = state.edit.aliased.get(Color::CONTAINER_BASE_BG) {
                state.detail.show.readability.bg_color.set_value(c.value());
            }
            Ok(Control::Changed)
        }
        PalEvent::ExportRs(p) => {
            proc::export_pal_to_rs(&p, state, ctx)?;
            Ok(Control::Changed)
        }
        PalEvent::PatchPath(p) => {
            state.patch_path = Some(p.clone());
            load_pal_patch(state, ctx)
        }
        PalEvent::ExportPatch(p) => {
            proc::export_pal_to_patch(&p, state, ctx)?;
            Ok(Control::Changed)
        }
        PalEvent::ImportColors(p) => {
            state.detail.tabs.select(Some(1));
            state.detail.foreign.load_from_file(&p)
        }
        _ => Ok(Control::Continue),
    }
}

fn save_pal_patch(state: &mut Scenery, ctx: &mut Global) -> Result<Control<PalEvent>, Error> {
    if let Some(patch_path) = &state.patch_path {
        if !state.edit.file_name().is_empty() {
            let patch_file = patch_path
                .join(state.edit.file_name())
                .with_extension("ppal");
            proc::save_patch(&patch_file, state, ctx)?;
        }
        Ok(Control::Changed)
    } else {
        Ok(Control::Continue)
    }
}

fn load_pal_patch(state: &mut Scenery, ctx: &mut Global) -> Result<Control<PalEvent>, Error> {
    if let Some(patch_path) = &state.patch_path {
        let patch_file = patch_path
            .join(state.edit.file_name())
            .with_extension("ppal");
        if patch_file.exists() {
            proc::load_patch(&patch_file, state, ctx)?;
            Ok(Control::Changed)
        } else {
            Ok(Control::Continue)
        }
    } else {
        Ok(Control::Continue)
    }
}

fn open_prev_pal(state: &mut Scenery, _ctx: &mut Global) -> Result<Control<PalEvent>, Error> {
    // mark
    let n = state.file_slider.value();
    if n > 0 {
        state.file_slider.set_value(n - 1);
        let path = &state.files[n - 1];
        Ok(Control::Event(PalEvent::Load(path.clone())))
    } else {
        Ok(Control::Unchanged)
    }
}

fn open_next_pal(state: &mut Scenery, _ctx: &mut Global) -> Result<Control<PalEvent>, Error> {
    let n = state.file_slider.value();
    if n + 1 < state.files.len() {
        state.file_slider.set_value(n + 1);
        let path = &state.files[n + 1];
        Ok(Control::Event(PalEvent::Load(path.clone())))
    } else {
        Ok(Control::Unchanged)
    }
}

fn import_colors_dlg(state: &mut Scenery, ctx: &mut Global) -> Result<Control<PalEvent>, Error> {
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
            Ok(p) => PalEvent::ImportColors(p),
            Err(_) => PalEvent::NoOp,
        }),
        s,
    );
    Ok(Control::Changed)
}

fn load_patch_from_dlg(state: &mut Scenery, ctx: &mut Global) -> Result<Control<PalEvent>, Error> {
    let s = state.file_dlg_export.clone();
    s.borrow_mut().directory_dialog(".")?;
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
            Ok(p) => PalEvent::PatchPath(p),
            Err(_) => PalEvent::NoOp,
        }),
        s,
    );
    Ok(Control::Changed)
}

fn export_patch_dlg(state: &mut Scenery, ctx: &mut Global) -> Result<Control<PalEvent>, Error> {
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
            Ok(p) => PalEvent::ExportPatch(p),
            Err(_) => PalEvent::NoOp,
        }),
        s,
    );
    Ok(Control::Changed)
}

fn export_rs_dlg(state: &mut Scenery, ctx: &mut Global) -> Result<Control<PalEvent>, Error> {
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
            Ok(p) => PalEvent::ExportRs(p),
            Err(_) => PalEvent::NoOp,
        }),
        s,
    );
    Ok(Control::Changed)
}

fn save_as_pal_dlg(state: &mut Scenery, ctx: &mut Global) -> Result<Control<PalEvent>, Error> {
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

fn save_pal_dlg(state: &mut Scenery, ctx: &mut Global) -> Result<Control<PalEvent>, Error> {
    if let Some(file_path) = state.file_path.clone() {
        proc::save_pal(&file_path, state, ctx)?;
        Ok(Control::Changed)
    } else {
        save_as_pal_dlg(state, ctx)
    }
}

fn load_pal_dlg(state: &mut Scenery, ctx: &mut Global) -> Result<Control<PalEvent>, Error> {
    let s = state.file_load_dlg.clone();
    s.borrow_mut().open_many_dialog(".")?;
    ctx.dlg.push(
        file_dialog_render(
            LayoutOuter::new()
                .left(Constraint::Percentage(19))
                .right(Constraint::Percentage(19))
                .top(Constraint::Length(4))
                .bottom(Constraint::Length(4)),
            ctx.theme.style(WidgetStyle::FILE_DIALOG),
        ),
        file_dialog_event2(|p| match p {
            FileOutcome::OkList(mut p) => {
                p.sort();
                PalEvent::LoadVec(p)
            }
            _ => PalEvent::NoOp,
        }),
        s,
    );
    Ok(Control::Changed)
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

fn help(ctx: &mut Global) -> Result<Control<PalEvent>, Error> {
    let doc_str = include_str!("../doc.md");
    let state = MsgDialogState::new_active("Docs", doc_str);
    ctx.dlg.push(
        |area, buf, state, ctx| {
            let state = state.downcast_mut::<MsgDialogState>().expect("msg-dialog");
            MsgDialog::new()
                .styles(ctx.theme.style(WidgetStyle::MSG_DIALOG))
                .left(Constraint::Percentage(19))
                .right(Constraint::Percentage(19))
                .top(Constraint::Length(4))
                .bottom(Constraint::Length(4))
                .markdown(true)
                .hide_paragraph_focus(true)
                .render(area, buf, state);
        },
        msg_dialog_event(|| PalEvent::NoOp),
        state,
    );

    Ok(Control::Changed)
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
