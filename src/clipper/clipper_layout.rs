use crate::clipper::AreaHandle;
use iset::IntervalSet;
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::cmp::{max, min};
use std::rc::Rc;

/// ClipperLayout holds all areas for the widgets that want
/// to be displayed.
///
/// It uses its own layout coordinates. The scroll offset is
/// in layout coordinates too.
///
#[derive(Debug, Default, Clone)]
pub struct ClipperLayout {
    core: Rc<RefCell<PageLayoutCore>>,
}

#[derive(Debug, Default, Clone)]
struct PageLayoutCore {
    // xview area in layout coordinates
    area: Rect,
    // extended xview area in layout coordinates
    ext_area: Rect,
    // collected areas
    areas: Vec<Rect>,
    // vertical ranges
    y_ranges: IntervalSet<u16>,
    // horizontal ranges
    x_ranges: IntervalSet<u16>,
}

impl ClipperLayout {
    /// New layout.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a layout area.
    pub fn add(&mut self, area: Rect) -> AreaHandle {
        let mut core = self.core.borrow_mut();
        // reset page to re-layout
        core.area = Default::default();
        core.areas.push(area);
        AreaHandle(core.areas.len() - 1)
    }

    /// Add layout area. Doesn't give you a handle.
    pub fn add_all(&mut self, areas: impl IntoIterator<Item = Rect>) {
        let mut core = self.core.borrow_mut();
        // reset page to re-layout
        core.area = Default::default();
        core.areas.extend(areas)
    }

    /// Add layout area. Appends the resulting handles.
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

    /// View/buffer area in layout coordinates
    pub fn buffer_area(&self) -> Rect {
        self.core.borrow().area
    }

    /// Extended xview/buffer area in layout coordinates.
    pub fn ext_buffer_area(&self) -> Rect {
        self.core.borrow().ext_area
    }

    /// Get the original area for the handle.
    pub fn layout_area_by_handle(&self, handle: AreaHandle) -> Rect {
        self.core.borrow().areas[handle.0]
    }

    /// Run the layout algorithm.
    ///
    /// Returns the extended area to render all visible widgets.
    /// The size of this area is the required size of the buffer.
    ///
    /// - page: in layout coordinates
    /// - ->: extended area in layout coordinates.
    pub fn layout(&mut self, page: Rect) -> Rect {
        let mut core = self.core.borrow_mut();
        core.layout(page)
    }

    /// Returns the bottom-right corner for the Layout.
    pub fn max_layout_pos(&self) -> (u16, u16) {
        let core = self.core.borrow();

        let x = core.x_ranges.largest().map(|v| v.end);
        let y = core.y_ranges.largest().map(|v| v.end);

        let x = x.unwrap_or(core.ext_area.right());
        let y = y.unwrap_or(core.ext_area.bottom());

        (x, y)
    }

    /// First visible area in buffer in layout coordinates.
    ///
    /// __Caution__
    /// Order is the order of addition, not necessarily the top-left area.
    pub fn first_layout_area(&self) -> Option<Rect> {
        let core = self.core.borrow();
        core.areas
            .iter()
            .find(|v| {
                core.ext_area.top() <= v.top()
                    && core.ext_area.bottom() >= v.bottom()
                    && core.ext_area.left() <= v.left()
                    && core.ext_area.right() >= v.right()
            })
            .cloned()
    }

    /// First visible area-handle.
    ///
    /// __Caution__
    /// Order is the order of addition, not necessarily the top-left area.
    pub fn first_layout_handle(&self) -> Option<AreaHandle> {
        let core = self.core.borrow();

        core.areas
            .iter()
            .enumerate()
            .find(|(_, v)| {
                core.ext_area.top() <= v.top()
                    && core.ext_area.bottom() >= v.bottom()
                    && core.ext_area.left() <= v.left()
                    && core.ext_area.right() >= v.right()
            })
            .map(|(idx, _)| AreaHandle(idx))
    }

    /// Converts the area behind the handle to buffer coordinates.
    /// This will return coordinates relative to the extended page.
    /// Or None.
    pub fn buf_area_by_handle(&self, handle: AreaHandle) -> Option<Rect> {
        let area = self.core.borrow().areas[handle.0];
        self.buf_area(area)
    }

    /// Converts the area behind the handle to buffer coordinates.
    /// This will return coordinates relative to the extended page.
    /// Or None.
    pub fn buf_area(&self, area: Rect) -> Option<Rect> {
        let core = self.core.borrow();

        let wide = core.ext_area;

        if core.ext_area.top() <= area.top()
            && core.ext_area.bottom() >= area.bottom()
            && core.ext_area.left() <= area.left()
            && core.ext_area.right() >= area.right()
        {
            Some(Rect::new(
                area.x - wide.x,
                area.y - wide.y,
                area.width,
                area.height,
            ))
        } else {
            None
        }
    }
}

impl PageLayoutCore {
    fn layout(&mut self, page: Rect) -> Rect {
        if self.area == page {
            return self.ext_area;
        }

        self.y_ranges.clear();
        self.x_ranges.clear();
        for v in self.areas.iter() {
            if v.height > 0 {
                self.y_ranges.insert(v.top()..v.bottom());
            }
            if v.width > 0 {
                self.x_ranges.insert(v.left()..v.right());
            }
        }

        self.area = page;

        // range that contains all widgets that are visible on the page.
        let y_range = self
            .y_ranges
            .iter(page.top()..page.bottom())
            .reduce(|a, b| min(a.start, b.start)..max(a.end, b.end));
        let x_range = self
            .x_ranges
            .iter(page.left()..page.right())
            .reduce(|a, b| min(a.start, b.start)..max(a.end, b.end));

        // default
        let y_range = y_range.unwrap_or(page.top()..page.bottom());
        let x_range = x_range.unwrap_or(page.left()..page.right());

        // page is the minimum
        let min_x = min(x_range.start, page.x);
        let min_y = min(y_range.start, page.y);
        let max_x = max(x_range.end, page.right());
        let max_y = max(y_range.end, page.bottom());

        self.ext_area = Rect::new(min_x, min_y, max_x - min_x, max_y - min_y);

        self.ext_area
    }
}
