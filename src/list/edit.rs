//!
//! Adds inline editing support for List.
//!

use crate::event::EditOutcome;
use crate::list::selection::RowSelection;
use crate::list::{List, ListSelection, ListState};
use log::warn;
use rat_event::util::MouseFlags;
use rat_event::{ct_event, flow, HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::{FocusFlag, HasFocus};
use rat_text::HasScreenCursor;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::StatefulWidget;

/// Editing mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    View,
    Edit,
    Insert,
}

/// Edit-List widget.
///
/// Contains the base list and the edit-widget.
#[derive(Debug, Default)]
pub struct EditList<'a, E>
where
    E: StatefulWidget + 'a,
{
    list: List<'a, RowSelection>,
    editor: E,
}

/// State & event-handling.
///
/// Contains `mode` to differentiate between edit/non-edit.
/// This will lock the focus to the input line while editing.
///
#[derive(Debug)]
pub struct EditListState<S> {
    /// Editing mode.
    pub mode: Mode,

    /// List state
    pub list: ListState<RowSelection>,
    /// EditorState. Some indicates editing is active.
    pub editor: S,

    /// Flags for mouse interaction.
    pub mouse: MouseFlags,
}

impl<'a, E> EditList<'a, E>
where
    E: StatefulWidget + 'a,
{
    pub fn new(list: List<'a, RowSelection>, edit: E) -> Self {
        Self { list, editor: edit }
    }
}

impl<'a, E> StatefulWidget for EditList<'a, E>
where
    E: StatefulWidget + 'a,
{
    type State = EditListState<E::State>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.list.render(area, buf, &mut state.list);

        if state.mode == Mode::Insert || state.mode == Mode::Edit {
            if let Some(row) = state.list.selected() {
                // but it might be out of view
                if let Some(row_area) = state.list.row_area(row) {
                    self.editor.render(row_area, buf, &mut state.editor);
                }
            } else {
                if cfg!(debug_assertions) {
                    warn!("no row selection, not rendering editor");
                }
            }
        }
    }
}

impl<S> Default for EditListState<S>
where
    S: Default,
{
    fn default() -> Self {
        Self {
            mode: Mode::View,
            list: Default::default(),
            editor: S::default(),
            mouse: Default::default(),
        }
    }
}

impl<S> HasFocus for EditListState<S>
where
    S: HasFocus,
{
    fn focus(&self) -> FocusFlag {
        match self.mode {
            Mode::View => self.list.focus(),
            Mode::Edit => self.editor.focus(),
            Mode::Insert => self.editor.focus(),
        }
    }

    fn area(&self) -> Rect {
        self.list.area()
    }
}

impl<S> HasScreenCursor for EditListState<S>
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

impl<S> EditListState<S> {
    /// New state.
    pub fn new(editor: S) -> Self {
        Self {
            mode: Mode::View,
            list: Default::default(),
            editor,
            mouse: Default::default(),
        }
    }

    /// New state with a named focus.
    pub fn named(name: &str, editor: S) -> Self {
        Self {
            mode: Mode::View,
            list: ListState::named(name),
            editor,
            mouse: Default::default(),
        }
    }
}

