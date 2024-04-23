///
/// Add scrolling behaviour to a widget.
///
/// Scrolled acts as a wrapper around a widget that implements HasVerticalScroll.
/// No HasHorizontalScroll at the moment, probably never will be and instead
/// a HasScroll covering both.
///
use crate::{
    check_break, ct_event, CanValidate, ControlUI, DefaultKeys, FocusFlag, HandleCrossterm,
    HasFocusFlag, HasScrolling, HasValidFlag, MouseOnly, ScrolledWidget, ValidFlag,
};
use crossterm::event::Event;
#[allow(unused_imports)]
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::prelude::{BlockExt, Style};
use ratatui::symbols::scrollbar::Set;
use ratatui::widgets::{
    Block, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget,
};
use std::cmp::min;

/// A wrapper widget that scrolls it's content.
#[derive(Debug, Default, Clone)]
pub struct Scrolled<'a, T> {
    /// widget
    pub widget: T,

    pub h_overscroll: usize,
    pub v_overscroll: usize,
    pub h_scroll_policy: ScrollbarPolicy,
    pub v_scroll_policy: ScrollbarPolicy,
    pub h_scroll_position: HScrollPosition,
    pub v_scroll_position: VScrollPosition,

    pub block: Option<Block<'a>>,

    pub thumb_symbol: Option<&'a str>,
    pub thumb_style: Option<Style>,
    pub track_symbol: Option<&'a str>,
    pub track_style: Option<Style>,
    pub begin_symbol: Option<&'a str>,
    pub begin_style: Option<Style>,
    pub end_symbol: Option<&'a str>,
    pub end_style: Option<Style>,

    pub non_exhaustive: (),
}

/// Scrolled state.
#[derive(Debug, Default, Clone)]
pub struct ScrolledState<S> {
    pub widget: S,

    pub area: Rect,
    pub view_area: Rect,
    pub h_scrollbar_area: Option<Rect>,
    pub v_scrollbar_area: Option<Rect>,

    pub v_overscroll: usize,
    pub h_overscroll: usize,

    /// mouse action in progress
    pub v_drag: bool,
    pub h_drag: bool,

    pub non_exhaustive: (),
}

/// This policy plus ScrollParam allow to decide what to show.
#[derive(Debug, Default, Clone, Copy)]
pub enum ScrollbarPolicy {
    Always,
    #[default]
    AsNeeded,
    Never,
}

/// Position of the vertical scrollbar.
#[derive(Debug, Default, Clone, Copy)]
pub enum VScrollPosition {
    Left,
    #[default]
    Right,
}

/// Position of the horizontal scrollbar.
#[derive(Debug, Default, Clone, Copy)]
pub enum HScrollPosition {
    Top,
    #[default]
    Bottom,
}

impl<'a, T> Scrolled<'a, T> {
    pub fn new(inner: T) -> Self {
        Self {
            widget: inner,

            h_overscroll: 0,
            v_overscroll: 0,
            h_scroll_policy: Default::default(),
            v_scroll_policy: Default::default(),
            h_scroll_position: Default::default(),
            v_scroll_position: Default::default(),
            block: None,
            thumb_symbol: None,
            thumb_style: None,
            track_symbol: None,
            track_style: None,
            begin_symbol: None,
            begin_style: None,
            end_symbol: None,
            end_style: None,
            non_exhaustive: (),
        }
    }

    /// Allow overscrolling the max_offset by n.
    pub fn v_overscroll(mut self, n: usize) -> Self {
        self.v_overscroll = n;
        self
    }

    /// Allow overscrolling the max_offset by n.
    pub fn h_overscroll(mut self, n: usize) -> Self {
        self.h_overscroll = n;
        self
    }

    /// Horizontal scrollbar policy.
    pub fn h_scrollbar_policy(mut self, policy: ScrollbarPolicy) -> Self {
        self.h_scroll_policy = policy;
        self
    }

    /// Vertical scrollbar policy.
    pub fn v_scrollbar_policy(mut self, policy: ScrollbarPolicy) -> Self {
        self.v_scroll_policy = policy;
        self
    }

    /// Position
    pub fn h_scroll_position(mut self, pos: HScrollPosition) -> Self {
        self.h_scroll_position = pos;
        self
    }

