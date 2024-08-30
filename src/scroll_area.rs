use crate::event::ScrollOutcome;
use crate::{Scroll, ScrollState, ScrollbarPolicy};
use rat_event::{ct_event, flow, HandleEvent, MouseOnly};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::widgets::{Block, ScrollbarOrientation, StatefulWidget, Widget};

///
/// Utility widget for layout/rendering the combined block and scrollbars.
///
/// It layouts the scrollbars on top of the border and leaves
/// the corners free if necessary.
///
#[derive(Debug, Default, Clone)]
pub struct ScrollArea<'a> {
    block: Option<Block<'a>>,
    h_scroll: Option<Scroll<'a>>,
    v_scroll: Option<Scroll<'a>>,
}

/// Temporary state for ScrollArea.
///
/// This state is not meant to keep, it just packages the widgets state
/// for use by ScrollArea.
#[derive(Debug)]
pub struct ScrollAreaState<'a> {
    pub area: Rect,
    pub h_scroll: Option<&'a mut ScrollState>,
    pub v_scroll: Option<&'a mut ScrollState>,
}

impl<'a> ScrollArea<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the block.
    pub fn block(mut self, block: Option<Block<'a>>) -> Self {
        self.block = block;
        self
    }

    /// Sets the horizontal scroll.
    pub fn h_scroll(mut self, scroll: Option<Scroll<'a>>) -> Self {
        self.h_scroll = scroll;
        self
    }

    /// Sets the vertical scroll.
    pub fn v_scroll(mut self, scroll: Option<Scroll<'a>>) -> Self {
        self.v_scroll = scroll;
        self
    }
}

impl<'a> ScrollArea<'a> {
    /// Calculates the inner area for the widget.
    ///
    /// __Panic__
    /// Panics if the orientation doesn't match,
    /// h_scroll doesn't accept ScrollBarOrientation::Vertical* and
    /// v_scroll doesn't accept ScrollBarOrientation::Horizontal*.
    ///
    /// Panics if the state doesn't contain the necessary scroll-states.
    pub fn inner(&self, area: Rect, state: ScrollAreaState<'_>) -> Rect {
        self.layout(area, &state).0
    }
}

impl<'a> ScrollArea<'a> {
    /// Calculate the layout for the given scrollbars.
    /// This prevents overlaps in the corners, if both scrollbars are
    /// visible, and tries to fit in the given block.
    ///
    /// Returns (inner, h_area, v_area).
    ///
    /// __Panic__
    /// Panics if the orientation doesn't match,
    /// h_scroll doesn't accept ScrollBarOrientation::Vertical* and
    /// v_scroll doesn't accept ScrollBarOrientation::Horizontal*.
    ///
    /// Panics if the state doesn't contain the necessary scroll-states.
    fn layout(&self, area: Rect, state: &ScrollAreaState<'_>) -> (Rect, Rect, Rect) {
        let mut inner = area;

        if let Some(block) = &self.block {
            inner = block.inner(area);
        }

        if let Some(h_scroll) = &self.h_scroll {
            let state = state.h_scroll.as_deref().expect("h_scroll state");
            let show = match h_scroll.get_policy() {
                ScrollbarPolicy::Always => true,
                ScrollbarPolicy::Minimize => true,
                ScrollbarPolicy::Collapse => state.max_offset > 0,
            };
            if show {
                match h_scroll.get_orientation() {
                    ScrollbarOrientation::VerticalRight => {
                        unimplemented!(
                        "ScrollbarOrientation::VerticalRight not supported for horizontal scrolling."
                    );
                    }
                    ScrollbarOrientation::VerticalLeft => {
                        unimplemented!(
                            "ScrollbarOrientation::VerticalLeft not supported for horizontal scrolling."
                        );
                    }
                    ScrollbarOrientation::HorizontalBottom => {
                        if inner.bottom() == area.bottom() {
                            inner.height = inner.height.saturating_sub(1);
                        }
                    }
                    ScrollbarOrientation::HorizontalTop => {
                        if inner.top() == area.top() {
                            inner.y += 1;
                            inner.height = inner.height.saturating_sub(1);
                        }
                    }
                }
            }
        }

        if let Some(v_scroll) = &self.v_scroll {
            let state = state.v_scroll.as_deref().expect("v_scroll state");
            let show = match v_scroll.get_policy() {
                ScrollbarPolicy::Always => true,
                ScrollbarPolicy::Minimize => true,
                ScrollbarPolicy::Collapse => state.max_offset > 0,
            };
            if show {
                match v_scroll.get_orientation() {
                    ScrollbarOrientation::VerticalRight => {
                        if inner.right() == area.right() {
                            inner.width = inner.width.saturating_sub(1);
                        }
                    }
                    ScrollbarOrientation::VerticalLeft => {
                        if inner.left() == area.left() {
                            inner.x += 1;
                            inner.width = inner.width.saturating_sub(1);
                        }
                    }
                    ScrollbarOrientation::HorizontalBottom => {
                        unimplemented!(
                            "ScrollbarOrientation::HorizontalBottom not supported for vertical scrolling."
                        );
                    }
                    ScrollbarOrientation::HorizontalTop => {
                        unimplemented!(
                            "ScrollbarOrientation::HorizontalTop not supported for vertical scrolling."
                        );
                    }
                }
            }
        }

        // horizontal
        let h_area = if let Some(h_scroll) = &self.h_scroll {
            let state = state.h_scroll.as_deref().expect("h_scroll state");
            let show = match h_scroll.get_policy() {
                ScrollbarPolicy::Always => true,
                ScrollbarPolicy::Minimize => true,
                ScrollbarPolicy::Collapse => state.max_offset > 0,
            };
            if show {
                match h_scroll.get_orientation() {
                    ScrollbarOrientation::HorizontalBottom => Rect::new(
                        inner.x + h_scroll.get_start_margin(),
                        area.bottom().saturating_sub(1),
                        inner.width.saturating_sub(
                            h_scroll.get_start_margin() + h_scroll.get_end_margin(),
                        ),
                        if area.height > 0 { 1 } else { 0 },
                    ),
                    ScrollbarOrientation::HorizontalTop => Rect::new(
                        inner.x + h_scroll.get_start_margin(),
                        area.y,
                        inner.width.saturating_sub(
                            h_scroll.get_start_margin() + h_scroll.get_end_margin(),
                        ),
                        if area.height > 0 { 1 } else { 0 },
                    ),
                    _ => unreachable!(),
                }
            } else {
                Rect::new(area.x, area.y, 0, 0)
            }
        } else {
            Rect::new(area.x, area.y, 0, 0)
        };

        // vertical
        let v_area = if let Some(v_scroll) = &self.v_scroll {
            let state = state.v_scroll.as_deref().expect("v_scroll state");
            let show = match v_scroll.get_policy() {
                ScrollbarPolicy::Always => true,
                ScrollbarPolicy::Minimize => true,
                ScrollbarPolicy::Collapse => state.max_offset > 0,
            };
            if show {
                match v_scroll.get_orientation() {
                    ScrollbarOrientation::VerticalRight => Rect::new(
                        area.right().saturating_sub(1),
                        inner.y + v_scroll.get_start_margin(),
                        if area.width > 0 { 1 } else { 0 },
                        inner.height.saturating_sub(
                            v_scroll.get_start_margin() + v_scroll.get_end_margin(),
                        ),
                    ),
                    ScrollbarOrientation::VerticalLeft => Rect::new(
                        area.x,
                        inner.y + v_scroll.get_start_margin(),
                        if area.width > 0 { 1 } else { 0 },
                        inner.height.saturating_sub(
                            v_scroll.get_start_margin() + v_scroll.get_end_margin(),
                        ),
                    ),
                    _ => unreachable!(),
                }
            } else {
                Rect::new(area.x, area.y, 0, 0)
            }
        } else {
            Rect::new(area.x, area.y, 0, 0)
        };

        (inner, h_area, v_area)
    }
}

