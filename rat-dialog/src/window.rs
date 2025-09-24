use crate::_private::NonExhaustive;
///
/// Widget for a moveable window.
///
use rat_event::util::MouseFlags;
use rat_event::{ConsumedEvent, HandleEvent, Outcome, Regular, ct_event};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{Block, StatefulWidget, Widget};
use std::cmp::max;

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
pub struct Window<'a> {
    block: Block<'a>,
    style: Style,
    hover: Style,
    drag: Style,
    limit: Option<Rect>,
    can_move: Option<bool>,
    can_resize: Option<bool>,
    can_close: Option<bool>,
}

#[derive(Debug)]
pub struct WindowState {
    /// Outer limit for the window.
    /// This will be set by the widget during render.
    /// __read only__
    pub limit: Rect,
    /// the rendered window-area.
    /// change this area to move the window.
    /// __read+write__
    pub area: Rect,
    /// area for window content.
    /// __read only__ renewed with each render.
    pub widget_area: Rect,

    /// Window can be moved.
    /// __read+write__ May be overwritten by the widget.
    pub can_move: bool,
    /// Window can be resized.
    /// __read+write__ May be overwritten by the widget.
    pub can_resize: bool,
    /// Window can be closed.
    /// __read+write__ May be overwritten by the widget.
    pub can_close: bool,

    /// move area
    pub move_: Rect,
    /// resize area
    pub resize: Rect,
    /// close area
    pub close: Rect,

    /// mouse flags for close area
    pub mouse_close: MouseFlags,
    /// mouse flags for resize area
    pub mouse_resize: MouseFlags,

    /// window at the start of move
    pub start_move_area: Rect,
    /// mouse position at the start of move
    pub start_move: Position,
    /// mouse flags for move area
    pub mouse_move: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WindowOutcome {
    /// The given event was not handled at all.
    Continue,
    /// The event was handled, no repaint necessary.
    Unchanged,
    /// The event was handled, repaint necessary.
    Changed,
    /// Request close.
    ShouldClose,
    /// Moved
    Moved,
    /// Resized
    Resized,
}

impl ConsumedEvent for WindowOutcome {
    fn is_consumed(&self) -> bool {
        *self != WindowOutcome::Continue
    }
}

impl From<WindowOutcome> for Outcome {
    fn from(value: WindowOutcome) -> Self {
        match value {
            WindowOutcome::Continue => Outcome::Continue,
            WindowOutcome::Unchanged => Outcome::Unchanged,
            WindowOutcome::Changed => Outcome::Changed,
            WindowOutcome::Moved => Outcome::Changed,
            WindowOutcome::Resized => Outcome::Changed,
            WindowOutcome::ShouldClose => Outcome::Continue,
        }
    }
}

impl<'a> Window<'a> {
    pub fn new() -> Self {
        Self {
            block: Default::default(),
            style: Default::default(),
            hover: Default::default(),
            drag: Default::default(),
            limit: Default::default(),
            can_move: Default::default(),
            can_resize: Default::default(),
            can_close: Default::default(),
        }
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

    /// Window block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = block.style(self.style);
        self
    }

    /// Window base style
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self.block = self.block.style(style);
        self
    }

    /// Hover style
    pub fn hover(mut self, hover: Style) -> Self {
        self.hover = hover;
        self
    }

    /// Drag style
    pub fn drag(mut self, drag: Style) -> Self {
        self.drag = drag;
        self
    }
}

impl<'a> StatefulWidget for Window<'a> {
    type State = WindowState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if let Some(limit) = self.limit {
            state.limit = limit;
        } else {
            state.limit = area;
        }
        state.widget_area = self.block.inner(state.area);
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
            state.resize = Rect::new(
                state.area.right().saturating_sub(2),
                state.area.bottom().saturating_sub(1),
                2,
                1,
            );
        } else {
            state.resize = Default::default();
        }
        if state.can_close {
            state.close = Rect::new(state.area.right().saturating_sub(4), state.area.top(), 3, 1);
        } else {
            state.close = Default::default();
        }
        if state.can_move {
            state.move_ = Rect::new(
                state.area.x + 1,
                state.area.y,
                state.area.width.saturating_sub(6),
                1,
            );
        } else {
            state.move_ = Default::default();
        }

        self.block.render(state.area, buf);

        if state.can_move {
            Span::from("[x]").style(self.style).render(state.close, buf);
        }

        if state.mouse_close.hover.get() {
            buf.set_style(state.close, self.hover);
        }
        if state.mouse_move.hover.get() {
            buf.set_style(state.move_, self.hover);
        }
        if state.mouse_resize.hover.get() {
            buf.set_style(state.resize, self.hover);
        }
        if state.mouse_move.drag.get() {
            buf.set_style(state.move_, self.drag);
        }
        if state.mouse_resize.drag.get() {
            buf.set_style(state.resize, self.drag);
        }
    }
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            limit: Default::default(),
            area: Default::default(),
            widget_area: Default::default(),
            can_move: true,
            can_resize: true,
            can_close: true,
            move_: Default::default(),
            resize: Default::default(),
            close: Default::default(),
            mouse_close: Default::default(),
            mouse_resize: Default::default(),
            start_move_area: Default::default(),
            start_move: Default::default(),
            mouse_move: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl WindowState {
    pub fn new() -> Self {
        Self::default()
    }
}

impl HandleEvent<crossterm::event::Event, Regular, WindowOutcome> for WindowState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> WindowOutcome {
        match event {
            ct_event!(mouse any for m) if self.mouse_close.hover(self.close, m) => {
                WindowOutcome::Changed
            }
            ct_event!(mouse any for m) if self.mouse_resize.hover(self.resize, m) => {
                WindowOutcome::Changed
            }
            ct_event!(mouse any for m) if self.mouse_move.hover(self.move_, m) => {
                WindowOutcome::Changed
            }
            ct_event!(mouse any for m) if self.mouse_resize.drag(self.resize, m) => {
                let mut new_area = self.area;

                new_area.width = max(10, m.column.saturating_sub(self.area.x));
                new_area.height = max(3, m.row.saturating_sub(self.area.y));

                if new_area.right() <= self.limit.right()
                    && new_area.bottom() <= self.limit.bottom()
                {
                    self.area = new_area;
                    WindowOutcome::Resized
                } else {
                    WindowOutcome::Continue
                }
            }
            ct_event!(mouse any for m) if self.mouse_move.drag(self.move_, m) => {
                let delta_x = m.column as i16 - self.start_move.x as i16;
                let delta_y = m.row as i16 - self.start_move.y as i16;
                let new_area = Rect::new(
                    self.start_move_area.x.saturating_add_signed(delta_x),
                    self.start_move_area.y.saturating_add_signed(delta_y),
                    self.start_move_area.width,
                    self.start_move_area.height,
                );

                if new_area.left() >= self.limit.left()
                    && new_area.top() >= self.limit.top()
                    && new_area.right() <= self.limit.right()
                    && new_area.bottom() <= self.limit.bottom()
                {
                    self.area = new_area;
                    WindowOutcome::Moved
                } else {
                    WindowOutcome::Continue
                }
            }
            ct_event!(mouse down Left for x,y) if self.move_.contains((*x, *y).into()) => {
                self.start_move_area = self.area;
                self.start_move = Position::new(*x, *y);
                WindowOutcome::Changed
            }
            ct_event!(mouse down Left for x,y) if self.close.contains((*x, *y).into()) => {
                WindowOutcome::ShouldClose
            }
            _ => WindowOutcome::Continue,
        }
    }
}
