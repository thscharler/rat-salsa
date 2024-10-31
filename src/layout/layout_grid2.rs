//!
//! Calculate a layout-grid from horizontal + vertical Constraints.
//!

use crate::layout::StructuredLayout;
#[cfg(test)]
use ratatui::layout::Constraint;
use ratatui::layout::{Layout, Rect};

///
/// Calculates a full grid of rects from the horizontal and vertical components.
///
/// ```
/// use ratatui::layout::{Constraint, Layout, Rect};
/// use rat_widget::layout::layout_grid;
///
/// let area = Rect::new(0,0,100,100);
///
/// let layout = layout_grid::<3, 5>(area,
///             Layout::horizontal([
///                 Constraint::Length(5),
///                 Constraint::Fill(1),
///                 Constraint::Length(5)
///             ]),
///             Layout::vertical([
///                 Constraint::Length(1),
///                 Constraint::Length(3),
///                 Constraint::Fill(1),
///                 Constraint::Length(3),
///                 Constraint::Length(1),
///             ])
/// );
///
/// // middle column, second block
/// let a_1_2 = layout[1][2];
/// ```
pub fn layout_grid<const X: usize, const Y: usize>(
    area: Rect,
    horizontal: Layout,
    vertical: Layout,
) -> StructuredLayout {
    let mut l = StructuredLayout::new(Y);
    l.set_area(area);

    let hori = horizontal.areas::<X>(Rect::new(area.x, 0, area.width, 0));
    let vert = vertical.areas::<Y>(Rect::new(0, area.y, 0, area.height));

    for x in 0..X {
        let mut res = [Rect::default(); Y];
        for y in 0..Y {
            res[y].x = hori[x].x;
            res[y].width = hori[x].width;
            res[y].y = vert[y].y;
            res[y].height = vert[y].height;
        }
        _ = l.add(&res);
    }

    l
}

#[cfg(test)]
#[test]
fn test_grid() {
    let ll = layout_grid::<2, 3>(
        Rect::new(0, 0, 10, 10),
        Layout::horizontal([Constraint::Length(2), Constraint::Length(2)]),
        Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
        ]),
    );

    assert_eq!(ll[0][0], Rect::new(0, 0, 2, 2));
    assert_eq!(ll[0][1], Rect::new(0, 2, 2, 2));
    assert_eq!(ll[0][2], Rect::new(0, 4, 2, 2));
    assert_eq!(ll[1][0], Rect::new(2, 0, 2, 2));
    assert_eq!(ll[1][1], Rect::new(2, 2, 2, 2));
    assert_eq!(ll[1][2], Rect::new(2, 4, 2, 2));

    dbg!(ll);
}
