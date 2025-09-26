//!
//! Widget for a moveable window.
//!
use crate::_private::NonExhaustive;
use rat_event::util::{MouseFlags, mouse_trap};
use rat_event::{ConsumedEvent, Dialog, HandleEvent, Outcome, ct_event};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
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
pub struct WindowFrame<'a> {
    block: Block<'a>,
    style: Style,
    hover: Style,
    drag: Style,
    limit: Option<Rect>,
    can_move: Option<bool>,
    can_resize: Option<bool>,
    can_close: Option<bool>,
    modal: Option<bool>,
}

#[derive(Debug)]
pub struct WindowFrameStyle {
    pub style: Style,
    pub block: Block<'static>,
    pub hover: Option<Style>,
    pub drag: Option<Style>,
    pub can_move: Option<bool>,
    pub can_resize: Option<bool>,
    pub can_close: Option<bool>,
    pub modal: Option<bool>,
    pub non_exhaustive: NonExhaustive,
}

/// Window state.
#[derive(Debug)]
pub struct WindowFrameState {
    /// Outer limit for the window.
    /// This will be set by the widget during render.
    /// __read only__
    pub limit: Rect,
    /// the rendered window-area.
    /// change this area to move the window.
    /// __read+write__
    pub area: Rect,
    /// archived area. used when switching between
    /// maximized and normal size.
    pub arc_area: Rect,
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
    /// Modal window.
    pub modal: bool,

    /// move area
    pub move_area: Rect,
    /// resize area
    pub resize_area: Rect,
    /// close area
    pub close_area: Rect,

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

    pub focus: FocusFlag,

    pub non_exhaustive: NonExhaustive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WindowFrameOutcome {
    /// The given event was not handled at all.
    Continue,
    /// The event was handled, no repaint necessary.
    Unchanged,
    /// The event was handled, repaint necessary.
    Changed,
    /// Request close.
    ShouldClose,
    /// Has moved.
    Moved,
    /// Has resized.
    Resized,
}

impl ConsumedEvent for WindowFrameOutcome {
    fn is_consumed(&self) -> bool {
        *self != WindowFrameOutcome::Continue
    }
}

impl Default for WindowFrameStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            block: Block::bordered(),
            hover: None,
            drag: None,
            can_move: None,
            can_resize: None,
            can_close: None,
            modal: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl From<Outcome> for WindowFrameOutcome {
    fn from(value: Outcome) -> Self {
        match value {
            Outcome::Continue => WindowFrameOutcome::Continue,
            Outcome::Unchanged => WindowFrameOutcome::Unchanged,
            Outcome::Changed => WindowFrameOutcome::Changed,
        }
    }
}

impl From<WindowFrameOutcome> for Outcome {
    fn from(value: WindowFrameOutcome) -> Self {
        match value {
            WindowFrameOutcome::Continue => Outcome::Continue,
            WindowFrameOutcome::Unchanged => Outcome::Unchanged,
            WindowFrameOutcome::Changed => Outcome::Changed,
            WindowFrameOutcome::Moved => Outcome::Changed,
            WindowFrameOutcome::Resized => Outcome::Changed,
            WindowFrameOutcome::ShouldClose => Outcome::Continue,
        }
    }
}

impl<'a> WindowFrame<'a> {
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
            modal: Default::default(),
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

    /// Is this a modal window that blocks everything else.
    /// Aka a Dialog
    pub fn modal(mut self, v: bool) -> Self {
        self.modal = Some(v);
        self
    }

    /// Window block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = block.style(self.style);
        self
    }

