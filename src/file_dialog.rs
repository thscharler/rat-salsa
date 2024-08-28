//!
//! File dialog
//!

use crate::_private::NonExhaustive;
use crate::button::{Button, ButtonOutcome, ButtonState, ButtonStyle};
use crate::event::{FileOutcome, TextOutcome};
use crate::fill::Fill;
use crate::layout::{layout_dialog, layout_grid};
use crate::list::edit::{EditList, EditListState};
use crate::list::selection::RowSelection;
use crate::list::{List, ListState, ListStyle};
use crate::util::revert_style;
use directories_next::UserDirs;
#[allow(unused_imports)]
use log::debug;
use rat_event::{ct_event, flow, flow_ok, ConsumedEvent, Dialog, HandleEvent, Outcome, Regular};
use rat_focus::{match_focus, on_lost, Focus, FocusFlag, HasFocusFlag};
use rat_ftable::event::EditOutcome;
use rat_scrolled::Scroll;
use rat_text::text_input::{TextInput, TextInputState, TextInputStyle};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Flex, Layout, Rect};
use ratatui::prelude::{StatefulWidget, Style, Text, Widget};
use ratatui::widgets::{Block, ListItem, StatefulWidgetRef, WidgetRef};
use std::ffi::OsString;
use std::fmt::{Debug, Formatter};
use std::path::{Path, PathBuf};
use std::{fs, io};
use sysinfo::Disks;

/// Shows a file dialog.
///
/// It can work in Open/Save mode or a separate Directory mode.
///
#[derive(Debug, Default, Clone)]
pub struct FileDialog<'a> {
    block: Option<Block<'a>>,

    style: Style,
    list_style: Option<Style>,
    path_style: Option<Style>,
    name_style: Option<Style>,
    invalid_style: Option<Style>,
    select_style: Option<Style>,
    focus_style: Option<Style>,
    button_style: Option<ButtonStyle>,

    ok_text: &'a str,
    cancel_text: &'a str,
}

/// Combined styles for the FileDialog.
#[derive(Debug)]
pub struct FileDialogStyle {
    pub style: Style,
    pub list: Option<Style>,
    pub path: Option<Style>,
    pub name: Option<Style>,
    pub invalid: Option<Style>,
    pub select: Option<Style>,
    pub focus: Option<Style>,
    pub button: Option<ButtonStyle>,

    pub non_exhaustive: NonExhaustive,
}

/// Open/Save or Directory dialog.
#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum Mode {
    Open,
    Save,
    Dir,
}

/// State & event-handling.
#[allow(clippy::type_complexity)]
pub struct FileDialogState {
    /// Dialog is active.
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
    root_state: ListState<RowSelection>,
    dir_state: EditListState<EditDirNameState>,
    file_state: ListState<RowSelection>,
    save_name_state: TextInputState,
    new_state: ButtonState,
    cancel_state: ButtonState,
    ok_state: ButtonState,
}

pub(crate) mod event {
    use rat_event::{ConsumedEvent, Outcome};
    use std::path::PathBuf;

    /// Result for the FileDialog.
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub enum FileOutcome {
        /// The given event has not been used at all.
        Continue,
        /// The event has been recognized, but the result was nil.
        /// Further processing for this event may stop.
        Unchanged,
        /// The event has been recognized and there is some change
        /// due to it.
        /// Further processing for this event may stop.
        /// Rendering the ui is advised.
        Changed,
        /// Cancel
        Cancel,
        /// Ok
        Ok(PathBuf),
    }

    impl ConsumedEvent for FileOutcome {
        fn is_consumed(&self) -> bool {
            !matches!(self, FileOutcome::Continue)
        }
    }

    impl From<FileOutcome> for Outcome {
        fn from(value: FileOutcome) -> Self {
            match value {
                FileOutcome::Continue => Outcome::Continue,
                FileOutcome::Unchanged => Outcome::Unchanged,
                FileOutcome::Changed => Outcome::Changed,
                FileOutcome::Ok(_) => Outcome::Changed,
                FileOutcome::Cancel => Outcome::Changed,
            }
        }
    }

