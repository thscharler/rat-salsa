//!
//! Adds inline editing support for List.
//!

use crate::event::EditOutcome;
use crate::list::selection::RowSelection;
use crate::list::{List, ListSelection, ListState};
#[allow(unused_imports)]
use log::debug;
use rat_event::util::MouseFlags;
use rat_event::{ct_event, flow, HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::{FocusFlag, HasFocusFlag};
use rat_text::HasScreenCursor;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::StatefulWidget;

/// Edit-List widget.
///
/// Contains the base list and the edit-widget.
#[derive(Debug, Default)]
pub struct EditList<'a, Editor: StatefulWidget + 'a> {
    list: List<'a, RowSelection>,
    editor: Editor,
}

/// State & even-handling.
#[derive(Debug, Default)]
pub struct EditListState<EditorState> {
    /// List state
    pub list: ListState<RowSelection>,
    /// EditorState. Some indicates editing is active.
    editor: Option<EditorState>,

    /// Flags for mouse interaction.
    pub mouse: MouseFlags,
}

impl<'a, Editor> EditList<'a, Editor>
where
    Editor: StatefulWidget + 'a,
{
    pub fn new(list: List<'a, RowSelection>, edit: Editor) -> Self {
        Self { list, editor: edit }
    }
}

impl<'a, Editor> StatefulWidget for EditList<'a, Editor>
where
    Editor: StatefulWidget + 'a,
{
    type State = EditListState<Editor::State>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.list.render(area, buf, &mut state.list);

        if let Some(edit_state) = &mut state.editor {
            // expect a selected row
            if let Some(row) = state.list.selected() {
                // but it might be out of view
                if let Some(row_area) = state.list.row_area(row) {
                    self.editor.render(row_area, buf, edit_state);
                }
            } else {
                // todo: warn or something??
            }
        }
    }
}

impl<EditorState> HasFocusFlag for EditListState<EditorState>
where
    EditorState: HasFocusFlag,
{
    fn focus(&self) -> FocusFlag {
        if let Some(edit_state) = self.editor.as_ref() {
            edit_state.focus()
        } else {
            self.list.focus()
        }
    }

    fn area(&self) -> Rect {
        if let Some(edit_state) = self.editor.as_ref() {
            edit_state.area()
        } else {
            self.list.area()
        }
    }
}

impl<EditorState> HasScreenCursor for EditListState<EditorState>
where
    EditorState: HasScreenCursor,
{
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        if let Some(edit) = self.editor.as_ref() {
            edit.screen_cursor()
        } else {
            None
        }
    }
}

impl<EditorState> EditListState<EditorState>
where
    EditorState: Default,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn named(name: &str) -> Self {
        Self {
            list: ListState::named(name),
            ..Self::default()
        }
    }
}

impl<EditorState> EditListState<EditorState>
where
    EditorState: HasFocusFlag,
{
    /// Editing is active?
    pub fn is_editing(&self) -> bool {
        self.editor.is_some()
    }

    /// Editor widget.
    ///
    /// __Panic__
    /// Panics if not editing.
    pub fn editor(&self) -> &EditorState {
        self.editor.as_ref().expect("editing")
    }

    /// Editor widget.
    pub fn try_editor(&self) -> Option<&EditorState> {
        self.editor.as_ref()
    }

    /// Editor widget.
    ///
    /// __Panic__
    /// Panics if not editing.
    pub fn editor_mut(&mut self) -> &mut EditorState {
        self.editor.as_mut().expect("editing")
    }

    /// Editor widget.
    pub fn try_editor_mut(&mut self) -> Option<&mut EditorState> {
        self.editor.as_mut()
    }

    /// Start editing at the position.
    ///
    /// The editor state must be initialized to an appropriate state.
    pub fn start_edit(&mut self, pos: usize, edit: EditorState) {
        if self.list.is_focused() {
            self.list.focus().set(false);
            edit.focus().set(true);
        }

        self.editor = Some(edit);

        self.list.items_added(pos, 1);
        self.list.move_to(pos);
    }

    /// Cancel editing.
    ///
    /// Updates the state to remove the edited row.
    pub fn cancel_edit(&mut self) {
        let selected = self.list.selected();

        self.stop_edit();

        if let Some(selected) = selected {
            self.list.items_removed(selected, 1);
            self.list.move_to(selected);
        }
    }

    /// Stops editing.
    ///
    /// Assumes the edited row will be inserted into the base data.
    pub fn stop_edit(&mut self) {
        if let Some(edit) = &mut self.editor {
            if edit.is_focused() {
                self.list.focus.set(true);
            }
        }
        self.editor = None;
    }
}

impl<EditorState, EQualifier> HandleEvent<crossterm::event::Event, EQualifier, EditOutcome>
    for EditListState<EditorState>
where
    EditorState: HandleEvent<crossterm::event::Event, EQualifier, EditOutcome>,
    EditorState: HandleEvent<crossterm::event::Event, MouseOnly, EditOutcome>,
    EditorState: HasFocusFlag,
{
    fn handle(&mut self, event: &crossterm::event::Event, qualifier: EQualifier) -> EditOutcome {
        if !self.is_editing() {
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
                    ct_event!(keycode press Down) => 'f: {
                        if let Some(row) = self.list.selection.lead_selection() {
                            if row == self.list.rows().saturating_sub(1) {
                                break 'f EditOutcome::Append;
                            }
                        }
                        EditOutcome::Continue
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
        } else {
            if self.editor().is_focused() {
                flow!(self.editor_mut().handle(event, qualifier));

                flow!(match event {
                    ct_event!(keycode press Esc) => {
                        EditOutcome::Cancel
                    }
                    ct_event!(keycode press Enter) | ct_event!(keycode press Up) => {
                        EditOutcome::Commit
                    }
                    ct_event!(keycode press Tab) => {
                        if self.list.selected() != Some(self.list.rows().saturating_sub(1)) {
                            EditOutcome::CommitAndEdit
                        } else {
                            EditOutcome::CommitAndAppend
                        }
                    }
                    ct_event!(keycode press Down) => {
                        if self.list.selected() != Some(self.list.rows().saturating_sub(1)) {
                            EditOutcome::Commit
                        } else {
                            EditOutcome::Continue
                        }
                    }
                    _ => EditOutcome::Continue,
                });
            } else {
                flow!(self.editor_mut().handle(event, MouseOnly));
            }
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
pub fn handle_edit_events<EditorState, Qualifier>(
    state: &mut EditListState<EditorState>,
    focus: bool,
    event: &crossterm::event::Event,
    qualifier: Qualifier,
) -> EditOutcome
where
    EditorState: HandleEvent<crossterm::event::Event, Qualifier, EditOutcome>,
    EditorState: HandleEvent<crossterm::event::Event, MouseOnly, EditOutcome>,
    EditorState: HasFocusFlag,
{
    state.list.focus.set(focus);
    state.handle(event, qualifier)
}
