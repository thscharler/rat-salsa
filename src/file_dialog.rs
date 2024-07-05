use crate::_private::NonExhaustive;
use crate::button::{Button, ButtonOutcome, ButtonState, ButtonStyle};
use crate::event::FileOutcome;
use crate::fill::Fill;
use crate::input::{TextInput, TextInputState, TextInputStyle};
use crate::layout::{layout_dialog, layout_grid};
use crate::list::selection::RowSelection;
use crate::list::{RList, RListState, RListStyle};
use crate::util::revert_style;
use directories_next::UserDirs;
use log::debug;
use normpath::{BasePath, BasePathBuf};
use rat_event::{ct_event, flow, flow_ok, Dialog, FocusKeys, HandleEvent, Outcome};
use rat_focus::{match_focus, on_lost, Focus, HasFocusFlag};
use rat_ftable::event::DoubleClickOutcome;
use rat_scrolled::Scroll;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Flex, Layout, Margin, Rect};
use ratatui::prelude::{BlockExt, StatefulWidget, Style, Text, Widget};
use ratatui::widgets::{Block, ListItem, WidgetRef};
use std::ffi::OsString;
use std::fmt::{Debug, Formatter};
use std::io;
use std::path::{Path, PathBuf};
use sysinfo::Disks;

#[derive(Debug, Default, Clone)]
pub struct FileOpen<'a> {
    block: Option<Block<'a>>,

    style: Style,
    list_style: Option<Style>,
    path_style: Option<Style>,
    name_style: Option<Style>,
    invalid_style: Option<Style>,
    select_style: Option<Style>,
    focus_style: Option<Style>,
    button_style: Option<Style>,
    armed_style: Option<Style>,

    ok_text: &'a str,
    cancel_text: &'a str,
}

#[derive(Debug)]
pub struct FileStyle {
    pub style: Style,
    pub list: Option<Style>,
    pub path: Option<Style>,
    pub name: Option<Style>,
    pub invalid: Option<Style>,
    pub select: Option<Style>,
    pub focus: Option<Style>,
    pub button: Option<Style>,
    pub armed: Option<Style>,

    pub non_exhaustive: NonExhaustive,
}

/// Dialog mode
#[derive(Debug, PartialEq, Eq)]
enum Mode {
    Open,
    Save,
    Dir,
}

pub struct FileOpenState {
    pub active: bool,

    mode: Mode,

    path: PathBuf,
    save_name: Option<OsString>,
    dirs: Vec<OsString>,
    filter: Option<Box<dyn Fn(&Path) -> bool + 'static>>,
    files: Vec<OsString>,
    use_default_roots: bool,
    roots: Vec<(OsString, PathBuf)>,

    path_state: TextInputState,
    root_state: RListState<RowSelection>,
    dir_state: RListState<RowSelection>,
    file_state: RListState<RowSelection>,
    name_state: TextInputState,
    cancel_state: ButtonState,
    ok_state: ButtonState,
}

impl Debug for FileOpenState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileOpenState")
            .field("active", &self.active)
            .field("mode", &self.mode)
            .field("path", &self.path)
            .field("save_name", &self.save_name)
            .field("dirs", &self.dirs)
            .field("files", &self.files)
            .field("use_default_roots", &self.use_default_roots)
            .field("roots", &self.roots)
            .field("path_state", &self.path_state)
            .field("root_state", &self.root_state)
            .field("dir_state", &self.dir_state)
            .field("file_state", &self.file_state)
            .field("name_state", &self.name_state)
            .field("cancel_state", &self.cancel_state)
            .field("ok_state", &self.ok_state)
            .finish()
    }
}

