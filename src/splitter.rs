use crate::util::revert_style;
use rat_event::util::MouseFlagsN;
use rat_event::{ct_event, flow, ConsumedEvent, HandleEvent, Outcome};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::prelude::{BlockExt, Line};
use ratatui::style::Style;
use ratatui::widgets::{Block, StatefulWidget, StatefulWidgetRef, WidgetRef};

/// Trait for rendering the split widget.
pub trait SplitWidget {
    type State;

    /// Render the n-th split area.
    fn render(&self, n: usize, area: Rect, buf: &mut Buffer, state: &mut Self::State);
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum SplitStyle {
    /// Render a full splitter between the widgets.
    #[default]
    Full,
    /// Render a minimal spliter between the widgets.
    Minimal,
    /// Don't render a splitter.
    None,
}

/// Splits the area in n parts and displays a moveable separator
/// between the areas.
///
/// The widget to render the parts is a SplitWidget.
#[derive(Debug)]
pub struct Split<'a, W> {
    direction: Direction,
    constraints: Vec<Constraint>,

    widget: W,

    split_style: SplitStyle,
    block: Option<Block<'a>>,

    style: Style,
    drag_style: Option<Style>,
}

const SPLIT_WIDTH: u16 = 1;

/// State of the Split.
#[derive(Debug, Default)]
pub struct SplitState<S> {
    /// Total area.
    pub area: Rect,
    /// Area inside the border.
    pub inner: Rect,

    /// The part areas.
    pub areas: Vec<Rect>,
    /// Areas used by the splitter itself.
    pub split: Vec<Rect>,

    /// Direction of the split.
    pub direction: Direction,
    pub split_style: SplitStyle,

    /// The state for the widget.
    pub w: S,

    /// Use keyboard to change the nth split.
    /// No events are forwarded to the inner widget
    /// as long as this takes.
    pub key_nav: Option<usize>,
    /// Mouseflags.
    pub mouse: MouseFlagsN,
}

