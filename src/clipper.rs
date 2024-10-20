use crate::_private::NonExhaustive;
///
/// An alternative view widget.
///
/// This works similar to [SinglePager], as it depends on a Layout
/// that contains all areas that can be rendered.
///
/// Dependent on the scroll offset a relocated area will be produced
/// by the widget where the actual inner widget can be drawn.
/// If the inner widget is off-screen a relocated area of `None`
/// indicates that a widget need not be drawn.
///
/// __Info__
/// Due to limitations in ratatui widgets cannot be displayed partially
/// off-screen to the left and top. Right and bottom work fine.
///
/// __See__
/// [example](https://github.com/thscharler/rat-widget/blob/master/examples/clipper1.rs)
///
use iset::IntervalSet;
use rat_event::util::MouseFlagsN;
use rat_event::{HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::ContainerFlag;
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::{Scroll, ScrollArea, ScrollAreaState, ScrollState, ScrollStyle};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::widgets::{Block, StatefulWidget};
use std::cell::RefCell;
use std::cmp::{max, min};
use std::rc::Rc;

/// PageLayout is fed with the areas that should be displayed.
///
/// The areas used here are widget relative, not screen coordinates.
/// It will keep track of the currently displayed view.
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
    // buffer
    wide_page: Rect,
    // collected areas
    areas: Vec<Rect>,
    // vertical ranges
    y_ranges: IntervalSet<u16>,
    // horizontal ranges
    x_ranges: IntervalSet<u16>,
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

    /// Add rects. Doesn't give you a rect-handle.
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

    /// Run the layout algorithm.
    ///
    /// Returns the required area to render all visible widgets.
    /// This can lead to a corrected render offset, and the size
    /// is used for the temp buffer.
    pub fn layout(&mut self, page: Rect) -> Rect {
        let mut core = self.core.borrow_mut();
        core.layout(page)
    }

    /// Returns the bottom-right most coordinates.
    pub fn max_position(&self) -> (u16, u16) {
        let core = self.core.borrow();

        let x = core.x_ranges.largest().map(|v| v.end);
        let y = core.y_ranges.largest().map(|v| v.end);

        let x = x.unwrap_or(core.wide_page.right());
        let y = y.unwrap_or(core.wide_page.bottom());

        (x, y)
    }

    /// Get the original area for the handle.
    pub fn area_by_handle(&self, handle: AreaHandle) -> Rect {
        self.core.borrow().areas[handle.0]
    }

    /// Locate an area by handle.
    ///
    /// This will return a Rect if it is displayed or None if not.
    pub fn locate_handle(&self, handle: AreaHandle) -> Option<Rect> {
        let area = self.core.borrow().areas[handle.0];
        self.locate(area)
    }

    /// Locate an area.
    ///
    /// This will return a Rect32 relative to the current visible
    /// area, or None if it's outside.
    pub fn locate(&self, area: Rect) -> Option<Rect> {
        let core = self.core.borrow();

        let wide = core.wide_page;

        if core.wide_page.top() <= area.top()
            && core.wide_page.bottom() >= area.bottom()
            && core.wide_page.left() <= area.left()
            && core.wide_page.right() >= area.right()
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

    /// First visible area.
    ///
    /// __Caution__
    /// Order is the order of addition, not necessarily the top-left area.
    pub fn first_area(&self) -> Option<Rect> {
        let core = self.core.borrow();
        core.areas
            .iter()
            .find(|v| {
                core.wide_page.top() <= v.top()
                    && core.wide_page.bottom() >= v.bottom()
                    && core.wide_page.left() <= v.left()
                    && core.wide_page.right() >= v.right()
            })
            .cloned()
    }

    /// First visible area-handle.
    ///
    /// Order is the order of addition, not necessarily the top-left area.
    pub fn first_handle(&self) -> Option<AreaHandle> {
        let core = self.core.borrow();

        core.areas
            .iter()
            .enumerate()
            .find(|(_, v)| {
                core.wide_page.top() <= v.top()
                    && core.wide_page.bottom() >= v.bottom()
                    && core.wide_page.left() <= v.left()
                    && core.wide_page.right() >= v.right()
            })
            .map(|(idx, _)| AreaHandle(idx))
    }
}

impl PageLayoutCore {
    fn layout(&mut self, page: Rect) -> Rect {
        if self.page == page {
            return self.wide_page;
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

        self.page = page;

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

        self.wide_page = Rect::new(min_x, min_y, max_x - min_x, max_y - min_y);

        self.wide_page
    }
}

// -------------------------------------------------------------

/// Construct a Clipper.
///
/// This
///
#[derive(Debug, Default, Clone)]
pub struct Clipper<'a> {
    layout: PageLayout,
    block: Option<Block<'a>>,
    hscroll: Option<Scroll<'a>>,
    vscroll: Option<Scroll<'a>>,
}

/// Render the buffer.
#[derive(Debug)]
pub struct ClipperRender<'a> {
    inner: Rect,
    layout: PageLayout,
    buffer: Buffer,
    block: Option<Block<'a>>,
    hscroll: Option<Scroll<'a>>,
    vscroll: Option<Scroll<'a>>,
}

