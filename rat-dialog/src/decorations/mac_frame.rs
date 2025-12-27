//!
//! Widget for a moveable window.
//!
use crate::decorations::frame_state::{WindowFrameState, WindowFrameStyle};
use rat_focus::HasFocus;
use rat_widget::util::revert_style;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Style;
use ratatui_core::text::Span;
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_widgets::block::{Block, BlockExt};

/// Widget for a moveable window.
///
/// This widget ignores the area given to render,
/// and uses the area stored in the state instead.
/// The area given to render is used as the outer limit for
/// the window instead.
///
/// Render this widget and then use WindowState::widget_area to
/// render your content.
///
/// It can handle events for move/resize/close.
#[derive(Debug, Default)]
pub struct MacFrame<'a> {
    no_fill: bool,
    block: Option<Block<'a>>,

    style: Style,
    top_style: Option<Style>,
    focus_style: Option<Style>,
    hover_style: Style,
    drag_style: Style,
    close_style: Option<Style>,
    min_style: Option<Style>,
    max_style: Option<Style>,

    limit: Option<Rect>,

    can_move: Option<bool>,
    can_resize: Option<bool>,
    can_close: Option<bool>,
    can_min: Option<bool>,
    can_max: Option<bool>,
}

impl<'a> MacFrame<'a> {
    pub fn new() -> Self {
        Self {
            no_fill: Default::default(),
            block: Default::default(),
            style: Default::default(),
            top_style: Default::default(),
            focus_style: Default::default(),
            hover_style: Default::default(),
            drag_style: Default::default(),
            close_style: Default::default(),
            min_style: Default::default(),
            max_style: Default::default(),
            limit: Default::default(),
            can_move: Default::default(),
            can_resize: Default::default(),
            can_close: Default::default(),
            can_min: Default::default(),
            can_max: Default::default(),
        }
    }

    /// Don't fill the area.
    pub fn no_fill(mut self) -> Self {
        self.no_fill = true;
        self
    }

    /// Limits for the window.
    ///
    /// If this is not set, the area given to render will be used.
    pub fn limit(mut self, area: Rect) -> Self {
        self.limit = Some(area);
        self
    }

    /// Window can be moved?
    pub fn can_move(mut self, v: bool) -> Self {
        self.can_move = Some(v);
        self
    }

    /// Window can be resized?
    pub fn can_resize(mut self, v: bool) -> Self {
        self.can_resize = Some(v);
        self
    }

    /// Window can be closed?
    pub fn can_close(mut self, v: bool) -> Self {
        self.can_close = Some(v);
        self
    }

    /// Window can be minimized?
    pub fn can_min(mut self, v: bool) -> Self {
        self.can_min = Some(v);
        self
    }

    /// Window can be maximized?
    pub fn can_max(mut self, v: bool) -> Self {
        self.can_max = Some(v);
        self
    }

    /// Window block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block.style(self.style));
        self
    }

    pub fn styles(mut self, styles: WindowFrameStyle) -> Self {
        self.style = styles.style;
        self.block = styles.block;
        if styles.top.is_some() {
            self.top_style = styles.top;
        }
        if styles.focus.is_some() {
            self.focus_style = styles.focus;
        }
        if let Some(hover) = styles.hover {
            self.hover_style = hover;
        }
        if let Some(drag) = styles.drag {
            self.drag_style = drag;
        }
        if let Some(drag) = styles.drag {
            self.drag_style = drag;
        }
        if let Some(close) = styles.close {
            self.close_style = Some(close);
        }
        if let Some(min) = styles.min {
            self.min_style = Some(min);
        }
        if let Some(max) = styles.max {
            self.max_style = Some(max);
        }
        if let Some(can_move) = styles.can_move {
            self.can_move = Some(can_move);
        }
        if let Some(can_resize) = styles.can_resize {
            self.can_resize = Some(can_resize);
        }
        if let Some(can_close) = styles.can_close {
            self.can_move = Some(can_close);
        }
        if let Some(can_min) = styles.can_min {
            self.can_min = Some(can_min);
        }
        if let Some(can_max) = styles.can_max {
            self.can_max = Some(can_max);
        }
        self
    }

    /// Window base style
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self.block = self.block.map(|v| v.style(style));
        self
    }

    /// Window title style
    pub fn title_style(mut self, style: Style) -> Self {
        self.top_style = Some(style);
        self
    }

    /// Window focus style
    pub fn focus_style(mut self, style: Style) -> Self {
        self.top_style = Some(style);
        self
    }

    /// Hover style
    pub fn hover_style(mut self, hover: Style) -> Self {
        self.hover_style = hover;
        self
    }

    /// Drag style
    pub fn drag_style(mut self, drag: Style) -> Self {
        self.drag_style = drag;
        self
    }
}

