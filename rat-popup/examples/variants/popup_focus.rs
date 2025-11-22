//! Popup acts as a single widget, and takes part of the focus.

use rat_event::{HandleEvent, Popup, Regular, ct_event};
use rat_focus::{Focus, FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_popup::event::PopupOutcome;
use rat_popup::{PopupConstraint, PopupCore, PopupCoreState};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::Span;
use ratatui::widgets::{Block, StatefulWidget, Widget};
use std::cmp::max;

#[derive(Debug)]
pub struct PopFocusBlue;

impl StatefulWidget for PopFocusBlue {
    type State = PopFocusBlueState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if state.popup.is_active() {
            PopupCore::new()
                .constraint(state.placement)
                .render(area, buf, &mut state.popup);

            let block = Block::bordered().style(Style::new().black().on_yellow());
            let widget_area = block.inner(area);
            block.render(area, buf);

            state.area = state.popup.area;

            buf.set_style(widget_area, Style::new().black().on_yellow());

            let mut a1 = widget_area;
            a1.height = 1;
            Span::from(" p-o-p-u-p ").render(a1, buf);

            let mut a2 = widget_area;
            a2.y += 1;
            a2.height = 1;
            Span::from(state.cc.to_string()).render(a2, buf);
        } else {
            state.popup.clear_areas();
            state.area = Rect::default();
        }
    }
}

#[derive(Debug, Default)]
pub struct PopFocusBlueState {
    pub area: Rect,

    pub cc: char,

    /// Where to place the popup
    pub placement: PopupConstraint,
    pub popup: PopupCoreState,

    pub focus: FocusFlag,
}

impl PopFocusBlueState {
    pub fn new() -> Self {
        Self {
            area: Default::default(),
            cc: ' ',
            placement: Default::default(),
            popup: Default::default(),
            focus: FocusFlag::new().with_name("foc"),
        }
    }

    pub fn is_active(&self) -> bool {
        self.popup.is_active()
    }

    pub fn show(&mut self, placement: PopupConstraint, focus: &Focus) {
        self.placement = placement;
        self.popup.set_active(true);
        focus.focus(&self.focus);
    }

    pub fn hide(&mut self, focus: &Focus) {
        self.popup.set_active(false);
        if self.focus.is_focused() {
            focus.next();
        }
    }
}

impl HasFocus for PopFocusBlueState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget_with_flags(self.focus.clone(), self.area, 0, Navigation::Leave);
    }

    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl HandleEvent<crossterm::event::Event, Regular, PopupOutcome> for PopFocusBlueState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> PopupOutcome {
        if self.focus.gained_focus() {
            self.popup.set_active(true);
        }
        if self.focus.lost_focus() {
            self.popup.set_active(false);
        }

        let r0 = self.popup.handle(event, Popup);

        let r1 = if self.is_active() {
            match event {
                ct_event!(key press c) => {
                    self.cc = *c;
                    PopupOutcome::Changed
                }
                _ => PopupOutcome::Continue,
            }
        } else {
            PopupOutcome::Continue
        };

        max(r0, r1)
    }
}
