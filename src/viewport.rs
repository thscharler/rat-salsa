/// A viewport allows scrolling of a `StatefulWidget` without
/// builtin support for scrolling.
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
use ratatui::prelude::StatefulWidget;
use ratatui::style::Style;
use ratatui::widgets::StatefulWidgetRef;
use std::mem;

/// View has its own size, and can contain a stateful widget
/// that will be rendered to a view sized buffer.
/// This buffer is then offset and written to the actual
/// frame buffer.
#[derive(Debug, Default, Clone)]
pub struct Viewport<T> {
    /// The widget.
    widget: T,
    /// Size of the view. The widget is drawn to a separate buffer
    /// with this size. x and y are set to the rendering area.
    view_size: Size,
    /// Style for any area outside the contained widget.
    style: Style,
}

/// State of the viewport.
#[derive(Debug, Clone)]
pub struct ViewportState<S> {
    /// Widget state.
    pub widget: S,

    /// The drawing area for the viewport.
    pub area: Rect,
    /// The viewport area that the inner widget sees.
    pub view_area: Rect,
    /// Horizontal offset
    pub h_offset: usize,
    /// Vertical offset
    pub v_offset: usize,

    /// Only construct with `..Default::default()`.
    pub non_exhaustive: NonExhaustive,
}

impl<T> Viewport<T> {
    /// New viewport.
    pub fn new(inner: T) -> Self {
        Self {
            style: Default::default(),
            widget: inner,
            view_size: Default::default(),
        }
    }

    /// Size for the inner widget.
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

impl<T> StatefulWidgetRef for Viewport<T>
where
    T: StatefulWidgetRef,
{
    type State = ViewportState<T::State>;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_impl(self, area, buf, state, |area, buf, state| {
            self.widget.render_ref(area, buf, state);
        });
    }
}

impl<T> StatefulWidget for Viewport<T>
where
    T: StatefulWidget + Default,
{
    type State = ViewportState<T::State>;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let inner = mem::take(&mut self.widget);

        render_impl(&self, area, buf, state, |area, buf, state| {
            inner.render(area, buf, state);
        });
    }
}

fn render_impl<T, S, FnRender>(
    widget: &Viewport<T>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut ViewportState<S>,
    render_inner: FnRender,
) where
    FnRender: FnOnce(Rect, &mut Buffer, &mut S),
{
    state.area = area;
    state.view_area = Rect::new(
        area.x,
        area.y,
        widget.view_size.width,
        widget.view_size.height,
    );

    let mut tmp = Buffer::empty(state.view_area);

    render_inner(state.view_area, &mut tmp, &mut state.widget);

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

impl<State, T> ScrollingWidget<State> for Viewport<T>
where
    T: StatefulWidget,
{
    fn need_scroll(&self, area: Rect, _state: &mut State) -> (bool, bool) {
        (
            area.width < self.view_size.width,
            area.height < self.view_size.height,
        )
    }
}

impl<S: Default> Default for ViewportState<S> {
    fn default() -> Self {
        Self {
            widget: S::default(),
            area: Default::default(),
            view_area: Default::default(),
            h_offset: 0,
            v_offset: 0,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<S> ViewportState<S> {
    /// Relocate mouse-events for use inside the viewport.
    pub fn relocate_crossterm(&self, event: &crossterm::event::Event) -> crossterm::event::Event {
        match event {
            crossterm::event::Event::FocusGained => event.clone(),
            crossterm::event::Event::FocusLost => event.clone(),
            crossterm::event::Event::Key(_) => event.clone(),
            crossterm::event::Event::Mouse(m) => {
                let mut m = *m;
                m.column += self.h_offset as u16;
                m.row += self.v_offset as u16;
                crossterm::event::Event::Mouse(m)
            }
            crossterm::event::Event::Paste(_) => event.clone(),
            crossterm::event::Event::Resize(_, _) => event.clone(),
        }
    }
}

impl<S> ScrollingState for ViewportState<S> {
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

/// Handle events if the widget has the focus.
impl<R, S> HandleEvent<crossterm::event::Event, FocusKeys, Outcome<R>> for ViewportState<S>
where
    S: HandleEvent<crossterm::event::Event, FocusKeys, R>,
    R: UsedEvent,
{
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> Outcome<R> {
        let event = self.relocate_crossterm(event);
        let r = self.widget.handle(&event, FocusKeys);
        Outcome::Inner(r)
    }
}

/// Handle only mouse-events.
impl<R, S> HandleEvent<crossterm::event::Event, MouseOnly, Outcome<R>> for ViewportState<S>
where
    S: HandleEvent<crossterm::event::Event, MouseOnly, R>,
    R: UsedEvent,
{
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome<R> {
        let event = self.relocate_crossterm(event);
        let r = self.widget.handle(&event, MouseOnly);
        Outcome::Inner(r)
    }
}
