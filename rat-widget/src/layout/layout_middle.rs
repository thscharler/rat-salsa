use ratatui_core::layout::{Constraint, Layout, Rect};

///
/// Calculate the middle Rect inside a given area.
///
pub fn layout_middle(
    area: Rect,
    left: Constraint,
    right: Constraint,
    top: Constraint,
    bottom: Constraint,
) -> Rect {
    let h_layout = Layout::horizontal([
        left, //
        Constraint::Fill(1),
        right,
    ])
    .split(area);
    let v_layout = Layout::vertical([
        top, //
        Constraint::Fill(1),
        bottom,
    ])
    .split(h_layout[1]);
    v_layout[1]
}
