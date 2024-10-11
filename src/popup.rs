use crate::Placement;
use crate::_private::NonExhaustive;
use crate::event::PopupOutcome;
use rat_event::util::MouseFlags;
use rat_event::{ct_event, HandleEvent, Regular};
use rat_focus::{ContainerFlag, HasFocus, ZRect};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{block::BlockExt, Block, StatefulWidget, Widget};
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::{StatefulWidgetRef, WidgetRef};
use std::cmp::max;

/// Render a popup widget.
///
/// This does all the calculations and renders the border.
/// The actual content is rendered by the user.
/// Use PopupState::widget_area for this.
///
#[derive(Debug, Clone)]
pub struct Popup<'a> {
    placement: Placement,
    offset: (i16, i16),
    boundary_area: Option<Rect>,

    block: Option<Block<'a>>,
}

#[derive(Debug, Clone)]
pub struct PopupState {
    /// Total area.
    /// __read only__. renewed for each render.
    pub area: Rect,
    /// Area with z-indes for Focus.
    /// __read only__. renewed for each render.
    pub z_areas: [ZRect; 1],
    /// Inner area for the popup content.
    /// Use this to render the content.
    pub widget_area: Rect,

    /// Container flag.
    /// You will need to provide your own FocusFlag for the widget.
    ///
    /// This container-flag indicates if the popup is visible/hidden.
    /// If you combine this container-flag and your widgets focus-flag
    /// when implementing HasFocus, the popup will be hidden whenever
    /// it looses focus.
    pub container: ContainerFlag,
    /// Mouse flags.
    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> Default for Popup<'a> {
    fn default() -> Self {
        Self {
            placement: Placement::None,
            offset: (0, 0),
            boundary_area: None,
            block: None,
        }
    }
}

impl<'a> Popup<'a> {
    /// new
    pub fn new() -> Self {
        Self::default()
    }

    /// Places the popup relative to the 'relative' area.
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    /// Gives the widget an extra offset
    pub fn offset(mut self, offset: (i16, i16)) -> Self {
        self.offset = offset;
        self
    }

    /// Gives the widget an extra x-offset
    pub fn x_offset(mut self, offset: i16) -> Self {
        self.offset.0 = offset;
        self
    }

    /// Gives the widget an extra y-offset
    pub fn y_offset(mut self, offset: i16) -> Self {
        self.offset.1 = offset;
        self
    }

    /// Sets outer bounds for the popup.
    /// If not set it uses Buffer::area.
    pub fn boundary(mut self, boundary: Rect) -> Self {
        self.boundary_area = Some(boundary);
        self
    }

    /// Block for borders.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

#[cfg(feature = "unstable-widget-ref")]
impl<'a> StatefulWidgetRef for Popup<'a> {
    type State = PopupState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if !state.container.is_container_focused() {
            state.clear_areas();
            return;
        }

        self.layout(area, self.boundary_area.unwrap_or(buf.area), state);

        self.block.render_ref(state.area, buf);
    }
}

impl<'a> StatefulWidget for Popup<'a> {
    type State = PopupState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if !state.container.is_container_focused() {
            state.clear_areas();
            return;
        }

        self.layout(area, self.boundary_area.unwrap_or(buf.area), state);

        self.block.render(state.area, buf);
    }
}

