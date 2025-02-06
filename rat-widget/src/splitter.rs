//!
//! Vertical and horizontal multiple split.
//!

use crate::_private::NonExhaustive;
use crate::util::{fill_buf_area, revert_style};
use rat_event::util::MouseFlagsN;
use rat_event::{ct_event, flow, HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_reloc::{relocate_area, relocate_areas, relocate_positions, RelocatableState};
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

#[derive(Debug, Default, Clone)]
/// Splits the area in multiple parts and renders an UI that
/// allows changing the sizes.
///
/// * Can hide/show regions.
/// * Resize
///     * neighbours only
///     * all regions
/// * Horizontal / vertical split. Choose one.
///
/// The widget doesn't hold references to the content widgets,
/// instead it gives back the regions where the content can be
/// rendered.
///
/// 1. Construct the Split.
/// 2. Call [Split::into_widget_layout] to create the actual
///    widget and the layout of the regions.
/// 3. Render the content areas.
/// 4. Render the split widget last. There are options that
///    will render above the widgets and use part of the content
///    area.
///
pub struct Split<'a> {
    // direction
    direction: Direction,
    // start constraints. used when
    // there are no widths in the state.
    constraints: Vec<Constraint>,
    // resize options
    resize: SplitResize,

    // split rendering
    split_type: SplitType,
    // joiner left/top
    join_0: Option<BorderType>,
    // joiner right/bottom
    join_1: Option<BorderType>,
    // offset from left/top for the split-mark
    mark_offset: u16,
    // mark char 1
    mark_0_char: Option<&'a str>,
    // mark char 2
    mark_1_char: Option<&'a str>,

    // styling
    block: Option<Block<'a>>,
    style: Style,
    arrow_style: Option<Style>,
    drag_style: Option<Style>,
}

/// Primary widget for rendering the Split.
#[derive(Debug, Clone)]
pub struct SplitWidget<'a> {
    split: Split<'a>,
    // internal mode:
    // 0 - legacy
    // 1 - used for into_widget_layout()
    mode: u8,
}

/// Secondary struct for rendering the overlay parts of the Split.
#[derive(Debug, Clone)]
#[deprecated(since = "1.0.2", note = "no longer needed")]
pub struct SplitOverlay<'a> {
    split: Option<Split<'a>>,
}

///
/// Combined styles for the Split.
///
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

    /// Block
    pub block: Option<Block<'static>>,

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
    /// nicely. You will have to set `start_margin` with Scroll, then
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

/// Strategy for resizing the split-areas.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SplitResize {
    /// When changing a split-position limit resizing to the two
    /// adjacent neighbours of the split.
    Neighbours,
    /// When changing a split-position, allow all positions in the
    /// widgets area. Minus the minimum space required to draw the
    /// split itself.
    #[default]
    Full,
}

const SPLIT_WIDTH: u16 = 1;

/// State & event handling.
#[derive(Debug)]
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
    pub splitline_mark_position: Vec<Position>,
    /// Offset of the mark from top/left.
    /// __readonly__ renewed for each render.
    pub mark_offset: u16,

    /// Direction of the split.
    /// __readonly__ renewed for each render.
    pub direction: Direction,
    /// __readonly__ renewed for each render.
    pub split_type: SplitType,
    /// __readonly__ renewed for each render.
    pub resize: SplitResize,

    /// Layout-widths for the split-areas.
    ///
    /// This information is used after the initial render to
    /// lay out the splitter.
    area_length: Vec<u16>,
    /// Saved lengths for hidden splits.
    hidden_length: Vec<u16>,

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

    pub non_exhaustive: NonExhaustive,
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
            block: None,
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

    pub fn horizontal() -> Self {
        Self {
            direction: Direction::Horizontal,
            ..Default::default()
        }
    }

    pub fn vertical() -> Self {
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

    /// Controls resizing the split areas.
    pub fn resize(mut self, resize: SplitResize) -> Self {
        self.resize = resize;
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
        if styles.drag_style.is_some() {
            self.drag_style = styles.drag_style;
        }
        if styles.arrow_style.is_some() {
            self.arrow_style = styles.arrow_style;
        }
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
        if styles.block.is_some() {
            self.block = styles.block;
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
    ///
    /// Returns the SplitWidget that actually renders the split.
    /// Returns a Vec<Rect> with the regions for each split.
    ///
    /// Render your content first, using the layout information.
    /// And the SplitWidget as last to allow rendering over
    /// the content widgets.
    pub fn into_widget_layout(
        self,
        area: Rect,
        state: &mut SplitState,
    ) -> (SplitWidget<'a>, Vec<Rect>) {
        self.layout_split(area, state);

        (
            SplitWidget {
                split: self,
                mode: 1,
            },
            state.widget_areas.clone(),
        )
    }

    /// Constructs the widgets for rendering.
    #[deprecated(
        since = "1.0.3",
        note = "use into_widget_layout(). it has a simpler contract."
    )]
    #[allow(deprecated)]
    pub fn into_widgets(self) -> (SplitWidget<'a>, SplitOverlay<'a>) {
        if self.split_type == SplitType::Scroll {
            (
                SplitWidget {
                    split: self.clone(),
                    mode: 0,
                },
                SplitOverlay { split: Some(self) },
            )
        } else {
            (
                SplitWidget {
                    split: self,
                    mode: 0,
                },
                SplitOverlay { split: None },
            )
        }
    }
}

