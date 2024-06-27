use rat_ftable::FTableState;
use rat_input::layout::{layout_edit, EditConstraint};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use std::fmt::Debug;

pub(crate) fn render_tablestate<Selection: Debug>(
    state: &FTableState<Selection>,
    area: Rect,
    buf: &mut Buffer,
) {
    let l = layout_edit(
        area,
        &[
            EditConstraint::Label("count_rows"),
            EditConstraint::Widget(20),
            //
            EditConstraint::Label("rows"),
            EditConstraint::Widget(20),
            EditConstraint::Label("row_offset"),
            EditConstraint::Widget(20),
            EditConstraint::Label("max_row_offset"),
            EditConstraint::Widget(20),
            EditConstraint::Label("row_page_len"),
            EditConstraint::Widget(20),
            //
            EditConstraint::Label("columns"),
            EditConstraint::Widget(20),
            EditConstraint::Label("col_offset"),
            EditConstraint::Widget(20),
            EditConstraint::Label("max_col_offset"),
            EditConstraint::Widget(20),
            EditConstraint::Label("col_page_len"),
            EditConstraint::Widget(20),
            EditConstraint::Label("selection"),
            EditConstraint::Widget(20),
        ],
    );
    let mut l = l.iter();

    "count_rows".render(l.label(), buf);
    format!("{}", state._counted_rows)
        .to_string()
        .render(l.widget(), buf);

    "rows".render(l.label(), buf);
    format!("{}", state.rows)
        .to_string()
        .render(l.widget(), buf);

    "row_offset".render(l.label(), buf);
    format!("{}", state.row_offset)
        .to_string()
        .render(l.widget(), buf);

    "max_row_offset".render(l.label(), buf);
    format!("{}", state.max_row_offset)
        .to_string()
        .render(l.widget(), buf);

    "row_page_len".render(l.label(), buf);
    format!("{}", state.row_page_len)
        .to_string()
        .render(l.widget(), buf);

    "columns".render(l.label(), buf);
    format!("{}", state.columns)
        .to_string()
        .render(l.widget(), buf);

    "col_offset".render(l.label(), buf);
    format!("{}", state.col_offset)
        .to_string()
        .render(l.widget(), buf);

    "max_col_offset".render(l.label(), buf);
    format!("{}", state.max_col_offset)
        .to_string()
        .render(l.widget(), buf);

    "col_page_len".render(l.label(), buf);
    format!("{}", state.col_page_len)
        .to_string()
        .render(l.widget(), buf);

    "selection".render(l.label(), buf);
    format!("{:?}", state.selection)
        .to_string()
        .render(l.widget(), buf);
}