impl<'a> Popup<'a> {
    fn layout(&self, area: Rect, boundary_area: Rect, state: &mut PopupState) {
        fn center(len: u16, within: u16) -> u16 {
            ((within as i32 - len as i32) / 2).clamp(0, i16::MAX as i32) as u16
        }
        let middle = center;
        fn right(len: u16, within: u16) -> u16 {
            within.saturating_sub(len)
        }
        let bottom = right;

        let mut area = match self.placement {
            Placement::None => area,
            Placement::AboveLeft(rel) => Rect::new(
                rel.x.saturating_add_signed(self.offset.0),
                rel.y.saturating_sub(area.height),
                area.width,
                area.height,
            ),
            Placement::AboveCenter(rel) => Rect::new(
                (rel.x + center(area.width, rel.width)).saturating_add_signed(self.offset.0),
                rel.y.saturating_sub(area.height),
                area.width,
                area.height,
            ),
            Placement::AboveRight(rel) => Rect::new(
                (rel.x + right(area.width, rel.width)).saturating_add_signed(self.offset.0),
                rel.y.saturating_sub(area.height),
                area.width,
                area.height,
            ),
            Placement::BelowLeft(rel) => Rect::new(
                rel.x.saturating_add_signed(self.offset.0),
                rel.bottom(),
                area.width,
                area.height,
            ),
            Placement::BelowCenter(rel) => Rect::new(
                (rel.x + center(area.width, rel.width)).saturating_add_signed(self.offset.0),
                rel.bottom(),
                area.width,
                area.height,
            ),
            Placement::BelowRight(rel) => Rect::new(
                (rel.x + right(area.width, rel.width)).saturating_add_signed(self.offset.0),
                rel.bottom(),
                area.width,
                area.height,
            ),

            Placement::LeftTop(rel) => Rect::new(
                rel.x.saturating_sub(area.width),
                rel.y.saturating_add_signed(self.offset.1),
                area.width,
                area.height,
            ),
            Placement::LeftMiddle(rel) => Rect::new(
                rel.x.saturating_sub(area.width),
                (rel.y + middle(area.height, rel.height)).saturating_add_signed(self.offset.1),
                area.width,
                area.height,
            ),
            Placement::LeftBottom(rel) => Rect::new(
                rel.x.saturating_sub(area.width),
                (rel.y + bottom(area.height, rel.height)).saturating_add_signed(self.offset.1),
                area.width,
                area.height,
            ),
            Placement::RightTop(rel) => Rect::new(
                rel.right(),
                rel.y.saturating_add_signed(self.offset.1),
                area.width,
                area.height,
            ),
            Placement::RightMiddle(rel) => Rect::new(
                rel.right(),
                (rel.y + middle(area.height, rel.height)).saturating_add_signed(self.offset.1),
                area.width,
                area.height,
            ),
            Placement::RightBottom(rel) => Rect::new(
                rel.right(),
                (rel.y + bottom(area.height, rel.height)).saturating_add_signed(self.offset.1),
                area.width,
                area.height,
            ),

            Placement::Position(x, y) => Rect::new(
                x.saturating_add_signed(self.offset.0),
                y.saturating_add_signed(self.offset.1),
                area.width,
                area.height,
            ),
        };

        // keep in sight
        if area.left() < boundary_area.left() {
            let corr = boundary_area.left().saturating_sub(area.left());
            area.x += corr;
        }
        if area.right() >= boundary_area.right() {
            let corr = area.right().saturating_sub(boundary_area.right());
            area.x = area.x.saturating_sub(corr);
        }
        if area.top() < boundary_area.top() {
            let corr = boundary_area.top().saturating_sub(area.top());
            area.y += corr;
        }
        if area.bottom() >= boundary_area.bottom() {
            let corr = area.bottom().saturating_sub(boundary_area.bottom());
            area.y = area.y.saturating_sub(corr);
        }

        state.area = area;
        state.widget_area = self.block.inner_if_some(area);
        state.z_areas[0] = ZRect::from((1, area));
    }
}

impl Default for PopupState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            z_areas: [Default::default()],
            widget_area: Default::default(),
            container: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl PopupState {
    /// New
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// New with a focus name.
    pub fn named(name: &str) -> Self {
        Self {
            container: ContainerFlag::named(name),
            ..Default::default()
        }
    }

    /// Show the popup.
    pub fn is_active(&self) -> bool {
        self.container.is_container_focused()
    }

    /// Show the popup.
    pub fn flip_active(&mut self) {
        self.container.set(!self.container.get());
    }

    /// Show the popup.
    pub fn set_active(&mut self, active: bool) {
        self.container.set(active);
    }

    /// Clear the areas.
    pub fn clear_areas(&mut self) {
        self.area = Default::default();
        self.widget_area = Default::default();
        self.z_areas = Default::default();
    }
}

impl HandleEvent<crossterm::event::Event, Regular, PopupOutcome> for PopupState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> PopupOutcome {
        let r0 = if self.container.container_lost_focus() {
            self.clear_areas();
            PopupOutcome::HiddenFocus
        } else {
            PopupOutcome::Continue
        };

        let r1 = if self.container.is_container_focused() {
            match event {
                ct_event!(mouse down Left for x,y)
                | ct_event!(mouse down Right for x,y)
                | ct_event!(mouse down Middle for x,y)
                    if !self.area.contains((*x, *y).into()) =>
                {
                    self.container.set(false);
                    self.clear_areas();
                    PopupOutcome::Hidden
                }
                _ => PopupOutcome::Continue,
            }
        } else {
            PopupOutcome::Continue
        };

        max(r0, r1)
    }
}
