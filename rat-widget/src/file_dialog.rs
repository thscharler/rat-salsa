//!
//! File dialog.
//!

use crate::_private::NonExhaustive;
use crate::button::{Button, ButtonState, ButtonStyle};
use crate::event::{ButtonOutcome, FileOutcome, TextOutcome};
use crate::layout::{layout_dialog, layout_grid, DialogItem};
use crate::list::edit::{EditList, EditListState};
use crate::list::selection::RowSelection;
use crate::list::{List, ListState, ListStyle};
use crate::util::{block_padding2, reset_buf_area};
#[cfg(feature = "user_directories")]
use dirs::{document_dir, home_dir};
use rat_event::{
    ct_event, flow, try_flow, ConsumedEvent, Dialog, HandleEvent, MouseOnly, Outcome, Regular,
};
use rat_focus::{on_lost, Focus, FocusBuilder, FocusFlag, HasFocus};
use rat_ftable::event::EditOutcome;
use rat_scrolled::Scroll;
use rat_text::text_input::{TextInput, TextInputState};
use rat_text::{HasScreenCursor, TextStyle};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Flex, Layout, Rect};
use ratatui::prelude::{StatefulWidget, Style, Text, Widget};
use ratatui::widgets::{Block, ListItem};
use std::cmp::max;
use std::ffi::OsString;
use std::fmt::{Debug, Formatter};
use std::path::{Path, PathBuf};
use std::{fs, io};
#[cfg(feature = "user_directories")]
use sysinfo::Disks;

/// Shows a file dialog.
///
/// * Display modes
///     * Open
///     * Save
///     * Directory
///
/// * Define your roots or let them provide by
///     [dirs](https://docs.rs/dirs/6.0.0/dirs/) and
///     [sysinfo](https://docs.rs/sysinfo/0.33.1/sysinfo/)
///   You need the feature "user_directories" for the latter.
///
///   * Standard roots are
///     * Last - The directory choosen the last time the dialog was opened.
///     * Start - The start directory provided by the application.
///
/// * Create new directories.
///
/// * Quick jump between lists with F1..F5.
///
#[derive(Debug, Clone)]
pub struct FileDialog<'a> {
    block: Option<Block<'a>>,

    style: Style,
    list_style: Option<ListStyle>,
    roots_style: Option<ListStyle>,
    text_style: Option<TextStyle>,
    button_style: Option<ButtonStyle>,
    ok_text: &'a str,
    cancel_text: &'a str,
}

/// Combined styles for the FileDialog.
#[derive(Debug)]
pub struct FileDialogStyle {
    pub style: Style,
    /// Lists
    pub list: Option<ListStyle>,
    /// FS roots
    pub roots: Option<ListStyle>,
    /// Text fields
    pub text: Option<TextStyle>,
    /// Buttons.
    pub button: Option<ButtonStyle>,
    /// Outer border.
    pub block: Option<Block<'static>>,

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
    save_ext: Option<OsString>,
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
            roots: None,
            button: None,
            block: None,
            non_exhaustive: NonExhaustive,
            text: None,
        }
    }
}

impl Default for FileDialogState {
    fn default() -> Self {
        let mut s = Self {
            active: false,
            mode: Mode::Open,
            path: Default::default(),
            save_name: Default::default(),
            save_ext: Default::default(),
            dirs: Default::default(),
            filter: Default::default(),
            files: Default::default(),
            use_default_roots: true,
            roots: Default::default(),
            path_state: Default::default(),
            root_state: Default::default(),
            dir_state: Default::default(),
            file_state: Default::default(),
            save_name_state: Default::default(),
            new_state: Default::default(),
            cancel_state: Default::default(),
            ok_state: Default::default(),
        };
        s.dir_state.list.set_scroll_selection(true);
        s.file_state.set_scroll_selection(true);
        s
    }
}

impl<'a> Default for FileDialog<'a> {
    fn default() -> Self {
        Self {
            block: Default::default(),
            style: Default::default(),
            list_style: Default::default(),
            roots_style: Default::default(),
            text_style: Default::default(),
            button_style: Default::default(),
            ok_text: "Ok",
            cancel_text: "Cancel",
        }
    }
}