    impl From<Outcome> for FileOutcome {
        fn from(value: Outcome) -> Self {
            match value {
                Outcome::Continue => FileOutcome::Continue,
                Outcome::Unchanged => FileOutcome::Unchanged,
                Outcome::Changed => FileOutcome::Changed,
            }
        }
    }

    // Useful for converting most navigation/edit results.
    impl From<bool> for FileOutcome {
        fn from(value: bool) -> Self {
            if value {
                FileOutcome::Changed
            } else {
                FileOutcome::Unchanged
            }
        }
    }
}

impl Debug for FileDialogState {
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
            .field("name_state", &self.save_name_state)
            .field("cancel_state", &self.cancel_state)
            .field("ok_state", &self.ok_state)
            .finish()
    }
}

impl Default for FileDialogStyle {
    fn default() -> Self {
        FileDialogStyle {
            style: Default::default(),
            list: None,
            path: None,
            name: None,
            invalid: None,
            select: None,
            focus: None,
            button: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl Default for FileDialogState {
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
            save_name_state: Default::default(),
            new_state: Default::default(),
            cancel_state: Default::default(),
            ok_state: Default::default(),
        };
        s.use_default_roots = true;
        s.dir_state.list.set_scroll_selection(true);
        s.file_state.set_scroll_selection(true);
        s
    }
}

impl<'a> FileDialog<'a> {
    /// New dialog
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
            ok_text: "Ok",
            cancel_text: "Cancel",
        }
    }

    /// Text for the ok button.
    pub fn ok_text(mut self, txt: &'a str) -> Self {
        self.ok_text = txt;
        self
    }

    /// Text for the cancel button.
    pub fn cancel_text(mut self, txt: &'a str) -> Self {
        self.cancel_text = txt;
        self
    }

    /// Block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Base style
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Style for the lists.
    pub fn list_style(mut self, style: Style) -> Self {
        self.list_style = Some(style);
        self
    }

    /// Style for path input.
    pub fn path_style(mut self, style: Style) -> Self {
        self.path_style = Some(style);
        self
    }

    /// Style for the save name.
    pub fn name_style(mut self, style: Style) -> Self {
        self.name_style = Some(style);
        self
    }

    /// Invalid indicator.
    pub fn invalid_style(mut self, style: Style) -> Self {
        self.invalid_style = Some(style);
        self
    }

    /// Text selection.
    pub fn select_style(mut self, style: Style) -> Self {
        self.select_style = Some(style);
        self
    }

    /// Focused widget.
    pub fn focus_style(mut self, style: Style) -> Self {
        self.focus_style = Some(style);
        self
    }

    /// Button style.
    pub fn button_style(mut self, style: ButtonStyle) -> Self {
        self.button_style = Some(style);
        self
    }

    /// All styles.
    pub fn styles(mut self, styles: FileDialogStyle) -> Self {
        self.style = styles.style;
        self.list_style = styles.list;
        self.path_style = styles.path;
        self.name_style = styles.name;
        self.invalid_style = styles.invalid;
        self.select_style = styles.select;
        self.focus_style = styles.focus;
        self.button_style = styles.button;
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

    fn style_roots(&self) -> ListStyle {
        ListStyle {
            style: self.style,
            select_style: self.defaulted_select(),
            focus_style: self.defaulted_focus(),
            ..Default::default()
        }
    }

    fn style_lists(&self) -> ListStyle {
        ListStyle {
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
        if let Some(button) = self.button_style {
            button
        } else {
            ButtonStyle {
                style: self.defaulted_select().expect("style"),
                focus: self.defaulted_focus(),
                ..Default::default()
            }
        }
    }
}

#[derive(Debug, Default)]
struct EditDirName;

#[derive(Debug, Default)]
struct EditDirNameState {
    edit_dir: TextInputState,
}

impl StatefulWidgetRef for EditDirName {
    type State = EditDirNameState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        TextInput::new().render_ref(area, buf, &mut state.edit_dir);
    }
}

