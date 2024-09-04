//!
//! Vertical and horizontal multiple split.
//!

use crate::_private::NonExhaustive;
use crate::util::{fill_buf_area, revert_style};
use rat_event::util::MouseFlagsN;
use rat_event::{ct_event, flow, HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::{FocusFlag, HasFocusFlag, Navigation};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Position, Rect};
use ratatui::prelude::BlockExt;
use ratatui::style::Style;
use ratatui::widgets::{Block, BorderType, StatefulWidget, Widget};
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::{StatefulWidgetRef, WidgetRef};
use std::cmp::{max, min};
use std::mem;
use unicode_segmentation::UnicodeSegmentation;

/// Splits the area in multiple parts and allows changing the sizes.
///
/// This widget doesn't hold a reference to the rendered widgets or such,
/// use [SplitState::areas] to render each part, after rendering the split
/// widget.
///
/// Additionally, [Split] itself can't be rendered but acts as a builder
/// for the actual widgets. Call `into_widgets` to get the actual
/// [SplitWidget] and the [SplitOverlay]. SplitWidget must be rendered
/// first, after that you can access the SplitState::areas to render the
/// parts, and last render SplitOverlay to render the markers that will
/// appear overlaid on the widgets.
#[derive(Debug, Default, Clone)]
pub struct Split<'a> {
    direction: Direction,
    constraints: Vec<Constraint>,

    split_type: SplitType,
    join_0: Option<BorderType>,
    join_1: Option<BorderType>,
    mark_offset: u16,
    mark_0_char: Option<&'a str>,
    mark_1_char: Option<&'a str>,
    block: Option<Block<'a>>,

    style: Style,
    arrow_style: Option<Style>,
    drag_style: Option<Style>,
}

/// Primary widget for rendering the Split.
#[derive(Debug, Default, Clone)]
pub struct SplitWidget<'a> {
    split: Split<'a>,
}

/// Secondary struct for rendering the overlay parts of the Split.
#[derive(Debug, Default, Clone)]
pub struct SplitOverlay<'a> {
    split: Option<Split<'a>>,
}

/// Combined style for the splitter.
#[derive(Debug)]
pub struct SplitStyle {
    /// Base style
    pub style: Style,
    /// Arrow style.
    pub arrow_style: Option<Style>,
    /// Style while dragging.
    pub drag_style: Option<Style>,

    /// Marker for a horizontal split.
    /// Only the first 2 chars are used.
    pub horizontal_mark: Option<&'static str>,
    /// Marker for a vertical split.
    /// Only the first 2 chars are used.
    pub vertical_mark: Option<&'static str>,

    pub non_exhaustive: NonExhaustive,
}

/// Render variants for the splitter.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum SplitType {
    /// Render a full splitter between the widgets. Reduces the area for
    /// each widget. Renders a blank border.
    #[default]
    FullEmpty,
    /// Render a full splitter between the widgets. Reduces the area for
    /// each widget. Renders a plain line border.
    FullPlain,
    /// Render a full splitter between the widgets. Reduces the area for
    /// each widget. Renders a double line border.
    FullDouble,
    /// Render a full splitter between the widgets. Reduces the area for
    /// each widget. Renders a thick line border.
    FullThick,
    /// Render a full splitter between the widgets. Reduces the area for
    /// each widget. Renders a border with a single line on the inside
    /// of a half block.
    FullQuadrantInside,
    /// Render a full splitter between the widgets. Reduces the area for
    /// each widget. Renders a border with a single line on the outside
    /// of a half block.
    FullQuadrantOutside,
    /// Render a minimal splitter, consisting just the two marker chars
    /// rendered over the left/top widget.
    ///
    /// If the left widget has a Scroll in that area this will integrate
    /// nicely. You will have to set `split_mark_offset` with Scroll, then
    /// Scroll can adjust its rendering to leave space for the markers.
    /// And you want to set a `mark_offset` here.
    ///
    /// The widget will get the full area, only the marker is used
    /// for mouse interactions.
    Scroll,
    /// Don't render a splitter, fully manual mode.
    ///
    /// The widget will have the full area, but the event-handling will
    /// use the last column/row of the widget for moving the split.
    /// This can be adjusted if you change `state.split[n]` which provides
    /// the active area.
    Widget,
}

const SPLIT_WIDTH: u16 = 1;

