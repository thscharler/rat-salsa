use crate::_private::NonExhaustive;
use crate::clipper::ClipperStyle;
use crate::layout::GenericLayout;
use rat_event::{ConsumedEvent, HandleEvent, MouseOnly, Outcome, Regular, ct_event};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_reloc::RelocatableState;
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::{Scroll, ScrollArea, ScrollAreaState, ScrollState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Position, Rect, Size};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Widget;
use ratatui::widgets::{Block, StatefulWidget};
use std::borrow::Cow;
use std::cell::{Ref, RefCell};
use std::cmp::{max, min};
use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;

/// This widget allows rendering to a temp-buffer and clips
/// it to size for the final rendering.
#[derive(Debug)]
pub struct Clipper<'a, W>
where
    W: Eq + Clone + Hash,
{
    layout: Option<GenericLayout<W>>,
    style: Style,
    block: Option<Block<'a>>,
    hscroll: Option<Scroll<'a>>,
    vscroll: Option<Scroll<'a>>,
    label_style: Option<Style>,
    label_alignment: Option<Alignment>,
    auto_label: bool,
}

/// Intermediate stage for rendering. Uses a temp-buffer.
#[derive(Debug)]
pub struct ClipperBuffer<'a, W>
where
    W: Eq + Clone + Hash,
{
    layout: Rc<RefCell<GenericLayout<W>>>,
    auto_label: bool,

    // offset from buffer to scroll area
    offset: Position,
    buffer: Buffer,

    // inner area that will finally be rendered.
    widget_area: Rect,

    style: Style,
    block: Option<Block<'a>>,
    hscroll: Option<Scroll<'a>>,
    vscroll: Option<Scroll<'a>>,
    label_style: Option<Style>,
    label_alignment: Option<Alignment>,
}

#[derive(Debug)]
pub struct ClipperWidget<'a, W>
where
    W: Eq + Clone + Hash,
{
    offset: Position,
    buffer: Buffer,

    style: Style,
    block: Option<Block<'a>>,
    hscroll: Option<Scroll<'a>>,
    vscroll: Option<Scroll<'a>>,
    phantom: PhantomData<W>,
}

#[derive(Debug)]
pub struct ClipperState<W>
where
    W: Eq + Clone + Hash,
{
    // Full area for the widget.
    /// __read only__ renewed for each render.
    pub area: Rect,
    /// Area inside the border.
    /// __read only__ renewed for each render.
    pub widget_area: Rect,

    /// Page layout.
    /// __read only__ renewed for each render.
    pub layout: Rc<RefCell<GenericLayout<W>>>,

    /// Horizontal scroll
    /// __read+write__
    pub hscroll: ScrollState,
    /// Vertical scroll
    /// __read+write__
    pub vscroll: ScrollState,

    /// This widget has no focus of its own, but this flag
    /// can be used to set a container state.
    pub container: FocusFlag,

    /// For the buffer to survive render()
    buffer: Option<Buffer>,

    /// Only construct with `..Default::default()`.
    pub non_exhaustive: NonExhaustive,
}

impl<W> Clone for Clipper<'_, W>
where
    W: Eq + Clone + Hash,
{
    fn clone(&self) -> Self {
        Self {
            style: Default::default(),
            block: self.block.clone(),
            layout: self.layout.clone(),
            hscroll: self.hscroll.clone(),
            vscroll: self.vscroll.clone(),
            label_style: self.label_style.clone(),
            label_alignment: self.label_alignment.clone(),
            auto_label: self.auto_label,
        }
    }
}

impl<W> Default for Clipper<'_, W>
where
    W: Eq + Clone + Hash,
{
    fn default() -> Self {
        Self {
            style: Default::default(),
            block: Default::default(),
            layout: Default::default(),
            hscroll: Default::default(),
            vscroll: Default::default(),
            label_style: Default::default(),
            label_alignment: Default::default(),
            auto_label: true,
        }
    }
}