impl Split<'_> {
    /// Calculates the first layout according to the constraints.
    /// When a resize is detected, the current widths are used as constraints.
    fn layout_split(&self, area: Rect, state: &mut SplitState) {
        state.area = area;
        state.inner = self.block.inner_if_some(area);

        // use only the inner from here on
        let inner = state.inner;

        let layout_change = state.area_length.len() != self.constraints.len();
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
            let old_length: u16 = state.area_length.iter().sum();
            if meta_change || old_len(&inner) != old_length {
                let mut constraints = Vec::new();
                for i in 0..state.area_length.len() {
                    constraints.push(Constraint::Fill(state.area_length[i]));
                }
                let new_areas = Layout::new(self.direction, constraints).split(inner);
                Some(new_areas)
            } else {
                None
            }
        };

        if let Some(new_split_areas) = new_split_areas {
            state.area_length.clear();
            for v in new_split_areas.iter() {
                state.area_length.push(new_len(v));
            }
            while state.hidden_length.len() < state.area_length.len() {
                state.hidden_length.push(0);
            }
            while state.hidden_length.len() > state.area_length.len() {
                state.hidden_length.pop();
            }
        }

        state.direction = self.direction;
        state.split_type = self.split_type;
        state.resize = self.resize;
        state.mark_offset = self.mark_offset;

        self.layout_from_widths(state);
    }

    fn layout_from_widths(&self, state: &mut SplitState) {
        // Areas changed, create areas and splits.
        state.widget_areas.clear();
        state.splitline_areas.clear();
        state.splitline_mark_position.clear();

        let inner = state.inner;

        let mut total = 0;
        for length in state
            .area_length
            .iter()
            .take(state.area_length.len().saturating_sub(1))
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
                    min(1, length),
                    inner.height,
                )
            } else {
                Rect::new(
                    inner.x,
                    inner.y + total + length.saturating_sub(SPLIT_WIDTH),
                    inner.width,
                    min(1, length),
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
            state.splitline_mark_position.push(mark);

            total += length;
        }
        if let Some(length) = state.area_length.last().copied() {
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
            area.width = area.width.saturating_sub(SPLIT_WIDTH);
        }
        (
            Vertical,
            FullEmpty | FullPlain | FullDouble | FullThick | FullQuadrantInside
            | FullQuadrantOutside,
        ) => {
            area.height = area.height.saturating_sub(SPLIT_WIDTH);
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

impl Split<'_> {
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

        s.map(|s| {
            (
                match self.direction {
                    Horizontal => Position::new(split_area.x, state.area.y),
                    Vertical => Position::new(state.area.x, split_area.y),
                },
                s,
            )
        })
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

        s.map(|s| {
            (
                match self.direction {
                    Horizontal => Position::new(split_area.x, state.area.y + state.area.height - 1),
                    Vertical => Position::new(state.area.x + state.area.width - 1, split_area.y),
                },
                s,
            )
        })
    }
}

#[cfg(feature = "unstable-widget-ref")]
impl<'a> StatefulWidgetRef for SplitWidget<'a> {
    type State = SplitState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if self.mode == 0 {
            self.split.layout_split(area, state);
        } else if self.mode == 1 {
            // done before
        } else {
            unreachable!()
        }

        if state.is_focused() {
            if state.focus_marker.is_none() {
                state.focus_marker = Some(0);
            }
        } else {
            state.focus_marker = None;
        }