impl<'a> StatefulWidget for MacFrame<'a> {
    type State = WindowFrameState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if let Some(limit) = self.limit {
            state.limit = limit;
        } else {
            state.limit = area;
        }
        state.area = state.area.intersection(state.limit);
        state.widget_area = self.block.inner_if_some(state.area);

        if let Some(v) = self.can_move {
            state.can_move = v;
        }
        if let Some(v) = self.can_resize {
            state.can_resize = v;
        }
        if let Some(v) = self.can_close {
            state.can_close = v;
        }

        if state.can_resize {
            state.resize_area = Rect::new(
                state.area.right().saturating_sub(2),
                state.area.bottom().saturating_sub(1),
                2,
                1,
            );
        } else {
            state.resize_area = Default::default();
        }
        if state.can_close && !state.area.is_empty() {
            state.close_area = Rect::new(state.area.x + 2, state.area.y, 3, 1);
        } else {
            state.close_area = Default::default();
        }
        if state.can_min && !state.area.is_empty() {
            state.min_area = Rect::new(state.area.x + 5, state.area.y, 3, 1);
        } else {
            state.min_area = Default::default();
        }
        if state.can_max && !state.area.is_empty() {
            state.max_area = Rect::new(state.area.x + 8, state.area.y, 3, 1);
        } else {
            state.max_area = Default::default();
        }

        if state.can_move {
            if state.can_close || state.can_min || state.can_max {
                state.move_area = Rect::new(
                    state.area.x + 11, //
                    state.area.y,
                    state.area.width.saturating_sub(13),
                    1,
                );
            } else {
                state.move_area = Rect::new(
                    state.area.x + 1,
                    state.area.y,
                    state.area.width.saturating_sub(2),
                    1,
                );
            }
        } else {
            state.move_area = Default::default();
        }

        if !self.no_fill {
            for y in state.area.top()..state.area.bottom() {
                for x in state.area.left()..state.area.right() {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.reset();
                    }
                }
            }
        }

        let block = if state.top {
            if state.is_focused() {
                if let Some(top_style) = self.focus_style.or(self.top_style) {
                    self.block.map(|v| v.title_style(top_style))
                } else {
                    self.block
                }
            } else {
                if let Some(top_style) = self.top_style {
                    self.block.map(|v| v.title_style(top_style))
                } else {
                    self.block
                }
            }
        } else {
            self.block
        };
        let block = if self.no_fill {
            block.map(|v| v.style(Style::new()))
        } else {
            block
        };

        block.render(state.area, buf);

        if state.can_close && !state.area.is_empty() {
            Span::from(" ⬤ ")
                .style(self.close_style.unwrap_or(self.style.red()))
                .render(state.close_area, buf);
        }
        if state.can_min && !state.area.is_empty() {
            Span::from(" ⬤ ")
                .style(self.min_style.unwrap_or(self.style.yellow()))
                .render(state.min_area, buf);
        }
        if state.can_max && !state.area.is_empty() {
            Span::from(" ⬤ ")
                .style(self.max_style.unwrap_or(self.style.green()))
                .render(state.max_area, buf);
        }

        if state.mouse_close.hover.get() {
            buf.set_style(
                state.close_area,
                revert_style(self.close_style.unwrap_or(self.style.red())),
            );
        }
        if state.mouse_min.hover.get() {
            buf.set_style(
                state.min_area,
                revert_style(self.min_style.unwrap_or(self.style.yellow())),
            );
        }
        if state.mouse_max.hover.get() {
            buf.set_style(
                state.max_area,
                revert_style(self.max_style.unwrap_or(self.style.green())),
            );
        }

        if state.mouse_move.drag.get() {
            buf.set_style(state.move_area, self.drag_style);
        } else if state.mouse_move.hover.get() {
            buf.set_style(state.move_area, self.hover_style);
        }

        if state.mouse_resize.drag.get() {
            buf.set_style(state.resize_area, self.drag_style);
        } else if state.mouse_resize.hover.get() {
            buf.set_style(state.resize_area, self.hover_style);
        }
    }
}
