use crate::_private::NonExhaustive;
use crate::event::ScrollOutcome;
use rat_event::util::MouseFlags;
use rat_event::{ct_event, flow, HandleEvent, MouseOnly};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::prelude::Style;
use ratatui::symbols::scrollbar::Set;
use ratatui::widgets::{
    Block, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, StatefulWidgetRef,
};
use std::cmp::{max, min};

/// Scrolling indicator.
///
/// Meant to be used like Block.
#[derive(Debug, Default, Clone)]
pub struct Scroll<'a> {
    policy: ScrollbarPolicy,
    orientation: ScrollbarOrientation,
    overscroll: usize,

    thumb_symbol: Option<&'a str>,
    thumb_style: Option<Style>,
    track_symbol: Option<&'a str>,
    track_style: Option<Style>,
    begin_symbol: Option<&'a str>,
    begin_style: Option<Style>,
    end_symbol: Option<&'a str>,
    end_style: Option<Style>,
    no_symbol: Option<&'a str>,
    no_style: Option<Style>,
}

/// Holds the Scrolled-State.
///
/// The current visible page is represented as the pair (offset, page_len).
/// The limit for scrolling is given as max_offset, which is the maximum offset
/// where a full page can still be displayed. Note that the total length of
/// the widgets data is NOT max_offset + page_len. The page_len can be different for
/// every offset selected. Only if the offset is set to max_offset and after
/// the next round of rendering len == max_offset + page_len will hold true.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrollState {
    pub area: Rect,

    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is shorter than the length of the content by whatever fills the last page.
    /// This is the base for the scrollbar content_length.
    pub max_offset: usize,
    /// Current  offset.
    pub offset: usize,
    /// Page-size at the current offset.
    pub page_len: usize,
    /// Scrolling step-size for mouse-scrolling
    pub scroll_by: Option<usize>,
    /// Allow overscroll by n items.
    pub overscroll_by: usize,

    pub mouse: MouseFlags,
    pub orientation: ScrollbarOrientation,

    pub non_exhaustive: NonExhaustive,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarPolicy {
    Always,
    #[default]
    AsNeeded,
}

#[derive(Debug, Clone)]
pub struct ScrolledStyle {
    pub thumb_style: Option<Style>,
    pub track_symbol: Option<&'static str>,
    pub track_style: Option<Style>,
    pub begin_symbol: Option<&'static str>,
    pub begin_style: Option<Style>,
    pub end_symbol: Option<&'static str>,
    pub end_style: Option<Style>,
    pub no_symbol: Option<&'static str>,
    pub no_style: Option<Style>,

    pub non_exhaustive: NonExhaustive,
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

    /// Scrollbar orientation.
    pub fn orientation(mut self, pos: ScrollbarOrientation) -> Self {
        self.orientation = pos;
        self
    }

    /// Overscrolling
    pub fn overscroll_by(mut self, overscroll: usize) -> Self {
        self.overscroll = overscroll;
        self
    }

    /// Ensures a vertical orientation.
    pub fn override_vertical(mut self) -> Self {
        self.orientation = match self.orientation {
            ScrollbarOrientation::VerticalRight => ScrollbarOrientation::VerticalRight,
            ScrollbarOrientation::VerticalLeft => ScrollbarOrientation::VerticalLeft,
            ScrollbarOrientation::HorizontalBottom => ScrollbarOrientation::VerticalRight,
            ScrollbarOrientation::HorizontalTop => ScrollbarOrientation::VerticalRight,
        };
        self
    }

    /// Ensures a horizotal orientation.
    pub fn override_horizontal(mut self) -> Self {
        self.orientation = match self.orientation {
            ScrollbarOrientation::VerticalRight => ScrollbarOrientation::HorizontalBottom,
            ScrollbarOrientation::VerticalLeft => ScrollbarOrientation::HorizontalBottom,
            ScrollbarOrientation::HorizontalBottom => ScrollbarOrientation::HorizontalBottom,
            ScrollbarOrientation::HorizontalTop => ScrollbarOrientation::HorizontalTop,
        };
        self
    }