    pub fn styles(mut self, styles: WindowFrameStyle) -> Self {
        self.style = styles.style;
        self.block = styles.block;
        if let Some(hover) = styles.hover {
            self.hover = hover;
        }
        if let Some(drag) = styles.drag {
            self.drag = drag;
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
        if let Some(modal) = styles.modal {
            self.can_move = Some(modal);
        }
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

impl<'a> StatefulWidget for WindowFrame<'a> {
    type State = WindowFrameState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if let Some(limit) = self.limit {
            state.limit = limit;
        } else {
            state.limit = area;
        }
        state.area = state.area.intersection(state.limit);
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
        if let Some(v) = self.modal {
            state.modal = v;
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
        if state.can_close {
            state.close_area =
                Rect::new(state.area.right().saturating_sub(4), state.area.top(), 3, 1);
        } else {
            state.close_area = Default::default();
        }
        if state.can_move {
            if state.can_close {
                state.move_area = Rect::new(
                    state.area.x + 1,
                    state.area.y,
                    state.area.width.saturating_sub(6),
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

        for y in state.area.top()..state.area.bottom() {
            for x in state.area.left()..state.area.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.reset();
                }
            }
        }

        self.block.render(state.area, buf);

        if state.can_move {
            Span::from("[x]")
                .style(self.style)
                .render(state.close_area, buf);
        }

        if state.mouse_close.hover.get() {
            buf.set_style(state.close_area, self.hover);
        }

        if state.mouse_move.drag.get() {
            buf.set_style(state.move_area, self.drag);
        } else if state.mouse_move.hover.get() {
            buf.set_style(state.move_area, self.hover);
        }

        if state.mouse_resize.drag.get() {
            buf.set_style(state.resize_area, self.drag);
        } else if state.mouse_resize.hover.get() {
            buf.set_style(state.resize_area, self.hover);
        }
    }
}

impl Default for WindowFrameState {
    fn default() -> Self {
        Self {
            limit: Default::default(),
            area: Default::default(),
            arc_area: Default::default(),
            widget_area: Default::default(),
            can_move: true,
            can_resize: true,
            can_close: true,
            modal: false,
            move_area: Default::default(),
            resize_area: Default::default(),
            close_area: Default::default(),
            mouse_close: Default::default(),
            mouse_resize: Default::default(),
            start_move_area: Default::default(),
            start_move: Default::default(),
            mouse_move: Default::default(),
            focus: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl HasFocus for WindowFrameState {
    fn build(&self, builder: &mut FocusBuilder) {
        if self.modal {
            builder.widget_with_flags(self.focus(), self.area(), self.area_z(), Navigation::Lock)
        } else {
            builder.widget_with_flags(self.focus(), self.area(), self.area_z(), Navigation::None)
        }
    }

    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        Rect::default()
    }
}

impl WindowFrameState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn flip_max(&mut self) {
        if self.area == self.limit && !self.arc_area.is_empty() {
            self.area = self.arc_area;
        } else {
            self.arc_area = self.area;
            self.area = self.limit;
        }
    }
}

impl HandleEvent<crossterm::event::Event, Dialog, WindowFrameOutcome> for WindowFrameState {
    fn handle(
        &mut self,
        event: &crossterm::event::Event,
        _qualifier: Dialog,
    ) -> WindowFrameOutcome {
        let r = match event {
            ct_event!(mouse any for m) if self.mouse_close.hover(self.close_area, m) => {
                WindowFrameOutcome::Changed
            }
            ct_event!(mouse down Left for x,y) if self.close_area.contains((*x, *y).into()) => {
                WindowFrameOutcome::ShouldClose
            }

            ct_event!(mouse any for m) if self.mouse_resize.hover(self.resize_area, m) => {
                WindowFrameOutcome::Changed
            }
            ct_event!(mouse any for m) if self.mouse_resize.drag(self.resize_area, m) => {
                let mut new_area = self.area;

                new_area.width = max(10, m.column.saturating_sub(self.area.x));
                new_area.height = max(3, m.row.saturating_sub(self.area.y));

                if new_area.right() <= self.limit.right()
                    && new_area.bottom() <= self.limit.bottom()
                {
                    self.area = new_area;
                    WindowFrameOutcome::Resized
                } else {
                    WindowFrameOutcome::Continue
                }
            }

            ct_event!(mouse any for m) if self.mouse_move.hover(self.move_area, m) => {
                WindowFrameOutcome::Changed
            }
            ct_event!(mouse any for m) if self.mouse_move.doubleclick(self.move_area, m) => {
                self.flip_max();
                WindowFrameOutcome::Resized
            }
            ct_event!(mouse any for m) if self.mouse_move.drag(self.move_area, m) => {
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
                    WindowFrameOutcome::Moved
                } else {
                    WindowFrameOutcome::Continue
                }
            }
            ct_event!(mouse down Left for x,y) if self.move_area.contains((*x, *y).into()) => {
                self.start_move_area = self.area;
                self.start_move = Position::new(*x, *y);
                WindowFrameOutcome::Changed
            }
            _ => WindowFrameOutcome::Continue,
        };

        r.or_else(|| {
            if self.modal {
                WindowFrameOutcome::Unchanged
            } else {
                mouse_trap(event, self.area).into()
            }
        })
    }
}