impl EditDirNameState {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        self.edit_dir.screen_cursor()
    }
}

impl HandleEvent<crossterm::event::Event, Regular, EditOutcome> for EditDirNameState {
    fn handle(&mut self, event: &crossterm::event::Event, qualifier: Regular) -> EditOutcome {
        match self.edit_dir.handle(event, qualifier) {
            TextOutcome::Continue => EditOutcome::Continue,
            TextOutcome::Unchanged => EditOutcome::Unchanged,
            TextOutcome::Changed => EditOutcome::Changed,
            TextOutcome::TextChanged => EditOutcome::Changed,
        }
    }
}

impl HasFocusFlag for EditDirNameState {
    fn focus(&self) -> FocusFlag {
        self.edit_dir.focus()
    }

    fn area(&self) -> Rect {
        self.edit_dir.area()
    }
}

impl<'a> StatefulWidget for FileDialog<'a> {
    type State = FileDialogState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if !state.active {
            return;
        }

        let block;
        let block = if let Some(block) = self.block.as_ref() {
            block
        } else {
            block = Block::bordered()
                .title(match state.mode {
                    Mode::Open => " Open ",
                    Mode::Save => " Save ",
                    Mode::Dir => " Directory ",
                })
                .style(self.style);
            &block
        };

        let layout = layout_dialog(
            area,
            Some(&block),
            [
                Constraint::Percentage(20),
                Constraint::Percentage(30),
                Constraint::Percentage(50),
            ],
            0,
            Flex::Center,
        );

        block.render_ref(area, buf);
        Fill::new()
            .fill_char(" ")
            .style(self.style)
            .render(layout.inner, buf);

        match state.mode {
            Mode::Open => {
                render_open(&self, layout.content, buf, state);
            }
            Mode::Save => {
                render_save(&self, layout.content, buf, state);
            }
            Mode::Dir => {
                todo!()
            }
        }

        let mut l_n = layout.buttons[1];
        l_n.width = 10;
        Button::new()
            .text(Text::from("New").alignment(Alignment::Center))
            .styles(self.style_button())
            .render(l_n, buf, &mut state.new_state);

        let l_oc = Layout::horizontal([Constraint::Length(10), Constraint::Length(10)])
            .spacing(1)
            .flex(Flex::End)
            .split(layout.buttons[2]);

        Button::new()
            .text(Text::from(self.cancel_text).alignment(Alignment::Center))
            .styles(self.style_button())
            .render(l_oc[0], buf, &mut state.cancel_state);

        Button::new()
            .text(Text::from(self.ok_text).alignment(Alignment::Center))
            .styles(self.style_button())
            .render(l_oc[1], buf, &mut state.ok_state);
    }
}

fn render_open(widget: &FileDialog<'_>, area: Rect, buf: &mut Buffer, state: &mut FileDialogState) {
    let l_grid = layout_grid::<3, 2>(
        area,
        Layout::horizontal([
            Constraint::Percentage(20),
            Constraint::Percentage(30),
            Constraint::Percentage(50),
        ]),
        Layout::new(
            Direction::Vertical,
            [Constraint::Length(1), Constraint::Fill(1)],
        ),
    );

    //
    let mut l_path = l_grid[1][0].union(l_grid[2][0]);
    l_path.width = l_path.width.saturating_sub(1);
    TextInput::new()
        .styles(widget.style_path())
        .render(l_path, buf, &mut state.path_state);

    List::default()
        .items(state.roots.iter().map(|v| {
            let s = v.0.to_string_lossy();
            ListItem::from(format!("{}", s))
        }))
        .scroll(Scroll::new())
        .styles(widget.style_roots())
        .render(l_grid[0][1], buf, &mut state.root_state);

    EditList::new(
        List::default()
            .items(state.dirs.iter().map(|v| {
                let s = v.to_string_lossy();
                ListItem::from(s)
            }))
            .scroll(Scroll::new())
            .styles(widget.style_lists()),
        EditDirName,
    )
    .render(l_grid[1][1], buf, &mut state.dir_state);

    List::default()
        .items(state.files.iter().map(|v| {
            let s = v.to_string_lossy();
            ListItem::from(s)
        }))
        .scroll(Scroll::new())
        .styles(widget.style_lists())
        .render(l_grid[2][1], buf, &mut state.file_state);
}

