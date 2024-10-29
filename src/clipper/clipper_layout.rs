use crate::clipper::AreaHandle;
use crate::layout::StructuredLayout;
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
    core: Rc<RefCell<ClipperLayoutCore>>,
}

#[derive(Debug, Default, Clone)]
struct ClipperLayoutCore {
    layout: StructuredLayout,
    // extended view area in layout coordinates
    ext_area: Rect,
    // vertical ranges
    y_ranges: IntervalSet<u16>,
    // horizontal ranges
    x_ranges: IntervalSet<u16>,
}

impl ClipperLayout {
    /// New layout.
    pub fn new(stride: usize) -> Self {
        Self {
            core: Rc::new(RefCell::new(ClipperLayoutCore {
                layout: StructuredLayout::new(stride),
                ..Default::default()
            })),
        }
    }

    /// New layout from StructuredLayout
    pub fn with_layout(layout: StructuredLayout) -> Self {
        Self {
            core: Rc::new(RefCell::new(ClipperLayoutCore {
                layout,
                ..Default::default()
            })),
        }
    }

    /// Has the target width of the layout changed.
    ///
    /// This is helpful if you only want vertical scrolling, and
    /// build your layout to fit.
    pub fn width_changed(&self, width: u16) -> bool {
        self.core.borrow().layout.width_change(width)
    }

    /// Add a layout area.
    pub fn add(&mut self, area: &[Rect]) -> AreaHandle {
        // reset page to re-layout
        self.core.borrow_mut().layout.set_area(Default::default());
        self.core.borrow_mut().layout.add(area)
    }

    /// Get the layout area for the given handle
    pub fn layout_handle(&self, handle: AreaHandle) -> Box<[Rect]> {
        self.core.borrow().layout[handle]
            .to_vec()
            .into_boxed_slice()
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

    /// Page area in layout coordinates
    pub fn page_area(&self) -> Rect {
        self.core.borrow().layout.area()
    }

    /// Extended page area in layout coordinates.
    /// This area is at least as large as page_area()
    /// and has enough space for partially visible widgets.
    pub fn ext_page_area(&self) -> Rect {
        self.core.borrow().ext_area
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
    pub fn first_layout_area(&self) -> Option<Box<[Rect]>> {
        let core = self.core.borrow();

        let r = core
            .layout
            .chunked()
            .find(|v| {
                v.iter()
                    .find(|w| {
                        core.ext_area.top() <= w.top()
                            && core.ext_area.bottom() >= w.bottom()
                            && core.ext_area.left() <= w.left()
                            && core.ext_area.right() >= w.right()
                    })
                    .is_some()
            })
            .map(|v| v.to_vec().into_boxed_slice());

        r
    }

    /// First visible area-handle.
    ///
    /// __Caution__
    /// Order is the order of addition, not necessarily the top-left area.
    pub fn first_layout_handle(&self) -> Option<AreaHandle> {
        let core = self.core.borrow();

        let r = core
            .layout
            .chunked() //
            .enumerate()
            .find_map(|(i, v)| {
                if v.iter()
                    .find(|w| {
                        core.ext_area.top() <= w.top()
                            && core.ext_area.bottom() >= w.bottom()
                            && core.ext_area.left() <= w.left()
                            && core.ext_area.right() >= w.right()
                    })
                    .is_some()
                {
                    Some(AreaHandle(i))
                } else {
                    None
                }
            });

        r
    }

    /// Converts the areas behind the handle to buffer coordinates.
    /// This will return coordinates relative to the extended page.
    /// Or None.
    pub fn buf_handle(&self, handle: AreaHandle) -> Box<[Rect]> {
        let area = &self.core.borrow().layout[handle];
        self.buf_areas(area)
    }

    /// Converts the areas to buffer coordinates.
    /// This will return coordinates relative to the extended page,
    /// or Rect::ZERO if clipped.
    fn buf_areas(&self, areas: &[Rect]) -> Box<[Rect]> {
        let core = self.core.borrow();

        let mut res = Vec::new();
        for area in areas {
            if core.ext_area.top() <= area.top()
                && core.ext_area.bottom() >= area.bottom()
                && core.ext_area.left() <= area.left()
                && core.ext_area.right() >= area.right()
            {
                res.push(Rect::new(
                    area.x - core.ext_area.x,
                    area.y - core.ext_area.y,
                    area.width,
                    area.height,
                ));
            } else {
                res.push(Rect::ZERO);
            }
        }

        res.into_boxed_slice()
    }

    /// Converts the layout area to buffer coordinates and clips to the
    /// buffer area.
    pub fn buf_area(&self, area: Rect) -> Rect {
        let core = self.core.borrow();

        if core.ext_area.top() <= area.top()
            && core.ext_area.bottom() >= area.bottom()
            && core.ext_area.left() <= area.left()
            && core.ext_area.right() >= area.right()
        {
            Rect::new(
                area.x - core.ext_area.x,
                area.y - core.ext_area.y,
                area.width,
                area.height,
            )
        } else {
            Rect::ZERO
        }
    }
}

impl ClipperLayoutCore {
    /// Run the layout algorithm.
    fn layout(&mut self, page: Rect) -> Rect {
        if self.layout.area() == page {
            return self.ext_area;
        }

        self.y_ranges.clear();
        self.x_ranges.clear();
        for v in self.layout.iter() {
            if v.height > 0 {
                self.y_ranges.insert(v.top()..v.bottom());
            }
            if v.width > 0 {
                self.x_ranges.insert(v.left()..v.right());
            }
        }

        self.layout.set_area(page);

        if self.layout.area().is_empty() {
            self.ext_area = self.layout.area();
            return self.ext_area;
        }

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
