//! A view allows scrolling of on or more widgets without builtin
//! support for scrolling.
//!
//! ```rust
//! # use rat_scrolled::Scroll;
//! use rat_widget::paragraph::{Paragraph, ParagraphState};
//! # use rat_widget::view::{View, ViewState};
//! # use ratatui_core::layout::Rect;
//! # use ratatui_core::buffer::Buffer;
//! #
//! # let l2 = [Rect::ZERO, Rect::ZERO];
//! # struct State {
//! #      view: ViewState,
//! #      first: ParagraphState,
//! #  }
//! # let mut state = State {
//! #     view: Default::default(),
//! #     first: Default::default(),
//! # };
//! # let mut buf = Buffer::default();
//!
//! ///
//! /// Create the view and set the layout area
//! /// for the buffer.
//! ///
//!
//! let mut view_buf = View::new()
//!     .layout(Rect::new(0, 0, 400, 400))
//!     .vscroll(Scroll::new())
//!     .hscroll(Scroll::new())
//!     .into_buffer(l2[1], &mut state.view);
//!
//! ///
//! /// Render the widgets to the view buffer.
//! ///
//! view_buf.render(
//!     Paragraph::new("Paragraph\nParagraph\n..."),
//!     Rect::new(0, 0, 40, 15),
//!     &mut state.first,
//! );
//!
//! ///
//! /// Render the finished buffer.
//! ///
//! view_buf.finish(&mut buf, &mut state.view);
//!
//! ```

use crate::_private::NonExhaustive;
use crate::event::ScrollOutcome;
use rat_event::{ConsumedEvent, HandleEvent, MouseOnly, Outcome, Regular, ct_event};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_reloc::RelocatableState;
use rat_scrolled::{Scroll, ScrollArea, ScrollAreaState, ScrollState, ScrollStyle};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect, Size};
use ratatui_core::style::Style;
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::Block;
use std::cmp::min;
use std::mem;

/// Configure the view.
#[derive(Debug, Default, Clone)]
pub struct View<'a> {
    view_layout: Option<Rect>,
    view_x: Option<u16>,
    view_y: Option<u16>,
    view_width: Option<u16>,
    view_height: Option<u16>,

    style: Style,
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
pub struct ViewBuffer<'a> {
    // Scroll offset into the view.
    offset: Position,
    buffer: Buffer,

    // inner area that will finally be rendered.
    widget_area: Rect,

    style: Style,
    block: Option<Block<'a>>,
    hscroll: Option<Scroll<'a>>,
    vscroll: Option<Scroll<'a>>,

    destruct: bool,
}

/// Clips and copies the temp buffer to the frame buffer.
// todo: deprecate
#[derive(Debug)]
pub struct ViewWidget<'a> {
    // Scroll offset into the view.
    offset: Position,
    buffer: Buffer,

    style: Style,
    block: Option<Block<'a>>,
    hscroll: Option<Scroll<'a>>,
    vscroll: Option<Scroll<'a>>,
}

/// All styles for a view.
#[derive(Debug, Clone)]
pub struct ViewStyle {
    pub style: Style,
    pub block: Option<Block<'static>>,
    pub border_style: Option<Style>,
    pub title_style: Option<Style>,
    pub scroll: Option<ScrollStyle>,
    pub non_exhaustive: NonExhaustive,
}

/// View state.
#[derive(Debug, Default, Clone)]
pub struct ViewState {
    /// Full area for the widget.
    /// __read only__ renewed for each render.
    pub area: Rect,
    /// Area inside the border.
    /// __read only__ renewed for each render.
    pub widget_area: Rect,

    /// The layout of the temp buffer uses.
    /// __read only__ renewed for each render.
    pub layout: Rect,

    /// Horizontal scroll
    /// __read+write__
    pub hscroll: ScrollState,
    /// Vertical scroll
    /// __read+write__
    pub vscroll: ScrollState,

    /// Current focus state.
    /// __read+write__
    pub focus: FocusFlag,

    /// For the buffer to survive render()
    buffer: Option<Buffer>,
}

impl<'a> View<'a> {
    /// New View.
    pub fn new() -> Self {
        Self::default()
    }

