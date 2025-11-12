use crate::_private::NonExhaustive;
use crate::event::PopupOutcome;
use crate::{Placement, PopupConstraint};
use rat_event::util::MouseFlags;
use rat_event::{HandleEvent, Popup, ct_event};
use rat_reloc::RelocatableState;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::StatefulWidget;
use std::cell::Cell;
use std::cmp::max;
use std::rc::Rc;

/// Provides the core for popup widgets.
///
/// This widget can calculate the placement of a popup widget
/// using [placement](PopupCore::constraint), [offset](PopupCore::offset)
/// and the outer [boundary](PopupCore::boundary).
///
/// It provides the widget area as [area](PopupCoreState::area).
///
/// After rendering the PopupCore the main widget can render it's
/// content in the calculated [PopupCoreState::area].
///
/// ## Event handling
///
/// Will detect any mouse-clicks outside its area and
/// return [PopupOutcome::Hide]. Actually showing/hiding the popup is
/// the job of the main widget.
///
/// __See__
/// See the examples some variants.
///
#[derive(Debug, Clone)]
pub struct PopupCore {
    /// Constraints for the popup.
    pub constraint: Cell<PopupConstraint>,
    /// Extra offset after calculating the position
    /// with constraint.
    pub offset: (i16, i16),
    /// Outer boundary for the popup-placement.
    /// If not set uses the buffer-area.
    pub boundary_area: Option<Rect>,

    pub non_exhaustive: NonExhaustive,
}

/// Complete styles for the popup.
#[derive(Debug, Clone)]
pub struct PopupStyle {
    /// Extra offset added after applying the constraints.
    pub offset: Option<(i16, i16)>,
    /// Alignment.
    pub alignment: Option<Alignment>,
    /// Placement
    pub placement: Option<Placement>,

    /// non-exhaustive struct.
    pub non_exhaustive: NonExhaustive,
}

/// State for the PopupCore.
#[derive(Debug)]
pub struct PopupCoreState {
    /// Area for the widget.
    /// This is the area given to render(), corrected by the
    /// given constraints.
    /// __read only__. renewed for each render.
    pub area: Rect,
    /// Z-Index for the popup.
    pub area_z: u16,

    /// Active flag for the popup.
    ///
    /// __read+write__
    pub active: Rc<Cell<bool>>,

    /// Mouse flags.
    /// __read+write__
    pub mouse: MouseFlags,

    /// non-exhaustive struct.
    pub non_exhaustive: NonExhaustive,
}

impl Default for PopupCore {
    fn default() -> Self {
        Self {
            constraint: Cell::new(PopupConstraint::None),
            offset: (0, 0),
            boundary_area: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl PopupCore {
    /// New.
    pub fn new() -> Self {
        Self::default()
    }

    /// Placement constraints for the popup widget.
    pub fn ref_constraint(&self, constraint: PopupConstraint) -> &Self {
        self.constraint.set(constraint);
        self
    }

    /// Placement constraints for the popup widget.
    pub fn constraint(self, constraint: PopupConstraint) -> Self {
        self.constraint.set(constraint);
        self
    }

    /// Adds an extra offset to the widget area.
    ///
    /// This can be used to
    /// * place the widget under the mouse cursor.
    /// * align the widget not by the outer bounds but by
    ///   the text content.
    pub fn offset(mut self, offset: (i16, i16)) -> Self {
        self.offset = offset;
        self
    }

    /// Sets only the x offset.
    /// See [offset](Self::offset)
    pub fn x_offset(mut self, offset: i16) -> Self {
        self.offset.0 = offset;
        self
    }

    /// Sets only the y offset.
    /// See [offset](Self::offset)
    pub fn y_offset(mut self, offset: i16) -> Self {
        self.offset.1 = offset;
        self
    }

    /// Sets outer boundaries for the popup widget.
    ///
    /// This will be used to ensure that the popup widget is fully visible.
    /// First it tries to move the popup in a way that is fully inside
    /// this area. If this is not enought the popup area will be clipped.
    ///
    /// If this is not set, [Buffer::area] will be used instead.
    pub fn boundary(mut self, boundary: Rect) -> Self {
        self.boundary_area = Some(boundary);
        self
    }

    /// Set styles
    pub fn styles(mut self, styles: PopupStyle) -> Self {
        if let Some(offset) = styles.offset {
            self.offset = offset;
        }

        self
    }

    /// Run the layout to calculate the popup area before rendering.
    pub fn layout(&self, area: Rect, buf: &Buffer) -> Rect {
        self._layout(area, self.boundary_area.unwrap_or(buf.area))
    }
}

impl StatefulWidget for &PopupCore {
    type State = PopupCoreState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_popup(self, area, buf, state);
    }
}

impl StatefulWidget for PopupCore {
    type State = PopupCoreState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_popup(&self, area, buf, state);
    }
}

fn render_popup(widget: &PopupCore, area: Rect, buf: &mut Buffer, state: &mut PopupCoreState) {
    if !state.active.get() {
        state.clear_areas();
        return;
    }

    state.area = widget._layout(area, widget.boundary_area.unwrap_or(buf.area));

    reset_buf_area(state.area, buf);
}

/// Fallback for popup style.
pub fn fallback_popup_style(style: Style) -> Style {
    if style.fg.is_some() || style.bg.is_some() {
        style
    } else {
        style.black().on_gray()
    }
}

/// Reset an area of the buffer.
pub fn reset_buf_area(area: Rect, buf: &mut Buffer) {
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.reset();
            }
        }
    }
}

