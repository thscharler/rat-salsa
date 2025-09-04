#![allow(unused_variables)]
#![allow(unreachable_pub)]

use crate::Relative::{Current, Full, Parent, SubDir};
use anyhow::Error;
use crossbeam::channel::Sender;
use rat_salsa2::poll::{PollCrossterm, PollTasks};
use rat_salsa2::thread_pool::Cancel;
use rat_salsa2::{run_tui, SalsaAppContext, SalsaContext, Control, RunConfig};
use rat_theme2::palettes::IMPERIAL;
use rat_theme2::{dark_themes, DarkTheme};
use rat_widget::event::{
    ct_event, try_flow, Dialog, DoubleClick, DoubleClickOutcome, HandleEvent, MenuOutcome, Popup,
    ReadOnly, Regular, TableOutcome,
};
use rat_widget::focus::{impl_has_focus, match_focus, FocusBuilder, HasFocus, Navigation};
use rat_widget::list::selection::RowSelection;
use rat_widget::menu::{MenuBuilder, MenuStructure, Menubar, MenubarState};
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::popup::Placement;
use rat_widget::scrolled::Scroll;
use rat_widget::splitter::{Split, SplitState, SplitType};
use rat_widget::statusline::{StatusLine, StatusLineState};
use rat_widget::table::textdata::{Cell, Row};
use rat_widget::table::{Table, TableContext, TableData, TableState};
use rat_widget::text::{impl_screen_cursor, HasScreenCursor};
use rat_widget::textarea::{TextArea, TextAreaState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::symbols::border::EMPTY;
use ratatui::text::{Line, Text};
use ratatui::widgets::block::Title;
use ratatui::widgets::{Block, BorderType, Borders, StatefulWidget, Widget};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::from_utf8;
use std::time::{Duration, SystemTime};
use sysinfo::Disks;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let config = FilesConfig::default();
    let theme = DarkTheme::new("Imperial".into(), IMPERIAL);
    let mut global = GlobalState::new(config, theme);
    let mut state = Files::default();

    run_tui(
        init,
        render,
        event,
        error,
        &mut global,
        &mut state,
        RunConfig::default()?
            .poll(PollCrossterm)
            .poll(PollTasks::default()),
    )?;

    Ok(())
}

// -----------------------------------------------------------------------

#[derive(Debug)]
pub struct GlobalState {
    ctx: SalsaAppContext<FilesEvent, Error>,
    pub cfg: FilesConfig,
    pub theme: DarkTheme,
}

impl SalsaContext<FilesEvent, Error> for GlobalState {
    fn set_salsa_ctx(&mut self, app_ctx: SalsaAppContext<FilesEvent, Error>) {
        self.ctx = app_ctx;
    }

    #[inline]
    fn salsa_ctx(&self) -> &SalsaAppContext<FilesEvent, Error> {
        &self.ctx
    }
}

impl GlobalState {
    fn new(cfg: FilesConfig, theme: DarkTheme) -> Self {
        Self {
            ctx: Default::default(),
            cfg,
            theme,
        }
    }
}

// -----------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct FilesConfig {}

#[derive(Debug)]
pub enum FilesEvent {
    Event(crossterm::event::Event),
    Message(String),
    Status(usize, String),
    ReadDir(Relative, PathBuf, Option<OsString>),
    Update(
        Relative,
        PathBuf,
        Option<OsString>,
        Vec<OsString>,
        Vec<OsString>,
        Option<String>,
    ),
    UpdateFile(PathBuf, String),
}

