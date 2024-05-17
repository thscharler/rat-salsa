/// A view allows scrolling of a widget without builtin
/// support for scrolling.
///
/// View and Viewport are the same in functionality.
///
/// The difference is that View works for [Widget]s and
/// Viewport for [StatefulWidget]s.
///
use crate::_private::NonExhaustive;
use crate::event::Outcome;
use crate::util::copy_buffer;
use crate::{ScrollingState, ScrollingWidget};
use rat_event::{FocusKeys, HandleEvent, MouseOnly, UsedEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::{Rect, Size};
use ratatui::prelude::{StatefulWidget, Widget};
use ratatui::style::Style;
use ratatui::widgets::{StatefulWidgetRef, WidgetRef};
use std::mem;

/// View has its own size, and can contain a stateless widget
/// that will be rendered to a view sized buffer.
/// This buffer is then offset and written to the actual
/// frame buffer.
#[derive(Debug, Default, Clone)]
pub struct View<T> {
    widget: T,
    /// Size of the view. The widget is drawn to a separate buffer
    /// with this size. x and y are set to the rendering area.
    view_size: Size,
    /// Style for any area outside the contained widget.
    style: Style,
}

#[derive(Debug, Clone)]
pub struct ViewState {
    /// The drawing area for the view.
    pub area: Rect,
    /// The view area that the inner widget sees.
    pub view_area: Rect,
    /// Horizontal offset
    pub h_offset: usize,
    /// Vertical offset
    pub v_offset: usize,

    /// Only construct with `..Default::default()`.
    pub non_exhaustive: NonExhaustive,
}

impl<T> View<T> {
    pub fn new(inner: T) -> Self {
        Self {
            widget: inner,
            view_size: Default::default(),
            style: Default::default(),
        }
    }

    pub fn view_size(mut self, size: Size) -> Self {
        self.view_size = size;
        self
    }

    /// Style for the empty space outside the rendered buffer.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<T> StatefulWidgetRef for View<T>
where
    T: WidgetRef,
{
    type State = ViewState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_impl(self, area, buf, state, |area, buf| {
            self.widget.render_ref(area, buf);
        })
    }
}

impl<T> StatefulWidget for View<T>
where
    T: Widget + Default,
{
    type State = ViewState;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let inner = mem::take(&mut self.widget);

        render_impl(&self, area, buf, state, |area, buf| {
            inner.render(area, buf);
        });
    }
}

fn render_impl<T, FnRender>(
    widget: &View<T>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut ViewState,
    render_inner: FnRender,
) where
    FnRender: FnOnce(Rect, &mut Buffer),
{
    state.area = area;
    state.view_area = Rect::new(
        area.x,
        area.y,
        widget.view_size.width,
        widget.view_size.height,
    );

    let mut tmp = Buffer::empty(state.view_area);

    render_inner(state.view_area, &mut tmp);

    copy_buffer(
        state.view_area,
        tmp,
        state.v_offset,
        state.h_offset,
        widget.style,
        area,
        buf,
    );
}

impl<State, T> ScrollingWidget<State> for View<T>
where
    T: Widget,
{
    fn need_scroll(&self, area: Rect, _state: &mut State) -> (bool, bool) {
        (
            area.width < self.view_size.width,
            area.height < self.view_size.height,
        )
    }
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            view_area: Default::default(),
            h_offset: 0,
            v_offset: 0,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl ScrollingState for ViewState {
    fn vertical_max_offset(&self) -> usize {
        self.view_area.height.saturating_sub(self.area.height) as usize
    }

    fn vertical_offset(&self) -> usize {
        self.v_offset
    }

    fn vertical_page(&self) -> usize {
        self.area.height as usize
    }

    fn horizontal_max_offset(&self) -> usize {
        self.view_area.width.saturating_sub(self.area.width) as usize
    }

    fn horizontal_offset(&self) -> usize {
        self.h_offset
    }

    fn horizontal_page(&self) -> usize {
        self.area.width as usize
    }

    fn set_vertical_offset(&mut self, offset: usize) -> bool {
        let old_offset = self.v_offset;

        if self.v_offset < self.view_area.height as usize {
            self.v_offset = offset;
        } else if self.v_offset >= self.view_area.height as usize {
            self.v_offset = self.view_area.height.saturating_sub(1) as usize;
        }

        old_offset != self.v_offset
    }

    fn set_horizontal_offset(&mut self, offset: usize) -> bool {
        let old_offset = self.h_offset;

        if self.h_offset < self.view_area.width as usize {
            self.h_offset = offset;
        } else if self.h_offset >= self.view_area.width as usize {
            self.h_offset = self.view_area.width.saturating_sub(1) as usize;
        }

        old_offset != self.h_offset
    }
}

/// Handle all events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
impl<R> HandleEvent<crossterm::event::Event, FocusKeys, Outcome<R>> for ViewState
where
    R: UsedEvent,
{
    fn handle(&mut self, _event: &crossterm::event::Event, _keymap: FocusKeys) -> Outcome<R> {
        // not supported for now. would need to translate the coordinates of the event too?
        Outcome::NotUsed
    }
}

/// Handle only mouse-events.
impl<R> HandleEvent<crossterm::event::Event, MouseOnly, Outcome<R>> for ViewState
where
    R: UsedEvent,
{
    fn handle(&mut self, _event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome<R> {
        // not supported for now. would need to translate the coordinates of the event too?
        Outcome::NotUsed
    }
}
