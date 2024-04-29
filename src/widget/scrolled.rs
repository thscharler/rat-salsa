use crate::_private::NonExhaustive;
use crate::widget::viewport::Viewport;
///
/// Add scrolling behaviour to a widget.
///
/// Scrolled acts as a wrapper around a widget that implements HasVerticalScroll.
/// No HasHorizontalScroll at the moment, probably never will be and instead
/// a HasScroll covering both.
///
use crate::{
    check_break, ct_event, CanValidate, ControlUI, DefaultKeys, FocusFlag, HandleCrossterm,
    HasFocusFlag, HasScrolling, HasValidFlag, MouseOnly, ScrollParam, ScrolledWidget, ValidFlag,
};
use crossterm::event::Event;
#[allow(unused_imports)]
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect, Size};
use ratatui::prelude::{BlockExt, Style};
use ratatui::symbols::scrollbar::Set;
use ratatui::widgets::{
    Block, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, StatefulWidgetRef,
    Widget, WidgetRef,
};
use std::cmp::min;
use std::mem;

/// A wrapper widget that scrolls it's content.
#[derive(Debug, Default, Clone)]
pub struct Scrolled<'a, T> {
    /// widget
    widget: T,

    h_overscroll: usize,
    v_overscroll: usize,
    h_scroll_policy: ScrollbarPolicy,
    v_scroll_policy: ScrollbarPolicy,
    h_scroll_position: HScrollPosition,
    v_scroll_position: VScrollPosition,

    block: Option<Block<'a>>,

    thumb_symbol: Option<&'a str>,
    thumb_style: Option<Style>,
    track_symbol: Option<&'a str>,
    track_style: Option<Style>,
    begin_symbol: Option<&'a str>,
    begin_style: Option<Style>,
    end_symbol: Option<&'a str>,
    end_style: Option<Style>,
}

/// Scrolled state.
#[derive(Debug, Clone)]
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

    pub non_exhaustive: NonExhaustive,
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

impl<'a, T> Scrolled<'a, Viewport<T>>
where
    T: Widget,
{
    pub fn new_viewport(inner: T) -> Scrolled<'a, Viewport<T>> {
        Self {
            widget: Viewport::new(inner),
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
        }
    }

    /// Size for the inner widget.
    pub fn viewport_size(mut self, size: Size) -> Self {
        self.widget = self.widget.viewport_size(size);
        self
    }

    /// Style for the empty space outside the rendered buffer.
    pub fn viewport_style(mut self, style: Style) -> Self {
        self.widget = self.widget.style(style);
        self
    }

    pub fn viewport_fill_char(mut self, fill_char: char) -> Self {
        self.widget = self.widget.fill_char(fill_char);
        self
    }
}

impl<'a, T> StatefulWidgetRef for Scrolled<'a, T>
where
    T: StatefulWidgetRef + ScrolledWidget<T::State>,
    T::State: HasScrolling,
{
    type State = ScrolledState<T::State>;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let scroll_param = self
            .widget
            .need_scroll(self.block.inner_if_some(area), &mut state.widget);

        render_impl(self, area, buf, state, scroll_param, |area, buf, state| {
            self.widget.render_ref(area, buf, state);
        });
    }
}

impl<'a, T> StatefulWidget for Scrolled<'a, T>
where
    T: StatefulWidget + ScrolledWidget<T::State> + Default,
    T::State: HasScrolling,
{
    type State = ScrolledState<T::State>;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let scroll_param = self
            .widget
            .need_scroll(self.block.inner_if_some(area), &mut state.widget);

        let inner = mem::take(&mut self.widget);

        render_impl(&self, area, buf, state, scroll_param, |area, buf, state| {
            inner.render(area, buf, state);
        });
    }
}