    /// Position
    pub fn v_scroll_position(mut self, pos: VScrollPosition) -> Self {
        self.v_scroll_position = pos;
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn thumb_symbol(mut self, thumb_symbol: &'a str) -> Self {
        self.thumb_symbol = Some(thumb_symbol);
        self
    }

    pub fn thumb_style<S: Into<Style>>(mut self, thumb_style: S) -> Self {
        self.thumb_style = Some(thumb_style.into());
        self
    }

    pub fn track_symbol(mut self, track_symbol: Option<&'a str>) -> Self {
        self.track_symbol = track_symbol;
        self
    }

    pub fn track_style<S: Into<Style>>(mut self, track_style: S) -> Self {
        self.track_style = Some(track_style.into());
        self
    }

    pub fn begin_symbol(mut self, begin_symbol: Option<&'a str>) -> Self {
        self.begin_symbol = begin_symbol;
        self
    }

    pub fn begin_style<S: Into<Style>>(mut self, begin_style: S) -> Self {
        self.begin_style = Some(begin_style.into());
        self
    }

    pub fn end_symbol(mut self, end_symbol: Option<&'a str>) -> Self {
        self.end_symbol = end_symbol;
        self
    }

    pub fn end_style<S: Into<Style>>(mut self, end_style: S) -> Self {
        self.end_style = Some(end_style.into());
        self
    }

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

    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        let style = style.into();
        self.track_style = Some(style);
        self.thumb_style = Some(style);
        self.begin_style = Some(style);
        self.end_style = Some(style);
        self
    }
}

