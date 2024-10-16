use crate::Placement;
use crate::_private::NonExhaustive;
use crate::event::PopupOutcome;
use rat_event::util::MouseFlags;
use rat_event::{ct_event, HandleEvent, Popup};
use rat_focus::{ContainerFlag, IsFocusContainer, ZRect};
use ratatui::buffer::Buffer;
use ratatui::layout::{Rect, Size};
use ratatui::style::Style;
use ratatui::widgets::{block::BlockExt, Block, StatefulWidget, Widget};
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::{StatefulWidgetRef, WidgetRef};
use std::cmp::max;

/// Provides the core for popup widgets.
///
/// This does widget can calculate the placement of a popup widget
/// using the [placement](PopupCore::placement), [offset](PopupCore::offset)
/// and the outer [boundary](PopupCore::boundary).
///
/// It provides the widget area as [widget_area](PopupCoreState::widget_area).
/// It's up to the user to render the actual content for the popup.
///
/// ## Event handling
///
/// The widget will detect any suspicious mouse activity outside its bounds
/// and returns [PopupOutcome::Hide] if it finds such.
///
/// The widget doesn't change its active/visible state by itself,
/// it's up to the caller to do this.
///
/// __See__
/// See the examples some variants.
///
#[derive(Debug, Clone)]
pub struct PopupCore<'a> {
    style: Style,

    placement: Placement,
    offset: (i16, i16),
    boundary_area: Option<Rect>,

    block: Option<Block<'a>>,
}

#[derive(Debug, Clone)]
pub struct PopupCoreState {
    /// Total area for the widget.
    /// This is the area given to render().
    /// __read only__. renewed for each render.
    pub area: Rect,
    /// Total area with a z-index of 1 for Focus.
    /// This is necessary for Focus to handle overlapping regions.
    /// __read only__. renewed for each render.
    pub z_areas: [ZRect; 1],
    /// Inner area for the popup content.
    /// Use this to render the content.
    /// __read only. renewed for each render.
    pub widget_area: Rect,

    /// Active flag for the popup.
    ///
    /// Uses a ContainerFlag that can be combined with the FocusFlags
    /// your widget uses for handling its focus to detect the
    /// transition 'Did the popup loose focus and should it be closed now'.
    ///
    /// If you don't rely on Focus this way, this will just be a boolean
    /// flag that indicates active/visible.
    ///
    /// __See__
    /// See the examples how to use for both cases.
    pub active: ContainerFlag,

    /// Mouse flags.
    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> Default for PopupCore<'a> {
    fn default() -> Self {
        Self {
            style: Default::default(),
            placement: Placement::None,
            offset: (0, 0),
            boundary_area: None,
            block: None,
        }
    }
}

impl<'a> PopupCore<'a> {
    /// New.
    pub fn new() -> Self {
        Self::default()
    }

    /// Placement of the popup widget.
    /// See placement for the options.
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
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

    /// Sets outer boundaries for the resulting widget.
    ///
    /// This will be used to ensure that the widget is fully visible,
    /// after calculation its position using the other parameters.
    ///
    /// If not set it will use [Buffer::area] for this.
    pub fn boundary(mut self, boundary: Rect) -> Self {
        self.boundary_area = Some(boundary);
        self
    }

    /// Base style for the popup.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Block for borders.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Return the size required for the block.
    pub fn get_block_size(&self) -> Size {
        let area = Rect::new(0, 0, 100, 100);
        let inner = self.block.as_ref().map_or(area, |v| v.inner(area));
        Size::new(area.width - inner.width, area.height - inner.height)
    }
}

#[cfg(feature = "unstable-widget-ref")]
impl<'a> StatefulWidgetRef for PopupCore<'a> {
    type State = PopupCoreState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if !state.active.is_container_focused() {
            state.clear_areas();
            return;
        }

        self.layout(area, self.boundary_area.unwrap_or(buf.area), state);

        for y in state.area.top()..state.area.bottom() {
            for x in state.area.left()..state.area.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.reset();
                    cell.set_style(self.style);
                }
            }
        }

        if let Some(block) = self.block.as_ref() {
            block.clone().style(self.style).render_ref(state.area, buf);
        }
    }
}

impl<'a> StatefulWidget for PopupCore<'a> {
    type State = PopupCoreState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if !state.active.is_container_focused() {
            state.clear_areas();
            return;
        }

        self.layout(area, self.boundary_area.unwrap_or(buf.area), state);