impl<'a, W> Split<'a, W>
where
    W: SplitWidget,
{
    pub fn new(widget: W) -> Self {
        let mut s = Self {
            direction: Default::default(),
            constraints: Default::default(),
            widget,
            split_style: Default::default(),
            block: Default::default(),
            style: Default::default(),
            drag_style: Default::default(),
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
    pub fn render_split(mut self, split_style: SplitStyle) -> Self {
        self.split_style = split_style;
        self
    }

    /// Border block.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Style for the split area.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Style while dragging the splitter.
    pub fn drag_style(mut self, style: Style) -> Self {
        self.drag_style = Some(style);
        self
    }
}

impl<'a, W> Split<'a, W> {
    /// Calculates the first layout according to the constraints.
    /// When a resize is detected, the current area-width/height is used as
    /// Fill() constraint for the new layout.
    fn layout<S>(&self, area: Rect, state: &mut SplitState<S>) {
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
                    SplitStyle::Full => {
                        width += state.split.iter().map(|v| v.width).sum::<u16>();
                    }
                    SplitStyle::Minimal => {}
                    SplitStyle::None => {}
                }
                if area.width != width {
                    let mut c = Vec::new();
                    for i in 0..state.areas.len() {
                        match self.split_style {
                            SplitStyle::Full => {
                                if i < state.split.len() {
                                    c.push(Constraint::Fill(
                                        state.areas[i].width + state.split[i].width,
                                    ));
                                } else {
                                    c.push(Constraint::Fill(state.areas[i].width));
                                }
                            }
                            SplitStyle::Minimal => {
                                c.push(Constraint::Fill(state.areas[i].width));
                            }
                            SplitStyle::None => {
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
                    SplitStyle::Full => {
                        height += state.split.iter().map(|v| v.height).sum::<u16>();
                    }
                    SplitStyle::Minimal => {}
                    SplitStyle::None => {}
                }
                if area.height != height {
                    let mut c = Vec::new();
                    for i in 0..state.areas.len() {
                        match self.split_style {
                            SplitStyle::Full => {
                                if i < state.split.len() {
                                    c.push(Constraint::Fill(
                                        state.areas[i].height + state.split[i].height,
                                    ));
                                } else {
                                    c.push(Constraint::Fill(state.areas[i].height));
                                }
                            }
                            SplitStyle::Minimal => {
                                c.push(Constraint::Fill(state.areas[i].height));
                            }
                            SplitStyle::None => {
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
                        SplitStyle::Full => (
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
                        SplitStyle::Minimal => (
                            Rect::new(area.x, area.y, area.width, area.height),
                            Rect::new(
                                area.x + area.width.saturating_sub(SPLIT_WIDTH),
                                area.y,
                                1,
                                2,
                            ),
                        ),
                        SplitStyle::None => (
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
                        SplitStyle::Full => (
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
                        SplitStyle::Minimal => (
                            Rect::new(area.x, area.y, area.width, area.height),
                            Rect::new(
                                area.x,
                                area.y + area.height.saturating_sub(SPLIT_WIDTH),
                                2,
                                1,
                            ),
                        ),
                        SplitStyle::None => (
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
    }
}

impl<'a, W, S> StatefulWidget for Split<'a, W>
where
    W: SplitWidget<State = S>,
{
    type State = SplitState<S>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.direction = self.direction;
        state.area = area;
        state.inner = self.block.inner_if_some(area);
        state.split_style = self.split_style;

        self.layout(state.inner, state);

        self.block.render_ref(area, buf);

        for (n, a) in state.areas.iter().enumerate() {
            self.widget.render(n, *a, buf, &mut state.w);
        }

        if matches!(self.split_style, SplitStyle::Full | SplitStyle::Minimal) {
            for (n, split_area) in state.split.iter().enumerate() {
                let style = if Some(n) == state.mouse.drag.get() || state.key_nav.is_some() {
                    if let Some(drag) = self.drag_style {
                        drag
                    } else {
                        revert_style(self.style)
                    }
                } else {
                    self.style
                };

                buf.set_style(*split_area, style);

                let (x, y) = (split_area.x, split_area.y);
                if self.direction == Direction::Horizontal {
                    if buf.area.contains((x, y).into()) {
                        buf.get_mut(x, y).set_symbol("<");
                    }
                    if buf.area.contains((x, y + 1).into()) {
                        buf.get_mut(x, y + 1).set_symbol(">");
                    }
                } else {
                    if buf.area.contains((x, y).into()) {
                        buf.get_mut(x, y).set_symbol("\u{21C5}");
                    }
                }
            }
        }
    }
}

impl<S> SplitState<S> {
    /// Set the position for the nth splitter.
    ///
    /// The position is limited the combined area of the two adjacent areas.
    pub fn set_screen_split_pos(&mut self, n: usize, pos: (u16, u16)) -> bool {
        let area1 = self.areas[n];
        let area2 = self.areas[n + 1];
        let area = area1.union(area2);

        if self.direction == Direction::Horizontal {
            let p = if pos.0 < area.left() {
                area.left()
            } else if pos.0 >= area.right() {
                area.right()
            } else {
                pos.0
            };

            match self.split_style {
                SplitStyle::Full => {
                    self.areas[n] = Rect::new(area1.x, area1.y, p - area1.x, area1.height);
                    self.split[n] = Rect::new(p, area1.y, 1, area1.height);
                    self.areas[n + 1] = Rect::new(p + 1, area2.y, area2.right() - p, area2.height);
                }
                SplitStyle::Minimal => {
                    self.areas[n] = Rect::new(area1.x, area1.y, (p + 1) - area1.x, area1.height);
                    self.split[n] = Rect::new(p, area1.y, 1, 2);
                    self.areas[n + 1] = Rect::new(p + 1, area2.y, area2.right() - p, area2.height);
                }
                SplitStyle::None => {
                    self.areas[n] = Rect::new(area1.x, area1.y, (p + 1) - area1.x, area1.height);
                    self.split[n] = Rect::new(p, area1.y, 1, area1.height);
                    self.areas[n + 1] = Rect::new(p + 1, area2.y, area2.right() - p, area2.height);
                }
            }
        } else {
            let p = if pos.1 < area.top() {
                area.top()
            } else if pos.1 >= area.bottom() {
                area.bottom()
            } else {
                pos.1
            };

            match self.split_style {
                SplitStyle::Full => {
                    self.areas[n] = Rect::new(area1.x, area1.y, area1.width, p - area1.y);
                    self.split[n] = Rect::new(area1.x, p, area1.width, 1);
                    self.areas[n + 1] = Rect::new(area2.x, p + 1, area2.width, area2.bottom() - p);
                }
                SplitStyle::Minimal => {
                    self.areas[n] = Rect::new(area1.x, area1.y, area1.width, (p + 1) - area1.y);
                    self.split[n] = Rect::new(area1.x, p, 2, 1);
                    self.areas[n + 1] = Rect::new(area2.x, p + 1, area2.width, area2.bottom() - p);
                }
                SplitStyle::None => {
                    self.areas[n] = Rect::new(area1.x, area1.y, area1.width, (p + 1) - area1.y);
                    self.split[n] = Rect::new(area1.x, p, area1.width, 1);
                    self.areas[n + 1] = Rect::new(area2.x, p + 1, area2.width, area2.bottom() - p);
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
}

impl<Q, R, S> HandleEvent<crossterm::event::Event, Q, R> for SplitState<S>
where
    S: HandleEvent<crossterm::event::Event, Q, R>,
    R: ConsumedEvent + From<Outcome>,
{
    fn handle(&mut self, event: &crossterm::event::Event, qualifier: Q) -> R {
        flow!(match event {
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
            _ => Outcome::NotUsed.into(),
        });

        if let Some(n) = self.key_nav {
            flow!(match event {
                ct_event!(keycode press Left) => self.move_split_left(n, 1).into(),
                ct_event!(keycode press Right) => self.move_split_right(n, 1).into(),
                ct_event!(keycode press Up) => self.move_split_up(n, 1).into(),
                ct_event!(keycode press Down) => self.move_split_down(n, 1).into(),
                ct_event!(keycode press Enter) | ct_event!(keycode press Esc) => {
                    self.key_nav = None;
                    Outcome::Changed
                }
                _ => Outcome::Unchanged, // todo: too crude??
            });
        }

        self.w.handle(event, qualifier)
    }
}
