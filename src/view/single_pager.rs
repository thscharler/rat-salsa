use crate::_private::NonExhaustive;
use crate::relocate::RelocatableState;
use crate::util::revert_style;
use crate::view::event::PagerOutcome;
use crate::view::{AreaHandle, PageLayout, PagerStyle};
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
pub struct SinglePage<'a> {
    layout: PageLayout,

    block: Option<Block<'a>>,
    style: Style,
    nav_style: Option<Style>,
    title_style: Option<Style>,
}

/// Render to the buffer.
#[derive(Debug)]
pub struct SinglePageBuffer<'a> {
    layout: PageLayout,

    // current page.
    page: usize,
    buffer: &'a mut Buffer,

    // inner area that will be rendered.
    widget_area: Rect,

    style: Style,
    nav_style: Option<Style>,
}

/// Rendering widget for SinglePage.
#[derive(Debug)]
pub struct SinglePageWidget {
    style: Style,
    nav_style: Option<Style>,
}

/// View state.
#[derive(Debug, Clone)]
pub struct SinglePageState {
    /// Full area for the widget.
    /// __read only__ renewed for each render.
    pub area: Rect,
    /// Area inside the border.
    /// __read only__ renewed for each render.
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

impl<'a> SinglePage<'a> {
    /// New SinglePage.
    pub fn new() -> Self {
        Self::default()
    }

    /// Page layout.
    pub fn layout(mut self, page_layout: PageLayout) -> Self {
        self.layout = page_layout;
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

    /// Calculate the layout width.
    pub fn layout_width(&self, area: Rect) -> u16 {
        self.inner(area).width
    }

    /// Calculate the view area.
    pub fn inner(&self, area: Rect) -> Rect {
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

    /// Run the layout and return a SinglePageBuffer.
    /// The SinglePageBuffer is used to actually render the contents.
    pub fn into_buffer<'b>(
        self,
        area: Rect,
        buf: &'b mut Buffer,
        state: &mut SinglePageState,
    ) -> SinglePageBuffer<'b> {
        state.area = area;

        state.widget_area = if let Some(block) = &self.block {
            block.inner(area)
        } else {
            Rect::new(
                area.x,
                area.y + 1,
                area.width,
                area.height.saturating_sub(1),
            )
        };

        let p1 = 5;
        let p4 = state.widget_area.width - p1;
        state.prev_area = Rect::new(state.widget_area.x, area.y, p1, 1);
        state.next_area = Rect::new(state.widget_area.x + p4, area.y, p1, 1);
        state.scroll_area = Rect::new(area.x + 1, area.y, area.width.saturating_sub(2), 1);

        // run page layout
        state.layout = self.layout;
        state.layout.layout(state.widget_area);
        // clip page nr
        state.set_page(state.page);

        // render
        let title = format!(" {}/{} ", state.page + 1, state.layout.len());
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

        SinglePageBuffer {
            layout: state.layout.clone(),
            page: state.page,
            buffer: buf,
            widget_area: state.widget_area,
            style: self.style,
            nav_style: self.nav_style,
        }
    }
}

impl<'a> SinglePageBuffer<'a> {
    /// Render a widget to the temp buffer.
    #[inline(always)]
    pub fn render_widget<W>(&mut self, widget: W, area: Rect)
    where
        W: Widget,
    {
        if let Some(buffer_area) = self.locate(area) {
            // render the actual widget.
            widget.render(buffer_area, self.buffer);
        } else {
            // noop
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
        if let Some(buffer_area) = self.locate(area) {
            // render the actual widget.
            widget.render(buffer_area, self.buffer, state);
        } else {
            self.relocate_clear(state);
        }
    }

    /// Render a widget to the temp buffer.
    #[inline(always)]
    pub fn render_widget_handle<W>(&mut self, widget: W, area: AreaHandle)
    where
        W: Widget,
    {
        if let Some(buffer_area) = self.locate_handle(area) {
            // render the actual widget.
            widget.render(buffer_area, self.buffer);
        } else {
            // noop
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
        if let Some(buffer_area) = self.locate_handle(area) {
            // render the actual widget.
            widget.render(buffer_area, self.buffer, state);
        } else {
            self.relocate_clear(state);
        }
    }

    /// Get the layout area for the handle.
    pub fn layout_area(&self, handle: AreaHandle) -> Rect {
        self.layout.layout_area_by_handle(handle)
    }

    /// Get the buffer area for the handle.
    /// Returns the tuple (page-nr, area)
    pub fn buf_area(&self, handle: AreaHandle) -> (usize, Rect) {
        self.layout.buf_area_by_handle(handle)
    }

    /// Is the given area visible?
    pub fn is_visible(&self, area: Rect) -> bool {
        self.layout.buf_area(area).0 == self.page
    }

    /// Is the given area visible?
    pub fn is_visible_handle(&self, handle: AreaHandle) -> bool {
        self.layout.buf_area_by_handle(handle).0 == self.page
    }

    /// Relocate an area by handle from Layout coordinates to
    /// screen coordinates.
    ///
    /// A result None indicates that the area is
    /// out of view.
    pub fn locate_handle(&self, handle: AreaHandle) -> Option<Rect> {
        let (page, target) = self.layout.buf_area_by_handle(handle);
        self._locate(page, target)
    }

    /// Relocate a rect from Layout coordinates to
    /// screen coordinates.
    ///
    /// A result None indicates that the area is
    /// out of view.
    pub fn locate(&self, area: Rect) -> Option<Rect> {
        let (page, target) = self.layout.buf_area(area);
        self._locate(page, target)
    }

    fn _locate(&self, page: usize, layout_area: Rect) -> Option<Rect> {
        if self.page == page {
            Some(Rect::new(
                layout_area.x + self.widget_area.x,
                layout_area.y + self.widget_area.y,
                layout_area.width,
                layout_area.height,
            ))
        } else {
            None
        }
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
    pub fn into_widget(self) -> SinglePageWidget {
        SinglePageWidget {
            style: self.style,
            nav_style: self.nav_style,
        }
    }
}

impl StatefulWidget for SinglePageWidget {
    type State = SinglePageState;

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

impl Default for SinglePageState {
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

impl SinglePageState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the page for this rect.
    pub fn show_handle(&mut self, handle: AreaHandle) {
        let (page, _) = self.layout.buf_area_by_handle(handle);
        self.page = page;
    }

    /// Show the page for this rect.
    pub fn show_area(&mut self, area: Rect) {
        let (page, _) = self.layout.buf_area(area);
        self.page = page;
    }

    /// First handle for the page.
    /// This returns the first handle for the page.
    /// Does not check whether the connected area is visible.
    pub fn first_handle(&self, page: usize) -> Option<AreaHandle> {
        self.layout.first_layout_handle(page)
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

impl HandleEvent<crossterm::event::Event, Regular, PagerOutcome> for SinglePageState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> PagerOutcome {
        self.handle(event, MouseOnly)
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, PagerOutcome> for SinglePageState {
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
