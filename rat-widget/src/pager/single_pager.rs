use crate::_private::NonExhaustive;
use crate::event::PagerOutcome;
use crate::layout::GenericLayout;
use crate::pager::{PageNavigation, PageNavigationState, Pager, PagerBuffer, PagerStyle};
use rat_event::{HandleEvent, MouseOnly, Regular};
use rat_reloc::RelocatableState;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect, Size};
use ratatui::style::Style;
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::{Block, Widget};
use std::borrow::Cow;
use std::cell::{Ref, RefCell, RefMut};
use std::hash::Hash;
use std::rc::Rc;

/// This widget renders a single page of a [GenericLayout].
#[derive(Debug, Clone)]
pub struct SinglePager<'a, W>
where
    W: Eq + Hash + Clone,
{
    layout: Option<GenericLayout<W>>,
    pager: Pager<W>,
    page_nav: PageNavigation<'a>,
}

/// Renders directly to the frame buffer.
///
/// * It maps your widget area from layout coordinates
///   to screen coordinates before rendering.
/// * It helps with cleanup of the widget state if your
///   widget is currently invisible.
#[derive(Debug)]
pub struct SinglePagerBuffer<'a, W>
where
    W: Eq + Hash + Clone,
{
    pager: PagerBuffer<'a, W>,
}

/// Widget state.
#[derive(Debug, Clone)]
pub struct SinglePagerState<W>
where
    W: Eq + Hash + Clone,
{
    /// Page layout
    /// __read+write__ might be overwritten from widget.
    pub layout: Rc<RefCell<GenericLayout<W>>>,

    /// PageNavigationState holds most of our state.
    /// __read+write__
    pub nav: PageNavigationState,

    /// Only construct with `..Default::default()`.
    pub non_exhaustive: NonExhaustive,
}

impl<W> Default for SinglePager<'_, W>
where
    W: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self {
            layout: Default::default(),
            pager: Default::default(),
            page_nav: Default::default(),
        }
    }
}

impl<'a, W> SinglePager<'a, W>
where
    W: Eq + Hash + Clone,
{
    /// New SinglePage.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the layout. If no layout is set here the layout is
    /// taken from the state.
    pub fn layout(mut self, layout: GenericLayout<W>) -> Self {
        self.layout = Some(layout);
        self
    }

    /// Base style.
    pub fn style(mut self, style: Style) -> Self {
        self.pager = self.pager.style(style);
        self.page_nav = self.page_nav.style(style);
        self
    }

    /// Style for text labels.
    pub fn label_style(mut self, style: Style) -> Self {
        self.pager = self.pager.label_style(style);
        self
    }

    /// Alignment for text labels.
    pub fn label_alignment(mut self, alignment: Alignment) -> Self {
        self.pager = self.pager.label_alignment(alignment);
        self
    }

    /// Style for navigation.
    pub fn nav_style(mut self, nav_style: Style) -> Self {
        self.page_nav = self.page_nav.nav_style(nav_style);
        self
    }

    /// Style for the title.
    pub fn title_style(mut self, title_style: Style) -> Self {
        self.page_nav = self.page_nav.title_style(title_style);
        self
    }

    /// Block for border
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.page_nav = self.page_nav.block(block);
        self
    }

    pub fn next_page_mark(mut self, txt: &'a str) -> Self {
        self.page_nav = self.page_nav.next_page_mark(txt);
        self
    }

    pub fn prev_page_mark(mut self, txt: &'a str) -> Self {
        self.page_nav = self.page_nav.prev_page_mark(txt);
        self
    }

    pub fn first_page_mark(mut self, txt: &'a str) -> Self {
        self.page_nav = self.page_nav.first_page_mark(txt);
        self
    }

    pub fn last_page_mark(mut self, txt: &'a str) -> Self {
        self.page_nav = self.page_nav.last_page_mark(txt);
        self
    }

    /// Set all styles.
    pub fn styles(mut self, styles: PagerStyle) -> Self {
        self.pager = self.pager.styles(styles.clone());
        self.page_nav = self.page_nav.styles(styles);
        self
    }

    /// Calculate the layout page size.
    pub fn layout_size(&self, area: Rect) -> Size {
        self.page_nav.layout_size(area)
    }

    // Calculate the view area for all columns.
    pub fn inner(&self, area: Rect) -> Rect {
        self.page_nav.inner(area)
    }

    /// Render the page navigation and create the SinglePagerBuffer
    /// that will do the actual widget rendering.
    pub fn into_buffer(
        self,
        area: Rect,
        buf: &'a mut Buffer,
        state: &'a mut SinglePagerState<W>,
    ) -> SinglePagerBuffer<'a, W> {
        // set layout
        if let Some(layout) = self.layout {
            state.layout = Rc::new(RefCell::new(layout));
        }

        state.nav.page_count = state.layout.borrow().page_count();
        state.nav.set_page(state.nav.page);

        self.page_nav.render(area, buf, &mut state.nav);

        SinglePagerBuffer {
            pager: self
                .pager //
                .layout(state.layout.clone())
                .page(state.nav.page)
                .into_buffer(state.nav.widget_areas[0], Rc::new(RefCell::new(buf))),
        }
    }
}

