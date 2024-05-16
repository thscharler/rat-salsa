//!
//! A viewport that allows scrolling of a Widget.
//!
use crate::_private::NonExhaustive;
use crate::event::Outcome;
use crate::{ScrollingState, ScrollingWidget};
use rat_event::{FocusKeys, HandleEvent, MouseOnly};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect, Size};
use ratatui::prelude::{StatefulWidget, Widget};
use ratatui::style::Style;
use ratatui::widgets::{StatefulWidgetRef, WidgetRef};
use std::mem;

/// The viewport has its own size that is used to create
/// the buffer where the contained widget is rendered.
///
/// This buffer is the base for scrolling behaviour.
#[derive(Debug, Default, Clone)]
pub struct Viewport<T> {
    /// Viewport size.
    /// There is only need for a size, the widget gets a buffer with
    /// x and y set to the rendering area.
    viewport_size: Size,
    /// Style for the empty space, when scrolling goes beyond the
    /// buffer size.
    style: Style,
    /// Character to be drawn when scrolling goes beyond the buffer
    /// size
    fill_char: char,

    /// The widget.
    widget: T,
}

/// State of the viewport.
#[derive(Debug, Clone)]
pub struct ViewportState {
    /// The drawing area of the viewport.
    pub area: Rect,
    /// The viewport area that the contained widget sees.
    pub viewport_area: Rect,
    /// Scroll offset
    pub h_offset: usize,
    /// Scroll offset.
    pub v_offset: usize,

    /// Only construct with `..Default::default()`.
    pub non_exhaustive: NonExhaustive,
}

impl<T> Viewport<T> {
    /// New viewport.
    pub fn new(inner: T) -> Self {
        Self {
            viewport_size: Default::default(),
            style: Default::default(),
            fill_char: ' ',
            widget: inner,
        }
    }

    /// Size for the inner widget.
    pub fn viewport_size(mut self, size: Size) -> Self {
        self.viewport_size = size;
        self
    }

    /// Style for the empty space outside the rendered buffer.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Fill character for empty space outside the rendererd buffer.
    pub fn fill_char(mut self, fill_char: char) -> Self {
        self.fill_char = fill_char;
        self
    }
}

impl<T> StatefulWidgetRef for Viewport<T>
where
    T: WidgetRef,
{
    type State = ViewportState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_impl(self, area, buf, state, |area, buf| {
            self.widget.render_ref(area, buf);
        });
    }
}

impl<T> StatefulWidget for Viewport<T>
where
    T: Widget + Default,
{
    type State = ViewportState;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let inner = mem::take(&mut self.widget);

        render_impl(&self, area, buf, state, |area, buf| {
            inner.render(area, buf);
        });
    }
}

fn render_impl<T, FnRender>(
    widget: &Viewport<T>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut ViewportState,
    render_inner: FnRender,
) where
    FnRender: FnOnce(Rect, &mut Buffer),
{
    state.area = area;
    state.viewport_area = Rect::new(
        area.x,
        area.y,
        widget.viewport_size.width,
        widget.viewport_size.height,
    );

    let mut tmp = Buffer::empty(state.viewport_area);

    render_inner(state.viewport_area, &mut tmp);

    // copy buffer
    for (cell_offset, cell) in tmp.content.drain(..).enumerate() {
        let tmp_row = cell_offset as u16 / tmp.area.width;
        let tmp_col = cell_offset as u16 % tmp.area.width;

        if area.y + tmp_row >= state.v_offset as u16 && area.x + tmp_row >= state.h_offset as u16 {
            let row = area.y + tmp_row - state.v_offset as u16;
            let col = area.x + tmp_col - state.h_offset as u16;

            if area.contains(Position::new(col, row)) {
                *buf.get_mut(col, row) = cell;
            } else {
                // clip
            }
        } else {
            // clip2
        }
    }

    // clear the rest
    let filled_left =
        (state.area.x + state.viewport_area.width).saturating_sub(state.h_offset as u16);
    let filled_bottom =
        (state.area.y + state.viewport_area.height).saturating_sub(state.v_offset as u16);

    for r in area.y..area.y + area.height {
        for c in area.x..area.x + area.width {
            if c >= filled_left || r >= filled_bottom {
                buf.get_mut(c, r)
                    .set_char(widget.fill_char)
                    .set_style(widget.style);
            }
        }
    }
}