/// All styles for a clipper.
#[derive(Debug, Clone)]
pub struct ClipperStyle {
    pub block: Option<Block<'static>>,
    pub scroll: Option<ScrollStyle>,
    pub non_exhaustive: NonExhaustive,
}

impl Default for ClipperStyle {
    fn default() -> Self {
        Self {
            block: None,
            scroll: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ClipperState {
    /// Full area.
    /// __read only__ renewed with each render.
    pub area: Rect,
    /// Area for widgets to render.
    /// __read only__ renewed with each render.
    pub inner: Rect,

    /// Page layout.
    /// __read only__ renewed with each render.
    pub layout: PageLayout,

    /// Horizontal scroll
    /// __read+write__
    pub hscroll: ScrollState,
    /// Vertical scroll
    /// __read+write__
    pub vscroll: ScrollState,

    /// This widget has no focus of its own, but this flag
    /// can be used to set a container state.
    pub c_focus: ContainerFlag,

    /// Mouse util.
    pub mouse: MouseFlagsN,

    // only for reuse with the next render.
    buffer: Option<Buffer>,
}

impl<'a> Clipper<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Page layout.
    pub fn layout(mut self, page_layout: PageLayout) -> Self {
        self.layout = page_layout;
        self
    }

    /// Block for border
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Scroll support.
    pub fn scroll(mut self, scroll: Scroll<'a>) -> Self {
        self.hscroll = Some(scroll.clone().override_horizontal());
        self.vscroll = Some(scroll.override_vertical());
        self
    }

    /// Horizontal scroll support.
    pub fn hscroll(mut self, scroll: Scroll<'a>) -> Self {
        self.hscroll = Some(scroll.override_horizontal());
        self
    }

    /// Vertical scroll support.
    pub fn vscroll(mut self, scroll: Scroll<'a>) -> Self {
        self.vscroll = Some(scroll.override_vertical());
        self
    }

    /// Set all styles
    pub fn styles(mut self, styles: ClipperStyle) -> Self {
        if let Some(block) = styles.block {
            self.block = Some(block);
        }
        self.hscroll = if let Some(hscroll) = self.hscroll {
            if let Some(styles) = styles.scroll.clone() {
                Some(hscroll.styles(styles))
            } else {
                Some(hscroll)
            }
        } else {
            None
        };
        self.vscroll = if let Some(vscroll) = self.vscroll {
            if let Some(styles) = styles.scroll.clone() {
                Some(vscroll.styles(styles))
            } else {
                Some(vscroll)
            }
        } else {
            None
        };
        self
    }

    /// Run the layout and create the final ClipperRender widget.
    pub fn into_widget(self, area: Rect, state: &mut ClipperState) -> ClipperRender<'a> {
        let sa = ScrollArea::new()
            .block(self.block.as_ref())
            .h_scroll(self.hscroll.as_ref())
            .v_scroll(self.vscroll.as_ref());

        state.area = area;
        state.inner = sa.inner(area, Some(&state.hscroll), Some(&state.vscroll));
        state.layout = self.layout;

        // run the layout
        let corrected = state.layout.layout(Rect::new(
            state.hscroll.offset as u16,
            state.vscroll.offset as u16,
            state.inner.width,
            state.inner.height,
        ));

        // adjust scroll
        // can only display complete widgets to the left/top.
        state.hscroll.offset = corrected.x as usize;
        state.vscroll.offset = corrected.y as usize;
        let max = state.layout.max_position();
        state.vscroll.set_page_len(state.inner.height as usize);
        state
            .vscroll
            .set_max_offset(max.1.saturating_sub(state.inner.height) as usize);

        state.hscroll.set_page_len(state.inner.width as usize);
        state
            .hscroll
            .set_max_offset(max.0.saturating_sub(state.inner.width) as usize);

        // resize buffer to fit all visible widgets.
        let buf_area = Rect::new(
            state.inner.x,
            state.inner.y,
            corrected.width,
            corrected.height,
        );
        if let Some(buffer) = &mut state.buffer {
            buffer.reset();
            buffer.resize(buf_area);
        } else {
            state.buffer = Some(Buffer::empty(buf_area));
        }

        ClipperRender {
            inner: state.inner,
            layout: state.layout.clone(),
            buffer: state.buffer.take().expect("valid buffer"),
            block: self.block,
            hscroll: self.hscroll,
            vscroll: self.vscroll,
        }
    }
}

impl<'a> ClipperRender<'a> {
    /// Relocate a view area to a screen area if it is visible.
    pub fn relocate(&self, area: impl Into<Rect>) -> Option<Rect> {
        if let Some(area) = self.layout.locate(area.into()) {
            Some(Rect::new(
                area.x + self.inner.x,
                area.y + self.inner.y,
                area.width,
                area.height,
            ))
        } else {
            None
        }
    }

