use crate::layout::GenericLayout;
use crate::pager::PagerStyle;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{StatefulWidget, Widget};
use std::borrow::Cow;
use std::cell::{RefCell, RefMut};
use std::hash::Hash;
use std::rc::Rc;

///
/// Renders widgets for one page of a [GenericLayout].
///
///
///
///
#[derive(Debug)]
pub struct Pager<W>
where
    W: Eq + Hash + Clone,
{
    layout: Rc<GenericLayout<W>>,
    page: usize,
    style: Style,
    label_style: Option<Style>,
    label_alignment: Option<Alignment>,
}

/// Rendering phase.
#[derive(Debug)]
pub struct PagerBuffer<'a, W>
where
    W: Eq + Hash + Clone,
{
    layout: Rc<GenericLayout<W>>,
    page_area: Rect,
    widget_area: Rect,
    buffer: Rc<RefCell<&'a mut Buffer>>,
    label_style: Option<Style>,
    label_alignment: Option<Alignment>,
}

impl<W> Clone for Pager<W>
where
    W: Eq + Hash + Clone,
{
    fn clone(&self) -> Self {
        Self {
            layout: self.layout.clone(),
            page: self.page,
            style: self.style,
            label_style: self.label_style,
            label_alignment: self.label_alignment,
        }
    }
}

impl<W> Default for Pager<W>
where
    W: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self {
            layout: Default::default(),
            page: Default::default(),
            style: Default::default(),
            label_style: Default::default(),
            label_alignment: Default::default(),
        }
    }
}

impl<W> Pager<W>
where
    W: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Layout
    pub fn layout(mut self, layout: Rc<GenericLayout<W>>) -> Self {
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

    pub fn label_style(mut self, style: Style) -> Self {
        self.label_style = Some(style);
        self
    }

    pub fn label_alignment(mut self, alignment: Alignment) -> Self {
        self.label_alignment = Some(alignment);
        self
    }

    /// Set all styles.
    pub fn styles(mut self, styles: PagerStyle) -> Self {
        self.style = styles.style;
        if let Some(label) = styles.label_style {
            self.label_style = Some(label);
        }
        if let Some(alignment) = styles.label_alignment {
            self.label_alignment = Some(alignment);
        }
        self
    }

    /// Create the second stage.
    #[allow(clippy::needless_lifetimes)]
    pub fn into_buffer<'b>(
        self,
        area: Rect,
        buf: Rc<RefCell<&'b mut Buffer>>,
    ) -> PagerBuffer<'b, W> {
        let page_size = self.layout.page_size();
        let page_area = Rect::new(
            0,
            self.page as u16 * page_size.height,
            page_size.width,
            page_size.height,
        );

        PagerBuffer {
            layout: self.layout,
            page_area,
            widget_area: area,
            buffer: buf,
            label_style: self.label_style,
            label_alignment: self.label_alignment,
        }
    }
}

