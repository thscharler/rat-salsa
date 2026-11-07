use crate::_private::NonExhaustive;
use crate::core::TableCoreState;
use crate::derive::derive_table_widget_state;
use crate::edit::{Mode, TableEditorState};
use crate::selection::row::RowSelection;
use rat_cursor::HasScreenCursor;
use rat_event::crossterm::event::Event;
use rat_event::util::MouseFlags;
use rat_event::{HandleEvent, Outcome, Regular, ct_event, event_flow};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_reloc::RelocatableState;
use ratatui_core::layout::Rect;

/// State for [TableEdit][crate::TableEdit].
///
/// It has a mode-field for insert/edit/view.
///
/// __Focus__
///
/// When editing the widget blocks the Focus it is part of.
/// You can't leave the table as long as it's editing.
///
/// It is intended that the editor-widget sets up a separate
/// Focus for all the widgets and navigates independently of
/// the main application Focus.
///
///
///
pub struct TableEditState<T, E> {
    /// Editing mode.
    pub mode: Mode,
    /// Backing table.
    pub core: TableCoreState<RowSelection>,
    /// Editor
    pub editor: E,
    /// Data store
    pub data: Vec<T>,

    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

impl<T, E> Default for TableEditState<T, E>
where
    E: Default,
{
    fn default() -> Self {
        Self {
            mode: Default::default(),
            core: Default::default(),
            editor: Default::default(),
            data: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<T, E> HasFocus for TableEditState<T, E> {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    fn focus(&self) -> FocusFlag {
        self.core.focus.clone()
    }

    fn area(&self) -> Rect {
        self.core.area
    }

    fn navigable(&self) -> Navigation {
        match self.mode {
            Mode::View => self.core.focus.navigable(),
            Mode::Edit | Mode::Insert => Navigation::Lock,
        }
    }
}

impl<T, E> HasScreenCursor for TableEditState<T, E>
where
    E: HasScreenCursor,
{
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        match self.mode {
            Mode::View => None,
            Mode::Edit | Mode::Insert => self.editor.screen_cursor(),
        }
    }
}

impl<T, E> RelocatableState for TableEditState<T, E>
where
    E: RelocatableState,
{
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        match self.mode {
            Mode::View => {}
            Mode::Edit | Mode::Insert => {
                self.editor.relocate(shift, clip);
            }
        }
    }
}

impl<T, E> TableEditState<T, E> {
    pub fn new(editor: E) -> Self {
        Self {
            mode: Mode::View,
            core: Default::default(),
            editor,
            data: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }

    pub fn named(name: &str, editor: E) -> Self {
        Self {
            mode: Mode::View,
            core: TableCoreState::named(name),
            editor,
            data: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }

    /// Populate the table-data.
    pub fn with(mut self, data: impl IntoIterator<Item = T>) -> Self {
        self.data.extend(data.into_iter());
        self
    }
}

impl<T, E> TableEditState<T, E> {
    derive_table_widget_state!(BASE);
}

impl<T, E> TableEditState<T, E> {
    derive_table_widget_state!(AREAS);
}

impl<T, E> TableEditState<T, E> {
    derive_table_widget_state!(OFFSET);
}

impl<T, E> TableEditState<T, E> {
    derive_table_widget_state!(SELECTION);
}

impl<T, E> TableEditState<T, E> {
    derive_table_widget_state!(ROWSELECTION);
}

impl<T, E> TableEditState<T, E>
where
    E: TableEditorState<T>,
    T: Clone,
{
    /// Set the data.
    pub fn set_value(&mut self, data: Vec<T>) {
        self.data = data;
        self.core.rows_changed(self.data.len());
    }

    /// Get the data.
    ///
    /// This will not contain the data of the currently
    /// editing row. Instead, it will contain the value
    /// before the edit started.
    pub fn value(&self) -> &Vec<T> {
        &self.data
    }

    /// Clear the widget.
    pub fn clear(&mut self) {
        self.mode = Mode::View;
        self.core.clear_offset();
        self.core.clear_selection();
        self.data.clear();
    }

    /// Editing is active?
    pub fn is_editing(&self) -> bool {
        self.mode == Mode::Edit || self.mode == Mode::Insert
    }

    /// Is the current edit an insert?
    pub fn is_insert(&self) -> bool {
        self.mode == Mode::Insert
    }

    /// Remove the item at the given row.
    pub fn remove(&mut self, row: usize) {
        if self.mode != Mode::View {
            return;
        }
        if row < self.data.len() {
            self.data.remove(row);
            self.core.items_removed(row, 1);
            if !self.core.scroll_to_row(row) {
                self.core.scroll_to_row(row.saturating_sub(1));
            }
        }
    }

    /// Edit a new item inserted at the selected row.
    pub fn edit_new<'a>(&mut self, row: usize, ctx: &'a E::Context<'a>) -> Result<(), E::Err> {
        if self.mode != Mode::View {
            return Ok(());
        }

        let value = self.editor.create_value(ctx)?;
        self.editor.set_value(value.clone(), ctx)?;
        self._start(None, row, Mode::Insert, ctx)?;

        self.data.insert(row, value);
        self.core.items_added(row, 1);

        Ok(())
    }

    /// Edit the item at the given row.
    /// A starting column can be provided too.
    pub fn edit<'a>(
        &mut self,
        col: Option<usize>,
        row: usize,
        ctx: &'a E::Context<'a>,
    ) -> Result<(), E::Err> {
        if self.mode != Mode::View {
            return Ok(());
        }

        let value = &self.data[row];
        self.editor.set_value(value.clone(), ctx)?;
        self._start(col, row, Mode::Edit, ctx)?;

        Ok(())
    }

    /// Cancel editing.
    /// When inserting a new row, the row will be removed.
    pub fn cancel<'a>(&mut self, ctx: &'a E::Context<'a>) -> Result<(), E::Err> {
        if self.mode == Mode::View {
            return Ok(());
        }
        let Some(row) = self.core.selected_checked() else {
            return Ok(());
        };

        self._stop(ctx)?;

        if self.mode == Mode::Insert {
            self.data.remove(row);
            self.core.items_removed(row, 1);
        }

        Ok(())
    }

    /// Commit the changes in the editor.
    pub fn commit<'a>(&mut self, ctx: &'a E::Context<'a>) -> Result<(), E::Err> {
        if self.mode == Mode::View {
            return Ok(());
        }
        let Some(row) = self.core.selected_checked() else {
            return Ok(());
        };

        self._stop(ctx)?;

        let value = self.editor.value(ctx)?;
        if let Some(value) = value {
            self.data[row] = value;
        } else {
            self.data.remove(row);
            self.core.items_removed(row, 1);
        }

        Ok(())
    }

