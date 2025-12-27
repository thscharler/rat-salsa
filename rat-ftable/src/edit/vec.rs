//! Specialized editing in a table. Keeps a Vec of
//! the row-data.
//!
//! A widget that renders the table and can render
//! an edit-widget on top.
//!
//! __Examples__
//! For examples go to the rat-widget crate.
//! There is `examples/table_edit2.rs`.

use crate::edit::{Mode, TableEditor, TableEditorState};
use crate::rowselection::RowSelection;
use crate::util::clear_buf_area;
use crate::{Table, TableState};
use log::warn;
use rat_cursor::HasScreenCursor;
use rat_event::util::MouseFlags;
use rat_event::{HandleEvent, Outcome, Regular, ct_event, event_flow, try_flow};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_reloc::RelocatableState;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::Event;
use std::fmt::{Debug, Formatter};

/// Widget that supports row-wise editing of a table.
///
/// This widget keeps a `Vec<RowData>` and modifies it.
///
/// It's parameterized with a `Editor` widget, that renders
/// the input line and handles events.
#[allow(clippy::type_complexity)]
pub struct EditableTableVec<'a, E>
where
    E: TableEditor + 'a,
{
    table: Box<
        dyn for<'b> Fn(
                &'b [<<E as TableEditor>::State as TableEditorState>::Value],
            ) -> Table<'b, RowSelection>
            + 'a,
    >,
    editor: E,
}

/// State for EditTable.
///
/// Contains `mode` to differentiate between edit/non-edit.
/// This will lock the focus to the input line while editing.
///
pub struct EditableTableVecState<S>
where
    S: TableEditorState,
{
    /// Editing mode.
    pub mode: Mode,

    /// Backing table.
    pub table: TableState<RowSelection>,
    /// Editor
    pub editor: S,

    /// Data store
    pub data: Vec<S::Value>,

    pub mouse: MouseFlags,
}

impl<'a, E> EditableTableVec<'a, E>
where
    E: TableEditor + 'a,
{
    /// Create a new editable table.
    ///
    /// A bit tricky bc lifetimes of the table-data.
    ///
    /// * table: constructor for the Table widget. This gets a `&[Value]` slice
    ///   to display and returns the configured table.
    /// * editor: editor widget.
    pub fn new(
        table: impl for<'b> Fn(
            &'b [<<E as TableEditor>::State as TableEditorState>::Value],
        ) -> Table<'b, RowSelection>
        + 'a,
        editor: E,
    ) -> Self {
        Self {
            table: Box::new(table),
            editor,
        }
    }
}

impl<'a, E> Debug for EditableTableVec<'a, E>
where
    E: Debug,
    E: TableEditor + 'a,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EditVec")
            .field("table", &"..dyn..")
            .field("editor", &self.editor)
            .finish()
    }
}

impl<'a, E> StatefulWidget for EditableTableVec<'a, E>
where
    E: TableEditor + 'a,
{
    type State = EditableTableVecState<E::State>;

    #[allow(clippy::collapsible_else_if)]
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let table = (self.table)(&state.data);
        table.render(area, buf, &mut state.table);

        if state.mode == Mode::Insert || state.mode == Mode::Edit {
            if let Some(row) = state.table.selected() {
                // but it might be out of view
                if let Some((row_area, cell_areas)) = state.table.row_cells(row) {
                    clear_buf_area(row_area, buf);
                    self.editor
                        .render(row_area, &cell_areas, buf, &mut state.editor);
                }
            } else {
                if cfg!(feature = "perf_warnings") {
                    warn!("no row selection, not rendering editor");
                }
            }
        }
    }
}

impl<S> Default for EditableTableVecState<S>
where
    S: TableEditorState + Default,
{
    fn default() -> Self {
        Self {
            mode: Mode::View,
            table: Default::default(),
            editor: S::default(),
            data: Vec::default(),
            mouse: Default::default(),
        }
    }
}

impl<S> Debug for EditableTableVecState<S>
where
    S: TableEditorState + Debug,
    S::Value: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EditVecState")
            .field("mode", &self.mode)
            .field("table", &self.table)
            .field("editor", &self.editor)
            .field("editor_data", &self.data)
            .field("mouse", &self.mouse)
            .finish()
    }
}

impl<S> HasFocus for EditableTableVecState<S>
where
    S: TableEditorState,
{
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    fn focus(&self) -> FocusFlag {
        self.table.focus()
    }

    fn area(&self) -> Rect {
        self.table.area()
    }

    fn navigable(&self) -> Navigation {
        match self.mode {
            Mode::View => self.table.navigable(),
            Mode::Edit | Mode::Insert => Navigation::Lock,
        }
    }

    fn is_focused(&self) -> bool {
        self.table.is_focused()
    }

    fn lost_focus(&self) -> bool {
        self.table.lost_focus()
    }

    fn gained_focus(&self) -> bool {
        self.table.gained_focus()
    }
}

