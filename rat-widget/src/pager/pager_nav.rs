use crate::_private::NonExhaustive;
use crate::event::PagerOutcome;
use crate::pager::PagerStyle;
use crate::util::revert_style;
use rat_event::util::MouseFlagsN;
use rat_event::{ct_event, ConsumedEvent, HandleEvent, MouseOnly, Regular};
use rat_focus::{FocusFlag, HasFocus};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Alignment, Rect, Size};
use ratatui_core::style::Style;
use ratatui_core::text::Span;
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_widgets::block::Block;
use std::cmp::min;
use unicode_display_width::width as unicode_width;

/// Render the navigation for one or more [Pager](crate::pager::Pager) widgets.
#[derive(Debug, Clone)]
pub struct PageNavigation<'a> {
    pages: u8,
    block: Option<Block<'a>>,
    style: Style,
    nav_style: Option<Style>,
    title_style: Option<Style>,
    next_page: &'a str,
    prev_page: &'a str,
    first_page: &'a str,
    last_page: &'a str,
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
    pub container: FocusFlag,

    /// Mouse
    pub mouse: MouseFlagsN,

    /// Only construct with `..Default::default()`.
    pub non_exhaustive: NonExhaustive,
}

impl Default for PageNavigation<'_> {
    fn default() -> Self {
        Self {
            pages: 1,
            block: Default::default(),
            style: Default::default(),
            nav_style: Default::default(),
            title_style: Default::default(),
            next_page: ">>>",
            prev_page: "<<<",
            first_page: "[·]",
            last_page: "[·]",
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

    pub fn next_page_mark(mut self, txt: &'a str) -> Self {
        self.next_page = txt;
        self
    }

    pub fn prev_page_mark(mut self, txt: &'a str) -> Self {
        self.prev_page = txt;
        self
    }

    pub fn first_page_mark(mut self, txt: &'a str) -> Self {
        self.first_page = txt;
        self
    }

    pub fn last_page_mark(mut self, txt: &'a str) -> Self {
        self.last_page = txt;
        self
    }

    /// Set all styles.
    pub fn styles(mut self, styles: PagerStyle) -> Self {
        self.style = styles.style;
        if let Some(nav) = styles.navigation {
            self.nav_style = Some(nav);
        }
        if let Some(title) = styles.title {
            self.title_style = Some(title);
        }
        if let Some(block) = styles.block {
            self.block = Some(block);
        }
        if let Some(txt) = styles.next_page_mark {
            self.next_page = txt;
        }
        if let Some(txt) = styles.prev_page_mark {
            self.prev_page = txt;
        }
        if let Some(txt) = styles.first_page_mark {
            self.first_page = txt;
        }
        if let Some(txt) = styles.last_page_mark {
            self.last_page = txt;
        }
        //todo
        self.block = self.block.map(|v| v.style(styles.style));
        self
    }

    /// Calculate the layout size for one column.
    pub fn layout_size(&self, area: Rect) -> Size {
        let inner = self.inner(area);
        Size::new(inner.width / self.pages as u16, inner.height)
    }

    // Calculate the view area for all columns.
    pub fn inner(&self, area: Rect) -> Rect {
        if let Some(block) = &self.block {
            block.inner(area)
        } else {
            area
        }
    }
}

impl StatefulWidget for PageNavigation<'_> {
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

        if state.page > 0 {
            state.prev_area = Rect::new(
                widget_area.x,
                area.y,
                unicode_width(self.prev_page) as u16,
                1,
            );
        } else {
            state.prev_area = Rect::new(
                widget_area.x,
                area.y,
                unicode_width(self.first_page) as u16,
                1,
            );
        }
        if (state.page + self.pages as usize) < state.page_count {
            let p = unicode_width(self.next_page) as u16;
            state.next_area = Rect::new(
                widget_area.x + widget_area.width.saturating_sub(p),
                area.y,
                p,
                1,
            );
        } else {
            let p = unicode_width(self.last_page) as u16;
            state.next_area = Rect::new(
                widget_area.x + widget_area.width.saturating_sub(p),
                area.y,
                p,
                1,
            );
        }

        // render
        let block = if state.page_count > 1 {
            let title = format!(" {}/{} ", state.page + 1, state.page_count);
            let block = self
                .block
                .unwrap_or_else(|| Block::new().style(self.style))
                .title_bottom(title)
                .title_alignment(Alignment::Right);
            if let Some(title_style) = self.title_style {
                block.title_style(title_style)
            } else {
                block
            }
        } else {
            self.block.unwrap_or_else(|| Block::new().style(self.style))
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
            Span::from(self.prev_page).render(state.prev_area, buf);
        } else {
            Span::from(self.first_page).render(state.prev_area, buf);
        }
        if matches!(state.mouse.hover.get(), Some(1)) {
            buf.set_style(state.next_area, revert_style(nav_style));
        } else {
            buf.set_style(state.next_area, nav_style);
        }
        if (state.page + self.pages as usize) < state.page_count {
            Span::from(self.next_page).render(state.next_area, buf);
        } else {
            Span::from(self.last_page).render(state.next_area, buf);
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
            container: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl PageNavigationState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset page and page-count.
    pub fn clear(&mut self) {
        self.page = 0;
        self.page_count = 0;
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
        let r = if self.container.is_focused() {
            match event {
                ct_event!(keycode press ALT-PageUp) => {
                    if self.prev_page() {
                        PagerOutcome::Page(self.page())
                    } else {
                        PagerOutcome::Continue
                    }
                }
                ct_event!(keycode press ALT-PageDown) => {
                    if self.next_page() {
                        PagerOutcome::Page(self.page())
                    } else {
                        PagerOutcome::Continue
                    }
                }
                _ => PagerOutcome::Continue,
            }
        } else {
            PagerOutcome::Continue
        };

        r.or_else(|| self.handle(event, MouseOnly))
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
                        PagerOutcome::Continue
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
                        PagerOutcome::Continue
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
