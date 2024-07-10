use crate::_private::NonExhaustive;
use crate::util::revert_style;
use crossterm::cursor::SetCursorStyle::DefaultUserShape;
use log::debug;
use rat_event::util::MouseFlagsN;
use rat_event::{ct_event, flow, FocusKeys, HandleEvent, MouseOnly, Outcome};
use rat_focus::{FocusFlag, HasFocusFlag, ZRect};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::prelude::BlockExt;
use ratatui::style::Style;
use ratatui::widgets::{Block, StatefulWidget, StatefulWidgetRef, WidgetRef};
use unicode_segmentation::UnicodeSegmentation;

/// Splits the area in multiple parts and allows changing the sizes.
///
/// This widget doesn't hold a reference to the rendered widgets or such,
/// instead it provides a [layout] function. This calculates all the
/// areas based on the constraints/user input.
///
/// Then you can access the areas for each widgets via `state.areas[n]`
/// and render each widget.
///
/// Only after the inner widgets have been rendered, you call `render()`
/// for the Split widget itself.
#[derive(Debug)]
pub struct Split<'a> {
    direction: Direction,
    constraints: Vec<Constraint>,

    split_style: SplitType,
    block: Option<Block<'a>>,

    style: Style,
    arrow_style: Option<Style>,
    drag_style: Option<Style>,
    horizontal_mark: Option<&'a str>,
    vertical_mark: Option<&'a str>,
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
    /// each widget.
    #[default]
    Full,
    /// Render a minimal splitter that will be rendered over the widget.
    /// If the widget has a Scroll, this will integrate nicely. The widget
    /// will get the full area.
    ///
    /// You will have to set `collab_split` with Scroll, then Scroll can
    /// adjust its rendering.
    Scroll,
    /// Render a minimal splitter that will be rendered over the widget.
    /// If the widget has a Scroll, this will integrate nicely. The widget
    /// will get the full area.
    ///
    /// This is what you choose if you have a right/bottom border for
    /// the widget. The marker will not be drawn over the corner, but
    /// a bit inset instead.
    ///
    /// You will have to set `collab_split` with Scroll, then Scroll can
    /// adjust its rendering.
    ScrollbarBlock,
    /// Don't render a splitter, but do full manual mode.
    ///
    /// The widget will have the full area, but the event-handling will
    /// use the last column/row of the widget for moving the split.
    /// This can be adjusted if you change `state.split[n]` which provides
    /// the active area.
    None,
}

const SPLIT_WIDTH: u16 = 1;

/// State of the Split.
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
    pub split: Vec<Rect>,

    /// Direction of the split.
    pub direction: Direction,
    /// Split type.
    pub split_style: SplitType,

    /// Mouseflags.
    pub mouse: MouseFlagsN,

    pub non_exhaustive: NonExhaustive,
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
        let mut s = Self {
            direction: Default::default(),
            constraints: Default::default(),
            split_style: Default::default(),
            block: Default::default(),
            style: Default::default(),
            arrow_style: Default::default(),
            drag_style: Default::default(),
            horizontal_mark: Default::default(),
            vertical_mark: Default::default(),
        };
        s.direction = Direction::Horizontal;
        s
    }

    /// Constraints.
    pub fn constraints(mut self, constraints: impl IntoIterator<Item = Constraint>) -> Self {
        self.constraints = constraints.into_iter().collect();
        self
    }

    /// Layout direction of the widgets.
    /// Direction::Horizontal means the widgets are layed out on
    /// beside the other, with a vertical split area in between.
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Controls rendering of the splitter.
    pub fn split_type(mut self, split_style: SplitType) -> Self {
        self.split_style = split_style;
        self
    }

    /// Border block.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set all styles.
    pub fn styles(mut self, styles: SplitStyle) -> Self {
        self.style = styles.style;
        self.drag_style = styles.drag_style;
        self.arrow_style = styles.arrow_style;
        self.horizontal_mark = styles.horizontal_mark;
        self.vertical_mark = styles.vertical_mark;
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

    /// Marker for a horizontal splitter.
    /// Only the first two characters are used.
    pub fn horizontal_mark(mut self, mark: &'a str) -> Self {
        self.horizontal_mark = Some(mark);
        self
    }

    /// Marker for a vertical splitter.
    /// Only the first two characters are used.
    pub fn vertical_mark(mut self, mark: &'a str) -> Self {
        self.vertical_mark = Some(mark);
        self
    }
}

impl<'a> Split<'a> {
    /// Just run all the layout for the widget.
    /// After this state.area has sensible data.
    pub fn layout(&self, area: Rect, state: &mut SplitState) {
        state.direction = self.direction;
        state.area = area;
        state.inner = self.block.inner_if_some(area);
        state.split_style = self.split_style;

        self.layout_split(state.inner, state);
    }

