use crate::_private::NonExhaustive;
use crate::WindowFrameOutcome;
use rat_event::util::MouseFlags;
use rat_event::{ConsumedEvent, Dialog, HandleEvent, ct_event};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Style;
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::Block;
use std::cmp::max;

#[derive(Debug)]
pub struct WindowFrameStyle {
    pub style: Style,
    pub top: Option<Style>,
    pub focus: Option<Style>,
    pub block: Option<Block<'static>>,
    pub hover: Option<Style>,
    pub drag: Option<Style>,
    pub close: Option<Style>,
    pub min: Option<Style>,
    pub max: Option<Style>,
    pub can_move: Option<bool>,
    pub can_resize: Option<bool>,
    pub can_close: Option<bool>,
    pub can_min: Option<bool>,
    pub can_max: Option<bool>,
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
    /// is this the top window?
    /// __read+write__
    pub top: bool,

    /// Window can be moved.
    /// __read+write__ May be overwritten by the widget.
    pub can_move: bool,
    /// Window can be resized.
    /// __read+write__ May be overwritten by the widget.
    pub can_resize: bool,
    /// Window can be closed.
    /// __read+write__ May be overwritten by the widget.
    pub can_close: bool,
    /// Window can be closed.
    /// __read+write__ May be overwritten by the widget.
    pub can_min: bool,
    /// Window can be closed.
    /// __read+write__ May be overwritten by the widget.
    pub can_max: bool,

    /// move area
    pub move_area: Rect,
    /// resize area
    pub resize_area: Rect,
    /// close area
    pub close_area: Rect,
    pub min_area: Rect,
    pub max_area: Rect,

    /// mouse flags for close area
    pub mouse_close: MouseFlags,
    pub mouse_min: MouseFlags,
    pub mouse_max: MouseFlags,
    /// mouse flags for resize area
    pub mouse_resize: MouseFlags,

    /// window and mouse position at the start of move
    pub start_move: (Rect, Position),
    /// mouse flags for move area
    pub mouse_move: MouseFlags,

    /// Focus for move/resize
    pub focus: FocusFlag,

    pub non_exhaustive: NonExhaustive,
}

impl Default for WindowFrameStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            top: Default::default(),
            focus: Default::default(),
            block: Default::default(),
            hover: Default::default(),
            drag: Default::default(),
            close: Default::default(),
            min: Default::default(),
            max: Default::default(),
            can_move: Default::default(),
            can_resize: Default::default(),
            can_close: Default::default(),
            can_min: Default::default(),
            can_max: Default::default(),
            non_exhaustive: NonExhaustive,
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
            top: Default::default(),
            can_move: true,
            can_resize: true,
            can_close: true,
            can_min: true,
            can_max: true,
            move_area: Default::default(),
            resize_area: Default::default(),
            close_area: Default::default(),
            min_area: Default::default(),
            max_area: Default::default(),
            mouse_close: Default::default(),
            mouse_min: Default::default(),
            mouse_max: Default::default(),
            mouse_resize: Default::default(),
            start_move: Default::default(),
            mouse_move: Default::default(),
            focus: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl HasFocus for WindowFrameState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        Rect::default()
    }

    fn navigable(&self) -> Navigation {
        Navigation::Leave
    }
}

