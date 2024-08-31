//! A viewport allows scrolling of a `StatefulWidget` without builtin
//! support for scrolling.
//!
//! View and Viewport are the same in functionality.
//!
//! The difference is that View works for [Widget]s and
//! Viewport for [StatefulWidget]s.
//!
//!
use crate::_private::NonExhaustive;
use crate::event::ScrollOutcome;
use crate::util::copy_buffer;
use rat_event::{flow, ConsumedEvent, HandleEvent, MouseOnly, Outcome};
use rat_scrolled::{Scroll, ScrollArea, ScrollAreaState, ScrollState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Rect, Size};
use ratatui::style::Style;
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::StatefulWidgetRef;
use ratatui::widgets::{Block, StatefulWidget};
use std::fmt::Debug;
use std::mem;

/// Viewport has its own size, and can contain a stateful widget
/// that will be rendered to a view sized buffer.
/// This buffer is then offset and written to the actual
/// frame buffer.
#[derive(Debug, Default, Clone)]
pub struct Viewport<'a, T> {
    widget: Option<T>,
    viewport: ViewportImpl<'a>,
}

#[derive(Debug, Default, Clone)]
struct ViewportImpl<'a> {
    block: Option<Block<'a>>,
    hscroll: Option<Scroll<'a>>,
    vscroll: Option<Scroll<'a>>,
    view_size: Size,
    style: Style,
}

/// State of the viewport.
#[derive(Debug, Clone)]
pub struct ViewportState<S> {
    pub widget: S,

    /// Complete area of the viewport.
    pub area: Rect,
    /// Inner area of the viewport.
    pub inner_area: Rect,
    /// The viewport area that the inner widget sees.
    pub view_area: Rect,
    /// Horizontal scroll
    pub hscroll: ScrollState,
    /// Vertical scroll
    pub vscroll: ScrollState,

    /// Only construct with `..Default::default()`.
    pub non_exhaustive: NonExhaustive,
}