impl<'a, W> SinglePagerBuffer<'a, W>
where
    W: Eq + Hash + Clone,
{
    /// Is the given area visible?
    pub fn is_visible(&self, widget: W) -> bool {
        if let Some(idx) = self.pager.widget_idx(widget) {
            self.pager.is_visible(idx)
        } else {
            false
        }
    }

    /// Render all blocks for the current page.
    pub fn render_block(&mut self) {
        self.pager.render_block()
    }

    /// Render a manual label.
    #[inline(always)]
    pub fn render_label<FN, WW>(&mut self, widget: W, render_fn: FN) -> bool
    where
        FN: FnOnce(&Option<Cow<'static, str>>) -> WW,
        WW: Widget,
    {
        let Some(idx) = self.pager.widget_idx(widget) else {
            return false;
        };
        self.pager.render_label(idx, render_fn)
    }

    /// Render a stateless widget and its label, if any.
    #[inline(always)]
    pub fn render_widget<FN, WW>(&mut self, widget: W, render_fn: FN) -> bool
    where
        FN: FnOnce() -> WW,
        WW: Widget,
    {
        let Some(idx) = self.pager.widget_idx(widget) else {
            return false;
        };
        self.pager.render_auto_label(idx);
        self.pager.render_widget(idx, render_fn)
    }

    /// Render an optional stateful widget and its label, if any.
    #[inline(always)]
    pub fn render_opt<FN, WW, SS>(&mut self, widget: W, render_fn: FN, state: &mut SS) -> bool
    where
        FN: FnOnce() -> Option<WW>,
        WW: StatefulWidget<State = SS>,
        SS: RelocatableState,
    {
        let Some(idx) = self.pager.widget_idx(widget) else {
            return false;
        };
        self.pager.render_auto_label(idx);
        if !self.pager.render_opt(idx, render_fn, state) {
            self.hidden(state);
            false
        } else {
            true
        }
    }

    /// Render a stateful widget and its label, if any.
    #[inline(always)]
    pub fn render<FN, WW, SS>(&mut self, widget: W, render_fn: FN, state: &mut SS) -> bool
    where
        FN: FnOnce() -> WW,
        WW: StatefulWidget<State = SS>,
        SS: RelocatableState,
    {
        let Some(idx) = self.pager.widget_idx(widget) else {
            return false;
        };
        self.pager.render_auto_label(idx);
        if !self.pager.render(idx, render_fn, state) {
            self.hidden(state);
            false
        } else {
            true
        }
    }

    /// Render a stateful widget and its label, if any.
    /// The closure can return a second value, which will be forwarded
    /// if the widget is visible.
    #[inline(always)]
    #[allow(clippy::question_mark)]
    pub fn render2<FN, WW, SS, R>(&mut self, widget: W, render_fn: FN, state: &mut SS) -> Option<R>
    where
        FN: FnOnce() -> (WW, R),
        WW: StatefulWidget<State = SS>,
        SS: RelocatableState,
    {
        let Some(idx) = self.pager.widget_idx(widget) else {
            return None;
        };
        self.pager.render_auto_label(idx);
        if let Some(remainder) = self.pager.render2(idx, render_fn, state) {
            Some(remainder)
        } else {
            self.hidden(state);
            None
        }
    }

    /// Calculate the necessary shift from view to screen.
    /// This does nothing as pager always places the widgets
    /// in screen coordinates.
    ///
    /// Just to keep the api in sync with [Clipper](crate::clipper::Clipper).
    pub fn shift(&self) -> (i16, i16) {
        (0, 0)
    }

    /// Relocate the widget area to screen coordinates.
    /// Returns None if the widget is not visible.
    /// This clips the area to page_area.
    #[allow(clippy::question_mark)]
    pub fn locate_widget(&self, widget: W) -> Option<Rect> {
        let Some(idx) = self.pager.widget_idx(widget) else {
            return None;
        };
        self.pager.locate_widget(idx)
    }

    /// Relocate the label area to screen coordinates.
    /// Returns None if the widget is not visible.
    /// This clips the area to page_area.
    #[allow(clippy::question_mark)]
    pub fn locate_label(&self, widget: W) -> Option<Rect> {
        let Some(idx) = self.pager.widget_idx(widget) else {
            return None;
        };
        self.pager.locate_label(idx)
    }

    /// Relocate an area from layout coordinates to screen coordinates.
    /// A result None indicates that the area is invisible.
    ///
    /// This will clip the area to the page_area.
    pub fn locate_area(&self, area: Rect) -> Option<Rect> {
        self.pager.locate_area(area)
    }

    /// Does nothing for pager.
    /// Just to keep the api in sync with [Clipper](crate::clipper::Clipper).
    pub fn relocate<S>(&self, _state: &mut S)
    where
        S: RelocatableState,
    {
    }

    /// Clear the areas in the widget-state.
    /// This is called by render_xx whenever a widget is invisible.
    pub fn hidden<S>(&self, state: &mut S)
    where
        S: RelocatableState,
    {
        state.relocate((0, 0), Rect::default())
    }

    /// Get access to the buffer during rendering a page.
    pub fn buffer<'b>(&'b mut self) -> RefMut<'b, &'a mut Buffer> {
        self.pager.buffer()
    }
}