impl<State, T> ScrollingWidget<State> for Viewport<T>
where
    T: Widget,
{
    fn need_scroll(&self, area: Rect, _state: &mut State) -> (bool, bool) {
        (
            area.width < self.viewport_size.width,
            area.height < self.viewport_size.height,
        )
    }
}

impl Default for ViewportState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            viewport_area: Default::default(),
            h_offset: 0,
            v_offset: 0,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl ViewportState {
    /// Relocate mouse-events for use inside the viewport.
    pub fn relocate_crossterm(&self, event: &crossterm::event::Event) -> crossterm::event::Event {
        match event {
            crossterm::event::Event::FocusGained => event.clone(),
            crossterm::event::Event::FocusLost => event.clone(),
            crossterm::event::Event::Key(_) => event.clone(),
            crossterm::event::Event::Mouse(m) => {
                let mut m = m.clone();
                m.column = m.column + self.h_offset as u16;
                m.row = m.row + self.v_offset as u16;
                crossterm::event::Event::Mouse(m)
            }
            crossterm::event::Event::Paste(_) => event.clone(),
            crossterm::event::Event::Resize(_, _) => event.clone(),
        }
    }
}

impl ScrollingState for ViewportState {
    fn vertical_max_offset(&self) -> usize {
        self.viewport_area.height.saturating_sub(self.area.height) as usize
    }

    fn vertical_offset(&self) -> usize {
        self.v_offset
    }

    fn vertical_page(&self) -> usize {
        self.area.height as usize
    }

    fn vertical_scroll(&self) -> usize {
        self.area.height as usize / 10
    }

    fn horizontal_max_offset(&self) -> usize {
        self.viewport_area.width.saturating_sub(self.area.width) as usize
    }

    fn horizontal_offset(&self) -> usize {
        self.h_offset
    }

    fn horizontal_page(&self) -> usize {
        self.area.width as usize
    }

    fn horizontal_scroll(&self) -> usize {
        self.area.width as usize / 10
    }

    fn set_vertical_offset(&mut self, offset: usize) -> bool {
        let old_offset = self.v_offset;

        if self.v_offset < self.viewport_area.height as usize {
            self.v_offset = offset;
        } else if self.v_offset >= self.viewport_area.height as usize {
            self.v_offset = self.viewport_area.height.saturating_sub(1) as usize;
        }

        old_offset != self.v_offset
    }

    fn set_horizontal_offset(&mut self, offset: usize) -> bool {
        let old_offset = self.h_offset;

        if self.h_offset < self.viewport_area.width as usize {
            self.h_offset = offset;
        } else if self.h_offset >= self.viewport_area.width as usize {
            self.h_offset = self.viewport_area.width.saturating_sub(1) as usize;
        }

        old_offset != self.h_offset
    }
}

/// Handle all events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
impl<R> HandleEvent<crossterm::event::Event, FocusKeys, Outcome<R>> for ViewportState {
    fn handle(&mut self, _event: &crossterm::event::Event, _keymap: FocusKeys) -> Outcome<R> {
        // not supported for now. would need to translate the coordinates of the event too?
        Outcome::NotUsed
    }
}

/// Handle only mouse-events.
impl<R> HandleEvent<crossterm::event::Event, MouseOnly, Outcome<R>> for ViewportState {
    fn handle(&mut self, _event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome<R> {
        // not supported for now. would need to translate the coordinates of the event too?
        Outcome::NotUsed
    }
}
