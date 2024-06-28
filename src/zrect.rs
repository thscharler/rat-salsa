use ratatui::layout::{Columns, Margin, Offset, Position, Positions, Rect, Rows, Size};

/// Extended Rectangle with a z-order added.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ZRect {
    /// Coordinates
    pub x: u16,
    pub y: u16,
    pub z: u16,

    pub width: u16,
    pub height: u16,
}

impl From<Rect> for ZRect {
    fn from(base_plane: Rect) -> Self {
        Self {
            x: base_plane.x,
            y: base_plane.y,
            z: 0,
            width: base_plane.width,
            height: base_plane.height,
        }
    }
}

impl From<(u16, Rect)> for ZRect {
    #[inline]
    fn from(plane_rect: (u16, Rect)) -> Self {
        Self {
            x: plane_rect.1.x,
            y: plane_rect.1.y,
            z: plane_rect.0,
            width: plane_rect.1.width,
            height: plane_rect.1.height,
        }
    }
}

impl From<(Position, Size)> for ZRect {
    fn from((position, size): (Position, Size)) -> Self {
        Self {
            x: position.x,
            y: position.y,
            z: 0,

            width: size.width,
            height: size.height,
        }
    }
}

impl ZRect {
    pub const ZERO: Self = Self {
        x: 0,
        y: 0,
        z: 0,

        width: 0,
        height: 0,
    };