    pub fn is_vertical(&self) -> bool {
        match self.orientation {
            ScrollbarOrientation::VerticalRight => true,
            ScrollbarOrientation::VerticalLeft => true,
            ScrollbarOrientation::HorizontalBottom => false,
            ScrollbarOrientation::HorizontalTop => false,
        }
    }

    pub fn is_horizontal(&self) -> bool {
        match self.orientation {
            ScrollbarOrientation::VerticalRight => false,
            ScrollbarOrientation::VerticalLeft => false,
            ScrollbarOrientation::HorizontalBottom => true,
            ScrollbarOrientation::HorizontalTop => true,
        }
    }

    pub fn styles(mut self, styles: ScrolledStyle) -> Self {
        self.thumb_style = styles.thumb_style;
        self.track_symbol = styles.track_symbol;
        self.track_style = styles.track_style;
        self.begin_symbol = styles.begin_symbol;
        self.begin_style = styles.begin_style;
        self.end_symbol = styles.end_symbol;
        self.end_style = styles.end_style;
        self.no_symbol = styles.no_symbol;
        self.no_style = styles.no_style;
        self
    }

    /// Symbol for the Scrollbar.
    pub fn thumb_symbol(mut self, thumb_symbol: &'a str) -> Self {
        self.thumb_symbol = Some(thumb_symbol);
        self
    }

    /// Style for the Scrollbar.
    pub fn thumb_style<S: Into<Style>>(mut self, thumb_style: S) -> Self {
        self.thumb_style = Some(thumb_style.into());
        self
    }

    /// Symbol for the Scrollbar.
    pub fn track_symbol(mut self, track_symbol: Option<&'a str>) -> Self {
        self.track_symbol = track_symbol;
        self
    }

    /// Style for the Scrollbar.
    pub fn track_style<S: Into<Style>>(mut self, track_style: S) -> Self {
        self.track_style = Some(track_style.into());
        self
    }

    /// Symbol for the Scrollbar.
    pub fn begin_symbol(mut self, begin_symbol: Option<&'a str>) -> Self {
        self.begin_symbol = begin_symbol;
        self
    }

    /// Style for the Scrollbar.
    pub fn begin_style<S: Into<Style>>(mut self, begin_style: S) -> Self {
        self.begin_style = Some(begin_style.into());
        self
    }

    /// Symbol for the Scrollbar.
    pub fn end_symbol(mut self, end_symbol: Option<&'a str>) -> Self {
        self.end_symbol = end_symbol;
        self
    }

    /// Style for the Scrollbar.
    pub fn end_style<S: Into<Style>>(mut self, end_style: S) -> Self {
        self.end_style = Some(end_style.into());
        self
    }

    /// Symbol when no Scrollbar is drawn.
    pub fn no_symbol(mut self, no_symbol: Option<&'a str>) -> Self {
        self.no_symbol = no_symbol;
        self
    }

    /// Style when no Scrollbar is drawn.
    pub fn no_style<S: Into<Style>>(mut self, no_style: S) -> Self {
        self.no_style = Some(no_style.into());
        self
    }

    /// Set all Scrollbar symbols.
    pub fn symbols(mut self, symbols: Set) -> Self {
        self.thumb_symbol = Some(symbols.thumb);
        if self.track_symbol.is_some() {
            self.track_symbol = Some(symbols.track);
        }
        if self.begin_symbol.is_some() {
            self.begin_symbol = Some(symbols.begin);
        }
        if self.end_symbol.is_some() {
            self.end_symbol = Some(symbols.end);
        }
        self
    }

    /// Set a style for all Scrollbar styles.
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        let style = style.into();
        self.track_style = Some(style);
        self.thumb_style = Some(style);
        self.begin_style = Some(style);
        self.end_style = Some(style);
        self.no_style = Some(style);
        self
    }
}