/// State & event handling.
#[derive(Debug)]
#[non_exhaustive]
pub struct SplitState {
    /// Total area.
    /// __readonly__. renewed for each render.
    pub area: Rect,
    /// Area inside the border.
    /// __readonly__. renewed for each render.
    pub inner: Rect,
    /// The widget areas.
    /// Use this after calling layout() to render your widgets.
    /// __readonly__ renewed for each render.
    pub widget_areas: Vec<Rect>,
    /// Area used by the splitter. This is area is used for moving the splitter.
    /// It might overlap with the widget area.
    /// __readonly__ renewed for each render.
    pub splitline_areas: Vec<Rect>,
    /// Start position for drawing the mark.
    /// __readonly__ renewed for each render.
    pub splitline_mark_areas: Vec<Position>,
    /// Offset of the mark from top/left.
    /// __readonly__ renewed for each render.
    pub mark_offset: u16,

    /// Direction of the split.
    /// __readonly__ renewed for each render.
    pub direction: Direction,
    /// __readonly__ renewed for each render.
    pub split_type: SplitType,

    /// Layout-widths for the split-areas.
    ///
    /// This information is used after the initial render to
    /// lay out the splitter.
    /// __read+write__
    pub lengths: Vec<u16>,
    /// Saved lengths for hidden splits.
    hidden: Vec<u16>,

    /// Focus.
    /// __read+write__
    pub focus: FocusFlag,
    /// If the splitter has the focus you can navigate between
    /// the split-markers. This is the currently active split-marker.
    /// __read+write__
    pub focus_marker: Option<usize>,

    /// Mouseflags.
    /// __read+write__
    pub mouse: MouseFlagsN,
}

impl SplitType {
    pub fn is_full(&self) -> bool {
        use SplitType::*;
        match self {
            FullEmpty => true,
            FullPlain => true,
            FullDouble => true,
            FullThick => true,
            FullQuadrantInside => true,
            FullQuadrantOutside => true,
            Scroll => false,
            Widget => false,
        }
    }
}

impl Default for SplitStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            arrow_style: None,
            drag_style: None,
            horizontal_mark: None,
            vertical_mark: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a> Split<'a> {
    pub fn new() -> Self {
        Self {
            direction: Direction::Horizontal,
            ..Default::default()
        }
    }

    /// Set constraints for the initial area sizes.
    /// If the window is resized the current widths are used as
    /// constraints for recalculating.
    ///
    /// The number of constraints determines the number of areas.
    pub fn constraints(mut self, constraints: impl IntoIterator<Item = Constraint>) -> Self {
        self.constraints = constraints.into_iter().collect();
        self
    }

    /// Layout direction of the widgets.
    /// Direction::Horizontal means the widgets are laid out left to right,
    /// with a vertical split area in between.
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Controls rendering of the splitter.
    pub fn split_type(mut self, split_type: SplitType) -> Self {
        self.split_type = split_type;
        self
    }

    /// Draw a join character between the split and the
    /// border on the left/top side. This sets the border type
    /// used for the left/top border.
    pub fn join(mut self, border: BorderType) -> Self {
        self.join_0 = Some(border);
        self.join_1 = Some(border);
        self
    }

    /// Draw a join character between the split and the
    /// border on the left/top side. This sets the border type
    /// used for the left/top border.
    pub fn join_0(mut self, border: BorderType) -> Self {
        self.join_0 = Some(border);
        self
    }

    /// Draw a join character between the split and the
    /// border on the right/bottom side. This sets the border type
    /// used for the right/bottom border.
    pub fn join_1(mut self, border: BorderType) -> Self {
        self.join_1 = Some(border);
        self
    }

    /// Outer block.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set all styles.
    pub fn styles(mut self, styles: SplitStyle) -> Self {
        self.style = styles.style;
        self.drag_style = styles.drag_style;
        self.arrow_style = styles.arrow_style;
        match self.direction {
            Direction::Horizontal => {
                if let Some(mark) = styles.horizontal_mark {
                    let mut g = mark.graphemes(true);
                    if let Some(g0) = g.next() {
                        self.mark_0_char = Some(g0);
                    }
                    if let Some(g1) = g.next() {
                        self.mark_1_char = Some(g1);
                    }
                }
            }
            Direction::Vertical => {
                if let Some(mark) = styles.vertical_mark {
                    let mut g = mark.graphemes(true);
                    if let Some(g0) = g.next() {
                        self.mark_0_char = Some(g0);
                    }
                    if let Some(g1) = g.next() {
                        self.mark_1_char = Some(g1);
                    }
                }
            }
        }
        self
    }

    /// Style for the split area.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Style for the arrows.
    pub fn arrow_style(mut self, style: Style) -> Self {
        self.arrow_style = Some(style);
        self
    }

    /// Style while dragging the splitter.
    pub fn drag_style(mut self, style: Style) -> Self {
        self.drag_style = Some(style);
        self
    }

    /// Offset for the split marker from the top/left.
    pub fn mark_offset(mut self, offset: u16) -> Self {
        self.mark_offset = offset;
        self
    }

    /// First marker char for the splitter.
    pub fn mark_0(mut self, mark: &'a str) -> Self {
        self.mark_0_char = Some(mark);
        self
    }

    /// Second marker char for the splitter.
    pub fn mark_1(mut self, mark: &'a str) -> Self {
        self.mark_1_char = Some(mark);
        self
    }

    /// Constructs the widgets for rendering.
    pub fn into_widgets(self) -> (SplitWidget<'a>, SplitOverlay<'a>) {
        if self.split_type == SplitType::Scroll {
            (
                SplitWidget {
                    split: self.clone(),
                },
                SplitOverlay { split: Some(self) },
            )
        } else {
            (SplitWidget { split: self }, SplitOverlay { split: None })
        }
    }
}