impl WindowFrameState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Switch between maximized and normal size.
    pub fn flip_maximize(&mut self) {
        if self.area == self.limit && !self.arc_area.is_empty() {
            self.area = self.arc_area;
        } else {
            self.arc_area = self.area;
            self.area = self.limit;
        }
    }

    /// Switch between minimized and normal state.
    pub fn flip_minimize(&mut self) {
        if self.area == Rect::default() && !self.arc_area.is_empty() {
            self.area = self.arc_area;
        } else {
            self.arc_area = self.area;
            self.area = Rect::default();
        }
    }

    /// Set the window area and check the limits.
    ///
    /// It always resizes the area to keep it within the limits.
    ///
    /// Return
    ///
    /// Returns WindowFrameOutcome::Resized if the area is changed.
    pub fn set_resized_area(&mut self, mut new_area: Rect) -> WindowFrameOutcome {
        if new_area.x < self.limit.x {
            new_area.width -= self.limit.x - new_area.x;
            new_area.x = self.limit.x;
        }
        if new_area.y < self.limit.y {
            new_area.height -= self.limit.y - new_area.y;
            new_area.y = self.limit.y;
        }
        if new_area.right() > self.limit.right() {
            new_area.width -= new_area.right() - self.limit.right();
        }
        if new_area.bottom() > self.limit.bottom() {
            new_area.height -= new_area.bottom() - self.limit.bottom();
        }

        if new_area != self.area {
            self.area = new_area;
            WindowFrameOutcome::Resized
        } else {
            WindowFrameOutcome::Continue
        }
    }

    /// Set the window area and check the limits.
    ///
    /// If possible it moves the area to stay within the limits.
    /// If the given area is bigger than the limit it is clipped.
    ///
    /// Return
    ///
    /// Returns WindowFrameOutcome::Moved if the area is changed.
    pub fn set_moved_area(&mut self, mut new_area: Rect) -> WindowFrameOutcome {
        if new_area.x < self.limit.x {
            new_area.x = self.limit.x;
        }
        if new_area.y < self.limit.y {
            new_area.y = self.limit.y;
        }
        if new_area.right() > self.limit.right() {
            let delta = new_area.right() - self.limit.right();
            new_area.x -= delta;
        }
        if new_area.bottom() > self.limit.bottom() {
            let delta = new_area.bottom() - self.limit.bottom();
            new_area.y -= delta;
        }

        // need clip
        if new_area.x < self.limit.x {
            new_area.x = self.limit.x;
            new_area.width = self.limit.width;
        }
        if new_area.y < self.limit.y {
            new_area.y = self.limit.y;
            new_area.height = self.limit.height;
        }

        if new_area != self.area {
            self.area = new_area;
            WindowFrameOutcome::Moved
        } else {
            WindowFrameOutcome::Continue
        }
    }
}