impl<'a, W> Clipper<'a, W>
where
    W: Eq + Clone + Hash,
{
    /// New Clipper.
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
        self.style = style;
        self.block = self.block.map(|v| v.style(style));
        self
    }

    /// Render the label automatically when rendering the widget.
    ///
    /// Default: true
    pub fn auto_label(mut self, auto: bool) -> Self {
        self.auto_label = auto;
        self
    }

    /// Widget labels.
    pub fn label_style(mut self, style: Style) -> Self {
        self.label_style = Some(style);
        self
    }

    /// Widget labels.
    pub fn label_alignment(mut self, alignment: Alignment) -> Self {
        self.label_alignment = Some(alignment);
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
        self.style = styles.style;
        if styles.label_style.is_some() {
            self.label_style = styles.label_style;
        }
        if styles.label_alignment.is_some() {
            self.label_alignment = styles.label_alignment;
        }
        if styles.block.is_some() {
            self.block = styles.block;
        }
        if let Some(styles) = styles.scroll {
            self.hscroll = self.hscroll.map(|v| v.styles(styles.clone()));
            self.vscroll = self.vscroll.map(|v| v.styles(styles.clone()));
        }
        self.block = self.block.map(|v| v.style(styles.style));
        self
    }

    /// Calculate the layout width.
    pub fn layout_size(&self, area: Rect, state: &ClipperState<W>) -> Size {
        let width = self.inner(area, state).width;
        Size::new(width, u16::MAX)
    }

    /// Calculate the view area.
    fn inner(&self, area: Rect, state: &ClipperState<W>) -> Rect {
        let sa = ScrollArea::new()
            .block(self.block.as_ref())
            .h_scroll(self.hscroll.as_ref())
            .v_scroll(self.vscroll.as_ref());
        sa.inner(area, Some(&state.hscroll), Some(&state.vscroll))
    }

    fn calc_layout(&self, area: Rect, state: &mut ClipperState<W>) -> (Rect, Position) {
        let layout = state.layout.borrow();

        let view = Rect::new(
            state.hscroll.offset() as u16,
            state.vscroll.offset() as u16,
            area.width,
            area.height,
        );

        // maxima for scroll bar max
        let mut max_pos = Position::default();

        // find the bounding box for the buffer.
        // convex hull of all visible widgets/labels/blocks.
        let mut ext_view: Option<Rect> = None;
        for idx in 0..layout.widget_len() {
            let area = layout.widget(idx);
            let label_area = layout.label(idx);

            if view.intersects(area) || view.intersects(label_area) {
                if !area.is_empty() {
                    ext_view = ext_view //
                        .map(|v| v.union(area))
                        .or(Some(area));
                }
                if !label_area.is_empty() {
                    ext_view = ext_view //
                        .map(|v| v.union(label_area))
                        .or(Some(label_area));
                }
            }

            max_pos.x = max(max_pos.x, area.right());
            max_pos.y = max(max_pos.y, area.bottom());
            max_pos.x = max(max_pos.x, label_area.right());
            max_pos.y = max(max_pos.y, label_area.bottom());
        }
        for idx in 0..layout.block_len() {
            let block_area = layout.block_area(idx);
            if view.intersects(block_area) {
                ext_view = ext_view //
                    .map(|v| v.union(block_area))
                    .or(Some(block_area));
            }

            max_pos.x = max(max_pos.x, block_area.right());
            max_pos.y = max(max_pos.y, block_area.bottom());
        }

        let ext_view = ext_view.unwrap_or(view);

        (ext_view, max_pos)
    }

    /// Calculates the layout and creates a temporary buffer.
    pub fn into_buffer(mut self, area: Rect, state: &mut ClipperState<W>) -> ClipperBuffer<'a, W> {
        state.area = area;
        if let Some(layout) = self.layout.take() {
            state.layout = Rc::new(RefCell::new(layout));
        }

        let sa = ScrollArea::new()
            .block(self.block.as_ref())
            .h_scroll(self.hscroll.as_ref())
            .v_scroll(self.vscroll.as_ref());
        state.widget_area = sa.inner(area, Some(&state.hscroll), Some(&state.vscroll));

        // run the layout
        let (ext_area, max_pos) = self.calc_layout(area, state);

        // adjust scroll
        state
            .vscroll
            .set_page_len(state.widget_area.height as usize);
        state
            .vscroll
            .set_max_offset(max_pos.y.saturating_sub(state.widget_area.height) as usize);
        state.hscroll.set_page_len(state.widget_area.width as usize);
        state
            .hscroll
            .set_max_offset(max_pos.x.saturating_sub(state.widget_area.width) as usize);

        let offset = Position::new(state.hscroll.offset as u16, state.vscroll.offset as u16);

        // resize buffer to fit all visible widgets.
        let buffer_area = ext_area;
        // resize buffer to fit the layout.
        let mut buffer = if let Some(mut buffer) = state.buffer.take() {
            buffer.reset();
            buffer.resize(buffer_area);
            buffer
        } else {
            Buffer::empty(buffer_area)
        };
        buffer.set_style(buffer_area, self.style);

        ClipperBuffer {
            layout: state.layout.clone(),
            auto_label: self.auto_label,
            offset,
            buffer,
            widget_area: state.widget_area,
            style: self.style,
            block: self.block,
            hscroll: self.hscroll,
            vscroll: self.vscroll,
            label_style: self.label_style,
            label_alignment: self.label_alignment,
        }
    }
}

