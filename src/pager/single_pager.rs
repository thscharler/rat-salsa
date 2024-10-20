use crate::_private::NonExhaustive;
use crate::event::PagerOutcome;
use crate::pager::{AreaHandle, PageLayout, PagerStyle};
use crate::util::revert_style;
use rat_event::util::MouseFlagsN;
use rat_event::{ct_event, HandleEvent, MouseOnly, Regular};
use rat_focus::ContainerFlag;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Span, StatefulWidget, Style};
use ratatui::widgets::{Block, Borders, Widget};

/// Widget that displays one page of the PageLayout.
///
/// This only renders the navigation, you must render each widget
/// yourself. Call relocate(area) to get the actual screen-area
/// for your widget. If this call returns None, your widget shall
/// not be displayed.
#[derive(Debug, Default, Clone)]
pub struct SinglePager<'a> {
    layout: PageLayout,
    style: Style,
    nav_style: Option<Style>,
    title_style: Option<Style>,
    block: Option<Block<'a>>,
}

#[derive(Debug)]
pub struct SinglePagerRender {
    inner: Rect,
    page: usize,
    layout: PageLayout,
    style: Style,
    nav_style: Option<Style>,
}

#[derive(Debug, Clone)]
pub struct SinglePagerState {
    /// Full area.
    /// __read only__ renewed with each render.
    pub area: Rect,
    /// Area for widgets to render.
    // __read only__ renewed with each render.
    pub widget_area: Rect,
    /// Title area except the page indicators.
    /// __read only__ renewed with each render
    pub scroll_area: Rect,
    /// Area for prev-page indicator.
    /// __read only__ renewed with each render.
    pub prev_area: Rect,
    /// Area for next-page indicator.
    /// __read only__ renewed with each render.
    pub next_area: Rect,

    /// Page layout
    /// __read only__ renewed with each render.
    pub layout: PageLayout,
    /// Current page.
    /// __read+write__
    pub page: usize,

    /// This widget has no focus of its own, but this flag
    /// can be used to set a container state.
    pub c_focus: ContainerFlag,

    /// Mouse
    pub mouse: MouseFlagsN,

    /// Only construct with `..Default::default()`.
    pub non_exhaustive: NonExhaustive,
}

impl<'a> SinglePager<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Page layout.
    pub fn layout(mut self, page_layout: PageLayout) -> Self {
        self.layout = page_layout;
        self
    }

    /// Set all styles.
    pub fn styles(mut self, styles: PagerStyle) -> Self {
        self.style = styles.style;
        if let Some(nav) = styles.nav {
            self.nav_style = Some(nav);
        }
        if let Some(title) = styles.title {
            self.title_style = Some(title);
        }
        if let Some(block) = styles.block {
            self.block = Some(block);
        }
        self
    }

    /// Base style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Style for navigation.
    pub fn nav_style(mut self, nav_style: Style) -> Self {
        self.nav_style = Some(nav_style);
        self
    }

    /// Style for the title.
    pub fn title_style(mut self, title_style: Style) -> Self {
        self.title_style = Some(title_style);
        self
    }

    /// Block for border
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Run the layout and create the final Pager widget.
    pub fn into_widget(
        self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut SinglePagerState,
    ) -> SinglePagerRender {
        state.area = area;

        let title = format!(" {}/{} ", state.page + 1, state.layout.len());
        let block = self
            .block
            .unwrap_or_else(|| Block::new().borders(Borders::TOP))
            .title_bottom(title)
            .title_alignment(Alignment::Right);
        let block = if let Some(title_style) = self.title_style {
            block.title_style(title_style)
        } else {
            block
        };

        let inner = block.inner(area);

        let p1 = 5;
        let p4 = inner.width - p1;
        state.prev_area = Rect::new(inner.x, area.y, p1, 1);
        state.next_area = Rect::new(inner.x + p4, area.y, p1, 1);
        state.scroll_area = Rect::new(area.x + 1, area.y, area.width.saturating_sub(2), 1);

        state.widget_area = Rect::new(inner.x, inner.y, inner.width, inner.height);

        // run page layout
        state.layout = self.layout;
        state.layout.layout(state.widget_area);
        // clip pages
        state.set_page(state.page);

        // render
        buf.set_style(area, self.style);
        block.render(area, buf);

        SinglePagerRender {
            inner: state.widget_area,
            page: state.page,
            layout: state.layout.clone(),
            style: self.style,
            nav_style: self.nav_style,
        }
    }
}

impl SinglePagerRender {
    /// Relocate an area by handle from Layout coordinates to
    /// screen coordinates.
    ///
    /// A result None indicates that the area is
    /// out of view.
    pub fn relocate_handle(&self, handle: AreaHandle) -> Option<Rect> {
        let (page, target) = self.layout.locate_handle(handle);
        self._relocate(page, target)
    }

