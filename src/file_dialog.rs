use crate::button::{Button, ButtonOutcome, ButtonState};
use crate::event::FileOutcome;
use crate::event::{ct_event, flow_ok, ConsumedEvent, Dialog, FocusKeys, HandleEvent, Outcome};
use crate::fill::Fill;
use crate::input::{TextInput, TextInputState};
use crate::layout::{layout_dialog, layout_grid};
use crate::list::selection::RowSelection;
use crate::list::{RList, RListState};
use crate::util::revert_style;
use directories_next::UserDirs;
use log::debug;
use rat_event::flow;
use rat_focus::{match_focus, on_lost, Focus, HasFocusFlag};
use rat_ftable::event::DoubleClickOutcome;
use rat_scrolled::Scroll;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Flex, Layout, Margin, Rect};
use ratatui::prelude::{BlockExt, StatefulWidget, Style, Text, Widget};
use ratatui::widgets::{Block, ListItem};
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
    select_style: Option<Style>,
    focus_style: Option<Style>,
    button_style: Option<Style>,
    armed_style: Option<Style>,

    ok_text: &'a str,
    cancel_text: &'a str,
}

pub struct FileOpenState {
    pub active: bool,

    path: PathBuf,
    dirs: Vec<OsString>,
    filter: Option<Box<dyn Fn(&Path) -> bool + 'static>>,
    files: Vec<OsString>,
    use_default_roots: bool,
    roots: Vec<(OsString, PathBuf)>,

    path_state: TextInputState,
    root_state: RListState<RowSelection>,
    dir_state: RListState<RowSelection>,
    file_state: RListState<RowSelection>,
    cancel_state: ButtonState,
    ok_state: ButtonState,
}

impl Debug for FileOpenState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileOpenState")
            .field("active", &self.active)
            .field("path", &self.path)
            .field("dirs", &self.dirs)
            .field("files", &self.files)
            .field("use_default_roots", &self.use_default_roots)
            .field("roots", &self.roots)
            .field("path_state", &self.path_state)
            .field("root_state", &self.root_state)
            .field("dir_state", &self.dir_state)
            .field("file_state", &self.file_state)
            .field("cancel_state", &self.cancel_state)
            .field("ok_state", &self.ok_state)
            .finish()
    }
}

