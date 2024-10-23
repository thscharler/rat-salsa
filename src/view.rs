//! A xview allows scrolling of on or more widgets without builtin
//! support for scrolling.
//!
//! View works in 3 phases:
//!
//! * Configuration
//!
//! ```rust ignore
//!     let mut view_buf = View::new()
//!         .xview(Rect::new(0, 0, 400, 400))
//!         .vscroll(Scroll::new())
//!         .hscroll(Scroll::new())
//!         .into_buffer(layout[1], &mut state.view_state);
//! ```
//!
//! * Rendering the widgets
//!
//! The into_render() call returns a struct which can render
//! widgets to an internal buffer.
//!
//! ```rust ignore
//!     view_buf.render_stateful(
//!         Paragraph::new(data.sample1.clone()),
//!         Rect::new(0, 0, 40, 15),
//!         &mut state.first,
//!     );
//! ```
//! This uses the views coordinate system. If you have a
//! stateful widget, it may keep Rect's in its state, that
//! would now also be in the views coordinate system.
//!
//! The widget can implement the trait [RelocatableState] to
//! deal with this situation. It provides information how to
//! shift and clip the widgets areas.
//!
//! * Rendering the buffer
//!
//! This panics if you give it a different area than to into_buffer(),
//! but otherwise it just renders the buffer contents.
//!
//! ```rust ignore
//!     view_buf
//!         .into_widget()
//!         .render(layout[1], frame.buffer_mut(), &mut state.view_state);
//! ```
//!

mod view_style;
pub use view_style::*;

use crate::event::ScrollOutcome;
use crate::relocate::RelocatableState;
use rat_event::{HandleEvent, MouseOnly, Outcome, Regular};
use rat_scrolled::{Scroll, ScrollArea, ScrollAreaState, ScrollState, ScrollStyle};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::prelude::{StatefulWidget, Widget};
use ratatui::widgets::Block;

/// Configure the view.
#[derive(Debug, Default, Clone)]
pub struct View<'a> {
    layout: Rect,

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
#[derive(Debug, Clone)]
pub struct ViewBuffer<'a> {
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

/// Rendering widget for View.
#[derive(Debug, Clone)]
pub struct ViewWidget<'a> {
    // Scroll offset into the xview.
    buf_offset_x: u16,
    buf_offset_y: u16,
    buffer: Buffer,

    block: Option<Block<'a>>,
    hscroll: Option<Scroll<'a>>,
    vscroll: Option<Scroll<'a>>,
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

    /// The viewport area that the inner widget uses.
    /// __read only__ renewed for each render.
    pub view: Rect,

    /// Horizontal scroll
    /// __read+write__
    pub hscroll: ScrollState,
    /// Vertical scroll
    /// __read+write__
    pub vscroll: ScrollState,

    /// For the buffer to survive render()
    buffer: Option<Buffer>,
}

impl<'a> View<'a> {
    /// New View.
    pub fn new() -> Self {
        Self::default()
    }

    /// Area for the temp buffer.
    pub fn view(mut self, area: Rect) -> Self {
        self.layout = area;
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
        if styles.block.is_some() {
            self.block = styles.block;
        }
        if let Some(styles) = styles.scroll {
            self.hscroll = self.hscroll.map(|v| v.styles(styles.clone()));
            self.vscroll = self.vscroll.map(|v| v.styles(styles.clone()));
        }
        self
    }

    /// Calculate the view area.
    pub fn inner(&self, area: Rect, state: &ViewState) -> Rect {
        let sa = ScrollArea::new()
            .block(self.block.as_ref())
            .h_scroll(self.hscroll.as_ref())
            .v_scroll(self.vscroll.as_ref());
        sa.inner(area, Some(&state.hscroll), Some(&state.vscroll))
    }

    /// Create the temporary buffer.
    pub fn into_buffer(self, area: Rect, state: &mut ViewState) -> ViewBuffer<'a> {
        state.area = area;
        state.view = self.layout;

        let sa = ScrollArea::new()
            .block(self.block.as_ref())
            .h_scroll(self.hscroll.as_ref())
            .v_scroll(self.vscroll.as_ref());
        state.widget_area = sa.inner(area, Some(&state.hscroll), Some(&state.vscroll));

        state
            .hscroll
            .set_max_offset(state.view.width.saturating_sub(state.widget_area.width) as usize);
        state.hscroll.set_page_len(state.widget_area.width as usize);
        state
            .vscroll
            .set_max_offset(state.view.height.saturating_sub(state.widget_area.height) as usize);
        state
            .vscroll
            .set_page_len(state.widget_area.height as usize);

        // internal buffer starts at (xview.x,xview.y)
        let buf_offset_x = state.hscroll.offset as u16 + self.layout.x;
        let buf_offset_y = state.vscroll.offset as u16 + self.layout.y;

        // resize buffer to fit all visible widgets.
        let buffer_area = state.view;
        let buffer = if let Some(mut buffer) = state.buffer.take() {
            buffer.reset();
            buffer.resize(buffer_area);
            buffer
        } else {
            Buffer::empty(buffer_area)
        };

        ViewBuffer {
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

impl<'a> ViewBuffer<'a> {
    /// Render a widget to the temp buffer.
    #[inline(always)]
    pub fn render_widget<W>(&mut self, widget: W, area: Rect)
    where
        W: Widget,
    {
        if area.intersects(self.buffer.area) {
            // render the actual widget.
            widget.render(area, self.buffer_mut());
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
        if area.intersects(self.buffer.area) {
            // render the actual widget.
            widget.render(area, self.buffer_mut(), state);
            // shift and clip the output areas.
            self.relocate(state);
        } else {
            self.relocate_clear(state);
        }
    }

    /// Is the given area visible?
    pub fn is_visible(&self, area: Rect) -> bool {
        area.intersects(self.buffer.area)
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
    pub fn into_widget(self) -> ViewWidget<'a> {
        ViewWidget {
            buf_offset_x: self.buf_offset_x,
            buf_offset_y: self.buf_offset_y,
            block: self.block,
            hscroll: self.hscroll,
            vscroll: self.vscroll,
            buffer: self.buffer,
        }
    }
}

impl<'a> StatefulWidget for ViewWidget<'a> {
    type State = ViewState;

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

        'y_loop: for y in 0..inner_area.height {
            'x_loop: for x in 0..inner_area.width {
                let xx = inner_area.x + x;
                let yy = inner_area.y + y;

                if xx >= inner_area.right() {
                    break 'x_loop;
                }
                if yy >= inner_area.bottom() {
                    break 'y_loop;
                }

                let buffer_x = x + self.buf_offset_x;
                let buffer_y = y + self.buf_offset_y;

                if let Some(cell) = self.buffer.cell(Position::new(buffer_x, buffer_y)) {
                    if let Some(tgt_cell) = buf.cell_mut(Position::new(xx, yy)) {
                        *tgt_cell = cell.clone();
                    }
                }
            }
        }

        // keep buffer
        state.buffer = Some(self.buffer);
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

impl HandleEvent<crossterm::event::Event, Regular, Outcome> for ViewState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> Outcome {
        self.handle(event, MouseOnly)
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for ViewState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> Outcome {
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
