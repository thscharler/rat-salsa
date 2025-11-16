use crate::configparser_ext::ConfigParserExt;
use crate::message::{msg_event, msg_render, MsgState};
use crate::palette_edit::PaletteEdit;
use crate::show_or_base46::ShowOrBase46;
use anyhow::{anyhow, Error};
use configparser::ini::Ini;
use log::{error, warn};
use pure_rust_locales::Locale;
use rat_event::{event_flow, Outcome};
use rat_focus::{FocusFlag, HasFocus, Navigation};
use rat_salsa::dialog_stack::file_dialog::{file_dialog_event, file_dialog_render};
use rat_salsa::dialog_stack::DialogStack;
use rat_salsa::event::RenderedEvent;
use rat_salsa::poll::{PollCrossterm, PollRendered};
use rat_salsa::{run_tui, Control, RunConfig, SalsaAppContext, SalsaContext};
use rat_theme5::{
    create_theme, dark_theme, shell_theme, ColorIdx, Colors, ColorsExt, Palette, Theme, WidgetStyle,
};
use rat_widget::event::{ct_event, HandleEvent, MenuOutcome, Regular};
use rat_widget::file_dialog::FileDialogState;
use rat_widget::focus::FocusBuilder;
use rat_widget::layout::LayoutOuter;
use rat_widget::menu::{MenuLine, MenuLineState};
use rat_widget::statusline_stacked::StatusLineStacked;
use rat_widget::text::clipboard::{set_global_clipboard, Clipboard, ClipboardError};
use rat_widget::text::HasScreenCursor;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{StatefulWidget, Widget};
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;
use std::{array, fs};
use try_as_traits::TryAsRef;