impl<'a> Split<'a> {
    /// Calculates the first layout according to the constraints.
    /// When a resize is detected, the current widths are used as constraints.
    fn layout_split(&self, area: Rect, state: &mut SplitState) {
        state.area = area;
        state.inner = self.block.inner_if_some(area);

        // use only the inner from here on
        let inner = state.inner;

        let layout_change = state.lengths.len() != self.constraints.len();
        let meta_change = state.direction != self.direction
            || state.split_type != self.split_type
            || state.mark_offset != self.mark_offset;

        let old_len = |v: &Rect| {
            // must use the old direction to get a correct value.
            if state.direction == Direction::Horizontal {
                v.width
            } else {
                v.height
            }
        };
        let new_len = |v: &Rect| {
            // must use the old direction to get a correct value.
            if self.direction == Direction::Horizontal {
                v.width
            } else {
                v.height
            }
        };

        let new_split_areas = if layout_change {
            // initial
            let new_areas = Layout::new(self.direction, self.constraints.clone())
                .flex(Flex::Legacy)
                .split(inner);
            Some(new_areas)
        } else {
            let old_length: u16 = state.lengths.iter().sum();
            if meta_change || old_len(&inner) != old_length {
                let mut constraints = Vec::new();
                for i in 0..state.lengths.len() {
                    constraints.push(Constraint::Fill(state.lengths[i]));
                }
                let new_areas = Layout::new(self.direction, constraints).split(inner);
                Some(new_areas)
            } else {
                None
            }
        };

        if let Some(new_split_areas) = new_split_areas {
            state.lengths.clear();
            for v in new_split_areas.iter() {
                state.lengths.push(new_len(v));
            }
        }

        state.direction = self.direction;
        state.split_type = self.split_type;
        state.mark_offset = self.mark_offset;

        self.layout_from_widths(state);
    }

    fn layout_from_widths(&self, state: &mut SplitState) {
        // Areas changed, create areas and splits.
        state.widget_areas.clear();
        state.splitline_areas.clear();
        state.splitline_mark_areas.clear();

        let inner = state.inner;

        let mut total = 0;
        for length in state
            .lengths
            .iter()
            .take(state.lengths.len().saturating_sub(1))
            .copied()
        {
            let mut area = if self.direction == Direction::Horizontal {
                Rect::new(inner.x + total, inner.y, length, inner.height)
            } else {
                Rect::new(inner.x, inner.y + total, inner.width, length)
            };
            let mut split = if self.direction == Direction::Horizontal {
                Rect::new(
                    inner.x + total + length.saturating_sub(SPLIT_WIDTH),
                    inner.y,
                    1,
                    inner.height,
                )
            } else {
                Rect::new(
                    inner.x,
                    inner.y + total + length.saturating_sub(SPLIT_WIDTH),
                    inner.width,
                    1,
                )
            };
            let mut mark = if self.direction == Direction::Horizontal {
                Position::new(
                    inner.x + total + length.saturating_sub(SPLIT_WIDTH),
                    inner.y + self.mark_offset,
                )
            } else {
                Position::new(
                    inner.x + self.mark_offset,
                    inner.y + total + length.saturating_sub(SPLIT_WIDTH),
                )
            };

            adjust_for_split_type(
                self.direction,
                self.split_type,
                &mut area,
                &mut split,
                &mut mark,
            );

            state.widget_areas.push(area);
            state.splitline_areas.push(split);
            state.splitline_mark_areas.push(mark);

            total += length;
        }
        if let Some(length) = state.lengths.last().copied() {
            let area = if self.direction == Direction::Horizontal {
                Rect::new(inner.x + total, inner.y, length, inner.height)
            } else {
                Rect::new(inner.x, inner.y + total, inner.width, length)
            };

            state.widget_areas.push(area);
        }

        // Set 2nd dimension too, if necessary.
        if let Some(test) = state.widget_areas.first() {
            if self.direction == Direction::Horizontal {
                if test.height != state.inner.height {
                    for r in &mut state.widget_areas {
                        r.height = state.inner.height;
                    }
                    for r in &mut state.splitline_areas {
                        r.height = state.inner.height;
                    }
                }
            } else {
                if test.width != state.inner.width {
                    for r in &mut state.widget_areas {
                        r.width = state.inner.width;
                    }
                    for r in &mut state.splitline_areas {
                        r.width = state.inner.width;
                    }
                }
            }
        }
    }
}