        if self.mode == 0 {
            if self.split.block.is_some() {
                self.split.block.render_ref(area, buf);
            } else {
                buf.set_style(area, self.split.style);
            }
        } else if self.mode == 1 {
            if let Some(mut block) = self.split.block.clone() {
                block = block.style(Style::default());
                block.render_ref(area, buf);
            }
        } else {
            unreachable!()
        }

        if self.mode == 0 {
            if !matches!(self.split.split_type, SplitType::Widget | SplitType::Scroll) {
                render_split(&self.split, buf, state);
            }
        } else if self.mode == 1 {
            render_split(&self.split, buf, state);
        } else {
            unreachable!()
        }
    }
}

impl StatefulWidget for SplitWidget<'_> {
    type State = SplitState;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if self.mode == 0 {
            self.split.layout_split(area, state);
        } else if self.mode == 1 {
            // done before
        } else {
            unreachable!()
        }

        if state.is_focused() {
            if state.focus_marker.is_none() {
                state.focus_marker = Some(0);
            }
        } else {
            state.focus_marker = None;
        }

        if self.mode == 0 {
            if self.split.block.is_some() {
                self.split.block.render(area, buf);
            } else {
                buf.set_style(area, self.split.style);
            }
        } else if self.mode == 1 {
            if let Some(mut block) = mem::take(&mut self.split.block) {
                // can't have the block overwrite all content styles.
                block = block.style(Style::default());
                block.render(area, buf);
            }
        } else {
            unreachable!()
        }

        if self.mode == 0 {
            if !matches!(self.split.split_type, SplitType::Widget | SplitType::Scroll) {
                render_split(&self.split, buf, state);
            }
        } else if self.mode == 1 {
            render_split(&self.split, buf, state);
        } else {
            unreachable!()
        }
    }
}

#[cfg(feature = "unstable-widget-ref")]
#[allow(deprecated)]
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

#[allow(deprecated)]
impl StatefulWidget for SplitOverlay<'_> {
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