impl<W> Default for SinglePagerState<W>
where
    W: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self {
            layout: Default::default(),
            nav: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<W> SinglePagerState<W>
where
    W: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Layout needs to change?
    pub fn valid_layout(&self, size: Size) -> bool {
        let layout = self.layout.borrow();
        !layout.size_changed(size) && !layout.is_empty()
    }

    /// Set the layout.
    pub fn set_layout(&mut self, layout: GenericLayout<W>) {
        self.layout = Rc::new(RefCell::new(layout));
    }

    /// Layout.
    pub fn layout(&self) -> Ref<'_, GenericLayout<W>> {
        self.layout.borrow()
    }

    /// Clear the layout data and reset the page/page-count.
    pub fn clear(&mut self) {
        self.layout.borrow_mut().clear();
        self.nav.clear();
    }

    /// Show the page for this widget.
    pub fn show(&mut self, widget: W) {
        if let Some(page) = self.layout.borrow().page_of(widget) {
            self.nav.set_page(page);
        }
    }

    /// Returns the first widget for the given page.
    pub fn first(&self, page: usize) -> Option<W> {
        self.layout.borrow().first(page)
    }

    /// Calculates the page of the widget.
    pub fn page_of(&self, widget: W) -> Option<usize> {
        self.layout.borrow().page_of(widget)
    }

    /// Set the visible page.
    pub fn set_page(&mut self, page: usize) -> bool {
        self.nav.set_page(page)
    }

    /// Visible page
    pub fn page(&self) -> usize {
        self.nav.page()
    }

    /// Select next page. Keeps the page in bounds.
    pub fn next_page(&mut self) -> bool {
        self.nav.next_page()
    }

    /// Select prev page.
    pub fn prev_page(&mut self) -> bool {
        self.nav.prev_page()
    }
}

impl<W> HandleEvent<crossterm::event::Event, Regular, PagerOutcome> for SinglePagerState<W>
where
    W: Eq + Hash + Clone,
{
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> PagerOutcome {
        self.nav.handle(event, Regular)
    }
}

impl<W> HandleEvent<crossterm::event::Event, MouseOnly, PagerOutcome> for SinglePagerState<W>
where
    W: Eq + Hash + Clone,
{
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> PagerOutcome {
        self.nav.handle(event, MouseOnly)
    }
}
