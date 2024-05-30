//!
//! A viewport that allows scrolling of a Widget.
//!
use crate::_private::NonExhaustive;
use crate::{
    ControlUI, DefaultKeys, HandleCrossterm, HasScrolling, MouseOnly, ScrollOutcome, ScrollParam,
    ScrolledWidget,
};
use crossterm::event::Event;
#[allow(unused_imports)]
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect, Size};
use ratatui::prelude::{StatefulWidget, Widget};
use ratatui::style::Style;
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

    pub fn fill_char(mut self, fill_char: char) -> Self {
        self.fill_char = fill_char;
        self
    }
}

// impl<T> StatefulWidgetRef for Viewport<T>
// where
//     T: WidgetRef,
// {
//     type State = ViewportState;
//
//     fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
//         render_impl(self, area, buf, state, |area, buf| {
//             self.widget.render_ref(area, buf);
//         });
//     }
// }

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

impl<State, T> ScrolledWidget<State> for Viewport<T>
where
    T: Widget,
{
    fn need_scroll(&self, area: Rect, _state: &mut State) -> ScrollParam {
        ScrollParam {
            has_hscroll: area.width < self.viewport_size.width,
            has_vscroll: area.height < self.viewport_size.height,
        }
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

impl HasScrolling for ViewportState {
    fn max_v_offset(&self) -> usize {
        self.viewport_area.height.saturating_sub(self.area.height) as usize
    }

    fn max_h_offset(&self) -> usize {
        self.viewport_area.width.saturating_sub(self.area.width) as usize
    }

    fn v_page_len(&self) -> usize {
        self.area.height as usize
    }

    fn h_page_len(&self) -> usize {
        self.area.width as usize
    }

    fn v_offset(&self) -> usize {
        self.v_offset
    }

    fn h_offset(&self) -> usize {
        self.h_offset
    }

    fn set_v_offset(&mut self, offset: usize) -> ScrollOutcome {
        if self.v_offset < self.viewport_area.height as usize {
            self.v_offset = offset;
            ScrollOutcome::Exact
        } else if self.v_offset == self.viewport_area.height.saturating_sub(1) as usize {
            ScrollOutcome::AtLimit
        } else {
            self.v_offset = self.viewport_area.height.saturating_sub(1) as usize;
            ScrollOutcome::Limited
        }
    }

    fn set_h_offset(&mut self, offset: usize) -> ScrollOutcome {
        if self.h_offset < self.viewport_area.width as usize {
            self.h_offset = offset;
            ScrollOutcome::Exact
        } else if self.h_offset == self.viewport_area.width.saturating_sub(1) as usize {
            ScrollOutcome::AtLimit
        } else {
            self.h_offset = self.viewport_area.width.saturating_sub(1) as usize;
            ScrollOutcome::Limited
        }
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, DefaultKeys> for ViewportState {
    fn handle(&mut self, _event: &Event, _keymap: DefaultKeys) -> ControlUI<A, E> {
        ControlUI::Continue
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for ViewportState {
    fn handle(&mut self, _event: &Event, _keymap: MouseOnly) -> ControlUI<A, E> {
        ControlUI::Continue
    }
}
