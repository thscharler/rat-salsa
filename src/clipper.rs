use crate::_private::NonExhaustive;
use crate::relocate::{relocate_area, RelocatableState};
use crate::util::rect_dbg;
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
use log::debug;
use rat_event::util::MouseFlagsN;
use rat_event::{HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::ContainerFlag;
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::{Scroll, ScrollArea, ScrollAreaState, ScrollState, ScrollStyle};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect, Size};
use ratatui::widgets::{Block, StatefulWidget, Widget};
use std::cell::RefCell;
use std::cmp::{max, min};
use std::rc::Rc;
use std::time::SystemTime;

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
    // just for checks on re-layout.
    page: Rect,
    // actual view size and position, internal coordinates.
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

    /// page rect in layout coordinates
    pub fn page(&self) -> Rect {
        self.core.borrow().page
    }

    /// extended page rect in layout coordinates.
    pub fn wide_page(&self) -> Rect {
        self.core.borrow().wide_page
    }

    /// Run the layout algorithm.
    ///
    /// Returns the required area to render all visible widgets.
    /// This can lead to a corrected render offset, and the size
    /// is used for the temp buffer.
    ///
    /// page: in layout coordinates
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
    /// This will return a Rect relative to the current visible
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
pub struct ClipperBuffer<'a> {
    // page layout
    layout: PageLayout,

    // Scroll offset into the view.
    buf_offset_x: u16,
    buf_offset_y: u16,
    // buffer for rendering a single widget.
    buffer: Buffer,

    // inner area that will finally be rendered.
    widget_area: Rect,

    block: Option<Block<'a>>,
    hscroll: Option<Scroll<'a>>,
    vscroll: Option<Scroll<'a>>,
}

/// Rendering widget for View.
#[derive(Debug, Clone)]
pub struct ClipperWidget<'a> {
    // Scroll offset into the view.
    buf_offset_x: u16,
    buf_offset_y: u16,
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
    pub widget_area: Rect,

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

    /// Combined style.
    pub fn styles(mut self, styles: ClipperStyle) -> Self {
        if styles.block.is_some() {
            self.block = styles.block;
        }
        if let Some(styles) = styles.scroll {
            self.hscroll = self.hscroll.map(|v| v.styles(styles.clone()));
            self.vscroll = self.vscroll.map(|v| v.styles(styles.clone()));
        }
        self
    }

    /// Calculate the view area.
    pub fn inner(&self, area: Rect, state: &ClipperState) -> Rect {
        let sa = ScrollArea::new()
            .block(self.block.as_ref())
            .h_scroll(self.hscroll.as_ref())
            .v_scroll(self.vscroll.as_ref());
        sa.inner(area, Some(&state.hscroll), Some(&state.vscroll))
    }

    /// Calculates the layout and a temporary buffer.
    pub fn into_buffer(self, area: Rect, state: &mut ClipperState) -> ClipperBuffer<'a> {
        state.area = area;
        state.layout = self.layout;

        let sa = ScrollArea::new()
            .block(self.block.as_ref())
            .h_scroll(self.hscroll.as_ref())
            .v_scroll(self.vscroll.as_ref());
        state.widget_area = sa.inner(area, Some(&state.hscroll), Some(&state.vscroll));

        // run the layout
        let ext_area = state.layout.layout(Rect::new(
            state.hscroll.offset as u16,
            state.vscroll.offset as u16,
            state.widget_area.width,
            state.widget_area.height,
        ));

        let dd_page = state.layout.page();
        let dd_wide = state.layout.wide_page();
        debug!(
            "layout page {:?} -> wide {:?}",
            rect_dbg(dd_page),
            rect_dbg(dd_wide)
        );

        let buf_offset_x = state.hscroll.offset as u16 - ext_area.x;
        let buf_offset_y = state.vscroll.offset as u16 - ext_area.y;

        // adjust scroll
        let (max_x, max_y) = state.layout.max_position();
        state
            .vscroll
            .set_page_len(state.widget_area.height as usize);
        state
            .vscroll
            .set_max_offset(max_y.saturating_sub(state.widget_area.height) as usize);
        state.hscroll.set_page_len(state.widget_area.width as usize);
        state
            .hscroll
            .set_max_offset(max_x.saturating_sub(state.widget_area.width) as usize);

        // resize buffer to fit all visible widgets.
        let buffer_area = Rect::new(0, 0, ext_area.width, ext_area.height);
        let buffer = if let Some(mut buffer) = state.buffer.take() {
            buffer.reset();
            buffer.resize(buffer_area);
            buffer
        } else {
            Buffer::empty(buffer_area)
        };

        ClipperBuffer {
            layout: state.layout.clone(),
            buf_offset_x,
            buf_offset_y,
            buffer,
            widget_area: state.widget_area,
            block: self.block,
            hscroll: self.hscroll,
            vscroll: self.vscroll,
        }
    }
}

impl<'a> ClipperBuffer<'a> {
    /// Render a widget to the temp buffer.
    #[inline(always)]
    pub fn render_widget<W>(&mut self, widget: W, area: Rect)
    where
        W: Widget,
    {
        if let Some(buffer_area) = self.layout.locate(area) {
            debug!("tr {:?}->{:?}", rect_dbg(area), rect_dbg(buffer_area));
            // render the actual widget.
            widget.render(buffer_area, self.buffer_mut());
        } else {
            debug!("tr {:?}->hidden", rect_dbg(area));
        }
    }

