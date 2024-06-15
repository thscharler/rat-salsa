use crate::ScrollingWidget;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::StatefulWidget;
use ratatui::widgets::{StatefulWidgetRef, Widget, WidgetRef};

/// Trait for unifying StatefulWidget and StatefulWidgetRef.
pub(crate) trait InnerWidget<W, S> {
    fn render_inner(self, area: Rect, buf: &mut Buffer, state: &mut S);
}

// -------------------------------------------------------------
// -------------------------------------------------------------

pub(crate) struct InnerStatefulOwned<W> {
    pub(crate) inner: W,
}

impl<W, S> ScrollingWidget<S> for InnerStatefulOwned<W>
where
    W: ScrollingWidget<S>,
{
    fn need_scroll(&self, area: Rect, state: &mut S) -> (bool, bool) {
        self.inner.need_scroll(area, state)
    }
}

impl<W, S> InnerWidget<W, S> for InnerStatefulOwned<W>
where
    W: StatefulWidget<State = S>,
{
    fn render_inner(self, area: Rect, buf: &mut Buffer, state: &mut S) {
        self.inner.render(area, buf, state);
    }
}

// -------------------------------------------------------------
// -------------------------------------------------------------

pub(crate) struct InnerStatefulRef<'a, W> {
    pub(crate) inner: &'a W,
}

impl<'a, W, S> ScrollingWidget<S> for InnerStatefulRef<'a, W>
where
    W: ScrollingWidget<S>,
{
    fn need_scroll(&self, area: Rect, state: &mut S) -> (bool, bool) {
        self.inner.need_scroll(area, state)
    }
}

impl<'a, W, S> InnerWidget<W, S> for InnerStatefulRef<'a, W>
where
    W: StatefulWidgetRef<State = S>,
{
    fn render_inner(self, area: Rect, buf: &mut Buffer, state: &mut S) {
        self.inner.render_ref(area, buf, state);
    }
}

// -------------------------------------------------------------
// -------------------------------------------------------------

pub(crate) struct InnerOwned<W> {
    pub(crate) inner: W,
}

impl<W> InnerWidget<W, ()> for InnerOwned<W>
where
    W: Widget,
{
    fn render_inner(self, area: Rect, buf: &mut Buffer, _: &mut ()) {
        self.inner.render(area, buf);
    }
}

// -------------------------------------------------------------
// --------------------------------------------------------------

pub(crate) struct InnerRef<'a, W> {
    pub(crate) inner: &'a W,
}

impl<'a, W> InnerWidget<W, ()> for InnerRef<'a, W>
where
    W: WidgetRef,
{
    fn render_inner(self, area: Rect, buf: &mut Buffer, _: &mut ()) {
        self.inner.render_ref(area, buf);
    }
}
