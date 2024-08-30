use crate::_private::NonExhaustive;
use crate::event::ScrollOutcome;
use crate::ScrollbarPolicy;
use rat_event::util::MouseFlags;
use rat_event::{ct_event, HandleEvent, MouseOnly};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Style;
use ratatui::symbols::scrollbar::Set;
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::StatefulWidgetRef;
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget};
use std::cmp::{max, min};

/// Scroll widget.
///
/// This is not a widget by itself, rather it is meant to be used
/// analogous to Block. A widget that supports scrolling accepts
/// one or two of these Scroll indicators.
#[derive(Debug, Default, Clone)]
pub struct Scroll<'a> {
    policy: ScrollbarPolicy,
    orientation: ScrollbarOrientation,

    start_margin: u16,
    end_margin: u16,
    overscroll_by: Option<usize>,
    scroll_by: Option<usize>,

    scrollbar: Scrollbar<'a>,

    min_symbol: Option<&'a str>,
    min_style: Option<Style>,
}

/// Scroll state.
///
/// The current visible page is represented as the pair (offset, page_len).
///
/// The limit for scrolling is given as max_offset, which is the maximum offset
/// where a full page can still be displayed.
///
/// __Note__ that the total length of the widgets data is NOT max_offset + page_len.
/// The page_len can be different for every offset selected. Only
/// if the offset is set to max_offset and after the next round of rendering
/// len == max_offset + page_len will hold true.
///
/// __Note__ In terms of ScrollbarState, offset is position,
/// page_len is viewport_content_length and max_offset is content_length.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrollState {
    /// Area of the Scrollbar.
    pub area: Rect,
    /// Vertical/Horizontal scroll?
    pub orientation: ScrollbarOrientation,

    /// Current offset.
    pub offset: usize,
    /// Page-size at the current offset.
    pub page_len: usize,
    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is shorter than the length of the content by whatever fills the last page.
    /// This is the base for the scrollbar content_length.
    pub max_offset: usize,

    /// Scrolling step-size for mouse-scrolling
    pub scroll_by: Option<usize>,
    /// Allow overscroll by n items.
    pub overscroll_by: Option<usize>,

    /// Mouse support.
    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

/// Collected styles for the Scroll.
#[derive(Debug, Clone)]
pub struct ScrollStyle {
    pub thumb_style: Option<Style>,
    pub thumb_symbol: Option<&'static str>,
    pub track_style: Option<Style>,
    pub track_symbol: Option<&'static str>,
    pub begin_style: Option<Style>,
    pub begin_symbol: Option<&'static str>,
    pub end_style: Option<Style>,
    pub end_symbol: Option<&'static str>,
    /// Style used when the scrollbar is not needed.
    pub min_style: Option<Style>,
    /// Symbol used instead of the scrollbar when it's not needed.
    pub min_symbol: Option<&'static str>,

    pub non_exhaustive: NonExhaustive,
}

