use crate::clipper::{AreaHandle, ClipperLayout, ClipperStyle};
use rat_event::{HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::ContainerFlag;
use rat_reloc::RelocatableState;
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::{Scroll, ScrollArea, ScrollAreaState, ScrollState};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, StatefulWidget, Widget};
use std::cmp::min;

/// Configure the Clipper.
#[derive(Debug, Default, Clone)]
pub struct Clipper<'a> {
    layout: ClipperLayout,

    block: Option<Block<'a>>,
    hscroll: Option<Scroll<'a>>,
    vscroll: Option<Scroll<'a>>,
}

/// Render to the temp buffer.
///
/// * It maps your widget area from layout coordinates
///   to screen coordinates before rendering.
/// * It helps with cleanup of the widget state if your
///   widget is currently invisible.
#[derive(Debug)]
pub struct ClipperBuffer<'a> {
    // page layout
    layout: ClipperLayout,

    // Scroll offset into the xview.
    buf_offset_x: u16,
    buf_offset_y: u16,
    buffer: Buffer,

    // inner area that will finally be rendered.
    widget_area: Rect,

    block: Option<Block<'a>>,
    hscroll: Option<Scroll<'a>>,
    vscroll: Option<Scroll<'a>>,
}

/// Clips and copies the temp buffer to the frame buffer.
#[derive(Debug, Clone)]
pub struct ClipperWidget<'a> {
    // Scroll offset into the xview.
    buf_offset_x: u16,
    buf_offset_y: u16,
    buffer: Buffer,

    block: Option<Block<'a>>,
    hscroll: Option<Scroll<'a>>,
    vscroll: Option<Scroll<'a>>,
}

/// Clipper state.
#[derive(Debug, Default, Clone)]
pub struct ClipperState {
    /// Full area for the widget.
    /// __read only__ renewed for each render.
    pub area: Rect,
    /// Area inside the border.
    /// __read only__ renewed for each render.
    pub widget_area: Rect,

    /// Page layout.
    /// __read only__ renewed for each render.
    pub layout: ClipperLayout,

    /// Horizontal scroll
    /// __read+write__
    pub hscroll: ScrollState,
    /// Vertical scroll
    /// __read+write__
    pub vscroll: ScrollState,

    /// This widget has no focus of its own, but this flag
    /// can be used to set a container state.
    pub c_focus: ContainerFlag,

    /// For the buffer to survive render()
    buffer: Option<Buffer>,
}

impl<'a> Clipper<'a> {
    /// New Clipper.
    pub fn new() -> Self {
        Self::default()
    }

    /// Page layout.
    pub fn layout(mut self, page_layout: ClipperLayout) -> Self {
        self.layout = page_layout;
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
        if styles.block.is_some() {
            self.block = styles.block;
        }
        if let Some(styles) = styles.scroll {
            self.hscroll = self.hscroll.map(|v| v.styles(styles.clone()));
            self.vscroll = self.vscroll.map(|v| v.styles(styles.clone()));
        }
        self
    }

    /// Calculate the layout width.
    pub fn layout_width(&self, area: Rect, state: &ClipperState) -> u16 {
        self.inner(area, state).width
    }

    /// Calculate the view area.
    pub fn inner(&self, area: Rect, state: &ClipperState) -> Rect {
        let sa = ScrollArea::new()
            .block(self.block.as_ref())
            .h_scroll(self.hscroll.as_ref())
            .v_scroll(self.vscroll.as_ref());
        sa.inner(area, Some(&state.hscroll), Some(&state.vscroll))
    }

    /// Calculates the layout and creates a temporary buffer.
    pub fn into_buffer(self, area: Rect, state: &mut ClipperState) -> ClipperBuffer<'a> {
        state.area = area;
        state.layout = self.layout;

        let sa = ScrollArea::new()
            .block(self.block.as_ref())
            .h_scroll(self.hscroll.as_ref())
            .v_scroll(self.vscroll.as_ref());
        state.widget_area = sa.inner(area, Some(&state.hscroll), Some(&state.vscroll));

        // run the layout
        let ext_area = state.layout.layout(Rect::new(
            state.hscroll.offset as u16,
            state.vscroll.offset as u16,
            state.widget_area.width,
            state.widget_area.height,
        ));

        // adjust scroll
        let (max_x, max_y) = state.layout.max_layout_pos();
        state
            .vscroll
            .set_page_len(state.widget_area.height as usize);
        state
            .vscroll
            .set_max_offset(max_y.saturating_sub(state.widget_area.height) as usize);
        state.hscroll.set_page_len(state.widget_area.width as usize);
        state
            .hscroll
            .set_max_offset(max_x.saturating_sub(state.widget_area.width) as usize);

        // offset is in layout coordinates.
        // internal buffer starts at (0,0).
        let buf_offset_x = state.hscroll.offset as u16 - ext_area.x;
        let buf_offset_y = state.vscroll.offset as u16 - ext_area.y;

        // resize buffer to fit all visible widgets.
        let buffer_area = Rect::new(0, 0, ext_area.width, ext_area.height);
        let buffer = if let Some(mut buffer) = state.buffer.take() {
            buffer.reset();
            buffer.resize(buffer_area);
            buffer
        } else {
            Buffer::empty(buffer_area)
        };

        ClipperBuffer {
            layout: state.layout.clone(),
            buf_offset_x,
            buf_offset_y,
            buffer,
            widget_area: state.widget_area,
            block: self.block,
            hscroll: self.hscroll,
            vscroll: self.vscroll,
        }
    }
}

