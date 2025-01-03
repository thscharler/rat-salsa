//!
//! Some utility functions that pop up all the time.
//!

use crate::Outcome;
use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Position, Rect};
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::SystemTime;

/// Which of the given rects is at the position.
pub fn item_at(areas: &[Rect], x_pos: u16, y_pos: u16) -> Option<usize> {
    for (i, r) in areas.iter().enumerate() {
        if y_pos >= r.top() && y_pos < r.bottom() && x_pos >= r.left() && x_pos < r.right() {
            return Some(i);
        }
    }
    None
}

/// Which row of the given contains the position.
/// This uses only the vertical components of the given areas.
///
/// You might want to limit calling this functions when the full
/// position is inside your target rect.
pub fn row_at(areas: &[Rect], y_pos: u16) -> Option<usize> {
    for (i, r) in areas.iter().enumerate() {
        if y_pos >= r.top() && y_pos < r.bottom() {
            return Some(i);
        }
    }
    None
}

/// Column at given position.
/// This uses only the horizontal components of the given areas.
///
/// You might want to limit calling this functions when the full
/// position is inside your target rect.
pub fn column_at(areas: &[Rect], x_pos: u16) -> Option<usize> {
    for (i, r) in areas.iter().enumerate() {
        if x_pos >= r.left() && x_pos < r.right() {
            return Some(i);
        }
    }
    None
}

/// Find a row position when dragging with the mouse. This uses positions
/// outside the given areas to estimate an invisible row that could be meant
/// by the mouse position. It uses the heuristic `1 row == 1 item` for simplicity’s
/// sake.
///
/// Rows outside the bounds are returned as Err(isize), rows inside as Ok(usize).
pub fn row_at_drag(encompassing: Rect, areas: &[Rect], y_pos: u16) -> Result<usize, isize> {
    if let Some(row) = row_at(areas, y_pos) {
        return Ok(row);
    }

    // assume row-height=1 for outside the box.
    #[allow(clippy::collapsible_else_if)]
    if y_pos < encompassing.top() {
        Err(y_pos as isize - encompassing.top() as isize)
    } else {
        if let Some(last) = areas.last() {
            Err(y_pos as isize - last.bottom() as isize + 1)
        } else {
            Err(y_pos as isize - encompassing.top() as isize)
        }
    }
}

/// Find a column position when dragging with the mouse. This uses positions
/// outside the given areas to estimate an invisible column that could be meant
/// by the mouse position. It uses the heuristic `1 column == 1 item` for simplicity’s
/// sake.
///
/// Columns outside the bounds are returned as Err(isize), rows inside as Ok(usize).
pub fn column_at_drag(encompassing: Rect, areas: &[Rect], x_pos: u16) -> Result<usize, isize> {
    if let Some(column) = column_at(areas, x_pos) {
        return Ok(column);
    }

    // change by 1 column if outside the box
    #[allow(clippy::collapsible_else_if)]
    if x_pos < encompassing.left() {
        Err(x_pos as isize - encompassing.left() as isize)
    } else {
        if let Some(last) = areas.last() {
            Err(x_pos as isize - last.right() as isize + 1)
        } else {
            Err(x_pos as isize - encompassing.left() as isize)
        }
    }
}

/// This function consumes all mouse-events in the given area,
/// except Drag events.
///
/// This should catch all events when using a popup area.
pub fn mouse_trap(event: &crossterm::event::Event, area: Rect) -> Outcome {
    match event {
        crossterm::event::Event::Mouse(MouseEvent {
            kind:
                MouseEventKind::ScrollLeft
                | MouseEventKind::ScrollRight
                | MouseEventKind::ScrollUp
                | MouseEventKind::ScrollDown
                | MouseEventKind::Down(_)
                | MouseEventKind::Up(_)
                | MouseEventKind::Moved,
            column,
            row,
            ..
        }) if area.contains(Position::new(*column, *row)) => Outcome::Unchanged,
        _ => Outcome::Continue,
    }
}

/// Click states for double click.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Clicks {
    #[default]
    None,
    Down1(usize),
    Up1(usize),
    Down2(usize),
}