impl Default for ScrollStyle {
    fn default() -> Self {
        Self {
            thumb_style: None,
            thumb_symbol: None,
            track_style: None,
            track_symbol: None,
            begin_style: None,
            begin_symbol: None,
            end_style: None,
            end_symbol: None,
            min_style: None,
            min_symbol: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a> Scroll<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Scrollbar policy.
    pub fn policy(mut self, policy: ScrollbarPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Scrollbar policy.
    pub fn get_policy(&self) -> ScrollbarPolicy {
        self.policy
    }

    /// Scrollbar orientation.
    pub fn orientation(mut self, pos: ScrollbarOrientation) -> Self {
        self.orientation = pos;
        self
    }

    /// Scrollbar orientation.
    pub fn get_orientation(&self) -> ScrollbarOrientation {
        self.orientation.clone()
    }

    /// Ensures a vertical orientation of the scrollbar.
    ///
    /// If the orientation is not vertical it will be set to VerticalRight.
    pub fn override_vertical(mut self) -> Self {
        self.orientation = match self.orientation {
            ScrollbarOrientation::VerticalRight => ScrollbarOrientation::VerticalRight,
            ScrollbarOrientation::VerticalLeft => ScrollbarOrientation::VerticalLeft,
            ScrollbarOrientation::HorizontalBottom => ScrollbarOrientation::VerticalRight,
            ScrollbarOrientation::HorizontalTop => ScrollbarOrientation::VerticalRight,
        };
        self
    }

    /// Ensures a horizontal orientation of the scrollbar.
    ///
    /// If the orientation is not horizontal, it will be set to HorizontalBottom.
    pub fn override_horizontal(mut self) -> Self {
        self.orientation = match self.orientation {
            ScrollbarOrientation::VerticalRight => ScrollbarOrientation::HorizontalBottom,
            ScrollbarOrientation::VerticalLeft => ScrollbarOrientation::HorizontalBottom,
            ScrollbarOrientation::HorizontalBottom => ScrollbarOrientation::HorizontalBottom,
            ScrollbarOrientation::HorizontalTop => ScrollbarOrientation::HorizontalTop,
        };
        self
    }

    /// Is this a vertical scrollbar.
    pub fn is_vertical(&self) -> bool {
        match self.orientation {
            ScrollbarOrientation::VerticalRight => true,
            ScrollbarOrientation::VerticalLeft => true,
            ScrollbarOrientation::HorizontalBottom => false,
            ScrollbarOrientation::HorizontalTop => false,
        }
    }

    /// Is this a horizontal scrollbar.
    pub fn is_horizontal(&self) -> bool {
        match self.orientation {
            ScrollbarOrientation::VerticalRight => false,
            ScrollbarOrientation::VerticalLeft => false,
            ScrollbarOrientation::HorizontalBottom => true,
            ScrollbarOrientation::HorizontalTop => true,
        }
    }

    /// Leave a margin at the start of the scrollbar.
    pub fn start_margin(mut self, start_margin: u16) -> Self {
        self.start_margin = start_margin;
        self
    }

    /// Margin before the start of the scrollbar.
    pub fn get_start_margin(&self) -> u16 {
        self.start_margin
    }

    /// Leave a margin at the end of the scrollbar.
    pub fn end_margin(mut self, end_margin: u16) -> Self {
        self.end_margin = end_margin;
        self
    }

    /// Margin after the end of the scrollbar.
    pub fn get_end_margin(&self) -> u16 {
        self.end_margin
    }

    /// Set overscrolling to this value.
    pub fn overscroll_by(mut self, overscroll: usize) -> Self {
        self.overscroll_by = Some(overscroll);
        self
    }

    /// Set scroll increment.
    pub fn scroll_by(mut self, scroll: usize) -> Self {
        self.scroll_by = Some(scroll);
        self
    }

    /// Set all styles.
    pub fn styles(mut self, styles: ScrollStyle) -> Self {
        if let Some(thumb_symbol) = styles.thumb_symbol {
            self.scrollbar = self.scrollbar.thumb_symbol(thumb_symbol);
        }
        if let Some(thumb_style) = styles.thumb_style {
            self.scrollbar = self.scrollbar.thumb_style(thumb_style);
        }
        if styles.track_symbol.is_some() {
            self.scrollbar = self.scrollbar.track_symbol(styles.track_symbol);
        }
        if let Some(track_style) = styles.track_style {
            self.scrollbar = self.scrollbar.track_style(track_style);
        }
        if styles.begin_symbol.is_some() {
            self.scrollbar = self.scrollbar.begin_symbol(styles.begin_symbol);
        }
        if let Some(begin_style) = styles.begin_style {
            self.scrollbar = self.scrollbar.begin_style(begin_style);
        }
        if styles.end_symbol.is_some() {
            self.scrollbar = self.scrollbar.end_symbol(styles.end_symbol);
        }
        if let Some(end_style) = styles.end_style {
            self.scrollbar = self.scrollbar.end_style(end_style);
        }
        if styles.min_symbol.is_some() {
            self.min_symbol = styles.min_symbol;
        }
        if styles.min_style.is_some() {
            self.min_style = styles.min_style;
        }
        self
    }

    /// Symbol for the Scrollbar.
    pub fn thumb_symbol(mut self, thumb_symbol: &'a str) -> Self {
        self.scrollbar = self.scrollbar.thumb_symbol(thumb_symbol);
        self
    }

    /// Style for the Scrollbar.
    pub fn thumb_style<S: Into<Style>>(mut self, thumb_style: S) -> Self {
        self.scrollbar = self.scrollbar.thumb_style(thumb_style);
        self
    }

    /// Symbol for the Scrollbar.
    pub fn track_symbol(mut self, track_symbol: Option<&'a str>) -> Self {
        self.scrollbar = self.scrollbar.track_symbol(track_symbol);
        self
    }

    /// Style for the Scrollbar.
    pub fn track_style<S: Into<Style>>(mut self, track_style: S) -> Self {
        self.scrollbar = self.scrollbar.track_style(track_style);
        self
    }

    /// Symbol for the Scrollbar.
    pub fn begin_symbol(mut self, begin_symbol: Option<&'a str>) -> Self {
        self.scrollbar = self.scrollbar.begin_symbol(begin_symbol);
        self
    }

    /// Style for the Scrollbar.
    pub fn begin_style<S: Into<Style>>(mut self, begin_style: S) -> Self {
        self.scrollbar = self.scrollbar.begin_style(begin_style);
        self
    }

    /// Symbol for the Scrollbar.
    pub fn end_symbol(mut self, end_symbol: Option<&'a str>) -> Self {
        self.scrollbar = self.scrollbar.end_symbol(end_symbol);
        self
    }

    /// Style for the Scrollbar.
    pub fn end_style<S: Into<Style>>(mut self, end_style: S) -> Self {
        self.scrollbar = self.scrollbar.end_style(end_style);
        self
    }

    /// Symbol when no Scrollbar is drawn.
    pub fn min_symbol(mut self, min_symbol: Option<&'a str>) -> Self {
        self.min_symbol = min_symbol;
        self
    }

    /// Style when no Scrollbar is drawn.
    pub fn min_style<S: Into<Style>>(mut self, min_style: S) -> Self {
        self.min_style = Some(min_style.into());
        self
    }

    /// Set all Scrollbar symbols.
    pub fn symbols(mut self, symbols: Set) -> Self {
        self.scrollbar = self.scrollbar.symbols(symbols);
        self
    }

    /// Set a style for all Scrollbar styles.
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        let style = style.into();
        self.scrollbar = self.scrollbar.style(style);
        self.min_style = Some(style);
        self
    }
}

impl<'a> Scroll<'a> {
    // create the correct scrollbar widget.
    fn scrollbar(&self) -> Scrollbar<'a> {
        self.scrollbar.clone().orientation(self.orientation.clone())
    }
}

impl<'a> StatefulWidget for Scroll<'a> {
    type State = ScrollState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_scroll(&self, area, buf, state)
    }
}