impl<'a> FileDialog<'a> {
    /// New dialog
    pub fn new() -> Self {
        Self::default()
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
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Base style
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Style for the lists.
    pub fn list_style(mut self, style: ListStyle) -> Self {
        self.list_style = Some(style);
        self
    }

    /// Filesystem roots style.
    pub fn roots_style(mut self, style: ListStyle) -> Self {
        self.roots_style = Some(style);
        self
    }

    /// Textfield style.
    pub fn text_style(mut self, style: TextStyle) -> Self {
        self.text_style = Some(style);
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
        if styles.list.is_some() {
            self.list_style = styles.list;
        }
        if styles.roots.is_some() {
            self.roots_style = styles.roots;
        }
        if styles.text.is_some() {
            self.text_style = styles.text;
        }
        if styles.button.is_some() {
            self.button_style = styles.button;
        }
        if styles.block.is_some() {
            self.block = styles.block;
        }
        self.block = self.block.map(|v| v.style(self.style));
        self
    }
}

#[derive(Debug, Default)]
struct EditDirName<'a> {
    edit_dir: TextInput<'a>,
}

#[derive(Debug, Default)]
struct EditDirNameState {
    edit_dir: TextInputState,
}

impl StatefulWidget for EditDirName<'_> {
    type State = EditDirNameState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.edit_dir.render(area, buf, &mut state.edit_dir);
    }
}

impl HasScreenCursor for EditDirNameState {
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

impl HandleEvent<crossterm::event::Event, MouseOnly, EditOutcome> for EditDirNameState {
    fn handle(&mut self, event: &crossterm::event::Event, qualifier: MouseOnly) -> EditOutcome {
        match self.edit_dir.handle(event, qualifier) {
            TextOutcome::Continue => EditOutcome::Continue,
            TextOutcome::Unchanged => EditOutcome::Unchanged,
            TextOutcome::Changed => EditOutcome::Changed,
            TextOutcome::TextChanged => EditOutcome::Changed,
        }
    }
}

impl HasFocus for EditDirNameState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    fn focus(&self) -> FocusFlag {
        self.edit_dir.focus()
    }

    fn area(&self) -> Rect {
        self.edit_dir.area()
    }
}

impl StatefulWidget for FileDialog<'_> {
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
            block_padding2(block),
            [
                Constraint::Percentage(20),
                Constraint::Percentage(30),
                Constraint::Percentage(50),
            ],
            0,
            Flex::Center,
        );

        reset_buf_area(layout.area(), buf);
        block.render(area, buf);

        match state.mode {
            Mode::Open => {
                render_open(&self, layout.widget_for(DialogItem::Content), buf, state);
            }
            Mode::Save => {
                render_save(&self, layout.widget_for(DialogItem::Content), buf, state);
            }
            Mode::Dir => {
                render_open_dir(&self, layout.widget_for(DialogItem::Content), buf, state);
            }
        }

        let mut l_n = layout.widget_for(DialogItem::Button(1));
        l_n.width = 10;
        Button::new(Text::from("New").alignment(Alignment::Center))
            .styles_opt(self.button_style.clone())
            .render(l_n, buf, &mut state.new_state);

        let l_oc = Layout::horizontal([Constraint::Length(10), Constraint::Length(10)])
            .spacing(1)
            .flex(Flex::End)
            .split(layout.widget_for(DialogItem::Button(2)));

        Button::new(Text::from(self.cancel_text).alignment(Alignment::Center))
            .styles_opt(self.button_style.clone())
            .render(l_oc[0], buf, &mut state.cancel_state);

        Button::new(Text::from(self.ok_text).alignment(Alignment::Center))
            .styles_opt(self.button_style.clone())
            .render(l_oc[1], buf, &mut state.ok_state);
    }
}