    /// Size of the view buffer.
    pub fn layout(mut self, area: Rect) -> Self {
        self.view_layout = Some(area);
        self
    }

    /// Width of the view buffer.
    pub fn view_width(mut self, width: u16) -> Self {
        self.view_width = Some(width);
        self
    }

    /// Width of the view buffer.
    pub fn view_height(mut self, height: u16) -> Self {
        self.view_height = Some(height);
        self
    }
    /// Start position of the view buffer.
    pub fn view_x(mut self, x: u16) -> Self {
        self.view_x = Some(x);
        self
    }

    /// Start position of the view buffer.
    pub fn view_y(mut self, y: u16) -> Self {
        self.view_y = Some(y);
        self
    }

    /// Size of the view buffer.
    pub fn view_size(mut self, view: Size) -> Self {
        self.view_width = Some(view.width);
        self.view_height = Some(view.height);
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
    pub fn styles(mut self, styles: ViewStyle) -> Self {
        self.style = styles.style;
        if styles.block.is_some() {
            self.block = styles.block;
        }
        if let Some(border_style) = styles.border_style {
            self.block = self.block.map(|v| v.border_style(border_style));
        }
        if let Some(title_style) = styles.title_style {
            self.block = self.block.map(|v| v.title_style(title_style));
        }
        self.block = self.block.map(|v| v.style(self.style));
        if let Some(styles) = styles.scroll {
            self.hscroll = self.hscroll.map(|v| v.styles(styles.clone()));
            self.vscroll = self.vscroll.map(|v| v.styles(styles.clone()));
        }
        self
    }

    /// Calculate the layout width.
    #[allow(deprecated)]
    pub fn layout_size(&self, area: Rect, state: &ViewState) -> u16 {
        self.inner(area, state).width
    }

    /// Calculate the layout width.
    #[allow(deprecated)]
    pub fn layout_width(&self, area: Rect, state: &ViewState) -> u16 {
        self.inner(area, state).width
    }

    /// Calculate the view area.
    #[deprecated(since = "2.3.0", note = "use layout_size instead")]
    pub fn inner(&self, area: Rect, state: &ViewState) -> Rect {
        let sa = ScrollArea::new()
            .block(self.block.as_ref())
            .h_scroll(self.hscroll.as_ref())
            .v_scroll(self.vscroll.as_ref());
        sa.inner(area, Some(&state.hscroll), Some(&state.vscroll))
    }

    /// Calculates the layout and creates a temporary buffer.
    pub fn into_buffer(self, area: Rect, state: &mut ViewState) -> ViewBuffer<'a> {
        state.area = area;

        let sa = ScrollArea::new()
            .block(self.block.as_ref())
            .h_scroll(self.hscroll.as_ref())
            .v_scroll(self.vscroll.as_ref());
        state.widget_area = sa.inner(area, Some(&state.hscroll), Some(&state.vscroll));

        state.layout = if let Some(layout) = self.view_layout {
            layout
        } else {
            let mut layout = Rect::new(0, 0, state.widget_area.width, state.widget_area.height);
            if let Some(x) = self.view_x {
                layout.x = x;
            }
            if let Some(y) = self.view_y {
                layout.y = y;
            }
            if let Some(width) = self.view_width {
                layout.width = width;
            }
            if let Some(height) = self.view_height {
                layout.height = height;
            }
            layout
        };

        state
            .hscroll
            .set_max_offset(state.layout.right().saturating_sub(state.widget_area.width) as usize);
        state.hscroll.set_page_len(state.widget_area.width as usize);

        state.vscroll.set_max_offset(
            state
                .layout
                .right()
                .saturating_sub(state.widget_area.height) as usize,
        );
        state
            .vscroll
            .set_page_len(state.widget_area.height as usize);

        // offset is in layout coordinates.
        let offset = Position::new(state.hscroll.offset as u16, state.vscroll.offset as u16);

        // resize buffer to fit the layout.
        let mut buffer = if let Some(mut buffer) = state.buffer.take() {
            buffer.reset();
            buffer.resize(state.layout);
            buffer
        } else {
            Buffer::empty(state.layout)
        };
        buffer.set_style(state.layout, self.style);

        ViewBuffer {
            offset,
            buffer,
            widget_area: state.widget_area,
            style: self.style,
            block: self.block,
            hscroll: self.hscroll,
            vscroll: self.vscroll,
            destruct: false,
        }
    }
}