#[cfg(feature = "unstable-widget-ref")]
impl<'a> StatefulWidgetRef for Scroll<'a> {
    type State = ScrollState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_scroll(self, area, buf, state);
    }
}

fn render_scroll(scroll: &Scroll<'_>, area: Rect, buf: &mut Buffer, state: &mut ScrollState) {
    state.set_orientation(scroll.orientation.clone());
    if scroll.overscroll_by.is_some() {
        state.set_overscroll_by(scroll.overscroll_by);
    }
    if scroll.scroll_by.is_some() {
        state.set_scroll_by(scroll.scroll_by);
    }
    state.area = area;

    if area.is_empty() {
        return;
    }

    if state.max_offset() == 0 {
        match scroll.policy {
            ScrollbarPolicy::Always => {
                scroll.scrollbar().render(
                    area,
                    buf,
                    &mut ScrollbarState::new(state.max_offset())
                        .position(state.offset())
                        .viewport_content_length(state.page_len()),
                );
            }
            ScrollbarPolicy::Minimize => {
                fill(scroll.min_symbol, scroll.min_style, area, buf);
            }
            ScrollbarPolicy::Collapse => {
                // widget renders
            }
        }
    } else {
        scroll.scrollbar().render(
            area,
            buf,
            &mut ScrollbarState::new(state.max_offset())
                .position(state.offset())
                .viewport_content_length(state.page_len()),
        );
    }
}

