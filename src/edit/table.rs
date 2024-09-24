//! More general editing in a table.
//!
//! A widget that renders the table and can render
//! an edit-widget on top.
//!
//! __Examples__
//! For examples go to the rat-widget crate.
//! There is `examples/table_edit1.rs`.

use crate::edit::{Editor, EditorState, Mode};
use crate::event::EditOutcome;
use crate::rowselection::RowSelection;
use crate::{Table, TableSelection, TableState};
use log::warn;
use rat_cursor::HasScreenCursor;
use rat_event::util::MouseFlags;
use rat_event::{ct_event, flow, HandleEvent, Outcome, Regular};
use rat_focus::{FocusFlag, HasFocusFlag, Navigation};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::StatefulWidget;
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::StatefulWidgetRef;

/// Widget that supports row-wise editing of a table.
///
/// It's parameterized with a `Editor` widget, that renders
/// the input line and handles events. The result of event-handling
/// is an [EditOutcome] that can be used to do the actual editing.
#[derive(Debug)]
pub struct EditTable<'a, E>
where
    E: Editor + 'a,
{
    table: Table<'a, RowSelection>,
    editor: E,
}

/// State for EditTable.
///
/// Contains `mode` to differentiate between edit/non-edit.
/// This will lock the focus to the input line while editing.
///
#[derive(Debug)]
pub struct EditTableState<S> {
    /// Editing mode.
    pub mode: Mode,

    /// Backing table.
    pub table: TableState<RowSelection>,
    /// Editor
    pub editor: S,
    /// Focus-flag for the whole editor widget.
    pub editor_focus: FocusFlag,

    pub mouse: MouseFlags,
}

impl<'a, E> EditTable<'a, E>
where
    E: Editor + 'a,
{
    pub fn new(table: Table<'a, RowSelection>, editor: E) -> Self {
        Self { table, editor }
    }
}

