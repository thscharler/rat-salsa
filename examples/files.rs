#![allow(unused_variables)]

use crate::FilesAction::{Message, ReadDir, ReadFile, Update, UpdateFile};
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
    ct_event, flow_ok, DoubleClick, DoubleClickOutcome, EditKeys, FocusKeys, HandleEvent, Outcome,
    ReadOnly, ScrollOutcome,
};
use rat_widget::focus::{match_focus, on_lost, Focus, HasFocus, HasFocusFlag};
use rat_widget::list::selection::RowSelection;
use rat_widget::menuline::{MenuOutcome, RMenuLine, RMenuLineState};
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::scrolled::{Scrolled, ScrolledState};
use rat_widget::statusline::{StatusLine, StatusLineState};
use rat_widget::table::textdata::{Cell, Row};
use rat_widget::table::{FTableContext, RTable, RTableState, TableData};
use rat_widget::textarea::{RTextArea, RTextAreaState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{StatefulWidget, Widget};
use std::borrow::Cow;
use std::cell::RefCell;
use std::ffi::{OsStr, OsString};
use std::ops::Deref;
use std::path::{Path, PathBuf};
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
    ReadFile(PathBuf),
    Update(
        Relative,
        PathBuf,
        Option<OsString>,
        Vec<OsString>,
        Vec<OsString>,
    ),
    UpdateFile(PathBuf, String),
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
    pub current_dir: PathBuf,
    pub sub_dirs: Vec<OsString>,
    pub files: Vec<OsString>,

    pub w_dirs: ScrolledState<RTableState<RowSelection>>,
    pub w_files: ScrolledState<RTableState<RowSelection>>,
    pub w_data: ScrolledState<RTextAreaState>,
    pub w_menu: RMenuLineState,
}

struct FileData<'a> {
    files: &'a [OsString],
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
        let item = &self.files[row];
        match column {
            0 => {
                let name = item.to_string_lossy();
                name.render(area, buf);
            }
            _ => {}
        }
    }
}

struct DirData<'a> {
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
                name.render(area, buf);
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
        Text::from(state.current_dir.to_string_lossy())
            .style(ctx.g.theme.bluegreen(0))
            .render(r[0], buf);

        Scrolled::new(
            RTable::new()
                .data(DirData {
                    dirs: &state.sub_dirs,
                })
                .styles(ctx.g.theme.table_style()),
        )
        .styles(ctx.g.theme.scrolled_style())
        .render(c[0], buf, &mut state.w_dirs);

        Scrolled::new(
            RTable::new()
                .data(FileData {
                    files: &state.files,
                })
                .styles(ctx.g.theme.table_style()),
        )
        .styles(ctx.g.theme.scrolled_style())
        .render(c[1], buf, &mut state.w_files);

        Scrolled::new(RTextArea::new().styles(ctx.g.theme.textarea_style()))
            .styles(ctx.g.theme.scrolled_style())
            .render(c[2], buf, &mut state.w_data);

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
        self.current_dir = if let Ok(dot) = PathBuf::from(".").canonicalize() {
            dot
        } else {
            PathBuf::from(".")
        };
        ctx.queue(Control::Action(ReadDir(
            Full,
            self.current_dir.clone(),
            None,
        )));

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
                self.show_files()?
            }
            v => Control::from(v),
        });

        flow_ok!(self.w_data.handle(event, FocusKeys));

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
            ReadFile(path) => {
                self.read_file(path, ctx)?
            }
            UpdateFile(path, text) => {
                self.update_file(path, text, ctx)?
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
    fn show_files(&mut self) -> Result<Control<FilesAction>, Error> {
        if let Some(n) = self.w_dirs.widget.selected() {
            if let Some(sub) = self.sub_dirs.get(n) {
                if sub == &OsString::from(".") {
                    Ok(Control::Action(ReadDir(
                        Current,
                        self.current_dir.clone(),
                        None,
                    )))
                } else if sub == &OsString::from("..") {
                    Ok(Control::Action(ReadDir(
                        Parent,
                        self.current_dir.clone(),
                        None,
                    )))
                } else {
                    Ok(Control::Action(ReadDir(
                        SubDir,
                        self.current_dir.clone(),
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

    fn update_file(
        &mut self,
        path: &mut PathBuf,
        text: &mut String,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<FilesAction>, Error> {
        let sel = self.current_file();
        let path = mem::take(path);
        let text = mem::take(text);

        if Some(path) == sel {
            self.w_data.widget.set_value(text.as_str()); // todo: might be slow?
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

        match rel {
            Full => {
                self.current_dir = mem::take(path);
                self.sub_dirs.clear();
                self.sub_dirs.extend_from_slice(ddd);
                self.files.clear();
                self.files.extend_from_slice(fff);

                self.w_dirs.widget.select(Some(0));
                self.w_files.widget.select(Some(0));
            }
            Parent => {
                if selected == Some(OsString::from("..")) {
                    self.files.clear();
                    self.files.extend_from_slice(ddd);
                    self.files.extend_from_slice(fff);
                    self.w_files.widget.select(Some(0));
                }
            }
            Current => {
                if selected == Some(OsString::from(".")) {
                    self.files.clear();
                    self.files.extend_from_slice(fff);
                    self.w_files.widget.select(Some(0));
                }
            }
            SubDir => {
                if selected == sub.as_ref().cloned() {
                    self.files.clear();
                    self.files.extend_from_slice(ddd);
                    self.files.extend_from_slice(fff);
                    self.w_files.widget.select(Some(0));
                }
            }
        }

        Ok(Control::Repaint)
    }

    fn read_file(
        &mut self,
        path: &mut PathBuf,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<FilesAction>, Error> {
        let path = mem::take(path);

        _ = ctx.spawn(move |can, snd| {
            let data = fs::read_to_string(&path)?;
            Ok(Control::Action(UpdateFile(path, data)))
        });

        Ok(Control::Continue)
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
            }
            if rel == Full || rel == Parent {
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
                    debug!("cancel");
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
                    if let Some(file) = self.current_dir.parent() {
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
                    let file = self.current_dir.join(sub);
                    ctx.queue(Control::Action(ReadDir(Full, file, None)))
                }
            }
        }
        Ok(Control::Continue)
    }

    fn current_file(&mut self) -> Option<PathBuf> {
        let dir = if let Some(n) = self.w_dirs.widget.selected() {
            if let Some(sub) = self.sub_dirs.get(n) {
                if sub == &OsString::from("..") {
                    self.current_dir.parent().map(|v| v.to_path_buf())
                } else if sub == &OsString::from(".") {
                    Some(self.current_dir.clone())
                } else {
                    Some(self.current_dir.join(sub))
                }
            } else {
                None
            }
        } else {
            None
        };

        let file = if let Some(n) = self.w_files.widget.selected() {
            self.files.get(n).map(|v| dir.map(|d| d.join(v))).flatten()
        } else {
            None
        };

        file
    }

    fn show_file(&mut self, ctx: &mut AppContext<'_>) -> Result<Control<FilesAction>, Error> {
        let file = self.current_file();

        if let Some(file) = file {
            if let Some(ext) = file.extension() {
                let ext = ext.to_string_lossy();

                match ext.as_ref() {
                    "rs" | "toml" => return Ok(Control::Action(ReadFile(file))),
                    _ => {}
                }
            }
        }

        self.w_data.widget.set_value("");
        Ok(Control::Repaint)
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