impl<'a, T> StatefulWidget for Scrolled<'a, T>
where
    T: StatefulWidget + ScrolledWidget,
    T::State: HasScrolling,
{
    type State = ScrolledState<T::State>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.v_overscroll = self.v_overscroll;
        state.h_overscroll = self.h_overscroll;

        let sconf = self.widget.need_scroll(self.block.inner_if_some(area));

        let has_vscroll = self.v_scroll_policy.apply(sconf.has_vscroll);
        let has_hscroll = self.h_scroll_policy.apply(sconf.has_hscroll);

        // Calculate the areas for the scrollbars and the view-area.
        // If there is a block set, assume there is a right and a bottom border too.
        // Currently, there is no way to know it. Overwriting part of the content is
        // ok in this case.
        if has_vscroll && has_hscroll {
            let mut vscrollbar_area = area.columns().last().expect("scroll");
            if self.block.is_some() {
                vscrollbar_area.y += 1;
                vscrollbar_area.height -= 2;
            } else {
                vscrollbar_area.height -= 1;
            }
            state.v_scrollbar_area = Some(vscrollbar_area);

            let mut hscrollbar_area = area.rows().last().expect("scroll");
            if self.block.is_some() {
                hscrollbar_area.x += 1;
                hscrollbar_area.width -= 2;
            } else {
                hscrollbar_area.width -= 1;
            }
            state.h_scrollbar_area = Some(hscrollbar_area);

            if let Some(block) = self.block.as_ref() {
                state.view_area = block.inner(area);
            } else {
                state.view_area = area;
                state.view_area.width -= 1;
                state.view_area.height -= 1;
            }
        } else if has_vscroll {
            let mut vscrollbar_area = area.columns().last().expect("scroll");
            if self.block.is_some() {
                vscrollbar_area.y += 1;
                vscrollbar_area.height -= 1;
            } else {
                vscrollbar_area.height -= 0;
            }
            state.v_scrollbar_area = Some(vscrollbar_area);

            state.h_scrollbar_area = None;

            if let Some(block) = self.block.as_ref() {
                state.view_area = block.inner(area);
            } else {
                state.view_area = area;
                state.view_area.width -= 1;
                state.view_area.height -= 0;
            }
        } else if has_hscroll {
            state.v_scrollbar_area = None;

            let mut hscrollbar_area = area.rows().last().expect("scroll");
            if self.block.is_some() {
                hscrollbar_area.x += 1;
                hscrollbar_area.width -= 1;
            } else {
                hscrollbar_area.width -= 0;
            }
            state.h_scrollbar_area = Some(hscrollbar_area);

            if let Some(block) = self.block.as_ref() {
                state.view_area = block.inner(area);
            } else {
                state.view_area = area;
                state.view_area.width -= 0;
                state.view_area.height -= 1;
            }
        } else {
            state.v_scrollbar_area = None;
            state.h_scrollbar_area = None;

            if let Some(block) = self.block.as_ref() {
                state.view_area = block.inner(area);
            } else {
                state.view_area = area;
                state.view_area.width -= 0;
                state.view_area.height -= 0;
            }
        }

        self.widget.render(state.view_area, buf, &mut state.widget);
        self.block.render(area, buf);

        if let Some(vscrollbar_area) = state.v_scrollbar_area {
            let mut vscroll = Scrollbar::new(self.v_scroll_position.orientation());
            if let Some(thumb_symbol) = self.thumb_symbol {
                vscroll = vscroll.thumb_symbol(thumb_symbol);
            }
            if let Some(track_symbol) = self.track_symbol {
                vscroll = vscroll.track_symbol(Some(track_symbol));
            }
            if let Some(begin_symbol) = self.begin_symbol {
                vscroll = vscroll.begin_symbol(Some(begin_symbol));
            }
            if let Some(end_symbol) = self.end_symbol {
                vscroll = vscroll.end_symbol(Some(end_symbol));
            }
            if let Some(thumb_style) = self.thumb_style {
                vscroll = vscroll.thumb_style(thumb_style);
            }
            if let Some(track_style) = self.track_style {
                vscroll = vscroll.track_style(track_style);
            }
            if let Some(begin_style) = self.begin_style {
                vscroll = vscroll.begin_style(begin_style);
            }
            if let Some(end_style) = self.end_style {
                vscroll = vscroll.end_style(end_style);
            }

            let max_offset = state.widget.max_v_offset();
            let offset = state.widget.v_offset();

            let mut vscroll_state = ScrollbarState::new(max_offset).position(offset);
            vscroll.render(vscrollbar_area, buf, &mut vscroll_state);
        }

        if let Some(hscrollbar_area) = state.h_scrollbar_area {
            let mut hscroll = Scrollbar::new(self.h_scroll_position.orientation());
            if let Some(thumb_symbol) = self.thumb_symbol {
                hscroll = hscroll.thumb_symbol(thumb_symbol);
            }
            if let Some(track_symbol) = self.track_symbol {
                hscroll = hscroll.track_symbol(Some(track_symbol));
            }
            if let Some(begin_symbol) = self.begin_symbol {
                hscroll = hscroll.begin_symbol(Some(begin_symbol));
            }
            if let Some(end_symbol) = self.end_symbol {
                hscroll = hscroll.end_symbol(Some(end_symbol));
            }
            if let Some(thumb_style) = self.thumb_style {
                hscroll = hscroll.thumb_style(thumb_style);
            }
            if let Some(track_style) = self.track_style {
                hscroll = hscroll.track_style(track_style);
            }
            if let Some(begin_style) = self.begin_style {
                hscroll = hscroll.begin_style(begin_style);
            }
            if let Some(end_style) = self.end_style {
                hscroll = hscroll.end_style(end_style);
            }

            let max_offset = state.widget.max_h_offset();
            let offset = state.widget.h_offset();

            let mut hscroll_state = ScrollbarState::new(max_offset).position(offset);
            hscroll.render(hscrollbar_area, buf, &mut hscroll_state);
        }
    }
}

impl ScrollbarPolicy {
    fn apply(&self, scroll: bool) -> bool {
        match self {
            ScrollbarPolicy::Always => true,
            ScrollbarPolicy::AsNeeded => scroll,
            ScrollbarPolicy::Never => false,
        }
    }
}

impl HScrollPosition {
    fn orientation(&self) -> ScrollbarOrientation {
        match self {
            HScrollPosition::Top => ScrollbarOrientation::HorizontalTop,
            HScrollPosition::Bottom => ScrollbarOrientation::HorizontalBottom,
        }
    }
}

impl VScrollPosition {
    fn orientation(&self) -> ScrollbarOrientation {
        match self {
            VScrollPosition::Left => ScrollbarOrientation::VerticalLeft,
            VScrollPosition::Right => ScrollbarOrientation::VerticalRight,
        }
    }
}

impl<S: HasScrolling> ScrolledState<S> {
    /// Current vertical offset.
    pub fn v_offset(&self) -> usize {
        self.widget.v_offset()
    }

    /// Current horizontal offset.
    pub fn h_offset(&self) -> usize {
        self.widget.h_offset()
    }

