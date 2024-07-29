//! todo: example is missing. this is hard to grasp.

use crate::event::EditOutcome;
use crate::rowselection::RowSelection;
use crate::{Table, TableSelection, TableState};
use rat_event::util::MouseFlags;
use rat_event::{ct_event, flow, HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::{Focus, HasFocus, HasFocusFlag};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::StatefulWidget;
use ratatui::widgets::StatefulWidgetRef;

/// Widget that supports row-wise editing of a table.
///
/// It's parameterized with a `Editor` widget, that renders
/// the input line and handles events.
#[derive(Debug)]
pub struct EditTable<'a, Editor: EditorWidget + 'a> {
    table: Table<'a, RowSelection>,
    edit: Editor,
}

/// State for EditTable.
///
/// If `edit` is set to Some, this widget switches to editing-mode.
/// Otherwise, it behaves like a normal table.
#[derive(Debug, Default)]
pub struct EditTableState<EditorState> {
    /// Backing table.
    pub table: TableState<RowSelection>,
    /// Editor state.
    pub edit: Option<EditorState>,

    pub mouse: MouseFlags,
}

/// StatefulWidget alike trait.
///
/// This one takes a slice of areas for all the cells in the table,
/// and renders all input widgets as it needs.
pub trait EditorWidget {
    /// State associated with the stateful widget.
    type State;

    /// Standard render call, but with added areas for each cell.
    fn render_ref(
        &self,
        area: Rect,
        cell_areas: &[Rect],
        buf: &mut Buffer,
        state: &mut Self::State,
    );
}

impl<'a, Editor> EditTable<'a, Editor>
where
    Editor: EditorWidget + 'a,
{
    pub fn new(table: Table<'a, RowSelection>, edit: Editor) -> Self {
        Self { table, edit }
    }
}

impl<'a, Editor> StatefulWidgetRef for EditTable<'a, Editor>
where
    Editor: EditorWidget + 'a,
{
    type State = EditTableState<Editor::State>;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

impl<'a, Editor> StatefulWidget for EditTable<'a, Editor>
where
    Editor: EditorWidget + 'a,
{
    type State = EditTableState<Editor::State>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

fn render_ref<'a, Editor>(
    widget: &EditTable<'a, Editor>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut EditTableState<Editor::State>,
) where
    Editor: EditorWidget + 'a,
{
    widget.table.render_ref(area, buf, &mut state.table);

    if let Some(edit_state) = &mut state.edit {
        // expect a selected row
        if let Some(row) = state.table.selected() {
            // but it might be out of view
            if let Some((row_area, cell_areas)) = state.table.row_cells(row) {
                widget
                    .edit
                    .render_ref(row_area, &cell_areas, buf, edit_state);
            }
        }
    }
}

impl<EditorState> HasFocus for EditTableState<EditorState>
where
    EditorState: HasFocus,
{
    fn focus(&self) -> Focus {
        let mut f = Focus::default();
        if let Some(edit_state) = self.edit.as_ref() {
            f.add_container(edit_state);
        } else {
            f.add_flag(&self.table.focus, self.table.area);
        }
        f
    }
}

impl<EState, EQualifier> HandleEvent<crossterm::event::Event, EQualifier, EditOutcome>
    for EditTableState<EState>
where
    EState: HandleEvent<crossterm::event::Event, EQualifier, EditOutcome>,
{
    fn handle(&mut self, event: &crossterm::event::Event, qualifier: EQualifier) -> EditOutcome {
        flow!(match event {
            ct_event!(mouse any for m) if self.mouse.doubleclick(self.table.table_area, m) => {
                if self.table.cell_at_clicked((m.column, m.row)).is_some() {
                    EditOutcome::Edit
                } else {
                    EditOutcome::NotUsed
                }
            }
            _ => EditOutcome::NotUsed,
        });

        if self.table.is_focused() {
            if let Some(edit_state) = self.edit.as_mut() {
                flow!(edit_state.handle(event, qualifier));

                flow!(match event {
                    ct_event!(keycode press Esc) => {
                        EditOutcome::Cancel
                    }
                    ct_event!(keycode press Enter) | ct_event!(keycode press Up) => {
                        EditOutcome::Commit
                    }
                    ct_event!(keycode press Down) => {
                        if self.table.selected() != Some(self.table.rows().saturating_sub(1)) {
                            EditOutcome::Commit
                        } else {
                            EditOutcome::NotUsed
                        }
                    }
                    _ => EditOutcome::NotUsed,
                });

                EditOutcome::NotUsed
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
                        if let Some((_column, row)) = self.table.selection.lead_selection() {
                            if row == self.table.rows().saturating_sub(1) {
                                break 'f EditOutcome::Append;
                            }
                        }
                        EditOutcome::NotUsed
                    }
                    _ => {
                        EditOutcome::NotUsed
                    }
                });

                match self.table.handle(event, Regular) {
                    Outcome::Continue => EditOutcome::NotUsed,
                    Outcome::Unchanged => EditOutcome::Unchanged,
                    Outcome::Changed => EditOutcome::Changed,
                }
            }
        } else {
            self.table.handle(event, MouseOnly).into()
        }
    }
}

/// Handle extended edit-events.
///
/// Table events are only handled if focus is true.
/// Mouse events are processed if they are in range.
///
/// The qualifier indicates which event-handler for EState will
/// be called. Or it can be used to pass in some context.
pub fn handle_edit_events<EState, EQualifier>(
    state: &mut EditTableState<EState>,
    focus: bool,
    event: &crossterm::event::Event,
    qualifier: EQualifier,
) -> EditOutcome
where
    EState: HandleEvent<crossterm::event::Event, EQualifier, EditOutcome>,
{
    state.table.focus.set(focus);
    state.handle(event, qualifier)
}
