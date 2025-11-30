//!
//! Draw a shadow around a widget.
//!
use crate::_private::NonExhaustive;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Style;
use ratatui_core::widgets::StatefulWidget;

/// Direction of the shadow.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ShadowDirection {
    #[default]
    BottomRight,
    BottomLeft,
    TopRight,
    TopLeft,
}

/// Draw a shadow around a widget.
///
/// render is called with the area of the original widget,
/// and this renders just outside of it.
/// It sets the style of the cells to the given style
/// but leaves the text-content untouched.
///
#[derive(Debug, Default, Clone)]
pub struct Shadow {
    style: Style,
    dir: ShadowDirection,
}

impl Shadow {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn styles(mut self, styles: ShadowStyle) -> Self {
        self.style = styles.style;
        self.dir = styles.dir;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn direction(mut self, direction: ShadowDirection) -> Self {
        self.dir = direction;
        self
    }
}

#[derive(Debug, Clone)]
pub struct ShadowStyle {
    pub style: Style,
    pub dir: ShadowDirection,
    pub non_exhaustive: NonExhaustive,
}

impl Default for ShadowStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            dir: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl StatefulWidget for &Shadow {
    type State = ();

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

impl StatefulWidget for Shadow {
    type State = ();

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

fn render_ref(widget: &Shadow, area: Rect, buf: &mut Buffer, _state: &mut ()) {
    match widget.dir {
        ShadowDirection::BottomRight => {
            for y in area.top() + 1..area.bottom() + 1 {
                if let Some(cell) = buf.cell_mut((area.right(), y)) {
                    cell.set_style(widget.style);
                }
            }
            for x in area.left() + 1..area.right() {
                if let Some(cell) = buf.cell_mut((x, area.bottom())) {
                    cell.set_style(widget.style);
                }
            }
        }
        ShadowDirection::BottomLeft => {
            if area.left() > 0 {
                for y in area.top() + 1..area.bottom() + 1 {
                    if let Some(cell) = buf.cell_mut((area.left() - 1, y)) {
                        cell.set_style(widget.style);
                    }
                }
            }
            for x in area.left()..area.right().saturating_sub(1) {
                if let Some(cell) = buf.cell_mut((x, area.bottom())) {
                    cell.set_style(widget.style);
                }
            }
        }
        ShadowDirection::TopRight => {
            for y in area.top().saturating_sub(1)..area.bottom().saturating_sub(1) {
                if let Some(cell) = buf.cell_mut((area.right(), y)) {
                    cell.set_style(widget.style);
                }
            }
            if area.top() > 0 {
                for x in area.left() + 1..area.right() {
                    if let Some(cell) = buf.cell_mut((x, area.top() - 1)) {
                        cell.set_style(widget.style);
                    }
                }
            }
        }
        ShadowDirection::TopLeft => {
            if area.left() > 0 {
                for y in area.top().saturating_sub(1)..area.bottom().saturating_sub(1) {
                    if let Some(cell) = buf.cell_mut((area.left() - 1, y)) {
                        cell.set_style(widget.style);
                    }
                }
            }
            if area.top() > 0 {
                for x in area.left()..area.right().saturating_sub(1) {
                    if let Some(cell) = buf.cell_mut((x, area.top() - 1)) {
                        cell.set_style(widget.style);
                    }
                }
            }
        }
    }
}
