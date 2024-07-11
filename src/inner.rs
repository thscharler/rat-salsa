//!
//! Utilities to accommodate for StatefulWidgetRef vs StatefulWidget
//! and WidgetRef vs Widget
//!

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{StatefulWidget, Widget};
use ratatui::widgets::{StatefulWidgetRef, WidgetRef};
use std::cell::Cell;

/// Wrapper to convert a StatefulWidget to a StatefulWidgetRef,
/// while upholding the no-clone ability of StatefulWidget.
pub(crate) struct InnerStatefulOwn<T, S>
where
    T: StatefulWidget<State = S>,
{
    pub(crate) w: Cell<Option<T>>,
}

impl<T, S> StatefulWidgetRef for InnerStatefulOwn<T, S>
where
    T: StatefulWidget<State = S>,
{
    type State = S;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if let Some(w) = self.w.take() {
            w.render(area, buf, state);
        } else {
            unreachable!()
        }
    }
}

/// Wrapper to convert a Widget to a WidgetRef,
/// while upholding the no-clone ability of Widget.
pub(crate) struct InnerOwn<T>
where
    T: Widget,
{
    pub(crate) w: Cell<Option<T>>,
}

impl<T> WidgetRef for InnerOwn<T>
where
    T: Widget,
{
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        if let Some(w) = self.w.take() {
            w.render(area, buf);
        } else {
            unreachable!()
        }
    }
}