impl<S> EditListState<S>
where
    S: HasFocus,
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
    /// all the bookkeeping with the list-state.
    pub fn remove(&mut self, row: usize) {
        if self.mode != Mode::View {
            return;
        }
        self.list.items_removed(row, 1);
        if !self.list.scroll_to(row) {
            self.list.scroll_to(row.saturating_sub(1));
        }
    }

    /// Edit a new item inserted at the selected row.
    ///
    /// The editor state must be initialized to an appropriate state
    /// beforehand.
    ///
    /// This does all the bookkeeping with the list-state and
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
    /// This does all the bookkeeping with the list-state and
    /// switches the mode to Mode::Edit.
    pub fn edit(&mut self, row: usize) {
        if self.mode != Mode::View {
            return;
        }
        self._start(row, Mode::Edit);
    }

    fn _start(&mut self, pos: usize, mode: Mode) {
        if self.list.is_focused() {
            self.list.focus().set(false);
            self.editor.focus().set(true);
        }

        self.mode = mode;
        if self.mode == Mode::Insert {
            self.list.items_added(pos, 1);
        }
        self.list.move_to(pos);
    }

    /// Cancel editing.
    ///
    /// This doesn't reset the edit-widget.
    ///
    /// But it does all the bookkeeping with the list-state and
    /// switches the mode back to Mode::View.
    pub fn cancel(&mut self) {
        if self.mode == Mode::View {
            return;
        }
        let Some(row) = self.list.selected() else {
            return;
        };
        if self.mode == Mode::Insert {
            self.list.items_removed(row, 1);
        }
        self._stop();
    }

    /// Commit the changes in the editor.
    ///
    /// This doesn't copy the data back from the editor to the
    /// row-item.
    ///
    /// But it does all the bookkeeping with the list-state and
    /// switches the mode back to Mode::View.
    pub fn commit(&mut self) {
        if self.mode == Mode::View {
            return;
        }
        self._stop();
    }

    fn _stop(&mut self) {
        self.mode = Mode::View;
        if self.editor.is_focused() {
            self.list.focus.set(true);
            self.editor.focus().set(false);
        }
    }
}

impl<S, C> HandleEvent<crossterm::event::Event, C, EditOutcome> for EditListState<S>
where
    S: HandleEvent<crossterm::event::Event, C, EditOutcome>,
    S: HandleEvent<crossterm::event::Event, MouseOnly, EditOutcome>,
    S: HasFocus,
{
    fn handle(&mut self, event: &crossterm::event::Event, ctx: C) -> EditOutcome {
        if self.mode == Mode::Edit || self.mode == Mode::Insert {
            if self.editor.is_focused() {
                flow!(self.editor.handle(event, ctx));

                flow!(match event {
                    ct_event!(keycode press Esc) => {
                        EditOutcome::Cancel
                    }
                    ct_event!(keycode press Enter) => {
                        EditOutcome::Commit
                    }
                    ct_event!(keycode press Tab) => {
                        if self.list.selected() < Some(self.list.rows().saturating_sub(1)) {
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
            } else {
                flow!(self.editor.handle(event, MouseOnly));
            }
        } else {
            flow!(match event {
                ct_event!(mouse any for m) if self.mouse.doubleclick(self.list.inner, m) => {
                    if self.list.row_at_clicked((m.column, m.row)).is_some() {
                        EditOutcome::Edit
                    } else {
                        EditOutcome::Continue
                    }
                }
                _ => EditOutcome::Continue,
            });

            if self.list.is_focused() {
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
                        if let Some(row) = self.list.selection.lead_selection() {
                            if row == self.list.rows().saturating_sub(1) {
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
                flow!(match self.list.handle(event, Regular) {
                    Outcome::Continue => EditOutcome::Continue,
                    Outcome::Unchanged => EditOutcome::Unchanged,
                    Outcome::Changed => EditOutcome::Changed,
                });
            }

            flow!(self.list.handle(event, MouseOnly));
        }

        EditOutcome::Continue
    }
}

/// Handle extended edit-events.
///
/// List events are only handled if focus is true.
/// Mouse events are processed if they are in range.
///
/// The qualifier indicates which event-handler for EState will
/// be called. Or it can be used to pass in some context.
pub fn handle_edit_events<S, C>(
    state: &mut EditListState<S>,
    focus: bool,
    event: &crossterm::event::Event,
    qualifier: C,
) -> EditOutcome
where
    S: HandleEvent<crossterm::event::Event, C, EditOutcome>,
    S: HandleEvent<crossterm::event::Event, MouseOnly, EditOutcome>,
    S: HasFocus,
{
    state.list.focus.set(focus);
    state.handle(event, qualifier)
}
