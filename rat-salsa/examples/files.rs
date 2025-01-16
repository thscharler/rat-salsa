#![allow(unused_variables)]
#![allow(unreachable_pub)]

use crate::FilesEvent::{Message, ReadDir, Update, UpdateFile};
use crate::Relative::{Current, Full, Parent, SubDir};
use anyhow::Error;
use crossbeam::channel::Sender;
use crossterm::event::Event;
use directories_next::UserDirs;
use rat_salsa::poll::{PollCrossterm, PollTasks};
use rat_salsa::thread_pool::Cancel;
use rat_salsa::{run_tui, AppState, AppWidget, Control, RunConfig};
use rat_theme::dark_theme::DarkTheme;
use rat_theme::dark_themes;
use rat_theme::scheme::IMPERIAL;
use rat_widget::event::{
    ct_event, try_flow, Dialog, DoubleClick, DoubleClickOutcome, HandleEvent, MenuOutcome, Popup,
    ReadOnly, Regular, TableOutcome,
};
use rat_widget::focus::{match_focus, FocusBuilder, FocusFlag, HasFocus};
use rat_widget::list::selection::RowSelection;
use rat_widget::menu::{MenuBuilder, MenuStructure, Menubar, MenubarState};
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::popup::Placement;
use rat_widget::scrolled::Scroll;
use rat_widget::splitter::{Split, SplitState, SplitType};
use rat_widget::statusline::{StatusLine, StatusLineState};
use rat_widget::table::textdata::{Cell, Row};
use rat_widget::table::{Table, TableContext, TableData, TableState};
use rat_widget::text::HasScreenCursor;
use rat_widget::textarea::{TextArea, TextAreaState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::symbols::border;
use ratatui::text::{Line, Text};
use ratatui::widgets::block::Title;
use ratatui::widgets::{Block, BorderType, Borders, StatefulWidget, Widget};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::from_utf8;
use std::time::{Duration, SystemTime};
use sysinfo::Disks;

type AppContext<'a> = rat_salsa::AppContext<'a, GlobalState, FilesEvent, Error>;
type RenderContext<'a> = rat_salsa::RenderContext<'a, GlobalState>;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let config = FilesConfig::default();
    let theme = DarkTheme::new("Imperial".into(), IMPERIAL);
    let mut global = GlobalState::new(config, theme);

    let app = FilesApp;
    let mut state = FilesState::default();

    run_tui(
        app,
        &mut global,
        &mut state,
        RunConfig::default()?
            .threads(1)
            .poll(PollCrossterm)
            .poll(PollTasks),
    )?;

    Ok(())
}

// -----------------------------------------------------------------------

#[derive(Debug)]
pub struct GlobalState {
    pub cfg: FilesConfig,
    pub theme: DarkTheme,
    pub status: StatusLineState,
    pub error_dlg: MsgDialogState,
}

