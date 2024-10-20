//!
//! If you are tired of scrolling, try paging :)
//!
//! If you have a lot of widgets to display, splitting
//! them into pages is an alternative to scrolling.
//!
//! [PageLayout] helps with the dynamic page-breaks.
//! [SinglePager] and [DualPager] are the widgets that display
//! everything as one or two columns.
//!
//! Same as the other containers in this crate they leave the
//! actual rendering of the widgets to the caller.
//! [relocate](SinglePagerState::relocate) tells you
//! if a widget is visible and where it should be rendered.
//!

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Block;
use std::cell::RefCell;
use std::rc::Rc;

mod dual_pager;
mod single_pager;

use crate::_private::NonExhaustive;
pub use dual_pager::*;
pub use single_pager::*;

/// PageLayout is fed with the areas that should be displayed.
///
/// The areas must use widget relative coordinates not screen
/// coordinates.
///
/// It then splits the list in pages in a way that there are
/// no broken areas.
#[derive(Debug, Default, Clone)]
pub struct PageLayout {
    core: Rc<RefCell<PageLayoutCore>>,
}

/// Handle for an added area. Can be used to get the displayed area.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AreaHandle(usize);

#[derive(Debug, Default, Clone)]
struct PageLayoutCore {
    // just for checks on re-layout
    page: Rect,
    // collected areas
    areas: Vec<Rect>,
    // manual breaks
    man_breaks: Vec<u16>,
    // calculated breaks
    breaks: Vec<u16>,
}

impl PageLayout {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a rect.
    pub fn add(&mut self, area: Rect) -> AreaHandle {
        let mut core = self.core.borrow_mut();
        // reset page to re-layout
        core.page = Default::default();
        core.areas.push(area);
        AreaHandle(core.areas.len() - 1)
    }

    /// Add rects.
    pub fn add_all(&mut self, areas: impl IntoIterator<Item = Rect>) {
        let mut core = self.core.borrow_mut();
        // reset page to re-layout
        core.page = Default::default();
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
        core.page = Default::default();

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
        core.page = Default::default();

        core.man_breaks.push(y + 1);
    }

    /// Add a manual break before the given position.
    pub fn break_before(&mut self, y: u16) {
        let mut core = self.core.borrow_mut();

        // reset page to re-layout
        core.page = Default::default();

        core.man_breaks.push(y);
    }

    /// Run the layout algorithm.
    pub fn layout(&mut self, page: Rect) {
        let mut core = self.core.borrow_mut();
        core.layout(page);
    }

    /// Number of pages.
    ///
    pub fn len(&self) -> usize {
        self.core.borrow().breaks.len()
    }

    /// Any pages
    pub fn is_empty(&self) -> bool {
        self.core.borrow().breaks.is_empty()
    }

    /// Get the original area for the handle.
    pub fn area_by_handle(&self, handle: AreaHandle) -> Rect {
        self.core.borrow().areas[handle.0]
    }

    /// Locate an area by handle.
    ///
    /// This will return a Rect with a y-value relative to the
    /// page it is in. But still in layout-coords.
    ///
    /// And it returns the page the Rect is on.
    pub fn locate_handle(&self, handle: AreaHandle) -> (usize, Rect) {
        let area = self.core.borrow().areas[handle.0];
        self.locate(area)
    }

    /// Locate an area.
    ///
    /// This will return a Rect with a y-value relative to the
    /// page it is in. But still in layout-coords.
    ///
    /// And it returns the page the Rect is on.
    pub fn locate(&self, area: Rect) -> (usize, Rect) {
        let core = self.core.borrow();

        // find page
        let (page, brk) = core
            .breaks
            .iter()
            .enumerate()
            .rev()
            .find(|(_i, v)| **v <= area.y)
            .expect("valid breaks");

        (
            page,
            Rect::new(area.x, area.y - *brk, area.width, area.height),
        )
    }

    /// First area on the given page.
    pub fn first_area(&self, page: usize) -> Option<Rect> {
        let core = self.core.borrow();

        let brk = core.breaks[page];
        core.areas.iter().skip(1).find(|v| v.y >= brk).cloned()
    }

    /// First area-handle on the given page.
    pub fn first_handle(&self, page: usize) -> Option<AreaHandle> {
        let core = self.core.borrow();

        let brk = core.breaks[page];
        core.areas.iter().enumerate().skip(1).find_map(|(i, v)| {
            if v.y >= brk {
                Some(AreaHandle(i))
            } else {
                None
            }
        })
    }
}

impl PageLayoutCore {
    /// Run the layout algorithm.
    fn layout(&mut self, page: Rect) {
        if self.page == page {
            return;
        }

        self.areas.sort_by(|a, b| a.y.cmp(&b.y));
        self.man_breaks.sort_by(|a, b| b.cmp(a));
        self.breaks.clear();

        self.breaks.push(0);
        let mut last_break = 0;
        let mut man_breaks = self.man_breaks.clone();

        for v in self.areas.iter() {
            if Some(&v.y) == man_breaks.last() {
                self.breaks.push(v.y);
                last_break = v.y;
                man_breaks.pop();
            } else if v.y >= last_break {
                let ry = v.y - last_break;
                if ry + v.height > page.height {
                    self.breaks.push(v.y);
                    last_break = v.y;
                }
            }
        }
    }
}

/// All styles for a pager.
#[derive(Debug, Clone)]
pub struct PagerStyle {
    pub style: Style,
    pub nav: Option<Style>,
    pub title: Option<Style>,
    pub block: Option<Block<'static>>,
    pub non_exhaustive: NonExhaustive,
}

impl Default for PagerStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            nav: None,
            title: None,
            block: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

pub(crate) mod event {
    use rat_event::{ConsumedEvent, Outcome};

    /// Result of event handling.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum PagerOutcome {
        /// The given event has not been used at all.
        Continue,
        /// The event has been recognized, but the result was nil.
        /// Further processing for this event may stop.
        Unchanged,
        /// The event has been recognized and there is some change
        /// due to it.
        /// Further processing for this event may stop.
        /// Rendering the ui is advised.
        Changed,
        /// Displayed page changed.
        Page(usize),
    }

    impl ConsumedEvent for PagerOutcome {
        fn is_consumed(&self) -> bool {
            *self != PagerOutcome::Continue
        }
    }

    // Useful for converting most navigation/edit results.
    impl From<bool> for PagerOutcome {
        fn from(value: bool) -> Self {
            if value {
                PagerOutcome::Changed
            } else {
                PagerOutcome::Unchanged
            }
        }
    }

    impl From<Outcome> for PagerOutcome {
        fn from(value: Outcome) -> Self {
            match value {
                Outcome::Continue => PagerOutcome::Continue,
                Outcome::Unchanged => PagerOutcome::Unchanged,
                Outcome::Changed => PagerOutcome::Changed,
            }
        }
    }

    impl From<PagerOutcome> for Outcome {
        fn from(value: PagerOutcome) -> Self {
            match value {
                PagerOutcome::Continue => Outcome::Continue,
                PagerOutcome::Unchanged => Outcome::Unchanged,
                PagerOutcome::Changed => Outcome::Changed,
                PagerOutcome::Page(_) => Outcome::Changed,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::pager::PageLayout;
    use ratatui::layout::Rect;
    use std::ops::Deref;

    fn hr(y: u16, height: u16) -> Rect {
        Rect::new(0, y, 0, height)
    }

    #[test]
    fn test_layout() {
        let mut p0 = PageLayout::new();

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