impl<'a> Drop for ViewBuffer<'a> {
    fn drop(&mut self) {
        if !self.destruct {
            panic!("ViewBuffer: Must be used. Call finish(..)");
        }
    }
}

impl<'a> ViewBuffer<'a> {
    /// Render a widget to the temp buffer.
    #[inline(always)]
    pub fn render_widget<W>(&mut self, widget: W, area: Rect) -> bool
    where
        W: Widget,
    {
        if area.intersects(self.buffer.area) {
            // render the actual widget.
            widget.render(area, self.buffer());
            true
        } else {
            false
        }
    }

    /// Render a widget to the temp buffer.
    /// This expects that the state is a [RelocatableState].
    #[inline(always)]
    #[allow(deprecated)]
    pub fn render<W, S>(&mut self, widget: W, area: Rect, state: &mut S) -> bool
    where
        W: StatefulWidget<State = S>,
        S: RelocatableState,
    {
        if area.intersects(self.buffer.area) {
            // render the actual widget.
            widget.render(area, self.buffer(), state);
            // shift and clip the output areas.
            state.relocate(self.shift(), self.widget_area);
            true
        } else {
            state.relocate_hidden();
            false
        }
    }

    /// Render an additional popup widget for the given main widget.
    ///
    /// Doesn't call relocate().
    #[inline(always)]
    #[allow(deprecated)]
    pub fn render_popup<W, S>(&mut self, widget: W, area: Rect, state: &mut S) -> bool
    where
        W: StatefulWidget<State = S>,
        S: RelocatableState,
    {
        if area.intersects(self.buffer.area) {
            // render the actual widget.
            widget.render(area, self.buffer(), state);
            // shift and clip the output areas.
            state.relocate_popup(self.shift(), self.widget_area);
            true
        } else {
            state.relocate_popup_hidden();
            false
        }
    }

    /// Return the buffer layout.
    pub fn layout(&self) -> Rect {
        self.buffer.area
    }

    /// Is this area inside the buffer area.
    pub fn is_visible_area(&self, area: Rect) -> bool {
        area.intersects(self.buffer.area)
    }

    /// Calculate the necessary shift from view to screen.
    ///
    /// __Note__
    ///
    pub fn shift(&self) -> (i16, i16) {
        (
            self.widget_area.x as i16 - self.offset.x as i16,
            self.widget_area.y as i16 - self.offset.y as i16,
        )
    }

    /// Experimental.
    pub fn clip(&self) -> Rect {
        self.widget_area
    }