fn main() -> Result<(), Error> {
    setup_logging()?;
    set_global_clipboard(CliClipboard::default());

    let config = Config::default();
    let theme = create_theme("Imperial Dark").expect("theme");
    let mut global = Global::new(config, theme);
    let mut state = Scenery::new(global.loc);

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
    pub theme: Theme,
    pub show_theme: Theme,
    pub loc: Locale,

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
    pub fn new(cfg: Config, theme: Theme) -> Self {
        let mut z = Self {
            ctx: Default::default(),
            dlg: Default::default(),
            cfg,
            theme,
            show_theme: Default::default(),
            loc: Default::default(),
            status_frame: 0,
            status: Default::default(),
        };
        let loc = sys_locale::get_locale().expect("locale");
        let loc = loc.replace("-", "_");
        z.loc = Locale::try_from(loc.as_str()).expect("locale");
        z
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

pub fn pal_choice(pal: Palette) -> Vec<(ColorIdx, Line<'static>)> {
    const COLOR_X_8: usize = Colors::LEN * 8;
    let pal_choice = array::from_fn::<_, COLOR_X_8, _>(|n| {
        let c = Colors::array()[n / 8];
        let n = n % 8;
        (c, n)
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
    pub fn new(loc: Locale) -> Self {
        Self {
            file_load_dlg: Rc::new(RefCell::new(FileDialogState::default())),
            file_save_dlg: Rc::new(RefCell::new(FileDialogState::default())),
            file_dlg_export: Rc::new(RefCell::new(FileDialogState::default())),
            file_dlg_import: Rc::new(RefCell::new(FileDialogState::default())),
            file_path: None,
            edit: PaletteEdit::new(loc),
            detail: ShowOrBase46::new(loc),
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
    show_or_base46::render(l2[1], buf, &mut state.detail, ctx)?;
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
        event_flow!(show_or_base46::event(event, &mut state.detail, ctx)?);

        event_flow!(match state.menu.handle(event, Regular) {
            MenuOutcome::Activated(0) => new_pal(state, ctx)?,
            MenuOutcome::Activated(1) => load_pal(state, ctx)?,
            MenuOutcome::Activated(2) => save_pal(state, ctx)?,
            MenuOutcome::Activated(3) => saveas_pal(state, ctx)?,
            MenuOutcome::Activated(4) => export_pal(state, ctx)?,
            MenuOutcome::Activated(5) => import_base46(state, ctx)?,
            MenuOutcome::Activated(6) => Control::Quit,
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
            let c = state.edit.color_ext[ColorsExt::ContainerBase as usize].value();
            state.detail.show.readability.colors.set_value(c);
            Ok(Control::Changed)
        }
        PalEvent::Export(p) => export_pal_file(&p, state, ctx),
        PalEvent::Base46(p) => import_base46_file(&p, state, ctx),
        _ => Ok(Control::Continue),
    }
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

    let c32 = Palette::color_to_u32;

    let name = state.edit.name.text();

    let mut wr = File::create(path)?;
    writeln!(wr, "use crate::{{Colors, ColorsExt, Palette}};")?;
    writeln!(wr, "use ratatui::style::Color;")?;
    writeln!(wr, "")?;
    writeln!(wr, "/// {}", name)?;
    for l in state.edit.docs.text().lines() {
        writeln!(wr, "/// {}", l)?;
    }
    writeln!(
        wr,
        "const DARKNESS: u8 = {};",
        state.edit.dark.value::<u8>().unwrap_or(64)
    )?;
    writeln!(wr, "")?;
    let const_name = name
        .chars()
        .filter_map(|v| {
            if v.is_alphanumeric() {
                Some(v)
            } else {
                Some('_')
            }
        })
        .collect::<String>();
    writeln!(wr, "pub const {}: Palette = {{", const_name.to_uppercase(),)?;
    writeln!(wr, "    let mut p = Palette {{")?;
    writeln!(wr, "        name: \"{}\", ", name)?;
    writeln!(wr, "")?;
    writeln!(wr, "        color: [")?;
    for c in [Colors::TextLight, Colors::TextDark] {
        let c0 = state.edit.color[c as usize].0.value();
        let c3 = state.edit.color[c as usize].3.value();
        writeln!(
            wr,
            "            Palette::interpolate2({:#08x}, {:#08x}, 0x0, 0x0),",
            c32(c0),
            c32(c3)
        )?;
    }
    for c in Colors::array_no_text() {
        let c0 = state.edit.color[c as usize].0.value();
        let c3 = state.edit.color[c as usize].3.value();
        writeln!(
            wr,
            "            Palette::interpolate({:#08x}, {:#08x}, DARKNESS),",
            c32(c0),
            c32(c3)
        )?;
    }
    writeln!(wr, "        ],")?;
    writeln!(wr, "        color_ext: [Color::Reset; ColorsExt::LEN],")?;
    writeln!(wr, "    }};")?;
    writeln!(wr, "")?;
    for c in ColorsExt::array() {
        let ccc = state.edit.color_ext[c as usize].value();
        writeln!(
            wr,
            "    p.color_ext[ColorsExt::{:?} as usize] = p.color[Colors::{:?} as usize][{}];",
            c, ccc.0, ccc.1
        )?;
    }
    writeln!(wr, "")?;
    writeln!(wr, "    p")?;
    writeln!(wr, "}};")?;

    Ok(Control::Changed)
}

fn saveas_pal(state: &mut Scenery, ctx: &mut Global) -> Result<Control<PalEvent>, Error> {
    let s = state.file_save_dlg.clone();
    let name = state
        .edit
        .name
        .text() //
        .chars()
        .filter_map(|v| {
            if v.is_alphanumeric() {
                Some(v)
            } else {
                Some('_')
            }
        })
        .collect::<String>();
    s.borrow_mut()
        .save_dialog_ext(".", name.to_lowercase(), "pal")?;
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
    ff.set_text("palette", "name", state.edit.name.text());
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
    for c in ColorsExt::array() {
        let color_idx = state.edit.color_ext[c as usize].value();
        ff.set_val("reference", c.name(), color_idx);
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
    for c in ColorsExt::array() {
        state.edit.color_ext[c as usize].set_value(ColorIdx(Colors::default(), 0));
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

    if ff.get("palette", "text_black").is_some() {
        state
            .edit
            .name
            .set_value(ff.get_text("palette", "name", ""));
        state.edit.docs.set_value("");
        _ = state.edit.dark.set_value(63);

        state.edit.color[Colors::TextLight as usize]
            .0
            .set_value(ff.parse_val("palette", "text_light", Color::default()));
        state.edit.color[Colors::TextLight as usize]
            .3
            .set_value(ff.parse_val("palette", "text_bright", Color::default()));
        state.edit.color[Colors::TextDark as usize]
            .0
            .set_value(ff.parse_val("palette", "text_black", Color::default()));
        state.edit.color[Colors::TextDark as usize]
            .3
            .set_value(ff.parse_val("palette", "text_dark", Color::default()));

        let a = ff.parse_array_v4::<8, _, _>("palette", "white", Color::default());
        state.edit.color[Colors::White as usize].0.set_value(a[0]);
        state.edit.color[Colors::White as usize].3.set_value(a[3]);
        let a = ff.parse_array_v4::<8, _, _>("palette", "gray", Color::default());
        state.edit.color[Colors::Gray as usize].0.set_value(a[0]);
        state.edit.color[Colors::Gray as usize].3.set_value(a[3]);
        let a = ff.parse_array_v4::<8, _, _>("palette", "black", Color::default());
        state.edit.color[Colors::Black as usize].0.set_value(a[0]);
        state.edit.color[Colors::Black as usize].3.set_value(a[3]);
        let a = ff.parse_array_v4::<8, _, _>("palette", "red", Color::default());
        state.edit.color[Colors::Red as usize].0.set_value(a[0]);
        state.edit.color[Colors::Red as usize].3.set_value(a[3]);
        let a = ff.parse_array_v4::<8, _, _>("palette", "orange", Color::default());
        state.edit.color[Colors::Orange as usize].0.set_value(a[0]);
        state.edit.color[Colors::Orange as usize].3.set_value(a[3]);
        let a = ff.parse_array_v4::<8, _, _>("palette", "yellow", Color::default());
        state.edit.color[Colors::Yellow as usize].0.set_value(a[0]);
        state.edit.color[Colors::Yellow as usize].3.set_value(a[3]);
        let a = ff.parse_array_v4::<8, _, _>("palette", "limegreen", Color::default());
        state.edit.color[Colors::LimeGreen as usize]
            .0
            .set_value(a[0]);
        state.edit.color[Colors::LimeGreen as usize]
            .3
            .set_value(a[3]);
        let a = ff.parse_array_v4::<8, _, _>("palette", "green", Color::default());
        state.edit.color[Colors::Green as usize].0.set_value(a[0]);
        state.edit.color[Colors::Green as usize].3.set_value(a[3]);
        let a = ff.parse_array_v4::<8, _, _>("palette", "bluegreen", Color::default());
        state.edit.color[Colors::BlueGreen as usize]
            .0
            .set_value(a[0]);
        state.edit.color[Colors::BlueGreen as usize]
            .3
            .set_value(a[3]);
        let a = ff.parse_array_v4::<8, _, _>("palette", "cyan", Color::default());
        state.edit.color[Colors::Cyan as usize].0.set_value(a[0]);
        state.edit.color[Colors::Cyan as usize].3.set_value(a[3]);
        let a = ff.parse_array_v4::<8, _, _>("palette", "blue", Color::default());
        state.edit.color[Colors::Blue as usize].0.set_value(a[0]);
        state.edit.color[Colors::Blue as usize].3.set_value(a[3]);
        let a = ff.parse_array_v4::<8, _, _>("palette", "deepblue", Color::default());
        state.edit.color[Colors::DeepBlue as usize]
            .0
            .set_value(a[0]);
        state.edit.color[Colors::DeepBlue as usize]
            .3
            .set_value(a[3]);
        let a = ff.parse_array_v4::<8, _, _>("palette", "purple", Color::default());
        state.edit.color[Colors::Purple as usize].0.set_value(a[0]);
        state.edit.color[Colors::Purple as usize].3.set_value(a[3]);
        let a = ff.parse_array_v4::<8, _, _>("palette", "magenta", Color::default());
        state.edit.color[Colors::Magenta as usize].0.set_value(a[0]);
        state.edit.color[Colors::Magenta as usize].3.set_value(a[3]);
        let a = ff.parse_array_v4::<8, _, _>("palette", "redpink", Color::default());
        state.edit.color[Colors::RedPink as usize].0.set_value(a[0]);
        state.edit.color[Colors::RedPink as usize].3.set_value(a[3]);
        let a = ff.parse_array_v4::<8, _, _>("palette", "primary", Color::default());
        state.edit.color[Colors::Primary as usize].0.set_value(a[0]);
        state.edit.color[Colors::Primary as usize].3.set_value(a[3]);
        let a = ff.parse_array_v4::<8, _, _>("palette", "secondary", Color::default());
        state.edit.color[Colors::Secondary as usize]
            .0
            .set_value(a[0]);
        state.edit.color[Colors::Secondary as usize]
            .3
            .set_value(a[3]);

        let color_idx = ColorIdx::from_str("white:0").expect("color");
        state.edit.color_ext[ColorsExt::LabelFg as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("gray:3").expect("color");
        state.edit.color_ext[ColorsExt::Input as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("primary:1").expect("color");
        state.edit.color_ext[ColorsExt::Focus as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("secondary:1").expect("color");
        state.edit.color_ext[ColorsExt::Select as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("gray:3").expect("color");
        state.edit.color_ext[ColorsExt::Disabled as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("red:1").expect("color");
        state.edit.color_ext[ColorsExt::Invalid as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("text-light:0").expect("color");
        state.edit.color_ext[ColorsExt::TitleFg as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("red:0").expect("color");
        state.edit.color_ext[ColorsExt::Title as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("text-light:0").expect("color");
        state.edit.color_ext[ColorsExt::HeaderFg as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("blue:0").expect("color");
        state.edit.color_ext[ColorsExt::Header as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("text-light:0").expect("color");
        state.edit.color_ext[ColorsExt::FooterFg as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("blue:0").expect("color");
        state.edit.color_ext[ColorsExt::Footer as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("text-dark:0").expect("color");
        state.edit.color_ext[ColorsExt::Shadows as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("primary:1").expect("color");
        state.edit.color_ext[ColorsExt::TextFocus as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("secondary:1").expect("color");
        state.edit.color_ext[ColorsExt::TextSelect as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("gray:0").expect("color");
        state.edit.color_ext[ColorsExt::ButtonBase as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("black:1").expect("color");
        state.edit.color_ext[ColorsExt::MenuBase as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("blue-green:0").expect("color");
        state.edit.color_ext[ColorsExt::KeyBinding as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("black:1").expect("color");
        state.edit.color_ext[ColorsExt::StatusBase as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("black:2").expect("color");
        state.edit.color_ext[ColorsExt::ContainerBase as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("gray:1").expect("color");
        state.edit.color_ext[ColorsExt::ContainerBorderFg as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("gray:1").expect("color");
        state.edit.color_ext[ColorsExt::ContainerArrowFg as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("white:0").expect("color");
        state.edit.color_ext[ColorsExt::PopupBase as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("gray:3").expect("color");
        state.edit.color_ext[ColorsExt::PopupBorderFg as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("gray:3").expect("color");
        state.edit.color_ext[ColorsExt::PopupArrowFg as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("gray:2").expect("color");
        state.edit.color_ext[ColorsExt::DialogBase as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("black:3").expect("color");
        state.edit.color_ext[ColorsExt::DialogBorderFg as usize].set_value(color_idx);
        let color_idx = ColorIdx::from_str("black:3").expect("color");
        state.edit.color_ext[ColorsExt::DialogArrowFg as usize].set_value(color_idx);
    } else {
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
        for c in ColorsExt::array() {
            let color_idx = ff.parse_val("reference", c.name(), ColorIdx::default());
            state.edit.color_ext[c as usize].set_value(color_idx);
        }
    }

    ctx.show_theme = create_edit_theme(state);

    Ok(Control::Changed)
}

fn create_edit_theme(state: &Scenery) -> Theme {
    let palette = state.edit.palette();
    match state.detail.show.themes.value().as_str() {
        "Shell" => shell_theme("Shell", palette),
        // "Fallback" => fallback_theme("Fallback", palette),
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

mod base46 {
    use crate::Global;
    use anyhow::Error;
    use rat_event::{event_flow, HandleEvent, Outcome, Regular};
    use rat_focus::{impl_has_focus, HasFocus};
    use rat_theme5::WidgetStyle;
    use rat_widget::clipper::{Clipper, ClipperState};
    use rat_widget::color_input::{ColorInput, ColorInputState, Mode};
    use rat_widget::event::TextOutcome;
    use rat_widget::layout::LayoutForm;
    use rat_widget::scrolled::Scroll;
    use rat_widget::text::impl_screen_cursor;
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Flex, Rect};

    #[allow(non_snake_case)]
    #[derive(Debug, Default)]
    pub struct Base46 {
        pub form: ClipperState,

        pub white: ColorInputState,
        pub darker_black: ColorInputState,
        pub black: ColorInputState,
        pub black2: ColorInputState,
        pub one_bg: ColorInputState,
        pub one_bg2: ColorInputState,
        pub one_bg3: ColorInputState,
        pub grey: ColorInputState,
        pub grey_fg: ColorInputState,
        pub grey_fg2: ColorInputState,
        pub light_grey: ColorInputState,
        pub red: ColorInputState,
        pub baby_pink: ColorInputState,
        pub pink: ColorInputState,
        pub line: ColorInputState,
        pub green: ColorInputState,
        pub vibrant_green: ColorInputState,
        pub nord_blue: ColorInputState,
        pub blue: ColorInputState,
        pub yellow: ColorInputState,
        pub sun: ColorInputState,
        pub purple: ColorInputState,
        pub dark_purple: ColorInputState,
        pub teal: ColorInputState,
        pub orange: ColorInputState,
        pub cyan: ColorInputState,
        pub statusline_bg: ColorInputState,
        pub lightbg: ColorInputState,
        pub pmenu_bg: ColorInputState,
        pub folder_bg: ColorInputState,
        pub base00: ColorInputState,
        pub base01: ColorInputState,
        pub base02: ColorInputState,
        pub base03: ColorInputState,
        pub base04: ColorInputState,
        pub base05: ColorInputState,
        pub base06: ColorInputState,
        pub base07: ColorInputState,
        pub base08: ColorInputState,
        pub base09: ColorInputState,
        pub base0A: ColorInputState,
        pub base0B: ColorInputState,
        pub base0C: ColorInputState,
        pub base0D: ColorInputState,
        pub base0E: ColorInputState,
        pub base0F: ColorInputState,
    }

    impl_has_focus!(
        white, darker_black, black, black2, one_bg, one_bg2, one_bg3, grey,
        grey_fg, grey_fg2, light_grey, red, baby_pink, pink, line, green,
        vibrant_green, nord_blue, blue, yellow, sun, purple, dark_purple,
        teal, orange, cyan, statusline_bg, lightbg, pmenu_bg, folder_bg,
        base00, base01, base02, base03, base04, base05, base06, base07,
        base08, base09, base0A, base0B, base0C, base0D, base0E, base0F
        for Base46);
    impl_screen_cursor!(
        white, darker_black, black, black2, one_bg, one_bg2, one_bg3, grey,
        grey_fg, grey_fg2, light_grey, red, baby_pink, pink, line, green,
        vibrant_green, nord_blue, blue, yellow, sun, purple, dark_purple,
        teal, orange, cyan, statusline_bg, lightbg, pmenu_bg, folder_bg,
        base00, base01, base02, base03, base04, base05, base06, base07,
        base08, base09, base0A, base0B, base0C, base0D, base0E, base0F
        for Base46
    );

    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        state: &mut Base46,
        ctx: &mut Global,
    ) -> Result<(), Error> {
        let mut form = Clipper::new()
            .buffer_uses_view_size()
            .vscroll(Scroll::new())
            .styles(ctx.theme.style(WidgetStyle::CLIPPER));
        let layout_size = form.layout_size(area, &mut state.form);

        if !state.form.valid_layout(layout_size) {
            use rat_widget::layout::{FormLabel as L, FormWidget as W};
            let mut layout = LayoutForm::<usize>::new().spacing(1).flex(Flex::Start);
            layout.widget(state.white.id(), L::Str("white"), W::Width(17));
            layout.widget(
                state.darker_black.id(),
                L::Str("darker_black"),
                W::Width(17),
            );
            layout.widget(state.black.id(), L::Str("black"), W::Width(17));
            layout.widget(state.black2.id(), L::Str("black2"), W::Width(17));
            layout.widget(state.one_bg.id(), L::Str("one_bg"), W::Width(17));
            layout.widget(state.one_bg2.id(), L::Str("one_bg2"), W::Width(17));
            layout.widget(state.one_bg3.id(), L::Str("one_bg3"), W::Width(17));
            layout.widget(state.grey.id(), L::Str("grey"), W::Width(17));
            layout.widget(state.grey_fg.id(), L::Str("grey_fg"), W::Width(17));
            layout.widget(state.grey_fg2.id(), L::Str("grey_fg2"), W::Width(17));
            layout.widget(state.light_grey.id(), L::Str("light_grey"), W::Width(17));
            layout.widget(state.red.id(), L::Str("red"), W::Width(17));
            layout.widget(state.baby_pink.id(), L::Str("baby_pink"), W::Width(17));
            layout.widget(state.pink.id(), L::Str("pink"), W::Width(17));
            layout.widget(state.line.id(), L::Str("line"), W::Width(17));
            layout.widget(state.green.id(), L::Str("green"), W::Width(17));
            layout.widget(
                state.vibrant_green.id(),
                L::Str("vibrant_green"),
                W::Width(17),
            );
            layout.widget(state.nord_blue.id(), L::Str("nord_blue"), W::Width(17));
            layout.widget(state.blue.id(), L::Str("blue"), W::Width(17));
            layout.widget(state.yellow.id(), L::Str("yellow"), W::Width(17));
            layout.widget(state.sun.id(), L::Str("sun"), W::Width(17));
            layout.widget(state.purple.id(), L::Str("purple"), W::Width(17));
            layout.widget(state.dark_purple.id(), L::Str("dark_purple"), W::Width(17));
            layout.widget(state.teal.id(), L::Str("teal"), W::Width(17));
            layout.widget(state.orange.id(), L::Str("orange"), W::Width(17));
            layout.widget(state.cyan.id(), L::Str("cyan"), W::Width(17));
            layout.widget(
                state.statusline_bg.id(),
                L::Str("statusline_bg"),
                W::Width(17),
            );
            layout.widget(state.lightbg.id(), L::Str("lightbg"), W::Width(17));
            layout.widget(state.pmenu_bg.id(), L::Str("pmenu_bg"), W::Width(17));
            layout.widget(state.folder_bg.id(), L::Str("folder_bg"), W::Width(17));
            layout.widget(state.base00.id(), L::Str("base00"), W::Width(17));
            layout.widget(state.base01.id(), L::Str("base01"), W::Width(17));
            layout.widget(state.base02.id(), L::Str("base02"), W::Width(17));
            layout.widget(state.base03.id(), L::Str("base03"), W::Width(17));
            layout.widget(state.base04.id(), L::Str("base04"), W::Width(17));
            layout.widget(state.base05.id(), L::Str("base05"), W::Width(17));
            layout.widget(state.base06.id(), L::Str("base06"), W::Width(17));
            layout.widget(state.base07.id(), L::Str("base07"), W::Width(17));
            layout.widget(state.base08.id(), L::Str("base08"), W::Width(17));
            layout.widget(state.base09.id(), L::Str("base09"), W::Width(17));
            layout.widget(state.base0A.id(), L::Str("base0A"), W::Width(17));
            layout.widget(state.base0B.id(), L::Str("base0B"), W::Width(17));
            layout.widget(state.base0C.id(), L::Str("base0C"), W::Width(17));
            layout.widget(state.base0D.id(), L::Str("base0D"), W::Width(17));
            layout.widget(state.base0E.id(), L::Str("base0E"), W::Width(17));
            layout.widget(state.base0F.id(), L::Str("base0F"), W::Width(17));
            form = form.layout(layout.build_endless(layout_size.width))
        }

        let mut form = form.into_buffer(area, &mut state.form);
        form.render(
            state.white.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.white,
        );
        form.render(
            state.darker_black.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.darker_black,
        );
        form.render(
            state.black.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.black,
        );
        form.render(
            state.black2.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.black2,
        );
        form.render(
            state.one_bg.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.one_bg,
        );
        form.render(
            state.one_bg2.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.one_bg2,
        );
        form.render(
            state.one_bg3.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.one_bg3,
        );
        form.render(
            state.grey.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.grey,
        );
        form.render(
            state.grey_fg.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.grey_fg,
        );
        form.render(
            state.grey_fg2.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.grey_fg2,
        );
        form.render(
            state.light_grey.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.light_grey,
        );
        form.render(
            state.red.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.red,
        );
        form.render(
            state.baby_pink.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.baby_pink,
        );
        form.render(
            state.pink.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.pink,
        );
        form.render(
            state.line.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.line,
        );
        form.render(
            state.green.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.green,
        );
        form.render(
            state.vibrant_green.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.vibrant_green,
        );
        form.render(
            state.nord_blue.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.nord_blue,
        );
        form.render(
            state.blue.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.blue,
        );
        form.render(
            state.yellow.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.yellow,
        );
        form.render(
            state.sun.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.sun,
        );
        form.render(
            state.purple.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.purple,
        );
        form.render(
            state.dark_purple.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.dark_purple,
        );
        form.render(
            state.teal.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.teal,
        );
        form.render(
            state.orange.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.orange,
        );
        form.render(
            state.cyan.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.cyan,
        );
        form.render(
            state.statusline_bg.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.statusline_bg,
        );
        form.render(
            state.lightbg.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.lightbg,
        );
        form.render(
            state.pmenu_bg.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.pmenu_bg,
        );
        form.render(
            state.folder_bg.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.folder_bg,
        );
        form.render(
            state.base00.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.base00,
        );
        form.render(
            state.base01.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.base01,
        );
        form.render(
            state.base02.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.base02,
        );
        form.render(
            state.base03.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.base03,
        );
        form.render(
            state.base04.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.base04,
        );
        form.render(
            state.base05.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.base05,
        );
        form.render(
            state.base06.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.base06,
        );
        form.render(
            state.base07.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.base07,
        );
        form.render(
            state.base08.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.base08,
        );
        form.render(
            state.base09.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.base09,
        );
        form.render(
            state.base0A.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.base0A,
        );
        form.render(
            state.base0B.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.base0B,
        );
        form.render(
            state.base0C.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.base0C,
        );
        form.render(
            state.base0D.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.base0D,
        );
        form.render(
            state.base0E.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.base0E,
        );
        form.render(
            state.base0F.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            &mut state.base0F,
        );
        form.finish(buf, &mut state.form);
        Ok(())
    }

    #[allow(unused_variables)]
    pub fn event(
        event: &crossterm::event::Event,
        state: &mut Base46,
        _ctx: &mut Global,
    ) -> Result<Outcome, Error> {
        let mut mode_change = None;
        let r = 'f: {
            event_flow!(break 'f handle_color(event, &mut state.white, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.darker_black, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.black, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.black2, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.one_bg, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.one_bg2, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.one_bg3, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.grey, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.grey_fg, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.grey_fg2, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.light_grey, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.red, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.baby_pink, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.pink, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.line, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.green, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.vibrant_green, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.nord_blue, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.blue, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.yellow, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.sun, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.purple, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.dark_purple, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.teal, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.orange, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.cyan, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.statusline_bg, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.lightbg, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.pmenu_bg, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.folder_bg, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.base00, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.base01, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.base02, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.base03, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.base04, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.base05, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.base06, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.base07, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.base08, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.base09, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.base0A, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.base0B, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.base0C, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.base0D, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.base0E, &mut mode_change)?);
            event_flow!(break 'f handle_color(event, &mut state.base0F, &mut mode_change)?);

            event_flow!(break 'f state.form.handle(event, Regular));
            Outcome::Continue
        };

        if let Some(mode_change) = mode_change {
            state.white.set_mode(mode_change);
            state.darker_black.set_mode(mode_change);
            state.black.set_mode(mode_change);
            state.black2.set_mode(mode_change);
            state.one_bg.set_mode(mode_change);
            state.one_bg2.set_mode(mode_change);
            state.one_bg3.set_mode(mode_change);
            state.grey.set_mode(mode_change);
            state.grey_fg.set_mode(mode_change);
            state.grey_fg2.set_mode(mode_change);
            state.light_grey.set_mode(mode_change);
            state.red.set_mode(mode_change);
            state.baby_pink.set_mode(mode_change);
            state.pink.set_mode(mode_change);
            state.line.set_mode(mode_change);
            state.green.set_mode(mode_change);
            state.vibrant_green.set_mode(mode_change);
            state.nord_blue.set_mode(mode_change);
            state.blue.set_mode(mode_change);
            state.yellow.set_mode(mode_change);
            state.sun.set_mode(mode_change);
            state.purple.set_mode(mode_change);
            state.dark_purple.set_mode(mode_change);
            state.teal.set_mode(mode_change);
            state.orange.set_mode(mode_change);
            state.cyan.set_mode(mode_change);
            state.statusline_bg.set_mode(mode_change);
            state.lightbg.set_mode(mode_change);
            state.pmenu_bg.set_mode(mode_change);
            state.folder_bg.set_mode(mode_change);
            state.base00.set_mode(mode_change);
            state.base01.set_mode(mode_change);
            state.base02.set_mode(mode_change);
            state.base03.set_mode(mode_change);
            state.base04.set_mode(mode_change);
            state.base05.set_mode(mode_change);
            state.base06.set_mode(mode_change);
            state.base07.set_mode(mode_change);
            state.base08.set_mode(mode_change);
            state.base09.set_mode(mode_change);
            state.base0A.set_mode(mode_change);
            state.base0B.set_mode(mode_change);
            state.base0C.set_mode(mode_change);
            state.base0D.set_mode(mode_change);
            state.base0E.set_mode(mode_change);
            state.base0F.set_mode(mode_change);
        }

        Ok(r)
    }

    fn handle_color(
        event: &crossterm::event::Event,
        color: &mut ColorInputState,
        mode_change: &mut Option<Mode>,
    ) -> Result<TextOutcome, Error> {
        let mode = color.mode();
        let r = color.handle(event, Regular);
        if color.mode() != mode {
            *mode_change = Some(color.mode());
        }
        Ok(r)
    }

    /*
    white
    darker_black
    black
    black2
    one_bg
    one_bg2
    one_bg3
    grey
    grey_fg
    grey_fg2
    light_grey
    red
    baby_pink
    pink
    line
    green
    vibrant_green
    nord_blue
    blue
    yellow
    sun
    purple
    dark_purple
    teal
    orange
    cyan
    statusline_bg
    lightbg
    pmenu_bg
    folder_bg
    base00
    base01
    base02
    base03
    base04
    base05
    base06
    base07
    base08
    base09
    base0A
    base0B
    base0C
    base0D
    base0E
    base0F
     */
}

mod palette_edit {
    use crate::color_span::{ColorSpan, ColorSpanState};
    use crate::{Global, PalEvent};
    use anyhow::Error;
    use pure_rust_locales::Locale;
    use rat_event::{event_flow, MouseOnly, Outcome, Popup};
    use rat_focus::{FocusFlag, HasFocus, Navigation};
    use rat_salsa::SalsaContext;
    use rat_theme5::{ColorIdx, Colors, ColorsExt, Palette, WidgetStyle};
    use rat_widget::choice::{Choice, ChoiceState};
    use rat_widget::clipper::{Clipper, ClipperState};
    use rat_widget::color_input::{ColorInput, ColorInputState, Mode};
    use rat_widget::event::{ChoiceOutcome, HandleEvent, Regular, TextOutcome};
    use rat_widget::focus::FocusBuilder;
    use rat_widget::layout::LayoutForm;
    use rat_widget::number_input::{NumberInput, NumberInputState};
    use rat_widget::popup::Placement;
    use rat_widget::scrolled::Scroll;
    use rat_widget::text::HasScreenCursor;
    use rat_widget::text_input::{TextInput, TextInputState};
    use rat_widget::textarea::{TextArea, TextAreaState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Flex, Rect};
    use ratatui::style::Color;
    use ratatui::widgets::{Block, BorderType};
    use std::array;

    #[derive(Debug)]
    pub struct PaletteEdit {
        pub palette: Palette,

        pub form: ClipperState,
        pub name: TextInputState,
        pub docs: TextAreaState,
        pub dark: NumberInputState,

        pub color: [(ColorInputState, (), (), ColorInputState); Colors::LEN],
        pub color_ext: [ChoiceState<ColorIdx>; ColorsExt::LEN],
    }

    impl PaletteEdit {
        pub fn new(loc: Locale) -> Self {
            let mut z = Self {
                palette: Default::default(),
                form: ClipperState::named("form"),
                name: TextInputState::named("name"),
                docs: TextAreaState::named("docs"),
                dark: NumberInputState::named("dark"),
                color: array::from_fn(|i| {
                    (
                        ColorInputState::named(format!("{}-0", Colors::array()[i].name()).as_str()),
                        (),
                        (),
                        ColorInputState::named(format!("{}-3", Colors::array()[i].name()).as_str()),
                    )
                }),
                color_ext: array::from_fn(|i| ChoiceState::named(ColorsExt::array()[i].name())),
            };
            z.dark.set_format_loc("999", loc).expect("format");
            z
        }
    }

    impl PaletteEdit {
        pub fn palette(&self) -> Palette {
            let mut palette = Palette::default();
            let name = Box::from(self.name.text());
            let name = Box::leak(name);
            palette.name = name;

            let dark = self.dark.value().unwrap_or(64);

            palette.color[Colors::TextLight as usize] = Palette::interpolatec2(
                self.color[Colors::TextLight as usize].0.value(),
                self.color[Colors::TextLight as usize].3.value(),
                Color::default(),
                Color::default(),
            );
            palette.color[Colors::TextDark as usize] = Palette::interpolatec2(
                self.color[Colors::TextDark as usize].0.value(),
                self.color[Colors::TextDark as usize].3.value(),
                Color::default(),
                Color::default(),
            );
            for c in Colors::array_no_text() {
                palette.color[c as usize] = Palette::interpolatec(
                    self.color[c as usize].0.value(),
                    self.color[c as usize].3.value(),
                    dark,
                );
            }
            for c in ColorsExt::array() {
                let ColorIdx(cc, n) = self.color_ext[c as usize].value();
                palette.color_ext[c as usize] = palette.color[cc as usize][n];
            }

            palette
        }
    }

    impl HasFocus for PaletteEdit {
        fn build(&self, builder: &mut FocusBuilder) {
            builder.widget(&self.name);
            builder.widget_navigate(&self.docs, Navigation::Regular);
            builder.widget(&self.dark);
            for c in Colors::array() {
                builder.widget(&self.color[c as usize].0);
                builder.widget(&self.color[c as usize].3);
            }
            for c in ColorsExt::array() {
                builder.widget(&self.color_ext[c as usize]);
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
                .or(self.docs.screen_cursor())
                .or_else(|| {
                    for c in Colors::array() {
                        if let Some(s) = self.color[c as usize].0.screen_cursor() {
                            return Some(s);
                        }
                        if let Some(s) = self.color[c as usize].3.screen_cursor() {
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
            .buffer_uses_view_size()
            .block(Block::bordered().border_type(BorderType::Rounded))
            .vscroll(Scroll::new())
            .styles(ctx.theme.style(WidgetStyle::CLIPPER));

        let layout_size = form.layout_size(area, &mut state.form);

        if !state.form.valid_layout(layout_size) {
            use rat_widget::layout::{FormLabel as L, FormWidget as W};
            let mut layout = LayoutForm::<usize>::new().spacing(1).flex(Flex::Start);
            layout.widget(state.name.id(), L::Str("Name"), W::Width(20));
            layout.widget(state.docs.id(), L::Str("Doc"), W::StretchX(20, 3));
            layout.widget(state.dark.id(), L::Str("Dark"), W::Width(4));
            layout.gap(1);
            for c in Colors::array() {
                layout.widget(
                    state.color[c as usize].0.id(),
                    L::String(c.to_string()),
                    W::Width(51),
                );
            }
            layout.gap(1);
            for c in ColorsExt::array() {
                layout.widget(
                    state.color_ext[c as usize].id(),
                    L::String(c.to_string()),
                    W::Width(15),
                );
            }
            form = form.layout(layout.build_endless(layout_size.width));
        }
        let mut form = form.into_buffer(area, &mut state.form);

        form.render(
            state.name.id(),
            || TextInput::new().styles(ctx.theme.style(WidgetStyle::TEXT)),
            &mut state.name,
        );
        form.render(
            state.docs.id(),
            || {
                TextArea::new()
                    .vscroll(Scroll::new())
                    .styles(ctx.theme.style(WidgetStyle::TEXTAREA))
            },
            &mut state.docs,
        );
        form.render(
            state.dark.id(),
            || NumberInput::new().styles(ctx.theme.style(WidgetStyle::TEXT)),
            &mut state.dark,
        );
        form.render(
            state.color[Colors::TextLight as usize].0.id(),
            || {
                ColorSpan::new()
                    .half()
                    .dark(state.dark.value().unwrap_or(63))
                    .color0(ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)))
                    .color3(ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)))
            },
            &mut ColorSpanState {
                color0: &mut state.color[Colors::TextLight as usize].0,
                color3: &mut state.color[Colors::TextLight as usize].3,
            },
        );
        form.render(
            state.color[Colors::TextDark as usize].0.id(),
            || {
                ColorSpan::new()
                    .half()
                    .dark(state.dark.value().unwrap_or(63))
                    .color0(ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)))
                    .color3(ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)))
            },
            &mut ColorSpanState {
                color0: &mut state.color[Colors::TextDark as usize].0,
                color3: &mut state.color[Colors::TextDark as usize].3,
            },
        );

        for c in Colors::array() {
            form.render(
                state.color[c as usize].0.id(),
                || {
                    ColorSpan::new()
                        .dark(state.dark.value().unwrap_or(63))
                        .color0(ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)))
                        .color3(ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)))
                },
                &mut ColorSpanState {
                    color0: &mut state.color[c as usize].0,
                    color3: &mut state.color[c as usize].3,
                },
            );
        }

        let pal = state.palette();
        let pal_choice = crate::pal_choice(pal);

        let mut popup_ext = Vec::new();
        for c in ColorsExt::array() {
            let popup = form.render2(
                state.color_ext[c as usize].id(),
                || {
                    Choice::new()
                        .items(pal_choice.iter().cloned())
                        .select_marker('*')
                        .popup_len(8)
                        .popup_scroll(Scroll::default())
                        .popup_placement(Placement::Right)
                        .styles(ctx.theme.style(WidgetStyle::CHOICE))
                        .into_widgets()
                },
                &mut state.color_ext[c as usize],
            );
            popup_ext.push((c, popup));
        }
        for (c, popup) in popup_ext {
            form.render_popup(
                state.color_ext[c as usize].id(),
                || popup,
                &mut state.color_ext[c as usize],
            );
        }

        form.finish(buf, &mut state.form);

        Ok(())
    }

    #[allow(unused_variables)]
    pub fn event(
        event: &crossterm::event::Event,
        state: &mut PaletteEdit,
        ctx: &mut Global,
    ) -> Result<Outcome, Error> {
        let mut mode_change = None;

        let r = 'f: {
            for c in ColorsExt::array() {
                event_flow!(
                    break 'f match state.color_ext[c as usize].handle(event, Popup) {
                        ChoiceOutcome::Value => {
                            if c == ColorsExt::ContainerBase {
                                ctx.queue_event(PalEvent::ContainerBase(
                                    state.color_ext[c as usize].value(),
                                ));
                            }
                            ChoiceOutcome::Value
                        }
                        r => r,
                    }
                )
            }

            event_flow!(break 'f state.name.handle(event, Regular));
            event_flow!(break 'f state.docs.handle(event, Regular));
            event_flow!(
                break 'f match state.dark.handle(event, Regular) {
                    TextOutcome::TextChanged => {
                        if state.dark.value().unwrap_or(0) > 255 {
                            state.dark.set_invalid(true);
                        } else {
                            state.dark.set_invalid(false);
                        }
                        TextOutcome::TextChanged
                    }
                    r => r,
                }
            );
            for c in Colors::array() {
                event_flow!(
                    break 'f handle_color(event, &mut state.color[c as usize].0, &mut mode_change)
                );
                event_flow!(
                    break 'f handle_color(event, &mut state.color[c as usize].3, &mut mode_change)
                );
            }

            event_flow!(break 'f state.form.handle(event, MouseOnly));

            Outcome::Continue
        };

        if let Some(mode_change) = mode_change {
            for c in Colors::array() {
                state.color[c as usize].0.set_mode(mode_change);
                state.color[c as usize].3.set_mode(mode_change);
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

pub mod show_or_base46 {
    use crate::base46::Base46;
    use crate::show_tabs::ShowTabs;
    use crate::{base46, show_tabs, Global};
    use anyhow::Error;
    use pure_rust_locales::Locale;
    use rat_event::{event_flow, HandleEvent, Outcome, Regular};
    use rat_focus::{Focus, FocusBuilder, FocusFlag, HasFocus};
    use rat_theme5::WidgetStyle;
    use rat_widget::tabbed::{Tabbed, TabbedState};
    use rat_widget::text::HasScreenCursor;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::{Block, BorderType, StatefulWidget};

    // mark tabs
    #[derive(Debug)]
    pub struct ShowOrBase46 {
        pub tabs: TabbedState,
        pub show: ShowTabs,
        pub base46: Base46,
    }

    impl ShowOrBase46 {
        pub fn new(loc: Locale) -> Self {
            Self {
                tabs: Default::default(),
                show: ShowTabs::new(loc),
                base46: Base46::default(),
            }
        }

        pub fn show_focused(&mut self, focus: &Focus) {
            match self.tabs.selected() {
                Some(0) => {
                    self.show.show_focused(focus);
                }
                Some(1) => {
                    self.base46.form.show_focused(focus);
                }
                _ => {}
            }
        }
    }

    impl HasFocus for ShowOrBase46 {
        fn build(&self, builder: &mut FocusBuilder) {
            builder.widget(&self.tabs);
            match self.tabs.selected() {
                Some(0) => {
                    builder.widget(&self.show);
                }
                Some(1) => {
                    builder.widget(&self.base46);
                }
                _ => {}
            }
        }

        fn focus(&self) -> FocusFlag {
            unimplemented!("not available")
        }

        fn area(&self) -> Rect {
            unimplemented!("not available")
        }
    }

    impl HasScreenCursor for ShowOrBase46 {
        fn screen_cursor(&self) -> Option<(u16, u16)> {
            match self.tabs.selected() {
                Some(0) => self.show.screen_cursor(),
                Some(2) => self.base46.screen_cursor(),
                _ => None,
            }
        }
    }

    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        state: &mut ShowOrBase46,
        ctx: &mut Global,
    ) -> Result<(), Error> {
        Tabbed::new()
            .tabs(["Preview", "Base46"])
            .block(Block::bordered().border_type(BorderType::Rounded))
            .styles(ctx.theme.style(WidgetStyle::TABBED))
            .render(area, buf, &mut state.tabs);

        match state.tabs.selected() {
            Some(0) => {
                show_tabs::render(state.tabs.widget_area, buf, &mut state.show, ctx)?;
            }
            Some(1) => {
                let mut area = state.tabs.widget_area;
                area.width += 1;
                base46::render(area, buf, &mut state.base46, ctx)?;
            }
            _ => {}
        };
        Ok(())
    }

    pub fn event(
        event: &crossterm::event::Event,
        state: &mut ShowOrBase46,
        ctx: &mut Global,
    ) -> Result<Outcome, Error> {
        event_flow!(match state.tabs.selected() {
            Some(0) => {
                show_tabs::event(event, &mut state.show, ctx)?
            }
            Some(1) => {
                base46::event(event, &mut state.base46, ctx)?
            }
            _ => {
                Outcome::Continue
            }
        });
        event_flow!(state.tabs.handle(event, Regular));
        Ok(Outcome::Continue)
    }
}

pub mod show_tabs {
    use crate::datainput::DataInput;
    use crate::other::Other;
    use crate::readability::Readability;
    use crate::{datainput, other, readability, Global};
    use anyhow::Error;
    use log::debug;
    use pure_rust_locales::Locale;
    use rat_event::{event_flow, HandleEvent, Outcome, Popup, Regular};
    use rat_focus::{Focus, FocusBuilder, FocusFlag, HasFocus};
    use rat_theme5::{dark_theme, shell_theme, StyleName, WidgetStyle};
    use rat_widget::choice::{Choice, ChoiceState};
    use rat_widget::event::ChoiceOutcome;
    use rat_widget::tabbed::{Tabbed, TabbedState};
    use rat_widget::text::HasScreenCursor;
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Alignment, Constraint, Layout, Position, Rect};
    use ratatui::style::Style;
    use ratatui::text::Text;
    use ratatui::widgets::{Block, BorderType, Borders, StatefulWidget, Widget};
    use std::iter::once;

    // mark tabs
    #[derive(Debug)]
    pub struct ShowTabs {
        pub themes: ChoiceState<String>,
        pub tabs: TabbedState,
        pub input: DataInput,
        pub readability: Readability,
        pub other: Other,
    }

    impl ShowTabs {
        pub fn new(loc: Locale) -> Self {
            Self {
                themes: ChoiceState::named("themes"),
                tabs: Default::default(),
                input: DataInput::new(loc),
                readability: Readability::default(),
                other: Other::default(),
            }
        }

        pub fn show_focused(&mut self, focus: &Focus) {
            match self.tabs.selected() {
                Some(0) => {
                    self.input.form.show_focused(focus);
                }
                Some(1) => { /*noop*/ }
                Some(2) => {
                    self.other.form.show_focused(focus);
                }
                _ => {}
            }
        }
    }

    impl HasFocus for ShowTabs {
        fn build(&self, builder: &mut FocusBuilder) {
            builder.widget(&self.tabs);
            builder.widget(&self.themes);
            match self.tabs.selected() {
                Some(0) => {
                    builder.widget(&self.input);
                }
                Some(1) => {
                    builder.widget(&self.readability);
                }
                Some(2) => {
                    builder.widget(&self.other);
                }
                _ => {}
            }
        }

        fn focus(&self) -> FocusFlag {
            unimplemented!("not available")
        }

        fn area(&self) -> Rect {
            unimplemented!("not available")
        }
    }

    impl HasScreenCursor for ShowTabs {
        fn screen_cursor(&self) -> Option<(u16, u16)> {
            match self.tabs.selected() {
                Some(0) => self.input.screen_cursor(),
                Some(1) => self.readability.screen_cursor(),
                Some(2) => self.other.screen_cursor(),
                _ => None,
            }
        }
    }

    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        state: &mut ShowTabs,
        ctx: &mut Global,
    ) -> Result<(), Error> {
        let l0 = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .spacing(1)
        .split(area);

        let base = ctx.show_theme.style_style(Style::CONTAINER_BASE);
        for r in area.top()..area.bottom() {
            for c in area.left()..area.right() {
                if let Some(cell) = buf.cell_mut(Position::new(c, r)) {
                    cell.reset();
                    cell.set_style(base);
                }
            }
        }

        Text::from("Preview")
            .alignment(Alignment::Center)
            .style(ctx.show_theme.style_style(Style::TITLE))
            .render(l0[1], buf);

        let l_function = Layout::horizontal([
            Constraint::Length(2), //
            Constraint::Length(12),
        ])
        .spacing(1)
        .split(l0[2]);
        let (choice, choice_theme) = Choice::new()
            .items(
                once("")
                    .chain([
                        "Dark",     //
                        "Shell",    //
                        "Fallback", //
                    ])
                    .map(|v| (v.to_string(), v.to_string())),
            )
            .styles(ctx.show_theme.style(WidgetStyle::CHOICE))
            .into_widgets();
        choice.render(l_function[1], buf, &mut state.themes);

        Tabbed::new()
            .tabs(["Input", "Text", "Other"])
            .block(
                Block::bordered()
                    .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                    .border_type(BorderType::Rounded),
            )
            .styles(ctx.show_theme.style(WidgetStyle::TABBED))
            .render(l0[3], buf, &mut state.tabs);

        match state.tabs.selected() {
            Some(0) => {
                let mut area = state.tabs.widget_area;
                area.width += 1;
                datainput::render(area, buf, &mut state.input, ctx)?;
            }
            Some(1) => {
                readability::render(state.tabs.widget_area, buf, &mut state.readability, ctx)?;
            }
            Some(2) => {
                other::render(state.tabs.widget_area, buf, &mut state.other, ctx)?;
            }
            _ => {}
        };

        choice_theme.render(l_function[1], buf, &mut state.themes);

        Ok(())
    }

    pub fn event(
        event: &crossterm::event::Event,
        state: &mut ShowTabs,
        ctx: &mut Global,
    ) -> Result<Outcome, Error> {
        event_flow!(match state.themes.handle(event, Popup) {
            ChoiceOutcome::Value => {
                let pal = ctx.show_theme.p;
                ctx.show_theme = match state.themes.value().as_str() {
                    "Shell" => shell_theme("Shell", pal),
                    // "Fallback" => fallback_theme("Fallback", palette),
                    _ => dark_theme("Dark", pal),
                };
                Outcome::Changed
            }
            r => r.into(),
        });

        event_flow!(match state.tabs.selected() {
            Some(0) => {
                datainput::event(event, &mut state.input, ctx)?
            }
            Some(1) => {
                readability::event(event, &mut state.readability, ctx)?
            }
            Some(2) => {
                other::event(event, &mut state.other, ctx)?
            }
            _ => {
                Outcome::Continue
            }
        });
        event_flow!(state.tabs.handle(event, Regular));
        Ok(Outcome::Continue)
    }
}

pub mod readability {
    use crate::Global;
    use anyhow::Error;
    use rat_event::{event_flow, HandleEvent, Outcome, Popup, Regular};
    use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
    use rat_theme5::{ColorIdx, Colors, WidgetStyle};
    use rat_widget::checkbox::{Checkbox, CheckboxState};
    use rat_widget::choice::{Choice, ChoiceState};
    use rat_widget::paragraph::{Paragraph, ParagraphState};
    use rat_widget::scrolled::Scroll;
    use rat_widget::text::HasScreenCursor;
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Layout, Rect};
    use ratatui::widgets::{StatefulWidget, Wrap};

    #[derive(Debug)]
    pub struct Readability {
        pub colors: ChoiceState<ColorIdx>,
        pub high_contrast: CheckboxState,
        pub para: ParagraphState,
    }

    impl Readability {
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl Default for Readability {
        fn default() -> Self {
            let mut z = Self {
                colors: Default::default(),
                high_contrast: Default::default(),
                para: Default::default(),
            };
            z.colors.set_value(ColorIdx(Colors::Gray, 0));
            z
        }
    }

    impl HasFocus for Readability {
        fn build(&self, builder: &mut FocusBuilder) {
            builder.widget(&self.colors);
            builder.widget(&self.high_contrast);
            builder.widget(&self.para);
        }

        fn focus(&self) -> FocusFlag {
            unimplemented!("not available")
        }

        fn area(&self) -> Rect {
            unimplemented!("not available")
        }
    }

    impl HasScreenCursor for Readability {
        fn screen_cursor(&self) -> Option<(u16, u16)> {
            None
        }
    }

    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        state: &mut Readability,
        ctx: &mut Global,
    ) -> Result<(), Error> {
        let l0 = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .split(area);
        let l1 = Layout::horizontal([
            Constraint::Fill(1), //
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .spacing(1)
        .split(l0[1]);

        let pal_choice = crate::pal_choice(ctx.show_theme.p);
        let (colors, colors_popup) = Choice::new()
            .items(pal_choice)
            .select_marker('*')
            .styles(ctx.show_theme.style(WidgetStyle::CHOICE))
            .into_widgets();
        colors.render(l1[0], buf, &mut state.colors);

        Checkbox::new()
            .styles(ctx.show_theme.style(WidgetStyle::CHECKBOX))
            .text("+Contrast")
            .render(l1[2], buf, &mut state.high_contrast);

        let sel_color = state.colors.value();
        let high_contrast = state.high_contrast.value();
        let text_style = if high_contrast {
            ctx.show_theme.p.high_style(sel_color.0, sel_color.1)
        } else {
            ctx.show_theme.p.style(sel_color.0, sel_color.1)
        };

        Paragraph::new(
            "
The __Paris Peace Accords__, officially the Agreement on Ending the War and Restoring Peace in Viet Nam, was a peace agreement signed on January 27, 1973, to establish peace in Vietnam and end the Vietnam War. The agreement was signed by the governments of the Democratic Republic of Vietnam (North Vietnam), the Republic of Vietnam (South Vietnam), the United States, and the Provisional Revolutionary Government of the Republic of South Vietnam (representing South Vietnamese communists).

The Paris Peace Accords removed the remaining United States forces, and fighting between the three remaining powers  temporarily stopped. The agreement's provisions were immediately and frequently broken by both North and South Vietnamese forces with no official response from the United States. Open fighting broke out in March 1973, and North Vietnamese offensives enlarged their territory by the end of the year. The war continued until the fall of Saigon to North Vietnamese forces in 1975. This photograph shows William P. Rogers, United States Secretary of State, signing the accords in Paris.
",
        )
            .vscroll(Scroll::new())
            .styles(ctx.show_theme.style(WidgetStyle::PARAGRAPH))
            .style(text_style)
            .wrap(Wrap { trim: false })
            .render(l0[3], buf, &mut state.para);

        // don't forget the popup ...
        colors_popup.render(l1[0], buf, &mut state.colors);

        Ok(())
    }

    pub fn event(
        event: &crossterm::event::Event,
        state: &mut Readability,
        _ctx: &mut Global,
    ) -> Result<Outcome, Error> {
        event_flow!(state.colors.handle(event, Popup));
        event_flow!(state.high_contrast.handle(event, Regular));
        event_flow!(state.para.handle(event, Regular));
        Ok(Outcome::Continue)
    }
}

// mark other
pub mod other {
    use crate::Global;
    use anyhow::Error;
    use rat_event::{event_flow, Dialog, HandleEvent, Outcome, Popup, Regular};
    use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
    use rat_theme5::WidgetStyle;
    use rat_widget::dialog_frame::{DialogFrame, DialogFrameState, DialogOutcome};
    use rat_widget::form::{Form, FormState};
    use rat_widget::layout::LayoutForm;
    use rat_widget::list::{List, ListState};
    use rat_widget::menu::{Menubar, MenubarState, StaticMenu};
    use rat_widget::popup::Placement;
    use rat_widget::scrolled::Scroll;
    use rat_widget::splitter::{Split, SplitState, SplitType};
    use rat_widget::statusline::{StatusLine, StatusLineState};
    use rat_widget::table::textdata::{Cell, Row};
    use rat_widget::table::{Table, TableState};
    use rat_widget::text::HasScreenCursor;
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
    use ratatui::prelude::StatefulWidget;
    use ratatui::widgets::Block;

    #[derive(Debug)]
    pub struct Other {
        pub form: FormState<usize>,
        pub dialog_flag: FocusFlag,
        pub dialog: DialogFrameState,
        pub split: SplitState,
        pub list: ListState,
        pub table: TableState,
        pub menu: MenubarState,
        pub status: StatusLineState,
    }

    impl HasFocus for Other {
        fn build(&self, builder: &mut FocusBuilder) {
            builder.widget(&self.menu);
            builder.widget(&self.form);
            builder.widget(&self.list);
            builder.widget(&self.table);
            builder.widget(&self.split);
            builder.widget(&self.dialog_flag);
        }

        fn focus(&self) -> FocusFlag {
            unimplemented!("not available")
        }

        fn area(&self) -> Rect {
            unimplemented!("not available")
        }
    }

    impl HasScreenCursor for Other {
        fn screen_cursor(&self) -> Option<(u16, u16)> {
            None
        }
    }

    impl Default for Other {
        fn default() -> Self {
            let mut z = Self {
                form: FormState::named("form"),
                dialog_flag: FocusFlag::new().with_name("dialog-flag"),
                dialog: DialogFrameState::named("dialog"),
                split: SplitState::named("split"),
                list: ListState::named("list"),
                table: TableState::named("table"),
                menu: MenubarState::named("menubar"),
                status: StatusLineState::named("status"),
            };
            z.status.status(0, "... something ...");
            z.status.status(1, "[join]");
            z.status.status(2, "[conn]");
            z.status.status(3, "[sync]");
            z
        }
    }

    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        state: &mut Other,
        ctx: &mut Global,
    ) -> Result<(), Error> {
        let l0 = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .spacing(1)
        .split(area);

        let mut form = Form::new().styles(ctx.show_theme.style(WidgetStyle::FORM));

        let layout_size = form.layout_size(area);
        if !state.form.valid_layout(layout_size) {
            use rat_widget::layout::{FormLabel as L, FormWidget as W};
            let mut layout = LayoutForm::<usize>::new().flex(Flex::Legacy);
            layout.widget(state.list.id(), L::Str("List"), W::WideStretchXY(20, 4));
            layout.page_break();
            layout.widget(state.table.id(), L::Str("Table"), W::WideStretchXY(20, 4));
            layout.page_break();
            layout.widget(state.split.id(), L::Str("Split"), W::WideStretchXY(20, 4));
            layout.page_break();
            layout.widget(
                state.dialog_flag.id(),
                L::Str("Dialog"),
                W::WideStretchXY(20, 4),
            );
            form = form.layout(layout.build_paged(layout_size));
        }
        let mut form = form.into_buffer(l0[1], buf, &mut state.form);

        form.render(
            state.list.id(),
            || {
                List::new([
                    "Backpacks: A portable bag with straps for carrying personal items, commonly used for school or travel.",
                    "Books: Written or printed works consisting of pages bound together along one side, used for reading and learning.",
                    "Bicycles: Human-powered vehicles with two wheels, used for transportation and recreation.",
                    "Coffee Makers: Appliances designed to brew coffee from ground beans, commonly found in homes and offices.",
                    "Smartphones: Portable devices combining a mobile phone with advanced computing capabilities, including internet access and apps.",
                    "Gardens: Plots of land cultivated for growing plants, flowers, or vegetables, often for aesthetic or practical purposes.",
                    "Music Boxes: Mechanical devices that play music through a rotating cylinder with pins, often used as decorative items.",
                    "Pens: Writing instruments that dispense ink, used for writing or drawing.",
                    "Laptops: Portable computers with integrated screen, keyboard, and battery, designed for mobile computing.",
                    "Dogs: Domesticated mammals commonly kept as pets, known for loyalty and companionship."
                ])
                    .scroll(Scroll::new())
                    .styles(ctx.show_theme.style(WidgetStyle::LIST))
            },
            &mut state.list,
        );
        form.render(
            state.table.id(),
            || {
                Table::new_ratatui(
                    [
                        Row::new([
                            Cell::new("1"),
                            Cell::new("67.9"),
                            Cell::new("Female"),
                            Cell::new("236.4"),
                            Cell::new("129.8"),
                            Cell::new("26.4"),
                            Cell::new("Yes"),
                            Cell::new("High"),
                        ]),
                        Row::new([
                            Cell::new("2"),
                            Cell::new("54.8"),
                            Cell::new("Female"),
                            Cell::new("256.3"),
                            Cell::new("133.4"),
                            Cell::new("28.4"),
                            Cell::new("No"),
                            Cell::new("Medium"),
                        ]),
                        Row::new([
                            Cell::new("3"),
                            Cell::new("68.4"),
                            Cell::new("Male"),
                            Cell::new("198.7"),
                            Cell::new("158.5"),
                            Cell::new("24.1"),
                            Cell::new("Yes"),
                            Cell::new("High"),
                        ]),
                        Row::new([
                            Cell::new("4"),
                            Cell::new("67.9"),
                            Cell::new("Male"),
                            Cell::new("205.0"),
                            Cell::new("136.0"),
                            Cell::new("19.9"),
                            Cell::new("No"),
                            Cell::new("Low"),
                        ]),
                        Row::new([
                            Cell::new("5"),
                            Cell::new("60.9"),
                            Cell::new("Male"),
                            Cell::new("207.7"),
                            Cell::new("145.4"),
                            Cell::new("26.7"),
                            Cell::new("No"),
                            Cell::new("Medium"),
                        ]),
                        Row::new([
                            Cell::new("6"),
                            Cell::new("44.9"),
                            Cell::new("Female"),
                            Cell::new("222.5"),
                            Cell::new("130.6"),
                            Cell::new("30.6"),
                            Cell::new("Noe"),
                            Cell::new("Low"),
                        ]),
                    ],
                    [
                        Constraint::Length(1),
                        Constraint::Length(4),
                        Constraint::Length(6),
                        Constraint::Length(11),
                        Constraint::Length(10),
                        Constraint::Length(5),
                        Constraint::Length(7),
                        Constraint::Length(9),
                    ],
                )
                .scroll(Scroll::new())
                .column_spacing(1)
                .header(Row::new([
                    Cell::new("#"),
                    Cell::new("Age"),
                    Cell::new("Gender"),
                    Cell::new("Cholesterol"),
                    Cell::new("SystolicBP"),
                    Cell::new("BMI"),
                    Cell::new("Smoking"),
                    Cell::new("Education"),
                ]))
                .styles(ctx.show_theme.style(WidgetStyle::TABLE))
                .layout_column_widths()
            },
            &mut state.table,
        );
        let split_overlay = form.render2(
            state.split.id(),
            || {
                Split::new()
                    .direction(Direction::Horizontal)
                    .split_type(SplitType::FullPlain)
                    .constraints([
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                        Constraint::Percentage(60),
                    ])
                    .styles(ctx.show_theme.style(WidgetStyle::SPLIT))
                    .into_widgets()
            },
            &mut state.split,
        );
        form.render_popup(state.split.id(), || split_overlay, &mut state.split);
        form.render(
            state.dialog_flag.id(),
            || {
                DialogFrame::new()
                    .left(Constraint::Length(3))
                    .right(Constraint::Length(3))
                    .top(Constraint::Length(3))
                    .bottom(Constraint::Length(3))
                    .block(Block::bordered().title("Dialog"))
                    .styles(ctx.show_theme.style(WidgetStyle::DIALOG_FRAME))
            },
            &mut state.dialog,
        );

        let (menu, menu_popup) = Menubar::new(&StaticMenu {
            menu: &[
                ("_File", &["_Open", "_Save", "\\___", "_Quit"]),
                ("_Help|F1", &["No Help"]),
            ],
        })
        .styles(ctx.show_theme.style(WidgetStyle::MENU))
        .popup_placement(Placement::Below)
        .into_widgets();
        menu.render(l0[0], buf, &mut state.menu);

        StatusLine::new()
            .layout([
                Constraint::Fill(1),
                Constraint::Length(6),
                Constraint::Length(6),
                Constraint::Length(6),
            ])
            .styles_ext(ctx.show_theme.style(WidgetStyle::STATUSLINE))
            .render(l0[2], buf, &mut state.status);

        menu_popup.render(l0[0], buf, &mut state.menu);

        Ok(())
    }

    pub fn event(
        event: &crossterm::event::Event,
        state: &mut Other,
        _ctx: &mut Global,
    ) -> Result<Outcome, Error> {
        event_flow!(state.menu.handle(event, Popup));

        event_flow!(state.list.handle(event, Regular));
        event_flow!(state.table.handle(event, Regular));
        event_flow!(state.split.handle(event, Regular));
        event_flow!(match state.dialog.handle(event, Dialog) {
            DialogOutcome::Unchanged => {
                // ignore this result!!
                DialogOutcome::Continue
            }
            r => r,
        });

        event_flow!(state.form.handle(event, Regular));

        Ok(Outcome::Continue)
    }
}

pub mod datainput {
    use crate::Global;
    use anyhow::Error;
    use pure_rust_locales::{locale_match, Locale};
    use rat_event::{event_flow, HandleEvent, Outcome, Popup, Regular};
    use rat_focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
    use rat_theme5::{StyleName, WidgetStyle};
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

    // mark
    #[derive(Debug)]
    pub struct DataInput {
        pub form: ClipperState,

        pub disabled: ButtonState,
        pub button: ButtonState,
        pub checkbox: CheckboxState,
        pub choice: ChoiceState,
        pub combobox: ComboboxState,
        pub date_input: DateInputState,
        pub number_input: NumberInputState,
        pub number_invalid: NumberInputState,
        pub radio: RadioState,
        pub slider: SliderState<usize>,
        pub text: TextInputState,
        pub textarea: TextAreaState,
        pub calendar: CalendarState<1, SingleSelection>,
    }

    impl HasFocus for DataInput {
        fn build(&self, builder: &mut FocusBuilder) {
            builder.widget(&self.disabled);
            builder.widget(&self.button);
            builder.widget(&self.checkbox);
            builder.widget(&self.choice);
            builder.widget(&self.combobox);
            builder.widget(&self.date_input);
            builder.widget(&self.number_input);
            builder.widget(&self.number_invalid);
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

    impl HasScreenCursor for DataInput {
        fn screen_cursor(&self) -> Option<(u16, u16)> {
            self.combobox
                .screen_cursor()
                .or(self.date_input.screen_cursor())
                .or(self.number_input.screen_cursor())
                .or(self.number_invalid.screen_cursor())
                .or(self.text.screen_cursor())
                .or(self.textarea.screen_cursor())
                .or(self.calendar.screen_cursor())
        }
    }

    impl DataInput {
        pub fn new(loc: Locale) -> Self {
            let mut z = Self {
                form: ClipperState::named("show"),
                disabled: ButtonState::named("disabled"),
                button: ButtonState::named("button"),
                checkbox: CheckboxState::named("checkbox"),
                choice: ChoiceState::named("choice"),
                combobox: ComboboxState::named("combobox"),
                date_input: DateInputState::named("date_input"),
                number_input: NumberInputState::named("number_input"),
                number_invalid: NumberInputState::named("number_invalid"),
                radio: RadioState::named("radio"),
                slider: SliderState::<usize>::named("slider"),
                text: TextInputState::named("text"),
                textarea: TextAreaState::named("textarea"),
                calendar: CalendarState::named("calendar"),
            };

            let fmt = locale_match!(loc => LC_TIME::D_FMT);
            z.date_input.set_format_loc(fmt, loc).expect("date_format");
            z.number_input
                .set_format_loc("###,##0.00#", loc)
                .expect("number_format");
            z.number_invalid
                .set_format_loc("###,##0.00#", loc)
                .expect("number_format");
            z.number_invalid.set_invalid(true);
            z.calendar.move_to_today();
            z
        }
    }

    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        state: &mut DataInput,
        ctx: &mut Global,
    ) -> Result<(), Error> {
        let mut form = Clipper::new() //
            .vscroll(Scroll::new())
            .buffer_uses_view_size()
            .styles(ctx.show_theme.style(WidgetStyle::CLIPPER));

        let layout_size = form.layout_size(area, &mut state.form);

        if !state.form.valid_layout(layout_size) {
            use rat_widget::layout::{FormLabel as L, FormWidget as W};
            let mut layout = LayoutForm::<usize>::new()
                .spacing(1)
                .line_spacing(1)
                .padding(Padding::new(1, 1, 1, 1))
                .flex(Flex::Start);
            layout.widget(state.disabled.id(), L::Str("Disabled"), W::Width(11));
            layout.widget(state.button.id(), L::Str("Button"), W::Width(11));
            layout.widget(state.checkbox.id(), L::Str("Checkbox"), W::Width(14));
            layout.widget(state.choice.id(), L::Str("Choice"), W::Width(14));
            layout.widget(state.combobox.id(), L::Str("Combobox"), W::Width(14));
            layout.widget(state.date_input.id(), L::Str("DateInput"), W::Width(11));
            layout.widget(state.number_input.id(), L::Str("NumberInput"), W::Width(11));
            layout.widget(state.number_invalid.id(), L::Str("Invalid"), W::Width(11));
            layout.widget(state.radio.id(), L::Str("Radio"), W::Width(25));
            layout.widget(state.slider.id(), L::Str("Slider"), W::Width(15));
            layout.widget(state.text.id(), L::Str("TextInput"), W::Width(20));
            layout.widget(state.textarea.id(), L::Str("TextArea"), W::Size(25, 5));
            layout.widget(state.calendar.id(), L::Str("Calendar"), W::Size(25, 8));
            form = form.layout(layout.build_endless(layout_size.width));
        }
        let mut form = form.into_buffer(area, &mut state.form);

        form.render(
            state.disabled.id(),
            || {
                Button::new("Disabled")
                    .styles(ctx.show_theme.style(WidgetStyle::BUTTON))
                    .style(ctx.show_theme.style_style(Style::DISABLED))
            },
            &mut state.disabled,
        );
        form.render(
            state.button.id(),
            || Button::new("Ok").styles(ctx.show_theme.style(WidgetStyle::BUTTON)),
            &mut state.button,
        );
        form.render(
            state.checkbox.id(),
            || {
                Checkbox::new()
                    .text("rat-salsa")
                    .styles(ctx.show_theme.style(WidgetStyle::CHECKBOX))
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
                    .styles(ctx.show_theme.style(WidgetStyle::CHOICE))
                    .into_widgets()
            },
            &mut state.choice,
        );
        let combo_popup = form.render2(
            state.combobox.id(),
            || {
                Combobox::new()
                    .items([
                        ("Α".to_string(), "Alpha"),
                        ("Β".to_string(), "Beta"),
                        ("Γ".to_string(), "Gamma"),
                        ("Δ".to_string(), "Delta"),
                        ("Ε".to_string(), "Epsilon"),
                        ("Η".to_string(), "Eta"),
                        ("Θ".to_string(), "Theta"),
                        ("Ι".to_string(), "Iota"),
                        ("Κ".to_string(), "Kappa"),
                        ("Λ".to_string(), "Lambda"),
                        ("Μ".to_string(), "My"),
                        ("Ν".to_string(), "Ny"),
                        ("Ξ".to_string(), "Xi"),
                        ("Ο".to_string(), "Omikron"),
                        ("Π".to_string(), "Pi"),
                        ("Χ".to_string(), "Chi"),
                        ("Ψ".to_string(), "Psi"),
                        ("Ω".to_string(), "Omega"),
                    ])
                    .popup_scroll(Scroll::new())
                    .popup_len(7)
                    .styles(ctx.show_theme.style(WidgetStyle::COMBOBOX))
                    .into_widgets()
            },
            &mut state.combobox,
        );
        form.render(
            state.date_input.id(),
            || {
                DateInput::new()
                    .on_focus_lost(TextFocusLost::Position0)
                    .styles(ctx.show_theme.style(WidgetStyle::TEXT))
            },
            &mut state.date_input,
        );
        form.render(
            state.number_input.id(),
            || NumberInput::new().styles(ctx.show_theme.style(WidgetStyle::TEXT)),
            &mut state.number_input,
        );
        form.render(
            state.number_invalid.id(),
            || NumberInput::new().styles(ctx.show_theme.style(WidgetStyle::TEXT)),
            &mut state.number_invalid,
        );
        form.render(
            state.radio.id(),
            || {
                Radio::new()
                    .direction(Direction::Horizontal)
                    .layout(RadioLayout::Stacked)
                    .items([(0, "abc"), (1, "def"), (2, "ghi"), (3, "jkl")])
                    .styles(ctx.show_theme.style(WidgetStyle::RADIO))
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
                        .styles(ctx.show_theme.style(WidgetStyle::SLIDER)),
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
            || TextInput::new().styles(ctx.show_theme.style(WidgetStyle::TEXT)),
            &mut state.text,
        );
        let text_area_focused = state.textarea.is_focused();
        form.render(
            state.textarea.id(),
            || {
                TextArea::new()
                    .vscroll(Scroll::new())
                    .styles(ctx.show_theme.style(WidgetStyle::TEXTAREA))
                    .block(if text_area_focused {
                        Block::new()
                            .style(ctx.show_theme.style_style(Style::INPUT))
                            .border_style(ctx.show_theme.style_style(Style::FOCUS))
                            .borders(Borders::LEFT)
                            .border_set(border::EMPTY)
                    } else {
                        Block::default().style(ctx.show_theme.style_style(Style::INPUT))
                        // .border_style(ctx.show_theme.style_style(Style::INPUT))
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
                    .locale(ctx.loc)
                    .styles(ctx.show_theme.style(WidgetStyle::MONTH))
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
        state: &mut DataInput,
        ctx: &mut Global,
    ) -> Result<Outcome, Error> {
        event_flow!(match state.choice.handle(event, Popup) {
            ChoiceOutcome::Changed => {
                ChoiceOutcome::Changed
            }
            ChoiceOutcome::Value => {
                ChoiceOutcome::Value
            }
            r => r,
        });
        event_flow!(state.combobox.handle(event, Popup));

        event_flow!(match state.button.handle(event, Regular) {
            ButtonOutcome::Pressed => {
                ctx.status = "!!OK!!".to_string();
                Outcome::Changed
            }
            r => r.into(),
        });
        event_flow!(state.checkbox.handle(event, Regular));
        event_flow!(state.date_input.handle(event, Regular));
        event_flow!(state.number_input.handle(event, Regular));
        event_flow!(state.number_invalid.handle(event, Regular));
        event_flow!(state.radio.handle(event, Regular));
        event_flow!(state.slider.handle(event, Regular));
        event_flow!(state.text.handle(event, Regular));
        event_flow!(state.textarea.handle(event, Regular));
        event_flow!(state.calendar.handle(event, Regular));

        event_flow!(state.form.handle(event, Regular));

        Ok(Outcome::Continue)
    }
}

mod color_span {
    use rat_theme5::Palette;
    use rat_widget::color_input::{ColorInput, ColorInputState};
    use rat_widget::reloc::RelocatableState;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::Style;
    use ratatui::widgets::StatefulWidget;

    #[derive(Default, Debug)]
    pub struct ColorSpan<'a> {
        half: bool,
        dark: u8,
        color0: ColorInput<'a>,
        color3: ColorInput<'a>,
    }

    pub struct ColorSpanState<'a> {
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

        pub fn half(mut self) -> Self {
            self.half = true;
            self
        }

        pub fn dark(mut self, dark: u8) -> Self {
            self.dark = dark;
            self
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

            if self.half {
                let width = (area.width.saturating_sub(33)) / 4;
                let colors = Palette::interpolate(
                    state.color0.value_u32(),
                    state.color3.value_u32(),
                    self.dark,
                );
                for i in 0usize..4usize {
                    let color_area =
                        Rect::new(area.x + 34 + (i as u16) * width, area.y, width, area.height);
                    buf.set_style(color_area, Style::new().bg(colors[i]));
                }
            } else {
                let width = (area.width.saturating_sub(33)) / 8;
                let colors = Palette::interpolate(
                    state.color0.value_u32(),
                    state.color3.value_u32(),
                    self.dark,
                );
                for i in 0usize..8usize {
                    let color_area =
                        Rect::new(area.x + 34 + (i as u16) * width, area.y, width, area.height);
                    buf.set_style(color_area, Style::new().bg(colors[i]));
                }
            }
        }
    }
}

mod message {
    use crate::{Global, PalEvent};
    use anyhow::Error;
    use rat_event::{event_flow, Dialog, HandleEvent, Regular};
    use rat_focus::{impl_has_focus, FocusBuilder};
    use rat_salsa::{Control, SalsaContext};
    use rat_theme5::WidgetStyle;
    use rat_widget::dialog_frame::{DialogFrame, DialogFrameState, DialogOutcome};
    use rat_widget::paragraph::{Paragraph, ParagraphState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Rect};
    use ratatui::widgets::StatefulWidget;
    use std::any::Any;

    pub struct MsgState {
        pub dlg: DialogFrameState,
        pub paragraph: ParagraphState,
        pub message: String,
    }

    impl_has_focus!(dlg, paragraph for MsgState);

    pub fn msg_render(area: Rect, buf: &mut Buffer, state: &mut dyn Any, ctx: &mut Global) {
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

    pub fn msg_event(
        event: &PalEvent,
        state: &mut dyn Any,
        ctx: &mut Global,
    ) -> Result<Control<PalEvent>, Error> {
        let state = state.downcast_mut::<MsgState>().expect("msg-dialog");

        if let PalEvent::Event(e) = event {
            let mut focus = FocusBuilder::build_for(state);
            ctx.queue(focus.handle(e, Regular));

            event_flow!(state.paragraph.handle(e, Regular));
            event_flow!(match state.dlg.handle(e, Dialog) {
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
    use std::fmt::Debug;
    use std::mem;
    use std::mem::MaybeUninit;
    use std::path::Path;
    use std::str::FromStr;

    /// Extensions to configparser for ease of use.
    #[allow(dead_code)]
    pub(crate) trait ConfigParserExt {
        fn new_std() -> Self;

        /// Get multiline text.
        fn get_text<S: AsRef<str>, D: Into<String>>(
            &self, //
            sec: S,
            key: &str,
            default: D,
        ) -> String;

        fn parse_array_v4<const N: usize, T: Copy + FromStr, S: AsRef<str>>(
            &mut self,
            sec: S,
            key: &str,
            default: T,
        ) -> [T; N];

        fn parse_array<const N: usize, T: Copy + FromStr + Debug, S: AsRef<str>>(
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

        fn parse_array_v4<const N: usize, T: Copy + FromStr, S: AsRef<str>>(
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

        fn parse_array<const N: usize, T: Copy + FromStr + Debug, S: AsRef<str>>(
            &mut self,
            sec: S,
            key: &str,
            default: T,
        ) -> [T; N] {
            let sec = sec.as_ref();

            let mut r = [MaybeUninit::uninit(); N];

            let val_str = self.get_text(sec, key, "");
            let mut val_str = val_str.split(',');
            for i in 0..N {
                if let Some(v) = val_str.next() {
                    let v = v.trim();
                    match v.parse::<T>() {
                        Ok(v) => {
                            r[i] = MaybeUninit::new(v);
                        }
                        Err(_) => {
                            r[i] = MaybeUninit::new(default);
                        }
                    }
                } else {
                    r[i] = MaybeUninit::new(default);
                }
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
            let mut val_str = String::new();
            for i in 0..N {
                if i > 0 {
                    val_str.push_str(", ");
                }
                val_str.push_str(&val[i].to_string());
            }
            self.set_text(sec, key, val_str);
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

        fn write_std(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
            self.pretty_write(path, &WriteOptions::new_with_params(false, 4, 1))
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