impl PopupCore {
    fn _layout(&self, area: Rect, boundary_area: Rect) -> Rect {
        // helper fn
        fn center(len: u16, within: u16) -> u16 {
            ((within as i32 - len as i32) / 2).clamp(0, i16::MAX as i32) as u16
        }
        let middle = center;
        fn right(len: u16, within: u16) -> u16 {
            within.saturating_sub(len)
        }
        let bottom = right;

        // offsets may change
        let mut offset = self.offset;

        let mut area = match self.constraint.get() {
            PopupConstraint::None => area,
            PopupConstraint::Above(Alignment::Left, rel) => Rect::new(
                rel.x,
                rel.y.saturating_sub(area.height),
                area.width,
                area.height,
            ),
            PopupConstraint::Above(Alignment::Center, rel) => Rect::new(
                rel.x + center(area.width, rel.width),
                rel.y.saturating_sub(area.height),
                area.width,
                area.height,
            ),
            PopupConstraint::Above(Alignment::Right, rel) => Rect::new(
                rel.x + right(area.width, rel.width),
                rel.y.saturating_sub(area.height),
                area.width,
                area.height,
            ),
            PopupConstraint::Below(Alignment::Left, rel) => Rect::new(
                rel.x, //
                rel.bottom(),
                area.width,
                area.height,
            ),
            PopupConstraint::Below(Alignment::Center, rel) => Rect::new(
                rel.x + center(area.width, rel.width),
                rel.bottom(),
                area.width,
                area.height,
            ),
            PopupConstraint::Below(Alignment::Right, rel) => Rect::new(
                rel.x + right(area.width, rel.width),
                rel.bottom(),
                area.width,
                area.height,
            ),

            PopupConstraint::Left(Alignment::Left, rel) => Rect::new(
                rel.x.saturating_sub(area.width),
                rel.y,
                area.width,
                area.height,
            ),
            PopupConstraint::Left(Alignment::Center, rel) => Rect::new(
                rel.x.saturating_sub(area.width),
                rel.y + middle(area.height, rel.height),
                area.width,
                area.height,
            ),
            PopupConstraint::Left(Alignment::Right, rel) => Rect::new(
                rel.x.saturating_sub(area.width),
                rel.y + bottom(area.height, rel.height),
                area.width,
                area.height,
            ),
            PopupConstraint::Right(Alignment::Left, rel) => Rect::new(
                rel.right(), //
                rel.y,
                area.width,
                area.height,
            ),
            PopupConstraint::Right(Alignment::Center, rel) => Rect::new(
                rel.right(),
                rel.y + middle(area.height, rel.height),
                area.width,
                area.height,
            ),
            PopupConstraint::Right(Alignment::Right, rel) => Rect::new(
                rel.right(),
                rel.y + bottom(area.height, rel.height),
                area.width,
                area.height,
            ),

            PopupConstraint::Position(x, y) => Rect::new(
                x, //
                y,
                area.width,
                area.height,
            ),

            PopupConstraint::AboveOrBelow(Alignment::Left, rel) => {
                if area.height.saturating_add_signed(-self.offset.1) < rel.y {
                    Rect::new(
                        rel.x,
                        rel.y.saturating_sub(area.height),
                        area.width,
                        area.height,
                    )
                } else {
                    offset = (offset.0, -offset.1);
                    Rect::new(
                        rel.x, //
                        rel.bottom(),
                        area.width,
                        area.height,
                    )
                }
            }
            PopupConstraint::AboveOrBelow(Alignment::Center, rel) => {
                if area.height.saturating_add_signed(-self.offset.1) < rel.y {
                    Rect::new(
                        rel.x + center(area.width, rel.width),
                        rel.y.saturating_sub(area.height),
                        area.width,
                        area.height,
                    )
                } else {
                    offset = (offset.0, -offset.1);
                    Rect::new(
                        rel.x + center(area.width, rel.width), //
                        rel.bottom(),
                        area.width,
                        area.height,
                    )
                }
            }
            PopupConstraint::AboveOrBelow(Alignment::Right, rel) => {
                if area.height.saturating_add_signed(-self.offset.1) < rel.y {
                    Rect::new(
                        rel.x + right(area.width, rel.width),
                        rel.y.saturating_sub(area.height),
                        area.width,
                        area.height,
                    )
                } else {
                    offset = (offset.0, -offset.1);
                    Rect::new(
                        rel.x + right(area.width, rel.width), //
                        rel.bottom(),
                        area.width,
                        area.height,
                    )
                }
            }
            PopupConstraint::BelowOrAbove(Alignment::Left, rel) => {
                if (rel.bottom() + area.height).saturating_add_signed(self.offset.1)
                    <= boundary_area.height
                {
                    Rect::new(
                        rel.x, //
                        rel.bottom(),
                        area.width,
                        area.height,
                    )
                } else {
                    offset = (offset.0, -offset.1);
                    Rect::new(
                        rel.x,
                        rel.y.saturating_sub(area.height),
                        area.width,
                        area.height,
                    )
                }
            }
            PopupConstraint::BelowOrAbove(Alignment::Center, rel) => {
                if (rel.bottom() + area.height).saturating_add_signed(self.offset.1)
                    <= boundary_area.height
                {
                    Rect::new(
                        rel.x + center(area.width, rel.width), //
                        rel.bottom(),
                        area.width,
                        area.height,
                    )
                } else {
                    offset = (offset.0, -offset.1);
                    Rect::new(
                        rel.x + center(area.width, rel.width),
                        rel.y.saturating_sub(area.height),
                        area.width,
                        area.height,
                    )
                }
            }
            PopupConstraint::BelowOrAbove(Alignment::Right, rel) => {
                if (rel.bottom() + area.height).saturating_add_signed(self.offset.1)
                    <= boundary_area.height
                {
                    Rect::new(
                        rel.x + right(area.width, rel.width), //
                        rel.bottom(),
                        area.width,
                        area.height,
                    )
                } else {
                    offset = (offset.0, -offset.1);
                    Rect::new(
                        rel.x + right(area.width, rel.width),
                        rel.y.saturating_sub(area.height),
                        area.width,
                        area.height,
                    )
                }
            }
        };

        // offset
        area.x = area.x.saturating_add_signed(offset.0);
        area.y = area.y.saturating_add_signed(offset.1);

        // keep in sight
        if area.left() < boundary_area.left() {
            area.x = boundary_area.left();
        }
        if area.right() >= boundary_area.right() {
            let corr = area.right().saturating_sub(boundary_area.right());
            area.x = max(boundary_area.left(), area.x.saturating_sub(corr));
        }
        if area.top() < boundary_area.top() {
            area.y = boundary_area.top();
        }
        if area.bottom() >= boundary_area.bottom() {
            let corr = area.bottom().saturating_sub(boundary_area.bottom());
            area.y = max(boundary_area.top(), area.y.saturating_sub(corr));
        }

        // shrink to size
        if area.right() > boundary_area.right() {
            let corr = area.right() - boundary_area.right();
            area.width = area.width.saturating_sub(corr);
        }
        if area.bottom() > boundary_area.bottom() {
            let corr = area.bottom() - boundary_area.bottom();
            area.height = area.height.saturating_sub(corr);
        }

        area
    }
}

