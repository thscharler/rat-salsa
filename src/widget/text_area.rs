use crate::_private::NonExhaustive;
use crate::{
    ControlUI, DefaultKeys, FocusFlag, HandleCrossterm, HasFocusFlag, HasScrolling, MouseOnly,
    ScrollParam, ScrolledWidget,
};
use crossterm::event::Event;
#[allow(unused_imports)]
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::Widget;
use std::marker::PhantomData;
use tui_textarea::{CursorMove, TextArea};

#[derive(Debug, Default, Clone)]
pub struct TextAreaExt<'a> {
    _phantom: PhantomData<&'a ()>,
}

#[derive(Debug, Clone)]
pub struct TextAreaExtState<'a> {
    pub area: Rect,
    pub focus: FocusFlag,

    pub max_v_offset: usize,
    pub max_h_offset: usize,
    pub widget: TextArea<'a>,

    pub non_exhaustive: NonExhaustive,
}

impl<'a, State> ScrolledWidget<State> for TextAreaExt<'a> {
    fn need_scroll(&self, _area: Rect, _uistate: &mut State) -> ScrollParam {
        ScrollParam {
            has_hscroll: true,
            has_vscroll: true,
        }
    }
}

impl<'a> StatefulWidget for TextAreaExt<'a> {
    type State = TextAreaExtState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.max_v_offset = state.widget.lines().len();
        state.max_h_offset = state
            .widget
            .lines()
            .iter()
            .map(|v| v.len())
            .max()
            .unwrap_or_default();

        let widget = state.widget.widget();
        widget.render(area, buf);
    }
}

impl<'a> Default for TextAreaExtState<'a> {
    fn default() -> Self {
        Self {
            area: Default::default(),
            focus: Default::default(),
            max_v_offset: 0,
            max_h_offset: 0,
            widget: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a> HasScrolling for TextAreaExtState<'a> {
    fn max_v_offset(&self) -> usize {
        self.max_v_offset
    }

    fn max_h_offset(&self) -> usize {
        self.max_h_offset
    }

    fn v_page_len(&self) -> usize {
        self.area.height as usize
    }

    fn h_page_len(&self) -> usize {
        self.area.width as usize
    }

    fn v_offset(&self) -> usize {
        // this is the closest we can get?
        self.widget.cursor().0
    }

    fn h_offset(&self) -> usize {
        self.widget.cursor().1
    }

    fn set_v_offset(&mut self, offset: usize) {
        self.widget
            .move_cursor(CursorMove::Jump(offset as u16, self.h_offset() as u16));
    }

    fn set_h_offset(&mut self, offset: usize) {
        self.widget
            .move_cursor(CursorMove::Jump(self.v_offset() as u16, offset as u16));
    }
}

impl<'a> HasFocusFlag for TextAreaExtState<'a> {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl<'a, A, E> HandleCrossterm<ControlUI<A, E>, DefaultKeys> for TextAreaExtState<'a> {
    fn handle(&mut self, event: &Event, _keymap: DefaultKeys) -> ControlUI<A, E> {
        if self.is_focused() {
            self.widget.input(event.clone());
            ControlUI::Change
        } else {
            self.handle(event, MouseOnly)
        }
    }
}

impl<'a, A, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for TextAreaExtState<'a> {
    fn handle(&mut self, _event: &Event, _keymap: MouseOnly) -> ControlUI<A, E> {
        // TODO: rect() not implemented in the baseline

        // match event {
        //     ct_event!(mouse up Left for column, row) => {
        //         if self.area.contains(Position::new(*column, *row)) {
        //             let column = *column - self.area.x + self.widget.rect().1;
        //             let row = *row - self.area.y + self.widget.rect().0;
        //
        //             self.widget.move_cursor(CursorMove::Jump(row, column));
        //             ControlUI::Change
        //         } else {
        //             ControlUI::Continue
        //         }
        //     }
        //     _ => ControlUI::Continue,
        // }

        ControlUI::Continue
    }
}
