/// A viewport allows scrolling of a `StatefulWidget` without builtin
/// support for scrolling.
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
use crate::{layout_scroll, Scroll, ScrollArea, ScrollState};
use log::debug;
use rat_event::{flow, ConsumedEvent, HandleEvent, MouseOnly, Outcome};
use ratatui::buffer::Buffer;
use ratatui::layout::{Rect, Size};
use ratatui::prelude::StatefulWidget;
use ratatui::style::Style;
use ratatui::widgets::{Block, StatefulWidgetRef, WidgetRef};
use std::fmt::Debug;

/// Viewport has its own size, and can contain a stateful widget
/// that will be rendered to a view sized buffer.
/// This buffer is then offset and written to the actual
/// frame buffer.
#[derive(Debug, Default, Clone)]
pub struct Viewport<'a, T> {
    widget: T,
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
            widget: inner,
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
        self.viewport.hscroll = Some(scroll.override_vertical());
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

impl<'a, T> StatefulWidgetRef for Viewport<'a, T>
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

impl<'a, T> StatefulWidget for Viewport<'a, T>
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
    viewport: &ViewportImpl<'_>,
    widget: impl InnerWidget<W, S>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut ViewportState<S>,
) {
    state.area = area;

    let (hscroll_area, vscroll_area, inner_area) = layout_scroll(
        area,
        viewport.block.as_ref(),
        viewport.hscroll.as_ref(),
        viewport.vscroll.as_ref(),
    );
    state.inner_area = inner_area;

    state.view_area = Rect::new(
        inner_area.x,
        inner_area.y,
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

    viewport.block.render_ref(area, buf);
    if let Some(hscroll) = &viewport.hscroll {
        hscroll.render_ref(hscroll_area, buf, &mut state.hscroll);
    }
    if let Some(vscroll) = &viewport.vscroll {
        vscroll.render_ref(vscroll_area, buf, &mut state.vscroll);
    }

    let mut tmp = Buffer::empty(state.view_area);
    widget.render_inner(state.view_area, &mut tmp, &mut state.widget);

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
        flow!(match self.hscroll.handle(event, MouseOnly) {
            ScrollOutcome::Offset(v) => {
                Outcome::from(self.horizontal_scroll_to(v))
            }
            r => Outcome::from(r),
        });
        flow!(match self.vscroll.handle(event, MouseOnly) {
            ScrollOutcome::Offset(v) => {
                Outcome::from(self.vertical_scroll_to(v))
            }
            r => Outcome::from(r),
        });

        flow!(self
            .widget
            .handle(&self.relocate_crossterm(event), qualifier));

        flow!(
            match ScrollArea(self.inner_area, Some(&self.hscroll), Some(&self.vscroll))
                .handle(event, MouseOnly)
            {
                ScrollOutcome::Up(v) => {
                    Outcome::from(self.scroll_up(v))
                }
                ScrollOutcome::Down(v) => {
                    Outcome::from(self.scroll_down(v))
                }
                ScrollOutcome::Left(v) => {
                    Outcome::from(self.scroll_left(v))
                }
                ScrollOutcome::Right(v) => {
                    Outcome::from(self.scroll_right(v))
                }
                r => r.into(),
            }
        );

        Outcome::NotUsed.into()
    }
}
