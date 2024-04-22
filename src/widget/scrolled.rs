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
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget};

/// A wrapper widget that scrolls it's content.
#[derive(Debug)]
pub struct Scrolled<T> {
    /// widget
    pub widget: T,
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

impl<T> Scrolled<T> {
    pub fn new(inner: T) -> Self {
        Self { widget: inner }
    }
}

impl<T> StatefulWidget for Scrolled<T>
where
    T: StatefulWidget + ScrolledWidget,
    T::State: HasScrolling,
{
    type State = ScrolledState<T::State>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;

        let sconf = self.widget.need_scroll(area);

        if sconf.has_vscroll && sconf.has_hscroll {
            let mut vscrollbar_area = area.columns().last().expect("scroll");
            vscrollbar_area.height -= 1;
            state.vscrollbar_area = Some(vscrollbar_area);

            let mut hscrollbar_area = area.rows().last().expect("scroll");
            hscrollbar_area.width -= 1;
            state.hscrollbar_area = Some(hscrollbar_area);

            debug!("hscroll {:?}", state.hscrollbar_area);

            state.view_area = area;
            state.view_area.width -= 1;
            state.view_area.height -= 1;
        } else if sconf.has_vscroll {
            state.vscrollbar_area = Some(area.columns().last().expect("scroll"));
            state.hscrollbar_area = None;

            state.view_area = area;
            state.view_area.width -= 1;
        } else if sconf.has_hscroll {
            state.vscrollbar_area = None;
            state.hscrollbar_area = Some(area.rows().last().expect("scroll"));

            state.view_area = area;
            state.view_area.height -= 1;
        } else {
            state.vscrollbar_area = None;
            state.hscrollbar_area = None;

            state.view_area = area;
        }

        self.widget.render(state.view_area, buf, &mut state.widget);

        if let Some(vscrollbar_area) = state.vscrollbar_area {
            let vscroll = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            let mut vscroll_state =
                ScrollbarState::new(state.widget.vmax()).position(state.widget.voffset());
            vscroll.render(vscrollbar_area, buf, &mut vscroll_state);
        }

        if let Some(hscrollbar_area) = state.hscrollbar_area {
            let hscroll = Scrollbar::new(ScrollbarOrientation::HorizontalBottom);
            let mut hscroll_state =
                ScrollbarState::new(state.widget.hmax()).position(state.widget.hoffset());
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

                        let pos = (row * self.widget.vmax()) / height;

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

                        let pos = (col * self.widget.hmax()) / width;

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

                        let pos = (row * self.widget.vmax()) / height;

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

                        let pos = (col * self.widget.hmax()) / width;

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
