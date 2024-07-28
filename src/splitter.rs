//!
//! Vertical and horizontal multiple split.
//!

use crate::_private::NonExhaustive;
use crate::fill::Fill;
use crate::util::revert_style;
use log::debug;
use rat_event::util::MouseFlagsN;
use rat_event::{ct_event, flow, HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::{FocusFlag, HasFocusFlag};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Position, Rect};
use ratatui::prelude::BlockExt;
use ratatui::style::Style;
use ratatui::widgets::{Block, BorderType, StatefulWidget, StatefulWidgetRef, Widget, WidgetRef};

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
    split_char: Option<&'a str>,
    join_0: Option<BorderType>,
    join_1: Option<BorderType>,
    join_0_char: Option<&'a str>,
    join_1_char: Option<&'a str>,
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

    /// Fill char for the split.
    pub split_char: Option<&'static str>,
    /// Left/Top join with the Split border.
    pub join_0_char: Option<&'static str>,
    /// Right/Bottom join with the Split border.
    pub join_1_char: Option<&'static str>,
    /// Offset for the mark from the top/left.
    pub mark_offset: u16,
    /// Marker for a horizontal split.
    /// Only the first 2 chars are used.
    pub mark_0_char: Option<&'static str>,
    /// Marker for a vertical split.
    /// Only the first 2 chars are used.
    pub mark_1_str: Option<&'static str>,

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
pub struct SplitState {
    /// Total area.
    pub area: Rect,
    /// Area inside the border.
    pub inner: Rect,

    /// Focus
    pub focus: FocusFlag,
    /// Which splitter exactly has the focus.
    pub focus_split: Option<usize>,

    /// The part areas. Use this after calling layout() to render your
    /// widgets.
    pub areas: Vec<Rect>,
    /// Area used by the splitter. This is area is used for moving the splitter.
    /// It might overlap with the widget area.
    pub split_areas: Vec<Rect>,
    /// If there is a mark_offset, that leaves a blind spot above. This is
    /// noticeable, when the split_type is Scroll.
    pub blind_area: Vec<Rect>,
    /// Start position for drawing the mark.
    pub mark_area: Vec<Position>,

    /// Direction of the split.
    pub direction: Direction,
    /// Split type.
    pub split_type: SplitType,
    /// Offset of the mark from top/left.
    pub mark_offset: u16,

    /// Mouseflags.
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
            split_char: None,
            join_0_char: None,
            join_1_char: None,
            mark_offset: 0,
            mark_0_char: None,
            mark_1_str: None,
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

    /// Fill char for the splitter.
    pub fn split_char(mut self, str: &'a str) -> Self {
        self.split_char = Some(str);
        self
    }

    /// Draw as join character between the split and the
    /// border on the left/top side.
    pub fn join_char(mut self, str: &'a str) -> Self {
        self.join_0_char = Some(str);
        self.join_1_char = Some(str);
        self
    }

    /// Draw as join character between the split and the
    /// border on the left/top side.
    pub fn join_0_char(mut self, str: &'a str) -> Self {
        self.join_0_char = Some(str);
        debug!("join_0 {:?}", self.join_0_char);
        self
    }