impl<'a> StatefulWidget for ScrollArea<'a> {
    type State = ScrollAreaState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let (_inner, h_scroll, v_scroll) = self.layout(area, state);

        if let Some(block) = self.block {
            block.render(area, buf);
        }
        if let Some(h) = self.h_scroll {
            h.render(
                h_scroll,
                buf,
                state.h_scroll.as_deref_mut().expect("h_scroll state"),
            );
        }
        if let Some(v) = self.v_scroll {
            v.render(
                v_scroll,
                buf,
                state.v_scroll.as_deref_mut().expect("v_scroll state"),
            )
        }
    }
}

///
/// Handle scrolling for the whole area spanned by the two scroll-states.
///
impl<'a> HandleEvent<crossterm::event::Event, MouseOnly, ScrollOutcome> for ScrollAreaState<'a> {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> ScrollOutcome {
        let area = self.area;

        if let Some(h_scroll) = &mut self.h_scroll {
            flow!(match event {
                // right scroll with ALT down. shift doesn't work?
                ct_event!(scroll ALT down for column, row) => {
                    if area.contains(Position::new(*column, *row)) {
                        ScrollOutcome::Right(h_scroll.scroll_by())
                    } else {
                        ScrollOutcome::Continue
                    }
                }
                // left scroll with ALT up. shift doesn't work?
                ct_event!(scroll ALT up for column, row) => {
                    if area.contains(Position::new(*column, *row)) {
                        ScrollOutcome::Left(h_scroll.scroll_by())
                    } else {
                        ScrollOutcome::Continue
                    }
                }
                _ => ScrollOutcome::Continue,
            });
            flow!(h_scroll.handle(event, MouseOnly));
        }
        if let Some(v_scroll) = &mut self.v_scroll {
            flow!(match event {
                ct_event!(scroll down for column, row) => {
                    if area.contains(Position::new(*column, *row)) {
                        ScrollOutcome::Down(v_scroll.scroll_by())
                    } else {
                        ScrollOutcome::Continue
                    }
                }
                ct_event!(scroll up for column, row) => {
                    if area.contains(Position::new(*column, *row)) {
                        ScrollOutcome::Up(v_scroll.scroll_by())
                    } else {
                        ScrollOutcome::Continue
                    }
                }
                _ => ScrollOutcome::Continue,
            });
            flow!(v_scroll.handle(event, MouseOnly));
        }

        ScrollOutcome::Continue
    }
}