impl<'a> Scroll<'a> {
    fn scrollbar(&self) -> Scrollbar<'a> {
        let mut scrollbar = Scrollbar::new(self.orientation.clone());
        if let Some(thumb_symbol) = self.thumb_symbol {
            scrollbar = scrollbar.thumb_symbol(thumb_symbol);
        }
        if let Some(track_symbol) = self.track_symbol {
            scrollbar = scrollbar.track_symbol(Some(track_symbol));
        }
        if let Some(begin_symbol) = self.begin_symbol {
            scrollbar = scrollbar.begin_symbol(Some(begin_symbol));
        }
        if let Some(end_symbol) = self.end_symbol {
            scrollbar = scrollbar.end_symbol(Some(end_symbol));
        }
        if let Some(thumb_style) = self.thumb_style {
            scrollbar = scrollbar.thumb_style(thumb_style);
        }
        if let Some(track_style) = self.track_style {
            scrollbar = scrollbar.track_style(track_style);
        }
        if let Some(begin_style) = self.begin_style {
            scrollbar = scrollbar.begin_style(begin_style);
        }
        if let Some(end_style) = self.end_style {
            scrollbar = scrollbar.end_style(end_style);
        }
        scrollbar
    }
}

/// Calculate the layout for the given scrollbars.
/// This prevents overlaps in the corners, if both scrollbars are
/// visible.
///
/// Returns (h_area, v_area, inner_area).
pub fn layout_scroll(
    area: Rect,
    block: Option<&Block<'_>>,
    h_scroll: Option<&Scroll<'_>>,
    v_scroll: Option<&Scroll<'_>>,
) -> (Rect, Rect, Rect) {
    let h_do_scroll;
    let mut h_area = if let Some(h_scroll) = h_scroll {
        h_do_scroll = true;
        match h_scroll.orientation {
            ScrollbarOrientation::VerticalRight => {
                unimplemented!()
            }
            ScrollbarOrientation::VerticalLeft => {
                unimplemented!()
            }
            ScrollbarOrientation::HorizontalBottom => {
                if area.height > 0 {
                    Rect::new(area.x, area.y + area.height - 1, area.width, 1)
                } else {
                    Rect::new(area.x, area.y, area.width, 0)
                }
            }
            ScrollbarOrientation::HorizontalTop => {
                if area.height > 0 {
                    Rect::new(area.x, area.y, area.width, 1)
                } else {
                    Rect::new(area.x, area.y, area.width, 0)
                }
            }
        }
    } else {
        h_do_scroll = false;
        Rect::new(area.x, area.y, 0, 0)
    };
    let v_do_scroll;
    let mut v_area = if let Some(v_scroll) = v_scroll {
        v_do_scroll = true;
        match v_scroll.orientation {
            ScrollbarOrientation::VerticalRight => {
                if area.width > 0 {
                    Rect::new(area.x + area.width - 1, area.y, 1, area.height)
                } else {
                    Rect::new(area.x, area.y, 0, area.height)
                }
            }
            ScrollbarOrientation::VerticalLeft => {
                if area.width > 0 {
                    Rect::new(area.x, area.y, 1, area.height)
                } else {
                    Rect::new(area.x, area.y, 0, area.height)
                }
            }
            ScrollbarOrientation::HorizontalBottom => {
                unimplemented!()
            }
            ScrollbarOrientation::HorizontalTop => {
                unimplemented!()
            }
        }
    } else {
        v_do_scroll = false;
        Rect::new(area.x, area.y, 0, 0)
    };

    if h_do_scroll && v_do_scroll {
        match h_scroll.map(|v| v.orientation.clone()) {
            None => {
                unreachable!()
            }
            Some(ScrollbarOrientation::VerticalRight) => {
                unimplemented!()
            }
            Some(ScrollbarOrientation::VerticalLeft) => {
                unimplemented!()
            }
            Some(ScrollbarOrientation::HorizontalTop) => {
                v_area.y += 1;
                v_area.height = v_area.height.saturating_sub(1);
            }
            Some(ScrollbarOrientation::HorizontalBottom) => {
                v_area.height = v_area.height.saturating_sub(1);
            }
        }
        match v_scroll.map(|v| v.orientation.clone()) {
            None => {
                unreachable!()
            }
            Some(ScrollbarOrientation::VerticalRight) => {
                h_area.width = h_area.width.saturating_sub(1);
            }
            Some(ScrollbarOrientation::VerticalLeft) => {
                h_area.x += 1;
                h_area.width = h_area.width.saturating_sub(1);
            }
            Some(ScrollbarOrientation::HorizontalTop) => {
                unimplemented!()
            }
            Some(ScrollbarOrientation::HorizontalBottom) => {
                unimplemented!()
            }
        }
    }
    if block.is_some() {
        v_area.y += 1;
        v_area.height = v_area.height.saturating_sub(1);

        h_area.x += 1;
        h_area.width = h_area.width.saturating_sub(1);
    }

    (
        h_area,
        v_area,
        layout_scroll_inner(area, block, h_scroll, v_scroll),
    )
}

