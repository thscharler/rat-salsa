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
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{StatefulWidget, StatefulWidgetRef};

/// Edit-List widget.
///
/// Contains the base list and the edit-widget.
#[derive(Debug, Default)]
pub struct EditList<'a, Editor: StatefulWidgetRef + 'a> {
    list: List<'a, RowSelection>,
    edit: Editor,
}

/// State & even-handling.
#[derive(Debug, Default)]
pub struct EditListState<EditorState> {
    /// List state
    pub list: ListState<RowSelection>,
    /// EditorState. Some indicates editing is active.
    pub edit: Option<EditorState>,

    pub mouse: MouseFlags,
}

impl<'a, Editor> EditList<'a, Editor>
where
    Editor: StatefulWidgetRef + 'a,
{
    pub fn new(list: List<'a, RowSelection>, edit: Editor) -> Self {
        Self { list, edit }
    }
}

impl<'a, Editor> StatefulWidgetRef for EditList<'a, Editor>
where
    Editor: StatefulWidgetRef + 'a,
{
    type State = EditListState<Editor::State>;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

impl<'a, Editor> StatefulWidget for EditList<'a, Editor>
where
    Editor: StatefulWidgetRef + 'a,
{
    type State = EditListState<Editor::State>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

fn render_ref<'a, Editor>(
    widget: &EditList<'a, Editor>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut EditListState<Editor::State>,
) where
    Editor: StatefulWidgetRef + 'a,
{
    widget.list.render_ref(area, buf, &mut state.list);

    if let Some(edit_state) = &mut state.edit {
        // expect a selected row
        if let Some(row) = state.list.selected() {
            // but it might be out of view
            if let Some(row_area) = state.list.row_area(row) {
                widget.edit.render_ref(row_area, buf, edit_state);
            }
        }
    }
}

impl<EditorState> HasFocusFlag for EditListState<EditorState>
where
    EditorState: HasFocusFlag,
{
    fn focus(&self) -> FocusFlag {
        if let Some(edit_state) = self.edit.as_ref() {
            edit_state.focus()
        } else {
            self.list.focus()
        }
    }

    fn area(&self) -> Rect {
        if let Some(edit_state) = self.edit.as_ref() {
            edit_state.area()
        } else {
            self.list.area()
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

impl<EditorState, EQualifier> HandleEvent<crossterm::event::Event, EQualifier, EditOutcome>
    for EditListState<EditorState>
where
    EditorState: HandleEvent<crossterm::event::Event, EQualifier, EditOutcome>,
{
    fn handle(&mut self, event: &crossterm::event::Event, qualifier: EQualifier) -> EditOutcome {
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
            if let Some(edit_state) = self.edit.as_mut() {
                flow!(edit_state.handle(event, qualifier));

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

                EditOutcome::Continue
            } else {
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

                match self.list.handle(event, Regular) {
                    Outcome::Continue => EditOutcome::Continue,
                    Outcome::Unchanged => EditOutcome::Unchanged,
                    Outcome::Changed => EditOutcome::Changed,
                }
            }
        } else {
            self.list.handle(event, MouseOnly).into()
        }
    }
}

/// Handle extended edit-events.
///
/// List events are only handled if focus is true.
/// Mouse events are processed if they are in range.
///
/// The qualifier indicates which event-handler for EState will
/// be called. Or it can be used to pass in some context.
pub fn handle_edit_events<EState, EQualifier>(
    state: &mut EditListState<EState>,
    focus: bool,
    event: &crossterm::event::Event,
    qualifier: EQualifier,
) -> EditOutcome
where
    EState: HandleEvent<crossterm::event::Event, EQualifier, EditOutcome>,
{
    state.list.focus.set(focus);
    state.handle(event, qualifier)
}