/// Some state for mouse interactions.
///
/// This helps with double-click and mouse drag recognition.
/// Add this to your widget state.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MouseFlags {
    /// Timestamp for double click
    pub time: Cell<Option<SystemTime>>,
    /// Flag for the first down.
    pub click: Cell<Clicks>,
    /// Drag enabled.
    pub drag: Cell<bool>,
    /// Hover detect.
    pub hover: Cell<bool>,
}

impl MouseFlags {
    /// Returns column/row extracted from the Mouse-Event.
    pub fn pos_of(&self, event: &MouseEvent) -> (u16, u16) {
        (event.column, event.row)
    }

    /// Which of the given rects is at the position.
    pub fn item_at(&self, areas: &[Rect], x_pos: u16, y_pos: u16) -> Option<usize> {
        item_at(areas, x_pos, y_pos)
    }

    /// Which row of the given contains the position.
    /// This uses only the vertical components of the given areas.
    ///
    /// You might want to limit calling this functions when the full
    /// position is inside your target rect.
    pub fn row_at(&self, areas: &[Rect], y_pos: u16) -> Option<usize> {
        row_at(areas, y_pos)
    }

    /// Column at given position.
    /// This uses only the horizontal components of the given areas.
    ///
    /// You might want to limit calling this functions when the full
    /// position is inside your target rect.
    pub fn column_at(&self, areas: &[Rect], x_pos: u16) -> Option<usize> {
        column_at(areas, x_pos)
    }

    /// Find a row position when dragging with the mouse. This uses positions
    /// outside the given areas to estimate an invisible row that could be meant
    /// by the mouse position. It uses the heuristic `1 row == 1 item` for simplicity’s
    /// sake.
    ///
    /// Rows outside the bounds are returned as Err(isize), rows inside as Ok(usize).
    pub fn row_at_drag(
        &self,
        encompassing: Rect,
        areas: &[Rect],
        y_pos: u16,
    ) -> Result<usize, isize> {
        row_at_drag(encompassing, areas, y_pos)
    }

    /// Find a column position when dragging with the mouse. This uses positions
    /// outside the given areas to estimate an invisible column that could be meant
    /// by the mouse position. It uses the heuristic `1 column == 1 item` for simplicity’s
    /// sake.
    ///
    /// Columns outside the bounds are returned as Err(isize), rows inside as Ok(usize).
    pub fn column_at_drag(
        &self,
        encompassing: Rect,
        areas: &[Rect],
        x_pos: u16,
    ) -> Result<usize, isize> {
        column_at_drag(encompassing, areas, x_pos)
    }

    /// Checks if this is a hover event for the widget.
    pub fn hover(&self, area: Rect, event: &MouseEvent) -> bool {
        match event {
            MouseEvent {
                kind: MouseEventKind::Moved,
                column,
                row,
                modifiers: KeyModifiers::NONE,
            } => {
                let old_hover = self.hover.get();
                if area.contains((*column, *row).into()) {
                    self.hover.set(true);
                } else {
                    self.hover.set(false);
                }
                old_hover != self.hover.get()
            }
            _ => false,
        }
    }

    /// Checks if this is a drag event for the widget.
    ///
    /// It makes sense to allow drag events outside the given area, if the
    /// drag has been started with a click to the given area.
    ///
    /// This can be integrated in the event-match with a guard:
    ///
    /// ```rust ignore
    /// match event {
    ///         Event::Mouse(m) if state.mouse.drag(state.area, m) => {
    ///             // ...
    ///             Outcome::Changed
    ///         }
    /// }
    /// ```
    pub fn drag(&self, area: Rect, event: &MouseEvent) -> bool {
        self.drag2(area, event, KeyModifiers::NONE)
    }