fn render_save(widget: &FileDialog<'_>, area: Rect, buf: &mut Buffer, state: &mut FileDialogState) {
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
                Constraint::Fill(1),
                Constraint::Length(1),
            ],
        ),
    );

    //
    let mut l_path = l_grid[1][0].union(l_grid[2][0]);
    l_path.width = l_path.width.saturating_sub(1);
    TextInput::new()
        .styles(widget.style_path())
        .render(l_path, buf, &mut state.path_state);

    List::default()
        .items(state.roots.iter().map(|v| {
            let s = v.0.to_string_lossy();
            ListItem::from(format!("{}", s))
        }))
        .scroll(Scroll::new())
        .styles(widget.style_roots())
        .render(l_grid[0][1], buf, &mut state.root_state);

    EditList::new(
        List::default()
            .items(state.dirs.iter().map(|v| {
                let s = v.to_string_lossy();
                ListItem::from(s)
            }))
            .scroll(Scroll::new())
            .styles(widget.style_lists()),
        EditDirName,
    )
    .render(l_grid[1][1], buf, &mut state.dir_state);

    List::default()
        .items(state.files.iter().map(|v| {
            let s = v.to_string_lossy();
            ListItem::from(s)
        }))
        .scroll(Scroll::new())
        .styles(widget.style_lists())
        .render(l_grid[2][1], buf, &mut state.file_state);

    TextInput::new().styles(widget.style_name()).render(
        l_grid[2][2],
        buf,
        &mut state.save_name_state,
    );
}