/// Adjust area and split according to the split_type.
fn adjust_for_split_type(
    direction: Direction,
    split_type: SplitType,
    area: &mut Rect,
    split: &mut Rect,
    mark: &mut Position,
) {
    use Direction::*;
    use SplitType::*;

    match (direction, split_type) {
        (
            Horizontal,
            FullEmpty | FullPlain | FullDouble | FullThick | FullQuadrantInside
            | FullQuadrantOutside,
        ) => {
            area.width = area.width.saturating_sub(1);
        }
        (
            Vertical,
            FullEmpty | FullPlain | FullDouble | FullThick | FullQuadrantInside
            | FullQuadrantOutside,
        ) => {
            area.height = area.height.saturating_sub(1);
        }

        (Horizontal, Scroll) => {
            split.y = mark.y;
            split.height = 2;
        }
        (Vertical, Scroll) => {
            split.x = mark.x;
            split.width = 2;
        }

        (Horizontal, Widget) => {}
        (Vertical, Widget) => {}
    }
}

impl<'a> Split<'a> {
    fn get_mark_0(&self) -> &str {
        if let Some(mark) = self.mark_0_char {
            mark
        } else if self.direction == Direction::Horizontal {
            "<"
        } else {
            "^"
        }
    }

    fn get_mark_1(&self) -> &str {
        if let Some(mark) = self.mark_1_char {
            mark
        } else if self.direction == Direction::Horizontal {
            ">"
        } else {
            "v"
        }
    }

    fn get_fill_char(&self) -> Option<&str> {
        use Direction::*;
        use SplitType::*;

        match (self.direction, self.split_type) {
            (Horizontal, FullEmpty) => Some(" "),
            (Vertical, FullEmpty) => Some(" "),
            (Horizontal, FullPlain) => Some("\u{2502}"),
            (Vertical, FullPlain) => Some("\u{2500}"),
            (Horizontal, FullDouble) => Some("\u{2551}"),
            (Vertical, FullDouble) => Some("\u{2550}"),
            (Horizontal, FullThick) => Some("\u{2503}"),
            (Vertical, FullThick) => Some("\u{2501}"),
            (Horizontal, FullQuadrantInside) => Some("\u{258C}"),
            (Vertical, FullQuadrantInside) => Some("\u{2580}"),
            (Horizontal, FullQuadrantOutside) => Some("\u{2590}"),
            (Vertical, FullQuadrantOutside) => Some("\u{2584}"),
            (_, Scroll) => None,
            (_, Widget) => None,
        }
    }

