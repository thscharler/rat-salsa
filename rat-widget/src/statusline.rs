//!
//! Statusbar with multiple sections.
//!

use crate::_private::NonExhaustive;
use rat_reloc::{relocate_area, relocate_areas, RelocatableState};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::style::Style;
use ratatui_core::text::Span;
// #[cfg(feature = "unstable-widget-ref")]
// use ratatui::widgets::StatefulWidgetRef;
use ratatui_core::widgets::{StatefulWidget, Widget};
use std::fmt::Debug;

/// Statusbar with multiple sections.
#[derive(Debug, Default, Clone)]
pub struct StatusLine {
    style: Vec<Style>,
    widths: Vec<Constraint>,
}

/// State & event handling.
#[derive(Debug, Clone)]
pub struct StatusLineState {
    /// Total area
    /// __readonly__. renewed for each render.
    pub area: Rect,
    /// Areas for each section.
    /// __readonly__. renewed for each render.
    pub areas: Vec<Rect>,

    /// Statustext for each section.
    /// __read+write__
    pub status: Vec<String>,

    pub non_exhaustive: NonExhaustive,
}

impl StatusLine {
    /// New widget.
    pub fn new() -> Self {
        Self {
            style: Default::default(),
            widths: Default::default(),
        }
    }

    /// Layout for the sections.
    ///
    /// This layout determines the number of sections.
    /// If the styles or the status text vec differ, defaults are used.
    pub fn layout<It, Item>(mut self, widths: It) -> Self
    where
        It: IntoIterator<Item = Item>,
        Item: Into<Constraint>,
    {
        self.widths = widths.into_iter().map(|v| v.into()).collect();
        self
    }

    /// Styles for each section.
    pub fn styles(mut self, style: impl IntoIterator<Item = impl Into<Style>>) -> Self {
        self.style = style.into_iter().map(|v| v.into()).collect();
        self
    }
}

impl Default for StatusLineState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            areas: Default::default(),
            status: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl RelocatableState for StatusLineState {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.area = relocate_area(self.area, shift, clip);
        relocate_areas(self.areas.as_mut(), shift, clip);
    }
}

impl StatusLineState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all status text.
    pub fn clear_status(&mut self) {
        self.status.clear();
    }

    /// Set the specific status section.
    pub fn status<S: Into<String>>(&mut self, idx: usize, msg: S) {
        while self.status.len() <= idx {
            self.status.push("".to_string());
        }
        self.status[idx] = msg.into();
    }
}

// #[cfg(feature = "unstable-widget-ref")]
// impl StatefulWidgetRef for StatusLine {
//     type State = StatusLineState;
//
//     fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
//         render_ref(self, area, buf, state);
//     }
// }

impl StatefulWidget for StatusLine {
    type State = StatusLineState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

fn render_ref(widget: &StatusLine, area: Rect, buf: &mut Buffer, state: &mut StatusLineState) {
    state.area = area;

    let layout = Layout::horizontal(widget.widths.iter()).split(state.area);

    for (i, rect) in layout.iter().enumerate() {
        let style = widget.style.get(i).copied().unwrap_or_default();
        let txt = state.status.get(i).map(|v| v.as_str()).unwrap_or("");

        buf.set_style(*rect, style);
        Span::from(txt).render(*rect, buf);
    }
}