impl From<crossterm::event::Event> for FilesEvent {
    fn from(value: crossterm::event::Event) -> Self {
        Self::Event(value)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Relative {
    Full,
    Parent,
    Current,
    SubDir,
}

// -----------------------------------------------------------------------

#[derive(Debug)]
pub struct Files {
    pub main_dir: PathBuf,
    pub sub_dirs: Vec<OsString>,
    pub files: Vec<(OsString, bool)>,
    pub err: Option<String>,

    pub cancel_show: Option<Cancel>,

    pub w_split: SplitState,
    pub w_dirs: TableState<RowSelection>,
    pub w_files: TableState<RowSelection>,
    pub w_data: TextAreaState,

    pub w_menu: MenubarState,

    pub status: StatusLineState,
    pub error_dlg: MsgDialogState,
}

impl Default for Files {
    fn default() -> Self {
        let mut zelf = Self {
            main_dir: Default::default(),
            sub_dirs: Default::default(),
            files: Default::default(),
            err: Default::default(),
            cancel_show: Default::default(),
            w_split: Default::default(),
            w_dirs: Default::default(),
            w_files: Default::default(),
            w_data: Default::default(),
            w_menu: Default::default(),
            status: Default::default(),
            error_dlg: Default::default(),
        };
        zelf.w_data.set_focus_navigation(Navigation::Regular);
        zelf
    }
}

#[allow(dead_code)]
struct FileData<'a> {
    dir: Option<PathBuf>,
    files: &'a [(OsString, bool)],
    err: &'a Option<String>,
    dir_style: Style,
    err_style: Style,
}

impl<'a> TableData<'a> for FileData<'a> {
    fn rows(&self) -> usize {
        self.files.len()
    }

    fn header(&self) -> Option<Row<'a>> {
        if let Some(err) = self.err {
            Some(Row::new([Cell::from(err.as_str())]).style(Some(self.err_style)))
        } else {
            None
        }
    }

    fn widths(&self) -> Vec<Constraint> {
        vec![Constraint::Length(25)]
    }

    fn render_cell(
        &self,
        ctx: &TableContext,
        column: usize,
        row: usize,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let (name, isdir) = &self.files[row];
        match column {
            0 => {
                let name = name.to_string_lossy();
                if name.as_ref() == ".." {
                    let mut l = Line::from(name.as_ref());
                    if let Some(dir) = &self.dir {
                        if let Some(parent) = dir.parent() {
                            if let Some(parent) = parent.parent() {
                                l.push_span(" > ");
                                if let Some(name) = parent.file_name() {
                                    l.push_span(name.to_string_lossy().into_owned());
                                }
                            }
                        }
                    }
                    l.render(area, buf);
                } else {
                    let mut l = Line::from(name.as_ref());
                    if *isdir {
                        l.push_span(" >");
                    }
                    l.render(area, buf);
                }
            }
            _ => {}
        }
    }
}

struct DirData<'a> {
    ctx: &'a GlobalState,
    dir: Option<PathBuf>,
    dirs: &'a [OsString],
}