    /// Change the offset.
    ///
    /// Due to overscroll it's possible that this is an invalid
    /// offset for the widget. The widget must deal with this
    /// situation.
    pub fn set_v_offset(&mut self, offset: usize) {
        self.widget.set_v_offset(offset);
    }

    /// Change the offset.
    ///
    /// Limits the offset to max_v_offset + v_overscroll.
    pub fn set_limited_v_offset(&mut self, offset: usize) {
        let voffset = min(offset, self.widget.max_v_offset() + self.v_overscroll);
        self.set_v_offset(voffset);
    }

    /// Change the offset.
    ///
    /// Due to overscroll it's possible that this is an invalid
    /// offset for the widget. The widget must deal with this
    /// situation.
    pub fn set_h_offset(&mut self, offset: usize) {
        self.widget.set_h_offset(offset);
    }

    /// Change the offset
    ///
    /// Limits the offset to max_v_offset + v_overscroll.
    pub fn set_limited_h_offset(&mut self, offset: usize) {
        let hoffset = min(offset, self.widget.max_h_offset() + self.h_overscroll);
        self.set_h_offset(hoffset);
    }

    /// Scroll up by n.
    pub fn scroll_up(&mut self, n: usize) {
        self.set_v_offset(self.v_offset().saturating_sub(n));
    }

    /// Scroll down by n.
    pub fn scroll_down(&mut self, n: usize) {
        self.set_v_offset(self.v_offset() + n);
    }

    /// Scroll up by n.
    pub fn scroll_left(&mut self, n: usize) {
        self.set_h_offset(self.h_offset().saturating_sub(n));
    }

    /// Scroll down by n.
    pub fn scroll_right(&mut self, n: usize) {
        self.set_h_offset(self.h_offset() + n);
    }

    /// Scroll down by n, but limited by the max_offset + overscroll
    pub fn limited_scroll_down(&mut self, n: usize) {
        let voffset = min(
            self.widget.v_offset() + n,
            self.widget.max_v_offset() + self.v_overscroll,
        );
        self.set_v_offset(voffset);
    }

    /// Scroll down by n, but limited by the max_offset + overscroll
    pub fn limited_scroll_right(&mut self, n: usize) {
        let hoffset = min(
            self.widget.h_offset() + n,
            self.widget.max_h_offset() + self.h_overscroll,
        );
        self.set_h_offset(hoffset);
    }
}

impl<S, A, E> HandleCrossterm<ControlUI<A, E>, DefaultKeys> for ScrolledState<S>
where
    S: HasScrolling
        + HandleCrossterm<ControlUI<A, E>, MouseOnly>
        + HandleCrossterm<ControlUI<A, E>, DefaultKeys>,
{
    fn handle(&mut self, event: &Event, _: DefaultKeys) -> ControlUI<A, E> {
        // Don't do key-events here. That's up to the widget, it has more
        // information about the indented behaviour.

        // Do handle some mouse events here.
        // * mouse interactions with the scroll-bars.
        // * scrolling.
        let res =
            <Self as HandleCrossterm<ControlUI<A, E>, MouseOnly>>::handle(self, event, MouseOnly);

        // Let the widget handle the rest.
        res.or_else(|| self.widget.handle(event, DefaultKeys))
    }
}