    /// Commit the current changes and start editing
    /// the next row. If the end of the table is reached
    /// a row will be appended.
    pub fn commit_and_append<'a>(&mut self, ctx: &'a E::Context<'a>) -> Result<(), E::Err> {
        self.commit(ctx)?;
        if let Some(row) = self.core.selected_checked() {
            self.edit_new(row + 1, ctx)?;
        }
        Ok(())
    }

    /// Commit the current changes and start editing
    /// the next row. Does not append at the end of the
    /// table.
    pub fn commit_and_edit<'a>(&mut self, ctx: &'a E::Context<'a>) -> Result<(), E::Err> {
        let Some(row) = self.core.selected_checked() else {
            return Ok(());
        };

        self.commit(ctx)?;
        if row + 1 < self.data.len() {
            self.core.select(Some(row + 1));
            self.edit(None, row + 1, ctx)?;
        }
        Ok(())
    }

    fn _start<'a>(
        &mut self,
        col: Option<usize>,
        row: usize,
        mode: Mode,
        ctx: &'a E::Context<'a>,
    ) -> Result<(), E::Err> {
        self.mode = mode;
        self.core.move_to(row);
        self.editor.start(col, ctx)?;
        if let Some(col) = self.editor.focused_col() {
            self.core.scroll_to_col(col);
        } else {
            self.core.scroll_to_col(0);
        }
        Ok(())
    }

    fn _stop<'a>(&mut self, ctx: &'a E::Context<'a>) -> Result<(), E::Err> {
        self.mode = Mode::View;
        self.editor.stop(ctx)?;
        self.core.scroll_to_col(0);
        Ok(())
    }
}