fn fill(sym: Option<&'_ str>, style: Option<Style>, area: Rect, buf: &mut Buffer) {
    let area = buf.area.intersection(area);
    match (sym, style) {
        (Some(sym), Some(style)) => {
            for y in area.top()..area.bottom() {
                for x in area.left()..area.right() {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        // cell.reset();
                        cell.set_symbol(sym);
                        cell.set_style(style);
                    }
                }
            }
        }
        (None, Some(style)) => {
            for y in area.top()..area.bottom() {
                for x in area.left()..area.right() {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        // cell.reset();
                        cell.set_style(style);
                    }
                }
            }
        }
        (Some(sym), None) => {
            for y in area.top()..area.bottom() {
                for x in area.left()..area.right() {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_symbol(sym);
                    }
                }
            }
        }
        (None, None) => {
            // noop
        }
    }
}

impl Default for ScrollState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            orientation: Default::default(),
            offset: 0,
            max_offset: 0,
            page_len: 0,
            scroll_by: None,
            overscroll_by: None,
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl ScrollState {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn set_orientation(&mut self, orientation: ScrollbarOrientation) {
        self.orientation = orientation;
    }

    /// Vertical scroll?
    #[inline]
    pub fn is_vertical(&self) -> bool {
        self.orientation.is_vertical()
    }

    /// Horizontal scroll?
    #[inline]
    pub fn is_horizontal(&self) -> bool {
        self.orientation.is_horizontal()
    }

    /// Current vertical offset.
    #[inline]
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Change the offset. Limits the offset to max_v_offset + v_overscroll.
    ///
    /// Due to overscroll it's possible that this is an invalid
    /// offset for the widget. The widget must deal with this
    /// situation.
    #[inline]
    pub fn set_offset(&mut self, offset: usize) -> bool {
        let old = self.offset;
        self.offset = self.limit_offset(offset);
        old != self.offset
    }

    /// Scroll to make the given pos visible. Adjusts the
    /// offset just enough to make this happen. Does nothing if
    /// the position is already visible.
    #[inline]
    pub fn scroll_to_pos(&mut self, pos: usize) -> bool {
        let old = self.offset;
        if pos >= self.offset + self.page_len {
            self.offset = pos - self.page_len + 1;
        } else if pos < self.offset {
            self.offset = pos;
        }
        old != self.offset
    }

    /// Scroll up by n.
    #[inline]
    pub fn scroll_up(&mut self, n: usize) -> bool {
        let old = self.offset;
        self.offset = self.limit_offset(self.offset.saturating_sub(n));
        old != self.offset
    }

    /// Scroll down by n.
    #[inline]
    pub fn scroll_down(&mut self, n: usize) -> bool {
        let old = self.offset;
        self.offset = self.limit_offset(self.offset.saturating_add(n));
        old != self.offset
    }

    /// Scroll left by n.
    #[inline]
    pub fn scroll_left(&mut self, n: usize) -> bool {
        self.scroll_up(n)
    }

    /// Scroll right by n.
    #[inline]
    pub fn scroll_right(&mut self, n: usize) -> bool {
        self.scroll_down(n)
    }

    /// Calculate the offset limited to max_offset+overscroll_by.
    #[inline]
    pub fn limit_offset(&self, offset: usize) -> usize {
        min(offset, self.max_offset.saturating_add(self.overscroll_by()))
    }

    /// Calculate the offset limited to max_offset+overscroll_by.
    #[inline]
    pub fn clamp_offset(&self, offset: isize) -> usize {
        offset.clamp(
            0,
            self.max_offset.saturating_add(self.overscroll_by()) as isize,
        ) as usize
    }

    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is shorter than the length of the content by whatever fills the last page.
    /// This is the base for the scrollbar content_length.
    #[inline]
    pub fn max_offset(&self) -> usize {
        self.max_offset
    }

    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is shorter than the length of the content by whatever fills the last page.
    /// This is the base for the scrollbar content_length.
    #[inline]
    pub fn set_max_offset(&mut self, max: usize) {
        self.max_offset = max;
    }

    /// Page-size at the current offset.
    #[inline]
    pub fn page_len(&self) -> usize {
        self.page_len
    }

    /// Page-size at the current offset.
    #[inline]
    pub fn set_page_len(&mut self, page: usize) {
        self.page_len = page;
    }

    /// Suggested scroll per scroll-event.
    /// Defaults to 1/10 of the page
    #[inline]
    pub fn scroll_by(&self) -> usize {
        if let Some(scroll) = self.scroll_by {
            max(scroll, 1)
        } else {
            max(self.page_len / 10, 1)
        }
    }

    /// Suggested scroll per scroll-event.
    /// Defaults to 1/10 of the page
    #[inline]
    pub fn set_scroll_by(&mut self, scroll: Option<usize>) {
        self.scroll_by = scroll;
    }

    /// Allowed overscroll
    #[inline]
    pub fn overscroll_by(&self) -> usize {
        self.overscroll_by.unwrap_or_default()
    }

    /// Allowed overscroll
    #[inline]
    pub fn set_overscroll_by(&mut self, overscroll_by: Option<usize>) {
        self.overscroll_by = overscroll_by;
    }

    /// Update the state to match adding items.
    #[inline]
    pub fn items_added(&mut self, pos: usize, n: usize) {
        if self.offset >= pos {
            self.offset += n;
        }
        self.max_offset += n;
    }

    /// Update the state to match removing items.
    #[inline]
    pub fn items_removed(&mut self, pos: usize, n: usize) {
        if self.offset >= pos {
            self.offset -= n;
        }
        self.max_offset = self.max_offset.saturating_sub(n);
    }
}

