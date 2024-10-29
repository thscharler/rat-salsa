//!
//! Calculate a layout-grid from horizontal + vertical Constraints.
//!

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
) -> [[Rect; Y]; X] {
    let hori = horizontal.split(Rect::new(area.x, 0, area.width, 0));
    let vert = vertical.split(Rect::new(0, area.y, 0, area.height));

    let mut res = [[Rect::default(); Y]; X];
    for x in 0..X {
        let coldata = &mut res[x];
        for y in 0..Y {
            coldata[y].x = hori[x].x;
            coldata[y].width = hori[x].width;
            coldata[y].y = vert[y].y;
            coldata[y].height = vert[y].height;
        }
    }

    res
}
