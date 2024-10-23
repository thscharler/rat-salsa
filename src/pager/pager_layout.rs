use crate::pager::AreaHandle;
use log::debug;
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::rc::Rc;

/// PagerLayout holds all areas for the widgets that want to be
/// displayed.
///
/// It uses its own layout coordinates.
///
/// The layout step breaks this list into pages that can fit the
/// widgets. If your widget is too big to fit in the page area it
/// will be placed at a new page and will be clipped into shape.
///
#[derive(Debug, Default, Clone)]
pub struct PagerLayout {
    core: Rc<RefCell<PageLayoutCore>>,
}

#[derive(Debug, Default, Clone)]
struct PageLayoutCore {
    // just for checks on re-layout
    area: Rect,
    // collected areas
    areas: Vec<Rect>,
    // manual breaks
    man_breaks: Vec<u16>,
    // calculated breaks
    breaks: Vec<u16>,
}

impl PagerLayout {
    /// New layout.
    pub fn new() -> Self {
        Self::default()
    }

    /// Has the target width of the layout changed.
    pub fn width_changed(&self, width: u16) -> bool {
        self.core.borrow().area.width != width
    }

    /// Add a rect.
    pub fn add(&mut self, area: Rect) -> AreaHandle {
        let mut core = self.core.borrow_mut();
        // reset page to re-layout
        core.area = Default::default();
        core.areas.push(area);
        AreaHandle(core.areas.len() - 1)
    }

    /// Add rects.
    pub fn add_all(&mut self, areas: impl IntoIterator<Item = Rect>) {
        let mut core = self.core.borrow_mut();
        // reset page to re-layout
        core.area = Default::default();
        core.areas.extend(areas)
    }

    /// Add rects. Appends the resulting handles.
    pub fn add_all_out(
        &mut self,
        areas: impl IntoIterator<Item = Rect>,
        handles: &mut Vec<AreaHandle>,
    ) {
        let mut core = self.core.borrow_mut();

        // reset page to re-layout
        core.area = Default::default();

        let start = core.areas.len();
        core.areas.extend(areas);
        let end = core.areas.len();

        for i in start..end {
            handles.push(AreaHandle(i));
        }
    }

    /// Add a manual break after the given position.
    pub fn break_after(&mut self, y: u16) {
        let mut core = self.core.borrow_mut();

        // reset page to re-layout
        core.area = Default::default();

        core.man_breaks.push(y + 1);
    }

    /// Add a manual break before the given position.
    pub fn break_before(&mut self, y: u16) {
        let mut core = self.core.borrow_mut();

        // reset page to re-layout
        core.area = Default::default();

        core.man_breaks.push(y);
    }

    /// View/buffer area in layout coordinates
    pub fn buffer_area(&self) -> Rect {
        self.core.borrow().area
    }

    /// Get the original area for the handle.
    pub fn layout_area_by_handle(&self, handle: AreaHandle) -> Rect {
        self.core.borrow().areas[handle.0]
    }

    /// Number of areas.
    pub fn len(&self) -> usize {
        self.core.borrow().areas.len()
    }

    /// Contains areas?
    pub fn is_empty(&self) -> bool {
        self.core.borrow().areas.is_empty()
    }

    /// Run the layout algorithm.
    pub fn layout(&mut self, page: Rect) {
        let mut core = self.core.borrow_mut();
        core.layout(page);
    }

    /// Number of pages.
    ///
    pub fn num_pages(&self) -> usize {
        self.core.borrow().breaks.len()
    }

    /// First area on the given page.
    pub fn first_layout_area(&self, page: usize) -> Option<Rect> {
        let core = self.core.borrow();

        let brk = core.breaks[page];
        core.areas.iter().find(|v| v.y >= brk).cloned()
    }

    /// First area-handle on the given page.
    pub fn first_layout_handle(&self, page: usize) -> Option<AreaHandle> {
        let core = self.core.borrow();

        let brk = core.breaks[page];
        core.areas.iter().enumerate().find_map(|(i, v)| {
            if v.y >= brk {
                Some(AreaHandle(i))
            } else {
                None
            }
        })
    }

    /// Locate an area by handle.
    ///
    /// This will return a Rect with a y-value relative to the
    /// page it is in. But still in layout-coords.
    ///
    /// And it returns the page the Rect is on.
    pub fn buf_area_by_handle(&self, handle: AreaHandle) -> (usize, Rect) {
        let area = self.core.borrow().areas[handle.0];
        self.buf_area(area)
    }

    /// Locate an area.
    ///
    /// This will return a Rect with a y-value relative to the
    /// page it is in. But still in layout-coords.
    ///
    /// This will clip the bounds to the page area.
    ///
    /// And it returns the page the Rect is on.
    pub fn buf_area(&self, area: Rect) -> (usize, Rect) {
        let core = self.core.borrow();

        // find page
        let (page_nr, brk) = core
            .breaks
            .iter()
            .enumerate()
            .rev()
            .find(|(_i, v)| **v <= area.y)
            .expect("valid breaks");
        let relocated = Rect::new(area.x, area.y - *brk, area.width, area.height);

        // clip to fit
        let clip_area = Rect::new(0, 0, core.area.width, core.area.height);

        (page_nr, relocated.intersection(clip_area))
    }
}

impl PageLayoutCore {
    /// Run the layout algorithm.
    fn layout(&mut self, page: Rect) {
        if self.area == page {
            return;
        }
        self.area = page;

        // must not change the order of the areas.
        // gave away handles ...
        let mut areas = self.areas.clone();
        areas.sort_by(|a, b| a.y.cmp(&b.y));

        self.man_breaks.sort_by(|a, b| b.cmp(a));
        self.man_breaks.dedup();

        self.breaks.clear();

        self.breaks.push(0);
        let mut last_break = 0;
        let mut man_breaks = self.man_breaks.clone();

        for v in areas.iter() {
            if let Some(brk_y) = man_breaks.last() {
                if v.y >= *brk_y {
                    // don't break at the breaks.
                    // start the new page with a fresh widget :)
                    self.breaks.push(v.y);
                    last_break = v.y;
                    man_breaks.pop();
                }
            }

            if v.y > last_break {
                let ry = v.y - last_break;
                if ry + v.height > page.height {
                    self.breaks.push(v.y);
                    last_break = v.y;
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::pager::PagerLayout;
    use ratatui::layout::Rect;
    use std::ops::Deref;

    fn hr(y: u16, height: u16) -> Rect {
        Rect::new(0, y, 0, height)
    }

    #[test]
    fn test_layout() {
        let mut p0 = PagerLayout::new();

        p0.add(hr(5, 1));
        p0.add(hr(5, 2));
        p0.add(hr(9, 1));
        p0.add(hr(9, 2));
        p0.add(hr(9, 1));
        p0.add(hr(9, 0));
        p0.add(hr(12, 1));
        p0.add(hr(14, 1));
        p0.add(hr(16, 1));
        p0.add(hr(18, 1));
        p0.add(hr(19, 1));
        p0.add(hr(20, 1));

        p0.layout(hr(0, 10));

        assert_eq!(p0.core.borrow().breaks.deref(), &vec![0, 9, 19]);
    }
}