impl FileDialogState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a filter.
    pub fn set_filter(&mut self, filter: impl Fn(&Path) -> bool + 'static) {
        self.filter = Some(Box::new(filter));
    }

    /// Use the default set of roots.
    pub fn use_default_roots(&mut self, roots: bool) {
        self.use_default_roots = roots;
    }

    /// Add a root path.
    pub fn add_root(&mut self, name: impl AsRef<str>, path: impl Into<PathBuf>) {
        self.roots
            .push((OsString::from(name.as_ref()), path.into()))
    }

    /// Clear all roots.
    pub fn clear_roots(&mut self) {
        self.roots.clear();
    }

    /// Append the default roots.
    pub fn default_roots(&mut self, start: &Path, last: &Path) {
        if let Some(user) = UserDirs::new() {
            if last.exists() {
                self.roots.push((
                    OsString::from("Last"), //
                    last.into(),
                ));
            }
            self.roots.push((
                OsString::from("Start"), //
                start.into(),
            ));
            self.roots.push((
                OsString::from("Home"), //
                user.home_dir().to_path_buf(),
            ));
            if let Some(document_dir) = user.document_dir() {
                self.roots
                    .push((OsString::from("Documents"), document_dir.to_path_buf()));
            }
        }

        let disks = Disks::new_with_refreshed_list();
        for d in disks.list() {
            self.roots
                .push((d.name().to_os_string(), d.mount_point().to_path_buf()));
        }

        self.root_state.select(Some(0));
    }

    /// Show as open-dialog.
    pub fn open_dialog(&mut self, path: impl AsRef<Path>) -> Result<(), io::Error> {
        let path = path.as_ref();
        let old_path = self.path.clone();

        self.active = true;
        self.mode = Mode::Open;
        self.save_name = None;
        self.dirs.clear();
        self.files.clear();
        self.path = Default::default();
        if self.use_default_roots {
            self.clear_roots();
            self.default_roots(path, &old_path);
            if old_path.exists() {
                self.set_path(&old_path)?;
            } else {
                self.set_path(path)?;
            }
        } else {
            self.set_path(path)?;
        }
        self.focus().focus(&self.file_state);
        Ok(())
    }

    /// Show as save-dialog.
    pub fn save_dialog(
        &mut self,
        path: impl AsRef<Path>,
        name: impl AsRef<str>,
    ) -> Result<(), io::Error> {
        let path = path.as_ref();
        let old_path = self.path.clone();

        self.active = true;
        self.mode = Mode::Save;
        self.save_name = Some(OsString::from(name.as_ref()));
        self.dirs.clear();
        self.files.clear();
        self.path = Default::default();
        if self.use_default_roots {
            self.clear_roots();
            self.default_roots(path, &old_path);
            if old_path.exists() {
                self.set_path(&old_path)?;
            } else {
                self.set_path(path)?;
            }
        } else {
            self.set_path(path)?;
        }
        self.focus().focus(&self.save_name_state);
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

    // change the path
    fn set_path(&mut self, path: &Path) -> Result<FileOutcome, io::Error> {
        let old = self.path.clone();
        let path = path.to_path_buf();

        if old != path {
            let mut dirs = Vec::new();
            let mut files = Vec::new();

            if self.find_parent(&path).is_some() {
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

            self.path_state.set_text(self.path.to_string_lossy());
            if self.path_state.inner.width != 0 {
                // only works when this has been rendered once. todo:
                self.path_state.move_to_line_end(false);
            }

            if !self.dirs.is_empty() {
                self.dir_state.list.select(Some(0));
            } else {
                self.dir_state.list.select(None);
            }
            self.dir_state.list.set_offset(0);
            if !self.files.is_empty() {
                self.file_state.select(Some(0));
                if let Some(name) = &self.save_name {
                    self.save_name_state.set_text(name.to_string_lossy());
                } else {
                    self.save_name_state
                        .set_text(self.files[0].to_string_lossy());
                }
            } else {
                self.file_state.select(None);
                if let Some(name) = &self.save_name {
                    self.save_name_state.set_text(name.to_string_lossy());
                } else {
                    self.save_name_state.set_text("");
                }
            }
            self.file_state.set_offset(0);

            Ok(FileOutcome::Changed)
        } else {
            Ok(FileOutcome::Unchanged)
        }
    }

    fn use_path_input(&mut self) -> Result<FileOutcome, io::Error> {
        let path = PathBuf::from(self.path_state.text());
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
        if let Some(select) = self.dir_state.list.selected() {
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
                self.save_name_state.set_text(name);
                return Ok(FileOutcome::Changed);
            }
        }
        Ok(FileOutcome::Unchanged)
    }

    /// Start creating a directory.
    fn start_edit_dir(&mut self) -> FileOutcome {
        self.dirs.push(OsString::from(""));
        self.dir_state.list.items_added(self.dirs.len(), 1);
        self.dir_state.list.move_to(self.dirs.len() - 1);
        let edit = EditDirNameState::default();
        edit.focus().set(true);
        self.dir_state.edit = Some(edit);
        FileOutcome::Changed
    }

    fn cancel_edit_dir(&mut self) -> FileOutcome {
        self.dirs.remove(self.dirs.len() - 1);
        self.dir_state.edit = None;
        FileOutcome::Changed
    }

    fn commit_edit_dir(&mut self) -> Result<FileOutcome, io::Error> {
        if let Some(edit) = &mut self.dir_state.edit {
            let name = edit.edit_dir.text().trim();
            let path = self.path.join(name);
            if fs::create_dir(&path).is_err() {
                edit.edit_dir.invalid = true;
                Ok(FileOutcome::Changed)
            } else {
                self.dir_state.edit = None;
                self.focus().focus_no_lost(&self.save_name_state);
                self.set_path(&path)
            }
        } else {
            Ok(FileOutcome::Unchanged)
        }
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
            let path = self.path.join(self.save_name_state.text().trim());
            self.active = false;
            return FileOutcome::Ok(path);
        }
        FileOutcome::Unchanged
    }

    pub fn screen_cursor(&self) -> Option<(u16, u16)> {
        if self.active {
            match_focus!(
                self.path_state => {
                    self.path_state.screen_cursor()
                },
                self.save_name_state => {
                    self.save_name_state.screen_cursor()
                },
                self.dir_state.list => {
                    if let Some(edit) = &self.dir_state.edit {
                          edit.screen_cursor()
                    } else {
                        None
                    }
                },
                _ => None
            )
        } else {
            None
        }
    }
}