impl<'a, W> ClipperBuffer<'a, W>
where
    W: Eq + Hash + Clone,
{
    /// Is the widget visible.
    pub fn is_visible(&self, widget: W) -> bool {
        let layout = self.layout.borrow();
        let Some(idx) = layout.try_index_of(widget) else {
            return false;
        };
        let area = layout.widget(idx);
        self.buffer.area.intersects(area)
    }

    /// Render the label with the set style and alignment.
    #[inline(always)]
    fn render_auto_label(&mut self, idx: usize) -> bool {
        let layout = self.layout.borrow();
        let Some(label_area) = self.locate_area(layout.label(idx)) else {
            return false;
        };
        let Some(label_str) = layout.try_label_str(idx) else {
            return false;
        };

        let style = self.label_style.unwrap_or_default();
        let align = self.label_alignment.unwrap_or_default();
        Line::from(label_str.as_ref())
            .style(style)
            .alignment(align)
            .render(label_area, &mut self.buffer);

        true
    }

    /// Render all visible blocks.
    #[deprecated(since = "1.2", note = "happens automatically")]
    pub fn render_block(&mut self) {
        let layout = self.layout.borrow();
        for (idx, block_area) in layout.block_area_iter().enumerate() {
            if let Some(block_area) = self.locate_area(*block_area) {
                if let Some(block) = layout.block(idx) {
                    block.render(block_area, &mut self.buffer);
                }
            }
        }
    }

    /// Render the label for the given widget.
    #[inline(always)]
    pub fn render_label<FN>(&mut self, widget: W, render_fn: FN) -> bool
    where
        FN: FnOnce(&Option<Cow<'static, str>>, Rect, &mut Buffer),
    {
        let layout = self.layout.borrow();
        let Some(idx) = layout.try_index_of(widget) else {
            return false;
        };
        let Some(label_area) = self.locate_area(layout.label(idx)) else {
            return false;
        };
        let label_str = layout.try_label_str(idx);
        render_fn(label_str, label_area, &mut self.buffer);
        true
    }

    /// Render a stateless widget and its label.
    #[inline(always)]
    pub fn render_widget<FN, WW>(&mut self, widget: W, render_fn: FN) -> bool
    where
        FN: FnOnce() -> WW,
        WW: Widget,
    {
        let Some(idx) = self.layout.borrow().try_index_of(widget) else {
            return false;
        };
        if self.auto_label {
            self.render_auto_label(idx);
        }
        let Some(widget_area) = self.locate_area(self.layout.borrow().widget(idx)) else {
            return false;
        };
        render_fn().render(widget_area, &mut self.buffer);
        true
    }

    /// Render a stateful widget and its label.
    #[inline(always)]
    pub fn render<FN, WW, SS>(&mut self, widget: W, render_fn: FN, state: &mut SS) -> bool
    where
        FN: FnOnce() -> WW,
        WW: StatefulWidget<State = SS>,
        SS: RelocatableState,
    {
        let Some(idx) = self.layout.borrow().try_index_of(widget) else {
            return false;
        };
        if self.auto_label {
            self.render_auto_label(idx);
        }
        let Some(widget_area) = self.locate_area(self.layout.borrow().widget(idx)) else {
            self.hidden(state);
            return false;
        };
        render_fn().render(widget_area, &mut self.buffer, state);
        self.relocate(state);
        true
    }

    /// Get the buffer coordinates for the given widget.
    #[inline]
    pub fn locate_widget(&self, widget: W) -> Option<Rect> {
        let layout = self.layout.borrow();
        let Some(idx) = layout.try_index_of(widget) else {
            return None;
        };
        self.locate_area(layout.widget(idx))
    }

    /// Get the buffer coordinates for the label of the given widget.
    #[inline]
    #[allow(clippy::question_mark)]
    pub fn locate_label(&self, widget: W) -> Option<Rect> {
        let layout = self.layout.borrow();
        let Some(idx) = layout.try_index_of(widget) else {
            return None;
        };
        self.locate_area(layout.label(idx))
    }

    /// Relocate the area from layout coordinates to buffer coordinates,
    /// which is a noop as those are aligned.
    ///
    /// But this will return None if the given area is outside the buffer.
    #[inline]
    pub fn locate_area(&self, area: Rect) -> Option<Rect> {
        let area = self.buffer.area.intersection(area);
        if area.is_empty() { None } else { Some(area) }
    }

    /// Calculate the necessary shift from layout to screen.
    #[deprecated(since = "1.2.0", note = "will be made private")]
    pub fn shift(&self) -> (i16, i16) {
        (
            self.widget_area.x as i16 - self.offset.x as i16,
            self.widget_area.y as i16 - self.offset.y as i16,
        )
    }

    /// After rendering the widget to the buffer it may have
    /// stored areas in its state. These will be in buffer
    /// coordinates instead of screen coordinates.
    ///
    /// Call this function to correct this after rendering.
    #[deprecated(since = "1.2.0", note = "will be made private")]
    pub fn relocate<S>(&self, state: &mut S)
    where
        S: RelocatableState,
    {
        state.relocate(self.shift(), self.widget_area);
    }

    /// If a widget is not rendered because it is out of
    /// the buffer area, it may still have left over areas
    /// in its state.
    ///
    /// This uses the mechanism for [relocate](Self::relocate) to zero them out.
    #[deprecated(since = "1.2.0", note = "will be made private")]
    pub fn hidden<S>(&self, state: &mut S)
    where
        S: RelocatableState,
    {
        state.relocate((0, 0), Rect::default())
    }

    /// Return a reference to the buffer.
    #[inline]
    pub fn buffer(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    /// Rendering the content is finished.
    ///
    /// Convert to the output widget that can be rendered in the target area.
    #[allow(deprecated)]
    pub fn into_widget(mut self) -> ClipperWidget<'a, W> {
        self.render_block();

        ClipperWidget {
            block: self.block,
            hscroll: self.hscroll,
            vscroll: self.vscroll,
            offset: self.offset,
            buffer: self.buffer,
            phantom: Default::default(),
            style: self.style,
        }
    }
}

impl<W> StatefulWidget for ClipperWidget<'_, W>
where
    W: Eq + Clone + Hash,
{
    type State = ClipperState<W>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        assert_eq!(area, state.area);

        ScrollArea::new()
            .style(self.style)
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

        let src_area = self.buffer.area;
        let tgt_area = state.widget_area;
        let offset = self.offset;

        // extra offset due to buffer starts right of offset.
        let off_x0 = src_area.x.saturating_sub(offset.x);
        let off_y0 = src_area.y.saturating_sub(offset.y);
        // cut source buffer due to start left of offset.
        let cut_x0 = offset.x.saturating_sub(src_area.x);
        let cut_y0 = offset.y.saturating_sub(src_area.y);

        // length to copy
        let len_src = src_area.width.saturating_sub(cut_x0);
        let len_tgt = tgt_area.width.saturating_sub(off_x0);
        let len = min(len_src, len_tgt);

        // area height to copy
        let height_src = src_area.height.saturating_sub(cut_y0);
        let height_tgt = tgt_area.height.saturating_sub(off_y0);
        let height = min(height_src, height_tgt);

        // ** slow version **
        // for y in 0..height {
        //     for x in 0..len {
        //         let src_pos = Position::new(src_area.x + cut_x0 + x, src_area.y + cut_y0 + y);
        //         let src_cell = self.buffer.cell(src_pos).expect("src-cell");
        //
        //         let tgt_pos = Position::new(tgt_area.x + off_x0 + x, tgt_area.y + off_y0 + y);
        //         let tgt_cell = buf.cell_mut(tgt_pos).expect("tgt_cell");
        //
        //         *tgt_cell = src_cell.clone();
        //     }
        // }

        for y in 0..height {
            let src_0 = self
                .buffer
                .index_of(src_area.x + cut_x0, src_area.y + cut_y0 + y);
            let tgt_0 = buf.index_of(tgt_area.x + off_x0, tgt_area.y + off_y0 + y);

            let src = &self.buffer.content[src_0..src_0 + len as usize];
            let tgt = &mut buf.content[tgt_0..tgt_0 + len as usize];
            tgt.clone_from_slice(src);
        }

        // keep buffer
        state.buffer = Some(self.buffer);
    }
}