    fn get_join_0(&self, split_area: Rect, state: &SplitState) -> Option<(Position, &str)> {
        use BorderType::*;
        use Direction::*;
        use SplitType::*;

        let s: Option<&str> = if let Some(join_0) = self.join_0 {
            match (self.direction, join_0, self.split_type) {
                (
                    Horizontal,
                    Plain | Rounded,
                    FullPlain | FullQuadrantInside | FullQuadrantOutside | FullEmpty | Scroll,
                ) => Some("\u{252C}"),
                (
                    Vertical,
                    Plain | Rounded,
                    FullPlain | FullQuadrantInside | FullQuadrantOutside | FullEmpty | Scroll,
                ) => Some("\u{251C}"),
                (Horizontal, Plain | Rounded | Thick, FullDouble) => Some("\u{2565}"),
                (Vertical, Plain | Rounded | Thick, FullDouble) => Some("\u{255E}"),
                (Horizontal, Plain | Rounded, FullThick) => Some("\u{2530}"),
                (Vertical, Plain | Rounded, FullThick) => Some("\u{251D}"),

                (
                    Horizontal,
                    Double,
                    FullPlain | FullThick | FullQuadrantInside | FullQuadrantOutside | FullEmpty
                    | Scroll,
                ) => Some("\u{2564}"),
                (
                    Vertical,
                    Double,
                    FullPlain | FullThick | FullQuadrantInside | FullQuadrantOutside | FullEmpty
                    | Scroll,
                ) => Some("\u{255F}"),
                (Horizontal, Double, FullDouble) => Some("\u{2566}"),
                (Vertical, Double, FullDouble) => Some("\u{2560}"),

                (
                    Horizontal,
                    Thick,
                    FullPlain | FullQuadrantInside | FullQuadrantOutside | FullEmpty | Scroll,
                ) => Some("\u{252F}"),
                (
                    Vertical,
                    Thick,
                    FullPlain | FullQuadrantInside | FullQuadrantOutside | FullEmpty | Scroll,
                ) => Some("\u{2520}"),
                (Horizontal, Thick, FullThick) => Some("\u{2533}"),
                (Vertical, Thick, FullThick) => Some("\u{2523}"),

                (Horizontal, QuadrantOutside, FullEmpty) => Some("\u{2588}"),
                (Vertical, QuadrantOutside, FullEmpty) => Some("\u{2588}"),

                (_, QuadrantInside, _) => None,
                (_, QuadrantOutside, _) => None,

                (_, _, Widget) => None,
            }
        } else {
            None
        };

        if let Some(s) = s {
            Some((
                match self.direction {
                    Horizontal => Position::new(split_area.x, state.area.y),
                    Vertical => Position::new(state.area.x, split_area.y),
                },
                s,
            ))
        } else {
            None
        }
    }

    fn get_join_1(&self, split_area: Rect, state: &SplitState) -> Option<(Position, &str)> {
        use BorderType::*;
        use Direction::*;
        use SplitType::*;

        let s: Option<&str> = if let Some(join_1) = self.join_1 {
            match (self.direction, join_1, self.split_type) {
                (
                    Horizontal,
                    Plain | Rounded,
                    FullPlain | FullQuadrantInside | FullQuadrantOutside | FullEmpty | Scroll,
                ) => Some("\u{2534}"),
                (
                    Vertical,
                    Plain | Rounded,
                    FullPlain | FullQuadrantInside | FullQuadrantOutside | FullEmpty | Scroll,
                ) => Some("\u{2524}"),
                (Horizontal, Plain | Rounded | Thick, FullDouble) => Some("\u{2568}"),
                (Vertical, Plain | Rounded | Thick, FullDouble) => Some("\u{2561}"),
                (Horizontal, Plain | Rounded, FullThick) => Some("\u{2538}"),
                (Vertical, Plain | Rounded, FullThick) => Some("\u{2525}"),

                (
                    Horizontal,
                    Double,
                    FullPlain | FullThick | FullQuadrantInside | FullQuadrantOutside | FullEmpty
                    | Scroll,
                ) => Some("\u{2567}"),
                (
                    Vertical,
                    Double,
                    FullPlain | FullThick | FullQuadrantInside | FullQuadrantOutside | FullEmpty
                    | Scroll,
                ) => Some("\u{2562}"),
                (Horizontal, Double, FullDouble) => Some("\u{2569}"),
                (Vertical, Double, FullDouble) => Some("\u{2563}"),

                (
                    Horizontal,
                    Thick,
                    FullPlain | FullQuadrantInside | FullQuadrantOutside | FullEmpty | Scroll,
                ) => Some("\u{2537}"),
                (
                    Vertical,
                    Thick,
                    FullPlain | FullQuadrantInside | FullQuadrantOutside | FullEmpty | Scroll,
                ) => Some("\u{2528}"),
                (Horizontal, Thick, FullThick) => Some("\u{253B}"),
                (Vertical, Thick, FullThick) => Some("\u{252B}"),

                (Horizontal, QuadrantOutside, FullEmpty) => Some("\u{2588}"),
                (Vertical, QuadrantOutside, FullEmpty) => Some("\u{2588}"),

                (_, QuadrantInside, _) => None,
                (_, QuadrantOutside, _) => None,

                (_, _, Widget) => None,
            }
        } else {
            None
        };

        if let Some(s) = s {
            Some((
                match self.direction {
                    Horizontal => Position::new(split_area.x, state.area.y + state.area.height - 1),
                    Vertical => Position::new(state.area.x + state.area.width - 1, split_area.y),
                },
                s,
            ))
        } else {
            None
        }
    }
}