impl<S> HasScreenCursor for EditableTableVecState<S>
where
    S: TableEditorState + HasScreenCursor,
{
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        match self.mode {
            Mode::View => None,
            Mode::Edit | Mode::Insert => self.editor.screen_cursor(),
        }
    }
}

impl<S> RelocatableState for EditableTableVecState<S>
where
    S: TableEditorState + RelocatableState,
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

impl<S> EditableTableVecState<S>
where
    S: TableEditorState,
{
    pub fn new(editor: S) -> Self {
        Self {
            mode: Mode::View,
            table: TableState::new(),
            editor,
            data: Default::default(),
            mouse: Default::default(),
        }
    }

    pub fn named(name: &str, editor: S) -> Self {
        Self {
            mode: Mode::View,
            table: TableState::named(name),
            editor,
            data: Default::default(),
            mouse: Default::default(),
        }
    }
}

impl<S> EditableTableVecState<S>
where
    S: TableEditorState,
{
    /// Set the edit data.
    pub fn set_value(&mut self, data: Vec<S::Value>) {
        self.data = data;
    }

    /// Get a clone of the edit data.
    ///
    /// This will not contain the data of the currently
    /// editing row. Instead, it will contain a default value
    /// at the current edit position.
    pub fn value(&self) -> Vec<S::Value> {
        self.data.clone()
    }

    /// Clear the widget.
    pub fn clear(&mut self) {
        self.mode = Mode::View;
        self.table.clear_offset();
        self.table.clear_selection();
        self.data.clear();
        // todo: self.editor.clear() is missing?
    }

    /// Editing is active?
    pub fn is_editing(&self) -> bool {
        self.mode == Mode::Edit || self.mode == Mode::Insert
    }

    /// Is the current edit an insert?
    pub fn is_insert(&self) -> bool {
        self.mode == Mode::Insert
    }

    /// Remove the item at the selected row.
    pub fn remove(&mut self, row: usize) {
        if self.mode != Mode::View {
            return;
        }
        if row < self.data.len() {
            self.data.remove(row);
            self.table.items_removed(row, 1);
            if !self.table.scroll_to_row(row) {
                self.table.scroll_to_row(row.saturating_sub(1));
            }
        }
    }

    /// Edit a new item inserted at the selected row.
    pub fn edit_new<'a>(&mut self, row: usize, ctx: &'a S::Context<'a>) -> Result<(), S::Err> {
        if self.mode != Mode::View {
            return Ok(());
        }
        let value = self.editor.create_value(ctx)?;
        self.editor.set_value(value.clone(), ctx)?;
        self.data.insert(row, value);
        self._start(0, row, Mode::Insert);
        Ok(())
    }

    /// Edit the item at the selected row.
    pub fn edit<'a>(
        &mut self,
        col: usize,
        row: usize,
        ctx: &'a S::Context<'a>,
    ) -> Result<(), S::Err> {
        if self.mode != Mode::View {
            return Ok(());
        }
        {
            let value = &self.data[row];
            self.editor.set_value(value.clone(), ctx)?;
        }
        self._start(col, row, Mode::Edit);
        Ok(())
    }

    fn _start(&mut self, col: usize, row: usize, mode: Mode) {
        if self.table.is_focused() {
            // black magic
            FocusBuilder::build_for(&self.editor).first();
        }

        self.mode = mode;
        if self.mode == Mode::Insert {
            self.table.items_added(row, 1);
        }
        self.table.move_to(row);
        self.table.scroll_to_col(col);
        self.editor.set_focused_col(col);
    }

    /// Cancel editing.
    ///
    /// Updates the state to remove the edited row.
    pub fn cancel(&mut self) {
        if self.mode == Mode::View {
            return;
        }
        let Some(row) = self.table.selected_checked() else {
            return;
        };
        if self.mode == Mode::Insert {
            self.data.remove(row);
            self.table.items_removed(row, 1);
        }
        self._stop();
    }

    /// Commit the changes in the editor.
    pub fn commit<'a>(&mut self, ctx: &'a S::Context<'a>) -> Result<(), S::Err> {
        if self.mode == Mode::View {
            return Ok(());
        }
        let Some(row) = self.table.selected_checked() else {
            return Ok(());
        };
        {
            let value = self.editor.value(ctx)?;
            if let Some(value) = value {
                self.data[row] = value;
            } else {
                self.data.remove(row);
                self.table.items_removed(row, 1);
            }
        }
        self._stop();
        Ok(())
    }

    pub fn commit_and_append<'a>(&mut self, ctx: &'a S::Context<'a>) -> Result<(), S::Err> {
        self.commit(ctx)?;
        if let Some(row) = self.table.selected_checked() {
            self.edit_new(row + 1, ctx)?;
        }
        Ok(())
    }

    pub fn commit_and_edit<'a>(&mut self, ctx: &'a S::Context<'a>) -> Result<(), S::Err> {
        let Some(row) = self.table.selected_checked() else {
            return Ok(());
        };

        self.commit(ctx)?;
        if row + 1 < self.data.len() {
            self.table.select(Some(row + 1));
            self.edit(0, row + 1, ctx)?;
        }
        Ok(())
    }

    fn _stop(&mut self) {
        self.mode = Mode::View;
        self.table.scroll_to_col(0);
    }
}

