use crate::event::ScrollOutcome;
use crate::{Scroll, ScrollState, ScrollbarPolicy};
use rat_event::{ct_event, flow, HandleEvent, MouseOnly};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Style;
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::StatefulWidgetRef;
use ratatui::widgets::{Block, Padding, ScrollbarOrientation, StatefulWidget, Widget};
use std::cmp::max;

/// Utility widget for layout/rendering the combined block and scrollbars.
///
/// It can calculate the layout for any combination and layouts the
/// scrollbars on top of the border if one exists.
#[derive(Debug, Default, Clone)]
pub struct ScrollArea<'a> {
    style: Style,
    block: Option<&'a Block<'a>>,
    h_scroll: Option<&'a Scroll<'a>>,
    v_scroll: Option<&'a Scroll<'a>>,
}

/// Temporary state for ScrollArea.
///
/// This state is not meant to keep, it just packages the widgets state
/// for use by ScrollArea.
#[derive(Debug, Default)]
pub struct ScrollAreaState<'a> {
    /// This area is only used for event-handling.
    /// Populate before calling the event-handler.
    area: Rect,
    /// Horizontal scroll state.
    h_scroll: Option<&'a mut ScrollState>,
    /// Vertical scroll state.
    v_scroll: Option<&'a mut ScrollState>,
}

impl<'a> ScrollArea<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the base style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the block.
    pub fn block(mut self, block: Option<&'a Block<'a>>) -> Self {
        self.block = block;
        self
    }

    /// Sets the horizontal scroll.
    pub fn h_scroll(mut self, scroll: Option<&'a Scroll<'a>>) -> Self {
        self.h_scroll = scroll;
        self
    }

    /// Sets the vertical scroll.
    pub fn v_scroll(mut self, scroll: Option<&'a Scroll<'a>>) -> Self {
        self.v_scroll = scroll;
        self
    }

    /// What padding does this effect.
    pub fn padding(&self) -> Padding {
        let mut padding = block_padding(&self.block);
        if let Some(h_scroll) = self.h_scroll {
            let scroll_pad = h_scroll.padding();
            padding.top = max(padding.top, scroll_pad.top);
            padding.bottom = max(padding.bottom, scroll_pad.bottom);
        }
        if let Some(v_scroll) = self.v_scroll {
            let scroll_pad = v_scroll.padding();
            padding.left = max(padding.left, scroll_pad.left);
            padding.right = max(padding.right, scroll_pad.right);
        }
        padding
    }

    /// Calculate the size of the inner area.
    pub fn inner(
        &self,
        area: Rect,
        hscroll_state: Option<&ScrollState>,
        vscroll_state: Option<&ScrollState>,
    ) -> Rect {
        layout(
            self.block,
            self.h_scroll,
            self.v_scroll,
            area,
            hscroll_state,
            vscroll_state,
        )
        .0
    }
}

/// Get the padding the block imposes as Padding.
fn block_padding(block: &Option<&Block<'_>>) -> Padding {
    let area = Rect::new(0, 0, 20, 20);
    let inner = if let Some(block) = block {
        block.inner(area)
    } else {
        area
    };
    Padding {
        left: inner.left() - area.left(),
        right: area.right() - inner.right(),
        top: inner.top() - area.top(),
        bottom: area.bottom() - inner.bottom(),
    }
}

impl<'a> StatefulWidget for ScrollArea<'a> {
    type State = ScrollAreaState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_scroll_area(&self, area, buf, state);
    }
}

#[cfg(feature = "unstable-widget-ref")]
impl<'a> StatefulWidgetRef for ScrollArea<'a> {
    type State = ScrollAreaState<'a>;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_scroll_area(self, area, buf, state);
    }
}

fn render_scroll_area(
    widget: &ScrollArea<'_>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut ScrollAreaState<'_>,
) {
    let (_, hscroll_area, vscroll_area) = layout(
        widget.block,
        widget.h_scroll,
        widget.v_scroll,
        area,
        state.h_scroll.as_deref(),
        state.v_scroll.as_deref(),
    );

    if let Some(block) = widget.block {
        block.render(area, buf);
    } else {
        buf.set_style(area, widget.style);
    }
    if let Some(h) = widget.h_scroll {
        if let Some(hstate) = &mut state.h_scroll {
            h.render(hscroll_area, buf, hstate);
        } else {
            panic!("no horizontal scroll state");
        }
    }
    if let Some(v) = widget.v_scroll {
        if let Some(vstate) = &mut state.v_scroll {
            v.render(vscroll_area, buf, vstate)
        } else {
            panic!("no vertical scroll state");
        }
    }
}