#[cfg(feature = "unstable-widget-ref")]
impl<'a, E> StatefulWidgetRef for EditTable<'a, E>
where
    E: Editor + 'a,
{
    type State = EditTableState<E::State>;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.table.render_ref(area, buf, &mut state.table);

        if state.mode == Mode::Edit || state.mode == Mode::Insert {
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

impl<'a, E> StatefulWidget for EditTable<'a, E>
where
    E: Editor + 'a,
{
    type State = EditTableState<E::State>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.table.render(area, buf, &mut state.table);

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

impl<S> Default for EditTableState<S>
where
    S: Default,
{
    fn default() -> Self {
        Self {
            mode: Mode::View,
            table: Default::default(),
            editor: S::default(),
            editor_focus: Default::default(),
            mouse: Default::default(),
        }
    }
}

impl<S> HasFocusFlag for EditTableState<S> {
    fn focus(&self) -> FocusFlag {
        match self.mode {
            Mode::View => self.table.focus(),
            Mode::Edit => self.editor_focus.clone(),
            Mode::Insert => self.editor_focus.clone(),
        }
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
        match self.mode {
            Mode::View => self.table.is_focused(),
            Mode::Edit | Mode::Insert => self.editor_focus.get(),
        }
    }

    fn lost_focus(&self) -> bool {
        match self.mode {
            Mode::View => self.table.is_focused(),
            Mode::Edit | Mode::Insert => self.editor_focus.lost(),
        }
    }

    fn gained_focus(&self) -> bool {
        match self.mode {
            Mode::View => self.table.is_focused(),
            Mode::Edit | Mode::Insert => self.editor_focus.gained(),
        }
    }
}

impl<S> HasScreenCursor for EditTableState<S>
where
    S: HasScreenCursor,
{
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        match self.mode {
            Mode::View => None,
            Mode::Edit | Mode::Insert => self.editor.screen_cursor(),
        }
    }
}

impl<S> EditTableState<S> {
    /// New state.
    pub fn new(editor: S) -> Self {
        Self {
            mode: Mode::View,
            table: TableState::new(),
            editor,
            editor_focus: Default::default(),
            mouse: Default::default(),
        }
    }

    /// New state with a named focus.
    pub fn named(name: &str, editor: S) -> Self {
        Self {
            mode: Mode::View,
            table: TableState::named(name),
            editor,
            mouse: Default::default(),
            editor_focus: Default::default(),
        }
    }
}

impl<S> EditTableState<S>
where
    S: EditorState,
{
    /// Editing is active?
    pub fn is_editing(&self) -> bool {
        self.mode == Mode::Edit || self.mode == Mode::Insert
    }

    /// Is the current edit an insert?
    pub fn is_insert(&self) -> bool {
        self.mode == Mode::Insert
    }

    /// Remove the item at the selected row.
    ///
    /// This doesn't change the actual list of items, but does
    /// all the bookkeeping with the table-state.
    pub fn remove(&mut self, row: usize) {
        if self.mode != Mode::View {
            return;
        }
        self.table.items_removed(row, 1);
        if !self.table.scroll_to_row(row) {
            self.table.scroll_to_row(row.saturating_sub(1));
        }
    }

    /// Edit a new item inserted at the selected row.
    ///
    /// The editor state must be initialized to an appropriate state
    /// beforehand.
    ///
    /// __See__
    /// [EditorState::set_edit_data]
    ///
    /// This does all the bookkeeping with the table-state and
    /// switches the mode to Mode::Insert.
    pub fn edit_new(&mut self, row: usize) {
        if self.mode != Mode::View {
            return;
        }
        self._start(row, Mode::Insert);
    }

    /// Edit the item at the selected row.
    ///
    /// The editor state must be initialized to an appropriate state
    /// beforehand.
    ///
    /// __See__
    /// [EditorState::set_edit_data]
    ///
    /// This does all the bookkeeping with the table-state and
    /// switches the mode to Mode::Edit.
    pub fn edit(&mut self, row: usize) {
        if self.mode != Mode::View {
            return;
        }
        self._start(row, Mode::Edit);
    }

    fn _start(&mut self, pos: usize, mode: Mode) {
        if self.table.is_focused() {
            self.table.focus().set(false);
            self.editor_focus.set(true);
            self.editor.focus().first();
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
    /// This doesn't reset the edit-widget.
    ///
    /// __See__
    /// [EditorState::set_edit_data]
    ///
    /// But it does all the bookkeeping with the table-state and
    /// switches the mode back to Mode::View.
    pub fn cancel(&mut self) {
        if self.mode == Mode::View {
            return;
        }
        let Some(row) = self.table.selected() else {
            return;
        };
        if self.mode == Mode::Insert {
            self.table.items_removed(row, 1);
        }
        self._stop();
    }

    /// Commit the changes in the editor.
    ///
    /// This doesn't copy the data back from the editor to the
    /// row-item.
    ///
    /// __See__
    /// [EditorState::get_edit_data]
    ///
    /// But it does all the bookkeeping with the table-state and
    /// switches the mode back to Mode::View.
    pub fn commit(&mut self) {
        if self.mode == Mode::View {
            return;
        }
        self._stop();
    }

    fn _stop(&mut self) {
        self.mode = Mode::View;
        if self.editor_focus.get() {
            self.table.focus.set(true);
            self.editor_focus.set(false);
        }
        self.table.scroll_to_col(0);
    }
}

impl<'a, S> HandleEvent<crossterm::event::Event, &'a S::Context<'a>, EditOutcome>
    for EditTableState<S>
where
    S: HandleEvent<crossterm::event::Event, &'a S::Context<'a>, EditOutcome>,
    S: EditorState,
{
    fn handle(&mut self, event: &crossterm::event::Event, ctx: &'a S::Context<'a>) -> EditOutcome {
        if self.mode == Mode::Edit || self.mode == Mode::Insert {
            if self.editor_focus.is_focused() {
                flow!(match self.editor.handle(event, ctx) {
                    EditOutcome::Continue => EditOutcome::Continue,
                    EditOutcome::Unchanged => EditOutcome::Unchanged,
                    r => {
                        if let Some(col) = self.editor.focused_col() {
                            self.table.scroll_to_col(col);
                        }
                        r
                    }
                });

                flow!(match event {
                    ct_event!(keycode press Esc) => {
                        EditOutcome::Cancel
                    }
                    ct_event!(keycode press Enter) => {
                        if self.table.selected() < Some(self.table.rows().saturating_sub(1)) {
                            EditOutcome::CommitAndEdit
                        } else {
                            EditOutcome::CommitAndAppend
                        }
                    }
                    ct_event!(keycode press Up) => {
                        EditOutcome::Commit
                    }
                    ct_event!(keycode press Down) => {
                        EditOutcome::Commit
                    }
                    _ => EditOutcome::Continue,
                });
            }
            EditOutcome::Continue
        } else {
            flow!(match event {
                ct_event!(mouse any for m) if self.mouse.doubleclick(self.table.table_area, m) => {
                    if self.table.cell_at_clicked((m.column, m.row)).is_some() {
                        EditOutcome::Edit
                    } else {
                        EditOutcome::Continue
                    }
                }
                _ => EditOutcome::Continue,
            });

            if self.table.is_focused() {
                flow!(match event {
                    ct_event!(keycode press Insert) => {
                        EditOutcome::Insert
                    }
                    ct_event!(keycode press Delete) => {
                        EditOutcome::Remove
                    }
                    ct_event!(keycode press Enter) | ct_event!(keycode press F(2)) => {
                        EditOutcome::Edit
                    }
                    ct_event!(keycode press Down) => {
                        if let Some((_column, row)) = self.table.selection.lead_selection() {
                            if row == self.table.rows().saturating_sub(1) {
                                EditOutcome::Append
                            } else {
                                EditOutcome::Continue
                            }
                        } else {
                            EditOutcome::Continue
                        }
                    }
                    _ => {
                        EditOutcome::Continue
                    }
                });
            }

            match self.table.handle(event, Regular) {
                Outcome::Continue => EditOutcome::Continue,
                Outcome::Unchanged => EditOutcome::Unchanged,
                Outcome::Changed => EditOutcome::Changed,
            }
        }
    }
}
