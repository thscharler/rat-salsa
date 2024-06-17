#![allow(unused_variables)]

use crate::FilesAction::{Message, ReadDir, Update, UpdateFile};
use crate::Relative::{Current, Full, Parent, SubDir};
use anyhow::Error;
use crossterm::event::Event;
#[allow(unused_imports)]
use log::debug;
use rat_salsa::timer::TimeOut;
use rat_salsa::{run_tui, AppEvents, AppWidget, Control, RunConfig};
use rat_theme::dark_theme::DarkTheme;
use rat_theme::imperial::IMPERIAL;
use rat_widget::event::{
    ct_event, flow_ok, DoubleClick, DoubleClickOutcome, FocusKeys, HandleEvent, Outcome, ReadOnly,
    ScrollOutcome,
};
use rat_widget::focus::{match_focus, Focus, HasFocus, HasFocusFlag};
use rat_widget::list::selection::RowSelection;
use rat_widget::menuline::{MenuOutcome, RMenuLine, RMenuLineState};
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::scrolled::{Inner, Scrolled, ScrolledState};
use rat_widget::statusline::{StatusLine, StatusLineState};
use rat_widget::table::{FTableContext, RTable, RTableState, TableData};
use rat_widget::textarea::{RTextArea, RTextAreaState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::symbols::border;
use ratatui::text::{Line, Text};
use ratatui::widgets::block::Title;
use ratatui::widgets::{Block, Borders, StatefulWidget, Widget};
use std::cell::RefCell;
use std::ffi::OsString;
use std::path::PathBuf;
use std::str::from_utf8;
use std::time::{Duration, SystemTime};
use std::{fs, mem};

type AppContext<'a> = rat_salsa::AppContext<'a, GlobalState, FilesAction, Error>;
type RenderContext<'a> = rat_salsa::RenderContext<'a, GlobalState>;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let config = FilesConfig::default();
    let theme = DarkTheme::new("ImperialDark".into(), IMPERIAL);
    let mut global = GlobalState::new(config, theme);

    let app = FilesApp;
    let mut state = FilesState::default();

    run_tui(
        app,
        &mut global,
        &mut state,
        RunConfig::default()?.threads(1),
    )?;

    Ok(())
}

// -----------------------------------------------------------------------

#[derive(Debug)]
pub struct GlobalState {
    pub cfg: FilesConfig,
    pub theme: DarkTheme,
    pub status: RefCell<StatusLineState>,
    pub error_dlg: RefCell<MsgDialogState>,
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
pub enum FilesAction {
    Message(String),
    ReadDir(Relative, PathBuf, Option<OsString>),
    Update(
        Relative,
        PathBuf,
        Option<OsString>,
        Vec<OsString>,
        Vec<OsString>,
    ),
    UpdateFile(PathBuf, Vec<u8>),
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

    pub w_dirs: ScrolledState<RTableState<RowSelection>>,
    pub w_files: ScrolledState<RTableState<RowSelection>>,
    pub w_data: ScrolledState<RTextAreaState>,
    pub w_menu: RMenuLineState,
}

#[allow(dead_code)]
struct FileData<'a> {
    dir: Option<PathBuf>,
    files: &'a [(OsString, bool)],
    dir_style: Style,
}