    /// Does nothing for view.
    /// Only exists to match [Clipper](crate::clipper::Clipper).
    #[deprecated(
        since = "2.0.0",
        note = "wrong api, use is_visible_area() or locate_area2()"
    )]
    pub fn locate_area(&self, area: Rect) -> Rect {
        area
    }

    /// Validates that this area is inside the buffer area.
    pub fn locate_area2(&self, area: Rect) -> Option<Rect> {
        if area.intersects(self.buffer.area) {
            Some(area)
        } else {
            None
        }
    }

    /// After rendering the widget to the buffer it may have
    /// stored areas in its state. These will be in buffer
    /// coordinates instead of screen coordinates.
    ///
    /// Call this function to correct this after rendering.
    #[deprecated(since = "2.0.0", note = "wrong api, use relocate2() instead")]
    #[allow(deprecated)]
    pub fn relocate<S>(&self, state: &mut S)
    where
        S: RelocatableState,
    {
        state.relocate(self.shift(), self.clip());
    }

    /// After rendering the widget to the buffer it may have
    /// stored areas in its state. These will be in buffer
    /// coordinates instead of screen coordinates.
    ///
    /// Call this function to correct this after rendering.
    ///
    /// It needs the render-area of the widget to find out
    /// if the widget will be display
    #[allow(deprecated)]
    pub fn relocate2<S>(&self, area: Rect, state: &mut S)
    where
        S: RelocatableState,
    {
        if self.is_visible_area(area) {
            state.relocate(self.shift(), self.widget_area);
        } else {
            state.relocate_hidden();
        }
    }

    /// If a widget is not rendered because it is out of
    /// the buffer area, it may still have left over areas
    /// in its state.
    ///
    /// This uses [relocate_hidden](RelocatableState::relocate_hidden) to zero them out.
    #[deprecated(since = "2.0.0", note = "bad api, use relocate2() instead")]
    pub fn hidden<S>(&self, state: &mut S)
    where
        S: RelocatableState,
    {
        state.relocate_hidden();
    }

    /// Access the temporary buffer.
    ///
    /// __Note__
    /// Use of render_widget is preferred.
    pub fn buffer(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    /// Rendering the content is finished.
    ///
    /// Convert to the output widget that can be rendered in the target area.
    #[deprecated(since = "2.3.0", note = "use finish() instead")]
    pub fn into_widget(mut self) -> ViewWidget<'a> {
        self.destruct = true;

        ViewWidget {
            block: mem::take(&mut self.block),
            hscroll: mem::take(&mut self.hscroll),
            vscroll: mem::take(&mut self.vscroll),
            offset: self.offset,
            buffer: mem::take(&mut self.buffer),
            style: self.style,
        }
    }

    /// Render the buffer.
    pub fn finish(mut self, tgt_buf: &mut Buffer, state: &mut ViewState) {
        self.destruct = true;

        ScrollArea::new()
            .style(self.style)
            .block(self.block.as_ref())
            .h_scroll(self.hscroll.as_ref())
            .v_scroll(self.vscroll.as_ref())
            .render(
                state.area,
                tgt_buf,
                &mut ScrollAreaState::new()
                    .h_scroll(&mut state.hscroll)
                    .v_scroll(&mut state.vscroll),
            );

        let v_src = Rect::new(
            self.offset.x,
            self.offset.y,
            state.widget_area.width,
            state.widget_area.height,
        );
        if !v_src.intersects(self.buffer.area) {
            return;
        }
        let mut src = self.buffer.area.intersection(v_src);

        let mut view = state.widget_area;
        if src.x > self.offset.x {
            view.x += src.x - self.offset.x;
            view.width -= src.x - self.offset.x;
        }
        if src.y > self.offset.y {
            view.y += src.y - self.offset.y;
            view.height -= src.y - self.offset.y;
        }

        let width = min(view.width, src.width);
        let height = min(view.height, src.height);

        src.width = width;
        view.width = width;
        src.height = height;
        view.height = height;

        for y in 0..src.height {
            let src_0 = self.buffer.index_of(src.x, src.y + y);
            let src_len = src.width as usize;
            let view_0 = tgt_buf.index_of(view.x, view.y + y);
            let view_len = view.width as usize;
            assert_eq!(src_len, view_len);

            let src = &self.buffer.content[src_0..src_0 + src_len];
            let tgt = &mut tgt_buf.content[view_0..view_0 + view_len];
            tgt.clone_from_slice(src);
        }

        // keep buffer
        state.buffer = Some(mem::take(&mut self.buffer));
    }
}