    /// Relocate a rect from Layout coordinates to
    /// screen coordinates.
    ///
    /// A result None indicates that the area is
    /// out of view.
    pub fn relocate(&self, area: Rect) -> Option<Rect> {
        let (page, target) = self.layout.locate(area);
        self._relocate(page, target)
    }

    fn _relocate(&self, page: usize, mut target_area: Rect) -> Option<Rect> {
        if self.page == page {
            target_area.x += self.inner.x;
            target_area.y += self.inner.y;
            Some(target_area)
        } else {
            None
        }
    }
}

impl StatefulWidget for SinglePagerRender {
    type State = SinglePagerState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        assert_eq!(area, state.area);

        // active areas
        let nav_style = self.nav_style.unwrap_or(self.style);
        if matches!(state.mouse.hover.get(), Some(0)) {
            buf.set_style(state.prev_area, revert_style(nav_style));
        } else {
            buf.set_style(state.prev_area, nav_style);
        }
        if state.page > 0 {
            Span::from(" <<< ").render(state.prev_area, buf);
        } else {
            Span::from(" [·] ").render(state.prev_area, buf);
        }
        if matches!(state.mouse.hover.get(), Some(1)) {
            buf.set_style(state.next_area, revert_style(nav_style));
        } else {
            buf.set_style(state.next_area, nav_style);
        }
        if state.page + 1 < state.layout.len() {
            Span::from(" >>> ").render(state.next_area, buf);
        } else {
            Span::from(" [·] ").render(state.next_area, buf);
        }
    }
}

impl Default for SinglePagerState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            widget_area: Default::default(),
            scroll_area: Default::default(),
            prev_area: Default::default(),
            next_area: Default::default(),
            layout: Default::default(),
            page: 0,
            c_focus: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl SinglePagerState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the page for this rect.
    pub fn show_handle(&mut self, handle: AreaHandle) {
        let (page, _) = self.layout.locate_handle(handle);
        self.page = page;
    }

    /// Show the page for this rect.
    pub fn show_area(&mut self, area: Rect) {
        let (page, _) = self.layout.locate(area);
        self.page = page;
    }

    /// First handle for the page.
    /// This returns the first handle for the page.
    /// Does not check whether the connected area is visible.
    pub fn first_handle(&self, page: usize) -> Option<AreaHandle> {
        self.layout.first_handle(page)
    }

    /// Set the visible page.
    pub fn set_page(&mut self, page: usize) -> bool {
        let old_page = self.page;
        if page >= self.layout.len() {
            self.page = self.layout.len() - 1;
        } else {
            self.page = page;
        }
        old_page != self.page
    }

    /// Select next page. Keeps the page in bounds.
    pub fn next_page(&mut self) -> bool {
        let old_page = self.page;

        if self.page + 1 >= self.layout.len() {
            self.page = self.layout.len() - 1;
        } else {
            self.page += 1;
        }

        old_page != self.page
    }

    /// Select prev page.
    pub fn prev_page(&mut self) -> bool {
        if self.page > 0 {
            self.page -= 1;
            true
        } else {
            false
        }
    }
}

impl HandleEvent<crossterm::event::Event, Regular, PagerOutcome> for SinglePagerState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> PagerOutcome {
        self.handle(event, MouseOnly)
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, PagerOutcome> for SinglePagerState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> PagerOutcome {
        match event {
            ct_event!(mouse down Left for x,y) if self.prev_area.contains((*x, *y).into()) => {
                if self.prev_page() {
                    PagerOutcome::Page(self.page)
                } else {
                    PagerOutcome::Unchanged
                }
            }
            ct_event!(mouse down Left for x,y) if self.next_area.contains((*x, *y).into()) => {
                if self.next_page() {
                    PagerOutcome::Page(self.page)
                } else {
                    PagerOutcome::Unchanged
                }
            }
            ct_event!(scroll down for x,y) => {
                if self.scroll_area.contains((*x, *y).into()) {
                    if self.next_page() {
                        PagerOutcome::Page(self.page)
                    } else {
                        PagerOutcome::Unchanged
                    }
                } else {
                    PagerOutcome::Continue
                }
            }
            ct_event!(scroll up for x,y) => {
                if self.scroll_area.contains((*x, *y).into()) {
                    if self.prev_page() {
                        PagerOutcome::Page(self.page)
                    } else {
                        PagerOutcome::Unchanged
                    }
                } else {
                    PagerOutcome::Continue
                }
            }
            ct_event!(mouse any for m)
                if self.mouse.hover(&[self.prev_area, self.next_area], m) =>
            {
                PagerOutcome::Changed
            }
            _ => PagerOutcome::Continue,
        }
    }
}
