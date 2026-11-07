use crate::core::TableCore;
use crate::derive::derive_table_widget;
use crate::selection::TableSelection;
use crate::selection::row::RowSelection;
use crate::table::TableState;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::widgets::StatefulWidget;

/// Table widget that renders data from an Iterator.
///
/// This will do minimal rendering of the visible rows/cells
/// of the table.
///
/// ```
/// Table::default()
///         .data(state.table_data.iter())
///         .widths([
///             Constraint::Length(6),
///             Constraint::Length(20),
///             Constraint::Length(15),
///             Constraint::Length(15),
///             Constraint::Length(3),
///         ])
///         .render_cell(0, |_v, area, buf, ctx| {
///             row_fmt.fmt_u(ctx.row).render(area, buf)
///         })
///         .render_cell(1, |v, area, buf, _ctx| v.text.render(area, buf))
///         .render_cell(2, |v, area, buf, _ctx| {
///             num1_fmt.fmt_u(v.num1).render(area, buf)
///         })
///         .render_cell(3, |v, area, buf, _ctx| {
///             num2_fmt.fmt_u(v.num2).render(area, buf)
///         })
///         .render_cell(4, |v, area, buf, _ctx| {
///             let cc = if v.check { "\u{2622}" } else { "\u{2623}" };
///             cc.render(area, buf);
///         })
///         .column_spacing(1)
///         .header(vec!["Nr", "Text", "Val1", "Val2", "State"])
///         .footer(vec!["a", "b", "c", "d", "e"])
///         .block(
///             Block::bordered()
///                 .border_type(BorderType::Rounded)
///                 .title("tabledata-iter (no row-count)"),
///         )
///         .vscroll(Scroll::new())
///         .styles(table(&ctx.theme))
///         .render(l0[0], buf, &mut state.table);
/// ```
pub struct Table<'a, T: Copy, Selection = RowSelection> {
    core: TableCore<'a, T, Selection>,
    data: Option<Box<dyn Iterator<Item = T> + 'a>>,
}

impl<'a, T: Copy, Selection> Default for Table<'a, T, Selection> {
    fn default() -> Self {
        Self {
            core: Default::default(),
            data: Default::default(),
        }
    }
}

impl<'a, T: Copy + 'a, Selection> Table<'a, T, Selection> {
    pub fn new() -> Self
    where
        Selection: Default,
    {
        Self::default()
    }

    #[inline]
    pub fn data(mut self, data: impl IntoIterator<Item = T> + 'a) -> Self {
        self.data = Some(Box::new(data.into_iter()));
        self
    }

    derive_table_widget!(T);
    derive_table_widget!();
}

impl<'a, T: Copy, Selection> StatefulWidget for Table<'a, T, Selection>
where
    T: 'a,
    Selection: TableSelection + Default,
{
    type State = TableState<Selection>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if let Some(mut data) = self.data {
            self.core
                .render(Some(&mut data), area, buf, &mut state.core);
        } else {
            self.core.render(None, area, buf, &mut state.core);
        }
    }
}