fn render_split(split: &Split<'_>, buf: &mut Buffer, state: &mut SplitState) {
    for (n, split_area) in state.splitline_areas.iter().enumerate() {
        // skip 0 width/height
        if split.direction == Direction::Horizontal {
            if split_area.width == 0 {
                continue;
            }
        } else {
            if split_area.height == 0 {
                continue;
            }
        }

        let (style, arrow_style) = if Some(n) == state.mouse.drag.get()
            || Some(n) == state.focus_marker
            || Some(n) == state.mouse.hover.get()
        {
            if let Some(drag) = split.drag_style {
                (drag, drag)
            } else {
                (revert_style(split.style), revert_style(split.style))
            }
        } else {
            if let Some(arrow) = split.arrow_style {
                (split.style, arrow)
            } else {
                (split.style, split.style)
            }
        };

        if let Some(fill) = split.get_fill_char() {
            fill_buf_area(buf, *split_area, fill, style);
        }

        let mark = state.splitline_mark_position[n];
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
            widget_areas: Default::default(),
            splitline_areas: Default::default(),
            splitline_mark_position: Default::default(),
            mark_offset: Default::default(),
            direction: Default::default(),
            split_type: Default::default(),
            resize: Default::default(),
            area_length: Default::default(),
            hidden_length: Default::default(),
            focus: Default::default(),
            focus_marker: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl Clone for SplitState {
    fn clone(&self) -> Self {
        Self {
            area: self.area,
            inner: self.inner,
            widget_areas: self.widget_areas.clone(),
            splitline_areas: self.splitline_areas.clone(),
            splitline_mark_position: self.splitline_mark_position.clone(),
            mark_offset: self.mark_offset,
            direction: self.direction,
            split_type: self.split_type,
            resize: self.resize,
            area_length: self.area_length.clone(),
            hidden_length: self.hidden_length.clone(),
            focus: FocusFlag::named(self.focus.name()),
            focus_marker: self.focus_marker,
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl HasFocus for SplitState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

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

impl RelocatableState for SplitState {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.area = relocate_area(self.area, shift, clip);
        self.inner = relocate_area(self.inner, shift, clip);
        relocate_areas(self.widget_areas.as_mut_slice(), shift, clip);
        relocate_areas(self.splitline_areas.as_mut_slice(), shift, clip);
        relocate_positions(self.splitline_mark_position.as_mut_slice(), shift, clip);
    }
}

#[allow(clippy::len_without_is_empty)]
impl SplitState {
    /// New state.
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
    /// The position is further limited to leave space for rendering the
    /// splitter.
    ///
    pub fn set_screen_split_pos(&mut self, n: usize, pos: (u16, u16)) -> bool {
        if self.is_hidden(n) {
            return false;
        }
        if self.direction == Direction::Horizontal {
            let pos = if pos.0 < self.inner.left() {
                0
            } else if pos.0 < self.inner.right() {
                pos.0 - self.inner.x
            } else {
                self.inner.width
            };

            let split_pos = self.split_pos(n);
            self.set_split_pos(n, pos);

            split_pos != self.split_pos(n)
        } else {
            let pos = if pos.1 < self.inner.top() {
                0
            } else if pos.1 < self.inner.bottom() {
                pos.1 - self.inner.y
            } else {
                self.inner.height
            };

            let split_pos = self.split_pos(n);
            self.set_split_pos(n, pos);

            split_pos != self.split_pos(n)
        }
    }

    /// Move the nth split position.
    /// If delta is greater than the area length it sets the
    /// length to 0.
    pub fn move_split_left(&mut self, n: usize, delta: u16) -> bool {
        let split_pos = self.split_pos(n);
        self.set_split_pos(n, split_pos - delta);

        split_pos != self.split_pos(n)
    }

    /// Move the nth split position.
    /// Does nothing if the change is bigger than the length of the split.
    pub fn move_split_right(&mut self, n: usize, delta: u16) -> bool {
        let split_pos = self.split_pos(n);
        self.set_split_pos(n, split_pos + delta);

        split_pos != self.split_pos(n)
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
            if n + 1 >= self.area_length.len().saturating_sub(1) {
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
                self.focus_marker =
                    Some(self.area_length.len().saturating_sub(1).saturating_sub(1));
            } else {
                self.focus_marker = Some(n - 1);
            }
            true
        } else {
            false
        }
    }

    /// Number of split-areas
    pub fn len(&self) -> usize {
        self.area_length.len()
    }

    /// Get all area lengths.
    pub fn area_lengths(&self) -> &[u16] {
        &self.area_length
    }

    /// Set all area lengths.
    ///
    /// This will adjust the list of the hidden splits too.
    ///
    /// __Caution__
    /// If the sum of the lengths doesn't match the display-width
    /// this will trigger a layout and will use the given lenghts
    /// as Constraint::Fill().
    ///
    /// __Caution__
    ///
    /// If a length is 0 it will not display the split at all.
    pub fn set_area_lengths(&mut self, lengths: Vec<u16>) {
        self.area_length = lengths;
        while self.hidden_length.len() < self.area_length.len() {
            self.hidden_length.push(0);
        }
        while self.hidden_length.len() > self.area_length.len() {
            self.hidden_length.pop();
        }
    }

    /// Get the value of the hidden lengths.
    pub fn hidden_lengths(&self) -> &[u16] {
        &self.hidden_length
    }

    /// Set the value of the hidden lengths.
    ///
    /// This will take at most area_length.len() items of this Vec.
    /// And it will fill missing items as 0.
    pub fn set_hidden_lengths(&mut self, hidden: Vec<u16>) {
        for i in 0..self.hidden_length.len() {
            if let Some(v) = hidden.get(i) {
                self.hidden_length[i] = *v;
            } else {
                self.hidden_length[i] = 0;
            }
        }
    }

    /// Length of the nth split.
    ///
    /// __Caution__
    ///
    /// This length **includes** the width of the split itself.
    /// Which may or may not take some space. Except for the last.
    /// So it will be better to use `widget_areas` for anything
    /// rendering related.
    ///
    pub fn area_len(&self, n: usize) -> u16 {
        self.area_length[n]
    }

    /// Sum of all area lengths.
    pub fn total_area_len(&self) -> u16 {
        self.area_length.iter().sum()
    }

    /// Set the length of the nth split.
    ///
    /// This resets any hidden state of the nth split.
    ///
    /// __Caution__
    /// The sum of all lengths must be equal with the width/height of
    /// the splitter. If it is not this operation doesn't set the
    /// absolute width of the nth split. Instead, it triggers a layout
    /// of the widget, takes all the lengths as Constraint::Fill()
    /// values and redistributes the size.
    ///
    /// You can either ensure to change some other len to accommodate
    /// for your changes. Or use [set_split_pos](Self::set_split_pos) or
    /// [set_screen_split_pos](Self::set_screen_split_pos)
    ///
    /// __Caution__
    ///
    /// This length **includes** the width of the split itself.
    /// Which may or may not take some space. Except for the last area.
    /// Which doesn't have a split.
    ///
    /// So:
    /// - If you set the length to 0 the area will be hidden completely
    ///   and no split will be shown.
    /// - A value of 1 is fine.
    /// - The last area can have a length 0 and that's fine too.
    ///
    pub fn set_area_len(&mut self, n: usize, len: u16) {
        self.area_length[n] = len;
        self.hidden_length[n] = 0;
    }

    /// Returns the position of the nth split.
    ///
    /// __Caution__
    ///
    /// The numbering for the splitters goes from `0` to `len-1` __exclusive__.
    /// Split `n` marks the gap between area `n` and `n+1`.
    ///
    /// __Caution__
    ///
    /// This returns the position of the gap between two adjacent
    /// split-areas. Use `splitline_areas` for anything rendering related.
    ///
    pub fn split_pos(&self, n: usize) -> u16 {
        self.area_length[..n + 1].iter().sum()
    }

    /// Sets the position of the nth split.
    ///
    /// Depending on the resize strategy this can limit the allowed positions
    /// for the split.
    ///
    /// __Caution__
    ///
    /// The numbering for the splitters goes from `0` to `len-1` __exclusive__.
    /// Split `n` marks the gap between area `n` and `n+1`.
    ///
    /// __Caution__
    ///
    /// This marks the position of the gap between two adjacent
    /// split-areas. If you start from screen-coordinates it might
    /// be easier to use [set_screen_split_pos](Self::set_screen_split_pos)
    pub fn set_split_pos(&mut self, n: usize, pos: u16) {
        if n + 1 >= self.area_length.len() {
            return;
        }

        match self.resize {
            SplitResize::Neighbours => {
                self.set_split_pos_neighbour(n, pos);
            }
            SplitResize::Full => {
                self.set_split_pos_full(n, pos);
            }
        }
    }

    /// Limits the possible position of the split to the
    /// width of the two direct neighbours of the split.
    fn set_split_pos_neighbour(&mut self, n: usize, pos: u16) {
        assert!(n + 1 < self.area_length.len());

        // create dual
        let mut pos_vec = Vec::new();
        let mut pp = 0;
        for len in &self.area_length {
            pp += *len;
            pos_vec.push(pp);
        }
        // last is not a split
        let pos_count = pos_vec.len();

        let (min_pos, max_pos) = if n == 0 {
            if n + 2 >= pos_count {
                (SPLIT_WIDTH, pos_vec[n + 1])
            } else {
                (SPLIT_WIDTH, pos_vec[n + 1] - SPLIT_WIDTH)
            }
        } else if n + 2 < pos_count {
            (pos_vec[n - 1] + 1, pos_vec[n + 1] - SPLIT_WIDTH)
        } else {
            (pos_vec[n - 1] + 1, pos_vec[n + 1])
        };

        pos_vec[n] = min(max(min_pos, pos), max_pos);

        // revert dual
        for i in 0..pos_vec.len() {
            if i > 0 {
                self.area_length[i] = pos_vec[i] - pos_vec[i - 1];
            } else {
                self.area_length[i] = pos_vec[i];
            }
        }
    }

    /// Allows the full range for the split-pos.
    /// Minus the space needed to render the split itself.
    #[allow(clippy::needless_range_loop)]
    #[allow(clippy::comparison_chain)]
    fn set_split_pos_full(&mut self, n: usize, pos: u16) {
        assert!(n + 1 < self.area_length.len());

        let total_len = self.total_area_len();

        // create dual
        let mut pos_vec = Vec::new();
        let mut pp = 0;
        for len in &self.area_length {
            pp += *len;
            pos_vec.push(pp);
        }
        // last is not a split
        pos_vec.pop();
        let pos_count = pos_vec.len();

        let mut min_pos = SPLIT_WIDTH;
        for i in 0..pos_vec.len() {
            if i < n {
                if self.area_length[i] == 0 {
                    pos_vec[i] = min_pos;
                } else if self.hidden_length[i] != 0 {
                    pos_vec[i] = min_pos;
                    min_pos += SPLIT_WIDTH;
                } else {
                    if pos_vec[i] >= pos {
                        // how many split between here and there
                        let rest_area_count = n - (i + 1);
                        let rest_area_width = rest_area_count as u16 * SPLIT_WIDTH;
                        // min
                        pos_vec[i] = max(
                            min_pos,
                            pos.saturating_sub(SPLIT_WIDTH)
                                .saturating_sub(rest_area_width),
                        );
                        min_pos += SPLIT_WIDTH;
                    } else {
                        // unchanged
                    }
                }
            } else if i == n {
                // remaining area count with a split
                let rest_area_count = pos_count - (i + 1);
                let rest_area_width = rest_area_count as u16 * SPLIT_WIDTH;
                let rest_len = total_len - (min_pos + 1);
                // min for remaining areas
                let rest_len = rest_len - rest_area_width;
                // last can be 0
                let rest_len = rest_len + SPLIT_WIDTH;

                let max_pos = min_pos + rest_len;

                pos_vec[i] = min(max(min_pos, pos), max_pos);

                min_pos = pos_vec[i] + SPLIT_WIDTH;
            } else {
                if self.area_length[i] == 0 {
                    pos_vec[i] = min_pos;
                } else if self.hidden_length[i] != 0 {
                    pos_vec[i] = min_pos;
                    min_pos += SPLIT_WIDTH;
                } else {
                    if pos_vec[i] <= pos {
                        pos_vec[i] = min_pos;
                        min_pos += SPLIT_WIDTH;
                    } else {
                        // unchanged
                    }
                }
            }
        }

        // revert dual
        for i in 0..pos_vec.len() {
            if i > 0 {
                self.area_length[i] = pos_vec[i] - pos_vec[i - 1];
            } else {
                self.area_length[i] = pos_vec[i];
            }
        }
        self.area_length[pos_count] = total_len - pos_vec[pos_count - 1];
    }

    /// Is the split hidden?
    pub fn is_hidden(&self, n: usize) -> bool {
        self.hidden_length[n] > 0
    }

    /// Hide the split and adds its area to the following split.
    /// If there is no following split it will go left/up.
    /// Leaves enough space to render the splitter.
    pub fn hide_split(&mut self, n: usize) -> bool {
        if self.hidden_length[n] == 0 {
            let mut hide = if n + 1 == self.area_length.len() {
                self.area_length[n]
            } else {
                self.area_length[n].saturating_sub(SPLIT_WIDTH)
            };
            for idx in n + 1..self.area_length.len() {
                if self.hidden_length[idx] == 0 {
                    self.area_length[idx] += hide;
                    hide = 0;
                    break;
                }
            }
            if hide > 0 {
                for idx in (0..n).rev() {
                    if self.hidden_length[idx] == 0 {
                        self.area_length[idx] += hide;
                        hide = 0;
                        break;
                    }
                }
            }

            if hide > 0 {
                // don't hide last split.
                self.hidden_length[n] = 0;
                false
            } else {
                if n + 1 == self.area_length.len() {
                    self.hidden_length[n] = self.area_length[n];
                    self.area_length[n] = 0;
                } else {
                    self.hidden_length[n] = self.area_length[n].saturating_sub(SPLIT_WIDTH);
                    self.area_length[n] = 1;
                };
                true
            }
        } else {
            false
        }
    }

    /// Show a hidden split.
    /// It will first try to reduce the areas to the right,
    /// and then the areas to the left to make space.
    pub fn show_split(&mut self, n: usize) -> bool {
        let mut show = self.hidden_length[n];
        if show > 0 {
            for idx in n + 1..self.area_length.len() {
                if self.hidden_length[idx] == 0 {
                    // steal as much as we can
                    if self.area_length[idx] > show + SPLIT_WIDTH {
                        self.area_length[idx] -= show;
                        show = 0;
                    } else if self.area_length[idx] > SPLIT_WIDTH {
                        show -= self.area_length[idx] - SPLIT_WIDTH;
                        self.area_length[idx] = SPLIT_WIDTH;
                    }
                    if show == 0 {
                        break;
                    }
                }
            }
            if show > 0 {
                for idx in (0..n).rev() {
                    if self.hidden_length[idx] == 0 {
                        if self.area_length[idx] > show + SPLIT_WIDTH {
                            self.area_length[idx] -= show;
                            show = 0;
                        } else if self.area_length[idx] > SPLIT_WIDTH {
                            show -= self.area_length[idx] - SPLIT_WIDTH;
                            self.area_length[idx] = SPLIT_WIDTH;
                        }
                        if show == 0 {
                            break;
                        }
                    }
                }
            }

            self.area_length[n] += self.hidden_length[n] - show;
            self.hidden_length[n] = 0;
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

                    ct_event!(keycode press ALT-Left) => self.select_prev_split().into(),
                    ct_event!(keycode press ALT-Right) => self.select_next_split().into(),
                    ct_event!(keycode press ALT-Up) => self.select_prev_split().into(),
                    ct_event!(keycode press ALT-Down) => self.select_next_split().into(),
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