/// Calculate the inner area in the presence of a Block
/// and maybe two Scroll instances.
///
/// max_offset > 0 indicates the wish to scroll by the widget.
pub fn layout_scroll_inner(
    area: Rect,
    block: Option<&Block<'_>>,
    h_scroll: Option<&Scroll<'_>>,
    v_scroll: Option<&Scroll<'_>>,
) -> Rect {
    if let Some(block) = block {
        block.inner(area)
    } else {
        let mut inner = area;
        if let Some(h_scroll) = h_scroll {
            match h_scroll.orientation {
                ScrollbarOrientation::VerticalRight => {
                    unimplemented!()
                }
                ScrollbarOrientation::VerticalLeft => {
                    unimplemented!()
                }
                ScrollbarOrientation::HorizontalBottom => {
                    inner.height -= 1;
                }
                ScrollbarOrientation::HorizontalTop => {
                    inner.y += 1;
                    inner.height = inner.height.saturating_sub(1);
                }
            }
        }
        if let Some(v_scroll) = v_scroll {
            match v_scroll.orientation {
                ScrollbarOrientation::VerticalRight => {
                    inner.width -= 1;
                }
                ScrollbarOrientation::VerticalLeft => {
                    inner.x += 1;
                    inner.width = inner.width.saturating_sub(1);
                }
                ScrollbarOrientation::HorizontalBottom => {
                    unimplemented!()
                }
                ScrollbarOrientation::HorizontalTop => {
                    unimplemented!()
                }
            }
        }
        inner
    }
}

impl<'a> StatefulWidget for Scroll<'a> {
    type State = ScrollState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_scroll(&self, area, buf, state)
    }
}

impl<'a> StatefulWidgetRef for Scroll<'a> {
    type State = ScrollState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_scroll(self, area, buf, state);
    }
}

fn render_scroll(scroll: &Scroll<'_>, area: Rect, buf: &mut Buffer, state: &mut ScrollState) {
    state.orientation = scroll.orientation.clone();
    state.overscroll_by = scroll.overscroll;

    state.area = area;

    if !scroll.policy.show_scrollbar(state) {
        let sym = if let Some(no_symbol) = scroll.no_symbol {
            no_symbol
        } else if scroll.is_vertical() {
            "\u{250A}"
        } else {
            "\u{2508}"
        };
        for row in area.y..area.y + area.height {
            for col in area.x..area.x + area.width {
                let cell = buf.get_mut(col, row);
                if let Some(no_style) = scroll.no_style {
                    cell.set_style(no_style);
                }
                cell.set_symbol(sym);
            }
        }
    } else {
        scroll
            .scrollbar()
            .render(area, buf, &mut scroll.policy.scrollbar(state));
    }
}