fn render_open_dir(
    widget: &FileDialog<'_>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut FileDialogState,
) {
    let l_grid = layout_grid::<2, 2>(
        area,
        Layout::horizontal([
            Constraint::Percentage(20), //
            Constraint::Percentage(80),
        ]),
        Layout::vertical([
            Constraint::Length(1), //
            Constraint::Fill(1),
        ]),
    );

    //
    let mut l_path = l_grid.widget_for((1, 0));
    l_path.width = l_path.width.saturating_sub(1);
    TextInput::new()
        .styles_opt(widget.text_style.clone())
        .render(l_path, buf, &mut state.path_state);

    List::default()
        .items(state.roots.iter().map(|v| {
            let s = v.0.to_string_lossy();
            ListItem::from(format!("{}", s))
        }))
        .scroll(Scroll::new())
        .styles_opt(widget.roots_style.clone())
        .render(l_grid.widget_for((0, 1)), buf, &mut state.root_state);

    EditList::new(
        List::default()
            .items(state.dirs.iter().map(|v| {
                let s = v.to_string_lossy();
                ListItem::from(s)
            }))
            .scroll(Scroll::new())
            .styles_opt(widget.list_style.clone()),
        EditDirName {
            edit_dir: TextInput::new().styles_opt(widget.text_style.clone()),
        },
    )
    .render(l_grid.widget_for((1, 1)), buf, &mut state.dir_state);
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
    let mut l_path = l_grid.widget_for((1, 0)).union(l_grid.widget_for((2, 0)));
    l_path.width = l_path.width.saturating_sub(1);
    TextInput::new()
        .styles_opt(widget.text_style.clone())
        .render(l_path, buf, &mut state.path_state);

    List::default()
        .items(state.roots.iter().map(|v| {
            let s = v.0.to_string_lossy();
            ListItem::from(format!("{}", s))
        }))
        .scroll(Scroll::new())
        .styles_opt(widget.roots_style.clone())
        .render(l_grid.widget_for((0, 1)), buf, &mut state.root_state);

    EditList::new(
        List::default()
            .items(state.dirs.iter().map(|v| {
                let s = v.to_string_lossy();
                ListItem::from(s)
            }))
            .scroll(Scroll::new())
            .styles_opt(widget.list_style.clone()),
        EditDirName {
            edit_dir: TextInput::new().styles_opt(widget.text_style.clone()),
        },
    )
    .render(l_grid.widget_for((1, 1)), buf, &mut state.dir_state);

    List::default()
        .items(state.files.iter().map(|v| {
            let s = v.to_string_lossy();
            ListItem::from(s)
        }))
        .scroll(Scroll::new())
        .styles_opt(widget.list_style.clone())
        .render(l_grid.widget_for((2, 1)), buf, &mut state.file_state);
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
    let mut l_path = l_grid.widget_for((1, 0)).union(l_grid.widget_for((2, 0)));
    l_path.width = l_path.width.saturating_sub(1);
    TextInput::new()
        .styles_opt(widget.text_style.clone())
        .render(l_path, buf, &mut state.path_state);

    List::default()
        .items(state.roots.iter().map(|v| {
            let s = v.0.to_string_lossy();
            ListItem::from(format!("{}", s))
        }))
        .scroll(Scroll::new())
        .styles_opt(widget.roots_style.clone())
        .render(l_grid.widget_for((0, 1)), buf, &mut state.root_state);

    EditList::new(
        List::default()
            .items(state.dirs.iter().map(|v| {
                let s = v.to_string_lossy();
                ListItem::from(s)
            }))
            .scroll(Scroll::new())
            .styles_opt(widget.list_style.clone()),
        EditDirName {
            edit_dir: TextInput::new().styles_opt(widget.text_style.clone()),
        },
    )
    .render(l_grid.widget_for((1, 1)), buf, &mut state.dir_state);

    List::default()
        .items(state.files.iter().map(|v| {
            let s = v.to_string_lossy();
            ListItem::from(s)
        }))
        .scroll(Scroll::new())
        .styles_opt(widget.list_style.clone())
        .render(l_grid.widget_for((2, 1)), buf, &mut state.file_state);

    TextInput::new()
        .styles_opt(widget.text_style.clone())
        .render(l_grid.widget_for((2, 2)), buf, &mut state.save_name_state);
}

impl FileDialogState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active(&self) -> bool {
        self.active
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

        #[cfg(feature = "user_directories")]
        {
            if let Some(home) = home_dir() {
                self.roots.push((OsString::from("Home"), home));
            }
            if let Some(documents) = document_dir() {
                self.roots.push((OsString::from("Documents"), documents));
            }
        }

        #[cfg(feature = "user_directories")]
        {
            let disks = Disks::new_with_refreshed_list();
            for d in disks.list() {
                self.roots
                    .push((d.name().to_os_string(), d.mount_point().to_path_buf()));
            }
        }