impl<'a, S> HandleEvent<Event, &'a S::Context<'a>, Result<Outcome, S::Err>>
    for EditableTableVecState<S>
where
    S: HandleEvent<Event, &'a S::Context<'a>, Result<Outcome, S::Err>>,
    S: TableEditorState,
{
    #[allow(clippy::collapsible_if)]
    fn handle(&mut self, event: &Event, ctx: &'a S::Context<'a>) -> Result<Outcome, S::Err> {
        if self.mode == Mode::Edit || self.mode == Mode::Insert {
            if self.is_focused() {
                event_flow!({
                    let r = self.editor.handle(event, ctx)?;
                    if let Some(col) = self.editor.focused_col() {
                        if self.table.scroll_to_col(col) {
                            Outcome::Changed
                        } else {
                            r
                        }
                    } else {
                        r
                    }
                });

                try_flow!(match event {
                    ct_event!(keycode press Esc) => {
                        self.cancel();
                        Outcome::Changed
                    }
                    ct_event!(keycode press Enter) => {
                        if self.table.selected_checked() < Some(self.table.rows().saturating_sub(1))
                        {
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
                        } else if let Some(row) = self.table.selected_checked()
                            && row > 0
                        {
                            self.table.select(Some(row));
                        }
                        Outcome::Changed
                    }
                    ct_event!(keycode press Down) => {
                        self.commit(ctx)?;
                        if self.data.is_empty() {
                            self.edit_new(0, ctx)?;
                        } else if let Some(row) = self.table.selected_checked()
                            && row + 1 < self.data.len()
                        {
                            self.table.select(Some(row + 1));
                        }
                        Outcome::Changed
                    }
                    _ => Outcome::Continue,
                });
            }

            Ok(Outcome::Continue)
        } else {
            if self.table.gained_focus() {
                if self.data.is_empty() {
                    self.edit_new(0, ctx)?;
                }
            }

            try_flow!(match event {
                ct_event!(mouse any for m) if self.mouse.doubleclick(self.table.table_area, m) => {
                    if let Some((col, row)) = self.table.cell_at_clicked((m.column, m.row)) {
                        self.edit(col, row, ctx)?;
                        Outcome::Changed
                    } else {
                        Outcome::Continue
                    }
                }
                _ => Outcome::Continue,
            });

            if self.is_focused() {
                try_flow!(match event {
                    ct_event!(keycode press Insert) => {
                        if let Some(row) = self.table.selected_checked() {
                            self.edit_new(row, ctx)?;
                        }
                        Outcome::Changed
                    }
                    ct_event!(keycode press Delete) => {
                        if let Some(row) = self.table.selected_checked() {
                            self.remove(row);
                            if self.data.is_empty() {
                                self.edit_new(0, ctx)?;
                            }
                        }
                        Outcome::Changed
                    }
                    ct_event!(keycode press Enter) | ct_event!(keycode press F(2)) => {
                        if let Some(row) = self.table.selected_checked() {
                            self.edit(0, row, ctx)?;
                            Outcome::Changed
                        } else if self.table.rows() == 0 {
                            self.edit_new(0, ctx)?;
                            Outcome::Changed
                        } else {
                            Outcome::Continue
                        }
                    }
                    ct_event!(keycode press Down) => {
                        if let Some(row) = self.table.selected_checked() {
                            if row == self.table.rows().saturating_sub(1) {
                                self.edit_new(row + 1, ctx)?;
                                Outcome::Changed
                            } else {
                                Outcome::Continue
                            }
                        } else if self.table.rows() == 0 {
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

            try_flow!(self.table.handle(event, Regular));

            Ok(Outcome::Continue)
        }
    }
}