    /// Relocate a view area to a screen area if it is visible.
    pub fn relocate_handle(&self, handle: AreaHandle) -> Option<Rect> {
        if let Some(area) = self.layout.locate_handle(handle) {
            Some(Rect::new(
                area.x + self.inner.x,
                area.y + self.inner.y,
                area.width,
                area.height,
            ))
        } else {
            None
        }
    }

    /// Access the temporary buffer.
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }
}

impl<'a> StatefulWidget for ClipperRender<'a> {
    type State = ClipperState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let sa = ScrollArea::new()
            .block(self.block.as_ref())
            .h_scroll(self.hscroll.as_ref())
            .v_scroll(self.vscroll.as_ref());
        sa.render(
            area,
            buf,
            &mut ScrollAreaState::new()
                .h_scroll(&mut state.hscroll)
                .v_scroll(&mut state.vscroll),
        );

        for y in state.inner.top()..state.inner.bottom() {
            for x in state.inner.left()..state.inner.right() {
                let cell = self.buffer.cell(Position::new(x, y)).expect("cell");
                if let Some(scr_cell) = buf.cell_mut(Position::new(x, y)) {
                    *scr_cell = cell.clone();
                }
            }
        }

        // keep buffer
        state.buffer = Some(self.buffer);
    }
}

impl ClipperState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the page for this rect.
    pub fn show_handle(&mut self, handle: AreaHandle) {
        let area = self.layout.area_by_handle(handle);
        self.hscroll.scroll_to_pos(area.x as usize);
        self.vscroll.scroll_to_pos(area.y as usize);
    }

    /// Show this rect.
    pub fn show_area(&mut self, area: Rect) {
        self.hscroll.scroll_to_pos(area.x as usize);
        self.vscroll.scroll_to_pos(area.y as usize);
    }

    /// First currently visible area.
    pub fn first_area(&self) -> Option<Rect> {
        self.layout.first_area()
    }

    /// First handle for the page.
    /// This returns the first handle for the page.
    /// Does not check whether the connected area is visible.
    pub fn first_handle(&self) -> Option<AreaHandle> {
        self.layout.first_handle()
    }
}

impl HandleEvent<crossterm::event::Event, Regular, Outcome> for ClipperState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: Regular) -> Outcome {
        self.handle(event, MouseOnly)
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for ClipperState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        let mut sas = ScrollAreaState::new()
            .area(self.inner)
            .h_scroll(&mut self.hscroll)
            .v_scroll(&mut self.vscroll);
        match sas.handle(event, MouseOnly) {
            ScrollOutcome::Up(v) => self.vscroll.scroll_up(v).into(),
            ScrollOutcome::Down(v) => self.vscroll.scroll_down(v).into(),
            ScrollOutcome::VPos(v) => self.vscroll.scroll_to_pos(v).into(),
            ScrollOutcome::Left(v) => self.hscroll.scroll_left(v).into(),
            ScrollOutcome::Right(v) => self.hscroll.scroll_right(v).into(),
            ScrollOutcome::HPos(v) => self.hscroll.scroll_to_pos(v).into(),
            r => r.into(),
        }
    }
}