        self.root_state.select(Some(0));
    }

    /// Show as directory-dialog.
    pub fn directory_dialog(&mut self, path: impl AsRef<Path>) -> Result<(), io::Error> {
        let path = path.as_ref();
        let old_path = self.path.clone();

        self.active = true;
        self.mode = Mode::Dir;
        self.save_name = None;
        self.save_ext = None;
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
        self.focus().focus(&self.dir_state);
        Ok(())
    }

    /// Show as open-dialog.
    pub fn open_dialog(&mut self, path: impl AsRef<Path>) -> Result<(), io::Error> {
        let path = path.as_ref();
        let old_path = self.path.clone();

        self.active = true;
        self.mode = Mode::Open;
        self.save_name = None;
        self.save_ext = None;
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
        self.save_dialog_ext(path, name, "")
    }

    /// Show as save-dialog.
    pub fn save_dialog_ext(
        &mut self,
        path: impl AsRef<Path>,
        name: impl AsRef<str>,
        ext: impl AsRef<str>,
    ) -> Result<(), io::Error> {
        let path = path.as_ref();
        let old_path = self.path.clone();

        self.active = true;
        self.mode = Mode::Save;
        self.save_name = Some(OsString::from(name.as_ref()));
        self.save_ext = Some(OsString::from(ext.as_ref()));
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

            self.dir_state.cancel();
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

    /// Set the selected file to the new name field.
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
        if !self.dir_state.is_editing() {
            self.focus().focus(&self.dir_state);

            self.dirs.push(OsString::from(""));
            self.dir_state.editor.edit_dir.set_text("");
            self.dir_state.edit_new(self.dirs.len() - 1);

            FileOutcome::Changed
        } else {
            FileOutcome::Continue
        }
    }

    fn cancel_edit_dir(&mut self) -> FileOutcome {
        if self.dir_state.is_editing() {
            self.dir_state.cancel();
            self.dirs.remove(self.dirs.len() - 1);
            FileOutcome::Changed
        } else {
            FileOutcome::Continue
        }
    }

    fn commit_edit_dir(&mut self) -> Result<FileOutcome, io::Error> {
        if self.dir_state.is_editing() {
            let name = self.dir_state.editor.edit_dir.text().trim();
            let path = self.path.join(name);
            if fs::create_dir(&path).is_err() {
                self.dir_state.editor.edit_dir.invalid = true;
                Ok(FileOutcome::Changed)
            } else {
                self.dir_state.commit();
                if self.mode == Mode::Save {
                    self.focus().focus_no_lost(&self.save_name_state);
                }
                self.set_path(&path)
            }
        } else {
            Ok(FileOutcome::Unchanged)
        }
    }

    /// Cancel the dialog.
    fn close_cancel(&mut self) -> FileOutcome {
        self.active = false;
        self.dir_state.cancel();
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
            let mut path = self.path.join(self.save_name_state.text().trim());
            if let Some(ext) = &self.save_ext {
                if !ext.is_empty() {
                    path.set_extension(ext);
                }
            }
            self.active = false;
            return FileOutcome::Ok(path);
        } else if self.mode == Mode::Dir {
            if let Some(select) = self.dir_state.list.selected() {
                if let Some(dir) = self.dirs.get(select).cloned() {
                    self.active = false;
                    if dir != ".." {
                        return FileOutcome::Ok(self.path.join(dir));
                    } else {
                        return FileOutcome::Ok(self.path.clone());
                    }
                }
            }
        }
        FileOutcome::Unchanged
    }
}

impl HasScreenCursor for FileDialogState {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        if self.active {
            self.path_state
                .screen_cursor()
                .or_else(|| self.save_name_state.screen_cursor())
                .or_else(|| self.dir_state.screen_cursor())
        } else {
            None
        }
    }
}