impl Default for ScrollState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            max_offset: 0,
            offset: 0,
            page_len: 0,
            scroll_by: None,
            overscroll_by: 0,
            mouse: Default::default(),
            orientation: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl ScrollState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Current vertical offset.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Change the offset. Limits the offset to max_v_offset + v_overscroll.
    ///
    /// Due to overscroll it's possible that this is an invalid
    /// offset for the widget. The widget must deal with this
    /// situation.
    pub fn set_offset(&mut self, offset: usize) -> bool {
        let old = self.offset;
        self.offset = self.limited_offset(offset);
        old != self.offset
    }

    /// Change the offset by some delta. Limits the offset to 0..max_v_offset + v_overscroll.
    ///
    /// Due to overscroll it's possible that this is an invalid
    /// offset for the widget. The widget must deal with this
    /// situation.
    pub fn change_offset(&mut self, delta: isize) -> bool {
        let old = self.offset;

        let offset = isize::try_from(self.offset).unwrap_or(isize::MAX);
        let offset = offset.saturating_add(delta);

        self.offset = self.clamp_offset(offset);
        old != self.offset
    }

    /// Calculate the offset limited to max_offset+overscroll_by.
    pub fn limited_offset(&self, offset: usize) -> usize {
        min(offset, self.max_offset.saturating_add(self.overscroll_by))
    }

    /// Clamp the offset given as isize between 0 and max_offset+overscroll_by.
    pub fn clamp_offset(&self, offset: isize) -> usize {
        let max_offset = isize::try_from(self.max_offset).unwrap_or(isize::MAX);
        let overscroll_by = isize::try_from(self.overscroll_by).unwrap_or(isize::MAX);
        let max_offset = max_offset.saturating_add(overscroll_by);

        offset.clamp(0, max_offset) as usize
    }

    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is shorter than the length of the content by whatever fills the last page.
    /// This is the base for the scrollbar content_length.
    pub fn max_offset(&self) -> usize {
        self.max_offset
    }

    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is shorter than the length of the content by whatever fills the last page.
    /// This is the base for the scrollbar content_length.
    pub fn set_max_offset(&mut self, max: usize) {
        self.max_offset = max;
    }

    /// Page-size at the current offset.
    pub fn page_len(&self) -> usize {
        self.page_len
    }

    /// Page-size at the current offset.
    pub fn set_page_len(&mut self, page: usize) {
        self.page_len = page;
    }

    /// Suggested scroll per scroll-event.
    /// Defaults to 1/10 of the page
    pub fn scroll_by(&self) -> usize {
        if let Some(scroll) = self.scroll_by {
            max(scroll, 1)
        } else {
            max(self.page_len / 10, 1)
        }
    }

    /// Suggested scroll per scroll-event.
    /// Defaults to 1/10 of the page
    pub fn set_scroll_by(&mut self, scroll: Option<usize>) {
        self.scroll_by = scroll;
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, ScrollOutcome> for ScrollState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> ScrollOutcome {
        match event {
            ct_event!(mouse any for m) if self.mouse.drag(self.area, m) => {
                match self.orientation {
                    ScrollbarOrientation::VerticalRight | ScrollbarOrientation::VerticalLeft => {
                        if m.row >= self.area.y {
                            // correct for the top `^` and bottom `v` arrows.
                            let row = m.row.saturating_sub(self.area.y).saturating_sub(1) as usize;
                            let height = self.area.height.saturating_sub(2) as usize;
                            let pos = (self.max_offset * row) / height;

                            ScrollOutcome::Offset(pos)
                        } else {
                            ScrollOutcome::Unchanged
                        }
                    }
                    ScrollbarOrientation::HorizontalBottom
                    | ScrollbarOrientation::HorizontalTop => {
                        if m.column >= self.area.x {
                            // correct for the left `<` and right `>` arrows.
                            let col =
                                m.column.saturating_sub(self.area.x).saturating_sub(1) as usize;
                            let width = self.area.width.saturating_sub(2) as usize;
                            let pos = (self.max_offset * col) / width;
                            ScrollOutcome::Offset(pos)
                        } else {
                            ScrollOutcome::Unchanged
                        }
                    }
                }
            }
            ct_event!(mouse down Left for col, row) if self.area.contains((*col, *row).into()) => {
                match self.orientation {
                    ScrollbarOrientation::VerticalRight | ScrollbarOrientation::VerticalLeft => {
                        // correct for the top `^` and bottom `v` arrows.
                        let row = row.saturating_sub(self.area.y).saturating_sub(1) as usize;
                        let height = self.area.height.saturating_sub(2) as usize;
                        let pos = (self.max_offset * row) / height;

                        ScrollOutcome::Offset(pos)
                    }
                    ScrollbarOrientation::HorizontalBottom
                    | ScrollbarOrientation::HorizontalTop => {
                        // correct for the left `<` and right `>` arrows.
                        let col = col.saturating_sub(self.area.x).saturating_sub(1) as usize;
                        let width = self.area.width.saturating_sub(2) as usize;
                        let pos = (self.max_offset * col) / width;
                        ScrollOutcome::Offset(pos)
                    }
                }
            }
            _ => ScrollOutcome::NotUsed,
        }
    }
}