    /// Render a widget to the temp buffer.
    #[inline(always)]
    pub fn render_widget_handle<W>(&mut self, widget: W, area: AreaHandle)
    where
        W: Widget,
    {
        if let Some(buffer_area) = self.layout.locate_handle(area) {
            debug!(
                "tr {:?}:{:?}->{:?}",
                area,
                self.view_area(area),
                rect_dbg(buffer_area)
            );
            // render the actual widget.
            widget.render(buffer_area, self.buffer_mut());
        } else {
            debug!("tr {:?}:{:?}->hidden", area, self.view_area(area),);
        }
    }

    /// Render a widget to the temp buffer.
    /// This expects that the state is a RelocatableState.
    #[inline(always)]
    pub fn render_stateful<W, S>(&mut self, widget: W, area: Rect, state: &mut S)
    where
        W: StatefulWidget<State = S>,
        S: RelocatableState,
    {
        if let Some(buffer_area) = self.layout.locate(area) {
            debug!("tr {:?}->{:?}", rect_dbg(area), rect_dbg(buffer_area));
            // render the actual widget.
            widget.render(buffer_area, self.buffer_mut(), state);
            // shift and clip the output areas.
            self.relocate(state);
        } else {
            debug!("tr {:?}->hidden", rect_dbg(area));
            self.relocate_clear(state);
        }
    }

    /// Render a widget to the temp buffer.
    /// This expects that the state is a RelocatableState.
    #[inline(always)]
    pub fn render_stateful_handle<W, S>(&mut self, widget: W, area: AreaHandle, state: &mut S)
    where
        W: StatefulWidget<State = S>,
        S: RelocatableState,
    {
        if let Some(buffer_area) = self.layout.locate_handle(area) {
            debug!(
                "tr {:?}:{:?}->{:?}",
                area,
                self.view_area(area),
                rect_dbg(buffer_area)
            );
            // render the actual widget.
            widget.render(buffer_area, self.buffer_mut(), state);
            // shift and clip the output areas.
            self.relocate(state);
        } else {
            debug!("tr {:?}:{:?}->hidden", area, self.view_area(area),);
            self.relocate_clear(state);
        }
    }

    /// Get the view area for the handle.
    pub fn view_area(&self, handle: AreaHandle) -> Option<Rect> {
        self.layout.locate_handle(handle)
    }

    /// Is the given area visible?
    pub fn is_visible(&self, area: Rect) -> bool {
        self.layout.locate(area).is_some()
    }

    /// Is the given area visible?
    pub fn is_visible_handle(&self, handle: AreaHandle) -> bool {
        self.layout.locate_handle(handle).is_some()
    }

    /// Calculate the necessary shift from view to screen.
    pub fn shift(&self) -> (i16, i16) {
        (
            self.widget_area.x as i16 - self.buf_offset_x as i16,
            self.widget_area.y as i16 - self.buf_offset_y as i16,
        )
    }

    /// Relocate all the widget state that refers to an area to
    /// the actual screen space used.
    pub fn relocate<S>(&self, state: &mut S)
    where
        S: RelocatableState,
    {
        state.relocate(self.shift(), self.widget_area);
    }

    /// Relocate in a way that clears the areas.
    /// This effectively hides any out of bounds widgets.
    pub fn relocate_clear<S>(&self, state: &mut S)
    where
        S: RelocatableState,
    {
        state.relocate((0, 0), Rect::default())
    }

    /// Access the temporary buffer.
    ///
    /// __Note__
    /// Use of render_widget is preferred.
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    /// Rendering the content is finished.
    ///
    /// Convert to the output widget that can be rendered in the target area.
    pub fn into_widget(self) -> ClipperWidget<'a> {
        ClipperWidget {
            buf_offset_x: self.buf_offset_x,
            buf_offset_y: self.buf_offset_y,
            block: self.block,
            hscroll: self.hscroll,
            vscroll: self.vscroll,
            buffer: self.buffer,
        }
    }
}

impl<'a> StatefulWidget for ClipperWidget<'a> {
    type State = ClipperState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        assert_eq!(area, state.area);

        ScrollArea::new()
            .block(self.block.as_ref())
            .h_scroll(self.hscroll.as_ref())
            .v_scroll(self.vscroll.as_ref())
            .render(
                area,
                buf,
                &mut ScrollAreaState::new()
                    .h_scroll(&mut state.hscroll)
                    .v_scroll(&mut state.vscroll),
            );

        let inner_area = state.widget_area;

        for y in 0..inner_area.height {
            for x in 0..inner_area.width {
                let xx = inner_area.x + x;
                let yy = inner_area.y + y;

                let buffer_x = x + self.buf_offset_x;
                let buffer_y = y + self.buf_offset_y;

                if let Some(cell) = self.buffer.cell(Position::new(buffer_x, buffer_y)) {
                    if let Some(tgt_cell) = buf.cell_mut(Position::new(xx, yy)) {
                        *tgt_cell = cell.clone();
                    }
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
            .area(self.widget_area)
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