impl FileDialogState {
    fn focus(&self) -> Focus {
        let mut fb = FocusBuilder::default();
        fb.widget(&self.dir_state);
        if self.mode == Mode::Save || self.mode == Mode::Open {
            fb.widget(&self.file_state);
        }
        if self.mode == Mode::Save {
            fb.widget(&self.save_name_state);
        }
        fb.widget(&self.ok_state);
        fb.widget(&self.cancel_state);
        fb.widget(&self.new_state);
        fb.widget(&self.root_state);
        fb.widget(&self.path_state);
        fb.build()
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

        let mut focus = self.focus();

        let mut f: FileOutcome = focus.handle(event, Regular).into();
        let mut r = FileOutcome::Continue;

        f = f.or_else(|| match event {
            ct_event!(keycode press F(1)) => {
                if !self.root_state.is_focused() {
                    focus.focus(&self.root_state);
                    FileOutcome::Changed
                } else {
                    FileOutcome::Continue
                }
            }
            ct_event!(keycode press F(2)) => {
                if !self.dir_state.is_focused() {
                    focus.focus(&self.dir_state);
                    FileOutcome::Changed
                } else {
                    FileOutcome::Continue
                }
            }
            ct_event!(keycode press F(3)) => {
                if !self.file_state.is_focused() {
                    focus.focus(&self.file_state);
                    FileOutcome::Changed
                } else {
                    FileOutcome::Continue
                }
            }
            ct_event!(keycode press F(4)) => {
                if !self.path_state.is_focused() {
                    focus.focus(&self.path_state);
                    FileOutcome::Changed
                } else {
                    FileOutcome::Continue
                }
            }
            ct_event!(keycode press F(5)) => {
                if !self.save_name_state.is_focused() {
                    focus.focus(&self.save_name_state);
                    FileOutcome::Changed
                } else {
                    FileOutcome::Continue
                }
            }
            _ => FileOutcome::Continue,
        });

        r = r.or_else_try(|| {
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
        })?;

        Ok(max(max(f, r), FileOutcome::Unchanged))
    }
}

fn handle_new(
    state: &mut FileDialogState,
    event: &crossterm::event::Event,
) -> Result<FileOutcome, io::Error> {
    try_flow!(match state.new_state.handle(event, Regular) {
        ButtonOutcome::Pressed => {
            state.start_edit_dir()
        }
        r => Outcome::from(r).into(),
    });
    try_flow!(match event {
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
    try_flow!(match state.ok_state.handle(event, Regular) {
        ButtonOutcome::Pressed => state.choose_selected(),
        r => Outcome::from(r).into(),
    });
    Ok(FileOutcome::Continue)
}

fn handle_cancel(
    state: &mut FileDialogState,
    event: &crossterm::event::Event,
) -> Result<FileOutcome, io::Error> {
    try_flow!(match state.cancel_state.handle(event, Regular) {
        ButtonOutcome::Pressed => {
            state.close_cancel()
        }
        r => Outcome::from(r).into(),
    });
    try_flow!(match event {
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
    try_flow!(Outcome::from(state.save_name_state.handle(event, Regular)));
    if state.save_name_state.is_focused() {
        try_flow!(match event {
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
    try_flow!(Outcome::from(state.path_state.handle(event, Regular)));
    if state.path_state.is_focused() {
        try_flow!(match event {
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
    try_flow!(match state.root_state.handle(event, Regular) {
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
    // capture F2. starts edit/selects dir otherwise.
    if matches!(event, ct_event!(keycode press F(2))) {
        return Ok(FileOutcome::Continue);
    }

    try_flow!(match state.dir_state.handle(event, Regular) {
        EditOutcome::Edit => {
            state.chdir_selected()?
        }
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
    if state.dir_state.list.is_focused() {
        try_flow!(handle_nav(&mut state.dir_state.list, &state.dirs, event));
    }
    Ok(FileOutcome::Continue)
}

fn handle_files(
    state: &mut FileDialogState,
    event: &crossterm::event::Event,
) -> Result<FileOutcome, io::Error> {
    if state.file_state.is_focused() {
        try_flow!(match event {
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
        try_flow!(
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
    try_flow!(match state.file_state.handle(event, Regular).into() {
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

#[allow(clippy::question_mark)]
fn find_next_by_key(c: char, start: usize, names: &[OsString]) -> Option<usize> {
    let Some(c) = c.to_lowercase().next() else {
        return None;
    };

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

        let nav = names[idx].to_string_lossy();

        let initials = nav
            .split([' ', '_', '-'])
            .flat_map(|v| v.chars().next())
            .flat_map(|c| c.to_lowercase().next())
            .collect::<Vec<_>>();
        if initials.contains(&c) {
            selected = Some(idx);
            break;
        }
    }

    selected
}