    /// Checks if this is a drag event for the widget.
    ///
    /// It makes sense to allow drag events outside the given area, if the
    /// drag has been started with a click to the given area.
    ///
    /// This function handles that case.
    pub fn drag2(&self, area: Rect, event: &MouseEvent, filter: KeyModifiers) -> bool {
        match event {
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                modifiers,
            } if *modifiers == filter => {
                if area.contains((*column, *row).into()) {
                    self.drag.set(true);
                } else {
                    self.drag.set(false);
                }
            }
            MouseEvent {
                kind: MouseEventKind::Drag(MouseButton::Left),
                modifiers,
                ..
            } if *modifiers == filter => {
                if self.drag.get() {
                    return true;
                }
            }
            MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left) | MouseEventKind::Moved,
                ..
            } => {
                self.drag.set(false);
            }

            _ => {}
        }

        false
    }

    /// Checks for double-click events.
    ///
    /// This can be integrated in the event-match with a guard:
    ///
    /// ```rust ignore
    /// match event {
    ///         Event::Mouse(m) if state.mouse.doubleclick(state.area, m) => {
    ///             state.flip = !state.flip;
    ///             Outcome::Changed
    ///         }
    /// }
    /// ```
    ///
    pub fn doubleclick(&self, area: Rect, event: &MouseEvent) -> bool {
        self.doubleclick2(area, event, KeyModifiers::NONE)
    }

    /// Checks for double-click events.
    /// This one can have an extra KeyModifiers.
    ///
    /// This can be integrated in the event-match with a guard:
    ///
    /// ```rust ignore
    /// match event {
    ///         Event::Mouse(m) if state.mouse.doubleclick(state.area, m) => {
    ///             state.flip = !state.flip;
    ///             Outcome::Changed
    ///         }
    /// }
    /// ```
    ///
    pub fn doubleclick2(&self, area: Rect, event: &MouseEvent, filter: KeyModifiers) -> bool {
        match event {
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                modifiers,
            } if *modifiers == filter => 'f: {
                if area.contains((*column, *row).into()) {
                    match self.click.get() {
                        Clicks::Up1(_) => {
                            if let Some(time) = self.time.get() {
                                if time.elapsed().unwrap_or_default().as_millis() as u32
                                    > double_click_timeout()
                                {
                                    self.time.set(Some(SystemTime::now()));
                                    self.click.set(Clicks::Down1(0));
                                    break 'f false;
                                }
                            }
                            self.click.set(Clicks::Down2(0));
                        }
                        _ => {
                            self.time.set(Some(SystemTime::now()));
                            self.click.set(Clicks::Down1(0));
                        }
                    }
                    break 'f false;
                } else {
                    self.time.set(None);
                    self.click.set(Clicks::None);
                    break 'f false;
                }
            }
            MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left),
                column,
                row,
                modifiers,
            } if *modifiers == filter => 'f: {
                if area.contains((*column, *row).into()) {
                    match self.click.get() {
                        Clicks::Down1(_) => {
                            self.click.set(Clicks::Up1(0));
                            break 'f false;
                        }
                        Clicks::Up1(_) => {
                            self.click.set(Clicks::None);
                            break 'f true;
                        }
                        Clicks::Down2(_) => {
                            self.click.set(Clicks::None);
                            break 'f true;
                        }
                        _ => {
                            self.click.set(Clicks::None);
                            break 'f false;
                        }
                    }
                } else {
                    self.click.set(Clicks::None);
                    break 'f false;
                }
            }
            _ => false,
        }
    }
}

/// Some state for mouse interactions with multiple areas.
///
/// This helps with double-click and mouse drag recognition.
/// Add this to your widget state.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MouseFlagsN {
    /// Timestamp for double click
    pub time: Cell<Option<SystemTime>>,
    /// Flag for the first down.
    pub click: Cell<Clicks>,
    /// Drag enabled.
    pub drag: Cell<Option<usize>>,
    /// Hover detect.
    pub hover: Cell<Option<usize>>,
}

impl MouseFlagsN {
    /// Returns column/row extracted from the Mouse-Event.
    pub fn pos_of(&self, event: &MouseEvent) -> (u16, u16) {
        (event.column, event.row)
    }

    /// Which of the given rects is at the position.
    pub fn item_at(&self, areas: &[Rect], x_pos: u16, y_pos: u16) -> Option<usize> {
        item_at(areas, x_pos, y_pos)
    }

