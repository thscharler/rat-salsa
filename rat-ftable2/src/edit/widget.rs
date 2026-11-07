use crate::core::TableCore;
use crate::derive::derive_table_widget;
use crate::edit::{Mode, TableEditState, TableEditor};
use crate::selection::row::RowSelection;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;

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
    core: TableCore<'a, &'a T, RowSelection>,
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

impl<'a, T, E> TableEdit<'a, T, E>
where
    E: TableEditor<T> + 'a,
{
    pub fn render(self, area: Rect, buf: &mut Buffer, state: &'a mut TableEditState<T, E::State>) {
        let mut iter = state.data.iter();
        let data = Some((&mut iter) as &mut dyn Iterator<Item = &T>);

        self.core.render(data, area, buf, &mut state.core);

        if state.mode == Mode::Insert || state.mode == Mode::Edit {
            if let Some(row) = state.core.selected() {
                // but it might be out of view
                if let Some((row_area, cell_areas)) = state.core.row_cells(row) {
                    self.editor
                        .render(row_area, &cell_areas, buf, &mut state.editor);
                }
            }
        }
    }
}