impl Default for FileOpenState {
    fn default() -> Self {
        let mut s = Self {
            active: Default::default(),
            path: Default::default(),
            dirs: Default::default(),
            filter: None,
            files: Default::default(),
            use_default_roots: true,
            roots: Default::default(),
            path_state: Default::default(),
            root_state: Default::default(),
            dir_state: Default::default(),
            file_state: Default::default(),
            cancel_state: Default::default(),
            ok_state: Default::default(),
        };
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

        let l_grid = layout_grid::<3, 3>(
            layout.area,
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
        let select_style = if let Some(select) = self.select_style {
            select
        } else {
            revert_style(self.style)
        };
        let focus_style = if let Some(focus) = self.focus_style {
            focus
        } else {
            revert_style(self.style)
        };
        let list_style = if let Some(list) = self.list_style {
            list
        } else {
            self.style
        };

        //
        let inner = if self.block.is_some() {
            let inner = self.block.inner_if_some(area);
            self.block.render(area, buf);
            inner
        } else {
            let block = Block::bordered().title(" Open ").style(self.style);
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        };

        Fill::new().style(self.style).render(inner, buf);

        let mut path = TextInput::new()
            .select_style(select_style)
            .focus_style(focus_style);
        if let Some(style) = self.path_style {
            path = path.style(style);
        }
        let mut l_path = l_grid[1][1].union(l_grid[2][1]);
        l_path.width = l_path.width.saturating_sub(1);
        path.render(l_path, buf, &mut state.path_state);

        RList::default()
            .items(state.roots.iter().map(|v| {
                let s = v.0.to_string_lossy();
                ListItem::from(format!("{}", s))
            }))
            .scroll(Scroll::new())
            .style(self.style)
            .focus_style(focus_style)
            .select_style(select_style)
            .render(l_grid[0][2], buf, &mut state.root_state);
        state.root_state.area = l_grid[0][2];

        RList::default()
            .items(state.dirs.iter().map(|v| {
                let s = v.to_string_lossy();
                ListItem::from(s)
            }))
            .scroll(Scroll::new())
            .style(list_style)
            .focus_style(focus_style)
            .select_style(select_style)
            .render(l_grid[1][2], buf, &mut state.dir_state);
        state.dir_state.area = l_grid[1][2];

        RList::default()
            .items(state.files.iter().map(|v| {
                let s = v.to_string_lossy();
                ListItem::from(s)
            }))
            .scroll(Scroll::new())
            .style(list_style)
            .focus_style(focus_style)
            .select_style(select_style)
            .render(l_grid[2][2], buf, &mut state.file_state);
        state.file_state.area = l_grid[2][2];

        let mut cancel = Button::new()
            .text(Text::from(self.cancel_text).alignment(Alignment::Center))
            .focus_style(focus_style);
        if let Some(style) = self.button_style {
            cancel = cancel.style(style);
        }
        if let Some(style) = self.armed_style {
            cancel = cancel.armed_style(style);
        }
        cancel.render(layout.button(0), buf, &mut state.cancel_state);

        let mut ok = Button::new()
            .text(Text::from(self.ok_text).alignment(Alignment::Center))
            .focus_style(focus_style);
        if let Some(style) = self.button_style {
            ok = ok.style(style);
        }
        if let Some(style) = self.armed_style {
            ok = ok.armed_style(style);
        }
        ok.render(layout.button(1), buf, &mut state.ok_state);
    }
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

    fn default_roots(&mut self) {
        if let Some(user) = UserDirs::new() {
            self.roots.push((
                OsString::from("Start"), //
                self.path.clone(),
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

    pub fn open(&mut self, path: &Path) -> Result<(), io::Error> {
        self.active = true;
        self.set_path(path.into())?;
        if self.use_default_roots {
            self.default_roots();
        }
        self.focus().init();
        Ok(())
    }

    pub fn find_parent(&self, path: &Path) -> Option<PathBuf> {
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

            self.path_state.set_value(self.path.to_string_lossy());
            self.path_state.move_to_line_end(false);

            if self.dirs.len() > 0 {
                self.dir_state.select(Some(0));
            } else {
                self.dir_state.select(None);
            }
            self.dir_state.set_offset(0);
            if self.files.len() > 0 {
                self.file_state.select(Some(0));
            } else {
                self.file_state.select(None);
            }
            self.file_state.set_offset(0);

            Ok(FileOutcome::Changed)
        } else {
            Ok(FileOutcome::Unchanged)
        }
    }

    pub fn chdir(&mut self, dir: &OsString) -> Result<FileOutcome, io::Error> {
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

    pub fn chroot_selected(&mut self) -> Result<FileOutcome, io::Error> {
        if let Some(select) = self.root_state.selected() {
            if let Some(d) = self.roots.get(select).cloned() {
                self.set_path(&d.1)?;
                return Ok(FileOutcome::Changed);
            }
        }
        Ok(FileOutcome::Unchanged)
    }

    pub fn chdir_selected(&mut self) -> Result<FileOutcome, io::Error> {
        if let Some(select) = self.dir_state.selected() {
            if let Some(dir) = self.dirs.get(select).cloned() {
                self.chdir(&dir)?;
                return Ok(FileOutcome::Changed);
            }
        }
        Ok(FileOutcome::Unchanged)
    }

    /// Cancel the dialog.
    pub fn close_cancel(&mut self) -> FileOutcome {
        self.active = false;
        FileOutcome::Cancel
    }

    /// Choose the selected and close the dialog.
    pub fn choose_selected(&mut self) -> FileOutcome {
        if let Some(select) = self.file_state.selected() {
            if let Some(file) = self.files.get(select).cloned() {
                self.active = false;
                return FileOutcome::Ok(self.path.join(file));
            }
        }
        FileOutcome::Unchanged
    }

    pub fn screen_cursor(&self) -> Option<(u16, u16)> {
        match_focus!(
            self.path_state => {
                self.path_state.screen_cursor()
            },
            _ => None
        )
    }
}

impl FileOpenState {
    fn focus(&self) -> Focus<'_> {
        Focus::new(&[
            &self.dir_state,
            &self.file_state,
            &self.path_state,
            &self.ok_state,
            &self.cancel_state,
            &self.root_state,
        ])
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
        debug!("focus -> {:?}", focus_outcome);

        flow_ok!(self.path_state.handle(event, FocusKeys), consider focus_outcome);
        if self.path_state.is_focused() {
            flow_ok!(match event {
                ct_event!(keycode press Enter) => {
                    let path = PathBuf::from(self.path_state.value());
                    if !path.exists() || !path.is_dir() {
                        self.path_state.invalid = true;
                    } else {
                        self.path_state.invalid = false;
                        self.set_path(&PathBuf::from(self.path_state.value()))?;
                    }
                    FileOutcome::Changed
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

                    debug!("nav {:?}", nav);

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
