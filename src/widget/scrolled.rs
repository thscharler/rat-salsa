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
use ratatui::prelude::Style;
use ratatui::symbols::scrollbar::Set;
use ratatui::widgets::{
    Block, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget,
};

/// A wrapper widget that scrolls it's content.
#[derive(Debug)]
pub struct Scrolled<'a, T> {
    /// widget
    pub widget: T,
    pub block: Option<Block<'a>>,

    pub thumb_symbol: Option<&'a str>,
    pub thumb_style: Option<Style>,
    pub track_symbol: Option<&'a str>,
    pub track_style: Option<Style>,
    pub begin_symbol: Option<&'a str>,
    pub begin_style: Option<Style>,
    pub end_symbol: Option<&'a str>,
    pub end_style: Option<Style>,
}

/// Scrolled state.
#[derive(Debug, Default)]
pub struct ScrolledState<S> {
    pub widget: S,

    pub area: Rect,
    pub view_area: Rect,
    pub hscrollbar_area: Option<Rect>,
    pub vscrollbar_area: Option<Rect>,

    /// mouse action in progress
    pub vdrag: bool,
    pub hdrag: bool,
}

impl<'a, T> Scrolled<'a, T> {
    pub fn new(inner: T) -> Self {
        Self {
            widget: inner,
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

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;

        let mut block = self.block;

        let sconf = self.widget.need_scroll(area);

        if sconf.has_vscroll && sconf.has_hscroll {
            let mut vscrollbar_area = area.columns().last().expect("scroll");
            if block.is_some() {
                vscrollbar_area.y += 1;
                vscrollbar_area.height -= 2;
            } else {
                vscrollbar_area.height -= 1;
            }
            state.vscrollbar_area = Some(vscrollbar_area);

            let mut hscrollbar_area = area.rows().last().expect("scroll");
            if block.is_some() {
                hscrollbar_area.x += 1;
                hscrollbar_area.width -= 2;
            } else {
                hscrollbar_area.width -= 1;
            }
            state.hscrollbar_area = Some(hscrollbar_area);

            if let Some(block) = block.as_ref() {
                state.view_area = block.inner(area);
            } else {
                state.view_area = area;
                state.view_area.width -= 1;
                state.view_area.height -= 1;
            }
        } else if sconf.has_vscroll {
            let mut vscrollbar_area = area.columns().last().expect("scroll");
            if block.is_some() {
                vscrollbar_area.y += 1;
                vscrollbar_area.height -= 1;
            } else {
                vscrollbar_area.height -= 0;
            }
            state.vscrollbar_area = Some(vscrollbar_area);

            state.hscrollbar_area = None;

            if let Some(block) = block.as_ref() {
                state.view_area = block.inner(area);
            } else {
                state.view_area = area;
                state.view_area.width -= 1;
                state.view_area.height -= 0;
            }
        } else if sconf.has_hscroll {
            state.vscrollbar_area = None;

            let mut hscrollbar_area = area.rows().last().expect("scroll");
            if block.is_some() {
                hscrollbar_area.x += 1;
                hscrollbar_area.width -= 1;
            } else {
                hscrollbar_area.width -= 0;
            }
            state.hscrollbar_area = Some(hscrollbar_area);

            if let Some(block) = block.as_ref() {
                state.view_area = block.inner(area);
            } else {
                state.view_area = area;
                state.view_area.width -= 0;
                state.view_area.height -= 1;
            }
        } else {
            state.vscrollbar_area = None;
            state.hscrollbar_area = None;

            if let Some(block) = block.as_ref() {
                state.view_area = block.inner(area);
            } else {
                state.view_area = area;
                state.view_area.width -= 0;
                state.view_area.height -= 0;
            }
        }

        self.widget.render(state.view_area, buf, &mut state.widget);
        block.render(area, buf);

        if let Some(vscrollbar_area) = state.vscrollbar_area {
            let mut vscroll = Scrollbar::new(ScrollbarOrientation::VerticalRight);
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

            let max_offset = state.widget.vmax_offset();
            let offset = state.widget.voffset();

            let mut vscroll_state = ScrollbarState::new(max_offset).position(offset);
            vscroll.render(vscrollbar_area, buf, &mut vscroll_state);
        }

        if let Some(hscrollbar_area) = state.hscrollbar_area {
            let mut hscroll = Scrollbar::new(ScrollbarOrientation::HorizontalBottom);
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

            let max_offset = state.widget.hmax_offset();
            let offset = state.widget.hoffset();

            let mut hscroll_state = ScrollbarState::new(max_offset).position(offset);
            hscroll.render(hscrollbar_area, buf, &mut hscroll_state);
        }
    }
}

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

impl<T> CanValidate for ScrolledState<T>
where
    T: CanValidate,
{
    fn validate(&mut self) {
        self.widget.validate()
    }
}

impl<S, A, E> HandleCrossterm<ControlUI<A, E>, DefaultKeys> for ScrolledState<S>
where
    S: HasScrolling
        + HandleCrossterm<ControlUI<A, E>, MouseOnly>
        + HandleCrossterm<ControlUI<A, E>, DefaultKeys>,
{
    fn handle(&mut self, event: &Event, _: DefaultKeys) -> ControlUI<A, E> {
        let res =
            <Self as HandleCrossterm<ControlUI<A, E>, MouseOnly>>::handle(self, event, MouseOnly);
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
                check_break!(if let Some(vscroll_area) = self.vscrollbar_area {
                    if vscroll_area.contains(Position::new(*column, *row)) {
                        let row = row.saturating_sub(vscroll_area.y) as usize;
                        let height = vscroll_area.height.saturating_sub(1) as usize;

                        let pos = (row * self.widget.vmax_offset()) / height;

                        self.vdrag = true;
                        self.widget.set_voffset(pos);

                        ControlUI::Change
                    } else {
                        ControlUI::Continue
                    }
                } else {
                    ControlUI::Continue
                });
                check_break!(if let Some(hscroll_area) = self.hscrollbar_area {
                    if hscroll_area.contains(Position::new(*column, *row)) {
                        let col = column.saturating_sub(hscroll_area.x) as usize;
                        let width = hscroll_area.width.saturating_sub(1) as usize;

                        let pos = (col * self.widget.hmax_offset()) / width;

                        self.hdrag = true;
                        self.widget.set_hoffset(pos);

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
                check_break!(if self.vdrag {
                    if let Some(vscroll_area) = self.vscrollbar_area {
                        let row = row.saturating_sub(vscroll_area.y) as usize;
                        let height = vscroll_area.height.saturating_sub(1) as usize;

                        let pos = (row * self.widget.vmax_offset()) / height;

                        self.widget.set_voffset(pos);
                        ControlUI::Change
                    } else {
                        ControlUI::Continue
                    }
                } else {
                    ControlUI::Continue
                });

                check_break!(if self.hdrag {
                    if let Some(hscroll_area) = self.hscrollbar_area {
                        let col = column.saturating_sub(hscroll_area.x) as usize;
                        let width = hscroll_area.width.saturating_sub(1) as usize;

                        let pos = (col * self.widget.hmax_offset()) / width;

                        self.widget.set_hoffset(pos);
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
                self.vdrag = false;
                self.hdrag = false;
                ControlUI::Continue
            }

            ct_event!(scroll down for column, row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.widget.scroll_down(self.view_area.height as usize / 10);
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
                if self.area.contains(Position::new(*column, *row)) {
                    self.widget.scroll_right(self.view_area.width as usize / 10);
                    ControlUI::Change
                } else {
                    ControlUI::Continue
                }
            }
            ct_event!(scroll ALT up for column, row) => {
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