/// Calculate the layout for the given scrollbars.
/// This prevents overlaps in the corners, if both scrollbars are
/// visible, and tries to fit in the given block.
///
/// Returns (inner, h_area, v_area).
///
/// __Panic__
///
/// Panics if the orientation doesn't match,
/// - h_scroll doesn't accept ScrollBarOrientation::Vertical* and
/// - v_scroll doesn't accept ScrollBarOrientation::Horizontal*.
///
/// __Panic__
///
/// if the state doesn't contain the necessary scroll-states.
fn layout<'a>(
    block: Option<&Block<'a>>,
    hscroll: Option<&Scroll<'a>>,
    vscroll: Option<&Scroll<'a>>,
    area: Rect,
    hscroll_state: Option<&ScrollState>,
    vscroll_state: Option<&ScrollState>,
) -> (Rect, Rect, Rect) {
    let mut inner = area;

    if let Some(block) = block {
        inner = block.inner(area);
    }

    if let Some(hscroll) = hscroll {
        if let Some(hscroll_state) = hscroll_state {
            let show = match hscroll.get_policy() {
                ScrollbarPolicy::Always => true,
                ScrollbarPolicy::Minimize => true,
                ScrollbarPolicy::Collapse => hscroll_state.max_offset > 0,
            };
            if show {
                match hscroll.get_orientation() {
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
        } else {
            panic!("no horizontal scroll state");
        }
    }

    if let Some(vscroll) = vscroll {
        if let Some(vscroll_state) = vscroll_state {
            let show = match vscroll.get_policy() {
                ScrollbarPolicy::Always => true,
                ScrollbarPolicy::Minimize => true,
                ScrollbarPolicy::Collapse => vscroll_state.max_offset > 0,
            };
            if show {
                match vscroll.get_orientation() {
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
        } else {
            panic!("no horizontal scroll state");
        }
    }

    // horizontal
    let h_area = if let Some(hscroll) = hscroll {
        if let Some(hscroll_state) = hscroll_state {
            let show = match hscroll.get_policy() {
                ScrollbarPolicy::Always => true,
                ScrollbarPolicy::Minimize => true,
                ScrollbarPolicy::Collapse => hscroll_state.max_offset > 0,
            };
            if show {
                match hscroll.get_orientation() {
                    ScrollbarOrientation::HorizontalBottom => Rect::new(
                        inner.x + hscroll.get_start_margin(),
                        area.bottom().saturating_sub(1),
                        inner
                            .width
                            .saturating_sub(hscroll.get_start_margin() + hscroll.get_end_margin()),
                        if area.height > 0 { 1 } else { 0 },
                    ),
                    ScrollbarOrientation::HorizontalTop => Rect::new(
                        inner.x + hscroll.get_start_margin(),
                        area.y,
                        inner
                            .width
                            .saturating_sub(hscroll.get_start_margin() + hscroll.get_end_margin()),
                        if area.height > 0 { 1 } else { 0 },
                    ),
                    _ => unreachable!(),
                }
            } else {
                Rect::new(area.x, area.y, 0, 0)
            }
        } else {
            panic!("no horizontal scroll state");
        }
    } else {
        Rect::new(area.x, area.y, 0, 0)
    };

    // vertical
    let v_area = if let Some(vscroll) = vscroll {
        if let Some(vscroll_state) = vscroll_state {
            let show = match vscroll.get_policy() {
                ScrollbarPolicy::Always => true,
                ScrollbarPolicy::Minimize => true,
                ScrollbarPolicy::Collapse => vscroll_state.max_offset > 0,
            };
            if show {
                match vscroll.get_orientation() {
                    ScrollbarOrientation::VerticalRight => Rect::new(
                        area.right().saturating_sub(1),
                        inner.y + vscroll.get_start_margin(),
                        if area.width > 0 { 1 } else { 0 },
                        inner
                            .height
                            .saturating_sub(vscroll.get_start_margin() + vscroll.get_end_margin()),
                    ),
                    ScrollbarOrientation::VerticalLeft => Rect::new(
                        area.x,
                        inner.y + vscroll.get_start_margin(),
                        if area.width > 0 { 1 } else { 0 },
                        inner
                            .height
                            .saturating_sub(vscroll.get_start_margin() + vscroll.get_end_margin()),
                    ),
                    _ => unreachable!(),
                }
            } else {
                Rect::new(area.x, area.y, 0, 0)
            }
        } else {
            panic!("no horizontal scroll state");
        }
    } else {
        Rect::new(area.x, area.y, 0, 0)
    };

    (inner, h_area, v_area)
}

impl<'a> ScrollAreaState<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn area(mut self, area: Rect) -> Self {
        self.area = area;
        self
    }

    pub fn v_scroll(mut self, v_scroll: &'a mut ScrollState) -> Self {
        self.v_scroll = Some(v_scroll);
        self
    }

    pub fn v_scroll_opt(mut self, v_scroll: Option<&'a mut ScrollState>) -> Self {
        self.v_scroll = v_scroll;
        self
    }

    pub fn h_scroll(mut self, h_scroll: &'a mut ScrollState) -> Self {
        self.h_scroll = Some(h_scroll);
        self
    }

    pub fn h_scroll_opt(mut self, h_scroll: Option<&'a mut ScrollState>) -> Self {
        self.h_scroll = h_scroll;
        self
    }
}

///
/// Handle scrolling for the whole area spanned by the two scroll-states.
///
impl HandleEvent<crossterm::event::Event, MouseOnly, ScrollOutcome> for ScrollAreaState<'_> {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> ScrollOutcome {
        if let Some(h_scroll) = &mut self.h_scroll {
            flow!(match event {
                // right scroll with ALT down. shift doesn't work?
                ct_event!(scroll ALT down for column, row) => {
                    if self.area.contains(Position::new(*column, *row)) {
                        ScrollOutcome::Right(h_scroll.scroll_by())
                    } else {
                        ScrollOutcome::Continue
                    }
                }
                // left scroll with ALT up. shift doesn't work?
                ct_event!(scroll ALT up for column, row) => {
                    if self.area.contains(Position::new(*column, *row)) {
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
                    if self.area.contains(Position::new(*column, *row)) {
                        ScrollOutcome::Down(v_scroll.scroll_by())
                    } else {
                        ScrollOutcome::Continue
                    }
                }
                ct_event!(scroll up for column, row) => {
                    if self.area.contains(Position::new(*column, *row)) {
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