impl Default for FileStyle {
    fn default() -> Self {
        FileStyle {
            style: Default::default(),
            list: None,
            path: None,
            name: None,
            invalid: None,
            select: None,
            focus: None,
            button: None,
            armed: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl Default for FileOpenState {
    fn default() -> Self {
        let mut s = Self {
            active: false,
            mode: Mode::Open,
            path: Default::default(),
            save_name: None,
            dirs: vec![],
            filter: None,
            files: vec![],
            use_default_roots: false,
            roots: vec![],
            path_state: Default::default(),
            root_state: Default::default(),
            dir_state: Default::default(),
            file_state: Default::default(),
            name_state: Default::default(),
            cancel_state: Default::default(),
            ok_state: Default::default(),
        };
        s.use_default_roots = true;
        s.dir_state.set_scroll_selection(true);
        s.file_state.set_scroll_selection(true);
        s
    }
}

impl<'a> FileOpen<'a> {
    pub fn new() -> Self {
        Self {
            block: None,
            style: Default::default(),
            list_style: None,
            path_style: None,
            name_style: None,
            invalid_style: None,
            select_style: None,
            focus_style: None,
            button_style: None,
            armed_style: None,
            ok_text: "Ok",
            cancel_text: "Cancel",
        }
    }

    pub fn ok_text(mut self, txt: &'a str) -> Self {
        self.ok_text = txt;
        self
    }

    pub fn cancel_text(mut self, txt: &'a str) -> Self {
        self.cancel_text = txt;
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn list_style(mut self, style: Style) -> Self {
        self.list_style = Some(style);
        self
    }

    pub fn path_style(mut self, style: Style) -> Self {
        self.path_style = Some(style);
        self
    }

    pub fn name_style(mut self, style: Style) -> Self {
        self.name_style = Some(style);
        self
    }

    pub fn invalid_style(mut self, style: Style) -> Self {
        self.invalid_style = Some(style);
        self
    }

    pub fn select_style(mut self, style: Style) -> Self {
        self.select_style = Some(style);
        self
    }

    pub fn focus_style(mut self, style: Style) -> Self {
        self.focus_style = Some(style);
        self
    }

    pub fn button_style(mut self, style: Style) -> Self {
        self.button_style = Some(style);
        self
    }

    pub fn armed_style(mut self, style: Style) -> Self {
        self.armed_style = Some(style);
        self
    }

    pub fn styles(mut self, styles: FileStyle) -> Self {
        self.style = styles.style;
        self.list_style = styles.list;
        self.path_style = styles.path;
        self.name_style = styles.name;
        self.invalid_style = styles.invalid;
        self.select_style = styles.select;
        self.focus_style = styles.focus;
        self.button_style = styles.button;
        self.armed_style = styles.armed;
        self
    }

    fn defaulted_focus(&self) -> Option<Style> {
        if let Some(focus) = &self.focus_style {
            Some(*focus)
        } else {
            Some(revert_style(self.style))
        }
    }

    fn defaulted_select(&self) -> Option<Style> {
        if let Some(select) = &self.select_style {
            Some(*select)
        } else {
            Some(revert_style(self.style))
        }
    }

    fn style_roots(&self) -> RListStyle {
        RListStyle {
            style: self.style,
            select_style: self.defaulted_select(),
            focus_style: self.defaulted_focus(),
            ..Default::default()
        }
    }

    fn style_lists(&self) -> RListStyle {
        RListStyle {
            style: if let Some(list) = self.list_style {
                list
            } else {
                self.style
            },
            select_style: self.defaulted_select(),
            focus_style: self.defaulted_focus(),
            ..Default::default()
        }
    }

    fn style_name(&self) -> TextInputStyle {
        TextInputStyle {
            style: if let Some(name) = self.name_style {
                name
            } else {
                self.style
            },
            focus: self.defaulted_focus(),
            select: self.defaulted_select(),
            invalid: self.invalid_style,
            ..Default::default()
        }
    }

    fn style_path(&self) -> TextInputStyle {
        TextInputStyle {
            style: if let Some(path) = self.path_style {
                path
            } else {
                self.style
            },
            focus: self.defaulted_focus(),
            select: self.defaulted_select(),
            invalid: self.invalid_style,
            ..Default::default()
        }
    }

    fn style_button(&self) -> ButtonStyle {
        ButtonStyle {
            style: if let Some(button) = self.button_style {
                button
            } else {
                self.style
            },
            focus: self.defaulted_focus(),
            armed: self.armed_style,
            ..Default::default()
        }
    }
}

impl<'a> StatefulWidget for FileOpen<'a> {
    type State = FileOpenState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if !state.active {
            return;
        }

        let layout = layout_dialog(
            area,
            Constraint::Fill(1),
            Constraint::Fill(1),
            Margin::new(1, 1),
            [Constraint::Length(10), Constraint::Length(10)],
            1,
            Flex::End,
        );

        let inner = if self.block.is_some() {
            let inner = self.block.inner_if_some(area);
            self.block.render_ref(area, buf);
            inner
        } else {
            let block = Block::bordered()
                .title(match state.mode {
                    Mode::Open => " Open ",
                    Mode::Save => " Save ",
                    Mode::Dir => " Directory ",
                })
                .style(self.style);
            let inner = block.inner(area);
            block.render_ref(area, buf);
            inner
        };

        Fill::new().style(self.style).render(inner, buf);

        match state.mode {
            Mode::Open => {
                render_open(&self, layout.area, buf, state);
            }
            Mode::Save => {
                render_save(&self, layout.area, buf, state);
            }
            Mode::Dir => {}
        }

        Button::new()
            .text(Text::from(self.cancel_text).alignment(Alignment::Center))
            .styles(self.style_button())
            .render(layout.button(0), buf, &mut state.cancel_state);

        Button::new()
            .text(Text::from(self.ok_text).alignment(Alignment::Center))
            .styles(self.style_button())
            .render(layout.button(1), buf, &mut state.ok_state);
    }
}

fn render_open(widget: &FileOpen<'_>, area: Rect, buf: &mut Buffer, state: &mut FileOpenState) {
    let l_grid = layout_grid::<3, 3>(
        area,
        Layout::horizontal([
            Constraint::Percentage(20),
            Constraint::Percentage(30),
            Constraint::Percentage(50),
        ]),
        Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ],
        ),
    );

    //
    let mut l_path = l_grid[1][1].union(l_grid[2][1]);
    l_path.width = l_path.width.saturating_sub(1);
    debug!("path {:#?}", widget.style_path());
    TextInput::new()
        .styles(widget.style_path())
        .render(l_path, buf, &mut state.path_state);

    RList::default()
        .items(state.roots.iter().map(|v| {
            let s = v.0.to_string_lossy();
            ListItem::from(format!("{}", s))
        }))
        .scroll(Scroll::new())
        .styles(widget.style_roots())
        .render(l_grid[0][2], buf, &mut state.root_state);
    state.root_state.area = l_grid[0][2];

    RList::default()
        .items(state.dirs.iter().map(|v| {
            let s = v.to_string_lossy();
            ListItem::from(s)
        }))
        .scroll(Scroll::new())
        .styles(widget.style_lists())
        .render(l_grid[1][2], buf, &mut state.dir_state);
    state.dir_state.area = l_grid[1][2];

    RList::default()
        .items(state.files.iter().map(|v| {
            let s = v.to_string_lossy();
            ListItem::from(s)
        }))
        .scroll(Scroll::new())
        .styles(widget.style_lists())
        .render(l_grid[2][2], buf, &mut state.file_state);
    state.file_state.area = l_grid[2][2];
}

fn render_save(widget: &FileOpen<'_>, area: Rect, buf: &mut Buffer, state: &mut FileOpenState) {
    let l_grid = layout_grid::<3, 4>(
        area,
        Layout::horizontal([
            Constraint::Percentage(20),
            Constraint::Percentage(30),
            Constraint::Percentage(50),
        ]),
        Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(1),
            ],
        ),
    );

    //
    let mut l_path = l_grid[1][1].union(l_grid[2][1]);
    l_path.width = l_path.width.saturating_sub(1);
    TextInput::new()
        .styles(widget.style_path())
        .render(l_path, buf, &mut state.path_state);

    RList::default()
        .items(state.roots.iter().map(|v| {
            let s = v.0.to_string_lossy();
            ListItem::from(format!("{}", s))
        }))
        .scroll(Scroll::new())
        .styles(widget.style_roots())
        .render(l_grid[0][2], buf, &mut state.root_state);
    state.root_state.area = l_grid[0][2];

    RList::default()
        .items(state.dirs.iter().map(|v| {
            let s = v.to_string_lossy();
            ListItem::from(s)
        }))
        .scroll(Scroll::new())
        .styles(widget.style_lists())
        .render(l_grid[1][2], buf, &mut state.dir_state);
    state.dir_state.area = l_grid[1][2];

    RList::default()
        .items(state.files.iter().map(|v| {
            let s = v.to_string_lossy();
            ListItem::from(s)
        }))
        .scroll(Scroll::new())
        .styles(widget.style_lists())
        .render(l_grid[2][2], buf, &mut state.file_state);
    state.file_state.area = l_grid[2][2];

    TextInput::new()
        .styles(widget.style_name())
        .render(l_grid[2][3], buf, &mut state.name_state);
}