impl<'a> ClipperBuffer<'a> {
    /// Render a widget to the temp buffer.
    #[inline(always)]
    pub fn render_widget<W>(&mut self, widget: W, area: Rect)
    where
        W: Widget,
    {
        if let Some(buffer_area) = self.layout.buf_area(area) {
            // render the actual widget.
            widget.render(buffer_area, self.buffer_mut());
        }
    }

    /// Render a widget to the temp buffer.
    /// This expects that the state is a RelocatableState,
    /// so it can reset the areas for hidden widgets.
    #[inline(always)]
    pub fn render_stateful<W, S>(&mut self, widget: W, area: Rect, state: &mut S)
    where
        W: StatefulWidget<State = S>,
        S: RelocatableState,
    {
        if let Some(buffer_area) = self.layout.buf_area(area) {
            // render the actual widget.
            widget.render(buffer_area, self.buffer_mut(), state);
            // shift and clip the output areas.
            self.relocate(state);
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
        if let Some(buffer_area) = self.layout.buf_area_by_handle(area) {
            // render the actual widget.
            widget.render(buffer_area, self.buffer_mut());
        }
    }

    /// Render a widget to the temp buffer.
    /// This expects that the state is a RelocatableState,
    /// so it can reset the areas for hidden widgets.
    #[inline(always)]
    pub fn render_stateful_handle<W, S>(&mut self, widget: W, area: AreaHandle, state: &mut S)
    where
        W: StatefulWidget<State = S>,
        S: RelocatableState,
    {
        if let Some(buffer_area) = self.layout.buf_area_by_handle(area) {
            // render the actual widget.
            widget.render(buffer_area, self.buffer_mut(), state);
            // shift and clip the output areas.
            self.relocate(state);
        } else {
            self.relocate_clear(state);
        }
    }

    /// Get the layout area for the handle.
    /// This uses layout coordinates.
    pub fn layout_area(&self, handle: AreaHandle) -> Rect {
        self.layout.layout_area_by_handle(handle)
    }

    /// Get the buffer area for the handle.
    /// This uses coordinates that fit the temp buffer.
    pub fn buf_area(&self, handle: AreaHandle) -> Option<Rect> {
        self.layout.buf_area_by_handle(handle)
    }

    /// Convert the layout coordinates to buffer coordinates.
    pub fn layout_area_to_buf_area(&self, area: Rect) -> Option<Rect> {
        self.layout.buf_area(area)
    }

    /// Is the given area visible?
    pub fn is_visible(&self, area: Rect) -> bool {
        self.layout.buf_area(area).is_some()
    }

    /// Is the given area visible?
    pub fn is_visible_handle(&self, handle: AreaHandle) -> bool {
        self.layout.buf_area_by_handle(handle).is_some()
    }

    /// Calculate the necessary shift from xview to screen.
    pub fn shift(&self) -> (i16, i16) {
        (
            self.widget_area.x as i16 - self.buf_offset_x as i16,
            self.widget_area.y as i16 - self.buf_offset_y as i16,
        )
    }

    /// Relocate all the widget state that refers to an area to
    /// the actual screen space used.
    pub fn relocate<S>(&self, state: &mut S)
    where
        S: RelocatableState,
    {
        state.relocate(self.shift(), self.widget_area);
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
    pub fn into_widget(self) -> ClipperWidget<'a> {
        ClipperWidget {
            buf_offset_x: self.buf_offset_x,
            buf_offset_y: self.buf_offset_y,
            block: self.block,
            hscroll: self.hscroll,
            vscroll: self.vscroll,
            buffer: self.buffer,
        }
    }
}

impl<'a> StatefulWidget for ClipperWidget<'a> {
    type State = ClipperState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        assert_eq!(area, state.area);

        ScrollArea::new()
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

        let inner_area = state.widget_area;

        let copy_width = min(inner_area.width, self.buffer.area.width) as usize;

        for y in 0..inner_area.height {
            let buf0 = self
                .buffer
                .index_of(self.buf_offset_x, self.buf_offset_y + y);
            let tgt0 = buf.index_of(inner_area.x, inner_area.y + y);
            buf.content[tgt0..tgt0 + copy_width]
                .clone_from_slice(&self.buffer.content[buf0..buf0 + copy_width]);
        }

        // keep buffer
        state.buffer = Some(self.buffer);
    }
}

impl ClipperState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the area for the given handle.
    pub fn show_handle(&mut self, handle: AreaHandle) {
        let area = self.layout.layout_area_by_handle(handle);
        self.hscroll.scroll_to_pos(area.x as usize);
        self.vscroll.scroll_to_pos(area.y as usize);
    }

    /// Show this rect in layout coordinates.
    pub fn show_area(&mut self, area: Rect) {
        self.hscroll.scroll_to_pos(area.x as usize);
        self.vscroll.scroll_to_pos(area.y as usize);
    }

    /// First handle for the page.
    /// This returns the first handle for the page.
    /// Does not check whether the connected area is visible.
    pub fn first_handle(&self) -> Option<AreaHandle> {
        self.layout.first_layout_handle()
    }

    /// First handle for the page.
    /// This returns the first handle for the page.
    /// Does not check whether the connected area is visible.
    pub fn first_area(&self) -> Option<Rect> {
        self.layout.first_layout_area()
    }
}

impl ClipperState {
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

impl HandleEvent<crossterm::event::Event, Regular, Outcome> for ClipperState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: Regular) -> Outcome {
        self.handle(event, MouseOnly)
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for ClipperState {
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