impl<'a, T> Viewport<'a, T> {
    /// New viewport.
    pub fn new(inner: T) -> Self {
        Self {
            widget: Some(inner),
            viewport: Default::default(),
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.viewport.block = Some(block);
        self
    }

    pub fn scroll(mut self, scroll: Scroll<'a>) -> Self {
        self.viewport.hscroll = Some(scroll.clone().override_horizontal());
        self.viewport.vscroll = Some(scroll.override_vertical());
        self
    }

    pub fn hscroll(mut self, scroll: Scroll<'a>) -> Self {
        self.viewport.hscroll = Some(scroll.override_horizontal());
        self
    }

    pub fn vscroll(mut self, scroll: Scroll<'a>) -> Self {
        self.viewport.vscroll = Some(scroll.override_vertical());
        self
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

#[cfg(feature = "unstable-widget-ref")]
impl<'a, T> StatefulWidgetRef for Viewport<'a, T>
where
    T: StatefulWidgetRef,
{
    type State = ViewportState<T::State>;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if let Some(w) = &self.widget {
            let scroll = ScrollArea::new()
                .block(self.viewport.block.clone())
                .h_scroll(self.viewport.hscroll.clone())
                .v_scroll(self.viewport.vscroll.clone());

            render_ref(
                &self.viewport,
                |area, buf, state| w.render_ref(area, buf, state),
                scroll,
                area,
                buf,
                state,
            );
        } else {
            unreachable!()
        }
    }
}

impl<'a, T> StatefulWidget for Viewport<'a, T>
where
    T: StatefulWidget,
{
    type State = ViewportState<T::State>;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if let Some(w) = self.widget.take() {
            let block = mem::take(&mut self.viewport.block);
            let hscroll = mem::take(&mut self.viewport.hscroll);
            let vscroll = mem::take(&mut self.viewport.vscroll);

            let scroll = ScrollArea::new()
                .block(block)
                .h_scroll(hscroll)
                .v_scroll(vscroll);

            render_ref(
                &self.viewport,
                |area, buf, state| w.render(area, buf, state),
                scroll,
                area,
                buf,
                state,
            );
        } else {
            unreachable!()
        }
    }
}

fn render_ref<S>(
    viewport: &ViewportImpl<'_>,
    render_widget: impl FnOnce(Rect, &mut Buffer, &mut S),
    scroll: ScrollArea<'_>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut ViewportState<S>,
) {
    state.area = area;

    state.inner_area = scroll.inner(
        area,
        ScrollAreaState {
            area,
            h_scroll: Some(&mut state.hscroll),
            v_scroll: Some(&mut state.vscroll),
        },
    );

    state.view_area = Rect::new(
        state.inner_area.x,
        state.inner_area.y,
        viewport.view_size.width,
        viewport.view_size.height,
    );

    state
        .hscroll
        .set_max_offset(state.view_area.width.saturating_sub(state.inner_area.width) as usize);
    state.hscroll.set_page_len(state.inner_area.width as usize);
    state.vscroll.set_max_offset(
        state
            .view_area
            .height
            .saturating_sub(state.inner_area.height) as usize,
    );
    state.vscroll.set_page_len(state.inner_area.height as usize);

    scroll.render(
        area,
        buf,
        &mut ScrollAreaState {
            area,
            h_scroll: Some(&mut state.hscroll),
            v_scroll: Some(&mut state.vscroll),
        },
    );

    let mut tmp = Buffer::empty(state.view_area);
    render_widget(state.view_area, &mut tmp, &mut state.widget);

    copy_buffer(
        state.view_area,
        tmp,
        state.hscroll.offset(),
        state.vscroll.offset(),
        viewport.style,
        state.inner_area,
        buf,
    );
}

impl<S: Default> Default for ViewportState<S> {
    fn default() -> Self {
        Self {
            widget: S::default(),
            area: Default::default(),
            inner_area: Default::default(),
            view_area: Default::default(),
            hscroll: Default::default(),
            vscroll: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<S> ViewportState<S>
where
    S: Default,
{
    pub fn new(widget_state: S) -> Self {
        Self {
            widget: widget_state,
            ..Self::default()
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
                m.column += self.hscroll.offset() as u16;
                m.row += self.vscroll.offset() as u16;
                crossterm::event::Event::Mouse(m)
            }
            crossterm::event::Event::Paste(_) => event.clone(),
            crossterm::event::Event::Resize(_, _) => event.clone(),
        }
    }
}

impl<S> ViewportState<S> {
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

impl<R, Q, S> HandleEvent<crossterm::event::Event, Q, R> for ViewportState<S>
where
    S: HandleEvent<crossterm::event::Event, Q, R>,
    R: From<Outcome> + ConsumedEvent + Debug,
{
    fn handle(&mut self, event: &crossterm::event::Event, qualifier: Q) -> R {
        flow!(self
            .widget
            .handle(&self.relocate_crossterm(event), qualifier));

        let mut sas = ScrollAreaState {
            area: self.inner_area,
            h_scroll: Some(&mut self.hscroll),
            v_scroll: Some(&mut self.vscroll),
        };

        let r = match sas.handle(event, MouseOnly) {
            ScrollOutcome::Up(v) => self.scroll_up(v),
            ScrollOutcome::Down(v) => self.scroll_down(v),
            ScrollOutcome::Left(v) => self.scroll_left(v),
            ScrollOutcome::Right(v) => self.scroll_right(v),
            ScrollOutcome::VPos(v) => self.set_vertical_offset(v),
            ScrollOutcome::HPos(v) => self.set_horizontal_offset(v),
            ScrollOutcome::Continue => false,
            ScrollOutcome::Unchanged => false,
            ScrollOutcome::Changed => true,
        };
        if r {
            return Outcome::Changed.into();
        }

        Outcome::Continue.into()
    }
}