/// Handle scroll events for the given area and the (possibly) two scrollbars.
#[derive(Debug)]
pub struct ScrollArea<'a>(
    pub Rect,
    pub Option<&'a ScrollState>,
    pub Option<&'a ScrollState>,
);

/// Handle scrolling for the whole area spanned by the two scroll-states.
impl<'a> HandleEvent<crossterm::event::Event, MouseOnly, ScrollOutcome> for ScrollArea<'a> {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> ScrollOutcome {
        let area = self.0;

        if let Some(hscroll) = self.1 {
            flow!(match event {
                // right scroll with ALT down. shift doesn't work?
                ct_event!(scroll ALT down for column, row) => {
                    if area.contains(Position::new(*column, *row)) {
                        ScrollOutcome::Delta(hscroll.scroll_by() as isize, 0)
                    } else {
                        ScrollOutcome::NotUsed
                    }
                }
                // left scroll with ALT up. shift doesn't work?
                ct_event!(scroll ALT up for column, row) => {
                    if area.contains(Position::new(*column, *row)) {
                        ScrollOutcome::Delta(-(hscroll.scroll_by() as isize), 0)
                    } else {
                        ScrollOutcome::NotUsed
                    }
                }
                _ => ScrollOutcome::NotUsed,
            });
        }
        if let Some(vscroll) = self.2 {
            flow!(match event {
                ct_event!(scroll down for column, row) => {
                    if area.contains(Position::new(*column, *row)) {
                        ScrollOutcome::Delta(0, vscroll.scroll_by() as isize)
                    } else {
                        ScrollOutcome::NotUsed
                    }
                }
                ct_event!(scroll up for column, row) => {
                    if area.contains(Position::new(*column, *row)) {
                        ScrollOutcome::Delta(0, -(vscroll.scroll_by() as isize))
                    } else {
                        ScrollOutcome::NotUsed
                    }
                }
                _ => ScrollOutcome::NotUsed,
            });
        }

        ScrollOutcome::NotUsed
    }
}

impl ScrollbarPolicy {
    fn scrollbar(self, state: &ScrollState) -> ScrollbarState {
        match self {
            ScrollbarPolicy::Always => ScrollbarState::new(max(state.max_offset, 1))
                .position(state.offset)
                .viewport_content_length(state.page_len),
            ScrollbarPolicy::AsNeeded => ScrollbarState::new(state.max_offset)
                .position(state.offset)
                .viewport_content_length(state.page_len),
        }
    }

    fn show_scrollbar(self, state: &ScrollState) -> bool {
        match self {
            ScrollbarPolicy::Always => true,
            ScrollbarPolicy::AsNeeded => state.max_offset > 0,
        }
    }
}

impl Default for ScrolledStyle {
    fn default() -> Self {
        Self {
            thumb_style: None,
            track_symbol: None,
            track_style: None,
            begin_symbol: None,
            begin_style: None,
            end_symbol: None,
            end_style: None,
            no_symbol: None,
            no_style: None,
            non_exhaustive: NonExhaustive,
        }
    }
}