impl Default for PopupStyle {
    fn default() -> Self {
        Self {
            offset: None,
            alignment: None,
            placement: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl Clone for PopupCoreState {
    fn clone(&self) -> Self {
        Self {
            area: self.area,
            area_z: self.area_z,
            active: self.active.clone(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl Default for PopupCoreState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            area_z: 1,
            active: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl RelocatableState for PopupCoreState {
    fn relocate(&mut self, _shift: (i16, i16), _clip: Rect) {}

    fn relocate_popup(&mut self, shift: (i16, i16), clip: Rect) {
        self.area.relocate(shift, clip);
    }
}

impl PopupCoreState {
    /// New
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Is the popup active/visible.
    pub fn is_active(&self) -> bool {
        self.active.get()
    }

    /// Flip visibility of the popup.
    pub fn flip_active(&mut self) {
        self.set_active(!self.is_active());
    }

    /// Show the popup.
    /// This will set gained/lost flags according to the change.
    /// If the popup is hidden this will clear all flags.
    pub fn set_active(&mut self, active: bool) -> bool {
        let old_value = self.is_active();
        self.active.set(active);
        old_value != self.is_active()
    }

    /// Clear all stored areas.
    pub fn clear_areas(&mut self) {
        self.area = Default::default();
    }
}

impl HandleEvent<crossterm::event::Event, Popup, PopupOutcome> for PopupCoreState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Popup) -> PopupOutcome {
        if self.is_active() {
            match event {
                ct_event!(mouse down Left for x,y)
                | ct_event!(mouse down Right for x,y)
                | ct_event!(mouse down Middle for x,y)
                    if !self.area.contains((*x, *y).into()) =>
                {
                    PopupOutcome::Hide
                }
                _ => PopupOutcome::Continue,
            }
        } else {
            PopupOutcome::Continue
        }
    }
}