impl StatefulWidget for ViewWidget<'_> {
    type State = ViewState;

    fn render(mut self, area: Rect, tgt_buf: &mut Buffer, state: &mut Self::State) {
        if cfg!(debug_assertions) {
            if area != state.area {
                panic!(
                    "ViewWidget::render() must be called with the same area as View::into_buffer()."
                )
            }
        }

        ScrollArea::new()
            .style(self.style)
            .block(self.block.as_ref())
            .h_scroll(self.hscroll.as_ref())
            .v_scroll(self.vscroll.as_ref())
            .render(
                state.area,
                tgt_buf,
                &mut ScrollAreaState::new()
                    .h_scroll(&mut state.hscroll)
                    .v_scroll(&mut state.vscroll),
            );

        let v_src = Rect::new(
            self.offset.x,
            self.offset.y,
            state.widget_area.width,
            state.widget_area.height,
        );
        if !v_src.intersects(self.buffer.area) {
            return;
        }
        let mut src = self.buffer.area.intersection(v_src);

        let mut view = state.widget_area;
        if src.x > self.offset.x {
            view.x += src.x - self.offset.x;
            view.width -= src.x - self.offset.x;
        }
        if src.y > self.offset.y {
            view.y += src.y - self.offset.y;
            view.height -= src.y - self.offset.y;
        }

        let width = min(view.width, src.width);
        let height = min(view.height, src.height);

        src.width = width;
        view.width = width;
        src.height = height;
        view.height = height;

        for y in 0..src.height {
            let src_0 = self.buffer.index_of(src.x, src.y + y);
            let src_len = src.width as usize;
            let view_0 = tgt_buf.index_of(view.x, view.y + y);
            let view_len = view.width as usize;
            assert_eq!(src_len, view_len);

            let src = &self.buffer.content[src_0..src_0 + src_len];
            let tgt = &mut tgt_buf.content[view_0..view_0 + view_len];
            tgt.clone_from_slice(src);
        }

        // keep buffer
        state.buffer = Some(mem::take(&mut self.buffer));
    }
}

impl Default for ViewStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            block: Default::default(),
            border_style: Default::default(),
            title_style: Default::default(),
            scroll: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl HasFocus for ViewState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl RelocatableState for ViewState {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.area.relocate(shift, clip);
        self.widget_area.relocate(shift, clip);
        self.hscroll.relocate(shift, clip);
        self.vscroll.relocate(shift, clip);
    }
}

impl ViewState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Show this rect.
    pub fn show_area(&mut self, area: Rect) {
        self.hscroll.scroll_to_pos(area.x as usize);
        self.vscroll.scroll_to_pos(area.y as usize);
    }
}

impl ViewState {
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

impl HandleEvent<Event, Regular, Outcome> for ViewState {
    fn handle(&mut self, event: &Event, _qualifier: Regular) -> Outcome {
        let r = if self.is_focused() {
            match event {
                ct_event!(keycode press Left) => self.scroll_left(self.hscroll.scroll_by()).into(),
                ct_event!(keycode press Right) => {
                    self.scroll_right(self.hscroll.scroll_by()).into()
                }
                ct_event!(keycode press Up) => self.scroll_up(self.vscroll.scroll_by()).into(),
                ct_event!(keycode press Down) => self.scroll_down(self.vscroll.scroll_by()).into(),

                ct_event!(keycode press PageUp) => self.scroll_up(self.vscroll.page_len()).into(),
                ct_event!(keycode press PageDown) => {
                    self.scroll_down(self.vscroll.page_len()).into()
                }
                ct_event!(keycode press Home) => self.vertical_scroll_to(0).into(),
                ct_event!(keycode press End) => {
                    self.vertical_scroll_to(self.vscroll.max_offset()).into()
                }

                ct_event!(keycode press ALT-PageUp) => {
                    self.scroll_left(self.hscroll.page_len()).into()
                }
                ct_event!(keycode press ALT-PageDown) => {
                    self.scroll_right(self.hscroll.page_len()).into()
                }
                ct_event!(keycode press ALT-Home) => self.horizontal_scroll_to(0).into(),
                ct_event!(keycode press ALT-End) => {
                    self.horizontal_scroll_to(self.hscroll.max_offset()).into()
                }
                _ => Outcome::Continue,
            }
        } else {
            Outcome::Continue
        };

        r.or_else(|| self.handle(event, MouseOnly))
    }
}

impl HandleEvent<Event, MouseOnly, Outcome> for ViewState {
    fn handle(&mut self, event: &Event, _qualifier: MouseOnly) -> Outcome {
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

/// Handle all events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events(state: &mut ViewState, focus: bool, event: &Event) -> Outcome {
    state.focus.set(focus);
    HandleEvent::handle(state, event, Regular)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(state: &mut ViewState, event: &Event) -> Outcome {
    HandleEvent::handle(state, event, MouseOnly)
}