impl FileDialogState {
    fn focus(&self) -> Focus {
        let mut f = Focus::default();
        f.add(&self.dir_state);
        f.add(&self.file_state);
        if self.mode == Mode::Save {
            f.add(&self.save_name_state);
        }
        f.add(&self.ok_state);
        f.add(&self.cancel_state);
        f.add(&self.new_state);
        f.add(&self.root_state);
        f.add(&self.path_state);
        f
    }
}

impl HandleEvent<crossterm::event::Event, Dialog, Result<FileOutcome, io::Error>>
    for FileDialogState
{
    fn handle(
        &mut self,
        event: &crossterm::event::Event,
        _qualifier: Dialog,
    ) -> Result<FileOutcome, io::Error> {
        if !self.active {
            return Ok(FileOutcome::Continue);
        }
        if matches!(event, ct_event!(mouse moved)) {
            return Ok(FileOutcome::Continue);
        }

        let f: FileOutcome = self.focus().handle(event, Regular).into();
        let f = f.and_try(|| {
            handle_path(self, event)?
                .or_else_try(|| {
                    if self.mode == Mode::Save {
                        handle_name(self, event)
                    } else {
                        Ok(FileOutcome::Continue)
                    }
                })?
                .or_else_try(|| handle_files(self, event))?
                .or_else_try(|| handle_dirs(self, event))?
                .or_else_try(|| handle_roots(self, event))?
                .or_else_try(|| handle_new(self, event))?
                .or_else_try(|| handle_cancel(self, event))?
                .or_else_try(|| handle_ok(self, event))
        });
        f
    }
}

fn handle_new(
    state: &mut FileDialogState,
    event: &crossterm::event::Event,
) -> Result<FileOutcome, io::Error> {
    flow_ok!(match state.new_state.handle(event, Regular) {
        ButtonOutcome::Pressed => {
            state.start_edit_dir()
        }
        r => Outcome::from(r).into(),
    });
    flow_ok!(match event {
        ct_event!(key press CONTROL-'n') => {
            state.start_edit_dir()
        }
        _ => FileOutcome::Continue,
    });
    Ok(FileOutcome::Continue)
}

fn handle_ok(
    state: &mut FileDialogState,
    event: &crossterm::event::Event,
) -> Result<FileOutcome, io::Error> {
    flow_ok!(match state.ok_state.handle(event, Regular) {
        ButtonOutcome::Pressed => state.choose_selected(),
        r => Outcome::from(r).into(),
    });
    Ok(FileOutcome::Continue)
}

fn handle_cancel(
    state: &mut FileDialogState,
    event: &crossterm::event::Event,
) -> Result<FileOutcome, io::Error> {
    flow_ok!(match state.cancel_state.handle(event, Regular) {
        ButtonOutcome::Pressed => {
            state.close_cancel()
        }
        r => Outcome::from(r).into(),
    });
    flow_ok!(match event {
        ct_event!(keycode press Esc) => {
            state.close_cancel()
        }
        _ => FileOutcome::Continue,
    });
    Ok(FileOutcome::Continue)
}

fn handle_name(
    state: &mut FileDialogState,
    event: &crossterm::event::Event,
) -> Result<FileOutcome, io::Error> {
    flow_ok!(Outcome::from(state.save_name_state.handle(event, Regular)));
    if state.save_name_state.is_focused() {
        flow_ok!(match event {
            ct_event!(keycode press Enter) => {
                state.choose_selected()
            }
            _ => FileOutcome::Continue,
        });
    }
    Ok(FileOutcome::Continue)
}