impl GlobalState {
    fn new(cfg: FilesConfig, theme: DarkTheme) -> Self {
        Self {
            cfg,
            theme,
            status: Default::default(),
            error_dlg: Default::default(),
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
    fn from(value: Event) -> Self {
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
pub struct FilesApp;

#[derive(Debug, Default)]
pub struct FilesState {
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
        ctx: &TableContext,
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
                    let mut l = Line::from(name.as_ref());
                    if let Some(dir) = &self.dir {
                        l.push_span(" == ");
                        if let Some(name) = dir.file_name() {
                            l.push_span(name.to_string_lossy().into_owned());
                        }
                    }
                    l.render(area, buf);
                } else if name.as_ref() == ".." {
                    let mut l = Line::from(name.as_ref());
                    if let Some(dir) = &self.dir {
                        if let Some(parent) = dir.parent() {
                            l.push_span(" > ");
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

impl AppWidget<GlobalState, FilesEvent, Error> for FilesApp {
    type State = FilesState;

    fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut Self::State,
        ctx: &mut RenderContext<'_>,
    ) -> Result<(), Error> {
        let t0 = SystemTime::now();

        let r = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(1),
            ],
        )
        .split(area);

        let c = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Length(25),
                Constraint::Length(25),
                Constraint::Fill(1),
            ],
        )
        .split(r[1]);

        let m = Layout::new(
            Direction::Horizontal,
            [Constraint::Fill(1), Constraint::Length(48)],
        )
        .split(r[2]);

        // -----------------------------------------------------
        Text::from(state.main_dir.to_string_lossy())
            .alignment(Alignment::Right)
            .style(ctx.g.theme.black(3).fg(ctx.g.theme.scheme().secondary[2]))
            .render(r[0], buf);

        let (split, split_overlay) = Split::new()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(25),
                Constraint::Length(25),
                Constraint::Fill(1),
            ])
            .split_type(SplitType::Scroll)
            .styles(ctx.g.theme.split_style())
            .into_widgets();
        split.render(r[1], buf, &mut state.w_split);

        Table::new()
            .data(DirData {
                dir: Some(state.main_dir.clone()),
                dirs: &state.sub_dirs,
            })
            .styles(ctx.g.theme.table_style())
            .block(
                Block::new()
                    .borders(Borders::RIGHT)
                    .border_type(BorderType::Rounded),
            )
            .vscroll(
                Scroll::new()
                    .styles(ctx.g.theme.scroll_style())
                    .start_margin(2)
                    .scroll_by(1),
            )
            .render(state.w_split.widget_areas[0], buf, &mut state.w_dirs);

        Table::new()
            .data(FileData {
                dir: state.current_dir(),
                files: &state.files,
                err: &state.err,
                dir_style: ctx.g.theme.gray(0),
                err_style: ctx.g.theme.red(1),
            })
            .styles(ctx.g.theme.table_style())
            .block(
                Block::new()
                    .borders(Borders::RIGHT)
                    .border_type(BorderType::Rounded),
            )
            .vscroll(
                Scroll::new()
                    .styles(ctx.g.theme.scroll_style())
                    .start_margin(2)
                    .scroll_by(1),
            )
            .render(state.w_split.widget_areas[1], buf, &mut state.w_files);

        let title = if state.w_data.is_focused() {
            Title::from(Line::from("Content").style(ctx.g.theme.focus()))
        } else {
            Title::from("Content")
        };
        let set = border::Set {
            top_left: " ",
            top_right: " ",
            bottom_left: " ",
            bottom_right: " ",
            vertical_left: " ",
            vertical_right: " ",
            horizontal_top: " ",
            horizontal_bottom: " ",
        };

        let mut content_style = ctx.g.theme.textarea_style();
        content_style.style = ctx.g.theme.black(2);
        TextArea::new()
            .styles(content_style)
            .scroll(Scroll::new().styles(ctx.g.theme.scroll_style()))
            .block(
                Block::bordered()
                    .borders(Borders::TOP)
                    .border_set(set)
                    .style(ctx.g.theme.container_base())
                    .title(title),
            )
            .render(state.w_split.widget_areas[2], buf, &mut state.w_data);
        ctx.cursor = state.w_data.screen_cursor();

        split_overlay.render(r[1], buf, &mut state.w_split);

        let (menu, menu_popup) = Menubar::new(&Menu)
            .title("[-.-]")
            .popup_block(Block::bordered())
            .popup_placement(Placement::Above)
            .styles(ctx.g.theme.menu_style())
            .into_widgets();
        menu.render(m[0], buf, &mut state.w_menu);
        menu_popup.render(m[0], buf, &mut state.w_menu);

        // -----------------------------------------------------

        if ctx.g.error_dlg.active() {
            let err = MsgDialog::new().styles(ctx.g.theme.msg_dialog_style());
            err.render(r[1], buf, &mut ctx.g.error_dlg);
        }

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        ctx.g.status.status(1, format!("R {:.3?}", el).to_string());

        let status = StatusLine::new()
            .layout([
                Constraint::Fill(1),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Length(12),
            ])
            .styles(ctx.g.theme.statusline_style());
        status.render(m[1], buf, &mut ctx.g.status);

        Ok(())
    }
}

impl AppState<GlobalState, FilesEvent, Error> for FilesState {
    fn init(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
        self.main_dir = if let Ok(dot) = PathBuf::from(".").canonicalize() {
            dot
        } else {
            PathBuf::from(".")
        };
        ctx.queue(Control::Message(ReadDir(Full, self.main_dir.clone(), None)));

        self.w_dirs.set_scroll_selection(true);
        self.w_dirs.focus().set(true);
        self.w_files.set_scroll_selection(true);

        Ok(())
    }