    /// Which row of the given contains the position.
    /// This uses only the vertical components of the given areas.
    ///
    /// You might want to limit calling this functions when the full
    /// position is inside your target rect.
    pub fn row_at(&self, areas: &[Rect], y_pos: u16) -> Option<usize> {
        row_at(areas, y_pos)
    }

    /// Column at given position.
    /// This uses only the horizontal components of the given areas.
    ///
    /// You might want to limit calling this functions when the full
    /// position is inside your target rect.
    pub fn column_at(&self, areas: &[Rect], x_pos: u16) -> Option<usize> {
        column_at(areas, x_pos)
    }

    /// Find a row position when dragging with the mouse. This uses positions
    /// outside the given areas to estimate an invisible row that could be meant
    /// by the mouse position. It uses the heuristic `1 row == 1 item` for simplicity’s
    /// sake.
    ///
    /// Rows outside the bounds are returned as Err(isize), rows inside as Ok(usize).
    pub fn row_at_drag(
        &self,
        encompassing: Rect,
        areas: &[Rect],
        y_pos: u16,
    ) -> Result<usize, isize> {
        row_at_drag(encompassing, areas, y_pos)
    }

    /// Find a column position when dragging with the mouse. This uses positions
    /// outside the given areas to estimate an invisible column that could be meant
    /// by the mouse position. It uses the heuristic `1 column == 1 item` for simplicity’s
    /// sake.
    ///
    /// Columns outside the bounds are returned as Err(isize), rows inside as Ok(usize).
    pub fn column_at_drag(
        &self,
        encompassing: Rect,
        areas: &[Rect],
        x_pos: u16,
    ) -> Result<usize, isize> {
        column_at_drag(encompassing, areas, x_pos)
    }

    /// Checks if this is a hover event for the widget.
    pub fn hover(&self, areas: &[Rect], event: &MouseEvent) -> bool {
        match event {
            MouseEvent {
                kind: MouseEventKind::Moved,
                column,
                row,
                modifiers: KeyModifiers::NONE,
            } => {
                let old_hover = self.hover.get();
                if let Some(n) = self.item_at(areas, *column, *row) {
                    self.hover.set(Some(n));
                } else {
                    self.hover.set(None);
                }
                old_hover != self.hover.get()
            }
            _ => false,
        }
    }

    /// Checks if this is a drag event for the widget.
    ///
    /// It makes sense to allow drag events outside the given area, if the
    /// drag has been started with a click to the given area.
    ///
    /// This function handles that case.
    pub fn drag(&self, areas: &[Rect], event: &MouseEvent) -> bool {
        self.drag2(areas, event, KeyModifiers::NONE)
    }

