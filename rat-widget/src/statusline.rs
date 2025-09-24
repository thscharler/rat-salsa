//!
//! Statusbar with multiple sections.
//!
//! ```
//!
//! use ratatui::buffer::Buffer;
//! use ratatui::layout::{Constraint, Rect};
//! use ratatui::style::{Style, Stylize};
//! use ratatui::widgets::StatefulWidget;
//! use rat_widget::statusline::{StatusLine, StatusLineState};
//!
//! let mut status_line_state = StatusLineState::new();
//! status_line_state.status(0, "Everything's fine.");
//! status_line_state.status(1, "50%");
//! status_line_state.status(2, "72%");
//!
//!
//! # let area = Rect::new(0,24,80,1);
//! # let mut buf = Buffer::empty(area);
//! # let buf = &mut buf;
//!
//! StatusLine::new()
//!     .layout([
//!         Constraint::Fill(1),
//!         Constraint::Length(8),
//!         Constraint::Length(8)
//!     ])
//!     .styles([
//!         Style::new().white().on_dark_gray(),
//!         Style::new().white().on_cyan(),
//!         Style::new().white().on_blue()
//!     ])
//!     .render(area, buf, &mut status_line_state);
//!
//! ```

use crate::_private::NonExhaustive;
use rat_reloc::{RelocatableState, relocate_area, relocate_areas};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{StatefulWidget, Widget};
use std::borrow::Cow;
use std::fmt::Debug;

/// Statusbar with multiple sections.
#[derive(Debug, Default, Clone)]
pub struct StatusLine {
    sep: Option<Cow<'static, str>>,
    style: Vec<Style>,
    widths: Vec<Constraint>,
}

/// Combined style.
#[derive(Debug, Clone)]
pub struct StatusLineStyle {
    // separator
    pub sep: Option<Cow<'static, str>>,
    // styles
    pub styles: Vec<Style>,
    pub non_exhaustive: NonExhaustive,
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

impl Default for StatusLineStyle {
    fn default() -> Self {
        Self {
            sep: Default::default(),
            styles: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl StatusLine {
    /// New widget.
    pub fn new() -> Self {
        Self {
            sep: Default::default(),
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

    /// Set all styles.
    pub fn styles_ext(mut self, styles: StatusLineStyle) -> Self {
        self.sep = styles.sep;
        self.style = styles.styles;
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

impl StatefulWidget for &StatusLine {
    type State = StatusLineState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

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

        let sep = if i > 0 {
            if let Some(sep) = widget.sep.as_ref().map(|v| v.as_ref()) {
                Span::from(sep)
            } else {
                Span::default()
            }
        } else {
            Span::default()
        };

        Line::from_iter([
            sep, //
            Span::from(txt),
        ])
        .render(*rect, buf);

        buf.set_style(*rect, style);
    }
}
