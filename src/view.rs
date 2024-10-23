//! A view allows scrolling of on or more widgets without builtin
//! support for scrolling.
//!
//! View works in 3 phases:
//!
//! * Configuration
//!
//! ```rust ignore
//!     let mut view_buf = View::new()
//!         .view(Rect::new(0, 0, 400, 400))
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
use crate::_private::NonExhaustive;
use crate::event::ScrollOutcome;
use crate::relocate::{relocate_area, relocate_pos, RelocatableState};
use rat_event::{HandleEvent, MouseOnly, Outcome, Regular};
use rat_scrolled::{Scroll, ScrollArea, ScrollAreaState, ScrollState, ScrollStyle};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::prelude::{StatefulWidget, Widget};
use ratatui::widgets::Block;

/// View uses an internal buffer to render the widgets
/// and then renders part of that buffer.
#[derive(Debug, Default, Clone)]
pub struct View<'a> {
    view: Rect,

    block: Option<Block<'a>>,
    hscroll: Option<Scroll<'a>>,
    vscroll: Option<Scroll<'a>>,
}

/// Rendering widget for View.
#[derive(Debug, Clone)]
pub struct ViewBuffer<'a> {
    // View area of the buffer.
    view: Rect,
    // Scroll offset into the view.
    x_offset: usize,
    y_offset: usize,
    // inner area that will finally be rendered.
    widget_area: Rect,

    block: Option<Block<'a>>,
    hscroll: Option<Scroll<'a>>,
    vscroll: Option<Scroll<'a>>,

    buffer: Buffer,
}

/// Rendering widget for View.
#[derive(Debug, Clone)]
pub struct ViewWidget<'a> {
    block: Option<Block<'a>>,
    hscroll: Option<Scroll<'a>>,
    vscroll: Option<Scroll<'a>>,
    buffer: Buffer,
}

#[derive(Debug)]
pub struct ViewStyle {
    pub block: Option<Block<'static>>,
    pub scroll: Option<ScrollStyle>,

    pub non_exhaustive: NonExhaustive,
}

impl Default for ViewStyle {
    fn default() -> Self {
        Self {
            block: None,
            scroll: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

/// State of the viewport.
#[derive(Debug, Default, Clone)]
pub struct ViewState {
    /// Complete area of the viewport.
    /// __readonly__. renewed for each render.
    pub area: Rect,
    /// Inner area without the border.
    /// __readonly__. renewed for each render.
    pub widget_area: Rect,

    /// The viewport area that the inner widget uses.
    /// __readonly__. renewed for each render.
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
    /// New viewport.
    pub fn new() -> Self {
        Self::default()
    }

    /// Area for the temp buffer.
    pub fn view(mut self, area: Rect) -> Self {
        self.view = area;
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

    /// Morph the View to a ViewBuffer.
    /// The ViewBuffer is used to actually render the content of the view.
    pub fn into_buffer(self, area: Rect, state: &mut ViewState) -> ViewBuffer<'a> {
        state.area = area;
        state.view = self.view;

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

        let mut buffer = state.buffer.take().unwrap_or_default();
        buffer.reset();
        buffer.resize(state.view);

        ViewBuffer {
            view: state.view,
            x_offset: state.hscroll.offset,
            y_offset: state.vscroll.offset,
            widget_area: state.widget_area,
            block: self.block,
            hscroll: self.hscroll,
            vscroll: self.vscroll,
            buffer,
        }
    }
}

impl<'a> ViewBuffer<'a> {
    /// Render a widget to the temp buffer.
    pub fn render<W>(&mut self, widget: W, area: Rect)
    where
        W: Widget,
    {
        // render the actual widget.
        widget.render(area, self.buffer_mut());
    }

    /// Render a widget to the temp buffer.
    /// This expects that the state is a RelocatableState.
    pub fn render_stateful<W, S>(&mut self, widget: W, area: Rect, state: &mut S)
    where
        W: StatefulWidget<State = S>,
        S: RelocatableState,
    {
        // render the actual widget.
        widget.render(area, self.buffer_mut(), state);
        // shift and clip the output areas.
        self.relocate(state);
    }

    /// Is the given area visible?
    pub fn is_visible(&self, area: Rect) -> bool {
        let shifted = relocate_area(area, self.shift(), self.widget_area);
        shifted.is_empty()
    }

    /// Calculate the necessary shift from view to screen.
    pub fn shift(&self) -> (i16, i16) {
        let shift_view_x = self.view.x + self.x_offset as u16;
        let shift_view_y = self.view.y + self.y_offset as u16;

        let shift_x = self.widget_area.x as i16 - shift_view_x as i16;
        let shift_y = self.widget_area.y as i16 - shift_view_y as i16;

        (shift_x, shift_y)
    }

    /// Relocate an area from view coordinates to screen coordinates.
    /// Out of bounds areas result in an empty area.
    pub fn relocate_area(&self, area: Rect) -> Rect {
        relocate_area(area, self.shift(), self.widget_area)
    }

    /// Relocate an area from view coordinates to screen coordinates.
    /// Out of bounds areas result in an empty area.
    pub fn relocate_pos(&self, pos: Position) -> Option<Position> {
        relocate_pos(pos, self.shift(), self.widget_area)
    }

    /// Relocate all the widget state that refers to an area to
    /// the actual screen space used.
    pub fn relocate<S>(&self, state: &mut S)
    where
        S: RelocatableState,
    {
        state.relocate(self.shift(), self.widget_area);
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

        let buf_area = self.buffer.area;
        let inner_area = state.widget_area;

        'y_loop: for y in 0..inner_area.height {
            'x_loop: for x in 0..inner_area.width {
                let tgt_x = inner_area.x + x;
                let tgt_y = inner_area.y + y;

                if tgt_x >= inner_area.right() {
                    break 'x_loop;
                }
                if tgt_y >= inner_area.bottom() {
                    break 'y_loop;
                }

                let tmp_x = buf_area.x + x + state.hscroll.offset as u16;
                let tmp_y = buf_area.y + y + state.vscroll.offset as u16;

                if let Some(cell) = self.buffer.cell(Position::new(tmp_x, tmp_y)) {
                    if let Some(tgt_cell) = buf.cell_mut(Position::new(tgt_x, tgt_y)) {
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
            ScrollOutcome::Left(v) => self.scroll_left(v).into(),
            ScrollOutcome::Right(v) => self.scroll_right(v).into(),
            ScrollOutcome::VPos(v) => self.set_vertical_offset(v).into(),
            ScrollOutcome::HPos(v) => self.set_horizontal_offset(v).into(),
            r => r.into(),
        }
    }
}
