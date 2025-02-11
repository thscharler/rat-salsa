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
use crate::textdata::Row;
use crate::{Table, TableContext, TableData, TableState};
use log::warn;
use rat_cursor::HasScreenCursor;
use rat_event::util::MouseFlags;
use rat_event::{ct_event, try_flow, HandleEvent, Outcome, Regular};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_reloc::RelocatableState;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::widgets::StatefulWidget;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

/// Extends TableData with the capability to set the actual data
/// at a later point in time.
///
/// This is needed to inject the data during rendering, while
/// leaving the rendering to the caller.
///
/// Due to life-time issues the data is given as Rc<>.
pub trait TableDataVec<D>: TableData<'static> {
    /// Set the actual table data.
    fn set_data(&mut self, data: Rc<RefCell<Vec<D>>>);
}

/// Widget that supports row-wise editing of a table.
///
/// This widget keeps a `Vec<RowData>` and modifies it.
///
/// It's parameterized with a `Editor` widget, that renders
/// the input line and handles events.
pub struct EditableTableVec<'a, E>
where
    E: TableEditor + 'a,
{
    table: Table<'a, RowSelection>,
    table_data: Box<dyn TableDataVec<<<E as TableEditor>::State as TableEditorState>::Value>>,
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
    mode: Mode,

    /// Backing table.
    pub table: TableState<RowSelection>,
    /// Editor
    editor: S,
    /// Data store
    editor_data: Rc<RefCell<Vec<S::Value>>>,

    mouse: MouseFlags,
}

impl<'a, E> EditableTableVec<'a, E>
where
    E: TableEditor + 'a,
{
    pub fn new(
        table_data: impl TableDataVec<<<E as TableEditor>::State as TableEditorState>::Value> + 'static,
        table: Table<'a, RowSelection>,
        editor: E,
    ) -> Self {
        Self {
            table,
            table_data: Box::new(table_data),
            editor,
        }
    }
}

impl<'a, D> TableData<'a> for Box<dyn TableDataVec<D> + 'a> {
    fn rows(&self) -> usize {
        (**self).rows()
    }

    fn header(&self) -> Option<Row<'a>> {
        (**self).header()
    }

    fn footer(&self) -> Option<Row<'a>> {
        (**self).footer()
    }

    fn row_height(&self, row: usize) -> u16 {
        (**self).row_height(row)
    }

    fn row_style(&self, row: usize) -> Option<Style> {
        (**self).row_style(row)
    }

    fn widths(&self) -> Vec<Constraint> {
        (**self).widths()
    }

    fn render_cell(
        &self,
        ctx: &TableContext,
        column: usize,
        row: usize,
        area: Rect,
        buf: &mut Buffer,
    ) {
        (**self).render_cell(ctx, column, row, area, buf)
    }
}

impl<'a, E> Debug for EditableTableVec<'a, E>
where
    E: Debug,
    E: TableEditor + 'a,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EditVec")
            .field("table", &self.table)
            .field("table_data", &"..dyn..")
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
    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.table_data.set_data(state.editor_data.clone());
        self.table
            .data(self.table_data)
            .render(area, buf, &mut state.table);

        if state.mode == Mode::Insert || state.mode == Mode::Edit {
            if let Some(row) = state.table.selected() {
                // but it might be out of view
                if let Some((row_area, cell_areas)) = state.table.row_cells(row) {
                    self.editor
                        .render(row_area, &cell_areas, buf, &mut state.editor);
                }
            } else {
                if cfg!(debug_assertions) {
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
            editor_data: Rc::new(RefCell::new(Vec::default())),
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
            .field("editor_data", &self.editor_data)
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
            editor_data: Rc::new(RefCell::new(vec![])),
            mouse: Default::default(),
        }
    }

    pub fn named(name: &str, editor: S) -> Self {
        Self {
            mode: Mode::View,
            table: TableState::named(name),
            editor,
            editor_data: Rc::new(RefCell::new(vec![])),
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
        self.editor_data = Rc::new(RefCell::new(data));
    }

    /// Get the edit data.
    pub fn value(&self) -> Vec<S::Value> {
        self.editor_data.borrow().clone()
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
        if row < self.editor_data.borrow().len() {
            self.editor_data.borrow_mut().remove(row);
            self.table.items_removed(row, 1);
            if !self.table.scroll_to_row(row) {
                self.table.scroll_to_row(row.saturating_sub(1));
            }
        }
    }

    /// Edit a new item inserted at the selected row.
    pub fn edit_new(&mut self, row: usize, ctx: S::Context<'_>) -> Result<(), S::Err> {
        if self.mode != Mode::View {
            return Ok(());
        }
        let value = self.editor.create_value(ctx.clone())?;
        self.editor.set_value(value.clone(), ctx.clone())?;
        self.editor_data.borrow_mut().insert(row, value);
        self._start(row, Mode::Insert);
        Ok(())
    }

    /// Edit the item at the selected row.
    pub fn edit(&mut self, row: usize, ctx: S::Context<'_>) -> Result<(), S::Err> {
        if self.mode != Mode::View {
            return Ok(());
        }
        {
            let value = &self.editor_data.borrow()[row];
            self.editor.set_value(value.clone(), ctx.clone())?;
        }
        self._start(row, Mode::Edit);
        Ok(())
    }

    fn _start(&mut self, pos: usize, mode: Mode) {
        if self.table.is_focused() {
            // black magic
            FocusBuilder::build_for(&self.editor).first();
        }

        self.mode = mode;
        if self.mode == Mode::Insert {
            self.table.items_added(pos, 1);
        }
        self.table.move_to(pos);
        self.table.scroll_to_col(0);
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
            self.editor_data.borrow_mut().remove(row);
            self.table.items_removed(row, 1);
        }
        self._stop();
    }

    /// Commit the changes in the editor.
    pub fn commit(&mut self, ctx: S::Context<'_>) -> Result<(), S::Err> {
        if self.mode == Mode::View {
            return Ok(());
        }
        let Some(row) = self.table.selected_checked() else {
            return Ok(());
        };
        {
            let value = self.editor.value(ctx.clone())?;
            if let Some(value) = value {
                self.editor_data.borrow_mut()[row] = value;
            } else {
                self.editor_data.borrow_mut().remove(row);
                self.table.items_removed(row, 1);
            }
        }
        self._stop();
        Ok(())
    }

    pub fn commit_and_append(&mut self, ctx: S::Context<'_>) -> Result<(), S::Err> {
        self.commit(ctx.clone())?;
        if let Some(row) = self.table.selected_checked() {
            self.edit_new(row + 1, ctx.clone())?;
        }
        Ok(())
    }

    pub fn commit_and_edit(&mut self, ctx: S::Context<'_>) -> Result<(), S::Err> {
        let Some(row) = self.table.selected_checked() else {
            return Ok(());
        };

        self.commit(ctx.clone())?;
        self.table.select(Some(row + 1));
        self.edit(row + 1, ctx.clone())?;
        Ok(())
    }

    fn _stop(&mut self) {
        self.mode = Mode::View;
        self.table.scroll_to_col(0);
    }
}