impl<'a> TableData<'a> for DirData<'a> {
    fn rows(&self) -> usize {
        self.dirs.len()
    }

    fn widths(&self) -> Vec<Constraint> {
        vec![Constraint::Length(25)]
    }

    fn render_cell(
        &self,
        t_ctx: &TableContext,
        column: usize,
        row: usize,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let item = &self.dirs[row];
        match column {
            0 => {
                let name = item.to_string_lossy();
                if name.as_ref() == "." {
                    let mut l = Line::default();
                    if !t_ctx.focus || !t_ctx.selected_row {
                        l = l.style(self.ctx.theme.limegreen(0));
                    }
                    if let Some(dir) = &self.dir {
                        if let Some(name) = dir.file_name() {
                            l.push_span(name.to_string_lossy().into_owned());
                        }
                    }
                    l.render(area, buf);
                } else if name.as_ref() == ".." {
                    let mut l = Line::default();
                    if !t_ctx.focus || !t_ctx.selected_row {
                        l = l.style(self.ctx.theme.green(0));
                    }
                    if let Some(dir) = &self.dir {
                        if let Some(parent) = dir.parent() {
                            l.push_span("< ");
                            if let Some(name) = parent.file_name() {
                                l.push_span(name.to_string_lossy().into_owned());
                            }
                        }
                    }
                    l.render(area, buf);
                } else {
                    name.render(area, buf);
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug)]
struct Menu;

impl<'a> MenuStructure<'a> for Menu {
    fn menus(&'a self, menu: &mut MenuBuilder<'a>) {
        menu.item_str("Roots") //
            .item_str("Theme")
            .item_str("Quit");
    }

    fn submenu(&'a self, n: usize, submenu: &mut MenuBuilder<'a>) {
        match n {
            0 => {
                for (s, _) in fs_roots() {
                    submenu.item_string(s);
                }
            }
            1 => {
                for t in dark_themes() {
                    submenu.item_string(t.name().into());
                }
            }
            _ => {}
        }
    }
}

fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut Files,
    ctx: &mut GlobalState,
) -> Result<(), Error> {
    let t0 = SystemTime::now();

    let &[path_area, split_area, menu_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .split(area)
    .as_ref() else {
        unreachable!()
    };

    let &[menu_area, status_area] = Layout::new(
        Direction::Horizontal,
        [Constraint::Fill(1), Constraint::Length(48)],
    )
    .split(menu_area)
    .as_ref() else {
        unreachable!()
    };

    Text::from(state.main_dir.to_string_lossy())
        .alignment(Alignment::Right)
        .style(ctx.theme.black(3).fg(ctx.theme.palette().secondary[2]))
        .render(path_area, buf);

    let (split, split_layout) = Split::horizontal()
        .constraints([
            Constraint::Length(25),
            Constraint::Length(25),
            Constraint::Fill(1),
        ])
        .split_type(SplitType::Scroll)
        .styles(ctx.theme.split_style())
        .into_widget_layout(split_area, &mut state.w_split);

    // split content
    {
        Table::new()
            .data(DirData {
                ctx,
                dir: Some(state.main_dir.clone()),
                dirs: &state.sub_dirs,
            })
            .block(
                Block::new()
                    .borders(Borders::RIGHT)
                    .border_type(BorderType::Rounded),
            )
            .vscroll(Scroll::new().start_margin(2).scroll_by(1))
            .styles(ctx.theme.table_style())
            .render(split_layout[0], buf, &mut state.w_dirs);

        Table::new()
            .data(FileData {
                dir: current_dir(state),
                files: &state.files,
                err: &state.err,
                dir_style: ctx.theme.gray(0),
                err_style: ctx.theme.red(1),
            })
            .block(
                Block::new()
                    .borders(Borders::RIGHT)
                    .border_type(BorderType::Rounded),
            )
            .vscroll(Scroll::new().start_margin(2).scroll_by(1))
            .styles(ctx.theme.table_style())
            .render(split_layout[1], buf, &mut state.w_files);

        let title = if state.w_data.is_focused() {
            Title::from(Line::from("Content").style(ctx.theme.focus()))
        } else {
            Title::from("Content")
        };
        TextArea::new()
            .vscroll(Scroll::new())
            .block(
                Block::bordered()
                    .borders(Borders::TOP)
                    .border_set(EMPTY)
                    .title(title),
            )
            .styles(ctx.theme.textarea_style())
            .render(split_layout[2], buf, &mut state.w_data);
    }

    // render split overlay parts
    split.render(split_area, buf, &mut state.w_split);

    let (menu, menu_popup) = Menubar::new(&Menu)
        .title("[-.-]")
        .popup_block(Block::bordered())
        .popup_placement(Placement::Above)
        .styles(ctx.theme.menu_style())
        .into_widgets();
    menu.render(menu_area, buf, &mut state.w_menu);
    menu_popup.render(menu_area, buf, &mut state.w_menu);

    // show visible cursor
    ctx.set_screen_cursor(state.screen_cursor());

    // render error dialog
    if state.error_dlg.active() {
        let err = MsgDialog::new().styles(ctx.theme.msg_dialog_style());
        err.render(split_area, buf, &mut state.error_dlg);
    }

    let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
    state.status.status(1, format!("R {:.3?}", el).to_string());

    let status = StatusLine::new()
        .layout([
            Constraint::Fill(1),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(12),
        ])
        .styles(ctx.theme.statusline_style());
    status.render(status_area, buf, &mut state.status);

    Ok(())
}

fn init(state: &mut Files, ctx: &mut GlobalState) -> Result<(), Error> {
    state.main_dir = if let Ok(dot) = PathBuf::from(".").canonicalize() {
        dot
    } else {
        PathBuf::from(".")
    };
    ctx.queue(Control::Event(FilesEvent::ReadDir(
        Full,
        state.main_dir.clone(),
        None,
    )));

    state.w_dirs.set_scroll_selection(true);
    state.w_dirs.focus().set(true);
    state.w_files.set_scroll_selection(true);

    Ok(())
}

fn event(
    event: &FilesEvent,
    state: &mut Files,
    ctx: &mut GlobalState,
) -> Result<Control<FilesEvent>, Error> {
    let t0 = SystemTime::now();

    let r = match event {
        FilesEvent::Event(event) => crossterm(event, state, ctx)?,
        FilesEvent::Message(s) => {
            state.error_dlg.append(&*s);
            Control::Changed
        }
        FilesEvent::Status(n, s) => {
            state.status.status(*n, s);
            Control::Changed
        }
        FilesEvent::ReadDir(rel, path, sub) => {
            read_dir(state, *rel, path, sub, ctx)? //
        }
        FilesEvent::Update(rel, path, subdir, ddd, fff, err) => {
            update_dirs(state, *rel, path, subdir, ddd, fff, err, ctx)?
        }
        FilesEvent::UpdateFile(path, text) => {
            //
            update_preview(state, path, text, ctx)?
        }
    };

    let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
    state.status.status(2, format!("H {:.3?}", el).to_string());

    Ok(r)
}

fn crossterm(
    event: &crossterm::event::Event,
    state: &mut Files,
    ctx: &mut GlobalState,
) -> Result<Control<FilesEvent>, Error> {
    try_flow!(match &event {
        ct_event!(resized) => Control::Changed,
        ct_event!(key press CONTROL-'q') => Control::Quit,
        _ => Control::Continue,
    });

    try_flow!({
        if state.error_dlg.active() {
            state.error_dlg.handle(&event, Dialog).into()
        } else {
            Control::Continue
        }
    });

    ctx.set_focus(FocusBuilder::rebuild_for(state, ctx.take_focus()));
    ctx.focus_event(event);

    try_flow!(match event {
        ct_event!(keycode press F(5)) => {
            if state.w_split.is_focused() {
                ctx.focus().next();
            } else {
                ctx.focus().focus(&state.w_split);
            }
            Control::Changed
        }
        _ => Control::Continue,
    });

    try_flow!(match state.w_menu.handle(event, Popup) {
        MenuOutcome::MenuSelected(0, n) => {
            Control::Changed
        }
        MenuOutcome::MenuActivated(0, n) => {
            if let Some(root) = fs_roots().get(n) {
                state.main_dir = root.1.clone();
                ctx.queue(Control::Event(FilesEvent::ReadDir(
                    Full,
                    state.main_dir.clone(),
                    None,
                )));
            }
            Control::Changed
        }
        MenuOutcome::MenuSelected(1, n) => {
            ctx.theme = dark_themes()[n].clone();
            Control::Changed
        }
        MenuOutcome::MenuActivated(1, n) => {
            ctx.theme = dark_themes()[n].clone();
            Control::Changed
        }
        MenuOutcome::Activated(2) => {
            Control::Quit
        }
        r => r.into(),
    });

    try_flow!(state.w_split.handle(event, Regular));

    try_flow!(match_focus!(
        state.w_files => {
            match event {
                ct_event!(keycode press Enter) => {
                    follow_file(state, ctx)?
                }
                _=> Control::Continue
            }
        },
        state.w_dirs => {
            match event {
                ct_event!(keycode press Enter) => {
                    follow_dir(state, ctx)?
                }
                _=> Control::Continue
            }
        },
        _ => {
            Control::Continue
        }
    ));
    try_flow!(match state.w_files.handle(event, DoubleClick) {
        DoubleClickOutcome::ClickClick(_, _) => {
            follow_file(state, ctx)?
        }
        r => r.into(),
    });
    try_flow!(match state.w_files.handle(event, Regular) {
        TableOutcome::Selected => {
            show_file(state, ctx)?
        }
        r => r.into(),
    });
    try_flow!(match state.w_dirs.handle(event, DoubleClick) {
        DoubleClickOutcome::ClickClick(_, _) => {
            follow_dir(state, ctx)?
        }
        r => r.into(),
    });
    try_flow!(match state.w_dirs.handle(event, Regular) {
        TableOutcome::Selected => {
            show_dir(state)?
        }
        v => Control::from(v),
    });
    try_flow!(state.w_data.handle(event, ReadOnly));

    Ok(Control::Continue)
}

fn error(
    event: Error,
    state: &mut Files,
    ctx: &mut GlobalState,
) -> Result<Control<FilesEvent>, Error> {
    state.error_dlg.append(format!("{:?}", &*event).as_str());
    Ok(Control::Changed)
}

impl_screen_cursor!(w_data for Files);
impl_has_focus!(w_split, w_dirs, w_files, w_data, w_menu for Files);

fn show_dir(state: &mut Files) -> Result<Control<FilesEvent>, Error> {
    if let Some(n) = state.w_dirs.selected() {
        if let Some(sub) = state.sub_dirs.get(n) {
            if sub == &OsString::from(".") {
                Ok(Control::Event(FilesEvent::ReadDir(
                    Current,
                    state.main_dir.clone(),
                    None,
                )))
            } else if sub == &OsString::from("..") {
                Ok(Control::Event(FilesEvent::ReadDir(
                    Parent,
                    state.main_dir.clone(),
                    None,
                )))
            } else {
                Ok(Control::Event(FilesEvent::ReadDir(
                    SubDir,
                    state.main_dir.clone(),
                    Some(sub.clone()),
                )))
            }
        } else {
            state.files.clear();
            Ok(Control::Changed)
        }
    } else {
        Ok(Control::Changed)
    }
}

fn update_preview(
    state: &mut Files,
    path: &PathBuf,
    text: &String,
    ctx: &mut GlobalState,
) -> Result<Control<FilesEvent>, Error> {
    let sel = current_file(state);
    if Some(path) == sel.as_ref() {
        state.w_data.set_text(text);
        Ok(Control::Changed)
    } else {
        Ok(Control::Continue)
    }
}

fn update_dirs(
    state: &mut Files,
    rel: Relative,
    path: &PathBuf,
    sub: &Option<OsString>,
    ddd: &Vec<OsString>,
    fff: &Vec<OsString>,
    err: &Option<String>,
    ctx: &mut GlobalState,
) -> Result<Control<FilesEvent>, Error> {
    let selected = if let Some(n) = state.w_dirs.selected() {
        state.sub_dirs.get(n).cloned()
    } else {
        None
    };

    state.err = err.clone();

    match rel {
        Full => {
            state.main_dir = path.clone();
            state.sub_dirs.clear();
            state.sub_dirs.extend(ddd.iter().cloned());
            state.files.clear();
            state.files.extend(fff.iter().cloned().map(|v| (v, false)));

            state.w_dirs.select(Some(0));
            state.w_files.select(Some(0));
        }
        Parent => {
            if selected == Some(OsString::from("..")) {
                state.files.clear();
                state.files.extend(ddd.iter().cloned().map(|v| (v, true)));
                state.files.extend(fff.iter().cloned().map(|v| (v, false)));
                state.w_files.select(Some(0));
            }
        }
        Current => {
            if selected == Some(OsString::from(".")) {
                state.files.clear();
                state.files.extend(fff.iter().cloned().map(|v| (v, false)));
                state.w_files.select(Some(0));
            }
        }
        SubDir => {
            if selected == sub.as_ref().cloned() {
                state.files.clear();
                state.files.extend(ddd.iter().cloned().map(|v| (v, true)));
                state.files.extend(fff.iter().cloned().map(|v| (v, false)));
                state.w_files.select(Some(0));
            }
        }
    }

    _ = show_file(state, ctx)?;

    Ok(Control::Changed)
}

fn read_dir(
    state: &mut Files,
    rel: Relative,
    path: &PathBuf,
    sub: &Option<OsString>,
    ctx: &mut GlobalState,
) -> Result<Control<FilesEvent>, Error> {
    let path = path.clone();
    let sub = sub.clone();

    _ = ctx.spawn(move |can, snd| {
        let Some(read_path) = (match rel {
            Full => Some(path.clone()),
            Parent => path.parent().map(|v| v.to_path_buf()),
            Current => Some(path.clone()),
            SubDir => {
                if let Some(sub) = sub.as_ref() {
                    Some(path.join(sub))
                } else {
                    None
                }
            }
        }) else {
            return Ok(Control::Continue);
        };

        match fs::read_dir(read_path) {
            Ok(r) => {
                let mut ddd = Vec::new();
                if rel == Full {
                    ddd.push(".".into());
                    ddd.push("..".into());
                }
                let mut fff = Vec::new();
                for f in r {
                    if can.is_canceled() {
                        return Ok(Control::Continue);
                    }

                    if let Ok(f) = f {
                        if f.metadata()?.is_dir() {
                            ddd.push(f.file_name());
                        } else {
                            fff.push(f.file_name());
                        }
                    }
                }
                Ok(Control::Event(FilesEvent::Update(
                    rel,
                    path.clone(),
                    sub.clone(),
                    ddd,
                    fff,
                    None,
                )))
            }
            Err(e) => {
                let msg = format!("{:?}", e);
                Ok(Control::Event(FilesEvent::Update(
                    rel,
                    path.clone(),
                    sub.clone(),
                    Vec::default(),
                    Vec::default(),
                    Some(msg),
                )))
            }
        }
    });

    Ok(Control::Continue)
}

fn follow_dir(state: &mut Files, ctx: &mut GlobalState) -> Result<Control<FilesEvent>, Error> {
    if let Some(n) = state.w_dirs.selected() {
        if let Some(sub) = state.sub_dirs.get(n) {
            if sub == &OsString::from("..") {
                if let Some(file) = state.main_dir.parent() {
                    ctx.queue(Control::Event(FilesEvent::ReadDir(
                        Full,
                        file.to_path_buf(),
                        Some(OsString::from("..")),
                    )));
                    ctx.queue(Control::Event(FilesEvent::ReadDir(
                        Parent,
                        file.to_path_buf(),
                        None,
                    )));
                }
            } else if sub == &OsString::from(".") {
                // noop
            } else {
                let file = state.main_dir.join(sub);
                ctx.queue(Control::Event(FilesEvent::ReadDir(Full, file, None)))
            }
        }
    }
    Ok(Control::Continue)
}

fn current_dir(state: &mut Files) -> Option<PathBuf> {
    let dir = if let Some(n) = state.w_dirs.selected() {
        if let Some(sub) = state.sub_dirs.get(n) {
            if sub == &OsString::from("..") {
                state.main_dir.parent().map(|v| v.to_path_buf())
            } else if sub == &OsString::from(".") {
                Some(state.main_dir.clone())
            } else {
                Some(state.main_dir.join(sub))
            }
        } else {
            None
        }
    } else {
        None
    };

    dir
}

fn current_file(state: &mut Files) -> Option<PathBuf> {
    let dir = current_dir(state);

    let file = if let Some(n) = state.w_files.selected() {
        state
            .files
            .get(n)
            .map(|(v, _)| dir.map(|d| d.join(v)))
            .flatten()
    } else {
        None
    };

    file
}

fn show_file(state: &mut Files, ctx: &mut GlobalState) -> Result<Control<FilesEvent>, Error> {
    let file = current_file(state);

    if let Some(file) = file {
        if file.is_file() {
            if let Some(cancel_show) = &state.cancel_show {
                cancel_show.cancel();
            }

            let cancel_show = ctx.spawn(move |can, snd| match fs::read(&file) {
                Ok(data) => {
                    let str_data = display_text(can, snd, &file, &data)?;
                    Ok(Control::Event(FilesEvent::UpdateFile(file, str_data)))
                }
                Err(e) => Ok(Control::Event(FilesEvent::UpdateFile(
                    file,
                    format!("{:?}", e).to_string(),
                ))),
            })?;

            state.cancel_show = Some(cancel_show);

            Ok(Control::Changed)
        } else {
            state.w_data.set_text("");
            Ok(Control::Changed)
        }
    } else {
        state.w_data.set_text("");
        Ok(Control::Changed)
    }
}

fn display_text(
    can: Cancel,
    snd: &Sender<Result<Control<FilesEvent>, Error>>,
    file: &Path,
    text: &Vec<u8>,
) -> Result<String, Error> {
    let t0 = Some(SystemTime::now());

    let hex = 'f: {
        for c in text.iter().take(1024) {
            if *c < 0x20 && *c != b'\n' && *c != b'\r' && *c != b'\t' {
                break 'f true;
            }
        }
        false
    };

    let str_text = if hex {
        use std::fmt::Write;

        let mut v = String::new();

        let mut mm = String::new();
        let mut b0 = Vec::new();
        let mut b1 = String::new();
        let hex = [
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
        ];
        let mut mega = 0;

        _ = write!(mm, "{:8x} ", 0);
        for (n, b) in text.iter().enumerate() {
            b0.push(hex[(b / 16) as usize] as u8);
            b0.push(hex[(b % 16) as usize] as u8);
            if n % 16 == 7 {
                b0.push(b' ');
            }

            if *b < 0x20 {
                b1.push('_');
            } else if *b < 0x7F {
                b1.push(*b as char);
            } else if *b < 0xA0 {
                b1.push('_')
            } else {
                b1.push(*b as char);
            }

            if n > 0 && (n + 1) % 16 == 0 {
                v.push_str(mm.as_str());
                v.push_str(from_utf8(&b0).expect("str"));
                v.push(' ');
                v.push_str(b1.as_str());
                v.push('\n');

                b0.clear();
                b1.clear();

                mm.clear();
                _ = write!(mm, "{:08x} ", n + 1);
            }

            if v.len() / 1_000_000 > mega {
                if can.is_canceled() {
                    return Ok("!Canceled!".to_string());
                }

                mega = v.len() / 1_000_000;

                if mega == 1 {
                    _ = snd.send(Ok(Control::Event(FilesEvent::UpdateFile(
                        file.to_path_buf(),
                        v.clone(),
                    ))));
                }
            }
        }
        v.push_str(mm.as_str());
        _ = write!(v, "{:33}", from_utf8(&b0).expect("str"));
        v.push(' ');
        v.push_str(b1.as_str());
        v.push('\n');
        v
    } else {
        String::from_utf8_lossy(&text).to_string()
    };

    Ok(str_text)
}

fn follow_file(state: &mut Files, ctx: &mut GlobalState) -> Result<Control<FilesEvent>, Error> {
    let file = current_file(state);

    if let Some(file) = file {
        if file.metadata()?.is_dir() {
            ctx.queue(Control::Event(FilesEvent::ReadDir(
                Full,
                file.clone(),
                None,
            )));
            ctx.queue(Control::Event(FilesEvent::ReadDir(Parent, file, None)));
        }
    };
    Ok(Control::Changed)
}

fn setup_logging() -> Result<(), Error> {
    if let Some(cache) = dirs::cache_dir() {
        let log_path = cache.join("rat-salsa");
        if !log_path.exists() {
            fs::create_dir_all(&log_path)?;
        }

        let log_file = log_path.join("life.log");
        _ = fs::remove_file(&log_file);
        fern::Dispatch::new()
            .format(|out, message, _record| {
                out.finish(format_args!("{}", message)) //
            })
            .level(log::LevelFilter::Debug)
            .chain(fern::log_file(&log_file)?)
            .apply()?;
    }
    Ok(())
}

fn fs_roots() -> Vec<(String, PathBuf)> {
    let mut roots = Vec::new();

    if let Some(p) = dirs::home_dir() {
        roots.push(("Home".into(), p));
    }
    if let Some(p) = dirs::document_dir() {
        roots.push(("Documents".into(), p));
    }
    if let Some(p) = dirs::download_dir() {
        roots.push(("Downloads".into(), p));
    }
    if let Some(p) = dirs::desktop_dir() {
        roots.push(("Desktop".into(), p));
    }
    if let Some(p) = dirs::audio_dir() {
        roots.push(("Audio".into(), p));
    }
    if let Some(p) = dirs::picture_dir() {
        roots.push(("Pictures".into(), p));
    }
    if let Some(p) = dirs::video_dir() {
        roots.push(("Videos".into(), p));
    }

    let disks = Disks::new_with_refreshed_list();
    for d in disks.list() {
        roots.push((
            d.name().to_string_lossy().to_string(),
            d.mount_point().to_path_buf(),
        ));
    }

    roots
}