impl HandleEvent<Event, Dialog, WindowFrameOutcome> for WindowFrameState {
    fn handle(&mut self, event: &Event, _qualifier: Dialog) -> WindowFrameOutcome {
        let r = if self.is_focused() {
            match event {
                ct_event!(keycode press Up) => {
                    let mut new_area = self.area;
                    if new_area.y > 0 {
                        new_area.y -= 1;
                    }
                    self.set_moved_area(new_area)
                }
                ct_event!(keycode press Down) => {
                    let mut new_area = self.area;
                    new_area.y += 1;
                    self.set_moved_area(new_area)
                }
                ct_event!(keycode press Left) => {
                    let mut new_area = self.area;
                    if new_area.x > 0 {
                        new_area.x -= 1;
                    }
                    self.set_moved_area(new_area)
                }
                ct_event!(keycode press Right) => {
                    let mut new_area = self.area;
                    new_area.x += 1;
                    self.set_moved_area(new_area)
                }

                ct_event!(keycode press Home) => {
                    let mut new_area = self.area;
                    new_area.x = self.limit.left();
                    self.set_moved_area(new_area)
                }
                ct_event!(keycode press End) => {
                    let mut new_area = self.area;
                    new_area.x = self.limit.right().saturating_sub(new_area.width);
                    self.set_moved_area(new_area)
                }
                ct_event!(keycode press CONTROL-Home) => {
                    let mut new_area = self.area;
                    new_area.y = self.limit.top();
                    self.set_moved_area(new_area)
                }
                ct_event!(keycode press CONTROL-End) => {
                    let mut new_area = self.area;
                    new_area.y = self.limit.bottom().saturating_sub(new_area.height);
                    self.set_moved_area(new_area)
                }

                ct_event!(keycode press ALT-Up) => {
                    let mut new_area = self.area;
                    if new_area.height > 1 {
                        new_area.height -= 1;
                    }
                    self.set_resized_area(new_area)
                }
                ct_event!(keycode press ALT-Down) => {
                    let mut new_area = self.area;
                    new_area.height += 1;
                    self.set_resized_area(new_area)
                }
                ct_event!(keycode press ALT-Left) => {
                    let mut new_area = self.area;
                    if new_area.width > 1 {
                        new_area.width -= 1;
                    }
                    self.set_resized_area(new_area)
                }
                ct_event!(keycode press ALT-Right) => {
                    let mut new_area = self.area;
                    new_area.width += 1;
                    self.set_resized_area(new_area)
                }

                ct_event!(keycode press CONTROL_ALT-Down) => {
                    let mut new_area = self.area;
                    if new_area.height > 1 {
                        new_area.y += 1;
                        new_area.height -= 1;
                    }
                    self.set_resized_area(new_area)
                }
                ct_event!(keycode press CONTROL_ALT-Up) => {
                    let mut new_area = self.area;
                    if new_area.y > 0 {
                        new_area.y -= 1;
                        new_area.height += 1;
                    }
                    self.set_resized_area(new_area)
                }
                ct_event!(keycode press CONTROL_ALT-Right) => {
                    let mut new_area = self.area;
                    if new_area.width > 1 {
                        new_area.x += 1;
                        new_area.width -= 1;
                    }
                    self.set_resized_area(new_area)
                }
                ct_event!(keycode press CONTROL_ALT-Left) => {
                    let mut new_area = self.area;
                    if new_area.x > 0 {
                        new_area.x -= 1;
                        new_area.width += 1;
                    }
                    self.set_resized_area(new_area)
                }

                ct_event!(keycode press CONTROL-Up) => {
                    let mut new_area = self.area;
                    if self.area.y != self.limit.y || self.area.height != self.limit.height {
                        new_area.y = self.limit.y;
                        new_area.height = self.limit.height;
                        self.arc_area.y = self.area.y;
                        self.arc_area.height = self.area.height;
                        self.set_resized_area(new_area)
                    } else {
                        WindowFrameOutcome::Unchanged
                    }
                }
                ct_event!(keycode press CONTROL-Down) => {
                    let mut new_area = self.area;
                    if !self.arc_area.is_empty() {
                        new_area.y = self.arc_area.y;
                        new_area.height = self.arc_area.height;
                        self.set_resized_area(new_area)
                    } else {
                        WindowFrameOutcome::Unchanged
                    }
                }
                ct_event!(keycode press CONTROL-Right) => {
                    let mut new_area = self.area;
                    if self.area.x != self.limit.x || self.area.width != self.limit.width {
                        new_area.x = self.limit.x;
                        new_area.width = self.limit.width;
                        self.arc_area.x = self.area.x;
                        self.arc_area.width = self.area.width;
                        self.set_resized_area(new_area)
                    } else {
                        WindowFrameOutcome::Unchanged
                    }
                }
                ct_event!(keycode press CONTROL-Left) => {
                    let mut new_area = self.area;
                    if !self.arc_area.is_empty() {
                        new_area.x = self.arc_area.x;
                        new_area.width = self.arc_area.width;
                        self.set_resized_area(new_area)
                    } else {
                        WindowFrameOutcome::Unchanged
                    }
                }

                _ => WindowFrameOutcome::Continue,
            }
        } else {
            WindowFrameOutcome::Continue
        };

        r.or_else(|| match event {
            ct_event!(mouse any for m) if self.mouse_close.hover(self.close_area, m) => {
                WindowFrameOutcome::Changed
            }
            ct_event!(mouse down Left for x,y) if self.close_area.contains((*x, *y).into()) => {
                WindowFrameOutcome::ShouldClose
            }
            ct_event!(mouse any for m) if self.mouse_min.hover(self.min_area, m) => {
                WindowFrameOutcome::Changed
            }
            ct_event!(mouse down Left for x,y) if self.min_area.contains((*x, *y).into()) => {
                self.flip_minimize();
                WindowFrameOutcome::Changed
            }
            ct_event!(mouse any for m) if self.mouse_max.hover(self.max_area, m) => {
                WindowFrameOutcome::Changed
            }
            ct_event!(mouse down Left for x,y) if self.max_area.contains((*x, *y).into()) => {
                self.flip_maximize();
                WindowFrameOutcome::Changed
            }

            ct_event!(mouse any for m) if self.mouse_resize.hover(self.resize_area, m) => {
                WindowFrameOutcome::Changed
            }
            ct_event!(mouse any for m) if self.mouse_resize.drag(self.resize_area, m) => {
                let mut new_area = self.area;
                new_area.width = max(10, m.column.saturating_sub(self.area.x));
                new_area.height = max(3, m.row.saturating_sub(self.area.y));
                self.set_resized_area(new_area)
            }

            ct_event!(mouse any for m) if self.mouse_move.hover(self.move_area, m) => {
                WindowFrameOutcome::Changed
            }
            ct_event!(mouse any for m) if self.mouse_move.doubleclick(self.move_area, m) => {
                self.flip_maximize();
                WindowFrameOutcome::Resized
            }
            ct_event!(mouse any for m) if self.mouse_move.drag(self.move_area, m) => {
                let delta_x = m.column as i16 - self.start_move.1.x as i16;
                let delta_y = m.row as i16 - self.start_move.1.y as i16;
                self.set_moved_area(Rect::new(
                    self.start_move.0.x.saturating_add_signed(delta_x),
                    self.start_move.0.y.saturating_add_signed(delta_y),
                    self.start_move.0.width,
                    self.start_move.0.height,
                ))
            }
            ct_event!(mouse down Left for x,y) if self.move_area.contains((*x, *y).into()) => {
                self.start_move = (self.area, Position::new(*x, *y));
                WindowFrameOutcome::Changed
            }
            _ => WindowFrameOutcome::Continue,
        })
    }
}