#[cfg(feature = "unstable-widget-ref")]
impl<'a> StatefulWidgetRef for SplitWidget<'a> {
    type State = SplitState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_widget(
            &self.split,
            |area, buf| self.split.block.render_ref(area, buf),
            area,
            buf,
            state,
        );
        if !matches!(self.split.split_type, SplitType::Widget | SplitType::Scroll) {
            render_split(&self.split, buf, state);
        }
    }
}

impl<'a> StatefulWidget for SplitWidget<'a> {
    type State = SplitState;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = mem::take(&mut self.split.block);
        render_widget(
            &self.split,
            |area, buf| block.render(area, buf),
            area,
            buf,
            state,
        );
        if !matches!(self.split.split_type, SplitType::Widget | SplitType::Scroll) {
            render_split(&self.split, buf, state);
        }
    }
}

#[cfg(feature = "unstable-widget-ref")]
impl<'a> StatefulWidgetRef for SplitOverlay<'a> {
    type State = SplitState;

    fn render_ref(&self, _area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // rely on layout already happened.
        if let Some(split) = &self.split {
            if matches!(split.split_type, SplitType::Scroll) {
                render_split(split, buf, state);
            }
        }
    }
}

impl<'a> StatefulWidget for SplitOverlay<'a> {
    type State = SplitState;

    fn render(self, _area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // rely on layout already happened.
        if let Some(split) = &self.split {
            if matches!(split.split_type, SplitType::Scroll) {
                render_split(split, buf, state);
            }
        }
    }
}

fn render_widget(
    split: &Split<'_>,
    block: impl FnOnce(Rect, &mut Buffer),
    area: Rect,
    buf: &mut Buffer,
    state: &mut SplitState,
) {
    split.layout_split(area, state);

    if state.is_focused() {
        if state.focus_marker.is_none() {
            state.focus_marker = Some(0);
        }
    } else {
        state.focus_marker = None;
    }

    block(area, buf);
}

fn render_split(split: &Split<'_>, buf: &mut Buffer, state: &mut SplitState) {
    for (n, split_area) in state.splitline_areas.iter().enumerate() {
        let arrow_style = if state.mouse.hover.get() == Some(n) {
            if let Some(drag) = split.drag_style {
                drag
            } else {
                revert_style(split.style)
            }
        } else {
            if let Some(arrow) = split.arrow_style {
                arrow
            } else {
                split.style
            }
        };

        let (style, arrow_style) =
            if Some(n) == state.mouse.drag.get() || Some(n) == state.focus_marker {
                if let Some(drag) = split.drag_style {
                    (drag, drag)
                } else {
                    (revert_style(split.style), arrow_style)
                }
            } else {
                (split.style, arrow_style)
            };

        if let Some(fill) = split.get_fill_char() {
            fill_buf_area(buf, *split_area, fill, style);
        }

        let mark = state.splitline_mark_areas[n];
        if split.direction == Direction::Horizontal {
            if buf.area.contains((mark.x, mark.y).into()) {
                if let Some(cell) = buf.cell_mut((mark.x, mark.y)) {
                    cell.set_style(arrow_style);
                    cell.set_symbol(split.get_mark_0());
                }
            }
            if buf.area.contains((mark.x, mark.y + 1).into()) {
                if let Some(cell) = buf.cell_mut((mark.x, mark.y + 1)) {
                    cell.set_style(arrow_style);
                    cell.set_symbol(split.get_mark_1());
                }
            }
        } else {
            if let Some(cell) = buf.cell_mut((mark.x, mark.y)) {
                cell.set_style(arrow_style);
                cell.set_symbol(split.get_mark_0());
            }
            if let Some(cell) = buf.cell_mut((mark.x + 1, mark.y)) {
                cell.set_style(arrow_style);
                cell.set_symbol(split.get_mark_1());
            }
        }

        if let Some((pos_0, c_0)) = split.get_join_0(*split_area, state) {
            if let Some(cell) = buf.cell_mut((pos_0.x, pos_0.y)) {
                cell.set_symbol(c_0);
            }
        }
        if let Some((pos_1, c_1)) = split.get_join_1(*split_area, state) {
            if let Some(cell) = buf.cell_mut((pos_1.x, pos_1.y)) {
                cell.set_symbol(c_1);
            }
        }
    }
}