impl ScrollState {
    /// Maps a screen-position to an offset.
    /// pos - row/column clicked
    /// base - x/y of the range
    /// length - width/height of the range.
    pub fn map_position_index(&self, pos: u16, base: u16, length: u16) -> usize {
        // correct for the arrows.
        let pos = pos.saturating_sub(base).saturating_sub(1) as usize;
        let span = length.saturating_sub(2) as usize;

        (self.max_offset.saturating_mul(pos)) / span
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, ScrollOutcome> for ScrollState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> ScrollOutcome {
        match event {
            ct_event!(mouse any for m) if self.mouse.drag(self.area, m) => {
                if self.is_vertical() {
                    if m.row >= self.area.y {
                        ScrollOutcome::VPos(self.map_position_index(
                            m.row,
                            self.area.y,
                            self.area.height,
                        ))
                    } else {
                        ScrollOutcome::Unchanged
                    }
                } else {
                    if m.column >= self.area.x {
                        ScrollOutcome::HPos(self.map_position_index(
                            m.column,
                            self.area.x,
                            self.area.width,
                        ))
                    } else {
                        ScrollOutcome::Unchanged
                    }
                }
            }
            ct_event!(mouse down Left for col, row) if self.area.contains((*col, *row).into()) => {
                if self.is_vertical() {
                    ScrollOutcome::VPos(self.map_position_index(
                        *row,
                        self.area.y,
                        self.area.height,
                    ))
                } else {
                    ScrollOutcome::HPos(self.map_position_index(*col, self.area.x, self.area.width))
                }
            }
            ct_event!(scroll down for col, row)
                if self.is_vertical() && self.area.contains((*col, *row).into()) =>
            {
                ScrollOutcome::Down(self.scroll_by())
            }
            ct_event!(scroll up for col, row)
                if self.is_vertical() && self.area.contains((*col, *row).into()) =>
            {
                ScrollOutcome::Up(self.scroll_by())
            }
            // right scroll with ALT down. shift doesn't work?
            ct_event!(scroll ALT down for col, row)
                if self.is_horizontal() && self.area.contains((*col, *row).into()) =>
            {
                ScrollOutcome::Right(self.scroll_by())
            }
            // left scroll with ALT up. shift doesn't work?
            ct_event!(scroll ALT up for col, row)
                if self.is_horizontal() && self.area.contains((*col, *row).into()) =>
            {
                ScrollOutcome::Left(self.scroll_by())
            }
            _ => ScrollOutcome::Continue,
        }
    }
}