    fn event(
        &mut self,
        event: &FilesEvent,
        ctx: &mut rat_salsa::AppContext<'_, GlobalState, FilesEvent, Error>,
    ) -> Result<Control<FilesEvent>, Error> {
        let t0 = SystemTime::now();

        let r = match event {
            FilesEvent::Event(event) => self.crossterm(event, ctx)?,
            Message(s) => {
                ctx.g.status.status(0, &*s);
                Control::Changed
            }
            ReadDir(rel, path, sub) => {
                self.read_dir(*rel, path, sub, ctx)? //
            }
            Update(rel, path, subdir, ddd, fff, err) => {
                self.update_dirs(*rel, path, subdir, ddd, fff, err, ctx)?
            }
            UpdateFile(path, text) => self.update_preview(path, text, ctx)?,
        };

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        ctx.g.status.status(2, format!("H {:.3?}", el).to_string());

        Ok(r)
    }

    fn error(&self, event: Error, ctx: &mut AppContext<'_>) -> Result<Control<FilesEvent>, Error> {
        ctx.g.error_dlg.append(format!("{:?}", &*event).as_str());
        Ok(Control::Changed)
    }
}

impl FilesState {
    fn crossterm(
        &mut self,
        event: &crossterm::event::Event,
        ctx: &mut AppContext,
    ) -> Result<Control<FilesEvent>, Error> {
        try_flow!(match &event {
            ct_event!(resized) => Control::Changed,
            ct_event!(key press CONTROL-'q') => Control::Quit,
            _ => Control::Continue,
        });

        try_flow!({
            if ctx.g.error_dlg.active() {
                ctx.g.error_dlg.handle(&event, Dialog).into()
            } else {
                Control::Continue
            }
        });

        ctx.focus = Some(FocusBuilder::rebuild_for(self, ctx.focus.take()));
        let f = ctx.focus_mut().handle(event, Regular);
        ctx.queue(f);

        try_flow!(match event {
            ct_event!(keycode press F(5)) => {
                if self.w_split.is_focused() {
                    ctx.focus().next();
                } else {
                    ctx.focus().focus(&self.w_split);
                }
                Control::Changed
            }
            _ => Control::Continue,
        });

        try_flow!(match self.w_menu.handle(event, Popup) {
            MenuOutcome::MenuSelected(0, n) => {
                Control::Changed
            }
            MenuOutcome::MenuActivated(0, n) => {
                if let Some(root) = fs_roots().get(n) {
                    self.main_dir = root.1.clone();
                    ctx.queue(Control::Message(ReadDir(Full, self.main_dir.clone(), None)));
                }
                Control::Changed
            }
            MenuOutcome::MenuSelected(1, n) => {
                ctx.g.theme = dark_themes()[n].clone();
                Control::Changed
            }
            MenuOutcome::MenuActivated(1, n) => {
                ctx.g.theme = dark_themes()[n].clone();
                Control::Changed
            }
            r => r.into(),
        });

        try_flow!(self.w_split.handle(event, Regular));

        try_flow!(match self.w_menu.handle(event, Regular) {
            MenuOutcome::Activated(2) => {
                Control::Quit
            }
            r => r.into(),
        });

        try_flow!(match_focus!(
            self.w_files => {
                match event {
                    ct_event!(keycode press Enter) => {
                        self.follow_file(ctx)?
                    }
                    _=> Control::Continue
                }
            },
            self.w_dirs => {
                match event {
                    ct_event!(keycode press Enter) => {
                        self.follow_dir(ctx)?
                    }
                    _=> Control::Continue
                }
            },
            _=> {
                Control::Continue
            }
        ));
        try_flow!(match self.w_files.handle(event, DoubleClick) {
            DoubleClickOutcome::ClickClick(_, _) => {
                self.follow_file(ctx)?
            }
            r => r.into(),
        });
        try_flow!(match self.w_dirs.handle(event, DoubleClick) {
            DoubleClickOutcome::ClickClick(_, _) => {
                self.follow_dir(ctx)?
            }
            r => r.into(),
        });
        try_flow!(match self.w_files.handle(event, Regular) {
            TableOutcome::Selected => {
                self.show_file(ctx)?
            }
            r => r.into(),
        });
        try_flow!(match self.w_dirs.handle(event, Regular) {
            TableOutcome::Selected => {
                self.show_dir()?
            }
            v => Control::from(v),
        });
        try_flow!(self.w_data.handle(event, ReadOnly));

        Ok(Control::Continue)
    }
}