    /// Calculates the first layout according to the constraints.
    /// When a resize is detected, the current area-width/height is used as
    /// Fill() constraint for the new layout.
    fn layout_split(&self, area: Rect, state: &mut SplitState) {
        let rects = if state.areas.is_empty() {
            // initial
            Some(
                Layout::new(self.direction, self.constraints.clone())
                    .flex(Flex::Legacy)
                    .split(area),
            )
        } else {
            if self.direction == Direction::Horizontal {
                // size changed
                let mut width = state.areas.iter().map(|v| v.width).sum::<u16>();
                match self.split_style {
                    SplitType::Full => {
                        width += state.split.iter().map(|v| v.width).sum::<u16>();
                    }
                    SplitType::Scroll | SplitType::ScrollbarBlock => {}
                    SplitType::None => {}
                }
                if area.width != width {
                    let mut c = Vec::new();
                    for i in 0..state.areas.len() {
                        match self.split_style {
                            SplitType::Full => {
                                if i < state.split.len() {
                                    c.push(Constraint::Fill(
                                        state.areas[i].width + state.split[i].width,
                                    ));
                                } else {
                                    c.push(Constraint::Fill(state.areas[i].width));
                                }
                            }
                            SplitType::Scroll | SplitType::ScrollbarBlock => {
                                c.push(Constraint::Fill(state.areas[i].width));
                            }
                            SplitType::None => {
                                c.push(Constraint::Fill(state.areas[i].width));
                            }
                        }
                    }
                    Some(Layout::horizontal(c).split(area))
                } else {
                    None
                }
            } else {
                // size changed
                let mut height = state.areas.iter().map(|v| v.height).sum::<u16>();
                match self.split_style {
                    SplitType::Full => {
                        height += state.split.iter().map(|v| v.height).sum::<u16>();
                    }
                    SplitType::Scroll | SplitType::ScrollbarBlock => {}
                    SplitType::None => {}
                }
                if area.height != height {
                    let mut c = Vec::new();
                    for i in 0..state.areas.len() {
                        match self.split_style {
                            SplitType::Full => {
                                if i < state.split.len() {
                                    c.push(Constraint::Fill(
                                        state.areas[i].height + state.split[i].height,
                                    ));
                                } else {
                                    c.push(Constraint::Fill(state.areas[i].height));
                                }
                            }
                            SplitType::Scroll | SplitType::ScrollbarBlock => {
                                c.push(Constraint::Fill(state.areas[i].height));
                            }
                            SplitType::None => {
                                c.push(Constraint::Fill(state.areas[i].height));
                            }
                        }
                    }
                    Some(Layout::vertical(c).split(area))
                } else {
                    None
                }
            }
        };

        if let Some(rects) = rects {
            state.areas.clear();
            state.split.clear();

            for area in rects.iter().take(rects.len().saturating_sub(1)) {
                if state.direction == Direction::Horizontal {
                    let (area, split) = match self.split_style {
                        SplitType::Full => (
                            Rect::new(
                                area.x,
                                area.y,
                                area.width.saturating_sub(SPLIT_WIDTH),
                                area.height,
                            ),
                            Rect::new(
                                area.x + area.width.saturating_sub(SPLIT_WIDTH),
                                area.y,
                                1,
                                area.height,
                            ),
                        ),
                        SplitType::Scroll => (
                            Rect::new(area.x, area.y, area.width, area.height),
                            Rect::new(
                                area.x + area.width.saturating_sub(SPLIT_WIDTH),
                                area.y,
                                1,
                                2,
                            ),
                        ),
                        SplitType::ScrollbarBlock => (
                            Rect::new(area.x, area.y, area.width, area.height),
                            Rect::new(
                                area.x + area.width.saturating_sub(SPLIT_WIDTH),
                                area.y + 1,
                                1,
                                2,
                            ),
                        ),
                        SplitType::None => (
                            Rect::new(area.x, area.y, area.width, area.height),
                            Rect::new(
                                area.x + area.width.saturating_sub(SPLIT_WIDTH),
                                area.y,
                                1,
                                area.height,
                            ),
                        ),
                    };

                    state.areas.push(area);
                    state.split.push(split);
                } else {
                    let (area, split) = match self.split_style {
                        SplitType::Full => (
                            Rect::new(
                                area.x,
                                area.y,
                                area.width,
                                area.height.saturating_sub(SPLIT_WIDTH),
                            ),
                            Rect::new(
                                area.x,
                                area.y + area.height.saturating_sub(SPLIT_WIDTH),
                                area.width,
                                1,
                            ),
                        ),
                        SplitType::Scroll => (
                            Rect::new(area.x, area.y, area.width, area.height),
                            Rect::new(
                                area.x,
                                area.y + area.height.saturating_sub(SPLIT_WIDTH),
                                2,
                                1,
                            ),
                        ),
                        SplitType::ScrollbarBlock => (
                            Rect::new(area.x, area.y, area.width, area.height),
                            Rect::new(
                                area.x + 1,
                                area.y + area.height.saturating_sub(SPLIT_WIDTH),
                                2,
                                1,
                            ),
                        ),
                        SplitType::None => (
                            Rect::new(area.x, area.y, area.width, area.height),
                            Rect::new(
                                area.x,
                                area.y + area.height.saturating_sub(SPLIT_WIDTH),
                                area.width,
                                1,
                            ),
                        ),
                    };

                    state.areas.push(area);
                    state.split.push(split);
                }
            }
            if let Some(area) = rects.last() {
                state.areas.push(*area);
            }
        }

        if let Some(test) = state.areas.get(0) {
            if state.direction == Direction::Horizontal {
                if test.height != area.height {
                    for r in &mut state.areas {
                        r.height = area.height;
                    }
                    for r in &mut state.split {
                        r.height = area.height;
                    }
                }
            } else {
                if test.width != area.width {
                    for r in &mut state.areas {
                        r.width = area.width;
                    }
                    for r in &mut state.split {
                        r.width = area.width;
                    }
                }
            }
        }
    }
}