    #[inline]
    pub const fn new(x: u16, y: u16, width: u16, height: u16, z: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
            z,
        }
    }

    #[inline]
    pub const fn as_rect(self) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
        }
    }

    /// The area of the `Rect`. If the area is larger than the maximum value of `u16`, it will be
    /// clamped to `u16::MAX`.
    #[inline]
    pub const fn area(self) -> u16 {
        self.as_rect().area()
    }

    /// Returns true if the `Rect` has no area.
    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// Returns the left coordinate of the `Rect`.
    pub const fn left(self) -> u16 {
        self.x
    }

    /// Returns the right coordinate of the `Rect`. This is the first coordinate outside of the
    /// `Rect`.
    ///
    /// If the right coordinate is larger than the maximum value of u16, it will be clamped to
    /// `u16::MAX`.
    pub const fn right(self) -> u16 {
        self.x.saturating_add(self.width)
    }

    /// Returns the top coordinate of the `Rect`.
    pub const fn top(self) -> u16 {
        self.y
    }

    /// Returns the bottom coordinate of the `Rect`. This is the first coordinate outside of the
    /// `Rect`.
    ///
    /// If the bottom coordinate is larger than the maximum value of u16, it will be clamped to
    /// `u16::MAX`.
    pub const fn bottom(self) -> u16 {
        self.y.saturating_add(self.height)
    }

    /// Returns a new `Rect` inside the current one, with the given margin on each side.
    ///
    /// If the margin is larger than the `Rect`, the returned `Rect` will have no area.
    #[must_use = "method returns the modified value"]
    #[inline]
    pub fn inner(self, margin: Margin) -> Self {
        ZRect::from((self.z, self.as_rect().inner(margin)))
    }

    /// Moves the `Rect` without modifying its size.
    ///
    /// Moves the `Rect` according to the given offset without modifying its [`width`](Rect::width)
    /// or [`height`](Rect::height).
    /// - Positive `x` moves the whole `Rect` to the right, negative to the left.
    /// - Positive `y` moves the whole `Rect` to the bottom, negative to the top.
    ///
    /// See [`Offset`] for details.
    #[must_use = "method returns the modified value"]
    #[inline]
    pub fn offset(self, offset: Offset) -> Self {
        ZRect::from((self.z, self.as_rect().offset(offset)))
    }

    // TODO: unclear what this means for different z.

    //
    // /// Returns a new `Rect` that contains both the current one and the given one.
    // #[must_use = "method returns the modified value"]
    // pub fn union(self, other: Self) -> Self {
    //     ZRect::from((self.z, self.as_rect().union(other.as_rect())))
    // }
    //
    // /// Returns a new `Rect` that is the intersection of the current one and the given one.
    // ///
    // /// If the two `Rect`s do not intersect, the returned `Rect` will have no area.
    // #[must_use = "method returns the modified value"]
    // pub fn intersection(self, other: Self) -> Self {
    //     ZRect::from((self.z, self.as_rect().intersection(other.as_rect())))
    // }
    //
    // /// Returns true if the two `Rect`s intersect.
    // pub const fn intersects(self, other: Self) -> bool {
    //     self.as_rect().intersects(other.as_rect())
    // }

    /// Returns true if the given position is inside the `Rect`.
    ///
    /// The position is considered inside the `Rect` if it is on the `Rect`'s border.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ratatui::{prelude::*, layout::Position};
    /// let rect = Rect::new(1, 2, 3, 4);
    /// assert!(rect.contains(Position { x: 1, y: 2 }));
    /// ````
    pub const fn contains(self, position: Position) -> bool {
        self.as_rect().contains(position)
    }

    // TODO: unclear what this means for different z.

    // /// Clamp this `Rect` to fit inside the other `Rect`.
    // ///
    // /// If the width or height of this `Rect` is larger than the other `Rect`, it will be clamped to
    // /// the other `Rect`'s width or height.
    // ///
    // /// If the left or top coordinate of this `Rect` is smaller than the other `Rect`, it will be
    // /// clamped to the other `Rect`'s left or top coordinate.
    // ///
    // /// If the right or bottom coordinate of this `Rect` is larger than the other `Rect`, it will be
    // /// clamped to the other `Rect`'s right or bottom coordinate.
    // ///
    // /// This is different from [`Rect::intersection`] because it will move this `Rect` to fit inside
    // /// the other `Rect`, while [`Rect::intersection`] instead would keep this `Rect`'s position and
    // /// truncate its size to only that which is inside the other `Rect`.
    // ///
    // /// # Examples
    // ///
    // /// ```rust
    // /// # use ratatui::prelude::*;
    // /// # fn render(frame: &mut Frame) {
    // /// let area = frame.size();
    // /// let rect = Rect::new(0, 0, 100, 100).clamp(area);
    // /// # }
    // /// ```
    // #[must_use = "method returns the modified value"]
    // pub fn clamp(self, other: Self) -> Self {
    //     ZRect::from((self.z, self.as_rect().clamp(other.as_rect())))
    // }

    /// An iterator over rows within the `Rect`.
    ///
    /// # Example
    ///
    /// ```
    /// # use ratatui::prelude::*;
    /// fn render(area: Rect, buf: &mut Buffer) {
    ///     for row in area.rows() {
    ///         Line::raw("Hello, world!").render(row, buf);
    ///     }
    /// }
    /// ```
    pub const fn rows(self) -> Rows {
        self.as_rect().rows()
    }

    /// An iterator over columns within the `Rect`.
    ///
    /// # Example
    ///
    /// ```
    /// # use ratatui::{prelude::*, widgets::*};
    /// fn render(area: Rect, buf: &mut Buffer) {
    ///     if let Some(left) = area.columns().next() {
    ///         Block::new().borders(Borders::LEFT).render(left, buf);
    ///     }
    /// }
    /// ```
    pub const fn columns(self) -> Columns {
        self.as_rect().columns()
    }

    /// An iterator over the positions within the `Rect`.
    ///
    /// The positions are returned in a row-major order (left-to-right, top-to-bottom).
    ///
    /// # Example
    ///
    /// ```
    /// # use ratatui::prelude::*;
    /// fn render(area: Rect, buf: &mut Buffer) {
    ///     for position in area.positions() {
    ///         buf.get_mut(position.x, position.y).set_symbol("x");
    ///     }
    /// }
    /// ```
    pub const fn positions(self) -> Positions {
        self.as_rect().positions()
    }

    /// Returns a [`Position`] with the same coordinates as this `Rect`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ratatui::prelude::*;
    /// let rect = Rect::new(1, 2, 3, 4);
    /// let position = rect.as_position();
    /// ````
    pub const fn as_position(self) -> Position {
        self.as_rect().as_position()
    }

    /// Converts the `Rect` into a size struct.
    pub const fn as_size(self) -> Size {
        self.as_rect().as_size()
    }
}
