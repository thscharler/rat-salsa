use crate::_private::NonExhaustive;
use crate::event::PagerOutcome;
use crate::pager::PagerStyle;
use crate::util::revert_style;
use rat_event::util::MouseFlagsN;
use rat_event::{ct_event, HandleEvent, MouseOnly, Regular};
use rat_focus::ContainerFlag;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect, Size};
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, StatefulWidget, Widget};
use std::cmp::min;

/// Render the navigation for one or more [Pager] widgets.
#[derive(Debug, Clone)]
pub struct PageNavigation<'a> {
    pages: u8,
    block: Option<Block<'a>>,
    style: Style,
    nav_style: Option<Style>,
    title_style: Option<Style>,
}

/// Widget state.
#[derive(Debug, Clone)]
pub struct PageNavigationState {
    /// Full area for the widget.
    /// __read only__ renewed for each render.
    pub area: Rect,
    /// Area for each page.
    /// __read only__ renewed for each render.
    pub widget_areas: Vec<Rect>,
    /// Area for prev-page indicator.
    /// __read only__ renewed with each render.
    pub prev_area: Rect,
    /// Area for next-page indicator.
    /// __read only__ renewed with each render.
    pub next_area: Rect,

    /// Current, left-most page.
    /// __read+write__
    pub page: usize,

    /// Page count.
    /// __read+write__
    pub page_count: usize,

    /// This widget has no focus of its own, but this flag
    /// can be used to set a container state.
    pub c_focus: ContainerFlag,

    /// Mouse
    pub mouse: MouseFlagsN,

    /// Only construct with `..Default::default()`.
    pub non_exhaustive: NonExhaustive,
}

impl<'a> Default for PageNavigation<'a> {
    fn default() -> Self {
        Self {
            pages: 1,
            block: Default::default(),
            style: Default::default(),
            nav_style: Default::default(),
            title_style: Default::default(),
        }
    }
}

impl<'a> PageNavigation<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of pages displayed.
    ///
    /// Splits the inner area into n equally sized areas.
    /// Any rounding areas are not given to any of these
    /// areas but area added as padding on the right side.
    pub fn pages(mut self, pages: u8) -> Self {
        self.pages = pages;
        self
    }

    /// Base style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self.block = self.block.map(|v| v.style(style));
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
        self.block = Some(block.style(self.style));
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
        self.block = self.block.map(|v| v.style(styles.style));
        self
    }

    /// Calculate the layout size for one column.
    pub fn layout_size(&self, area: Rect) -> Size {
        let inner = self.inner(area);
        Size::new(inner.width / self.pages as u16, inner.height)
    }

    // Calculate the view area for all columns.
    fn inner(&self, area: Rect) -> Rect {
        if let Some(block) = &self.block {
            block.inner(area)
        } else {
            Rect::new(
                area.x,
                area.y + 1,
                area.width,
                area.height.saturating_sub(1),
            )
        }
    }
}

impl<'a> StatefulWidget for PageNavigation<'a> {
    type State = PageNavigationState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;

        let widget_area = self.inner(area);

        let width = widget_area.width / self.pages as u16;
        let mut column_area = Rect::new(widget_area.x, widget_area.y, width, widget_area.height);
        state.widget_areas.clear();
        for _ in 0..self.pages {
            state.widget_areas.push(column_area);
            column_area.x += column_area.width;
        }

        let p1 = 5;
        let p4 = widget_area.width.saturating_sub(p1);
        state.prev_area = Rect::new(widget_area.x, area.y, p1, 1);
        state.next_area = Rect::new(widget_area.x + p4, area.y, p1, 1);

        // render
        let title = format!(" {}/{} ", state.page + 1, state.page_count);
        let block = self
            .block
            .unwrap_or_else(|| Block::new().borders(Borders::TOP).style(self.style))
            .title_bottom(title)
            .title_alignment(Alignment::Right);
        let block = if let Some(title_style) = self.title_style {
            block.title_style(title_style)
        } else {
            block
        };
        block.render(area, buf);

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
        if (state.page + self.pages as usize) < state.page_count {
            Span::from(" >>> ").render(state.next_area, buf);
        } else {
            Span::from(" [·] ").render(state.next_area, buf);
        }
    }
}

impl Default for PageNavigationState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            widget_areas: Default::default(),
            prev_area: Default::default(),
            next_area: Default::default(),
            page: Default::default(),
            page_count: Default::default(),
            c_focus: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl PageNavigationState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the page.
    pub fn set_page(&mut self, page: usize) -> bool {
        let old_page = self.page;
        self.page = min(page, self.page_count.saturating_sub(1));
        old_page != self.page
    }

    /// Current page.
    pub fn page(&self) -> usize {
        self.page
    }

    /// Set the page count.
    pub fn set_page_count(&mut self, count: usize) {
        self.page_count = count;
        self.page = min(self.page, count.saturating_sub(1));
    }

    /// Page count.
    pub fn page_count(&self) -> usize {
        self.page_count
    }

    /// Select next page. Keeps the page in bounds.
    pub fn next_page(&mut self) -> bool {
        let old_page = self.page;

        if self.page + 1 >= self.page_count {
            self.page = self.page_count.saturating_sub(1);
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

impl HandleEvent<crossterm::event::Event, Regular, PagerOutcome> for PageNavigationState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> PagerOutcome {
        self.handle(event, MouseOnly)
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, PagerOutcome> for PageNavigationState {
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
                if self.area.contains((*x, *y).into()) {
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
                if self.area.contains((*x, *y).into()) {
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