impl Default for SplitState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            inner: Default::default(),
            focus: Default::default(),
            focus_marker: Default::default(),
            widget_areas: Default::default(),
            splitline_areas: Default::default(),
            splitline_mark_areas: Default::default(),
            direction: Default::default(),
            split_type: Default::default(),
            mark_offset: 0,
            mouse: Default::default(),
            lengths: vec![],
            hidden: vec![],
        }
    }
}

impl HasFocusFlag for SplitState {
    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        // not mouse focusable
        Rect::default()
    }

    fn navigable(&self) -> Navigation {
        Navigation::Leave
    }
}

impl SplitState {
    ///
    pub fn new() -> Self {
        Self::default()
    }

    /// New state with a focus-name.
    pub fn named(name: &str) -> Self {
        Self {
            focus: FocusFlag::named(name),
            ..Self::default()
        }
    }

    /// Set the position for the nth splitter.
    ///
    /// The position is limited the combined area of the two adjacent areas.
    pub fn set_screen_split_pos(&mut self, n: usize, pos: (u16, u16)) -> bool {
        use SplitType::*;

        let area1 = self.widget_areas[n];
        let area2 = self.widget_areas[n + 1];

        if self.direction == Direction::Horizontal {
            let min_pos = area1.x;
            let mut max_pos = if matches!(self.split_type, Scroll | Widget) {
                area2.right().saturating_sub(2)
            } else {
                area2.right().saturating_sub(1)
            };
            if n + 2 == self.widget_areas.len() {
                max_pos += 1;
            }

            let pos_x = min(max(pos.0, min_pos), max_pos);

            self.lengths[n] = pos_x - area1.x + 1;
            self.lengths[n + 1] = area2.right().saturating_sub(pos_x + 1);
        } else {
            let min_pos = area1.y;
            let max_pos = if matches!(self.split_type, Scroll | Widget) {
                area2.bottom().saturating_sub(2)
            } else {
                area2.bottom().saturating_sub(1)
            };

            let pos_y = min(max(pos.1, min_pos), max_pos);

            self.lengths[n] = pos_y - area1.y + 1;
            self.lengths[n + 1] = area2.bottom().saturating_sub(pos_y + 1);
        }

        true
    }

    /// Move the nth split position.
    /// Does nothing if the change is bigger than the length of the split.
    pub fn move_split_left(&mut self, n: usize, delta: u16) -> bool {
        if self.lengths[n] > delta {
            self.lengths[n] -= delta;
            self.lengths[n + 1] += delta;
            true
        } else {
            false
        }
    }

    /// Move the nth split position.
    /// Does nothing if the change is bigger than the length of the split.
    pub fn move_split_right(&mut self, n: usize, delta: u16) -> bool {
        if self.lengths[n + 1] >= delta {
            self.lengths[n] += delta;
            self.lengths[n + 1] -= delta;
            true
        } else {
            false
        }
    }

    /// Move the nth split position.
    /// Does nothing if the change is bigger than the length of the split.
    pub fn move_split_up(&mut self, n: usize, delta: u16) -> bool {
        self.move_split_left(n, delta)
    }

    /// Move the nth split position.
    /// Does nothing if the change is bigger than the length of the split.
    pub fn move_split_down(&mut self, n: usize, delta: u16) -> bool {
        self.move_split_right(n, delta)
    }

    /// Select the next splitter for manual adjustment.
    pub fn select_next_split(&mut self) -> bool {
        if self.is_focused() {
            let n = self.focus_marker.unwrap_or_default();
            if n + 1 >= self.lengths.len() {
                self.focus_marker = Some(0);
            } else {
                self.focus_marker = Some(n + 1);
            }
            true
        } else {
            false
        }
    }

    /// Select the previous splitter for manual adjustment.
    pub fn select_prev_split(&mut self) -> bool {
        if self.is_focused() {
            let n = self.focus_marker.unwrap_or_default();
            if n == 0 {
                self.focus_marker = Some(self.lengths.len() - 1);
            } else {
                self.focus_marker = Some(n - 1);
            }
            true
        } else {
            false
        }
    }

