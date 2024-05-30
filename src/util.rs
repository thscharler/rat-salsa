//!
//! Some utility functions that pop up all the time.
//!

use crate::ConsumedEvent;
use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Position, Rect};
use std::cmp::{max, min};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

/// Which row of the given contains the position.
/// This uses only the vertical components of the given areas.
///
/// You might want to limit calling this functions when the full
/// position is inside your target rect.
pub fn row_at_clicked(areas: &[Rect], y_pos: u16) -> Option<usize> {
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
pub fn column_at_clicked(areas: &[Rect], x_pos: u16) -> Option<usize> {
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
    if let Some(row) = row_at_clicked(areas, y_pos) {
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

/// Column when dragging. Can go outside the area.
pub fn column_at_drag(encompassing: Rect, areas: &[Rect], x_pos: u16) -> Result<usize, isize> {
    if let Some(column) = column_at_clicked(areas, x_pos) {
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

/// A baseline Outcome for event-handling.
///
/// A widget can define its own, if it has more things to report.
/// It would be nice of the widget though, if its outcome would be
/// convertible to this baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Outcome {
    /// The given event has not been used at all.
    NotUsed,
    /// The event has been recognized, but the result was nil.
    /// Further processing for this event may stop.
    Unchanged,
    /// The event has been recognized and there is some change
    /// due to it.
    /// Further processing for this event may stop.
    /// Rendering the ui is advised.
    Changed,
}

impl ConsumedEvent for Outcome {
    fn is_consumed(&self) -> bool {
        *self != Outcome::NotUsed
    }
}

impl BitOr for Outcome {
    type Output = Outcome;

    fn bitor(self, rhs: Self) -> Self::Output {
        max(self, rhs)
    }
}

impl BitAnd for Outcome {
    type Output = Outcome;

    fn bitand(self, rhs: Self) -> Self::Output {
        min(self, rhs)
    }
}

impl BitOrAssign for Outcome {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.bitor(rhs);
    }
}

impl BitAndAssign for Outcome {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = self.bitand(rhs);
    }
}

/// Some state for mouse interactions.
///
/// This helps with double-click and mouse drag recognition.
/// Add this to your widget state.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MouseFlags {
    /// Flag for the first down.
    pub click: bool,
    /// Flag for the first up.
    pub clack: bool,
    /// Drag enabled.
    pub drag: bool,
    /// Drag is not a good indicator for a valid drag pos.
    /// An option would do, but that's annoying.
    pub some_drag_pos: bool,
    /// Last drag pos, if drag is false this is reset to (0,0).
    pub drag_pos: (u16, u16),
}

impl MouseFlags {
    /// Returns the last drag-position if drag is active.
    pub fn drag_pos(&self) -> (u16, u16) {
        self.drag_pos
    }

    /// Checks if this is a drag event for the widget.
    ///
    /// It makes sense to allow drag events outside the given area, if the
    /// drag has been started with a click to the given area.
    ///
    /// This function handles that case.
    pub fn drag(&mut self, area: Rect, event: &MouseEvent) -> bool {
        self.drag2(area, event, KeyModifiers::NONE)
    }

    /// Checks if this is a drag event for the widget.
    ///
    /// It makes sense to allow drag events outside the given area, if the
    /// drag has been started with a click to the given area.
    ///
    /// This function handles that case.
    pub fn drag2(&mut self, area: Rect, event: &MouseEvent, filter: KeyModifiers) -> bool {
        match event {
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                modifiers,
            } if *modifiers == filter => {
                if area.contains(Position::new(*column, *row)) {
                    self.drag = true;
                } else {
                    self.some_drag_pos = false;
                    self.drag_pos = (0, 0);
                    self.drag = false;
                }
            }
            MouseEvent {
                kind: MouseEventKind::Drag(MouseButton::Left),
                column,
                row,
                modifiers,
            } if *modifiers == filter => {
                if self.drag {
                    self.some_drag_pos = true;
                    self.drag_pos = (*column, *row);
                    return true;
                }
            }
            MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left) | MouseEventKind::Moved,
                ..
            } => {
                self.some_drag_pos = false;
                self.drag_pos = (0, 0);
                self.drag = false;
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
    pub fn doubleclick(&mut self, area: Rect, event: &MouseEvent) -> bool {
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
    pub fn doubleclick2(&mut self, area: Rect, event: &MouseEvent, filter: KeyModifiers) -> bool {
        match event {
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                modifiers,
            } if *modifiers == filter => {
                if area.contains(Position::new(*column, *row)) {
                    self.click = true;
                    self.clack = false;
                } else {
                    self.click = false;
                    self.clack = false;
                }
            }
            MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left),
                column,
                row,
                modifiers,
            } if *modifiers == filter => {
                if area.contains(Position::new(*column, *row)) {
                    if self.click {
                        if !self.clack {
                            self.clack = true;
                        } else {
                            return true;
                        }
                    }
                } else {
                    self.click = false;
                    self.clack = false;
                }
            }
            _ => {}
        }
        false
    }
}
