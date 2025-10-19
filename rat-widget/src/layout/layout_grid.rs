use crate::layout::GenericLayout;
use ratatui::layout::{Constraint, Layout, Rect};

///
/// Calculates a full grid of rects from the horizontal and vertical components.
///
/// ```
/// use ratatui::layout::{Constraint, Layout, Rect};
/// use rat_widget::layout::layout_as_grid;
///
/// let area = Rect::new(0,0,100,100);
///
/// let layout = layout_as_grid(area,
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
/// let a_1_2 = layout.widget(layout.try_index_of((1,2)).expect("fine"));
/// ```
pub fn layout_as_grid(
    area: Rect,
    horizontal: Layout,
    vertical: Layout,
) -> GenericLayout<(usize, usize)> {
    let mut gen_layout = GenericLayout::new();
    gen_layout.set_area(area);
    gen_layout.set_page_size(area.as_size());

    let hori = horizontal.split(Rect::new(area.x, 0, area.width, 0));
    let vert = vertical.split(Rect::new(0, area.y, 0, area.height));

    for x in 0..hori.len() {
        for y in 0..vert.len() {
            let grid_area = Rect::new(hori[x].x, vert[y].y, hori[x].width, vert[y].height);
            gen_layout.add((x, y), grid_area, None, Rect::default());
        }
    }

    gen_layout
}

/// Create a basic grid of areas using the given Constraints.
pub fn simple_grid<const X: usize, const Y: usize>(
    area: Rect,
    horizontal: [Constraint; X],
    vertical: [Constraint; Y],
) -> [[Rect; Y]; X] {
    let mut layout = [[Rect::default(); Y]; X];

    let hori = Layout::horizontal(horizontal).split(Rect::new(area.x, 0, area.width, 0));
    let vert = Layout::vertical(vertical).split(Rect::new(0, area.y, 0, area.height));

    for x in 0..X {
        for y in 0..Y {
            let grid_area = Rect::new(hori[x].x, vert[y].y, hori[x].width, vert[y].height);
            layout[x][y] = grid_area;
        }
    }

    layout
}
