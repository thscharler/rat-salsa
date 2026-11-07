use crate::core::TableCore;
use crate::derive::derive_table_widget;
use crate::selection::TableSelection;
use crate::selection::row::RowSelection;
use crate::vec::TableVecState;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;

///
/// A table that holds a Vec of the data in its state.
///
/// ```
/// TableVec::default()
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
pub struct TableVec<'a, T, Selection = RowSelection> {
    core: TableCore<'a, &'a T, Selection>,
}

impl<'a, T, Selection> Default for TableVec<'a, T, Selection> {
    fn default() -> Self {
        Self {
            core: Default::default(),
        }
    }
}

impl<'a, T, Selection> TableVec<'a, T, Selection> {
    pub fn new() -> Self
    where
        Selection: Default,
    {
        Self::default()
    }

    derive_table_widget!(REF_T);
    derive_table_widget!();
}

impl<'a, T, Selection> TableVec<'a, T, Selection>
where
    Selection: TableSelection + Default,
{
    pub fn render(self, area: Rect, buf: &mut Buffer, state: &'a mut TableVecState<T, Selection>) {
        let mut iter = state.data.iter();
        let data = Some((&mut iter) as &mut dyn Iterator<Item = &T>);
        self.core.render(data, area, buf, &mut state.core);
    }
}