impl FilesState {
    fn show_dir(&mut self) -> Result<Control<FilesEvent>, Error> {
        if let Some(n) = self.w_dirs.selected() {
            if let Some(sub) = self.sub_dirs.get(n) {
                if sub == &OsString::from(".") {
                    Ok(Control::Message(ReadDir(
                        Current,
                        self.main_dir.clone(),
                        None,
                    )))
                } else if sub == &OsString::from("..") {
                    Ok(Control::Message(ReadDir(
                        Parent,
                        self.main_dir.clone(),
                        None,
                    )))
                } else {
                    Ok(Control::Message(ReadDir(
                        SubDir,
                        self.main_dir.clone(),
                        Some(sub.clone()),
                    )))
                }
            } else {
                self.files.clear();
                Ok(Control::Changed)
            }
        } else {
            Ok(Control::Changed)
        }
    }

    fn update_preview(
        &mut self,
        path: &PathBuf,
        text: &String,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<FilesEvent>, Error> {
        let sel = self.current_file();
        if Some(path) == sel.as_ref() {
            self.w_data.set_text(text);
            Ok(Control::Changed)
        } else {
            Ok(Control::Continue)
        }
    }

    fn update_dirs(
        &mut self,
        rel: Relative,
        path: &PathBuf,
        sub: &Option<OsString>,
        ddd: &Vec<OsString>,
        fff: &Vec<OsString>,
        err: &Option<String>,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<FilesEvent>, Error> {
        let selected = if let Some(n) = self.w_dirs.selected() {
            self.sub_dirs.get(n).cloned()
        } else {
            None
        };

        self.err = err.clone();

        match rel {
            Full => {
                self.main_dir = path.clone();
                self.sub_dirs.clear();
                self.sub_dirs.extend(ddd.iter().cloned());
                self.files.clear();
                self.files.extend(fff.iter().cloned().map(|v| (v, false)));

                self.w_dirs.select(Some(0));
                self.w_files.select(Some(0));
            }
            Parent => {
                if selected == Some(OsString::from("..")) {
                    self.files.clear();
                    self.files.extend(ddd.iter().cloned().map(|v| (v, true)));
                    self.files.extend(fff.iter().cloned().map(|v| (v, false)));
                    self.w_files.select(Some(0));
                }
            }
            Current => {
                if selected == Some(OsString::from(".")) {
                    self.files.clear();
                    self.files.extend(fff.iter().cloned().map(|v| (v, false)));
                    self.w_files.select(Some(0));
                }
            }
            SubDir => {
                if selected == sub.as_ref().cloned() {
                    self.files.clear();
                    self.files.extend(ddd.iter().cloned().map(|v| (v, true)));
                    self.files.extend(fff.iter().cloned().map(|v| (v, false)));
                    self.w_files.select(Some(0));
                }
            }
        }

        _ = self.show_file(ctx)?;

        Ok(Control::Changed)
    }

    fn read_dir(
        &mut self,
        rel: Relative,
        path: &PathBuf,
        sub: &Option<OsString>,
        ctx: &mut AppContext<'_>,
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
                    Ok(Control::Message(Update(
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
                    Ok(Control::Message(Update(
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

    fn follow_dir(&mut self, ctx: &mut AppContext<'_>) -> Result<Control<FilesEvent>, Error> {
        if let Some(n) = self.w_dirs.selected() {
            if let Some(sub) = self.sub_dirs.get(n) {
                if sub == &OsString::from("..") {
                    if let Some(file) = self.main_dir.parent() {
                        ctx.queue(Control::Message(ReadDir(
                            Full,
                            file.to_path_buf(),
                            Some(OsString::from("..")),
                        )));
                        ctx.queue(Control::Message(ReadDir(Parent, file.to_path_buf(), None)));
                    }
                } else if sub == &OsString::from(".") {
                    // noop
                } else {
                    let file = self.main_dir.join(sub);
                    ctx.queue(Control::Message(ReadDir(Full, file, None)))
                }
            }
        }
        Ok(Control::Continue)
    }

    fn current_dir(&mut self) -> Option<PathBuf> {
        let dir = if let Some(n) = self.w_dirs.selected() {
            if let Some(sub) = self.sub_dirs.get(n) {
                if sub == &OsString::from("..") {
                    self.main_dir.parent().map(|v| v.to_path_buf())
                } else if sub == &OsString::from(".") {
                    Some(self.main_dir.clone())
                } else {
                    Some(self.main_dir.join(sub))
                }
            } else {
                None
            }
        } else {
            None
        };

        dir
    }

    fn current_file(&mut self) -> Option<PathBuf> {
        let dir = self.current_dir();

        let file = if let Some(n) = self.w_files.selected() {
            self.files
                .get(n)
                .map(|(v, _)| dir.map(|d| d.join(v)))
                .flatten()
        } else {
            None
        };

        file
    }

    fn show_file(&mut self, ctx: &mut AppContext<'_>) -> Result<Control<FilesEvent>, Error> {
        let file = self.current_file();

        if let Some(file) = file {
            if file.is_file() {
                if let Some(cancel_show) = &self.cancel_show {
                    cancel_show.cancel();
                }

                let cancel_show = ctx.spawn(move |can, snd| match fs::read(&file) {
                    Ok(data) => {
                        let str_data = FilesState::display_text(can, snd, &file, &data)?;
                        Ok(Control::Message(UpdateFile(file, str_data)))
                    }
                    Err(e) => Ok(Control::Message(UpdateFile(
                        file,
                        format!("{:?}", e).to_string(),
                    ))),
                })?;

                self.cancel_show = Some(cancel_show);

                Ok(Control::Changed)
            } else {
                self.w_data.set_text("");
                Ok(Control::Changed)
            }
        } else {
            self.w_data.set_text("");
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
                        _ = snd.send(Ok(Control::Message(UpdateFile(
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

    fn follow_file(&mut self, ctx: &mut AppContext<'_>) -> Result<Control<FilesEvent>, Error> {
        let file = self.current_file();

        if let Some(file) = file {
            if file.metadata()?.is_dir() {
                ctx.queue(Control::Message(ReadDir(Full, file.clone(), None)));
                ctx.queue(Control::Message(ReadDir(Parent, file, None)));
            }
        };
        Ok(Control::Changed)
    }
}

impl HasFocus for FilesState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder
            .widget(&self.w_split)
            .widget(&self.w_dirs)
            .widget(&self.w_files)
            .widget(&self.w_data)
            .widget(&self.w_menu);
    }

    fn focus(&self) -> FocusFlag {
        unimplemented!("not in use, silent container")
    }

    fn area(&self) -> Rect {
        unimplemented!("not in use, silent container")
    }
}

fn setup_logging() -> Result<(), Error> {
    _ = fs::remove_file("log.log");
    fern::Dispatch::new()
        .format(|out, message, _record| out.finish(format_args!("{}", message)))
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("log.log")?)
        .apply()?;
    Ok(())
}

fn fs_roots() -> Vec<(String, PathBuf)> {
    let mut roots = Vec::new();
    if let Some(user) = UserDirs::new() {
        roots.push(("Home".into(), user.home_dir().to_path_buf()));
        if let Some(dir) = user.document_dir() {
            roots.push(("Documents".into(), dir.to_path_buf()));
        }
        if let Some(dir) = user.download_dir() {
            roots.push(("Downloads".into(), dir.to_path_buf()));
        }
        if let Some(dir) = user.audio_dir() {
            roots.push(("Audio".into(), dir.to_path_buf()));
        }
        if let Some(dir) = user.desktop_dir() {
            roots.push(("Desktop".into(), dir.to_path_buf()));
        }
        if let Some(dir) = user.picture_dir() {
            roots.push(("Pictures".into(), dir.to_path_buf()));
        }
        if let Some(dir) = user.video_dir() {
            roots.push(("Video".into(), dir.to_path_buf()));
        }
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
