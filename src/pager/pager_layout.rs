use crate::layout::StructuredLayout;
use crate::pager::AreaHandle;
use ratatui::layout::Rect;
use ratatui::text::Span;
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
    core: Rc<RefCell<PagerLayoutCore>>,
}

#[derive(Debug, Default, Clone)]
struct PagerLayoutCore {
    layout: StructuredLayout,
    // calculated breaks
    breaks: Vec<u16>,
}

impl PagerLayout {
    /// New layout.
    pub fn new(stride: usize) -> Self {
        Self {
            core: Rc::new(RefCell::new(PagerLayoutCore {
                layout: StructuredLayout::new(stride),
                ..Default::default()
            })),
        }
    }

    /// New layout from StructuredLayout
    pub fn with_layout(layout: StructuredLayout) -> Self {
        Self {
            core: Rc::new(RefCell::new(PagerLayoutCore {
                layout,
                ..Default::default()
            })),
        }
    }

    /// Has the target width of the layout changed.
    pub fn width_changed(&self, width: u16) -> bool {
        self.core.borrow().layout.width_change(width)
    }

    /// Add a rect.
    pub fn add(&mut self, area: &[Rect]) -> AreaHandle {
        // reset page to re-layout
        self.core.borrow_mut().layout.set_area(Default::default());
        self.core.borrow_mut().layout.add(area)
    }

    /// Returns a Vec with all handles.
    pub fn handles(&self) -> Vec<AreaHandle> {
        self.core.borrow().layout.handles()
    }

    /// Get the layout area for the given handle
    pub fn layout_area(&self, handle: AreaHandle) -> Box<[Rect]> {
        self.core.borrow().layout[handle]
            .to_vec()
            .into_boxed_slice()
    }

    /// Get the label for the given handle.
    pub fn label(&self, handle: AreaHandle) -> Option<Span<'static>> {
        self.core.borrow().layout.label(handle)
    }

    /// Add a manual break after the given position.
    pub fn break_after(&mut self, y: u16) {
        // reset page to re-layout
        self.core.borrow_mut().layout.set_area(Default::default());
        self.core.borrow_mut().layout.break_after_row(y);
    }

    /// Add a manual break before the given position.
    pub fn break_before(&mut self, y: u16) {
        // reset page to re-layout
        self.core.borrow_mut().layout.set_area(Default::default());
        self.core.borrow_mut().layout.break_before_row(y);
    }

    /// Number of areas.
    pub fn len(&self) -> usize {
        self.core.borrow().layout.len()
    }

    /// Contains areas?
    pub fn is_empty(&self) -> bool {
        self.core.borrow().layout.is_empty()
    }

    /// Run the layout algorithm.
    pub fn layout(&mut self, page: Rect) {
        self.core.borrow_mut().layout(page);
    }

    /// Page area in layout coordinates
    pub fn page_area(&self) -> Rect {
        self.core.borrow().layout.area()
    }

    /// Number of pages after calculating the layout.
    pub fn num_pages(&self) -> usize {
        self.core.borrow().breaks.len()
    }

    /// First area-handle on the given page.
    pub fn first_on_page(&self, page: usize) -> Option<AreaHandle> {
        let core = self.core.borrow();

        let brk = core.breaks[page];

        let r = core
            .layout
            .chunked() //
            .enumerate()
            .find_map(|(i, v)| {
                if v.iter().find(|w| w.y >= brk).is_some() {
                    Some(AreaHandle(i))
                } else {
                    None
                }
            });

        r
    }

    /// Locate an area by handle.
    ///
    /// This will return a Rect with a y-value relative to the
    /// page it is in. But still in layout-coords.
    ///
    /// And it returns the page the Rect is on.
    pub fn buf_handle(&self, handle: AreaHandle) -> (usize, Box<[Rect]>) {
        let area = &self.core.borrow().layout[handle];
        self.buf_areas(area)
    }

    /// Locate an area.
    ///
    /// This will return a Rect with a y-value relative to the
    /// page it is in. But still in layout-coords.
    ///
    /// This will clip the bounds to the page area if not
    /// displayable otherwise.
    ///
    /// And it returns the page the Rect is on.
    pub fn buf_area(&self, area: Rect) -> (usize, Rect) {
        let tmp = self.buf_areas(&[area]);
        (tmp.0, tmp.1[0])
    }

    /// Locate the given areas on one page.
    ///
    /// The correct page for top-most area is used for all areas.
    ///
    /// This will return a Rect with a y-value relative to the
    /// page it is in. But still in layout-coords.
    ///
    /// This will clip the bounds to the page area.
    ///
    /// And it returns the page the Rect is on.
    fn buf_areas(&self, area: &[Rect]) -> (usize, Box<[Rect]>) {
        let core = self.core.borrow();

        let min_y = area.iter().map(|v| v.y).min().expect("array of rect");

        // find page
        let (page_nr, brk) = core
            .breaks
            .iter()
            .enumerate()
            .rev()
            .find(|(_i, v)| **v <= min_y)
            .expect("valid breaks");

        // clip to fit
        let clip_area = Rect::new(
            0, //
            0,
            core.layout.area().width,
            core.layout.area().height,
        );

        let mut res = Vec::new();
        for a in area.iter() {
            let r = Rect::new(
                a.x, //
                a.y - *brk,
                a.width,
                a.height,
            )
            .intersection(clip_area);
            res.push(r);
        }

        (page_nr, res.into_boxed_slice())
    }
}

impl From<StructuredLayout> for PagerLayout {
    fn from(value: StructuredLayout) -> Self {
        PagerLayout::with_layout(value)
    }
}

impl PagerLayoutCore {
    /// Run the layout algorithm.
    fn layout(&mut self, page: Rect) {
        if self.layout.area() == page {
            return;
        }
        self.layout.set_area(page);

        // must not change the order of the areas.
        // gave away handles ...
        let mut areas = self.layout.as_slice().to_vec();
        areas.sort_by(|a, b| a.y.cmp(&b.y));

        self.layout.sort_row_breaks_desc();
        self.breaks.clear();

        self.breaks.push(0);

        let mut last_break = 0;
        let mut man_breaks = self.layout.row_breaks().to_vec();

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

    fn hr(y: u16, height: u16) -> [Rect; 1] {
        [Rect::new(0, y, 0, height)]
    }

    #[test]
    fn test_layout() {
        let mut p0 = PagerLayout::new(1);

        p0.add(&hr(5, 1));
        p0.add(&hr(5, 2));
        p0.add(&hr(9, 1));
        p0.add(&hr(9, 2));
        p0.add(&hr(9, 1));
        p0.add(&hr(9, 0));
        p0.add(&hr(12, 1));
        p0.add(&hr(14, 1));
        p0.add(&hr(16, 1));
        p0.add(&hr(18, 1));
        p0.add(&hr(19, 1));
        p0.add(&hr(20, 1));

        p0.layout(Rect::new(0, 0, 0, 10));

        assert_eq!(p0.core.borrow().breaks.deref(), &vec![0, 9, 19]);
    }
}
