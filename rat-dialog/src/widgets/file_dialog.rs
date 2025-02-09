use crate::{DialogState, RenderContext};
use rat_salsa::{AppContext, AppState, AppWidget, Control};
use rat_widget::event::{Dialog, FileOutcome, HandleEvent};
use rat_widget::file_dialog::FileDialogStyle;
use rat_widget::layout::layout_middle;
use rat_widget::text::HasScreenCursor;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Constraint;
use ratatui::widgets::StatefulWidget;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

pub struct FileDialog {
    widget: rat_widget::file_dialog::FileDialog<'static>,
}

pub struct FileDialogState<Global, Event, Error> {
    state: rat_widget::file_dialog::FileDialogState,
    tr: Box<dyn Fn(FileOutcome) -> Control<Event> + 'static>,
    phantom: PhantomData<(Global, Event, Error)>,
}

impl FileDialog {
    pub fn new() -> Self {
        Self {
            widget: Default::default(),
        }
    }

    pub fn styles(mut self, styles: FileDialogStyle) -> Self {
        self.widget = self.widget.styles(styles);
        self
    }
}

impl<Global, Event, Error> AppWidget<Global, Event, Error> for FileDialog
where
    for<'a> &'a crossterm::event::Event: TryFrom<&'a Event>,
    Global: 'static,
    Event: Send + 'static,
    Error: Send + 'static + From<std::io::Error>,
{
    type State = dyn DialogState<Global, Event, Error>;

    fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut Self::State,
        ctx: &mut RenderContext<'_, Global>,
    ) -> Result<(), Error> {
        let state = state
            .downcast_mut::<FileDialogState<Global, Event, Error>>()
            .expect("state");

        let dlg_area = layout_middle(
            area,
            Constraint::Percentage(19),
            Constraint::Percentage(19),
            Constraint::Length(2),
            Constraint::Length(2),
        );

        self.widget.clone().render(dlg_area, buf, &mut state.state);

        ctx.set_screen_cursor(state.state.screen_cursor());

        Ok(())
    }
}

impl<Global, Event, Error> FileDialogState<Global, Event, Error>
where
    for<'a> &'a crossterm::event::Event: TryFrom<&'a Event>,
    Global: 'static,
    Event: Send + 'static,
    Error: Send + From<std::io::Error> + 'static,
{
    pub fn new() -> Self {
        Self {
            state: rat_widget::file_dialog::FileDialogState::new(),
            tr: Box::new(|f| Control::from(f)),
            phantom: Default::default(),
        }
    }

    pub fn open_dialog(mut self, path: impl AsRef<Path>) -> Result<Self, Error> {
        self.state.open_dialog(path)?;
        Ok(self)
    }

    pub fn save_dialog(
        mut self,
        path: impl AsRef<Path>,
        name: impl AsRef<str>,
    ) -> Result<Self, Error> {
        self.state.save_dialog(path, name)?;
        Ok(self)
    }

    pub fn save_dialog_ext(
        mut self,
        path: impl AsRef<Path>,
        name: impl AsRef<str>,
        ext: impl AsRef<str>,
    ) -> Result<Self, Error> {
        self.state.save_dialog_ext(path, name, ext)?;
        Ok(self)
    }

    pub fn map_outcome(mut self, m: impl Fn(FileOutcome) -> Control<Event> + 'static) -> Self {
        self.tr = Box::new(m);
        self
    }

    pub fn directory_dialog(mut self, path: impl AsRef<Path>) -> Result<Self, Error> {
        self.state.directory_dialog(path)?;
        Ok(self)
    }

    /// Set a filter.
    pub fn set_filter(mut self, filter: impl Fn(&Path) -> bool + 'static) -> Self {
        self.state.set_filter(filter);
        self
    }

    /// Use the default set of roots.
    pub fn use_default_roots(mut self, roots: bool) -> Self {
        self.state.use_default_roots(roots);
        self
    }

    /// Add a root path.
    pub fn add_root(mut self, name: impl AsRef<str>, path: impl Into<PathBuf>) -> Self {
        self.state.add_root(name, path);
        self
    }

    /// Clear all roots.
    pub fn clear_roots(mut self) -> Self {
        self.state.clear_roots();
        self
    }

    /// Append the default roots.
    pub fn default_roots(mut self, start: &Path, last: &Path) -> Self {
        self.state.default_roots(start, last);
        self
    }
}

impl<Global, Event, Error> AppState<Global, Event, Error> for FileDialogState<Global, Event, Error>
where
    for<'a> &'a crossterm::event::Event: TryFrom<&'a Event>,
    Global: 'static,
    Event: Send + 'static,
    Error: Send + 'static + From<std::io::Error>,
{
    fn event(
        &mut self,
        event: &Event,
        _ctx: &mut AppContext<'_, Global, Event, Error>,
    ) -> Result<Control<Event>, Error> {
        let r = if let Ok(event) = event.try_into() {
            let r = self.state.handle(event, Dialog)?.into();
            (self.tr)(r)
        } else {
            Control::Continue
        };

        Ok(r)
    }
}

impl<Global, Event, Error> DialogState<Global, Event, Error>
    for FileDialogState<Global, Event, Error>
where
    for<'a> &'a crossterm::event::Event: TryFrom<&'a Event>,
    Global: 'static,
    Event: Send + 'static,
    Error: Send + 'static + From<std::io::Error>,
{
    fn active(&self) -> bool {
        self.state.active()
    }
}
