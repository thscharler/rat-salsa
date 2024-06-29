/// A viewport allows scrolling of a `StatefulWidget` without
/// builtin support for scrolling.
///
/// View and Viewport are the same in functionality.
///
/// The difference is that View works for [Widget]s and
/// Viewport for [StatefulWidget]s.
///
use crate::_private::NonExhaustive;
use crate::event::ScrollOutcome;
use crate::inner::{InnerStatefulOwned, InnerStatefulRef, InnerWidget};
use crate::util::copy_buffer;
use crate::{ScrollingState, ScrollingWidget};
use rat_event::{ConsumedEvent, HandleEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::{Rect, Size};
use ratatui::prelude::StatefulWidget;
use ratatui::style::Style;
use ratatui::widgets::StatefulWidgetRef;

/// View has its own size, and can contain a stateful widget
/// that will be rendered to a view sized buffer.
/// This buffer is then offset and written to the actual
/// frame buffer.
#[derive(Debug, Default, Clone)]
pub struct Viewport<T> {
    /// The widget.
    widget: T,
    viewport: ViewportImpl,
}

#[derive(Debug, Default, Clone)]
struct ViewportImpl {
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
            widget: inner,
            viewport: ViewportImpl::default(),
        }
    }

    /// Size for the inner widget.
    pub fn view_size(mut self, size: Size) -> Self {
        self.viewport.view_size = size;
        self
    }

    /// Style for the empty space outside the rendered buffer.
    pub fn style(mut self, style: Style) -> Self {
        self.viewport.style = style;
        self
    }
}

impl<T> StatefulWidgetRef for Viewport<T>
where
    T: StatefulWidgetRef,
{
    type State = ViewportState<T::State>;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let inner = InnerStatefulRef {
            inner: &self.widget,
        };
        render_ref(&self.viewport, inner, area, buf, state);
    }
}

impl<T> StatefulWidget for Viewport<T>
where
    T: StatefulWidget,
{
    type State = ViewportState<T::State>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let inner = InnerStatefulOwned { inner: self.widget };
        render_ref(&self.viewport, inner, area, buf, state);
    }
}

fn render_ref<W, S>(
    viewport: &ViewportImpl,
    inner: impl InnerWidget<W, S>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut ViewportState<S>,
) {
    state.area = area;
    state.view_area = Rect::new(
        area.x,
        area.y,
        viewport.view_size.width,
        viewport.view_size.height,
    );

    let mut tmp = Buffer::empty(state.view_area);

    inner.render_inner(state.view_area, &mut tmp, &mut state.widget);

    copy_buffer(
        state.view_area,
        tmp,
        state.v_offset,
        state.h_offset,
        viewport.style,
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
            area.width < self.viewport.view_size.width,
            area.height < self.viewport.view_size.height,
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

/// Handle events.
/// This forwards to the inner widget and corrects all
/// positions in the event.
impl<R, Q, S> HandleEvent<crossterm::event::Event, Q, ScrollOutcome<R>> for ViewportState<S>
where
    S: HandleEvent<crossterm::event::Event, Q, R>,
    R: ConsumedEvent,
{
    fn handle(&mut self, event: &crossterm::event::Event, keymap: Q) -> ScrollOutcome<R> {
        let event = self.relocate_crossterm(event);
        let r = self.widget.handle(&event, keymap);
        ScrollOutcome::Inner(r)
    }
}