        for y in state.area.top()..state.area.bottom() {
            for x in state.area.left()..state.area.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.reset();
                }
            }
        }

        if let Some(block) = self.block {
            block.style(self.style).render(state.area, buf);
        }
    }
}

impl<'a> PopupCore<'a> {
    fn layout(&self, area: Rect, boundary_area: Rect, state: &mut PopupCoreState) {
        // helper fn
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
                rel.x,
                rel.y.saturating_sub(area.height),
                area.width,
                area.height,
            ),
            Placement::AboveCenter(rel) => Rect::new(
                rel.x + center(area.width, rel.width),
                rel.y.saturating_sub(area.height),
                area.width,
                area.height,
            ),
            Placement::AboveRight(rel) => Rect::new(
                rel.x + right(area.width, rel.width),
                rel.y.saturating_sub(area.height),
                area.width,
                area.height,
            ),
            Placement::BelowLeft(rel) => Rect::new(rel.x, rel.bottom(), area.width, area.height),
            Placement::BelowCenter(rel) => Rect::new(
                rel.x + center(area.width, rel.width),
                rel.bottom(),
                area.width,
                area.height,
            ),
            Placement::BelowRight(rel) => Rect::new(
                rel.x + right(area.width, rel.width),
                rel.bottom(),
                area.width,
                area.height,
            ),

            Placement::LeftTop(rel) => Rect::new(
                rel.x.saturating_sub(area.width),
                rel.y,
                area.width,
                area.height,
            ),
            Placement::LeftMiddle(rel) => Rect::new(
                rel.x.saturating_sub(area.width),
                rel.y + middle(area.height, rel.height),
                area.width,
                area.height,
            ),
            Placement::LeftBottom(rel) => Rect::new(
                rel.x.saturating_sub(area.width),
                rel.y + bottom(area.height, rel.height),
                area.width,
                area.height,
            ),
            Placement::RightTop(rel) => Rect::new(rel.right(), rel.y, area.width, area.height),
            Placement::RightMiddle(rel) => Rect::new(
                rel.right(),
                rel.y + middle(area.height, rel.height),
                area.width,
                area.height,
            ),
            Placement::RightBottom(rel) => Rect::new(
                rel.right(),
                rel.y + bottom(area.height, rel.height),
                area.width,
                area.height,
            ),

            Placement::Position(x, y) => Rect::new(x, y, area.width, area.height),
        };

        // offset
        area.x = area.x.saturating_add_signed(self.offset.0);
        area.y = area.y.saturating_add_signed(self.offset.1);

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

        // shrink to size
        if area.right() > boundary_area.right() {
            let corr = area.right() - boundary_area.right();
            area.width = area.width.saturating_sub(corr);
        }
        if area.bottom() > boundary_area.bottom() {
            let corr = area.bottom() - boundary_area.bottom();
            area.height = area.height.saturating_sub(corr);
        }

        state.area = area;
        state.widget_area = self.block.inner_if_some(area);
        state.z_areas[0] = ZRect::from((1, area));
    }
}

impl Default for PopupCoreState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            z_areas: [Default::default()],
            widget_area: Default::default(),
            active: ContainerFlag::named("popup"),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl PopupCoreState {
    /// New
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// New with a focus name.
    pub fn named(name: &str) -> Self {
        Self {
            active: ContainerFlag::named(name),
            ..Default::default()
        }
    }

    /// Is the popup active/visible.
    pub fn is_active(&self) -> bool {
        self.active.is_container_focused()
    }

    /// Flip visibility of the popup.
    pub fn flip_active(&mut self) {
        self.set_active(!self.active.get());
    }

    /// Show the popup.
    ///
    /// If the popup is hidden this will clear all the areas.
    pub fn set_active(&mut self, active: bool) {
        self.active.set(active);
        if !active {
            // reset all extra flags too.
            self.active.clear();
            self.clear_areas();
        }
    }

    /// Clear the areas.
    pub fn clear_areas(&mut self) {
        self.area = Default::default();
        self.widget_area = Default::default();
        self.z_areas = Default::default();
    }
}

impl HandleEvent<crossterm::event::Event, Popup, PopupOutcome> for PopupCoreState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Popup) -> PopupOutcome {
        // this only works out if the active flag is actually used
        // as a container flag. but that's fine.
        let r0 = if self.active.container_lost_focus() {
            PopupOutcome::Hide
        } else {
            PopupOutcome::Continue
        };

        let r1 = if self.is_active() {
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
        };

        max(r0, r1)
    }
}
