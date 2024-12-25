use crate::_private::NonExhaustive;
use crate::layout::GenericLayout;
use crate::pager::PagerStyle;
use rat_reloc::RelocatableState;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::BlockExt;
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{Block, StatefulWidget, Widget};
use std::marker::PhantomData;
use std::rc::Rc;

/// Renders a single page of widgets.
#[derive(Debug, Clone)]
pub struct Pager<'a, W, C = ()>
where
    W: Eq,
    C: Eq,
{
    page: usize,

    style: Style,
    block: Option<Block<'a>>,
    label_style: Option<Style>,

    phantom: PhantomData<(W, C)>,
}

/// Rendering phase.
#[derive(Debug)]
pub struct PagerBuffer<'a, W, C = ()>
where
    W: Eq,
    C: Eq,
{
    layout: Rc<GenericLayout<W, C>>,

    page_area: Rect,
    widget_area: Rect,

    buffer: &'a mut Buffer,

    label_style: Option<Style>,
}

/// Pager state.
#[derive(Debug)]
pub struct PagerState<W, C = ()>
where
    W: Eq,
    C: Eq,
{
    /// Full area.
    /// __read only__ renewed for each render.
    pub area: Rect,
    /// Area inside the border.
    /// __read only__ renewed for each render.
    pub widget_area: Rect,

    /// Layout
    /// __read+write__
    pub layout: Rc<GenericLayout<W, C>>,

    /// Only construct with `..Default::default()`.
    pub non_exhaustive: NonExhaustive,
}

impl<'a, W, C> Default for Pager<'a, W, C>
where
    W: Eq,
    C: Eq,
{
    fn default() -> Self {
        Self {
            page: Default::default(),
            style: Default::default(),
            block: Default::default(),
            label_style: Default::default(),
            phantom: Default::default(),
        }
    }
}

impl<'a, W, C> Pager<'a, W, C>
where
    W: Eq,
    C: Eq,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Display page.
    pub fn page(mut self, page: usize) -> Self {
        self.page = page;
        self
    }

    /// Base style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self.block = self.block.map(|v| v.style(style));
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
        if let Some(label) = styles.label_style {
            self.label_style = Some(label);
        }
        if let Some(block) = styles.block {
            self.block = Some(block);
        }
        self.block = self.block.map(|v| v.style(styles.style));
        self
    }

    /// Create the second stage.
    pub fn into_buffer<'b>(
        self,
        area: Rect,
        buf: &'b mut Buffer,
        state: &mut PagerState<W, C>,
    ) -> PagerBuffer<'b, W, C> {
        state.area = area;
        state.widget_area = self.block.inner_if_some(area);

        PagerBuffer {
            layout: state.layout.clone(),
            page_area: Rect::new(0, self.page as u16 * area.height, area.width, area.height),
            widget_area: state.widget_area,
            buffer: buf,
            label_style: None,
        }
    }
}

impl<'a, W, C> PagerBuffer<'a, W, C>
where
    W: Eq,
    C: Eq,
{
    /// Is the widget visible.
    pub fn is_visible(&self, widget: &W) -> bool {
        let Some(idx) = self.layout.widget_idx(widget) else {
            return false;
        };
        self.page_area.intersects(self.layout.areas[idx])
    }

    /// Render a stateless widget and its label, if any.
    pub fn render_widget<FN, WW>(&mut self, widget: &W, render_fn: FN)
    where
        FN: FnOnce() -> WW,
        WW: Widget,
    {
        let Some(idx) = self.layout.widget_idx(widget) else {
            return;
        };
        let Some(widget_area) = self.locate_area(self.layout.areas[idx]) else {
            return;
        };

        self.render_label(idx);
        render_fn().render(widget_area, self.buffer);
    }

    /// Render a stateful widget and its label, if any.
    pub fn render<FN, WW, SS>(&mut self, widget: &W, render_fn: FN, state: &mut SS)
    where
        FN: FnOnce() -> WW,
        WW: StatefulWidget<State = SS>,
    {
        let Some(idx) = self.layout.widget_idx(widget) else {
            return;
        };
        let Some(widget_area) = self.locate_area(self.layout.areas[idx]) else {
            return;
        };

        self.render_label(idx);
        render_fn().render(widget_area, self.buffer, state);
    }

    fn render_label(&mut self, idx: usize) {
        if let Some(label) = &self.layout.labels[idx] {
            if let Some(label_area) = self.locate_area(self.layout.label_areas[idx]) {
                let style = self.label_style.unwrap_or_default();
                Span::from(label.as_ref())
                    .style(style)
                    .render(label_area, self.buffer);
            }
        }
    }

    /// Calculate the necessary shift from view to screen.
    /// This does nothing as Pager always places the widgets
    /// in screen coordinates.
    ///
    /// Just to keep the api in sync with [Clipper].
    pub fn shift(&self) -> (i16, i16) {
        (0, 0)
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

    /// Access the buffer.
    /// __Note__
    /// Use of render_xxx is preferred.
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        self.buffer
    }

    /// Relocate the widget area to screen coordinates.
    /// Returns None if the widget is not visible.
    /// This clips the area to page_area.
    pub fn locate_widget(&self, widget: &W) -> Option<Rect> {
        let Some(idx) = self.layout.widget_idx(widget) else {
            return None;
        };
        self.locate_area(self.layout.areas[idx])
    }

    /// Relocate the label area to screen coordinates.
    /// Returns None if the widget is not visible.
    /// This clips the area to page_area.
    pub fn locate_label(&self, widget: &W) -> Option<Rect> {
        let Some(idx) = self.layout.widget_idx(widget) else {
            return None;
        };
        self.locate_area(self.layout.label_areas[idx])
    }

    /// Relocate the container area to screen coordinates.
    ///
    /// Returns None if the container is not visible.
    /// If the container is split into multiple parts, this
    /// returns the first visible part.
    /// This clips the area to page_area.
    pub fn locate_container(&self, container: &C) -> Option<Rect> {
        for (idx, c) in self.layout.containers.iter().enumerate() {
            if c == container {
                if self.layout.container_areas[idx].intersects(self.page_area) {
                    return self.locate_area(self.layout.container_areas[idx]);
                }
            }
        }
        None
    }

    /// Relocate an area from layout coordinates to screen coordinates.
    /// A result None indicates that the area is invisible.
    ///
    /// This will clip the area to the page_area.
    pub fn locate_area(&self, area: Rect) -> Option<Rect> {
        let area = self.page_area.intersection(area);
        if area.is_empty() {
            None
        } else {
            Some(Rect::new(
                area.x - self.page_area.x + self.widget_area.x,
                area.y - self.page_area.y + self.widget_area.y,
                area.width,
                area.height,
            ))
        }
    }

    /// Rendering the content is finished.
    ///
    /// This function only exists to keep the api in sync with
    /// [Clipper].
    pub fn into_widget(self) -> () {}
}

impl<W, C> Default for PagerState<W, C>
where
    W: Eq,
    C: Eq,
{
    fn default() -> Self {
        Self {
            area: Default::default(),
            widget_area: Default::default(),
            layout: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<W, C> PagerState<W, C>
where
    W: Eq,
    C: Eq,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Page of the given widget.
    pub fn page_of(&self, widget: &W) -> Option<usize> {
        self.layout.page_of(widget)
    }

    /// First widget on the given page.
    pub fn first(&self, page: usize) -> Option<&W> {
        self.layout.first(page)
    }
}