impl<S, A, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for ScrolledState<S>
where
    S: HasScrolling + HandleCrossterm<ControlUI<A, E>, MouseOnly>,
{
    fn handle(&mut self, event: &Event, _: MouseOnly) -> ControlUI<A, E> {
        let res = match event {
            ct_event!(mouse down Left for column,row) => {
                // Click in the scrollbar sets the offset to some absolute position.
                check_break!(if let Some(vscroll_area) = self.v_scrollbar_area {
                    if vscroll_area.contains(Position::new(*column, *row)) {
                        let row = row.saturating_sub(vscroll_area.y) as usize;
                        // max_v_offset is inclusive, so height should be too.
                        let height = vscroll_area.height.saturating_sub(1) as usize;

                        let pos = (self.widget.max_v_offset() * row) / height;

                        self.v_drag = true;
                        self.widget.set_v_offset(pos);

                        ControlUI::Change
                    } else {
                        ControlUI::Continue
                    }
                } else {
                    ControlUI::Continue
                });
                check_break!(if let Some(hscroll_area) = self.h_scrollbar_area {
                    if hscroll_area.contains(Position::new(*column, *row)) {
                        let col = column.saturating_sub(hscroll_area.x) as usize;
                        let width = hscroll_area.width.saturating_sub(1) as usize;

                        let pos = (self.widget.max_h_offset() * col) / width;

                        self.h_drag = true;
                        self.widget.set_h_offset(pos);

                        ControlUI::Change
                    } else {
                        ControlUI::Continue
                    }
                } else {
                    ControlUI::Continue
                });

                ControlUI::Continue
            }
            ct_event!(mouse drag Left for column, row) => {
                // dragging around the scroll bar
                check_break!(if self.v_drag {
                    if let Some(vscroll_area) = self.v_scrollbar_area {
                        let row = row.saturating_sub(vscroll_area.y) as usize;
                        let height = vscroll_area.height.saturating_sub(1) as usize;

                        let pos = (self.widget.max_v_offset() * row) / height;
                        self.set_limited_v_offset(pos);

                        ControlUI::Change
                    } else {
                        ControlUI::Continue
                    }
                } else {
                    ControlUI::Continue
                });
                check_break!(if self.h_drag {
                    if let Some(hscroll_area) = self.h_scrollbar_area {
                        let col = column.saturating_sub(hscroll_area.x) as usize;
                        let width = hscroll_area.width.saturating_sub(1) as usize;

                        let pos = (col * self.widget.max_h_offset()) / width;
                        self.set_limited_h_offset(pos);

                        ControlUI::Change
                    } else {
                        ControlUI::Continue
                    }
                } else {
                    ControlUI::Continue
                });

                ControlUI::Continue
            }

            ct_event!(mouse moved) => {
                // reset drag
                self.v_drag = false;
                self.h_drag = false;
                ControlUI::Continue
            }

            ct_event!(scroll down for column, row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.limited_scroll_down(self.view_area.height as usize / 10);
                    ControlUI::Change
                } else {
                    ControlUI::Continue
                }
            }
            ct_event!(scroll up for column, row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.widget.scroll_up(self.view_area.height as usize / 10);
                    ControlUI::Change
                } else {
                    ControlUI::Continue
                }
            }
            ct_event!(scroll ALT down for column, row) => {
                // right scroll with ALT. shift doesn't work?
                if self.area.contains(Position::new(*column, *row)) {
                    self.limited_scroll_right(self.view_area.width as usize / 10);
                    ControlUI::Change
                } else {
                    ControlUI::Continue
                }
            }
            ct_event!(scroll ALT up for column, row) => {
                // right scroll with ALT. shift doesn't work?
                if self.area.contains(Position::new(*column, *row)) {
                    self.widget.scroll_left(self.view_area.width as usize / 10);
                    ControlUI::Change
                } else {
                    ControlUI::Continue
                }
            }
            _ => ControlUI::Continue,
        };

        res.or_else(|| self.widget.handle(event, MouseOnly))
    }
}

// forward some library traits.
impl<T> HasFocusFlag for ScrolledState<T>
where
    T: HasFocusFlag,
{
    fn focus(&self) -> &FocusFlag {
        self.widget.focus()
    }

    fn area(&self) -> Rect {
        self.widget.area()
    }

    fn is_focused(&self) -> bool {
        self.widget.is_focused()
    }

    fn lost_focus(&self) -> bool {
        self.widget.lost_focus()
    }

    fn gained_focus(&self) -> bool {
        self.widget.gained_focus()
    }

    fn focus_tag(&self) -> u16 {
        self.widget.focus_tag()
    }
}

// forward some library traits.
impl<T> HasValidFlag for ScrolledState<T>
where
    T: HasValidFlag,
{
    fn valid(&self) -> &ValidFlag {
        self.widget.valid()
    }

    fn is_valid(&self) -> bool {
        self.widget.is_valid()
    }

    fn is_invalid(&self) -> bool {
        self.widget.is_invalid()
    }

    fn set_valid(&self, valid: bool) {
        self.widget.set_valid(valid);
    }

    fn set_valid_from<V, E>(&self, result: Result<V, E>) -> Option<V> {
        self.widget.set_valid_from(result)
    }
}

// forward some library traits.
impl<T> CanValidate for ScrolledState<T>
where
    T: CanValidate,
{
    fn validate(&mut self) {
        self.widget.validate()
    }
}