impl<'a, T, S> HandleEvent<Event, &'a S::Context<'a>, Result<Outcome, S::Err>>
    for TableEditState<T, S>
where
    S: HandleEvent<Event, &'a S::Context<'a>, Result<Outcome, S::Err>>,
    S: TableEditorState<T>,
    T: Clone,
{
    #[allow(clippy::collapsible_if)]
    fn handle(&mut self, event: &Event, ctx: &'a S::Context<'a>) -> Result<Outcome, S::Err> {
        if self.mode == Mode::Edit || self.mode == Mode::Insert {
            if self.is_focused() {
                event_flow!({
                    let r = self.editor.handle(event, ctx)?;
                    if let Some(col) = self.editor.focused_col() {
                        if self.core.scroll_to_col(col) {
                            Outcome::Changed
                        } else {
                            r
                        }
                    } else {
                        r
                    }
                });

                event_flow!(match event {
                    ct_event!(keycode press Esc) => {
                        self.cancel(ctx)?;
                        Outcome::Changed
                    }
                    ct_event!(keycode press Enter) => {
                        if self.core.selected_checked() < Some(self.core.rows().saturating_sub(1)) {
                            self.commit_and_edit(ctx)?;
                            Outcome::Changed
                        } else {
                            self.commit_and_append(ctx)?;
                            Outcome::Changed
                        }
                    }
                    ct_event!(keycode press Up) => {
                        self.commit(ctx)?;
                        if self.data.is_empty() {
                            self.edit_new(0, ctx)?;
                        } else if let Some(row) = self.core.selected_checked()
                            && row > 0
                        {
                            self.core.select(Some(row - 1));
                        }
                        Outcome::Changed
                    }
                    ct_event!(keycode press Down) => {
                        self.commit(ctx)?;
                        if self.data.is_empty() {
                            self.edit_new(0, ctx)?;
                        } else if let Some(row) = self.core.selected_checked()
                            && row + 1 < self.data.len()
                        {
                            self.core.select(Some(row + 1));
                        }
                        Outcome::Changed
                    }
                    _ => Outcome::Continue,
                });
            }

            Ok(Outcome::Continue)
        } else {
            if self.core.focus.gained_focus() {
                if self.data.is_empty() {
                    self.edit_new(0, ctx)?;
                }
            }

            event_flow!(match event {
                ct_event!(mouse any for m) if self.mouse.doubleclick(self.core.table_area, m) => {
                    if let Some((col, row)) = self.core.cell_at_clicked((m.column, m.row)) {
                        self.edit(Some(col), row, ctx)?;
                        Outcome::Changed
                    } else {
                        Outcome::Continue
                    }
                }
                _ => Outcome::Continue,
            });

            if self.is_focused() {
                event_flow!(match event {
                    ct_event!(keycode press Insert) => {
                        if let Some(row) = self.core.selected() {
                            self.edit_new(row, ctx)?;
                        }
                        Outcome::Changed
                    }
                    ct_event!(keycode press Delete) => {
                        if let Some(row) = self.core.selected_checked() {
                            self.remove(row);
                            if self.data.is_empty() {
                                self.edit_new(0, ctx)?;
                            }
                        }
                        Outcome::Changed
                    }
                    ct_event!(keycode press Enter) | ct_event!(keycode press F(2)) => {
                        if let Some(row) = self.core.selected_checked() {
                            self.edit(None, row, ctx)?;
                            Outcome::Changed
                        } else if self.core.rows() == 0 {
                            self.edit_new(0, ctx)?;
                            Outcome::Changed
                        } else {
                            Outcome::Continue
                        }
                    }
                    ct_event!(keycode press Down) => {
                        if let Some(row) = self.core.selected() {
                            if row == self.core.rows().saturating_sub(1) {
                                self.edit_new(row + 1, ctx)?;
                                Outcome::Changed
                            } else {
                                Outcome::Continue
                            }
                        } else if self.core.rows() == 0 {
                            self.edit_new(0, ctx)?;
                            Outcome::Changed
                        } else {
                            Outcome::Continue
                        }
                    }
                    _ => {
                        Outcome::Continue
                    }
                });
            }

            event_flow!(self.core.handle(event, Regular));

            Ok(Outcome::Continue)
        }
    }
}
