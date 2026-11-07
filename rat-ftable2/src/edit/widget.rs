use crate::derive::derive_table_widget;
use crate::edit::{Mode, TableEditState, TableEditor};
use crate::selection::RowSelection;
use crate::table::Table;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::widgets::StatefulWidget;

/// Table-widget and inline editor for one table-row.
///
/// This widget keeps a `Vec<RowData>` and modifies it.
///
/// It's parameterized with an `Editor` widget, that renders
/// the input-widgets.
///
/// See `examples/table_edit.rs`.
///
pub struct TableEdit<'a, T, E>
where
    E: TableEditor<T> + 'a,
{
    core: Table<'a, &'a T, RowSelection>,
    editor: E,
}

impl<'a, T, E> TableEdit<'a, T, E>
where
    E: TableEditor<T> + 'a,
{
    pub fn new(editor: E) -> Self {
        Self {
            core: Default::default(),
            editor,
        }
    }

    derive_table_widget!();
    derive_table_widget!(REF_T);
}

impl<'a, 'b, T, E> TableEdit<'a, T, E>
where
    E: TableEditor<T> + 'a,
    'b: 'a,
{
    pub fn render(
        mut self,
        area: Rect,
        buf: &mut Buffer,
        state: &'b mut TableEditState<T, E::State>,
    ) {
        self.core = self.core.data(state.data.iter());
        self.core.render(area, buf, &mut state.table);

        if state.mode == Mode::Insert || state.mode == Mode::Edit {
            if let Some(row) = state.table.selected() {
                // but it might be out of view
                if let Some((row_area, cell_areas)) = state.table.row_cells(row) {
                    self.editor
                        .render(row_area, &cell_areas, buf, &mut state.editor);
                }
            }
        }
    }
}