impl<'a> StatefulWidget for Split<'a> {
    type State = SplitState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.layout(state.inner, state);

        if state.is_focused() {
            if state.focus_split.is_none() {
                state.focus_split = Some(0);
            }
        } else {
            state.focus_split = None;
        }

        self.block.render_ref(area, buf);

        if !matches!(self.split_style, SplitType::None) {
            for (n, split_area) in state.split.iter().enumerate() {
                let arrow_style = if let Some(arrow) = self.arrow_style {
                    arrow
                } else {
                    self.style
                };
                let (style, arrow_style) =
                    if Some(n) == state.mouse.drag.get() || Some(n) == state.focus_split {
                        if let Some(drag) = self.drag_style {
                            (drag, drag)
                        } else {
                            (revert_style(self.style), arrow_style)
                        }
                    } else {
                        (self.style, arrow_style)
                    };

                buf.set_style(*split_area, style);

                let (x, y) = (split_area.x, split_area.y);
                if self.direction == Direction::Horizontal {
                    let mark = if let Some(mark) = self.horizontal_mark {
                        mark
                    } else {
                        "<>"
                    };
                    let mut mark = mark.graphemes(true);
                    if buf.area.contains((x, y).into()) {
                        buf.get_mut(x, y).set_style(arrow_style);
                        buf.get_mut(x, y).set_symbol(mark.next().unwrap_or(" "));
                    }
                    if buf.area.contains((x, y + 1).into()) {
                        buf.get_mut(x, y + 1).set_style(arrow_style);
                        buf.get_mut(x, y + 1).set_symbol(mark.next().unwrap_or(" "));
                    }
                } else {
                    let mark = if let Some(mark) = self.horizontal_mark {
                        mark
                    } else {
                        "^v"
                    };
                    let mut mark = mark.graphemes(true);
                    if buf.area.contains((x, y).into()) {
                        buf.get_mut(x, y).set_style(arrow_style);
                        buf.get_mut(x, y).set_symbol(mark.next().unwrap_or(" "));
                    }
                    if buf.area.contains((x + 1, y).into()) {
                        buf.get_mut(x + 1, y).set_style(arrow_style);
                        buf.get_mut(x + 1, y).set_symbol(mark.next().unwrap_or(" "));
                    }
                }
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
            focus_split: Default::default(),
            areas: Default::default(),
            split: Default::default(),
            direction: Default::default(),
            split_style: Default::default(),
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
        let area1 = self.areas[n];
        let area2 = self.areas[n + 1];
        let area = area1.union(area2);

        if self.direction == Direction::Horizontal {
            match self.split_style {
                SplitType::Full => {
                    let p = if pos.0 < area.left() {
                        area.left()
                    } else if pos.0 >= area.right() {
                        area.right()
                    } else {
                        pos.0
                    };

                    self.areas[n] = Rect::new(area1.x, area1.y, p - area1.x, area1.height);
                    self.split[n] = Rect::new(p, area1.y, 1, area1.height);
                    self.areas[n + 1] = Rect::new(p + 1, area2.y, area2.right() - p, area2.height);
                }
                SplitType::Scroll => {
                    let p = if pos.0 < area.left() {
                        area.left()
                    } else if pos.0 >= area.right().saturating_sub(1) {
                        area.right().saturating_sub(2)
                    } else {
                        pos.0
                    };

                    self.areas[n] = Rect::new(area1.x, area1.y, (p + 1) - area1.x, area1.height);
                    self.split[n] = Rect::new(p, area1.y, 1, 2);
                    self.areas[n + 1] =
                        Rect::new(p + 1, area2.y, area2.right() - 1 - p, area2.height);
                }
                SplitType::ScrollbarBlock => {
                    let p = if pos.0 < area.left() {
                        area.left()
                    } else if pos.0 >= area.right().saturating_sub(1) {
                        area.right().saturating_sub(2)
                    } else {
                        pos.0
                    };

                    self.areas[n] = Rect::new(area1.x, area1.y, (p + 1) - area1.x, area1.height);
                    self.split[n] = Rect::new(p, area1.y + 1, 1, 2);
                    self.areas[n + 1] =
                        Rect::new(p + 1, area2.y, area2.right() - 1 - p, area2.height);
                }
                SplitType::None => {
                    let p = if pos.0 < area.left() {
                        area.left()
                    } else if pos.0 >= area.right().saturating_sub(1) {
                        area.right().saturating_sub(2)
                    } else {
                        pos.0
                    };
                    self.areas[n] = Rect::new(area1.x, area1.y, (p + 1) - area1.x, area1.height);
                    self.split[n] = Rect::new(p, area1.y, 1, area1.height);
                    self.areas[n + 1] =
                        Rect::new(p + 1, area2.y, area2.right() - 1 - p, area2.height);
                }
            }
        } else {
            match self.split_style {
                SplitType::Full => {
                    let p = if pos.1 < area.top() {
                        area.top()
                    } else if pos.1 >= area.bottom() {
                        area.bottom()
                    } else {
                        pos.1
                    };
                    self.areas[n] = Rect::new(area1.x, area1.y, area1.width, p - area1.y);
                    self.split[n] = Rect::new(area1.x, p, area1.width, 1);
                    self.areas[n + 1] = Rect::new(area2.x, p + 1, area2.width, area2.bottom() - p);
                }
                SplitType::Scroll => {
                    let p = if pos.1 < area.top() {
                        area.top()
                    } else if pos.1 >= area.bottom().saturating_sub(1) {
                        area.bottom().saturating_sub(2)
                    } else {
                        pos.1
                    };
                    self.areas[n] = Rect::new(area1.x, area1.y, area1.width, (p + 1) - area1.y);
                    self.split[n] = Rect::new(area1.x, p, 2, 1);
                    self.areas[n + 1] =
                        Rect::new(area2.x, p + 1, area2.width, area2.bottom() - 1 - p);
                }
                SplitType::ScrollbarBlock => {
                    let p = if pos.1 < area.top() {
                        area.top()
                    } else if pos.1 >= area.bottom().saturating_sub(1) {
                        area.bottom().saturating_sub(2)
                    } else {
                        pos.1
                    };
                    self.areas[n] = Rect::new(area1.x, area1.y, area1.width, (p + 1) - area1.y);
                    self.split[n] = Rect::new(area1.x + 1, p, 2, 1);
                    self.areas[n + 1] =
                        Rect::new(area2.x, p + 1, area2.width, area2.bottom() - 1 - p);
                }
                SplitType::None => {
                    let p = if pos.1 < area.top() {
                        area.top()
                    } else if pos.1 >= area.bottom().saturating_sub(1) {
                        area.bottom().saturating_sub(2)
                    } else {
                        pos.1
                    };
                    self.areas[n] = Rect::new(area1.x, area1.y, area1.width, (p + 1) - area1.y);
                    self.split[n] = Rect::new(area1.x, p, area1.width, 1);
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
        let split = self.split[n];
        if self.direction == Direction::Horizontal {
            self.set_screen_split_pos(n, (split.left().saturating_sub(delta), split.y))
        } else {
            false
        }
    }

    /// Move the nth split position.
    /// Does nothing if the direction is not matching.
    pub fn move_split_right(&mut self, n: usize, delta: u16) -> bool {
        let split = self.split[n];
        if self.direction == Direction::Horizontal {
            self.set_screen_split_pos(n, (split.right() + delta, split.y))
        } else {
            false
        }
    }

    /// Move the nth split position.
    /// Does nothing if the direction is not matching.
    pub fn move_split_up(&mut self, n: usize, delta: u16) -> bool {
        let split = self.split[n];
        if self.direction == Direction::Vertical {
            self.set_screen_split_pos(n, (split.x, split.top().saturating_sub(delta)))
        } else {
            false
        }
    }

    /// Move the nth split position.
    /// Does nothing if the direction is not matching.
    pub fn move_split_down(&mut self, n: usize, delta: u16) -> bool {
        let split = self.split[n];
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
            if n + 1 >= self.split.len() {
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
                self.focus_split = Some(self.split.len() - 1);
            } else {
                self.focus_split = Some(n - 1);
            }
            true
        } else {
            false
        }
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for SplitState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: FocusKeys) -> Outcome {
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
                if self.mouse.drag(&self.split, m) {
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