impl<'a, W> PagerBuffer<'a, W>
where
    W: Eq + Hash + Clone,
{
    /// Is the widget visible.
    pub fn is_visible(&self, widget: W) -> bool {
        let Some(idx) = self.layout.try_index_of(widget) else {
            return false;
        };
        let area = self.layout.widget(idx);
        self.page_area.intersects(area)
    }

    /// Get the widget index.
    #[inline(always)]
    pub fn widget_idx(&self, widget: W) -> Option<usize> {
        self.layout.try_index_of(widget)
    }

    /// Render a manual label.
    #[inline(always)]
    pub fn render_label<FN, WW>(&mut self, idx: usize, render_fn: FN) -> bool
    where
        FN: FnOnce(&Option<Cow<'static, str>>) -> WW,
        WW: Widget,
    {
        let Some(label_area) = self.locate_area(self.layout.label(idx)) else {
            return false;
        };
        let label_str = self.layout.label_str(idx);

        let mut buffer = self.buffer.borrow_mut();
        render_fn(label_str).render(label_area, *buffer);

        true
    }

    /// Render the label with the set style and alignment.
    #[inline(always)]
    pub fn render_auto_label(&mut self, idx: usize) -> bool {
        let Some(label_area) = self.locate_area(self.layout.label(idx)) else {
            return false;
        };
        let Some(label_str) = self.layout.label_str(idx) else {
            return false;
        };

        let mut buffer = self.buffer.borrow_mut();
        let mut label = Line::from(label_str.as_ref());
        if let Some(style) = self.label_style {
            label = label.style(style)
        };
        if let Some(align) = self.label_alignment {
            label = label.alignment(align);
        }
        label.render(label_area, *buffer);

        true
    }

    /// Render a stateless widget.
    #[inline(always)]
    pub fn render_widget<FN, WW>(&mut self, idx: usize, render_fn: FN) -> bool
    where
        FN: FnOnce() -> WW,
        WW: Widget,
    {
        let Some(widget_area) = self.locate_area(self.layout.widget(idx)) else {
            return false;
        };

        let mut buffer = self.buffer.borrow_mut();
        render_fn().render(widget_area, *buffer);
        true
    }

    /// Render a stateful widget.
    #[inline(always)]
    pub fn render<FN, WW, SS>(&mut self, idx: usize, render_fn: FN, state: &mut SS) -> bool
    where
        FN: FnOnce() -> WW,
        WW: StatefulWidget<State = SS>,
    {
        let Some(widget_area) = self.locate_area(self.layout.widget(idx)) else {
            return false;
        };

        let mut buffer = self.buffer.borrow_mut();
        render_fn().render(widget_area, *buffer, state);

        true
    }

    /// Render a stateful widget.
    #[inline(always)]
    pub fn render_opt<FN, WW, SS>(&mut self, idx: usize, render_fn: FN, state: &mut SS) -> bool
    where
        FN: FnOnce() -> Option<WW>,
        WW: StatefulWidget<State = SS>,
    {
        let Some(widget_area) = self.locate_area(self.layout.widget(idx)) else {
            return false;
        };

        let mut buffer = self.buffer.borrow_mut();
        let widget = render_fn();
        if let Some(widget) = widget {
            widget.render(widget_area, *buffer, state);
        }

        true
    }

    /// Render a stateful widget.
    #[inline(always)]
    pub fn render2<FN, WW, SS, R>(&mut self, idx: usize, render_fn: FN, state: &mut SS) -> Option<R>
    where
        FN: FnOnce() -> (WW, R),
        WW: StatefulWidget<State = SS>,
    {
        let Some(widget_area) = self.locate_area(self.layout.widget(idx)) else {
            return None;
        };

        let mut buffer = self.buffer.borrow_mut();
        let (widget, remainder) = render_fn();
        widget.render(widget_area, *buffer, state);

        Some(remainder)
    }

    /// Render all blocks for the current page.
    pub fn render_block(&mut self) {
        for (idx, block_area) in self.layout.block_area_iter().enumerate() {
            if let Some(block_area) = self.locate_area(*block_area) {
                let mut buffer = self.buffer.borrow_mut();
                self.layout.block(idx).render(block_area, *buffer);
            }
        }
    }

    /// Relocate the widget area to screen coordinates.
    /// Returns None if the widget is not visible.
    /// This clips the area to page_area.
    #[inline]
    pub fn locate_widget(&self, idx: usize) -> Option<Rect> {
        self.locate_area(self.layout.widget(idx))
    }

    /// Relocate the label area to screen coordinates.
    /// Returns None if the widget is not visible.
    /// This clips the area to page_area.
    #[inline]
    pub fn locate_label(&self, idx: usize) -> Option<Rect> {
        self.locate_area(self.layout.label(idx))
    }

    /// Relocate an area from layout coordinates to screen coordinates.
    /// A result None indicates that the area is invisible.
    ///
    /// This will clip the area to the page_area.
    #[inline]
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

    /// Return a clone of the layout.
    #[inline]
    pub fn layout(&self) -> Rc<GenericLayout<W>> {
        self.layout.clone()
    }

    /// Get access to the buffer during rendering a page.
    pub fn buffer<'b>(&'b mut self) -> RefMut<'b, &'a mut Buffer> {
        self.buffer.borrow_mut()
    }
}