impl<W> Default for ClipperState<W>
where
    W: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self {
            area: Default::default(),
            widget_area: Default::default(),
            layout: Default::default(),
            hscroll: Default::default(),
            vscroll: Default::default(),
            container: Default::default(),
            buffer: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<W> Clone for ClipperState<W>
where
    W: Eq + Hash + Clone,
{
    fn clone(&self) -> Self {
        Self {
            area: self.area,
            widget_area: self.widget_area,
            layout: self.layout.clone(),
            hscroll: self.hscroll.clone(),
            vscroll: self.vscroll.clone(),
            container: FocusFlag::named(self.container.name()),
            buffer: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<W> HasFocus for ClipperState<W>
where
    W: Eq + Clone + Hash,
{
    fn build(&self, _builder: &mut FocusBuilder) {
        // not an autonomous widget
    }

    fn focus(&self) -> FocusFlag {
        self.container.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl<W> ClipperState<W>
where
    W: Eq + Clone + Hash,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear the layout data and reset any scroll
    pub fn clear(&mut self) {
        self.layout.borrow_mut().clear();
        self.hscroll.clear();
        self.vscroll.clear();
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

    /// Scroll to the given widget.
    pub fn show(&mut self, widget: W) -> bool {
        let layout = self.layout.borrow();
        let Some(idx) = layout.try_index_of(widget) else {
            return false;
        };
        let widget_area = layout.widget(idx);
        let label_area = layout.label(idx);

        let area = if !widget_area.is_empty() {
            if !label_area.is_empty() {
                Some(widget_area.union(label_area))
            } else {
                Some(widget_area)
            }
        } else {
            if !label_area.is_empty() {
                Some(label_area)
            } else {
                None
            }
        };

        if let Some(area) = area {
            let h = self
                .hscroll
                .scroll_to_range(area.left() as usize..area.right() as usize);
            let v = self
                .vscroll
                .scroll_to_range(area.top() as usize..area.bottom() as usize);
            h || v
        } else {
            false
        }
    }

    /// Returns the first visible widget.
    /// This uses insertion order of the widgets, not
    /// any graphical ordering.
    pub fn first(&self) -> Option<W> {
        let layout = self.layout.borrow();

        let area = Rect::new(
            self.hscroll.offset() as u16,
            self.vscroll.offset() as u16,
            self.widget_area.width,
            self.widget_area.height,
        );

        for idx in 0..layout.widget_len() {
            if layout.widget(idx).intersects(area) {
                return Some(layout.widget_key(idx).clone());
            }
        }

        None
    }
}

impl<W> ClipperState<W>
where
    W: Eq + Clone + Hash,
{
    pub fn vertical_offset(&self) -> usize {
        self.vscroll.offset()
    }

    pub fn set_vertical_offset(&mut self, offset: usize) -> bool {
        let old = self.vscroll.offset();
        self.vscroll.set_offset(offset);
        old != self.vscroll.offset()
    }

    pub fn vertical_page_len(&self) -> usize {
        self.vscroll.page_len()
    }

    pub fn horizontal_offset(&self) -> usize {
        self.hscroll.offset()
    }

    pub fn set_horizontal_offset(&mut self, offset: usize) -> bool {
        let old = self.hscroll.offset();
        self.hscroll.set_offset(offset);
        old != self.hscroll.offset()
    }

    pub fn horizontal_page_len(&self) -> usize {
        self.hscroll.page_len()
    }

    pub fn horizontal_scroll_to(&mut self, pos: usize) -> bool {
        self.hscroll.scroll_to_pos(pos)
    }

    pub fn vertical_scroll_to(&mut self, pos: usize) -> bool {
        self.vscroll.scroll_to_pos(pos)
    }

    /// Scroll the widget to visible.
    pub fn scroll_to(&mut self, widget: W) -> bool {
        self.show(widget)
    }

    pub fn scroll_up(&mut self, delta: usize) -> bool {
        self.vscroll.scroll_up(delta)
    }

    pub fn scroll_down(&mut self, delta: usize) -> bool {
        self.vscroll.scroll_down(delta)
    }

    pub fn scroll_left(&mut self, delta: usize) -> bool {
        self.hscroll.scroll_left(delta)
    }

    pub fn scroll_right(&mut self, delta: usize) -> bool {
        self.hscroll.scroll_right(delta)
    }
}

impl<W> HandleEvent<crossterm::event::Event, Regular, Outcome> for ClipperState<W>
where
    W: Eq + Clone + Hash,
{
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: Regular) -> Outcome {
        let r = if self.container.is_focused() {
            match event {
                ct_event!(keycode press PageUp) => self.scroll_up(self.vscroll.page_len()).into(),
                ct_event!(keycode press PageDown) => {
                    self.scroll_down(self.vscroll.page_len()).into()
                }
                ct_event!(keycode press Home) => self.vertical_scroll_to(0).into(),
                ct_event!(keycode press End) => {
                    self.vertical_scroll_to(self.vscroll.max_offset()).into()
                }
                _ => Outcome::Continue,
            }
        } else {
            Outcome::Continue
        };

        r.or_else(|| self.handle(event, MouseOnly))
    }
}

impl<W> HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for ClipperState<W>
where
    W: Eq + Clone + Hash,
{
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        let mut sas = ScrollAreaState::new()
            .area(self.widget_area)
            .h_scroll(&mut self.hscroll)
            .v_scroll(&mut self.vscroll);
        match sas.handle(event, MouseOnly) {
            ScrollOutcome::Up(v) => self.scroll_up(v).into(),
            ScrollOutcome::Down(v) => self.scroll_down(v).into(),
            ScrollOutcome::VPos(v) => self.set_vertical_offset(v).into(),
            ScrollOutcome::Left(v) => self.scroll_left(v).into(),
            ScrollOutcome::Right(v) => self.scroll_right(v).into(),
            ScrollOutcome::HPos(v) => self.set_horizontal_offset(v).into(),
            r => r.into(),
        }
    }
}
