///
/// Add scrolling behaviour to a widget.
///
/// Scrolled acts as a wrapper around a widget that implements HasVerticalScroll.
/// No HasHorizontalScroll at the moment, probably never will be and instead
/// a HasScroll covering both.
///
use crate::HasVerticalScroll;
use crate::{
    CanValidate, ControlUI, DefaultKeys, FocusFlag, HandleCrossterm, HasFocusFlag, HasValidFlag,
    MouseOnly, ValidFlag,
};
use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
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
    /// widget state
    pub widget: S,
    /// area
    pub area: Rect,
    /// scrollbar area
    pub scrollbar_area: Option<Rect>,
    /// mouse action in progress
    pub mouse: bool,
}

impl<T> Scrolled<T> {
    pub fn new(inner: T) -> Self {
        Self { widget: inner }
    }
}

impl<T> StatefulWidget for Scrolled<T>
where
    T: StatefulWidget,
    T::State: HasVerticalScroll,
{
    type State = ScrolledState<T::State>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;

        if state.widget.need_vscroll() {
            state.scrollbar_area = area.columns().last();

            let mut widget_area = area.clone();
            widget_area.width -= 1;
            self.widget.render(widget_area, buf, &mut state.widget);

            let scroll = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            let mut scroll_state =
                ScrollbarState::new(state.widget.vlen()).position(state.widget.voffset());
            scroll.render(area, buf, &mut scroll_state);
        } else {
            state.scrollbar_area = None;
            self.widget.render(area, buf, &mut state.widget);
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
    S: HasVerticalScroll
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
    S: HasVerticalScroll + HandleCrossterm<ControlUI<A, E>, MouseOnly>,
{
    fn handle(&mut self, event: &Event, _: MouseOnly) -> ControlUI<A, E> {
        let res = match event {
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => {
                if let Some(scroll_area) = self.scrollbar_area {
                    if scroll_area.contains(Position::new(*column, *row)) {
                        let row = row.saturating_sub(scroll_area.y) as usize;
                        let height = scroll_area.height.saturating_sub(1) as usize;

                        let pos = (row * self.widget.vlen()) / height;

                        self.mouse = true;
                        self.widget.set_voffset(pos);

                        ControlUI::Change
                    } else {
                        ControlUI::Continue
                    }
                } else {
                    ControlUI::Continue
                }
            }
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left),
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                self.mouse = false;
                ControlUI::Continue
            }

            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Drag(MouseButton::Left),
                row,
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                if self.mouse {
                    if let Some(scroll_area) = self.scrollbar_area {
                        let row = row.saturating_sub(scroll_area.y) as usize;
                        let height = scroll_area.height.saturating_sub(1) as usize;

                        let pos = (row * self.widget.vlen()) / height;

                        self.widget.set_voffset(pos);
                        ControlUI::Change
                    } else {
                        ControlUI::Continue
                    }
                } else {
                    ControlUI::Continue
                }
            }

            Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollDown,
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.widget.vscroll_down(self.widget.vpage() / 5);
                    ControlUI::Change
                } else {
                    ControlUI::Continue
                }
            }
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollUp,
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.widget.vscroll_up(self.widget.vpage() / 5);
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