impl<'a, S> HandleEvent<crossterm::event::Event, S::Context<'a>, Result<Outcome, S::Err>>
    for EditableTableVecState<S>
where
    S: HandleEvent<crossterm::event::Event, S::Context<'a>, Result<Outcome, S::Err>>,
    S: TableEditorState,
{
    fn handle(
        &mut self,
        event: &crossterm::event::Event,
        ctx: S::Context<'a>,
    ) -> Result<Outcome, S::Err> {
        if self.mode == Mode::Edit || self.mode == Mode::Insert {
            try_flow!(match self.editor.handle(event, ctx.clone())? {
                Outcome::Continue => Outcome::Continue,
                Outcome::Unchanged => Outcome::Unchanged,
                r => {
                    if let Some(col) = self.editor.focused_col() {
                        self.table.scroll_to_col(col);
                    }
                    r
                }
            });

            try_flow!(match event {
                ct_event!(keycode press Esc) => {
                    self.cancel();
                    Outcome::Changed
                }
                ct_event!(keycode press Enter) => {
                    if self.table.selected_checked() < Some(self.table.rows().saturating_sub(1)) {
                        self.commit_and_edit(ctx.clone())?;
                        Outcome::Changed
                    } else {
                        self.commit_and_append(ctx.clone())?;
                        Outcome::Changed
                    }
                }
                ct_event!(keycode press Up) => {
                    self.commit(ctx.clone())?;
                    Outcome::Changed
                }
                ct_event!(keycode press Down) => {
                    self.commit(ctx.clone())?;
                    Outcome::Changed
                }
                _ => Outcome::Continue,
            });

            Ok(Outcome::Continue)
        } else {
            try_flow!(match event {
                ct_event!(mouse any for m) if self.mouse.doubleclick(self.table.table_area, m) => {
                    if let Some((_col, row)) = self.table.cell_at_clicked((m.column, m.row)) {
                        self.edit(row, ctx.clone())?;
                        Outcome::Changed
                    } else {
                        Outcome::Continue
                    }
                }
                _ => Outcome::Continue,
            });

            try_flow!(match event {
                ct_event!(keycode press Insert) => {
                    if let Some(row) = self.table.selected_checked() {
                        self.edit_new(row, ctx.clone())?;
                    }
                    Outcome::Changed
                }
                ct_event!(keycode press Delete) => {
                    if let Some(row) = self.table.selected_checked() {
                        self.remove(row);
                    }
                    Outcome::Changed
                }
                ct_event!(keycode press Enter) | ct_event!(keycode press F(2)) => {
                    if let Some(row) = self.table.selected_checked() {
                        self.edit(row, ctx.clone())?;
                        Outcome::Changed
                    } else if self.table.rows() == 0 {
                        self.edit_new(0, ctx.clone())?;
                        Outcome::Changed
                    } else {
                        Outcome::Continue
                    }
                }
                ct_event!(keycode press Down) => {
                    if let Some(row) = self.table.selected_checked() {
                        if row == self.table.rows().saturating_sub(1) {
                            self.edit_new(row + 1, ctx.clone())?;
                            Outcome::Changed
                        } else {
                            Outcome::Continue
                        }
                    } else if self.table.rows() == 0 {
                        self.edit_new(0, ctx.clone())?;
                        Outcome::Changed
                    } else {
                        Outcome::Continue
                    }
                }
                _ => {
                    Outcome::Continue
                }
            });

            try_flow!(self.table.handle(event, Regular));

            Ok(Outcome::Continue)
        }
    }
}