    /// Is the split hidden?
    pub fn is_hidden(&self, n: usize) -> bool {
        if n < self.hidden.len() {
            self.hidden[n] > 0
        } else {
            false
        }
    }

    /// Hide the split and adds its area to the following split.
    pub fn hide_split(&mut self, n: usize) -> bool {
        while self.hidden.len() < self.lengths.len() {
            self.hidden.push(0);
        }

        if self.hidden[n] == 0 {
            let mut hide = self.lengths[n].saturating_sub(1);
            for idx in n + 1..self.lengths.len() {
                if self.hidden[idx] == 0 {
                    self.lengths[idx] += hide;
                    hide = 0;
                    break;
                }
            }
            if hide > 0 {
                for idx in (n - 1..0).rev() {
                    if self.hidden[idx] == 0 {
                        self.lengths[idx] += hide;
                        hide = 0;
                        break;
                    }
                }
            }

            if hide > 0 {
                // don't hide last.
                self.hidden[n] = 0;
            } else {
                self.hidden[n] = self.lengths[n].saturating_sub(1);
                self.lengths[n] = 1;
            }
            true
        } else {
            false
        }
    }

    // Show a hidden split.
    pub fn show_split(&mut self, n: usize) -> bool {
        while self.hidden.len() < self.lengths.len() {
            self.hidden.push(0);
        }

        let mut show = self.hidden[n];
        if show > 0 {
            for idx in n + 1..self.lengths.len() {
                if self.hidden[idx] == 0 {
                    // steal as much as we can
                    if self.lengths[idx] > show + 1 {
                        self.lengths[idx] -= show;
                        show = 0;
                    } else if self.lengths[idx] > 1 {
                        show -= self.lengths[idx] - 1;
                        self.lengths[idx] = 1;
                    }
                    if show == 0 {
                        break;
                    }
                }
            }
            if show > 0 {
                for idx in (n - 1..0).rev() {
                    if self.hidden[idx] == 0 {
                        if self.lengths[idx] > show + 1 {
                            self.lengths[idx] -= show;
                            show = 0;
                        } else if self.lengths[idx] > 1 {
                            show -= self.lengths[idx] - 1;
                            self.lengths[idx] = 1;
                        }
                        if show == 0 {
                            break;
                        }
                    }
                }
            }

            if show > 0 {
                self.lengths[n] += self.hidden[n] - show;
                self.hidden[n] = 0;
            } else {
                self.lengths[n] += self.hidden[n];
                self.hidden[n] = 0;
            }
            true
        } else {
            false
        }
    }
}

impl HandleEvent<crossterm::event::Event, Regular, Outcome> for SplitState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> Outcome {
        flow!(if self.is_focused() {
            if let Some(n) = self.focus_marker {
                match event {
                    ct_event!(keycode press Left) => self.move_split_left(n, 1).into(),
                    ct_event!(keycode press Right) => self.move_split_right(n, 1).into(),
                    ct_event!(keycode press Up) => self.move_split_up(n, 1).into(),
                    ct_event!(keycode press Down) => self.move_split_down(n, 1).into(),

                    ct_event!(keycode press CONTROL-Left) => self.select_next_split().into(),
                    ct_event!(keycode press CONTROL-Right) => self.select_prev_split().into(),
                    ct_event!(keycode press CONTROL-Up) => self.select_next_split().into(),
                    ct_event!(keycode press CONTROL-Down) => self.select_prev_split().into(),
                    _ => Outcome::Continue,
                }
            } else {
                Outcome::Continue
            }
        } else {
            Outcome::Continue
        });

        self.handle(event, MouseOnly)
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for SplitState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> Outcome {
        match event {
            ct_event!(mouse any for m) if self.mouse.hover(&self.splitline_areas, m) => {
                Outcome::Changed
            }
            ct_event!(mouse any for m) => {
                let was_drag = self.mouse.drag.get();
                if self.mouse.drag(&self.splitline_areas, m) {
                    if let Some(n) = self.mouse.drag.get() {
                        self.set_screen_split_pos(n, self.mouse.pos_of(m)).into()
                    } else {
                        Outcome::Continue
                    }
                } else {
                    // repaint after drag is finished. resets the displayed style.
                    if was_drag.is_some() {
                        Outcome::Changed
                    } else {
                        Outcome::Continue
                    }
                }
            }
            _ => Outcome::Continue,
        }
    }
}