impl FileOpenState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_filter(&mut self, filter: impl Fn(&Path) -> bool + 'static) {
        self.filter = Some(Box::new(filter));
    }

    pub fn use_default_roots(&mut self, roots: bool) {
        self.use_default_roots = roots;
    }

    pub fn add_root(&mut self, name: impl AsRef<str>, path: impl Into<PathBuf>) {
        self.roots
            .push((OsString::from(name.as_ref()), path.into()))
    }

    pub fn clear_roots(&mut self) {
        self.roots.clear();
    }

    /// Append the default roots.
    pub fn default_roots(&mut self, start: PathBuf) {
        if let Some(user) = UserDirs::new() {
            self.roots.push((
                OsString::from("Start"), //
                start,
            ));
            self.roots.push((
                OsString::from("Home"), //
                user.home_dir().to_path_buf(),
            ));
            self.roots.push((
                OsString::from("Documents"),
                user.document_dir().unwrap().to_path_buf(),
            ));
        }

        let disks = Disks::new_with_refreshed_list();
        for d in disks.list() {
            self.roots
                .push((d.name().to_os_string(), d.mount_point().to_path_buf()));
        }

        self.root_state.select(Some(0));
    }

    pub fn open_dialog(&mut self, path: &Path) -> Result<(), io::Error> {
        self.active = true;
        self.mode = Mode::Open;
        self.set_path(path.into())?;
        if self.use_default_roots {
            self.default_roots(self.path.clone());
        }
        self.focus().initial();
        Ok(())
    }

    pub fn save_dialog(
        &mut self,
        path: &Path,
        name: Option<impl AsRef<str>>,
    ) -> Result<(), io::Error> {
        self.active = true;
        self.mode = Mode::Save;
        self.save_name = if let Some(name) = name {
            Some(OsString::from(name.as_ref()))
        } else {
            None
        };
        self.set_path(path.into())?;
        if self.use_default_roots {
            self.default_roots(self.path.clone());
        }
        self.focus().initial();
        Ok(())
    }

    fn find_parent(&self, path: &Path) -> Option<PathBuf> {
        if path == Path::new(".") || path.file_name().is_none() {
            let parent = path.join("..");
            let canon_parent = parent.canonicalize().ok();
            let canon_path = path.canonicalize().ok();
            if canon_parent == canon_path {
                None
            } else if parent.exists() && parent.is_dir() {
                Some(parent)
            } else {
                None
            }
        } else if let Some(parent) = path.parent() {
            if parent.exists() && parent.is_dir() {
                Some(parent.to_path_buf())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn set_path(&mut self, path: &Path) -> Result<FileOutcome, io::Error> {
        let old = self.path.clone();
        let path = path.to_path_buf();

        if old != path {
            let mut dirs = Vec::new();
            let mut files = Vec::new();

            if let Some(parent) = self.find_parent(&path) {
                dirs.push(OsString::from(".."));
            }

            for r in path.read_dir()? {
                let Ok(r) = r else {
                    continue;
                };

                if let Ok(meta) = r.metadata() {
                    if meta.is_dir() {
                        dirs.push(r.file_name());
                    } else if meta.is_file() {
                        if let Some(filter) = self.filter.as_ref() {
                            if filter(&r.path()) {
                                files.push(r.file_name());
                            }
                        } else {
                            files.push(r.file_name());
                        }
                    }
                }
            }

            self.path = path;
            self.dirs = dirs;
            self.files = files;

            debug!("set path {:?}", self.path);
            self.path_state.set_value(self.path.to_string_lossy());
            if self.path_state.value.width() != 0 {
                // only works when this has been rendered once. todo:
                self.path_state.move_to_line_end(false);
            }

            if self.dirs.len() > 0 {
                self.dir_state.select(Some(0));
            } else {
                self.dir_state.select(None);
            }
            self.dir_state.set_offset(0);
            if self.files.len() > 0 {
                self.file_state.select(Some(0));
                if let Some(name) = &self.save_name {
                    self.name_state.set_value(name.to_string_lossy());
                } else {
                    self.name_state.set_value(self.files[0].to_string_lossy());
                }
            } else {
                self.file_state.select(None);
                if let Some(name) = &self.save_name {
                    self.name_state.set_value(name.to_string_lossy());
                } else {
                    self.name_state.set_value("");
                }
            }
            self.file_state.set_offset(0);

            Ok(FileOutcome::Changed)
        } else {
            Ok(FileOutcome::Unchanged)
        }
    }

    fn use_named_path(&mut self) -> Result<FileOutcome, io::Error> {
        let path = if cfg!(windows) {
            let path = BasePathBuf::new(self.path_state.value())?;
            let path = path.normalize_virtually()?;
            path.into_path_buf()
        } else {
            PathBuf::from(self.path_state.value())
        };

        if !path.exists() || !path.is_dir() {
            self.path_state.invalid = true;
        } else {
            self.path_state.invalid = false;
            self.set_path(&path)?;
        }

        Ok(FileOutcome::Changed)
    }

    fn chdir(&mut self, dir: &OsString) -> Result<FileOutcome, io::Error> {
        if dir == &OsString::from("..") {
            if let Some(parent) = self.find_parent(&self.path) {
                self.set_path(&parent)
            } else {
                Ok(FileOutcome::Unchanged)
            }
        } else {
            self.set_path(&self.path.join(dir))
        }
    }

    fn chroot_selected(&mut self) -> Result<FileOutcome, io::Error> {
        if let Some(select) = self.root_state.selected() {
            if let Some(d) = self.roots.get(select).cloned() {
                self.set_path(&d.1)?;
                return Ok(FileOutcome::Changed);
            }
        }
        Ok(FileOutcome::Unchanged)
    }

    fn chdir_selected(&mut self) -> Result<FileOutcome, io::Error> {
        if let Some(select) = self.dir_state.selected() {
            if let Some(dir) = self.dirs.get(select).cloned() {
                self.chdir(&dir)?;
                return Ok(FileOutcome::Changed);
            }
        }
        Ok(FileOutcome::Unchanged)
    }

    fn name_selected(&mut self) -> Result<FileOutcome, io::Error> {
        if let Some(select) = self.file_state.selected() {
            if let Some(file) = self.files.get(select).cloned() {
                let name = file.to_string_lossy();
                self.name_state.set_value(name);
                return Ok(FileOutcome::Changed);
            }
        }
        Ok(FileOutcome::Unchanged)
    }

    /// Cancel the dialog.
    fn close_cancel(&mut self) -> FileOutcome {
        self.active = false;
        FileOutcome::Cancel
    }

    /// Choose the selected and close the dialog.
    fn choose_selected(&mut self) -> FileOutcome {
        if self.mode == Mode::Open {
            if let Some(select) = self.file_state.selected() {
                if let Some(file) = self.files.get(select).cloned() {
                    self.active = false;
                    return FileOutcome::Ok(self.path.join(file));
                }
            }
        } else if self.mode == Mode::Save {
            let path = self.path.join(self.name_state.value().trim());
            return FileOutcome::Ok(path);
        }
        FileOutcome::Unchanged
    }

    pub fn screen_cursor(&self) -> Option<(u16, u16)> {
        match_focus!(
            self.path_state => {
                self.path_state.screen_cursor()
            },
            self.name_state => {
                self.name_state.screen_cursor()
            },
            _ => None
        )
    }
}

impl FileOpenState {
    fn focus(&self) -> Focus<'_> {
        let mut f = Focus::default();
        f.add(&self.dir_state);
        f.add(&self.file_state);
        if self.mode == Mode::Save {
            f.add(&self.name_state);
        }
        f.add(&self.ok_state);
        f.add(&self.cancel_state);
        f.add(&self.root_state);
        f.add(&self.path_state);
        f
    }
}

impl HandleEvent<crossterm::event::Event, Dialog, Result<FileOutcome, io::Error>>
    for FileOpenState
{
    fn handle(
        &mut self,
        event: &crossterm::event::Event,
        _qualifier: Dialog,
    ) -> Result<FileOutcome, io::Error> {
        if !self.active {
            return Ok(FileOutcome::NotUsed);
        }
        if matches!(event, ct_event!(mouse moved)) {
            return Ok(FileOutcome::NotUsed);
        }

        let mut focus_outcome = self.focus().handle(event, FocusKeys).into();

        flow_ok!(self.path_state.handle(event, FocusKeys), consider focus_outcome);
        if self.path_state.is_focused() {
            flow_ok!(match event {
                ct_event!(keycode press Enter) => {
                    self.use_named_path()?;
                    self.focus().focus_widget_no_lost(&self.dir_state);
                    FileOutcome::Changed
                }
                _ => FileOutcome::NotUsed,
            });
        }
        on_lost!(
            self.path_state => {
                self.use_named_path()?
            }
        );

        if self.mode == Mode::Save {
            flow_ok!(self.name_state.handle(event, FocusKeys), consider focus_outcome);
            if self.name_state.is_focused() {
                flow_ok!(match event {
                    ct_event!(keycode press Enter) => {
                        self.choose_selected()
                    }
                    _ => FileOutcome::NotUsed,
                });
            }
            on_lost!(
                self.path_state => {
                    self.set_path(&PathBuf::from(self.path_state.value()))?;
                    focus_outcome = FileOutcome::Changed;
                }
            );
        }

        flow_ok!(match self.cancel_state.handle(event, FocusKeys) {
            ButtonOutcome::Pressed => {
                self.close_cancel()
            }
            _ => FileOutcome::NotUsed,
        });
        flow_ok!(match event {
            ct_event!(keycode press Esc) => {
                self.close_cancel()
            }
            _ => FileOutcome::NotUsed,
        });

        flow_ok!(match self.ok_state.handle(event, FocusKeys) {
            ButtonOutcome::Pressed => {
                self.choose_selected()
            }
            r => r.into(),
        });

        let mut file_state_ext = (&mut self.file_state, self.files.as_slice());
        flow_ok!(match file_state_ext.handle(event, Navigate) {
            DoubleClickOutcome::ClickClick(_, _) => {
                self.choose_selected()
            }
            DoubleClickOutcome::Changed => {
                if self.mode == Mode::Save {
                    self.name_selected()?
                } else {
                    FileOutcome::Changed
                }
            }
            r => r.into(),
        }, consider focus_outcome);
        if self.file_state.is_focused() {
            flow_ok!(match event {
                ct_event!(keycode press Enter) => {
                    self.choose_selected()
                }
                _ => FileOutcome::NotUsed,
            })
        }

        let mut dir_state_ext = (&mut self.dir_state, self.dirs.as_slice());
        flow_ok!(match dir_state_ext.handle(event, Navigate) {
            DoubleClickOutcome::ClickClick(_, _) => {
                self.chdir_selected()?
            }
            r => r.into(),
        }, consider focus_outcome);
        if self.dir_state.is_focused() {
            flow_ok!(match event {
                ct_event!(keycode press Enter) => self.chdir_selected()?,
                _ => FileOutcome::NotUsed,
            })
        }

        flow_ok!(match self.root_state.handle(event, FocusKeys) {
            Outcome::Changed => {
                self.chroot_selected()?
            }
            r => r.into(),
        }, consider focus_outcome);

        Ok(FileOutcome::Unchanged | focus_outcome)
    }
}

struct Navigate;

impl HandleEvent<crossterm::event::Event, Navigate, DoubleClickOutcome>
    for (&mut RListState<RowSelection>, &[OsString])
{
    fn handle(
        &mut self,
        event: &crossterm::event::Event,
        _qualifier: Navigate,
    ) -> DoubleClickOutcome {
        let state = &mut self.0;
        let nav = &self.1;

        if state.is_focused() {
            flow!(match event {
                ct_event!(mouse any for m) if state.mouse.doubleclick(state.inner, m) => {
                    DoubleClickOutcome::ClickClick(0, state.selected().expect("select"))
                }
                ct_event!(key press c) => {
                    let c = c.to_lowercase().next();
                    let start = state.selected().unwrap_or(0);

                    let mut idx = start;
                    let mut selected = None;
                    loop {
                        idx += 1;
                        if idx >= nav.len() {
                            idx = 0;
                        }
                        if idx == start {
                            break;
                        }

                        let nav = nav[idx]
                            .to_string_lossy()
                            .chars()
                            .next()
                            .map(|v| v.to_lowercase().next())
                            .flatten();
                        if c == nav {
                            selected = Some(idx);
                            break;
                        }
                    }

                    if let Some(selected) = selected {
                        state.move_to(selected);
                    }
                    DoubleClickOutcome::Changed
                }
                _ => DoubleClickOutcome::NotUsed,
            });
        }
        flow!(state.handle(event, FocusKeys));
        DoubleClickOutcome::NotUsed
    }
}
