use crate::layout::GenericLayout;
use ratatui::layout::{Layout, Rect, Size};

pub fn layout_grid<const X: usize, const Y: usize>(
    area: Size,
    horizontal: Layout,
    vertical: Layout,
) -> GenericLayout<(usize, usize)> {
    let mut gen_layout = GenericLayout::new();
    gen_layout.set_page_size(area);

    let hori = horizontal.areas::<X>(Rect::new(0, 0, area.width, 0));
    let vert = vertical.areas::<Y>(Rect::new(0, 0, 0, area.height));

    for x in 0..X {
        for y in 0..Y {
            let grid_area = Rect::new(hori[x].x, vert[y].y, hori[x].width, vert[y].height);
            gen_layout.add((x, y), grid_area, None, Rect::default());
        }
    }

    gen_layout
}