impl<'a> TableData<'a> for FileData<'a> {
    fn rows(&self) -> usize {
        self.files.len()
    }

    fn widths(&self) -> Vec<Constraint> {
        vec![Constraint::Length(25)]
    }

    fn render_cell(
        &self,
        ctx: &FTableContext,
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
        ctx: &FTableContext,
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

impl AppWidget<GlobalState, FilesAction, Error> for FilesApp {
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
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(1),
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
        .split(r[2]);

        // -----------------------------------------------------
        Text::from(state.main_dir.to_string_lossy())
            .style(ctx.g.theme.bluegreen(0))
            .render(r[0], buf);

        Scrolled::new(
            RTable::new()
                .data(DirData {
                    dir: Some(state.main_dir.clone()),
                    dirs: &state.sub_dirs,
                })
                .styles(ctx.g.theme.table_style()),
        )
        .styles(ctx.g.theme.scrolled_style())
        .render(c[0], buf, &mut state.w_dirs);

        Scrolled::new(
            RTable::new()
                .data(FileData {
                    dir: state.current_dir(),
                    files: &state.files,
                    dir_style: ctx.g.theme.gray(0),
                })
                .styles(ctx.g.theme.table_style()),
        )
        .styles(ctx.g.theme.scrolled_style())
        .render(c[1], buf, &mut state.w_files);

        let title = if state.w_data.widget.is_focused() {
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
        Scrolled::new(RTextArea::new().styles(content_style))
            .styles(ctx.g.theme.scrolled_style())
            .block(
                Block::bordered()
                    .borders(Borders::TOP | Borders::BOTTOM | Borders::RIGHT)
                    .border_set(set)
                    .title(title),
            )
            .render(c[2], buf, &mut state.w_data);
        ctx.cursor = state.w_data.widget.screen_cursor();

        let menu = RMenuLine::new()
            .styles(ctx.g.theme.menu_style())
            .title("[-.-]")
            .add("_Quit");
        menu.render(r[3], buf, &mut state.w_menu);

        // -----------------------------------------------------

        if ctx.g.error_dlg.borrow().active {
            let err = MsgDialog::new().styles(ctx.g.theme.msg_dialog_style());
            err.render(r[2], buf, &mut ctx.g.error_dlg.borrow_mut());
        }

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        ctx.g
            .status
            .borrow_mut()
            .status(1, format!("R {:.3?}", el).to_string());

        let status = StatusLine::new()
            .layout([
                Constraint::Fill(1),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Length(12),
            ])
            .styles(ctx.g.theme.statusline_style());
        status.render(r[4], buf, &mut ctx.g.status.borrow_mut());

        Ok(())
    }
}

impl AppEvents<GlobalState, FilesAction, Error> for FilesState {
    fn init(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
        self.main_dir = if let Ok(dot) = PathBuf::from(".").canonicalize() {
            dot
        } else {
            PathBuf::from(".")
        };
        ctx.queue(Control::Action(ReadDir(Full, self.main_dir.clone(), None)));

        self.w_dirs.widget.set_scroll_selection(true);
        self.w_dirs.widget.focus().set();
        self.w_files.widget.set_scroll_selection(true);
        self.w_menu.select(Some(0));

        Ok(())
    }

    fn timer(
        &mut self,
        event: &TimeOut,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<FilesAction>, Error> {
        Ok(Control::Continue)
    }

    fn crossterm(
        &mut self,
        event: &Event,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<FilesAction>, Error> {
        use crossterm::event::*;

        let t0 = SystemTime::now();

        flow_ok!(match &event {
            Event::Resize(_, _) => Control::Repaint,
            ct_event!(key press CONTROL-'q') => Control::Quit,
            _ => Control::Continue,
        });

        flow_ok!({
            if ctx.g.error_dlg.borrow().active {
                ctx.g
                    .error_dlg
                    .borrow_mut()
                    .handle(&event, FocusKeys)
                    .into()
            } else {
                Control::Continue
            }
        });

        ctx.queue(self.focus().handle(event, FocusKeys));

        flow_ok!(match_focus!(
            self.w_files.widget => {
                match event {
                    ct_event!(keycode press Enter) => {
                        self.follow_file(ctx)?
                    }
                    _=> Control::Continue
                }
            },
            self.w_dirs.widget => {
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
        flow_ok!(match self.w_files.widget.handle(event, DoubleClick) {
            DoubleClickOutcome::ClickClick(_, _) => {
                self.follow_file(ctx)?
            }
            r => r.into(),
        });
        flow_ok!(match self.w_dirs.widget.handle(event, DoubleClick) {
            DoubleClickOutcome::ClickClick(_, _) => {
                self.follow_dir(ctx)?
            }
            r => r.into(),
        });
        flow_ok!(match self.w_files.handle(event, FocusKeys) {
            ScrollOutcome::Inner(Outcome::Changed) => {
                self.show_file(ctx)?
            }
            r => r.into(),
        });
        flow_ok!(match self.w_dirs.handle(event, FocusKeys) {
            ScrollOutcome::Inner(Outcome::Changed) => {
                self.show_dir()?
            }
            v => Control::from(v),
        });

        flow_ok!(self.w_data.handle(event, Inner(ReadOnly)));

        flow_ok!(match self.w_menu.handle(event, FocusKeys) {
            MenuOutcome::Activated(0) => {
                Control::Quit
            }
            v => v.into(),
        });

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        ctx.g
            .status
            .borrow_mut()
            .status(2, format!("H {:.3?}", el).to_string());

        Ok(Control::Continue)
    }

    fn action(
        &mut self,
        event: &mut FilesAction,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<FilesAction>, Error> {
        let t0 = SystemTime::now();

        // TODO: actions
        flow_ok!(match event {
            Message(s) => {
                ctx.g.status.borrow_mut().status(0, &*s);
                Control::Repaint
            }
            ReadDir(rel, path, sub) => {
                self.read_dir(*rel, path, sub, ctx)?
            }
            Update(rel, path, subdir, ddd, fff) =>
                self.update_dirs(*rel, path, subdir, ddd, fff, ctx)?,
            UpdateFile(path, text) => {
                self.update_preview(path, text, ctx)?
            }
        });

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        ctx.g
            .status
            .borrow_mut()
            .status(3, format!("A {:.3?}", el).to_string());

        Ok(Control::Continue)
    }

    fn error(&self, event: Error, ctx: &mut AppContext<'_>) -> Result<Control<FilesAction>, Error> {
        ctx.g
            .error_dlg
            .borrow_mut()
            .append(format!("{:?}", &*event).as_str());
        Ok(Control::Repaint)
    }
}

impl FilesState {
    fn show_dir(&mut self) -> Result<Control<FilesAction>, Error> {
        if let Some(n) = self.w_dirs.widget.selected() {
            if let Some(sub) = self.sub_dirs.get(n) {
                if sub == &OsString::from(".") {
                    Ok(Control::Action(ReadDir(
                        Current,
                        self.main_dir.clone(),
                        None,
                    )))
                } else if sub == &OsString::from("..") {
                    Ok(Control::Action(ReadDir(
                        Parent,
                        self.main_dir.clone(),
                        None,
                    )))
                } else {
                    Ok(Control::Action(ReadDir(
                        SubDir,
                        self.main_dir.clone(),
                        Some(sub.clone()),
                    )))
                }
            } else {
                self.files.clear();
                Ok(Control::Repaint)
            }
        } else {
            Ok(Control::Repaint)
        }
    }

    fn update_preview(
        &mut self,
        path: &mut PathBuf,
        text: &mut Vec<u8>,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<FilesAction>, Error> {
        let sel = self.current_file();
        let path = mem::take(path);
        let text = mem::take(text);

        if Some(path) == sel {
            let hex = 'f: {
                for c in text.iter().take(32) {
                    if *c < 0x20 && *c != b'\n' && *c != b'\r' {
                        break 'f true;
                    }
                }
                false
            };

            let mut v = String::new();
            let w;
            let str_text = if hex {
                use std::fmt::Write;

                let mut mm = String::new();
                let mut b0 = Vec::new();
                let mut b1 = String::new();
                let hex = [
                    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
                ];

                _ = write!(mm, "{:8x} ", 0);
                for (n, b) in text.iter().enumerate() {
                    b0.push(hex[(b / 16) as usize] as u8);
                    b0.push(hex[(b % 16) as usize] as u8);
                    if n % 16 == 7 {
                        b0.push(b' ');
                    }

                    if *b < 0x20 {
                        b1.push(' ');
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
                }
                v.push_str(mm.as_str());
                _ = write!(v, "{:33}", from_utf8(&b0).expect("str"));
                v.push(' ');
                v.push_str(b1.as_str());
                v.push('\n');

                v.as_str()
            } else {
                w = String::from_utf8_lossy(&text);
                w.as_ref()
            };

            self.w_data.widget.set_value(str_text);
            Ok(Control::Repaint)
        } else {
            Ok(Control::Continue)
        }
    }

    fn update_dirs(
        &mut self,
        rel: Relative,
        path: &mut PathBuf,
        sub: &mut Option<OsString>,
        ddd: &mut Vec<OsString>,
        fff: &mut Vec<OsString>,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<FilesAction>, Error> {
        let selected = if let Some(n) = self.w_dirs.widget.selected() {
            self.sub_dirs.get(n).cloned()
        } else {
            None
        };

        let path = mem::take(path);
        let sub = mem::take(sub);
        let ddd = mem::take(ddd);
        let fff = mem::take(fff);

        match rel {
            Full => {
                self.main_dir = path;
                self.sub_dirs.clear();
                self.sub_dirs.extend(ddd.into_iter());
                self.files.clear();
                self.files.extend(fff.into_iter().map(|v| (v, false)));

                self.w_dirs.widget.select(Some(0));
                self.w_files.widget.select(Some(0));
            }
            Parent => {
                if selected == Some(OsString::from("..")) {
                    self.files.clear();
                    self.files.extend(ddd.into_iter().map(|v| (v, true)));
                    self.files.extend(fff.into_iter().map(|v| (v, false)));
                    self.w_files.widget.select(Some(0));
                }
            }
            Current => {
                if selected == Some(OsString::from(".")) {
                    self.files.clear();
                    self.files.extend(fff.into_iter().map(|v| (v, false)));
                    self.w_files.widget.select(Some(0));
                }
            }
            SubDir => {
                if selected == sub.as_ref().cloned() {
                    self.files.clear();
                    self.files.extend(ddd.into_iter().map(|v| (v, true)));
                    self.files.extend(fff.into_iter().map(|v| (v, false)));
                    self.w_files.widget.select(Some(0));
                }
            }
        }

        _ = self.show_file(ctx)?;

        Ok(Control::Repaint)
    }

    fn read_dir(
        &mut self,
        rel: Relative,
        path: &mut PathBuf,
        sub: &mut Option<OsString>,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<FilesAction>, Error> {
        let path = mem::take(path);
        let sub = mem::take(sub);

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

            let r = fs::read_dir(read_path)?;

            let mut ddd = Vec::new();
            if rel == Full {
                ddd.push(".".into());
                ddd.push("..".into());
            }
            let mut fff = Vec::new();
            for f in r {
                let cancel = {
                    if let Ok(guard) = can.lock() {
                        *guard
                    } else {
                        true
                    }
                };
                if cancel {
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
            Ok(Control::Action(Update(rel, path, sub, ddd, fff)))
        });

        Ok(Control::Continue)
    }

    fn follow_dir(&mut self, ctx: &mut AppContext<'_>) -> Result<Control<FilesAction>, Error> {
        if let Some(n) = self.w_dirs.widget.selected() {
            if let Some(sub) = self.sub_dirs.get(n) {
                if sub == &OsString::from("..") {
                    if let Some(file) = self.main_dir.parent() {
                        ctx.queue(Control::Action(ReadDir(
                            Full,
                            file.to_path_buf(),
                            Some(OsString::from("..")),
                        )));
                        ctx.queue(Control::Action(ReadDir(Parent, file.to_path_buf(), None)));
                    }
                } else if sub == &OsString::from(".") {
                    // noop
                } else {
                    let file = self.main_dir.join(sub);
                    ctx.queue(Control::Action(ReadDir(Full, file, None)))
                }
            }
        }
        Ok(Control::Continue)
    }

    fn current_dir(&mut self) -> Option<PathBuf> {
        let dir = if let Some(n) = self.w_dirs.widget.selected() {
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

        let file = if let Some(n) = self.w_files.widget.selected() {
            self.files
                .get(n)
                .map(|(v, _)| dir.map(|d| d.join(v)))
                .flatten()
        } else {
            None
        };

        file
    }

    fn show_file(&mut self, ctx: &mut AppContext<'_>) -> Result<Control<FilesAction>, Error> {
        let file = self.current_file();

        if let Some(file) = file {
            if file.is_file() {
                _ = ctx.spawn(move |can, snd| match fs::read(&file) {
                    Ok(data) => Ok(Control::Action(UpdateFile(file, data))),
                    Err(e) => Ok(Control::Action(UpdateFile(
                        file,
                        format!("{:?}", e).to_string().into_bytes(),
                    ))),
                });
                Ok(Control::Repaint)
            } else {
                self.w_data.widget.set_value("");
                Ok(Control::Repaint)
            }
        } else {
            self.w_data.widget.set_value("");
            Ok(Control::Repaint)
        }
    }

    fn follow_file(&mut self, ctx: &mut AppContext<'_>) -> Result<Control<FilesAction>, Error> {
        let file = self.current_file();

        if let Some(file) = file {
            if file.metadata()?.is_dir() {
                ctx.queue(Control::Action(ReadDir(Full, file.clone(), None)));
                ctx.queue(Control::Action(ReadDir(Parent, file, None)));
            }
        };
        Ok(Control::Repaint)
    }
}

impl HasFocus for FilesState {
    fn focus(&self) -> Focus<'_> {
        Focus::new(&[
            &self.w_dirs.widget,
            &self.w_files.widget,
            &self.w_data.widget,
            &self.w_menu,
        ])
    }
}

fn setup_logging() -> Result<(), Error> {
    _ = fs::remove_file("log.log");
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}]\n        {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("log.log")?)
        .apply()?;
    Ok(())
}