fn handle_path(
    state: &mut FileDialogState,
    event: &crossterm::event::Event,
) -> Result<FileOutcome, io::Error> {
    flow_ok!(Outcome::from(state.path_state.handle(event, Regular)));
    if state.path_state.is_focused() {
        flow_ok!(match event {
            ct_event!(keycode press Enter) => {
                state.use_path_input()?;
                state.focus().focus_no_lost(&state.dir_state.list);
                FileOutcome::Changed
            }
            _ => FileOutcome::Continue,
        });
    }
    on_lost!(
        state.path_state => {
            state.use_path_input()?
        }
    );
    Ok(FileOutcome::Continue)
}

fn handle_roots(
    state: &mut FileDialogState,
    event: &crossterm::event::Event,
) -> Result<FileOutcome, io::Error> {
    flow_ok!(log root_state: match state.root_state.handle(event, Regular) {
        Outcome::Changed => {
            state.chroot_selected()?
        }
        r => r.into(),
    });
    Ok(FileOutcome::Continue)
}

fn handle_dirs(
    state: &mut FileDialogState,
    event: &crossterm::event::Event,
) -> Result<FileOutcome, io::Error> {
    if state.dir_state.edit.is_none() {
        if state.dir_state.list.is_focused() {
            flow_ok!(match event {
                ct_event!(mouse any for m)
                    if state
                        .dir_state
                        .list
                        .mouse
                        .doubleclick(state.dir_state.list.inner, m) =>
                {
                    state.chdir_selected()?
                }
                ct_event!(keycode press Enter) => {
                    state.chdir_selected()?
                }
                _ => FileOutcome::Continue,
            });
            flow_ok!(handle_nav(&mut state.dir_state.list, &state.dirs, event));
        }
    }
    flow_ok!(match state.dir_state.handle(event, Regular) {
        EditOutcome::Cancel => {
            state.cancel_edit_dir()
        }
        EditOutcome::Commit | EditOutcome::CommitAndAppend | EditOutcome::CommitAndEdit => {
            state.commit_edit_dir()?
        }
        r => {
            Outcome::from(r).into()
        }
    });
    Ok(FileOutcome::Continue)
}

fn handle_files(
    state: &mut FileDialogState,
    event: &crossterm::event::Event,
) -> Result<FileOutcome, io::Error> {
    if state.file_state.is_focused() {
        flow_ok!(match event {
            ct_event!(mouse any for m)
                if state
                    .file_state
                    .mouse
                    .doubleclick(state.file_state.inner, m) =>
            {
                state.choose_selected()
            }
            ct_event!(keycode press Enter) => {
                state.choose_selected()
            }
            _ => FileOutcome::Continue,
        });
        flow_ok!(
            match handle_nav(&mut state.file_state, &state.files, event) {
                FileOutcome::Changed => {
                    if state.mode == Mode::Save {
                        state.name_selected()?
                    } else {
                        FileOutcome::Changed
                    }
                }
                r => r,
            }
        );
    }
    flow_ok!(match state.file_state.handle(event, Regular).into() {
        FileOutcome::Changed => {
            if state.mode == Mode::Save {
                state.name_selected()?
            } else {
                FileOutcome::Changed
            }
        }
        r => r,
    });
    Ok(FileOutcome::Continue)
}

fn handle_nav(
    list: &mut ListState<RowSelection>,
    nav: &[OsString],
    event: &crossterm::event::Event,
) -> FileOutcome {
    flow!(match event {
        ct_event!(key press c) => {
            let next = find_next_by_key(*c, list.selected().unwrap_or(0), nav);
            if let Some(next) = next {
                list.move_to(next).into()
            } else {
                FileOutcome::Unchanged
            }
        }
        _ => FileOutcome::Continue,
    });
    FileOutcome::Continue
}

fn find_next_by_key(c: char, start: usize, names: &[OsString]) -> Option<usize> {
    let c = c.to_lowercase().next();

    let mut idx = start;
    let mut selected = None;
    loop {
        idx += 1;
        if idx >= names.len() {
            idx = 0;
        }
        if idx == start {
            break;
        }

        let nav = names[idx]
            .to_string_lossy()
            .chars()
            .next()
            .and_then(|v| v.to_lowercase().next());
        if c == nav {
            selected = Some(idx);
            break;
        }
    }

    selected
}
