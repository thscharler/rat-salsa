use crate::_private::NonExhaustive;
use crate::event::PagerOutcome;
use crate::layout::GenericLayout;
use crate::pager::{PageNavigation, PageNavigationState, Pager, PagerBuffer, PagerStyle};
use rat_event::{HandleEvent, MouseOnly, Regular};
use rat_reloc::RelocatableState;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect, Size};
use ratatui::prelude::{StatefulWidget, Style};
use ratatui::widgets::{Block, Widget};
use std::cell::RefCell;
use std::hash::Hash;
use std::rc::Rc;

/// This widget renders a single page of a [GenericLayout].
#[derive(Debug, Clone)]
pub struct SinglePager<'a, W, C = ()>
where
    W: Eq + Hash + Clone,
    C: Eq,
{
    pager: Pager<W, C>,
    page_nav: PageNavigation<'a>,
}

/// Renders directly to the frame buffer.
///
/// * It maps your widget area from layout coordinates
///   to screen coordinates before rendering.
/// * It helps with cleanup of the widget state if your
///   widget is currently invisible.
#[derive(Debug)]
pub struct SinglePagerBuffer<'a, W, C = ()>
where
    W: Eq + Hash + Clone,
    C: Eq,
{
    pager: PagerBuffer<'a, W, C>,
}

/// Widget state.
#[derive(Debug, Clone)]
pub struct SinglePagerState<W, C = ()>
where
    W: Eq + Hash + Clone,
    C: Eq,
{
    /// Page layout
    /// __read+write__ renewed with each render.
    pub layout: Rc<GenericLayout<W, C>>,

    /// PageNavigationState holds most of our state.
    /// __read+write__
    pub nav: PageNavigationState,

    /// Only construct with `..Default::default()`.
    pub non_exhaustive: NonExhaustive,
}

impl<'a, W, C> Default for SinglePager<'a, W, C>
where
    W: Eq + Hash + Clone,
    C: Eq,
{
    fn default() -> Self {
        Self {
            pager: Default::default(),
            page_nav: Default::default(),
        }
    }
}

impl<'a, W, C> SinglePager<'a, W, C>
where
    W: Eq + Hash + Clone,
    C: Eq,
{
    /// New SinglePage.
    pub fn new() -> Self {
        Self::default()
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

    /// Run the layout and create the second stage.
    pub fn into_buffer(
        self,
        area: Rect,
        buf: &'a mut Buffer,
        state: &mut SinglePagerState<W, C>,
    ) -> SinglePagerBuffer<'a, W, C> {
        state.nav.page_count = state.layout.page_count();

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

impl<'a, W, C> SinglePagerBuffer<'a, W, C>
where
    W: Eq + Hash + Clone,
    C: Eq,
{
    /// Render all containers for the current page.
    pub fn render_container(&mut self) {
        self.pager.render_container()
    }

    /// Render a manual label.
    #[inline(always)]
    pub fn render_label<FN, WW>(&mut self, widget: &W, render_fn: FN) -> bool
    where
        FN: FnOnce() -> WW,
        WW: Widget,
    {
        let Some(idx) = self.pager.widget_idx(widget) else {
            return false;
        };
        self.pager.render_label(idx, render_fn)
    }

    /// Render a stateless widget and its label, if any.
    #[inline(always)]
    pub fn render_widget<FN, WW>(&mut self, widget: &W, render_fn: FN) -> bool
    where
        FN: FnOnce() -> WW,
        WW: Widget,
    {
        let Some(idx) = self.pager.widget_idx(widget) else {
            return false;
        };
        self.pager.render_widget(idx, render_fn)
    }

    /// Render a stateful widget and its label, if any.
    #[inline(always)]
    pub fn render<FN, WW, SS>(&mut self, widget: &W, render_fn: FN, state: &mut SS) -> bool
    where
        FN: FnOnce() -> WW,
        WW: StatefulWidget<State = SS>,
        SS: RelocatableState,
    {
        let Some(idx) = self.pager.widget_idx(widget) else {
            return false;
        };
        if !self.pager.render(idx, render_fn, state) {
            self.hidden(state);
            false
        } else {
            true
        }
    }

    /// Is the given area visible?
    pub fn is_visible(&self, widget: &W) -> bool {
        self.pager.is_visible(widget)
    }

    /// Calculate the necessary shift from view to screen.
    /// This does nothing as pager always places the widgets
    /// in screen coordinates.
    ///
    /// Just to keep the api in sync with [Clipper].
    pub fn shift(&self) -> (i16, i16) {
        (0, 0)
    }

    /// Relocate the widget area to screen coordinates.
    /// Returns None if the widget is not visible.
    /// This clips the area to page_area.
    pub fn locate_widget(&self, widget: &W) -> Option<Rect> {
        let Some(idx) = self.pager.widget_idx(widget) else {
            return None;
        };
        self.pager.locate_widget(idx)
    }

    /// Relocate the label area to screen coordinates.
    /// Returns None if the widget is not visible.
    /// This clips the area to page_area.
    pub fn locate_label(&self, widget: &W) -> Option<Rect> {
        let Some(idx) = self.pager.widget_idx(widget) else {
            return None;
        };
        self.pager.locate_label(idx)
    }

    /// Relocate the container area to screen coordinates.
    ///
    /// Returns None if the container is not visible.
    /// If the container is split into multiple parts, this
    /// returns the first visible part.
    /// This clips the area to page_area.
    pub fn locate_container(&self, container: &C) -> Option<Rect> {
        self.pager.locate_container(container)
    }

    /// Relocate an area from layout coordinates to screen coordinates.
    /// A result None indicates that the area is invisible.
    ///
    /// This will clip the area to the page_area.
    pub fn locate_area(&self, area: Rect) -> Option<Rect> {
        self.pager.locate_area(area)
    }

    /// Does nothing for pager.
    /// Just to keep the api in sync with [Clipper].
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
}

impl<W, C> Default for SinglePagerState<W, C>
where
    W: Eq + Hash + Clone,
    C: Eq,
{
    fn default() -> Self {
        Self {
            layout: Default::default(),
            nav: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<W, C> SinglePagerState<W, C>
where
    W: Eq + Hash + Clone,
    C: Eq,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the page for this widget.
    pub fn show(&mut self, widget: &W) {
        if let Some(page) = self.layout.page_of(widget) {
            self.nav.set_page(page);
        }
    }

    /// Returns the first widget for the given page.
    pub fn first(&self, page: usize) -> Option<&W> {
        self.layout.first(page)
    }

    /// Calculates the page of the widget.
    pub fn page_of(&self, widget: &W) -> Option<usize> {
        self.layout.page_of(widget)
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

impl<W, C> HandleEvent<crossterm::event::Event, Regular, PagerOutcome> for SinglePagerState<W, C>
where
    W: Eq + Hash + Clone,
    C: Eq,
{
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> PagerOutcome {
        self.nav.handle(event, Regular)
    }
}

impl<W, C> HandleEvent<crossterm::event::Event, MouseOnly, PagerOutcome> for SinglePagerState<W, C>
where
    W: Eq + Hash + Clone,
    C: Eq,
{
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> PagerOutcome {
        self.nav.handle(event, MouseOnly)
    }
}
