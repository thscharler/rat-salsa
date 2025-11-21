//!
//! Constrains an area from its outside.
//!
use crate::layout::{DialogItem, GenericLayout, layout_dialog};
use ratatui::layout::{Constraint, Flex, Layout, Position, Rect, Size};
use ratatui::widgets::Padding;

/// This lets you define the outer bounds of a target area.
///
/// The constraints are applied in this order.
///
/// * Constrain the left/right/top/bottom border outside the area.
/// * Set a fixed position.
/// * Constrain the width/height of the area.
/// * Set a fixed size.
///
/// In the end it gives you the middle Rect.
///
#[derive(Debug, Default, Clone)]
pub struct LayoutOuter {
    pub left: Option<Constraint>,
    pub top: Option<Constraint>,
    pub right: Option<Constraint>,
    pub bottom: Option<Constraint>,
    pub position: Option<Position>,
    pub width: Option<Constraint>,
    pub height: Option<Constraint>,
    pub size: Option<Size>,
}

impl LayoutOuter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Margin constraint for the left side.
    pub fn left(mut self, left: Constraint) -> Self {
        self.left = Some(left);
        self
    }

    /// Margin constraint for the top side.
    pub fn top(mut self, top: Constraint) -> Self {
        self.top = Some(top);
        self
    }

    /// Margin constraint for the right side.
    pub fn right(mut self, right: Constraint) -> Self {
        self.right = Some(right);
        self
    }

    /// Margin constraint for the bottom side.
    pub fn bottom(mut self, bottom: Constraint) -> Self {
        self.bottom = Some(bottom);
        self
    }

    /// Put at a fixed position.
    pub fn position(mut self, pos: Position) -> Self {
        self.position = Some(pos);
        self
    }

    /// Constraint for the width.
    pub fn width(mut self, width: Constraint) -> Self {
        self.width = Some(width);
        self
    }

    /// Constraint for the height.
    pub fn height(mut self, height: Constraint) -> Self {
        self.height = Some(height);
        self
    }

    /// Set at a fixed size.
    pub fn size(mut self, size: Size) -> Self {
        self.size = Some(size);
        self
    }

    /// Calculate the area.
    #[inline]
    pub fn layout(&self, area: Rect) -> Rect {
        let mut hor = [
            Constraint::Length(0),
            Constraint::Fill(1),
            Constraint::Length(0),
        ];
        let mut ver = [
            Constraint::Length(0),
            Constraint::Fill(1),
            Constraint::Length(0),
        ];
        if let Some(left) = self.left {
            hor[0] = left;
        }
        if let Some(top) = self.top {
            ver[0] = top;
        }
        if let Some(right) = self.right {
            hor[2] = right;
        }
        if let Some(bottom) = self.bottom {
            ver[2] = bottom;
        }
        if let Some(pos) = self.position {
            ver[0] = Constraint::Length(pos.y);
            hor[0] = Constraint::Length(pos.x);
        }
        if let Some(width) = self.width {
            hor[1] = width;
            hor[2] = Constraint::Fill(1);
        }
        if let Some(height) = self.height {
            ver[1] = height;
            ver[2] = Constraint::Fill(1);
        }
        if let Some(size) = self.size {
            ver[1] = Constraint::Length(size.height);
            ver[2] = Constraint::Fill(1);
            hor[1] = Constraint::Length(size.width);
            hor[2] = Constraint::Fill(1);
        }

        let h_layout = Layout::horizontal(hor).split(area);
        let v_layout = Layout::vertical(ver).split(h_layout[1]);
        v_layout[1]
    }

    /// Create a dialog layout with the given constraints.
    #[inline]
    pub fn layout_dialog<const N: usize>(
        &self,
        area: Rect,
        padding: Padding,
        buttons: [Constraint; N],
        button_spacing: u16,
        button_flex: Flex,
    ) -> GenericLayout<DialogItem> {
        let inner = self.layout(area);
        layout_dialog(inner, padding, buttons, button_spacing, button_flex)
    }
}
