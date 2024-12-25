use crate::layout::GenericLayout;
use crate::pager::PagerStyle;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{StatefulWidget, Widget};
use std::rc::Rc;

/// Renders a single page of widgets.
#[derive(Debug, Clone)]
pub struct Pager<W, C = ()>
where
    W: Eq,
    C: Eq,
{
    layout: Rc<GenericLayout<W, C>>,
    page: usize,
    style: Style,
    label_style: Option<Style>,
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

impl<W, C> Default for Pager<W, C>
where
    W: Eq,
    C: Eq,
{
    fn default() -> Self {
        Self {
            layout: Default::default(),
            page: Default::default(),
            style: Default::default(),
            label_style: Default::default(),
        }
    }
}

impl<W, C> Pager<W, C>
where
    W: Eq,
    C: Eq,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Layout
    pub fn layout(mut self, layout: Rc<GenericLayout<W, C>>) -> Self {
        self.layout = layout;
        self
    }

    /// Display page.
    pub fn page(mut self, page: usize) -> Self {
        self.page = page;
        self
    }

    /// Base style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set all styles.
    pub fn styles(mut self, styles: PagerStyle) -> Self {
        self.style = styles.style;
        if let Some(label) = styles.label_style {
            self.label_style = Some(label);
        }
        self
    }

    /// Create the second stage.
    pub fn into_buffer<'b>(self, area: Rect, buf: &'b mut Buffer) -> PagerBuffer<'b, W, C> {
        PagerBuffer {
            layout: self.layout,
            page_area: Rect::new(0, self.page as u16 * area.height, area.width, area.height),
            widget_area: area,
            buffer: buf,
            label_style: self.label_style,
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
        self.page_area.intersects(self.layout.widget(idx))
    }

    /// Render a manual label.
    #[inline(always)]
    pub fn render_label<FN, WW>(&mut self, widget: &W, render_fn: FN) -> bool
    where
        FN: FnOnce() -> WW,
        WW: Widget,
    {
        let Some(idx) = self.layout.widget_idx(widget) else {
            return false;
        };
        let Some(label_area) = self.locate_area(self.layout.label(idx)) else {
            return false;
        };

        render_fn().render(label_area, self.buffer);
        true
    }

    /// Render a stateless widget and its label, if any.
    #[inline(always)]
    pub fn render_widget<FN, WW>(&mut self, widget: &W, render_fn: FN) -> bool
    where
        FN: FnOnce() -> WW,
        WW: Widget,
    {
        let Some(idx) = self.layout.widget_idx(widget) else {
            return false;
        };
        let Some(widget_area) = self.locate_area(self.layout.widget(idx)) else {
            return false;
        };

        self.render_auto_label(idx);
        render_fn().render(widget_area, self.buffer);
        true
    }

    /// Render a stateful widget and its label, if any.
    #[inline(always)]
    pub fn render<FN, WW, SS>(&mut self, widget: &W, render_fn: FN, state: &mut SS) -> bool
    where
        FN: FnOnce() -> WW,
        WW: StatefulWidget<State = SS>,
    {
        let Some(idx) = self.layout.widget_idx(widget) else {
            return false;
        };
        let Some(widget_area) = self.locate_area(self.layout.widget(idx)) else {
            return false;
        };

        self.render_auto_label(idx);
        render_fn().render(widget_area, self.buffer, state);

        true
    }

    fn render_auto_label(&mut self, idx: usize) {
        if let Some(label) = &self.layout.label_str(idx) {
            if let Some(label_area) = self.locate_area(self.layout.label(idx)) {
                let style = self.label_style.unwrap_or_default();
                Span::from(label.as_ref())
                    .style(style)
                    .render(label_area, self.buffer);
            }
        }
    }

    /// Render all containers for the current page.
    pub fn render_container(&mut self) {
        for (idx, container_area) in self.layout.containers().enumerate() {
            if let Some(container_area) = self.locate_area(*container_area) {
                (&self.layout.container_block(idx)).render(container_area, self.buffer);
            }
        }
    }

    /// Relocate the widget area to screen coordinates.
    /// Returns None if the widget is not visible.
    /// This clips the area to page_area.
    pub fn locate_widget(&self, widget: &W) -> Option<Rect> {
        let Some(idx) = self.layout.widget_idx(widget) else {
            return None;
        };
        self.locate_area(self.layout.widget(idx))
    }

    /// Relocate the label area to screen coordinates.
    /// Returns None if the widget is not visible.
    /// This clips the area to page_area.
    pub fn locate_label(&self, widget: &W) -> Option<Rect> {
        let Some(idx) = self.layout.widget_idx(widget) else {
            return None;
        };
        self.locate_area(self.layout.label(idx))
    }

    /// Relocate the container area to screen coordinates.
    ///
    /// Returns None if the container is not visible.
    /// If the container is split into multiple parts, this
    /// returns the first visible part.
    /// This clips the area to page_area.
    pub fn locate_container(&self, container: &C) -> Option<Rect> {
        for (idx, c) in self.layout.container_keys().enumerate() {
            if c == container {
                if self.layout.container(idx).intersects(self.page_area) {
                    return self.locate_area(self.layout.container(idx));
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
}