    /// Checks if this is a drag event for the widget.
    ///
    /// It makes sense to allow drag events outside the given area, if the
    /// drag has been started with a click to the given area.
    ///
    /// This function handles that case.
    pub fn drag2(&self, areas: &[Rect], event: &MouseEvent, filter: KeyModifiers) -> bool {
        match event {
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                modifiers,
            } if *modifiers == filter => {
                self.drag.set(None);
                for (n, area) in areas.iter().enumerate() {
                    if area.contains((*column, *row).into()) {
                        self.drag.set(Some(n));
                    }
                }
            }
            MouseEvent {
                kind: MouseEventKind::Drag(MouseButton::Left),
                modifiers,
                ..
            } if *modifiers == filter => {
                if self.drag.get().is_some() {
                    return true;
                }
            }
            MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left) | MouseEventKind::Moved,
                ..
            } => {
                self.drag.set(None);
            }

            _ => {}
        }

        false
    }

    /// Checks for double-click events.
    ///
    /// This can be integrated in the event-match with a guard:
    ///
    /// ```rust ignore
    /// match event {
    ///         Event::Mouse(m) if state.mouse.doubleclick(state.area, m) => {
    ///             state.flip = !state.flip;
    ///             Outcome::Changed
    ///         }
    /// }
    /// ```
    ///
    pub fn doubleclick(&self, areas: &[Rect], event: &MouseEvent) -> bool {
        self.doubleclick2(areas, event, KeyModifiers::NONE)
    }

    /// Checks for double-click events.
    /// This one can have an extra KeyModifiers.
    ///
    /// This can be integrated in the event-match with a guard:
    ///
    /// ```rust ignore
    /// match event {
    ///         Event::Mouse(m) if state.mouse.doubleclick(state.area, m) => {
    ///             state.flip = !state.flip;
    ///             Outcome::Changed
    ///         }
    /// }
    /// ```
    ///
    pub fn doubleclick2(&self, areas: &[Rect], event: &MouseEvent, filter: KeyModifiers) -> bool {
        match event {
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                modifiers,
            } if *modifiers == filter => 'f: {
                for (n, area) in areas.iter().enumerate() {
                    if area.contains((*column, *row).into()) {
                        match self.click.get() {
                            Clicks::Up1(v) => {
                                if let Some(time) = self.time.get() {
                                    if time.elapsed().unwrap_or_default().as_millis() as u32
                                        > double_click_timeout()
                                    {
                                        self.time.set(Some(SystemTime::now()));
                                        self.click.set(Clicks::Down1(n));
                                        break 'f false;
                                    }
                                }
                                if n == v {
                                    self.click.set(Clicks::Down2(n));
                                } else {
                                    self.click.set(Clicks::None);
                                }
                            }
                            _ => {
                                self.time.set(Some(SystemTime::now()));
                                self.click.set(Clicks::Down1(n));
                            }
                        }
                        break 'f false;
                    }
                }
                self.time.set(None);
                self.click.set(Clicks::None);
                false
            }
            MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left),
                column,
                row,
                modifiers,
            } if *modifiers == filter => 'f: {
                for (n, area) in areas.iter().enumerate() {
                    if area.contains((*column, *row).into()) {
                        match self.click.get() {
                            Clicks::Down1(v) => {
                                if n == v {
                                    self.click.set(Clicks::Up1(v));
                                } else {
                                    self.click.set(Clicks::None);
                                }
                            }
                            Clicks::Up1(v) => {
                                if n == v {
                                    self.click.set(Clicks::None);
                                    break 'f true;
                                } else {
                                    self.click.set(Clicks::None);
                                }
                            }
                            Clicks::Down2(v) => {
                                if n == v {
                                    self.click.set(Clicks::None);
                                    break 'f true;
                                } else {
                                    self.click.set(Clicks::None);
                                }
                            }
                            _ => {
                                self.click.set(Clicks::None);
                            }
                        }
                        break 'f false;
                    }
                }
                self.click.set(Clicks::None);
                false
            }
            _ => false,
        }
    }
}

static DOUBLE_CLICK: AtomicU32 = AtomicU32::new(250);

/// Sets the global double click time-out between consecutive clicks.
/// In milliseconds.
pub fn set_double_click_timeout(timeout: u32) {
    DOUBLE_CLICK.store(timeout, Ordering::Release);
}

/// The global double click time-out between consecutive clicks.
/// In milliseconds.
pub fn double_click_timeout() -> u32 {
    DOUBLE_CLICK.load(Ordering::Acquire)
}

static ENHANCED_KEYS: AtomicBool = AtomicBool::new(false);

/// Are enhanced keys available?
/// Only then Release and Repeat keys are available.
///
/// This flag is set during startup of the application when
/// configuring the terminal.
pub fn have_keyboard_enhancement() -> bool {
     ENHANCED_KEYS.load(Ordering::Acquire)
}

/// Set the flag for enhanced keys.
///
/// For windows + crossterm this can always be set true.
///
/// For unix this needs to activate the enhancements with PushKeyboardEnhancementFlags
/// + it still needs to query supports_keyboard_enhancement().
/// If you enable REPORT_ALL_KEYS_AS_ESCAPE_CODES you need REPORT_ALTERNATE_KEYS to,
/// otherwise shift+key will not return something useful.
///
pub fn set_have_keyboard_enhancement(have: bool) {
    ENHANCED_KEYS.store(have, Ordering::Release);
}