fn render_impl<FnRender, T, S>(
    widget: &Scrolled<'_, T>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut ScrolledState<S>,
    scroll_param: ScrollParam,
    render_inner: FnRender,
) where
    T: ScrolledWidget<S>,
    S: HasScrolling,
    FnRender: FnOnce(Rect, &mut Buffer, &mut S),
{
    state.area = area;
    state.v_overscroll = widget.v_overscroll;
    state.h_overscroll = widget.h_overscroll;

    let has_vscroll = widget.v_scroll_policy.apply(scroll_param.has_vscroll);
    let has_hscroll = widget.h_scroll_policy.apply(scroll_param.has_hscroll);

    // Calculate the areas for the scrollbars and the view-area.
    // If there is a block set, assume there is a right and a bottom border too.
    // Currently, there is no way to know it. Overwriting part of the content is
    // ok in this case.
    if has_vscroll && has_hscroll {
        let mut vscrollbar_area = area.columns().last().expect("scroll");
        if widget.block.is_some() {
            vscrollbar_area.y += 1;
            vscrollbar_area.height -= 2;
        } else {
            vscrollbar_area.height -= 1;
        }
        state.v_scrollbar_area = Some(vscrollbar_area);

        let mut hscrollbar_area = area.rows().last().expect("scroll");
        if widget.block.is_some() {
            hscrollbar_area.x += 1;
            hscrollbar_area.width -= 2;
        } else {
            hscrollbar_area.width -= 1;
        }
        state.h_scrollbar_area = Some(hscrollbar_area);

        if let Some(block) = widget.block.as_ref() {
            state.view_area = block.inner(area);
        } else {
            state.view_area = area;
            state.view_area.width -= 1;
            state.view_area.height -= 1;
        }
    } else if has_vscroll {
        let mut vscrollbar_area = area.columns().last().expect("scroll");
        if widget.block.is_some() {
            vscrollbar_area.y += 1;
            vscrollbar_area.height -= 1;
        } else {
            vscrollbar_area.height -= 0;
        }
        state.v_scrollbar_area = Some(vscrollbar_area);

        state.h_scrollbar_area = None;

        if let Some(block) = widget.block.as_ref() {
            state.view_area = block.inner(area);
        } else {
            state.view_area = area;
            state.view_area.width -= 1;
            state.view_area.height -= 0;
        }
    } else if has_hscroll {
        state.v_scrollbar_area = None;

        let mut hscrollbar_area = area.rows().last().expect("scroll");
        if widget.block.is_some() {
            hscrollbar_area.x += 1;
            hscrollbar_area.width -= 1;
        } else {
            hscrollbar_area.width -= 0;
        }
        state.h_scrollbar_area = Some(hscrollbar_area);

        if let Some(block) = widget.block.as_ref() {
            state.view_area = block.inner(area);
        } else {
            state.view_area = area;
            state.view_area.width -= 0;
            state.view_area.height -= 1;
        }
    } else {
        state.v_scrollbar_area = None;
        state.h_scrollbar_area = None;

        if let Some(block) = widget.block.as_ref() {
            state.view_area = block.inner(area);
        } else {
            state.view_area = area;
            state.view_area.width -= 0;
            state.view_area.height -= 0;
        }
    }

    render_inner(state.view_area, buf, &mut state.widget);

    widget.block.render_ref(area, buf);

    if let Some(vscrollbar_area) = state.v_scrollbar_area {
        let mut vscroll = Scrollbar::new(widget.v_scroll_position.orientation());
        if let Some(thumb_symbol) = widget.thumb_symbol {
            vscroll = vscroll.thumb_symbol(thumb_symbol);
        }
        if let Some(track_symbol) = widget.track_symbol {
            vscroll = vscroll.track_symbol(Some(track_symbol));
        }
        if let Some(begin_symbol) = widget.begin_symbol {
            vscroll = vscroll.begin_symbol(Some(begin_symbol));
        }
        if let Some(end_symbol) = widget.end_symbol {
            vscroll = vscroll.end_symbol(Some(end_symbol));
        }
        if let Some(thumb_style) = widget.thumb_style {
            vscroll = vscroll.thumb_style(thumb_style);
        }
        if let Some(track_style) = widget.track_style {
            vscroll = vscroll.track_style(track_style);
        }
        if let Some(begin_style) = widget.begin_style {
            vscroll = vscroll.begin_style(begin_style);
        }
        if let Some(end_style) = widget.end_style {
            vscroll = vscroll.end_style(end_style);
        }

        let max_offset = state.widget.max_v_offset();
        let offset = state.widget.v_offset();

        let mut vscroll_state = ScrollbarState::new(max_offset).position(offset);
        vscroll.render(vscrollbar_area, buf, &mut vscroll_state);
    }

    if let Some(hscrollbar_area) = state.h_scrollbar_area {
        let mut hscroll = Scrollbar::new(widget.h_scroll_position.orientation());
        if let Some(thumb_symbol) = widget.thumb_symbol {
            hscroll = hscroll.thumb_symbol(thumb_symbol);
        }
        if let Some(track_symbol) = widget.track_symbol {
            hscroll = hscroll.track_symbol(Some(track_symbol));
        }
        if let Some(begin_symbol) = widget.begin_symbol {
            hscroll = hscroll.begin_symbol(Some(begin_symbol));
        }
        if let Some(end_symbol) = widget.end_symbol {
            hscroll = hscroll.end_symbol(Some(end_symbol));
        }
        if let Some(thumb_style) = widget.thumb_style {
            hscroll = hscroll.thumb_style(thumb_style);
        }
        if let Some(track_style) = widget.track_style {
            hscroll = hscroll.track_style(track_style);
        }
        if let Some(begin_style) = widget.begin_style {
            hscroll = hscroll.begin_style(begin_style);
        }
        if let Some(end_style) = widget.end_style {
            hscroll = hscroll.end_style(end_style);
        }

        let max_offset = state.widget.max_h_offset();
        let offset = state.widget.h_offset();

        let mut hscroll_state = ScrollbarState::new(max_offset).position(offset);
        hscroll.render(hscrollbar_area, buf, &mut hscroll_state);
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

impl<S: Default> Default for ScrolledState<S> {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            area: Default::default(),
            view_area: Default::default(),
            h_scrollbar_area: None,
            v_scrollbar_area: None,
            v_overscroll: 0,
            h_overscroll: 0,
            v_drag: false,
            h_drag: false,
            non_exhaustive: NonExhaustive,
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

    pub fn widget_mut(&mut self) -> &mut S {
        &mut self.widget
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
        let res = HandleCrossterm::handle(self, event, MouseOnly);

        // Let the widget handle the rest.
        res.on_continue(|| self.widget.handle(event, DefaultKeys))
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
                    self.limited_scroll_down(self.widget.v_page_len() / 10);
                    ControlUI::Change
                } else {
                    ControlUI::Continue
                }
            }
            ct_event!(scroll up for column, row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.widget.scroll_up(self.widget.v_page_len() / 10);
                    ControlUI::Change
                } else {
                    ControlUI::Continue
                }
            }
            ct_event!(scroll ALT down for column, row) => {
                // right scroll with ALT. shift doesn't work?
                if self.area.contains(Position::new(*column, *row)) {
                    self.limited_scroll_right(self.widget.h_page_len() / 10);
                    ControlUI::Change
                } else {
                    ControlUI::Continue
                }
            }
            ct_event!(scroll ALT up for column, row) => {
                // right scroll with ALT. shift doesn't work?
                if self.area.contains(Position::new(*column, *row)) {
                    self.widget.scroll_left(self.widget.h_page_len() / 10);
                    ControlUI::Change
                } else {
                    ControlUI::Continue
                }
            }
            _ => ControlUI::Continue,
        };

        res.on_continue(|| self.widget.handle(event, MouseOnly))
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
}

// forward some library traits.
impl<T> HasValidFlag for ScrolledState<T>
where
    T: HasValidFlag,
{
    fn valid(&self) -> &ValidFlag {
        self.widget.valid()
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