    /// Draw as join character between the split and the
    /// border on the right/bottom side.
    pub fn join_1_char(mut self, str: &'a str) -> Self {
        self.join_1_char = Some(str);
        debug!("join_1 {:?}", self.join_1_char);
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
        self.mark_0_char = styles.mark_0_char;
        self.mark_1_char = styles.mark_1_str;
        self.split_char = styles.split_char;
        self.join_0_char = styles.join_0_char;
        self.join_1_char = styles.join_1_char;
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
    /// When a resize is detected, the current area-width/height is used as
    /// Fill() constraint for the new layout.
    fn layout_split(&self, area: Rect, state: &mut SplitState) {
        state.area = area;
        state.inner = self.block.inner_if_some(area);

        // use only the inner from here on
        let inner = state.inner;

        let meta_change = state.direction != self.direction
            || state.split_type != self.split_type
            || state.mark_offset != self.mark_offset;

        let new_split_areas = if state.areas.is_empty() {
            // initial
            let new_areas = Layout::new(self.direction, self.constraints.clone())
                .flex(Flex::Legacy)
                .split(inner);
            Some(new_areas)
        } else {
            let length = |v: &Rect| {
                // must use the old direction to get a correct value.
                if state.direction == Direction::Horizontal {
                    v.width
                } else {
                    v.height
                }
            };

            let mut old_length: u16 = state.areas.iter().map(length).sum();
            if self.split_type.is_full() {
                old_length += state.split_areas.iter().map(length).sum::<u16>();
            }

            if meta_change || length(&inner) != old_length {
                let mut constraints = Vec::new();
                for i in 0..state.areas.len() {
                    if self.split_type.is_full() {
                        if i < state.split_areas.len() {
                            constraints.push(Constraint::Fill(
                                length(&state.areas[i]) + length(&state.split_areas[i]),
                            ));
                        } else {
                            constraints.push(Constraint::Fill(length(&state.areas[i])));
                        }
                    } else {
                        constraints.push(Constraint::Fill(length(&state.areas[i])));
                    }
                }

                let new_areas = Layout::new(self.direction, constraints).split(inner);
                Some(new_areas)
            } else {
                None
            }
        };

        // Areas changed, create areas and splits.
        if let Some(rects) = new_split_areas {
            state.areas.clear();
            state.split_areas.clear();
            state.blind_area.clear();
            state.mark_area.clear();

            for mut area in rects.iter().take(rects.len().saturating_sub(1)).copied() {
                let mut split = if self.direction == Direction::Horizontal {
                    Rect::new(
                        area.x + area.width.saturating_sub(SPLIT_WIDTH),
                        area.y,
                        1,
                        area.height,
                    )
                } else {
                    Rect::new(
                        area.x,
                        area.y + area.height.saturating_sub(SPLIT_WIDTH),
                        area.width,
                        1,
                    )
                };
                let mut mark = if self.direction == Direction::Horizontal {
                    Position::new(
                        area.x + area.width.saturating_sub(SPLIT_WIDTH),
                        area.y + self.mark_offset,
                    )
                } else {
                    Position::new(
                        area.x + self.mark_offset,
                        area.y + area.height.saturating_sub(SPLIT_WIDTH),
                    )
                };
                let mut blind = if self.direction == Direction::Horizontal {
                    Rect::new(
                        area.x + area.width.saturating_sub(SPLIT_WIDTH),
                        area.y,
                        1,
                        self.mark_offset,
                    )
                } else {
                    Rect::new(
                        area.x,
                        area.y + area.height.saturating_sub(SPLIT_WIDTH),
                        self.mark_offset,
                        1,
                    )
                };

                self.adjust_for_split_type(&mut area, &mut split, &mut blind, &mut mark);

                state.areas.push(area);
                state.split_areas.push(split);
                state.blind_area.push(blind);
                state.mark_area.push(mark);
            }
            if let Some(area) = rects.last() {
                state.areas.push(*area);
            }
        }

        // Set 2nd dimension too, if necessary.
        if let Some(test) = state.areas.first() {
            if self.direction == Direction::Horizontal {
                if test.height != inner.height {
                    for r in &mut state.areas {
                        r.height = inner.height;
                    }
                    for r in &mut state.split_areas {
                        r.height = inner.height;
                    }
                }
            } else {
                if test.width != inner.width {
                    for r in &mut state.areas {
                        r.width = inner.width;
                    }
                    for r in &mut state.split_areas {
                        r.width = inner.width;
                    }
                }
            }
        }

        //
        state.direction = self.direction;
        state.split_type = self.split_type;
        state.mark_offset = self.mark_offset;
    }

    /// Adjust area and split according to the split_type.
    fn adjust_for_split_type(
        &self,
        area: &mut Rect,
        split: &mut Rect,
        _blind: &mut Rect,
        mark: &mut Position,
    ) {
        use Direction::*;
        use SplitType::*;

        match (self.direction, self.split_type) {
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
        if self.split_char.is_some() {
            return self.split_char;
        };

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

    fn get_blind_char(&self) -> Option<&str> {
        if self.split_char.is_some() {
            return self.split_char;
        };

        use Direction::*;
        use SplitType::*;

        match (self.direction, self.split_type) {
            (_, FullEmpty) => None,
            (Horizontal, Scroll) => Some("\u{2502}"),
            (Vertical, Scroll) => Some("\u{2500}"),
            (_, FullPlain) => None,
            (_, FullDouble) => None,
            (_, FullThick) => None,
            (_, FullQuadrantInside) => None,
            (_, FullQuadrantOutside) => None,
            (_, Widget) => None,
        }
    }

    fn get_join_0(&self, split_area: Rect, state: &SplitState) -> Option<(Position, &str)> {
        use BorderType::*;
        use Direction::*;
        use SplitType::*;

        let s: Option<&str> = if self.join_0_char.is_some() {
            self.join_0_char
        } else if let Some(join_0) = self.join_0 {
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

        let s: Option<&str> = if self.join_1_char.is_some() {
            self.join_1_char
        } else if let Some(join_1) = self.join_1 {
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

impl<'a> StatefulWidgetRef for SplitWidget<'a> {
    type State = SplitState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_widget(&self.split, area, buf, state);
        if !matches!(self.split.split_type, SplitType::Widget | SplitType::Scroll) {
            render_split(&self.split, buf, state);
        }
    }
}

impl<'a> StatefulWidget for SplitWidget<'a> {
    type State = SplitState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_widget(&self.split, area, buf, state);
        if !matches!(self.split.split_type, SplitType::Widget | SplitType::Scroll) {
            render_split(&self.split, buf, state);
        }
    }
}

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

fn render_widget(split: &Split<'_>, area: Rect, buf: &mut Buffer, state: &mut SplitState) {
    split.layout_split(area, state);

    if state.is_focused() {
        if state.focus_split.is_none() {
            state.focus_split = Some(0);
        }
    } else {
        state.focus_split = None;
    }

    split.block.render_ref(area, buf);
}

fn render_split(split: &Split<'_>, buf: &mut Buffer, state: &mut SplitState) {
    for (n, split_area) in state.split_areas.iter().enumerate() {
        let arrow_style = if let Some(arrow) = split.arrow_style {
            arrow
        } else {
            split.style
        };
        let (style, arrow_style) =
            if Some(n) == state.mouse.drag.get() || Some(n) == state.focus_split {
                if let Some(drag) = split.drag_style {
                    (drag, drag)
                } else {
                    (revert_style(split.style), arrow_style)
                }
            } else {
                (split.style, arrow_style)
            };

        if let Some(fill) = split.get_fill_char() {
            Fill::new()
                .style(style)
                .fill_char(fill)
                .render(*split_area, buf);
        }

        // mark_offset leaves some parts unrendered.
        if split.split_type == SplitType::Scroll {
            if split.mark_offset > 0 {
                let mut f = Fill::new().style(split.style);
                if let Some(blind) = split.get_blind_char() {
                    f = f.fill_char(blind);
                }
                f.render(state.blind_area[n], buf);
            }
        }

        let mark = state.mark_area[n];
        if split.direction == Direction::Horizontal {
            if buf.area.contains((mark.x, mark.y).into()) {
                buf.get_mut(mark.x, mark.y).set_style(arrow_style);
                buf.get_mut(mark.x, mark.y).set_symbol(split.get_mark_0());
            }
            if buf.area.contains((mark.x, mark.y + 1).into()) {
                buf.get_mut(mark.x, mark.y + 1).set_style(arrow_style);
                buf.get_mut(mark.x, mark.y + 1)
                    .set_symbol(split.get_mark_1());
            }
        } else {
            if buf.area.contains((mark.x, mark.y).into()) {
                buf.get_mut(mark.x, mark.y).set_style(arrow_style);
                buf.get_mut(mark.x, mark.y).set_symbol(split.get_mark_0());
            }
            if buf.area.contains((mark.x + 1, mark.y).into()) {
                buf.get_mut(mark.x + 1, mark.y).set_style(arrow_style);
                buf.get_mut(mark.x + 1, mark.y)
                    .set_symbol(split.get_mark_1());
            }
        }

        if let Some((pos_0, c_0)) = split.get_join_0(*split_area, state) {
            buf.get_mut(pos_0.x, pos_0.y).set_symbol(c_0);
        }
        if let Some((pos_1, c_1)) = split.get_join_1(*split_area, state) {
            buf.get_mut(pos_1.x, pos_1.y).set_symbol(c_1);
        }
    }
}

impl Default for SplitState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            inner: Default::default(),
            focus: Default::default(),
            focus_split: Default::default(),
            areas: Default::default(),
            split_areas: Default::default(),
            blind_area: Default::default(),
            mark_area: Default::default(),
            direction: Default::default(),
            split_type: Default::default(),
            mark_offset: 0,
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl HasFocusFlag for SplitState {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        // not mouse focusable
        Rect::default()
    }

    fn navigable(&self) -> bool {
        false
    }
}

impl SplitState {
    /// Set the position for the nth splitter.
    ///
    /// The position is limited the combined area of the two adjacent areas.
    pub fn set_screen_split_pos(&mut self, n: usize, pos: (u16, u16)) -> bool {
        use SplitType::*;

        let area1 = self.areas[n];
        let area2 = self.areas[n + 1];
        let area = area1.union(area2);

        if self.direction == Direction::Horizontal {
            match self.split_type {
                FullEmpty | FullPlain | FullDouble | FullThick | FullQuadrantInside
                | FullQuadrantOutside => {
                    let p = if pos.0 < area.left() {
                        area.left()
                    } else if pos.0 >= area.right() {
                        area.right()
                    } else {
                        pos.0
                    };

                    self.areas[n] = Rect::new(area1.x, area1.y, p - area1.x, area1.height);
                    self.split_areas[n] = Rect::new(p, area1.y, 1, area1.height);
                    self.blind_area[n] = Rect::new(p, area1.y, 1, self.mark_offset);
                    self.mark_area[n] = Position::new(p, area1.y + self.mark_offset);
                    self.areas[n + 1] = Rect::new(p + 1, area2.y, area2.right() - p, area2.height);
                }
                Scroll => {
                    let p = if pos.0 < area.left() {
                        area.left()
                    } else if pos.0 >= area.right() {
                        area.right().saturating_sub(1)
                    } else {
                        pos.0
                    };

                    self.areas[n] = Rect::new(area1.x, area1.y, (p + 1) - area1.x, area1.height);
                    self.split_areas[n] = Rect::new(p, area1.y + self.mark_offset, 1, 2);
                    self.blind_area[n] = Rect::new(p, area1.y, 1, self.mark_offset);
                    self.mark_area[n] = Position::new(p, area1.y + self.mark_offset);
                    self.areas[n + 1] =
                        Rect::new(p + 1, area2.y, area2.right() - 1 - p, area2.height);
                }
                Widget => {
                    let p = if pos.0 < area.left() {
                        area.left()
                    } else if pos.0 >= area.right() {
                        area.right().saturating_sub(1)
                    } else {
                        pos.0
                    };
                    self.areas[n] = Rect::new(area1.x, area1.y, (p + 1) - area1.x, area1.height);
                    self.split_areas[n] = Rect::new(p, area1.y, 1, area1.height);
                    self.blind_area[n] = Rect::new(p, area1.y, 1, self.mark_offset);
                    self.mark_area[n] = Position::default();
                    self.areas[n + 1] =
                        Rect::new(p + 1, area2.y, area2.right() - 1 - p, area2.height);
                }
            }
        } else {
            match self.split_type {
                FullEmpty | FullPlain | FullDouble | FullThick | FullQuadrantInside
                | FullQuadrantOutside => {
                    let p = if pos.1 < area.top() {
                        area.top()
                    } else if pos.1 >= area.bottom() {
                        area.bottom()
                    } else {
                        pos.1
                    };
                    self.areas[n] = Rect::new(area1.x, area1.y, area1.width, p - area1.y);
                    self.split_areas[n] = Rect::new(area1.x, p, area1.width, 1);
                    self.blind_area[n] = Rect::new(area1.x, p, self.mark_offset, 1);
                    self.mark_area[n] = Position::new(area1.x + self.mark_offset, p);
                    self.areas[n + 1] = Rect::new(area2.x, p + 1, area2.width, area2.bottom() - p);
                }
                Scroll => {
                    let p = if pos.1 < area.top() {
                        area.top()
                    } else if pos.1 >= area.bottom() {
                        area.bottom().saturating_sub(1)
                    } else {
                        pos.1
                    };
                    self.areas[n] = Rect::new(area1.x, area1.y, area1.width, (p + 1) - area1.y);
                    self.split_areas[n] = Rect::new(area1.x + self.mark_offset, p, 2, 1);
                    self.blind_area[n] = Rect::new(area1.x, p, self.mark_offset, 1);
                    self.mark_area[n] = Position::new(area1.x + self.mark_offset, p);
                    self.areas[n + 1] =
                        Rect::new(area2.x, p + 1, area2.width, area2.bottom() - 1 - p);
                }
                Widget => {
                    let p = if pos.1 < area.top() {
                        area.top()
                    } else if pos.1 >= area.bottom().saturating_sub(1) {
                        area.bottom().saturating_sub(2)
                    } else {
                        pos.1
                    };
                    self.areas[n] = Rect::new(area1.x, area1.y, area1.width, (p + 1) - area1.y);
                    self.split_areas[n] = Rect::new(area1.x, p, area1.width, 1);
                    self.blind_area[n] = Rect::new(area1.x, p, self.mark_offset, 1);
                    self.mark_area[n] = Position::default();
                    self.areas[n + 1] =
                        Rect::new(area2.x, p + 1, area2.width, area2.bottom() - 1 - p);
                }
            }
        }

        area1 != self.areas[n] || area2 != self.areas[n + 1]
    }

    /// Move the nth split position.
    /// Does nothing if the direction is not matching.
    pub fn move_split_left(&mut self, n: usize, delta: u16) -> bool {
        let split = self.split_areas[n];
        if self.direction == Direction::Horizontal {
            self.set_screen_split_pos(n, (split.left().saturating_sub(delta), split.y))
        } else {
            false
        }
    }

    /// Move the nth split position.
    /// Does nothing if the direction is not matching.
    pub fn move_split_right(&mut self, n: usize, delta: u16) -> bool {
        let split = self.split_areas[n];
        if self.direction == Direction::Horizontal {
            self.set_screen_split_pos(n, (split.right() + delta, split.y))
        } else {
            false
        }
    }

    /// Move the nth split position.
    /// Does nothing if the direction is not matching.
    pub fn move_split_up(&mut self, n: usize, delta: u16) -> bool {
        let split = self.split_areas[n];
        if self.direction == Direction::Vertical {
            self.set_screen_split_pos(n, (split.x, split.top().saturating_sub(delta)))
        } else {
            false
        }
    }

    /// Move the nth split position.
    /// Does nothing if the direction is not matching.
    pub fn move_split_down(&mut self, n: usize, delta: u16) -> bool {
        let split = self.split_areas[n];
        if self.direction == Direction::Vertical {
            self.set_screen_split_pos(n, (split.x, split.bottom() + delta))
        } else {
            false
        }
    }

    /// Select the next splitter for manual adjustment.
    pub fn select_next_split(&mut self) -> bool {
        if self.is_focused() {
            let n = self.focus_split.unwrap_or_default();
            if n + 1 >= self.split_areas.len() {
                self.focus_split = Some(0);
            } else {
                self.focus_split = Some(n + 1);
            }
            true
        } else {
            false
        }
    }

    /// Select the previous splitter for manual adjustment.
    pub fn select_prev_split(&mut self) -> bool {
        if self.is_focused() {
            let n = self.focus_split.unwrap_or_default();
            if n == 0 {
                self.focus_split = Some(self.split_areas.len() - 1);
            } else {
                self.focus_split = Some(n - 1);
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
            if let Some(n) = self.focus_split {
                match event {
                    ct_event!(keycode press Left) => self.move_split_left(n, 1).into(),
                    ct_event!(keycode press Right) => self.move_split_right(n, 1).into(),
                    ct_event!(keycode press Up) => self.move_split_up(n, 1).into(),
                    ct_event!(keycode press Down) => self.move_split_down(n, 1).into(),

                    ct_event!(keycode press CONTROL-Left) => self.select_next_split().into(),
                    ct_event!(keycode press CONTROL-Right) => self.select_prev_split().into(),
                    ct_event!(keycode press CONTROL-Up) => self.select_next_split().into(),
                    ct_event!(keycode press CONTROL-Down) => self.select_prev_split().into(),
                    _ => Outcome::NotUsed,
                }
            } else {
                Outcome::NotUsed
            }
        } else {
            Outcome::NotUsed
        });

        self.handle(event, MouseOnly)
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for SplitState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> Outcome {
        match event {
            ct_event!(mouse any for m) => {
                let was_drag = self.mouse.drag.get();
                if self.mouse.drag(&self.split_areas, m) {
                    if let Some(n) = self.mouse.drag.get() {
                        self.set_screen_split_pos(n, self.mouse.pos_of(m)).into()
                    } else {
                        Outcome::NotUsed
                    }
                } else {
                    // repaint after drag is finished. resets the displayed style.
                    if was_drag.is_some() {
                        Outcome::Changed
                    } else {
                        Outcome::NotUsed
                    }
                }
            }
            _ => Outcome::NotUsed,
        }
    }
}
